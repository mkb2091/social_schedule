use std::collections::HashSet;
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

fn mutate(event: Vec<Vec<Vec<i32>>>) -> Vec<Vec<Vec<i32>>> {
    let mut rng = thread_rng();
    let r: i32 = rng.gen_range(0, 6);
    event
}

fn main() {
    println!("Welcome to Social Scheduler");
    let mut max: i32 = 0;
    loop {
        let mut event: Vec<Vec<Vec<i32>>> = gen_layout();
        event = mutate(event.clone());
        let score: i32 = get_score(event.clone());
        if score > max {
            max = score;
            println!("{:?}", score);
            println!("{:?}", event);
        }
    }
}
