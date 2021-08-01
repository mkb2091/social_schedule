use clap::Clap;

use futures::pin_mut;
use futures::stream::{SplitSink, SplitStream};
use futures::FutureExt;
use futures::{SinkExt, StreamExt};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use schedule_util::{BatchDeserialize, BatchOutputSerialize};

use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Debug, Clap)]
struct Opts {
    server: String,
    tables: Vec<usize>,
    #[clap(short, long)]
    rounds: Option<usize>,
    #[clap(short, long, default_value = "10000")]
    iterations_per_sync: u64,
    #[clap(short, long)]
    jobs: Option<std::num::NonZeroUsize>,
}

fn solving_thread(
    tables: Vec<usize>,
    rounds: usize,
    steps_per_sync: u64,
    in_queue: std::sync::mpsc::Receiver<Vec<u8>>,
    in_queue_size: Arc<AtomicUsize>,
    sender: tokio::sync::mpsc::UnboundedSender<(Vec<u8>, schedule_util::Stats)>,
) {
    let scheduler = schedule_solver::Scheduler::new(&tables, rounds);
    let mut buffer = vec![0; scheduler.get_block_size()];
    while let Ok(next) = in_queue.recv() {
        in_queue_size.fetch_sub(1, Ordering::Relaxed);
        let deserialized = if let Ok(de) = BatchDeserialize::deserialize(&next) {
            de
        } else {
            continue;
        };
        let id = deserialized.get_id();
        assert_eq!(deserialized.get_length(), scheduler.get_block_size());
        for (ptr, val) in buffer.iter_mut().zip(deserialized.get_data()) {
            *ptr = val;
        }
        let mut current_depth = 0;
        let mut steps: u64 = 0;
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
        let output = if !emptied {
            &buffer[..(current_depth + 1) * scheduler.get_block_size()]
        } else {
            &[]
        };
        let stats = schedule_util::Stats {
            steps,
            elapsed: start.elapsed(),
        };
        let batch_result =
            BatchOutputSerialize::new(id, scheduler.get_block_size(), output, &[], stats);
        let mut buf = vec![0; batch_result.get_size()];
        batch_result.serialize(&mut buf).unwrap();
        if let Err(error) = sender.send((buf, stats)) {
            println!("Error: {:?}", error);
            return;
        }
    }
    println!("Thread exiting");
}

type WebSocketStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn handle_send(
    total_steps: Arc<AtomicUsize>,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<(Vec<u8>, schedule_util::Stats)>,
    mut ws_tx: SplitSink<WebSocketStream, Message>,
) -> Result<std::convert::Infallible, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let (batch_result, stats) = rx.recv().await.ok_or(std::sync::mpsc::RecvError)?;
        ws_tx.send(Message::Binary(batch_result)).await?;
        total_steps.fetch_add(stats.steps as usize, Ordering::Relaxed);
    }
}

async fn handle_recv(
    mut ws_rx: SplitStream<WebSocketStream>,
    threads: Vec<(Arc<AtomicUsize>, std::sync::mpsc::Sender<Vec<u8>>)>,
) -> Result<std::convert::Infallible, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let next = ws_rx.next().await.ok_or(std::sync::mpsc::RecvError)?;
        let next = next?.into_data();
        let (queue_size, queue) = threads
            .iter()
            .min_by_key(|(queue_size, _queue)| queue_size.load(Ordering::Relaxed))
            .unwrap();
        queue_size.fetch_add(1, Ordering::Relaxed);
        queue.send(next)?;
    }
}

async fn handle_display(
    total_steps: Arc<AtomicUsize>,
    threads: Vec<(Arc<AtomicUsize>, std::sync::mpsc::Sender<Vec<u8>>)>,
) {
    let start = std::time::Instant::now();
    loop {
        let total_steps = total_steps.load(Ordering::Relaxed);
        println!(
            "Total Steps: {} Rate: {}",
            total_steps,
            total_steps as f32 / start.elapsed().as_secs_f32()
        );
        for (queue_size, _) in threads.iter() {
            println!("Queue size: {}", queue_size.load(Ordering::Relaxed));
        }
		tokio::time::sleep(std::time::Duration::from_millis(300)).await;
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
    for _ in 0..opts
        .jobs
        .map(|jobs| jobs.get())
        .unwrap_or_else(|| num_cpus::get())
    {
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
	let total_steps = Arc::new(AtomicUsize::new(0));
    let handle_batches = tokio::spawn(handle_send(total_steps.clone(), rx, ws_tx));
    let handle_blocks = tokio::spawn(handle_recv(ws_rx, threads.clone()));
	let handle_display = tokio::spawn(handle_display(total_steps.clone(), threads.clone()));
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
