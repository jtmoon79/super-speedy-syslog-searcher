// benches/bench_slice_contains.rs
//
// compare `slice.contains` to a custom search

#![allow(non_snake_case)]
#![allow(unused)]

use ::criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};
use ::lazy_static::lazy_static;
use s4lib::data::datetime::{
    slice_contains_D2_custom,
    slice_contains_X_2_memchr,
    slice_contains_X_2_unroll,
};
#[cfg(feature = "bench_jetscii")]
use s4lib::data::datetime::{
    slice_contains_D2_jetscii,
    slice_contains_X_2_jetscii,
};
#[cfg(feature = "bench_stringzilla")]
use s4lib::data::datetime::{
    slice_contains_D2_stringzilla,
    slice_contains_X_2_stringzilla,
};

lazy_static! {
    /// search this slice for a 2 byte "needle"
    pub static ref B200: &'static [u8; 200] = b"\
a_c_e_f_h_1_k_l_n_p_r_t_v_x_z_3ABCDEFGHI4__345678951234567896__34567897__34567898__34567899123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVW__ZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUV";
    /// for D2 search, has two digits in a row `"456789"`
    pub static ref D2_200_HAS_D2: &'static [u8; 200] = b"\
0abcdefghijklmnopqrstuvwxyz1ABCDEFGHIJKLMNOPQRSTUVWXYZ2abcdefghijklmnopqrstuvwxyz3ABCDEFGHIJKLMNOPQRSTUVWXYZ456789fghijklmnopqrstuvwxyz5abcdefghijklmnopqrstuvwxyz6abcdefghijklmnopqrstuvwxyz7abcdefghij";
    /// for D2 search, no two digits in a row
    pub static ref D2_200_NO_D2: &'static [u8; 200] = b"\
0abcdefghijklmnopqrstuvwxyz1ABCDEFGHIJKLMNOPQRSTUVWXYZ2abcdefghijklmnopqrstuvwxyz3ABCDEFGHIJKLMNOPQRSTUVWXYZ4abcdefghijklmnopqrstuvwxyz5abcdefghijklmnopqrstuvwxyz6abcdefghijklmnopqrstuvwxyz7abcdefghij";
}

/// "needle" in B200
const SEARCH12: &[u8; 2] = b"12";
/// "needle" not in B200
const SEARCHXY: &[u8; 2] = b"XY";

#[inline(never)]
fn X_2_baseline() {
    let slice_ = &B200[0..];
    black_box(slice_);
    assert!(true);
}

#[inline(never)]
fn X_2_custom1_slice_iter_found() {
    let slice_ = &B200[0..];
    black_box(slice_);
    for c in slice_.iter() {
        if c == &SEARCH12[0] || c == &SEARCH12[1] {
            assert!(true);
            return;
        }
    }
    assert!(false);
}

#[inline(never)]
fn X_2_custom1_slice_iter_notfound() {
    let slice_ = &B200[0..];
    black_box(slice_);
    for c in slice_.iter() {
        if c == &SEARCHXY[0] || c == &SEARCHXY[1] {
            assert!(false);
            return;
        }
    }
    assert!(true);
}

#[inline(never)]
fn X_2_custom2_slice_iter_found() {
    let slice_ = &B200[0..];
    black_box(slice_);
    for c in slice_.iter() {
        for s in SEARCH12.iter() {
            if c == s {
                assert!(true);
                return;
            }
        }
    }
    assert!(false);
}

#[inline(never)]
fn X_2_custom2_slice_iter_notfound() {
    let slice_ = &B200[0..];
    black_box(slice_);
    for c in slice_.iter() {
        for s in SEARCHXY.iter() {
            if c == s {
                assert!(false);
                return;
            }
        }
    }
    assert!(true);
}

#[inline(never)]
fn X_2_custom3_contains_found() {
    let slice_ = &B200[0..];
    black_box(slice_);
    assert!(slice_.contains(&SEARCH12[0]) || slice_.contains(&SEARCH12[1]));
}

#[inline(never)]
fn X_2_custom3_contains_notfound() {
    let slice_ = &B200[0..];
    black_box(slice_);
    assert!(!(slice_.contains(&SEARCHXY[0]) || slice_.contains(&SEARCHXY[1])));
}

#[inline(never)]
fn X_2_unroll_found_200() {
    let slice_ = &B200[0..];
    black_box(slice_);
    assert!(slice_contains_X_2_unroll(slice_, &SEARCH12));
}

#[inline(never)]
fn X_2_unroll_found_80() {
    let slice_ = &B200[0..80];
    black_box(slice_);
    assert!(slice_contains_X_2_unroll(slice_, &SEARCH12));
}

#[inline(never)]
fn X_2_unroll_notfound_200() {
    let slice_ = &B200[0..];
    black_box(slice_);
    assert!(!slice_contains_X_2_unroll(slice_, &SEARCHXY));
}

#[inline(never)]
fn X_2_unroll_notfound_80() {
    let slice_ = &B200[0..80];
    black_box(slice_);
    assert!(!slice_contains_X_2_unroll(slice_, &SEARCHXY));
}

#[inline(never)]
#[cfg(feature = "bench_jetscii")]
fn X_2_jetscii_found() {
    let slice_ = &B200[0..];
    black_box(slice_);
    assert!(slice_contains_X_2_jetscii(slice_, &SEARCH12));
}

#[inline(never)]
#[cfg(feature = "bench_jetscii")]
fn X_2_jetscii_notfound() {
    let slice_ = &B200[0..];
    black_box(slice_);
    assert!(!slice_contains_X_2_jetscii(slice_, &SEARCHXY));
}

#[inline(never)]
#[cfg(feature = "bench_memchr")]
fn X_2_memchr_found() {
    let slice_ = &B200[0..];
    black_box(slice_);
    assert!(slice_contains_X_2_memchr(slice_, &SEARCH12));
}

#[inline(never)]
#[cfg(feature = "bench_memchr")]
fn X_2_memchr_notfound() {
    let slice_ = &B200[0..];
    black_box(slice_);
    assert!(!slice_contains_X_2_memchr(slice_, &SEARCHXY));
}

#[inline(never)]
#[cfg(feature = "bench_stringzilla")]
fn X_2_stringzilla_found() {
    let slice_ = &B200[0..];
    black_box(slice_);
    assert!(slice_contains_X_2_stringzilla(slice_, &SEARCH12));
}

#[inline(never)]
#[cfg(feature = "bench_stringzilla")]
fn X_2_stringzilla_notfound() {
    let slice_ = &B200[0..];
    black_box(slice_);
    assert!(!slice_contains_X_2_stringzilla(slice_, &SEARCHXY));
}

#[inline(never)]
fn D2_baseline() {
    let slice_ = &B200[0..];
    black_box(slice_);
    assert!(true);
}

#[inline(never)]
fn D2_slice_contains_D2_custom() {
    let slice_ = &D2_200_HAS_D2[0..];
    black_box(slice_);
    assert!(slice_contains_D2_custom(slice_));
}

#[inline(never)]
fn D2_slice_contains_D2_custom_not() {
    let slice_ = &D2_200_NO_D2[0..];
    black_box(slice_);
    assert!(!slice_contains_D2_custom(slice_));
}

#[inline(never)]
#[cfg(feature = "bench_stringzilla")]
fn D2_slice_contains_D2_stringzilla() {
    let slice_ = &D2_200_HAS_D2[0..];
    black_box(slice_);
    assert!(slice_contains_D2_stringzilla(slice_));
}

#[inline(never)]
#[cfg(feature = "bench_stringzilla")]
fn D2_slice_contains_D2_stringzilla_not() {
    let slice_ = &D2_200_NO_D2[0..];
    black_box(slice_);
    assert!(!slice_contains_D2_stringzilla(slice_));
}

// criterion runners

fn criterion_benchmark(c: &mut Criterion) {
    let mut bg = c.benchmark_group("slice_contains");
    // slice contains X_2
    bg.bench_function("X_2_baseline", |b| b.iter(X_2_baseline));
    bg.bench_function("X_2_custom1_slice_iter_found", |b| b.iter(X_2_custom1_slice_iter_found));
    bg.bench_function("X_2_custom1_slice_iter_notfound", |b| b.iter(X_2_custom1_slice_iter_notfound));
    bg.bench_function("X_2_custom2_slice_iter_found", |b| b.iter(X_2_custom2_slice_iter_found));
    bg.bench_function("X_2_custom2_slice_iter_notfound", |b| b.iter(X_2_custom2_slice_iter_notfound));
    bg.bench_function("X_2_custom3_contains_found", |b| b.iter(X_2_custom3_contains_found));
    bg.bench_function("X_2_custom3_contains_notfound", |b| b.iter(X_2_custom3_contains_notfound));
    #[cfg(feature = "bench_jetscii")]
    {
        bg.bench_function("X_2_jetscii_found", |b| b.iter(X_2_jetscii_found));
        bg.bench_function("X_2_jetscii_notfound", |b| b.iter(X_2_jetscii_notfound));
    }
    #[cfg(feature = "bench_memchr")]
    {
        bg.bench_function("X_2_memchr_found", |b| b.iter(X_2_memchr_found));
        bg.bench_function("X_2_memchr_notfound", |b| b.iter(X_2_memchr_notfound));
    }
    bg.bench_function("X_2_unroll_found_200", |b| b.iter(X_2_unroll_found_200));
    bg.bench_function("X_2_unroll_notfound_200", |b| b.iter(X_2_unroll_notfound_200));
    bg.bench_function("X_2_unroll_found_80", |b| b.iter(X_2_unroll_found_80));
    bg.bench_function("X_2_unroll_notfound_80", |b| b.iter(X_2_unroll_notfound_80));
    #[cfg(feature = "bench_stringzilla")]
    {
        bg.bench_function("X_2_stringzilla_found", |b| b.iter(X_2_stringzilla_found));
        bg.bench_function("X_2_stringzilla_notfound", |b| b.iter(X_2_stringzilla_notfound));
    }
    // slice contains D2
    bg.bench_function("D2_baseline", |b| b.iter(D2_baseline));
    bg.bench_function("D2_slice_contains_D2_custom", |b| b.iter(D2_slice_contains_D2_custom));
    bg.bench_function("D2_slice_contains_D2_custom_not", |b| b.iter(D2_slice_contains_D2_custom_not));
    #[cfg(feature = "bench_stringzilla")]
    {
        bg.bench_function("D2_slice_contains_D2_stringzilla", |b| b.iter(D2_slice_contains_D2_stringzilla));
        bg.bench_function("D2_slice_contains_D2_stringzilla_not", |b| b.iter(D2_slice_contains_D2_stringzilla_not));
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
