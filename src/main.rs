#[cfg(feature = "cli")]
extern crate clap;
#[cfg(feature = "cli")]
extern crate dirs;
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

use std::io::prelude::*;

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
fn display_schedule<T: schedule::ScheduleStructure>(output: &mut String, schedule: &T) {
    let schedule = schedule.to_schedule();
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
        for _ in 0..schedule.get_tables() {
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
}

#[cfg(feature = "cli")]
fn display_performance(output: &mut String, operations: u64, random_starts: u64, nanos: u128) {
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
}

#[cfg(feature = "cli")]
fn main() {
    let opts: Opts = Opts::parse();
    let path = match dirs::data_local_dir() {
        Some(path) => [path, std::path::PathBuf::from("social_schedule")]
            .iter()
            .collect(),
        None => std::path::PathBuf::from("."),
    };
    let ideal_path_base: std::path::PathBuf =
        [path, std::path::PathBuf::from("ideal")].iter().collect();
    let _ = std::fs::create_dir_all(&ideal_path_base);
    let ideal_path: std::path::PathBuf = [
        ideal_path_base,
        std::path::PathBuf::from(format!("{}_players_{}_tables", opts.players, opts.tables)),
    ]
    .iter()
    .collect();
    {
        let search_paths = [
            &ideal_path,
            &std::path::PathBuf::from(format!(
                "cache/ideal/{}_players_{}_tables",
                opts.players, opts.tables
            )),
        ];
        for path in search_paths.iter() {
            println!("Attempting to loading cache from {}", path.display());
            if let Ok(mut file) = std::fs::File::open(&path) {
                let mut contents = String::new();
                let _ = file.read_to_string(&mut contents);
                if let Ok(schedule) = serde_json::from_str::<schedule::SerdeSchedule>(&contents) {
                    let mut output = String::new();
                    display_schedule(&mut output, &schedule);
                    println!("Found ideal from cache: \n{}", output);
                    return;
                }
            }
        }
    }
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
        let mut best_score = best_schedule.lock().unwrap().get_score();
        std::thread::spawn(move || loop {
            let schedule = rx.recv().unwrap();
            if schedule.get_score() > best_score {
                *best_schedule.lock().unwrap() = schedule;
                best_score = best_schedule.lock().unwrap().get_score();
            }
        });
    }
    let instant = std::time::Instant::now();
    loop {
        if let Ok(schedule) = best_schedule.lock() {
            if schedule.is_ideal() {
                println!("\n\nFound ideal schedule\n");
            }
            let mut output = String::new();
            display_performance(
                &mut output,
                operations.load(std::sync::atomic::Ordering::Relaxed),
                random_starts.load(std::sync::atomic::Ordering::Relaxed),
                instant.elapsed().as_nanos(),
            );
            display_schedule(&mut output, &*schedule);
            println!("{}", output);
            if schedule.is_ideal() {
                let serde_schedule = schedule.to_serde_schedule();

                if let Ok(string_form) = serde_json::to_string(&serde_schedule) {
                    if let Ok(mut file) = std::fs::File::create(ideal_path) {
                        file.write_all(string_form.as_bytes()).unwrap();
                    }
                }
                break;
            }
        } else {
            panic!("Failed");
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

#[cfg(not(feature = "cli"))]
fn main() {}
