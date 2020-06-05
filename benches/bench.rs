extern crate criterion;
extern crate rand;

extern crate social_schedule;

use criterion::{criterion_group, criterion_main, Criterion};
use rand::SeedableRng;

fn criterion_benchmark(c: &mut Criterion) {
    let mut seed: [u8; 16] = [0; 16];
    let rng = rand::thread_rng();
    let mut generator = social_schedule::schedule::Generator::new(rng, 24, 6);
    c.bench_function("6 by 4 process", |b| b.iter(|| generator.process()));
    c.bench_function("6 by 4 find_unique_opponents", |b| {
        b.iter(|| generator.best.find_unique_opponents())
    });
    c.bench_function("6 by 4 find_unique_games_played", |b| {
        b.iter(|| generator.best.find_unique_games_played())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
