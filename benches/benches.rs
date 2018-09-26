use criterion::{criterion_group, criterion_main, Criterion};
use patterns::{count_frequency, generate_pattern};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("pattern generation", |bencher| {
        let string = "LALALAXOXOXO";
        bencher.iter(|| generate_pattern(&string));
    });

    c.bench_function("raw counts", |bencher| {
        let v = vec![
            vec![0, 0, 1, 0, 0],
            vec![0, 0, 1, 0, 0, 0],
            vec![0, 0, 1, 0, 0],
        ];
        bencher.iter(|| count_frequency(&v))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
