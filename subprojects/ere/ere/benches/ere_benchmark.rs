use std::hash::Hasher;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ere::{compile_regex, Regex};
use pprof::criterion::{Output, PProfProfiler};

macro_rules! key_to_engine {
    ($re:literal, flat_lockstep_nfa) => {
        (
            "flat_lockstep_nfa",
            ::ere::compile_regex_flat_lockstep_nfa!($re),
        )
    };
    ($re:literal, flat_lockstep_nfa_u8) => {
        (
            "flat_lockstep_nfa_u8",
            ::ere::compile_regex_flat_lockstep_nfa_u8!($re),
        )
    };
    ($re:literal, one_pass_u8) => {
        ("one_pass_u8", ::ere::compile_regex_u8onepass!($re))
    };
    ($re:literal, fixed_offset) => {
        ("fixed_offset", ::ere::compile_regex_fixed_offset!($re))
    };
}

/// ## Usage:
/// `compile_engines!("some regex", engine[, engines...])`
///
/// Where valid engine ids are:
/// - `flat_lockstep_nfa` for [`::ere::compile_regex_flat_lockstep_nfa`]
/// - `flat_lockstep_nfa_u8` for [`::ere::compile_regex_flat_lockstep_nfa_u8`]
/// - `one_pass_u8` for [`::ere::compile_regex_u8onepass`]
/// - `fixed_offset` for [`::ere::compile_regex_fixed_offset`]
///
/// ## Output
/// Pairs with labels and regexes e.g.
/// `("flat_lockstep_nfa", <regex struct>), ...`
macro_rules! compile_engines {
    ($re:literal, $($engines:ident),+$(,)?) => {
        [
            $(
                key_to_engine!($re, $engines)
            ),+
        ]
    }
}

fn rgb_simple(c: &mut Criterion) {
    // test all relevant engines
    const REGEXES: [(&'static str, Regex<4>); 1] = compile_engines!(
        r"^#([[:alnum:]]{2})([[:alnum:]]{2})([[:alnum:]]{2})$",
        // flat_lockstep_nfa,
        // flat_lockstep_nfa_u8,
        // one_pass_u8,
        fixed_offset,
    );

    let regex_runtime =
        ::regex::Regex::new(r"^#([[:alnum:]]{2})([[:alnum:]]{2})([[:alnum:]]{2})$").unwrap();

    let mut group = c.benchmark_group("rgb_simple");
    for i in 0..5 {
        let mut hasher = ::std::hash::DefaultHasher::new();
        hasher.write_u8(i);
        let [r, g, b, ..] = hasher.finish().to_le_bytes();
        let haystack = format!("#{r:02x}{g:02x}{b:02x}");

        for (engine_name, engine) in REGEXES.iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("ere {engine_name} (test)"), &haystack),
                &haystack,
                |b, s| b.iter(|| assert!(engine.test(s))),
            );
            group.bench_with_input(
                BenchmarkId::new(format!("ere {engine_name} (exec)"), &haystack),
                &haystack,
                |b, s| b.iter(|| assert!(engine.exec(s).is_some())),
            );
        }
        group.bench_with_input(
            BenchmarkId::new("regex (test)", &haystack),
            &haystack,
            |b, s| b.iter(|| assert!(regex_runtime.is_match(s))),
        );
        group.bench_with_input(
            BenchmarkId::new("regex (exec)", &haystack),
            &haystack,
            |b, s| b.iter(|| assert!(regex_runtime.captures(s).is_some())),
        );
    }
}

fn rgba(c: &mut Criterion) {
    // test all relevant engines
    const REGEXES: [(&'static str, Regex<5>); 1] = compile_engines!(
        r"^#([[:alnum:]]{2})([[:alnum:]]{2})([[:alnum:]]{2})([[:alnum:]]{2})?$",
        // flat_lockstep_nfa,
        // flat_lockstep_nfa_u8,
        one_pass_u8,
    );

    let regex_runtime = ::regex::Regex::new(
        r"^#([[:alnum:]]{2})([[:alnum:]]{2})([[:alnum:]]{2})([[:alnum:]]{2})?$",
    )
    .unwrap();

    let mut group = c.benchmark_group("rgba");
    for i in 0..5 {
        let mut hasher = ::std::hash::DefaultHasher::new();
        hasher.write_u8(i);
        let [r, g, b, a, has_alpha, ..] = hasher.finish().to_le_bytes();
        let haystack = if has_alpha >= 128 {
            format!("#{r:02x}{g:02x}{b:02x}")
        } else {
            format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
        };

        for (engine_name, engine) in REGEXES.iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("ere {engine_name} (test)"), &haystack),
                &haystack,
                |b, s| b.iter(|| assert!(engine.test(s))),
            );
            group.bench_with_input(
                BenchmarkId::new(format!("ere {engine_name} (exec)"), &haystack),
                &haystack,
                |b, s| b.iter(|| assert!(engine.exec(s).is_some())),
            );
        }
        group.bench_with_input(
            BenchmarkId::new("regex (test)", &haystack),
            &haystack,
            |b, s| b.iter(|| assert!(regex_runtime.is_match(s))),
        );
        group.bench_with_input(
            BenchmarkId::new("regex (exec)", &haystack),
            &haystack,
            |b, s| b.iter(|| assert!(regex_runtime.captures(s).is_some())),
        );
    }
}

fn phone_number_usa(c: &mut Criterion) {
    // test all relevant engines
    const REGEXES: [(&'static str, Regex<2>); 1] = compile_engines!(
        r"^(\+1 )?[0-9]{3}-[0-9]{3}-[0-9]{4}$",
        // flat_lockstep_nfa,
        // flat_lockstep_nfa_u8,
        one_pass_u8,
    );

    let regex_runtime = ::regex::Regex::new(r"^(\+1 )?[0-9]{3}-[0-9]{3}-[0-9]{4}$").unwrap();

    let mut group = c.benchmark_group("phone_number_usa");
    for i in 0..5 {
        let mut hasher = ::std::hash::DefaultHasher::new();
        hasher.write_u8(i);
        let value = hasher.finish();
        let has_country_code = value & 1 != 0;
        let value = format!("{value:010}");
        let haystack = if has_country_code {
            format!("+1 {}-{}-{}", &value[0..3], &value[3..6], &value[6..10])
        } else {
            format!("{}-{}-{}", &value[0..3], &value[3..6], &value[6..10])
        };

        for (engine_name, engine) in REGEXES.iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("ere {engine_name} (test)"), &haystack),
                &haystack,
                |b, s| b.iter(|| assert!(engine.test(s))),
            );
            group.bench_with_input(
                BenchmarkId::new(format!("ere {engine_name} (exec)"), &haystack),
                &haystack,
                |b, s| b.iter(|| assert!(engine.exec(s).is_some())),
            );
        }
        group.bench_with_input(
            BenchmarkId::new("regex (test)", &haystack),
            &haystack,
            |b, s| b.iter(|| assert!(regex_runtime.is_match(s))),
        );
        group.bench_with_input(
            BenchmarkId::new("regex (exec)", &haystack),
            &haystack,
            |b, s| b.iter(|| assert!(regex_runtime.captures(s).is_some())),
        );
    }
}

fn ipv4(c: &mut Criterion) {
    const REGEX: Regex<5> = compile_regex!(
        r"^(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])$"
    );

    let regex_runtime = ::regex::Regex::new(r"^(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])$").unwrap();

    let mut group = c.benchmark_group("ipv4");
    for i in 0..5 {
        let mut hasher = ::std::hash::DefaultHasher::new();
        hasher.write_u8(i);
        let [b0, b1, b2, b3, ..] = hasher.finish().to_le_bytes();
        let haystack = format!("{b0}.{b1}.{b2}.{b3}");

        group.bench_with_input(
            BenchmarkId::new("ere (test)", &haystack),
            &haystack,
            |b, s| b.iter(|| assert!(REGEX.test(s))),
        );
        group.bench_with_input(
            BenchmarkId::new("ere (exec)", &haystack),
            &haystack,
            |b, s| b.iter(|| assert!(REGEX.exec(s).is_some())),
        );
        group.bench_with_input(
            BenchmarkId::new("regex (test)", &haystack),
            &haystack,
            |b, s| b.iter(|| assert!(regex_runtime.is_match(s))),
        );
        group.bench_with_input(
            BenchmarkId::new("regex (exec)", &haystack),
            &haystack,
            |b, s| b.iter(|| assert!(regex_runtime.captures(s).is_some())),
        );
    }
}

fn big_haystack_1(c: &mut Criterion) {
    const REGEX: Regex<1> = compile_regex!("ab+c");

    let regex_runtime = ::regex::Regex::new("ab+c").unwrap();

    let haystacks = [
        "abc".to_owned() + &"ac".repeat(10000),
        "ac".repeat(2500) + "abc" + &"ac".repeat(7500),
        "ac".repeat(5000) + "abc" + &"ac".repeat(5000),
        "ac".repeat(7500) + "abc" + &"ac".repeat(2500),
        "ac".repeat(10000) + "abc",
        "abc".to_owned() + &"az".repeat(10000),
        "az".repeat(2500) + "abc" + &"az".repeat(7500),
        "az".repeat(5000) + "abc" + &"az".repeat(5000),
        "az".repeat(7500) + "abc" + &"az".repeat(2500),
        "az".repeat(10000) + "abc",
        "abc".to_owned() + &"zz".repeat(10000),
        "zz".repeat(2500) + "abc" + &"zz".repeat(7500),
        "zz".repeat(5000) + "abc" + &"zz".repeat(5000),
        "zz".repeat(7500) + "abc" + &"zz".repeat(2500),
        "zz".repeat(10000) + "abc",
    ];

    let mut group = c.benchmark_group("big_haystack_1");
    for (i, haystack) in haystacks.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("ere (test)", i),
            haystack.as_str(),
            |b, s| b.iter(|| assert!(REGEX.test(s))),
        );
        group.bench_with_input(
            BenchmarkId::new("ere (exec)", i),
            haystack.as_str(),
            |b, s| b.iter(|| assert!(REGEX.exec(s).is_some())),
        );
        group.bench_with_input(
            BenchmarkId::new("regex (test)", i),
            haystack.as_str(),
            |b, s| b.iter(|| assert!(regex_runtime.is_match(s))),
        );
        group.bench_with_input(
            BenchmarkId::new("regex (exec)", i),
            haystack.as_str(),
            |b, s| b.iter(|| assert!(regex_runtime.captures(s).is_some())),
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = rgb_simple, rgba, phone_number_usa, ipv4, big_haystack_1
}
criterion_main!(benches);
