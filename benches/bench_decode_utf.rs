// benches/bench_decode_utf.rs

//! Decoding `[u8]` to `str` is a surprisingly resource intensive process,
//! according to reviews of this program run using `tools/flamegraph.sh`
//! this bench file is a comparison of strategies.
//!
//! Some other interesting projects and write-ups:
//! * <https://github.com/killercup/simd-utf8-check>
//! * <https://killercup.github.io/simd-utf8-check/report/index.html>
//! * <https://woboq.com/blog/utf-8-processing-using-simd.html>
//! * <https://github.com/shepmaster/jetscii>
//! * <https://github.com/gnzlbg/is_utf8>
//! * <https://lemire.me/blog/2018/05/16/validating-utf-8-strings-using-as-little-as-0-7-cycles-per-byte/>
//! * <https://gist.github.com/jFransham/369a86eff00e5f280ed25121454acec1#loop-unrolling-is-still-cool>
//! * <https://lise-henry.github.io/articles/optimising_strings.html>

#![allow(non_upper_case_globals, dead_code, non_snake_case)]

extern crate arraystring;

extern crate bstr;
use bstr::ByteSlice; // adds method `to_bstr` on some built-ins

extern crate criterion;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

extern crate encoding_rs;

//extern crate jetscii;
//use jetscii::bytes;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate memchr;

type Bytes = Vec<u8>;

lazy_static! {
    static ref Data1: Bytes = vec![b'2', b'0', b'0', b'0', b'T', b'0', b'0', b'0', b'0', b'0', b'1',];
}

lazy_static! {
    static ref Data2: Bytes = vec![b'2', b'0', b'0', b'0', b'T', b'0', b'0', b'0', b'0', b'0', b'1', b' ', b' ',];
}

lazy_static! {
    static ref Data3: Bytes = vec![
        b'2', b'0', b'0', b'0', b'T', b'0', b'0', b'0', b'0', b'0', b'1', b' ', b'A', b'B', b'C', b'D', b'E', b'F',
        b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N',
    ];
}

lazy_static! {
    static ref Data4: Bytes = vec![
        b'2', b'0', b'0', b'0', b'T',
        b'0', b'0',
        b'0', b'0',
        b'0', b'1',
        // valid UTF8, non-ASCII chars
        0xC3, 0x81, // Á
        b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I',
    ];
}

lazy_static! {
    // invalid UTF8
    static ref Data5i: Bytes = vec![
        b'2', b'0', b'0', b'0', b'T',
        b'0', b'0',
        b'0', b'0',
        b'0', b'1',
        // value 0xFF is invalid UTF8 value
        0xFF, b' ',
    ];
}

lazy_static! {
    // invalid UTF8
    static ref Data6i: Bytes = vec![
        b'2', b'0', b'0', b'0', b'T',
        b'0', b'0',
        b'0', b'0',
        b'0', b'1',
        // four UTF8 continuation bytes in a row; seems invalid to me
        0x80, 0x80,
        0x80, 0x80,
        b'A',
    ];
}

lazy_static! {
    static ref Datas: Vec<&'static Vec<u8>> = vec![
        // easy runs (no invalid UTF8) x18
        &Data1, &Data1, &Data1,
        &Data2, &Data2, &Data2, &Data2, &Data2,
        &Data3, &Data3, &Data3, &Data3, &Data3,
        &Data4, &Data4, &Data4, &Data4, &Data4,
        // hard runs (invalid UTF8) x2 (10% of runs)
        &Data5i, &Data6i
    ];
}
const COUNT_DATAS: u32 = 20;
const COUNT_VALID_UTF8_IN_DATAS: u32 = 18;
const COUNT_ASCIIONLY_IN_DATAS: u32 = 13;

const DATA_MAX_SZ: usize = 30; // with some overhead

fn Datas_check() {
    // quick self-check the `DataX` are as expected
    match std::str::from_utf8(Data1.as_slice()) {
        | Ok(_) => {}
        | Err(err) => {
            panic!("ERROR Data1 {}", err);
        }
    }
    match std::str::from_utf8(Data2.as_slice()) {
        | Ok(_) => {}
        | Err(err) => {
            panic!("ERROR Data2 {}", err);
        }
    }
    match std::str::from_utf8(Data3.as_slice()) {
        | Ok(_) => {}
        | Err(err) => {
            panic!("ERROR Data3 {}", err);
        }
    }
    match std::str::from_utf8(Data4.as_slice()) {
        | Ok(_) => {}
        | Err(err) => {
            panic!("ERROR Data4 {}", err);
        }
    }
    #[allow(clippy::single_match)]
    match std::str::from_utf8(Data5i.as_slice()) {
        | Ok(_) => {
            panic!("ERROR expected invalid UTF8 Data5i, but it appears valid according to `from_utf8`");
        }
        | Err(_) => {}
    }
    #[allow(clippy::single_match)]
    match std::str::from_utf8(Data6i.as_slice()) {
        | Ok(_) => {
            panic!("ERROR expected invalid UTF8 Data6i, but it appears valid according to `from_utf8`");
        }
        | Err(_) => {}
    }
}

//
// differing decoding functions
//

#[inline(never)]
fn dutf8_baseline_no_decoding() {
    // run common set of decoding processing but skip actual decoding
    let mut count: u32 = 0;
    for data in Datas.iter() {
        let data_slice = data.as_slice();
        black_box(data_slice);
        let dts: &str = "ABCDEFGHIJKL"; // replaces decoding
        let s1 = String::from(dts);
        black_box(s1);
        count += 1;
    }
    assert_eq!(count, COUNT_DATAS, "FAILED TO CATCH PROCESS ALL DATAS");
}

#[inline(never)]
fn dutf8_encodingrs_decode_to_string() {
    // use encoding_rs::Decoder::decode_to_string
    let mut count: u32 = 0;
    let mut bufs: String = String::with_capacity(50);
    for data in Datas.iter() {
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let data_slice = data.as_slice();
        let (_dresult, _sz, replaced) = decoder.decode_to_string(data_slice, &mut bufs, true);
        if replaced {
            continue; // invalid UTF8
        }
        let dts = bufs.as_str();
        //let s1 = String::from(dts);  // not needed
        black_box(dts);
        count += 1;
        bufs.clear();
    }
    assert_eq!(count, COUNT_VALID_UTF8_IN_DATAS, "FAILED TO CATCH INVALID UTF8");
}

#[inline(never)]
fn dutf8_encodingrs_decode_to_string_without_replacement() {
    // use encoding_rs::Decoder::decode_to_string_without_replacement
    let mut count: u32 = 0;
    let mut bufs: String = String::with_capacity(50);
    for data in Datas.iter() {
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let data_slice = data.as_slice();
        let (dresult, _) = decoder.decode_to_string_without_replacement(data_slice, &mut bufs, true);
        #[allow(clippy::single_match)]
        match dresult {
            | encoding_rs::DecoderResult::Malformed(_, _) => {
                continue; // invalid UTF8
            }
            | _ => {}
        }
        let dts = bufs.as_str();
        //let s1 = String::from(dts);  // not needed
        black_box(dts);
        count += 1;
        bufs.clear();
    }
    assert_eq!(count, COUNT_VALID_UTF8_IN_DATAS, "FAILED TO CATCH INVALID UTF8");
}

#[inline(never)]
fn dutf8_encodingrs_mem_utf8_latin1_up_to__std_str_from_utf8_unchecked() {
    // use encoding_rs::mem::utf8_latin1_up_to check before unsafe decoding
    let mut count: u32 = 0;
    for data in Datas.iter() {
        let data_slice = data.as_slice();
        let va = encoding_rs::mem::utf8_latin1_up_to(data_slice);
        if va != data_slice.len() {
            continue; // invalid UTF8
        }
        let dts: &str;
        unsafe {
            dts = std::str::from_utf8_unchecked(data_slice);
        };
        let s1 = String::from(dts);
        black_box(s1);
        count += 1;
    }
    assert_eq!(count, COUNT_VALID_UTF8_IN_DATAS, "FAILED TO CATCH INVALID UTF8");
}

#[inline(never)]
fn dutf8_std_str_from_utf8() {
    // use built-in safe decoding
    let mut count: u32 = 0;
    for data in Datas.iter() {
        let data_slice = data.as_slice();
        // using built-in safe str::from_utf8
        let dts = match std::str::from_utf8(data_slice) {
            | Ok(val) => val,
            | Err(_) => {
                continue; // invalid UTF8
            }
        };
        let s1 = String::from(dts);
        black_box(s1);
        count += 1;
    }
    assert_eq!(count, COUNT_VALID_UTF8_IN_DATAS, "FAILED TO CATCH INVALID UTF8");
}

#[inline(never)]
fn dutf8_std_str_from_utf8_unchecked__allows_invalid() {
    // use built-in unsafe decoding; invalid UTF8 not caught
    let mut count: u32 = 0;
    for data in Datas.iter() {
        let data_slice = data.as_slice();
        let dts: &str;
        unsafe {
            dts = std::str::from_utf8_unchecked(data_slice);
        };
        let s1 = String::from(dts);
        black_box(s1);
        count += 1;
    }
    // underzealous, will not catch all invalid UTF8
    //assert_eq!(count, COUNT_VALID_UTF8_IN_DATAS, "FAILED TO CATCH INVALID UTF8");
    assert_eq!(count, COUNT_DATAS, "FAILED TO CATCH PROCESS ALL DATAS");
}

#[inline(never)]
fn dutf8_custom_check1_lt0x80__overzealous() {
    // custom check data before built-in unsafe decoding
    // this custom check has errors! just a quick hack to try out ideas
    let mut count: u32 = 0;
    for data in Datas.iter() {
        let data_slice = data.as_slice();
        // custom check for utf8 invalid bytes
        let mut invalid = false;
        for b in data_slice.iter() {
            let c = *b;
            //if (0x80 as u8 <= c && c <= 0xC2 as u8) || (192 as u8 <= c && c <= 193 as u8) || (245 as u8 <= c && c <= 255 as u8) {
            if 0x80 <= c {
                invalid = true;
                break; // "invalid" UTF8; refuse to check further
            };
        }
        if invalid {
            continue;
        };
        let dts: &str;
        unsafe {
            dts = std::str::from_utf8_unchecked(data_slice);
        };
        let s1 = String::from(dts);
        black_box(s1);
        count += 1;
    }
    // overzealous marking of "invalid"
    //assert_eq!(count, COUNT_VALID_UTF8_IN_DATAS, "FAILED TO CATCH INVALID UTF8");
    assert_eq!(count, COUNT_ASCIIONLY_IN_DATAS, "FAILED TO CATCH INVALID UTF8 or ASCII");
}

#[inline(never)]
fn dutf8_custom_check2_lt0x80__fallback__encodingrs_mem_utf8_latin1_up_to__overzealous() {
    // custom check data before built-in unsafe decoding
    // custom check will fallback to encoding_rs::mem::utf8_latin1_up_to if things get difficult
    let mut count: u32 = 0;
    for data in Datas.iter() {
        let data_slice = data.as_slice();
        let dts: &str;
        // custom check for UTF8 some known invalid UTF8 bytes
        // see https://en.wikipedia.org/wiki/UTF-8#Codepage_layout
        let mut fallback = false;
        for b in data_slice.iter() {
            let c = *b;
            // invalid UTF8 codes
            //if (0xC0 as u8 <= c && c <= 0xC1 as u8) || (0xF5 as u8 <= c && c <= 0xFF as u8) {
            if 0x80 <= c {
                fallback = true;
                break;
            };
        }
        // UTF8 special chars found, fallback to `utf8_latin1_up_to` which can do this correctly
        if fallback {
            let va = encoding_rs::mem::utf8_latin1_up_to(data_slice);
            if va != data_slice.len() {
                continue; // invalid UTF8
            }
            break; // valid UTF8
        }
        unsafe {
            dts = std::str::from_utf8_unchecked(data_slice);
        };
        let s1 = String::from(dts);
        black_box(s1);
        count += 1;
    }
    // overzealous; `utf8_latin1_up_to` does not handle UTF8 continuation bytes
    //assert_eq!(count, COUNT_VALID_UTF8_IN_DATAS, "FAILED TO CATCH INVALID UTF8");
    assert_eq!(count, COUNT_ASCIIONLY_IN_DATAS, "FAILED TO CATCH INVALID UTF8 or ASCII");
}

#[inline(never)]
fn dutf8_custom_check3__is_ascii__fallback__encodingrs_mem_utf8_latin1_up_to__overzealous() {
    // custom check data before built-in unsafe decoding
    // custom check will fallback to encoding_rs::mem::utf8_latin1_up_to if things get difficult
    let mut count: u32 = 0;
    for data in Datas.iter() {
        let data_slice = data.as_slice();
        let dts: &str;
        // custom check for UTF8 some known invalid UTF8 bytes
        let mut fallback = false;
        if !data_slice.is_ascii() {
            fallback = true;
        }
        // non-ASCII found, fallback to `utf8_latin1_up_to` which can do a better check
        if fallback {
            let va = encoding_rs::mem::utf8_latin1_up_to(data_slice);
            if va != data_slice.len() {
                continue; // invalid UTF8
            }
            break; // valid UTF8
        }
        unsafe {
            dts = std::str::from_utf8_unchecked(data_slice);
        };
        let s1 = String::from(dts);
        black_box(s1);
        count += 1;
    }
    // overzealous; `utf8_latin1_up_to` does not handle UTF8 continuation bytes
    //assert_eq!(count, COUNT_VALID_UTF8_IN_DATAS, "FAILED TO CATCH INVALID UTF8");
    assert_eq!(count, COUNT_ASCIIONLY_IN_DATAS, "FAILED TO CATCH INVALID UTF8 or ASCII");
}

#[inline(never)]
fn dutf8_bstr_to_str() {
    // use bstr::BStr (as_bstr)
    let mut count: u32 = 0;
    for data in Datas.iter() {
        let data_slice = data.as_slice();
        let bstr1 = data_slice.as_bstr();
        let str1 = match bstr1.to_str() {
            | Ok(val) => val,
            | Err(_) => {
                continue;
            }
        };
        let s1 = String::from(str1);
        black_box(s1);
        count += 1;
    }
    assert_eq!(count, COUNT_VALID_UTF8_IN_DATAS, "FAILED TO CATCH INVALID UTF8");
}

#[inline(never)]
fn dutf8_arraystring__SmallString_from_utf8() {
    // use arraystring::ArrayString (as_bstr)
    let mut count: u32 = 0;
    for data in Datas.iter() {
        let data_slice = data.as_slice();
        //let arraystring1 = match ArrayString::<DATA_MAX_SZ>::from_utf8(data_slice) {
        // XXX: docs suggest using `SmallString` to create an `ArrayString`, somewhat confusing
        let arraystring1 = match arraystring::SmallString::from_utf8(data_slice) {
            | Ok(val) => val,
            | Err(_) => {
                continue; // invalid UTF8
            }
        };
        let s1 = arraystring1.to_string();
        black_box(s1);
        count += 1;
    }
    assert_eq!(count, COUNT_VALID_UTF8_IN_DATAS, "FAILED TO CATCH INVALID UTF8");
}

#[inline(never)]
fn dutf8_arraystring__CacheString_from_utf8() {
    // use arraystring::ArrayString (as_bstr)
    let mut count: u32 = 0;
    for data in Datas.iter() {
        let data_slice = data.as_slice();
        let cachestring1 = match arraystring::CacheString::from_utf8(data_slice) {
            | Ok(val) => val,
            | Err(_) => {
                continue; // invalid UTF8 or too large size
            }
        };
        let s1 = cachestring1.to_string();
        black_box(s1);
        count += 1;
    }
    assert_eq!(count, COUNT_VALID_UTF8_IN_DATAS, "FAILED TO CATCH INVALID UTF8");
}

/*
#[inline(never)]
fn dutf8_jetscii() {
    // use jetscii::Bytes
    let mut count: u32 = 0;
    for data in Datas.iter() {
        let data_slice = data.as_slice();
        let bytes_ = match bytes!(data_slice) {
            Ok(val) => val,
            Err(_) => {
                continue;  // invalid UTF8 or too large size
            }
        };
        let s1 = bytes_.to_string();
        black_box(s1);
        count += 1;
    };
    assert_eq!(count, COUNT_VALID_UTF8_IN_DATAS, "FAILED TO CATCH INVALID UTF8");
}
*/

// TODO: try https://docs.rs/memchr/latest/memchr/fn.memchr.html

/*
// using crate bstr
let dts = &data_slice.as_bstr();
*/

// criterion runners

fn criterion_benchmark(c: &mut Criterion) {
    Datas_check();
    let mut bg = c.benchmark_group("decode utf8");
    bg.bench_function("dutf8_baseline_no_decoding", |b| b.iter(dutf8_baseline_no_decoding));
    bg.bench_function("dutf8_encodingrs_decode_to_string", |b| {
        b.iter(dutf8_encodingrs_decode_to_string)
    });
    bg.bench_function("dutf8_encodingrs_decode_to_string_without_replacement", |b| {
        b.iter(dutf8_encodingrs_decode_to_string_without_replacement)
    });
    bg.bench_function(
        "dutf8_encodingrs_mem_utf8_latin1_up_to__std_str_from_utf8_unchecked",
        |b| b.iter(dutf8_encodingrs_mem_utf8_latin1_up_to__std_str_from_utf8_unchecked),
    );
    bg.bench_function("dutf8_std_str_from_utf8", |b| b.iter(dutf8_std_str_from_utf8));
    bg.bench_function("dutf8_std_str_from_utf8_unchecked__allows_invalid", |b| {
        b.iter(dutf8_std_str_from_utf8_unchecked__allows_invalid)
    });
    bg.bench_function("dutf8_custom_check1_lt0x80__overzealous", |b| {
        b.iter(dutf8_custom_check1_lt0x80__overzealous)
    });
    bg.bench_function(
        "dutf8_custom_check2_lt0x80__fallback__encodingrs_mem_utf8_latin1_up_to__overzealous",
        |b| b.iter(dutf8_custom_check2_lt0x80__fallback__encodingrs_mem_utf8_latin1_up_to__overzealous),
    );
    bg.bench_function(
        "dutf8_custom_check3__is_ascii__fallback__encodingrs_mem_utf8_latin1_up_to__overzealous",
        |b| b.iter(dutf8_custom_check3__is_ascii__fallback__encodingrs_mem_utf8_latin1_up_to__overzealous),
    );
    bg.bench_function("dutf8_bstr_to_str", |b| b.iter(dutf8_bstr_to_str));
    bg.bench_function("dutf8_arraystring__SmallString_from_utf8", |b| {
        b.iter(dutf8_arraystring__SmallString_from_utf8)
    });
    bg.bench_function("dutf8_arraystring__CacheString_from_utf8", |b| {
        b.iter(dutf8_arraystring__CacheString_from_utf8)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
