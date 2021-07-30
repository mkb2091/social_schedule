use clap::Clap;

use futures::{SinkExt, StreamExt};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

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
    in_queue: std::sync::mpsc::Receiver<Vec<u64>>,
    in_queue_size: Arc<AtomicUsize>,
    sender: tokio::sync::mpsc::UnboundedSender<schedule_util::BatchOutput>,
) {
    let mut buffer = Vec::new();
    let scheduler = schedule_solver::Scheduler::new(&tables, rounds);
    while let Ok(next) = in_queue.recv() {
        in_queue_size.fetch_sub(1, Ordering::Relaxed);
        if buffer.len() < next.len() {
            buffer.resize(next.len(), 0);
        }
        buffer[..next.len()].copy_from_slice(&next);
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
                    assert_eq!(scheduler.get_players_placed(buf_1), 4 * 6 * 6);
                    assert_eq!(scheduler.get_empty_table_count(buf_1), 0);
                    println!("Found a solution: {:?}", buf_1);
                    //current_depth -= 1;
                    return;
                } else {
                    assert_ne!(buf_1, buf_2);
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
            base: next,
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
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
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
    let start = std::time::Instant::now();
    let mut total_steps: usize = 0;
    let mut last_print = std::time::Instant::now();
    let (mut ws_stream, _response) = tokio_tungstenite::connect_async(&opts.server).await?;

    let encoded = bincode::serialize(&arg)?;
    ws_stream
        .send(tokio_tungstenite::tungstenite::protocol::Message::Binary(
            encoded.clone(),
        ))
        .await?;

    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let handle_batches = async {
        while let Some(batch_result) = rx.recv().await {
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
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    let handle_blocks = async {
        while let Some(next) = ws_rx.next().await {
            let next = next?.into_data();
            if let Ok(decoded) = bincode::deserialize(&next) {
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
        println!("Disconnected");
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    futures::future::try_join(handle_batches, handle_blocks).await?;
    Ok(())
}
