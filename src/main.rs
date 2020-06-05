#[cfg(feature = "cli")]
extern crate clap;
#[cfg(feature = "cli")]
extern crate num_cpus;
extern crate num_format;
extern crate rand;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(feature = "cli")]
use clap::Clap;
use num_format::{Locale, WriteFormatted};
use rand::SeedableRng;


#[cfg(feature = "cli")]
pub mod schedule;

const PROCESS_LOOP_COUNT: usize = 1000;

#[cfg(feature = "cli")]
#[derive(Clap)]
#[clap(version = "1.0")]
struct Opts {
    players: usize,
    tables: usize,
}


#[cfg(feature = "cli")]
fn display_schedule(
    schedule: &schedule::Schedule,
    operations: u64,
    random_starts: u64,
    nanos: u128,
) {
    let mut output = String::new();
    output.push_str("Testing ");
    let per_second = (10_f64.powi(9) * operations as f64 / nanos as f64) as u64;
    output.write_formatted(&per_second, &Locale::en).unwrap();
    output.push_str(" schedules per second\n");
    output.push_str("Total schedules tested:");
    output.write_formatted(&operations, &Locale::en).unwrap();
    output.push('\n');
    output.push_str("Using ");
    let per_second = (random_starts as f64 / (nanos as f64 / 10_f64.powi(9))) as u64;
    output.write_formatted(&per_second, &Locale::en).unwrap();
    output.push_str(" random starts per second\n");
    output.push_str("Total random starts:");
    output.write_formatted(&random_starts, &Locale::en).unwrap();
    output.push('\n');
    output.push_str(&format!(
        "Average number of unique games played: {}\n",
        schedule.unique_games_played() as f32 / schedule.get_player_count() as f32
    ));
    output.push_str(&format!(
        "Average number of unique opponents/teammates played with: {}\n",
        (schedule.unique_opponents() as f32 / schedule.get_player_count() as f32)
    ));
    output.push_str(&format!(
        "Minimum number of unique opponents/teammates played with: {}\n",
        schedule.min_unique_opponents()
    ));
    output.push_str("     ");
    for table in 0..schedule.get_tables() {
        let now = (table + 1).to_string();
        output.push('|');
        for _ in 0..(3 - now.len()) {
            output.push(' ');
        }
        output.push_str(&now);
        output.push_str("  ");
    }

    for round in 0..schedule.get_tables() {
        output.push_str("\n-----");
        for table in 0..schedule.get_tables() {
            output.push('+');
            output.push_str("-----")
        }
        for i in 0..(schedule.get_player_count() / schedule.get_tables() + 1) {
            if i == (schedule.get_player_count() / schedule.get_tables() + 1) / 2 {
                output.push('\n');
                let now = (round + 1).to_string();
                for _ in 0..(3 - now.len()) {
                    output.push(' ');
                }
                output.push_str(&now);
                output.push_str("  ");
            } else {
                output.push_str("\n     ");
            }
            for table in 0..schedule.get_tables() {
                let player_list = schedule.get_players_from_game(round, table);
                output.push('|');
                if let Some(now) = player_list.get(i) {
                    let now = now.to_string();
                    for _ in 0..(3 - now.len()) {
                        output.push(' ');
                    }
                    output.push_str(&now);
                    output.push_str("  ");
                } else {
                    output.push_str("     ");
                }
            }
        }
    }
    println!("{}", output);
}

#[cfg(feature = "cli")]
fn main() {
    let opts: Opts = Opts::parse();
    let (tx, rx) = std::sync::mpsc::channel::<schedule::Schedule>();
    let operations = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let random_starts = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    println!("Spawning {} threads", num_cpus::get());
    for _ in 0..num_cpus::get() {
        let tx = tx.clone();
        let operations = std::sync::Arc::clone(&operations);
        let random_starts = std::sync::Arc::clone(&random_starts);
        let players = opts.players;
        let tables = opts.tables;
        std::thread::spawn(move || {
            let mut schedule_generator =
                schedule::Generator::new(rand::thread_rng(), players, tables);

            tx.send(schedule_generator.best.clone()).unwrap();
            loop {
                let old_score = schedule_generator.best.get_score();
                let mut local_operations: u64 = 0;
                let mut local_random_starts: u64 = 0;
                for _ in 0..PROCESS_LOOP_COUNT {
                    let (ops, rs) = schedule_generator.process();
                    local_operations += ops as u64;
                    local_random_starts += rs as u64;
                }

                operations.fetch_add(local_operations, std::sync::atomic::Ordering::SeqCst);
                random_starts.fetch_add(local_random_starts, std::sync::atomic::Ordering::SeqCst);

                if schedule_generator.best.get_score() > old_score {
                    tx.send(schedule_generator.best.clone()).unwrap();
                }
            }
        });
    }
    let best_schedule = std::sync::Arc::new(std::sync::Mutex::new(rx.recv().unwrap()));
    {
        let best_schedule = std::sync::Arc::clone(&best_schedule);
        std::thread::spawn(move || loop {
            let mut best_score = best_schedule.lock().unwrap().get_score();
            let schedule = rx.recv().unwrap();
            if schedule.get_score() > best_score {
                *best_schedule.lock().unwrap() = schedule;
                best_score = best_schedule.lock().unwrap().get_score();
            }
        });
    }
    let instant = std::time::Instant::now();
    loop {
        display_schedule(
            &*best_schedule.lock().unwrap(),
            operations.load(std::sync::atomic::Ordering::Relaxed),
            random_starts.load(std::sync::atomic::Ordering::Relaxed),
            instant.elapsed().as_nanos(),
        );
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

#[cfg(not(feature = "cli"))]
fn main() {}
