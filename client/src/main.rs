use clap::Clap;

use futures::{SinkExt, StreamExt};
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

#[derive(Debug, Clap)]
struct Opts {
    server: String,
    tables: Vec<usize>,
    #[clap(short, long)]
    rounds: Option<usize>,
    #[clap(short, long, default_value = "10000")]
    iterations_per_sync: usize,
}

#[derive(Debug)]
struct Stats {
    steps: usize,
}

fn solving_thread(
    tables: Vec<usize>,
    rounds: usize,
    steps_per_sync: usize,
    in_queue: std::sync::mpsc::Receiver<Vec<usize>>,
    in_queue_size: Arc<AtomicUsize>,
    sender: std::sync::mpsc::Sender<(Vec<usize>, Vec<Vec<usize>>, Stats)>,
) {
    let mut buffer = Vec::new();
    let scheduler = schedule_solver::Scheduler::new(&tables, rounds);
	let scheduler = schedule_solver::Scheduler::new(&[4; 6], 6);
    'queue: while let Ok(next) = in_queue.recv() {
        in_queue_size.fetch_sub(1, Ordering::Relaxed);
        buffer.extend(&next);
        let mut current_depth = 0;
        let mut steps = 0;
        'inner_loop: while steps < steps_per_sync {
            let target_size = (current_depth + 2) * scheduler.get_block_size();
            if target_size > buffer.len() {
                buffer.resize(target_size, 0);
            }
            let buffer: &mut [usize] = &mut buffer[current_depth * scheduler.get_block_size()..];
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
                    assert!(current_depth <= scheduler.get_players_placed(buf_2));
                    current_depth += 1;
                }
            } else {
                if current_depth == 0 {
                    break 'inner_loop;
                } else {
                    current_depth -= 1;
                }
            }

            steps += 1;
        }
        let mut output = Vec::with_capacity(current_depth);
        for i in 0..=current_depth {
            output.push(
                buffer[i * scheduler.get_block_size()..(i + 1) * scheduler.get_block_size()]
                    .to_vec(),
            );
        }
        let stats = Stats { steps };
        if sender.send((next, output, stats)).is_err() {
            return;
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
    let (tx, rx) = std::sync::mpsc::channel();
    for i in 0..4 {
        let tables = tables.clone();
        let tx = tx.clone();
        let (local_tx, local_rx) = std::sync::mpsc::channel();
        let queue_size_base = Arc::new(AtomicUsize::new(0));
        let queue_size = queue_size_base.clone();
        let thread = std::thread::spawn(move || {
            solving_thread(tables, rounds, 1_000_000, local_rx, queue_size, tx)
        });
        threads.push((queue_size_base, local_tx));
    }

    let scheduler = schedule_solver::Scheduler::new(&tables, rounds);
    let mut init = vec![0; scheduler.get_block_size()];
    scheduler.initialise_buffer(&mut init);
    threads[0].0.fetch_add(1, Ordering::Relaxed);
    threads[0].1.send(init.clone());
    let start = std::time::Instant::now();
    let mut i: usize = 0;
    let mut total_steps = 0;
    let mut last_print = std::time::Instant::now();
    while let Ok((_, output, stats)) = rx.recv() {
        for block in output.into_iter() {
            if i >= threads.len() {
                i = 0;
            }
            threads[i].0.fetch_add(1, Ordering::Relaxed);
            threads[i].1.send(block);
            i += 1;
        }
        total_steps += stats.steps;
        if last_print.elapsed().as_millis() > 300 {
            println!(
                "Total Steps: {} Rate: {}",
                total_steps,
                total_steps as f32 / start.elapsed().as_secs_f32()
            );
            last_print = std::time::Instant::now();
        }
    }

    println!("{:?}", opts);

    let encoded = serde_json::to_string(&(tables, rounds))?;

    loop {
        let (mut ws_stream, _response) = tokio_tungstenite::connect_async(&opts.server).await?;

        ws_stream
            .send(tokio_tungstenite::tungstenite::protocol::Message::Text(
                encoded.clone(),
            ))
            .await;
    }

    Ok(())
}
