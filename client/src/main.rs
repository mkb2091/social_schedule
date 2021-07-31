use clap::Clap;

use futures::pin_mut;
use futures::stream::{SplitSink, SplitStream};
use futures::FutureExt;
use futures::{SinkExt, StreamExt};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use schedule_util::{Batch, BatchOutput};

use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Debug, Clap)]
struct Opts {
    server: String,
    tables: Vec<usize>,
    #[clap(short, long)]
    rounds: Option<usize>,
    #[clap(short, long, default_value = "10000")]
    iterations_per_sync: usize,
}

fn solving_thread(
    tables: Vec<usize>,
    rounds: usize,
    steps_per_sync: usize,
    in_queue: std::sync::mpsc::Receiver<Batch>,
    in_queue_size: Arc<AtomicUsize>,
    sender: tokio::sync::mpsc::UnboundedSender<BatchOutput>,
) {
    let mut buffer = Vec::new();
    let scheduler = schedule_solver::Scheduler::new(&tables, rounds);
    while let Ok(next) = in_queue.recv() {
        let (id, data) = next.split();
        let data = data.get_ref();
        in_queue_size.fetch_sub(1, Ordering::Relaxed);
        if buffer.len() < data.len() {
            buffer.resize(data.len(), 0);
        }
        buffer[..data.len()].copy_from_slice(&data);
        let mut current_depth = 0;
        let mut steps = 0;
        let start = std::time::Instant::now();
        let mut emptied = false;
        'inner_loop: while steps <= steps_per_sync {
            let target_size = (current_depth + 2) * scheduler.get_block_size();
            if target_size > buffer.len() {
                buffer.resize(target_size, 0);
            }
            let buffer: &mut [u64] = &mut buffer[current_depth * scheduler.get_block_size()..];
            let (buf_1, buf_2) = buffer.split_at_mut(scheduler.get_block_size());
            if let Some(finished) = scheduler.step(buf_1, buf_2) {
                if finished {
                    println!("Found a solution: {:?}", buf_1);
                    if current_depth == 0 {
                        emptied = true;
                        break 'inner_loop;
                    } else {
                        current_depth -= 1;
                    }
                } else {
                    assert!(current_depth <= scheduler.get_players_placed(buf_2) as usize);
                    current_depth += 1;
                }
            } else {
                if current_depth == 0 {
                    emptied = true;
                    break 'inner_loop;
                } else {
                    current_depth -= 1;
                }
            }

            steps += 1;
        }
        let mut output = Vec::with_capacity(current_depth);
        if !emptied {
            for i in 0..=current_depth {
                output.push(
                    buffer[i * scheduler.get_block_size()..(i + 1) * scheduler.get_block_size()]
                        .to_vec(),
                );
            }
            assert_eq!(output.len(), current_depth + 1);
        }
        let stats = schedule_util::Stats {
            steps,
            elapsed: start.elapsed(),
        };
        let batch_result = schedule_util::BatchOutput {
            base: id,
            children: output,
            notable: Vec::new(),
            stats,
        };
        if let Err(error) = sender.send(batch_result) {
            println!("Error: {:?}", error);
            return;
        }
    }
    println!("Thread exiting");
}

type WebSocketStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn handle_send(
    mut rx: tokio::sync::mpsc::UnboundedReceiver<BatchOutput>,
    mut ws_tx: SplitSink<WebSocketStream, Message>,
    threads: Vec<(Arc<AtomicUsize>, std::sync::mpsc::Sender<Batch>)>,
) -> Result<std::convert::Infallible, Box<dyn std::error::Error + Send + Sync>> {
    let start = std::time::Instant::now();
    let mut total_steps: usize = 0;
    let mut last_print = std::time::Instant::now();
    loop {
        let batch_result = rx.recv().await.ok_or(std::sync::mpsc::RecvError)?;
        let encoded = bincode::serialize(&batch_result).unwrap();
        ws_tx.send(Message::Binary(encoded)).await?;
        total_steps += batch_result.stats.steps;
        if last_print.elapsed().as_millis() > 300 {
            println!(
                "Total Steps: {} Rate: {}",
                total_steps,
                total_steps as f32 / start.elapsed().as_secs_f32()
            );
            for (queue_size, _) in threads.iter() {
                println!("Queue size: {}", queue_size.load(Ordering::Relaxed));
            }
            last_print = std::time::Instant::now();
        }
    }
}

async fn handle_recv(
    mut ws_rx: SplitStream<WebSocketStream>,
    threads: Vec<(Arc<AtomicUsize>, std::sync::mpsc::Sender<Batch>)>,
) -> Result<std::convert::Infallible, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let next = ws_rx.next().await.ok_or(std::sync::mpsc::RecvError)?;
        let next = next?.into_data();
        if let Ok(decoded) = bincode::deserialize::<Batch>(&next) {
            let (queue_size, queue) = threads
                .iter()
                .min_by_key(|(queue_size, _queue)| queue_size.load(Ordering::Relaxed))
                .unwrap();
            queue_size.fetch_add(1, Ordering::Relaxed);
            queue.send(decoded)?;
        } else {
            println!("Failed to decode: {:?}", &next);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();
    let rounds = if let Some(rounds) = opts.rounds {
        if rounds > opts.tables.len() {
            println!("Rounds greater than tables");
            opts.tables.len()
        } else {
            rounds
        }
    } else {
        opts.tables.len()
    };
    let mut tables = opts.tables.clone();
    tables.sort_unstable();

    let mut threads = Vec::new();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    for _ in 0..num_cpus::get() {
        let tables = tables.clone();
        let tx = tx.clone();
        let (local_tx, local_rx) = std::sync::mpsc::channel();
        let queue_size_base = Arc::new(AtomicUsize::new(0));
        let queue_size = queue_size_base.clone();
        let iterations_per_sync = opts.iterations_per_sync;
        let _thread = std::thread::spawn(move || {
            solving_thread(
                tables,
                rounds,
                iterations_per_sync,
                local_rx,
                queue_size,
                tx,
            )
        });
        threads.push((queue_size_base, local_tx));
    }
    let arg = schedule_util::ScheduleArg::new(&tables, rounds);
    let (mut ws_stream, _response) = tokio_tungstenite::connect_async(&opts.server).await?;

    let encoded = bincode::serialize(&arg)?;
    ws_stream
        .send(tokio_tungstenite::tungstenite::protocol::Message::Binary(
            encoded.clone(),
        ))
        .await?;

    let (ws_tx, ws_rx) = ws_stream.split();
    let handle_batches = tokio::spawn(handle_send(rx, ws_tx, threads.clone()));
    let handle_blocks = tokio::spawn(handle_recv(ws_rx, threads.clone()));
    pin_mut!(handle_batches);
    pin_mut!(handle_blocks);
    let result = futures::future::select(&mut handle_batches, &mut handle_blocks)
        .map(|either| either.factor_first().0)
        .map(|result| {
            let result = result.map_err(|join_err| {
                let err: Box<dyn std::error::Error + Send + Sync> = Box::new(join_err);
                err
            });
            result.unwrap_or_else(|err| Err(err))
        })
        .await;
    handle_batches.abort();
    handle_blocks.abort();
    println!("Error {:?}", result);
    Ok(())
}
