use std::collections::HashSet;
use std::cmp::Ordering;
extern crate rand;
use rand::{thread_rng, Rng};

fn get_score(event: Vec<Vec<Vec<i32>>>) -> i32 {
    let mut game: Vec<HashSet<i32>> = Vec::new();
    let mut opponents: Vec<HashSet<i32>> = Vec::new();
    for _ in 0..24 {
        game.push(HashSet::new());
        opponents.push(HashSet::new())
    }
    for r in event {
        let mut i: i32 = 0;
        for table in r {
            i += 1;
            let table2: Vec<i32> = table.clone();
            for player in table {
                game[player as usize].insert(i);
                for p in table2.iter() {
                    opponents[player as usize].insert(*p);
                }
            }
        }
    }
    let mut score: i32 = 0;
    for i in game {
        score += i.len() as i32
    }
    let mut min: i32 = 18;
    //let mut total: i32 = 0;
    for i in opponents {
        let now: i32 = i.len() as i32;
        if now < min {
            min = now;
        }
        //total += i;
    }
    score += min;
    //score += total / opponents.len();
    score
}

fn gen_layout() -> Vec<Vec<Vec<i32>>> {
    let mut event: Vec<Vec<Vec<i32>>> = Vec::new();
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
        let mut round: Vec<Vec<i32>> = Vec::new();
        rand::thread_rng().shuffle(&mut options);
        for t in 0..6 {
            let mut table: Vec<i32> = Vec::new();
            for p in 0..4 {
                table.push(options[4 * t + p])
            }
            round.push(table);
        }
        event.push(round);
    }
    event
}

fn mutate(mut event: Vec<Vec<Vec<i32>>>) -> Vec<Vec<Vec<i32>>> {
    let mut rng = thread_rng();
    let r: usize = rng.gen_range(0, 6);
    let t1: usize = rng.gen_range(0, 6);
    let t2: usize = match t1.cmp(&0) {
        Ordering::Greater => t1 - 1,
        Ordering::Equal => 5,
        Ordering::Less => 5,
    };
    let p1: usize = rng.gen_range(0, 4);
    let p2: usize = rng.gen_range(0, 4);
    let person1: i32 = event[r][t1][p1];
    event[r][t1][p1] = event[r][t1][p2];
    event[r][t2][p2] = person1;
    //let mut event = &gen_layout();
    event.to_vec()
}

fn main() {
    println!("Welcome to Social Scheduler");
    let mut event: Vec<Vec<Vec<i32>>> = gen_layout();
    let mut score: i32 = get_score(event.clone());
    let mut max: i32 = score;
    loop {
        let mut new: Vec<Vec<Vec<i32>>> = Vec::new();
        let mut changed: bool = false;
        for _ in 0..100 {
            let cevent = mutate(event.clone());
            let cscore = get_score(cevent.clone());
            if cscore > score {
                score = cscore;
                new = cevent.to_vec();
                changed = true;
            }
        }
        if changed {
            event = new;
            if score > max {
                println!("{:?}", score);
                println!("{:?}", event);
                max = score;
            }
        } else {
            event = gen_layout();
            score = get_score(event.clone());
        }
    }
}
