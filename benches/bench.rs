extern crate criterion;
extern crate rand;

extern crate social_schedule;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::SeedableRng;

fn criterion_benchmark(c: &mut Criterion) {
    let mut seed: [u8; 16] = [0; 16];
    getrandom::getrandom(&mut seed).unwrap();
    let rng = rand_xorshift::XorShiftRng::from_seed(seed);
    let mut generator = social_schedule::schedule::Generator::new(rng, 24, 6);
    c.bench_function("6 by 4 process", |b| b.iter(|| generator.process()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
