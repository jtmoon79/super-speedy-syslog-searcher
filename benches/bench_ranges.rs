// bench_ranges.rs
//
// using `RangeMap` is surprisingly expensive, according to `tools/flamegraph.sh`
// https://docs.rs/rangemap/latest/rangemap/
// 

extern crate criterion;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

extern crate rangemap;
use rangemap::{RangeMap,RangeSet};

type RangeMapT = RangeMap<u64, u64>;
type RangeSetT = RangeSet<u64>;

fn baseline_no_ranges() {
    return;
}

// TODO: compare `RangeMapT` and `RangeSetT` times for `insert` and `get` and `contains`
//       also compare `BTreeMap.contains_key`

// criterion runners

fn criterion_benchmark(c: &mut Criterion) {
    let mut bg = c.benchmark_group("RangeMap");
    bg.bench_function("baseline_no_ranges", |b| b.iter(|| baseline_no_ranges()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
