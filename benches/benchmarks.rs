#[macro_use]
extern crate criterion;

use criterion::Criterion;

fn criterion_benchmark(c: &mut Criterion) {
    let round: Vec<Vec<usize>> = vec![
        vec![0, 1, 2, 3],
        vec![4, 5, 6, 7],
        vec![8, 9, 10, 11],
        vec![12, 13, 14, 15],
        vec![16, 17, 18, 19],
        vec![20, 21, 22, 23],
    ];
    let mut game: Vec<Vec<Vec<usize>>> = Vec::new();
    for _ in 0..6 {
        game.push(round.clone());
    }

    if let Ok(mut schedule) = social_schedule::schedule::Schedule::from_vec(24, 6, game) {
        c.bench_function("unique_games_played", move |b| {
            b.iter(|| {
                criterion::black_box(schedule.unique_games_played());
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
