#![feature(test)]

extern crate test;
use patterns::{count_frequency, generate_pattern};

#[bench]
fn bench_pattern(b: &mut test::Bencher) {
    let string = "LALALAXOXOXO";
    b.iter(|| generate_pattern(&string))
}
#[bench]
fn bench_counts(b: &mut test::Bencher) {
    let v = vec![
        vec![0, 0, 1, 0, 0],
        vec![0, 0, 1, 0, 0, 0],
        vec![0, 0, 1, 0, 0],
    ];
    b.iter(|| count_frequency(&v))
}
