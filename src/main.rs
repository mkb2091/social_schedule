use std::collections::HashSet;
extern crate rand;
use rand::{thread_rng, Rng};

fn get_score(event: [u8; 144]) -> f32 {
    let mut game: Vec<HashSet<u8>> = Vec::with_capacity(24);
    let mut opponents: Vec<HashSet<u8>> = Vec::with_capacity(24);
    for _ in 0..24 {
        game.push(HashSet::with_capacity(6));
        opponents.push(HashSet::with_capacity(19));
    }
    for r in 0..6 {
        let r: usize = r * 24;
        let mut i: u8 = 0;
        for t in 0..6 {
            let t: usize = t * 4;
            i += 1;
            for player in 0..4 {
                let player: usize = event[r + t + player] as usize;
                game[player].insert(i);
                for p in 0..4 {
                    let p: u8 = event[r + t + p];
                    opponents[player].insert(p);

                }
            }
        }
    }
    let mut score: f32 = 0.0;
    for i in game {
        score += i.len() as f32;
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

fn gen_layout() -> [u8; 144] {
    let mut rng = thread_rng();
    let mut event: [u8; 144] = [0; 144];
    for i in 0..24 {
        event[i as usize] = i;
    }
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
    for r in 1..6 {
        rng.shuffle(&mut options);
        for pos in 0..24 {
            event[r * 24 + pos as usize] = options[pos as usize];
        }
    }
    event
}

fn main() {
    println!("Welcome to Social Scheduler");
    let mut event: [u8; 144] = gen_layout();
    let mut cevent: [u8; 144];
    let mut score: f32 = get_score(event.clone());
    let mut max: f32 = score;
    let mut iterations: u32 = 0;
    loop {
        iterations += 1;
        let mut changed: bool = false;
        let mut new: [u8; 144] = [0; 144];
        for r in 0..6 {
            let r: usize = r * 24;
            for t1 in 0..6 {
                let t1: usize = t1 * 4;
                for t2 in 0..6 {
                    let t2: usize = t2 * 4;
                    if t1 != t2 {
                        for p1 in 0..4 {
                            for p2 in 0..4 {
                                cevent = event.clone();
                                let person1: u8 = cevent[r + t1 + p1];
                                cevent[r + t1 + p1] = cevent[r + t2 + p2];
                                cevent[r + t2 + p2] = person1;
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
        if changed {
            event = new;
        } else {
            if score > max {
                println!(
                    "Found local max: {:?} after {:?} iterations ({:?} attempts)",
                    score,
                    iterations,
                    6 * 6 * 5 * 4 * 4 * iterations,
                );
                max = score;
                let mut json_event: Vec<Vec<Vec<u8>>> = Vec::with_capacity(6);
                for r in 0..6 {
                    let mut round: Vec<Vec<u8>> = Vec::with_capacity(6);
                    for t in 0..6 {
                        let mut table: Vec<u8> = Vec::with_capacity(4);
                        for p in 0..4 {
                            table.push(event[r * 24 + t * 4 + p]);
                        }
                        round.push(table);
                    }
                    json_event.push(round);
                }
                println!("{:?}", json_event);
            }
            iterations = 0;
            event = gen_layout();
            score = get_score(event.clone());
        }
    }
}
