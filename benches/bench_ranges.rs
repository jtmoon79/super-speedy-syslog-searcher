// benches/bench_ranges.rs

// using `RangeMap` is surprisingly expensive, according to script
// `tools/flamegraph.sh`
// <https://docs.rs/rangemap/latest/rangemap/map/struct.RangeMap.html>
// <https://docs.rs/range-set-blaze/latest/range_set_blaze/struct.RangeMapBlaze.html>

use std::hint::black_box;
use std::ops::Range;

use ::criterion::{
    BenchmarkId,
    criterion_group,
    criterion_main,
    Criterion,
    Throughput,
};

use ::rangemap::RangeMap;
use ::range_set_blaze::RangeMapBlaze;

type Key = u64;
type Value = u64;
type Rangemap = RangeMap<Key, Value>;
type RangeSetBlaze = RangeMapBlaze<Key, Value>;

const RANGE_COUNTS: [usize; 4] = [10, 100, 1_000, 10_000];
const RANGE_WIDTH: Key = 80;
const RANGE_GAP: Key = 20;

struct Fixture {
    ranges: Vec<(Range<Key>, Value)>,
    hits: Vec<Key>,
    misses: Vec<Key>,
}

impl Fixture {
    fn new(range_count: usize) -> Self {
        let mut ranges = Vec::with_capacity(range_count);
        let mut hits = Vec::with_capacity(range_count);
        let mut misses = Vec::with_capacity(range_count);
        let stride = RANGE_WIDTH + RANGE_GAP;

        for index in 0..range_count {
            let start = index as Key * stride;
            let end = start + RANGE_WIDTH;
            ranges.push((start..end, start));
            hits.push(start + RANGE_WIDTH / 2);
            misses.push(end + RANGE_GAP / 2);
        }

        Self {
            ranges,
            hits,
            misses,
        }
    }
}

fn build_rangemap(fixture: &Fixture) -> Rangemap {
    let mut map = Rangemap::new();
    for (range, value) in &fixture.ranges {
        map.insert(range.clone(), *value);
    }
    map
}

fn build_range_set_blaze(fixture: &Fixture) -> RangeSetBlaze {
    let mut map = RangeSetBlaze::new();
    for (range, value) in &fixture.ranges {
        map.ranges_insert(range.start..=range.end - 1, *value);
    }
    map
}

fn assert_equivalent(
    fixture: &Fixture,
    rangemap: &Rangemap,
    range_set_blaze: &RangeSetBlaze,
) {
    assert_eq!(rangemap.len(), fixture.ranges.len());
    assert_eq!(range_set_blaze.range_values_len(), fixture.ranges.len());

    for key in &fixture.hits {
        assert_eq!(rangemap.get(key), range_set_blaze.get(*key));
    }
    for key in &fixture.misses {
        assert_eq!(rangemap.get(key), None);
        assert_eq!(range_set_blaze.get(*key), None);
    }
}

fn baseline_no_ranges() {
    black_box(0);
}

fn benchmark_get_key_value(
    c: &mut Criterion,
    query_name: &str,
    get_keys: fn(&Fixture) -> &[Key],
) {
    let mut group = c.benchmark_group(format!("ranges/get_key_value/{query_name}"));
    for range_count in RANGE_COUNTS {
        let fixture = Fixture::new(range_count);
        let rangemap = build_rangemap(&fixture);
        let range_set_blaze = build_range_set_blaze(&fixture);
        assert_equivalent(&fixture, &rangemap, &range_set_blaze);
        let keys = get_keys(&fixture);

        group.throughput(Throughput::Elements(range_count as u64));
        group.bench_with_input(
            BenchmarkId::new("rangemap", range_count),
            keys,
            |b, keys| b.iter(|| {
                for key in keys {
                    black_box(rangemap.get_key_value(black_box(key)));
                }
            }),
        );
        group.bench_with_input(
            BenchmarkId::new("range_set_blaze", range_count),
            keys,
            |b, keys| b.iter(|| {
                for key in keys {
                    black_box(range_set_blaze.get_key_value(black_box(*key)));
                }
            }),
        );
    }
    group.finish();
}

fn benchmark_contains_key(
    c: &mut Criterion,
    query_name: &str,
    get_keys: fn(&Fixture) -> &[Key],
) {
    let mut group = c.benchmark_group(format!("ranges/contains_key/{query_name}"));
    for range_count in RANGE_COUNTS {
        let fixture = Fixture::new(range_count);
        let rangemap = build_rangemap(&fixture);
        let range_set_blaze = build_range_set_blaze(&fixture);
        assert_equivalent(&fixture, &rangemap, &range_set_blaze);
        let keys = get_keys(&fixture);

        group.throughput(Throughput::Elements(range_count as u64));
        group.bench_with_input(
            BenchmarkId::new("rangemap", range_count),
            keys,
            |b, keys| b.iter(|| {
                for key in keys {
                    black_box(rangemap.contains_key(black_box(key)));
                }
            }),
        );
        group.bench_with_input(
            BenchmarkId::new("range_set_blaze", range_count),
            keys,
            |b, keys| b.iter(|| {
                for key in keys {
                    black_box(range_set_blaze.contains_key(black_box(*key)));
                }
            }),
        );
    }
    group.finish();
}

// criterion runners

fn criterion_benchmark(c: &mut Criterion) {
    let mut bg = c.benchmark_group("ranges");
    bg.bench_function("baseline_no_ranges", |b| b.iter(baseline_no_ranges));
    bg.finish();

    let mut insert_group = c.benchmark_group("ranges/insert");
    for range_count in RANGE_COUNTS {
        let fixture = Fixture::new(range_count);
        insert_group.throughput(Throughput::Elements(range_count as u64));
        insert_group.bench_with_input(
            BenchmarkId::new("rangemap", range_count),
            &fixture,
            |b, fixture| b.iter(|| black_box(build_rangemap(black_box(fixture)))),
        );
        insert_group.bench_with_input(
            BenchmarkId::new("range_set_blaze", range_count),
            &fixture,
            |b, fixture| b.iter(|| black_box(build_range_set_blaze(black_box(fixture)))),
        );
    }
    insert_group.finish();

    benchmark_get_key_value(c, "hit", |fixture| &fixture.hits);
    benchmark_get_key_value(c, "miss", |fixture| &fixture.misses);
    benchmark_contains_key(c, "hit", |fixture| &fixture.hits);
    benchmark_contains_key(c, "miss", |fixture| &fixture.misses);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
