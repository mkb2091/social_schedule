use std::collections::HashSet;
extern crate rand;
use rand::{thread_rng, Rng};

fn get_score(event: Vec<Vec<Vec<u8>>>) -> f32 {
    let mut game: Vec<HashSet<u8>> = Vec::new();
    let mut opponents: Vec<HashSet<u8>> = Vec::new();
    for _ in 0..24 {
        game.push(HashSet::new());
        opponents.push(HashSet::new())
    }
    for r in event {
        let mut i: u8 = 0;
        for table in r {
            i += 1;
            let table2: Vec<u8> = table.clone();
            for player in table {
                game[player as usize].insert(i);
                for p in table2.iter() {
                    opponents[player as usize].insert(*p);

                }
            }
        }
    }
    let mut score: f32 = 0.0;
    for i in game {
        score += i.len() as f32
    }
    let mut min: f32 = 18.0;
    let mut total: f32 = 0.0;
    for i in opponents.iter() {
        let now: f32 = i.len() as f32;
        if now < min {
            min = now;
        }
        total += now;
    }
    score += min;
    score += total / opponents.len() as f32;
    score
}

fn gen_layout() -> Vec<Vec<Vec<u8>>> {
    let mut rng = thread_rng();
    let mut event: Vec<Vec<Vec<u8>>> = Vec::new();
    let mut options = vec![
        0,
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        9,
        10,
        11,
        12,
        13,
        14,
        15,
        16,
        17,
        18,
        19,
        20,
        21,
        22,
        23,
    ];
    for _ in 0..6 {
        let mut round: Vec<Vec<u8>> = Vec::new();
        rng.shuffle(&mut options);
        for t in 0..6 {
            let mut table: Vec<u8> = Vec::new();
            for p in 0..4 {
                table.push(options[4 * t + p])
            }
            round.push(table);
        }
        event.push(round);
    }
    event
}

fn mutate(mut event: Vec<Vec<Vec<u8>>>) -> Vec<Vec<Vec<u8>>> {
    let mut rng = thread_rng();
    let r: usize = rng.gen_range(0, 6);
    let t1: usize = rng.gen_range(0, 6);
    let mut t2: usize = rng.gen_range(0, 6);
    while t1 == t2 {
        t2 = rng.gen_range(0, 6);
    }
    let p1: usize = rng.gen_range(0, 4);
    let p2: usize = rng.gen_range(0, 4);
    let person1: u8 = event[r][t1][p1];
    event[r][t1][p1] = event[r][t1][p2];
    event[r][t2][p2] = person1;
    event.to_vec()
}

fn main() {
    println!("Welcome to Social Scheduler");
    let mut event: Vec<Vec<Vec<u8>>> = gen_layout();
    let mut cevent: Vec<Vec<Vec<u8>>>;
    let mut score: f32 = get_score(event.clone());
    let mut max: f32 = score;
    let mut iterations: u32 = 0;
    let mut calculations: u32 = 0;
    loop {
        iterations += 1;
        let mut changed: bool = false;
        let mut new: Vec<Vec<Vec<u8>>> = Vec::new();
        for r in 0..6 {
            for t1 in 0..6 {
                for t2 in 0..6 {
                    if t1 != t2 {
                        for p1 in 0..4 {
                            for p2 in 0..4 {
                                cevent = event.clone();
                                let person1: u8 = cevent[r][t1][p1];
                                cevent[r][t1][p1] = cevent[r][t1][p2];
                                cevent[r][t2][p2] = person1;
                                let cscore: f32 = get_score(cevent.clone());
                                if cscore > score {
                                    score = cscore;
                                    new = cevent;
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        calculations += 6 * 6 * 5 * 4 * 4;
        if changed {
            event = new;
        } else {
            if score > max {
                println!(
                    "Found local max: {:?} after {:?} iterations ({:?} attempts)",
                    score,
                    iterations,
                    calculations
                );
                max = score;
                println!("{:?}", event);
            }
            iterations = 0;
            calculations = 0;
            event = gen_layout();
            score = get_score(event.clone());
        }
    }
}
