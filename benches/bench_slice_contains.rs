// benches/bench_slice_contains.rs
//
// compare `slice.contains` to a custom search

#![allow(non_snake_case)]
#![allow(unused)]

use ::jetscii::bytes;
use ::criterion::{black_box, criterion_group, criterion_main, Criterion};
use ::lazy_static::lazy_static;

use s4lib::data::datetime::slice_contains_X_2;

lazy_static! {
    pub static ref B70: &'static [u8; 70] = b"\
0_23456789112345678921234567893123456789412345678951234567896123456789";
}

// data present in B70
const SEARCH12: &[u8; 2] = b"12";
// data not present in B70
const SEARCHXY: &[u8; 2] = b"XY";

#[inline(never)]
fn B70_baseline() {
    let slice_ = &B70[0..];
    black_box(slice_);
}

#[inline(never)]
fn custom1_found() {
    let slice_ = &B70[0..];
    black_box(slice_);
    for c in slice_.iter() {
        if c == &SEARCH12[0] || c == &SEARCH12[1] {
            return;
        }
    }
}

#[inline(never)]
fn custom1_notfound() {
    let slice_ = &B70[0..];
    black_box(slice_);
    for c in slice_.iter() {
        if c == &SEARCHXY[0] || c == &SEARCHXY[1] {
            return;
        }
    }
}

#[inline(never)]
fn custom2_found() {
    let slice_ = &B70[0..];
    black_box(slice_);
    for c in slice_.iter() {
        for s in SEARCH12.iter() {
            if c == s {
                return;
            }
        }
    }
}

#[inline(never)]
fn custom2_notfound() {
    let slice_ = &B70[0..];
    black_box(slice_);
    for c in slice_.iter() {
        for s in SEARCHXY.iter() {
            if c == s {
                return;
            }
        }
    }
}

#[inline(never)]
fn custom3_found() {
    let slice_ = &B70[0..];
    black_box(slice_);
    if slice_.contains(&SEARCH12[0]) || slice_.contains(&SEARCH12[1]) {
        return;
    }
}

#[inline(never)]
fn custom3_notfound() {
    let slice_ = &B70[0..];
    black_box(slice_);
    if slice_.contains(&SEARCHXY[0]) || slice_.contains(&SEARCHXY[1]) {
        return;
    }
}

#[inline(never)]
fn custom4_found_slice_contains_X_2() {
    let slice_ = &B70[0..];
    black_box(slice_);
    slice_contains_X_2(slice_, &SEARCH12);
}

#[inline(never)]
fn custom4_notfound_slice_contains_X_2() {
    let slice_ = &B70[0..];
    black_box(slice_);
    slice_contains_X_2(slice_, &SEARCHXY);
}

#[inline(never)]
fn custom5_found_jetscii() {
    let slice_ = &B70[0..];
    black_box(slice_);
    bytes!(SEARCH12[0], SEARCH12[1]).find(&slice_);
}

#[inline(never)]
fn custom5_notfound_jetscii() {
    let slice_ = &B70[0..];
    black_box(slice_);
    bytes!(SEARCHXY[0], SEARCHXY[1]).find(&slice_);
}

// criterion runners

fn criterion_benchmark(c: &mut Criterion) {
    let mut bg = c.benchmark_group("slice_contains");
    bg.bench_function("B70_baseline", |b| b.iter(B70_baseline));
    //bg.bench_function("B70_slice_contains_found1", |b| b.iter(slice_contains_found1));
    bg.bench_function("custom1_found", |b| b.iter(custom1_found));
    bg.bench_function("custom1_notfound", |b| b.iter(custom1_notfound));
    bg.bench_function("custom2_found", |b| b.iter(custom2_found));
    bg.bench_function("custom2_notfound", |b| b.iter(custom2_notfound));
    bg.bench_function("custom3_found", |b| b.iter(custom3_found));
    bg.bench_function("custom3_notfound", |b| b.iter(custom3_notfound));
    bg.bench_function("custom4_found_slice_contains_X_2", |b| b.iter(custom4_found_slice_contains_X_2));
    bg.bench_function("custom4_notfound_slice_contains_X_2", |b| b.iter(custom4_notfound_slice_contains_X_2));
    bg.bench_function("custom5_found_jetscii", |b| b.iter(custom5_found_jetscii));
    bg.bench_function("custom5_notfound_jetscii", |b| b.iter(custom5_notfound_jetscii));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
