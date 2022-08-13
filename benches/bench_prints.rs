// benches/bench_prints.rs
//
// benchmark different printing approaches

#![allow(non_upper_case_globals, dead_code, non_snake_case)]

extern crate criterion;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

extern crate lazy_static;
use lazy_static::lazy_static;

//
// test data
//

// LAST WORKING HERE 2022/04/30 19:38:02
// just get this going quickly, no need to be too fancy.
// later, can test with mutex and whatnot.

lazy_static! {
    static ref Data1: Vec<u8> = Vec::from(
        b"2000-01-01 00:00:00 abcdefghijklmnopqrstuvwxyz".as_slice()
    );
}

//
// differing decoding functions
//

/// do common set of activities
#[inline(never)]
fn print_baseline() {
    black_box(0);
}

/// use one global `termcolor::ClrOut` instance
#[inline(never)]
fn print_termcolor_one() {
    
}

/// create `termcolor::ClrOut` instance each loop
#[inline(never)]
fn print_termcolor_many() {
    
}

/// call `stdout::write` directly
#[inline(never)]
fn print_write() {
    
}

//
// criterion runners
//

fn criterion_benchmark(c: &mut Criterion) {
    let mut bg = c.benchmark_group("print");
    bg.bench_function("print_baseline", |b| b.iter(print_baseline));
    bg.bench_function("print_termcolor_one", |b| b.iter(print_termcolor_one));
    bg.bench_function("print_termcolor_many", |b| b.iter(print_termcolor_many));
    bg.bench_function("print_write", |b| b.iter(print_write));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
