// benches/bench_slice_contains.rs
//
// compare `slice.contains` to a custom search

extern crate criterion;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const B70: &[u8; 70] = b"\
0123456789112345678921234567893123456789412345678951234567896123456789";

const SEARCH1: &[u8; 2] = b"12";

#[inline(never)]
fn baseline() {
    let slice_ = &B70[0..];
    black_box(slice_);
}

#[inline(never)]
fn slice_contains_found1() {
    let slice_ = &B70[0..];
    if slice_.contains(&SEARCH1[0]) || slice_.contains(&SEARCH1[1]){
        black_box(0);
    }
}

#[inline(never)]
fn custom1_found1() {
    let slice_ = &B70[0..];
    for c in slice_.iter() {
        if c == &SEARCH1[0] || c == &SEARCH1[1] {
            return;
        }
    }
}

#[inline(never)]
fn custom2_found1() {
    let slice_ = &B70[0..];
    for c in slice_.iter() {
        for s in SEARCH1.iter() {
            if c == s {
                return;
            }
        }
    }
}

const SEARCH2: &[u8; 2] = b"XY";

#[inline(never)]
fn slice_contains_notfound2() {
    let slice_ = &B70[0..];
    if slice_.contains(&SEARCH2[0]) || slice_.contains(&SEARCH2[1]){
        black_box(0);
    }
}

#[inline(never)]
fn custom1_notfound2() {
    let slice_ = &B70[0..];
    for c in slice_.iter() {
        if c == &SEARCH2[0] || c == &SEARCH2[1] {
            return;
        }
    }
}

#[inline(never)]
fn custom2_notfound2() {
    let slice_ = &B70[0..];
    for c in slice_.iter() {
        for s in SEARCH2.iter() {
            if c == s {
                return;
            }
        }
    }
}

// criterion runners

fn criterion_benchmark(c: &mut Criterion) {
    let mut bg = c.benchmark_group("slice_contains");
    bg.bench_function("baseline", |b| b.iter(baseline));
    bg.bench_function("slice_contains_found1", |b| b.iter(slice_contains_found1));
    bg.bench_function("custom1_found1", |b| b.iter(custom1_found1));
    bg.bench_function("custom2_found1", |b| b.iter(custom2_found1));
    bg.bench_function("slice_contains_notfound2", |b| b.iter(slice_contains_notfound2));
    bg.bench_function("custom1_notfound2", |b| b.iter(custom1_notfound2));
    bg.bench_function("custom2_notfound2", |b| b.iter(custom2_notfound2));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
