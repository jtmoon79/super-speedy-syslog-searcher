// src/tests/utmp_tests.rs
// …

//! tests for `utmp.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::common::FileOffset;
use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    ymdhms,
    ymdhmsm,
};
use crate::data::utmpx::{
    buffer_to_utmpx,
    convert_tvsec_utvcsec_datetime,
    determine_type_utmpx,
    InfoAsBytes,
    linux_gnu::UTMPX_SZ as linux_gnu_UTMPX_SZ,
    UTMPX_SZ_MAX,
    Utmpx,
    UtmpxDynPtr,
    UtmpxType,
    tv_sec_type,
    tv_usec_type,
};
use crate::debug::printers::buffer_to_String_noraw;
use crate::readers::blockreader::{BlockOffset, BlockSz};
use crate::tests::common::{
    FO_0,
    UTMPX_BUFFER1,
    UTMPX_BUFFER2,
};

use std::mem::size_of_val;
use std::str; // for `from_utf8`

use ::chrono::FixedOffset;
use ::lazy_static::lazy_static;
use ::test_case::test_case;
#[allow(unused_imports)]
use ::more_asserts::{
    assert_ge,
    assert_gt,
    assert_le,
    assert_lt,
    debug_assert_ge,
    debug_assert_gt,
    debug_assert_le,
    debug_assert_lt,
};
#[allow(unused_imports)]
use ::si_trace_print::{
    defn,
    defo,
    defx,
    defñ,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// arbitrary fileoffset for `UTMPX2`
const UTMPX2_FO2: FileOffset = linux_gnu_UTMPX_SZ as FileOffset;

lazy_static! {
    static ref UTMPX2_: UtmpxDynPtr = {
        buffer_to_utmpx(&UTMPX_BUFFER2, Some(UtmpxType::LinuxGnu)).unwrap()
    };
    static ref UTMPX2: Utmpx = {
        Utmpx::new(UTMPX2_FO2, &FO_0, &UTMPX_BUFFER2, Some(UtmpxType::LinuxGnu)).unwrap()
    };
    static ref UTMPX2_DT: DateTimeL = {
        ymdhmsm(
            &FO_0,
            2020,
            1,
            1,
            12,
            0,
            2,
            123636,
        )
    };
    static ref UTMPX2_STRING_NORAW: String = {
        UTMPX2.to_String_noraw()
    };
}

#[test]
fn test_linux_gnu_UTMPX_SZ() {
    assert_eq!(linux_gnu_UTMPX_SZ, 384);
}

#[test_case(
    1893543133, 999999, *FO_0,
    Some(ymdhmsm(&FO_0, 2030, 1, 2, 0, 12, 13, 999999)))
]
#[test_case(
    1677566475, 1345, *FO_0,
    Some(ymdhmsm(&FO_0, 2023, 2, 28, 6, 41, 15, 1345)))
]
#[test_case(
    0, 0, *FO_0,
    Some(ymdhms(&FO_0, 1970, 1, 1, 0, 0, 0)))
]
#[test_case(tv_sec_type::MAX, tv_usec_type::MAX, *FO_0, None)]
#[test_case(tv_sec_type::MIN, tv_usec_type::MIN, *FO_0, None)]
fn test_convert_tvsec_utvcsec_datetime(
    tv_sec: tv_sec_type,
    tv_usec: tv_usec_type,
    fo: FixedOffset,
    expect_dt: DateTimeLOpt,
) {
    let dt1 = convert_tvsec_utvcsec_datetime(tv_sec, tv_usec, &fo);
    match expect_dt {
        Some(val) => {
            assert_eq!(dt1.unwrap(), val,
                "convert_tvsec_utvcsec_datetime returned {:?}, expected {:?}",
                dt1,
                expect_dt,
            );
        },
        None => {
            assert!(dt1.is_none(),
                "convert_tvsec_utvcsec_datetime returned {:?}, expected None",
                dt1,
            );
        },
    }
}

#[test]
fn test_utmpx_new_0() {
    match buffer_to_utmpx(&[0; linux_gnu_UTMPX_SZ], Some(UtmpxType::LinuxGnu)) {
        Some(val) => val,
        None => {
            panic!("ERROR: buffer_to_utmpx failed");
        }
    };
}

#[test]
fn test_utmpx_new_toosmall() {
    // pass one byte buffer, should return `None`
    if buffer_to_utmpx(&[0; 1], Some(UtmpxType::LinuxGnu)).is_some() {
        panic!("ERROR: buffer_to_utmpx should have failed")
    }
}

#[test_case(&UTMPX_BUFFER1, Some(UtmpxType::LinuxGnu))]
#[test_case(&UTMPX_BUFFER2, Some(UtmpxType::LinuxGnu))]
fn test_determine_type_utmpx(
    buffer: &[u8],
    expected: Option<UtmpxType>,
) {
    defo!("buffer len: {}", buffer.len());
    let utmpx_type = determine_type_utmpx(buffer);
    assert_eq!(utmpx_type, expected, "determine_type_utmpx");
}

#[test]
fn test_Utmpx_new_0() {
    Utmpx::new(0, &FO_0, &[0; linux_gnu_UTMPX_SZ], Some(UtmpxType::LinuxGnu));
}

/// helper to `test_Utmpx_new1`
///
/// create a `Vec<T>` copying from slice `data`.
/// Returned `Vec<T>` will have total length `len`. This may require
/// resizes with `T::default()` value or truncating to `len`.
fn create_vec_from<T: Copy + Default>(data: &[T], len: usize) -> Vec<T>
{
    let mut vt: Vec<T> = Vec::<T>::with_capacity(len);
    for t in data.iter() {
        vt.push(*t);
    }
    match vt.len().cmp(&len) {
        std::cmp::Ordering::Less => {
            vt.resize_with(len, || T::default());
        }
        std::cmp::Ordering::Greater => {
            vt.truncate(len);
        }
        _ => {}
    }

    vt
}

#[test]
fn test_Utmpx_new1() {
    let utmpx_s = buffer_to_String_noraw(&UTMPX_BUFFER1);
    eprintln!("UTMPX_BUFFER1: {}", utmpx_s);
    let entry = Utmpx::new(0, &FO_0, &UTMPX_BUFFER1, Some(UtmpxType::LinuxGnu)).unwrap();
    eprintln!("UTMPX1: {}", entry.to_String_raw());

    assert_eq!(entry.entry.ut_type(), 5, "ut_type");
    assert_eq!(entry.entry.ut_pid(), 41908, "ut_pid");

    let mut ut_line_expect: Vec<i8> = create_vec_from(
        &[b'p' as i8, b't' as i8, b's' as i8, b'/' as i8, b'1' as i8],
        size_of_val(&entry.entry.ut_line())
    );
    ut_line_expect.resize_with(size_of_val(&entry.entry.ut_line()), || 0);
    assert_eq!(entry.entry.ut_line(), ut_line_expect.as_slice(), "ut_line");

    assert_eq!(
        entry.entry.ut_id(),
        [b't' as i8, b's' as i8, b'/' as i8, b'1' as i8],
        "ut_id"
    );

    let ut_user_expect: Vec<i8> = create_vec_from(
        &[b'a' as i8, b'd' as i8, b'm' as i8, b'i' as i8, b'n' as i8],
        size_of_val(&entry.entry.ut_user())
    );
    assert_eq!(entry.entry.ut_user(), ut_user_expect.as_slice(), "ut_user");

    let ut_host_expect: Vec<i8> = create_vec_from(
        &[
            b'1' as i8,
            b'9' as i8,
            b'2' as i8,
            b'.' as i8,
            b'1' as i8,
            b'6' as i8,
            b'8' as i8,
            b'.' as i8,
            b'1' as i8,
            b'.' as i8,
            b'5' as i8,
        ],
        size_of_val(&entry.entry.ut_host())
    );
    assert_eq!(entry.entry.ut_host(), ut_host_expect.as_slice(), "ut_host");
    assert_eq!(entry.entry.ut_exit_e_termination(), 7, "ut_exit.e_termination");
    assert_eq!(entry.entry.ut_exit_e_exit(), 1, "ut_exit.e_exit");
    assert_eq!(entry.entry.ut_session(), 0, "ut_session");
    assert_eq!(entry.entry.ut_tv_tv_sec(), 1577880000, "ut_tv.tv_sec");
    assert_eq!(entry.entry.ut_tv_tv_usec(), 0, "ut_tv.tv_usec");
    assert_eq!(entry.entry.ut_addr_v6(), [0x2F7CA8D0, 0, 0, 0], "ut_addr_v6");
}

#[test]
fn test_Utmpx_new2() {
    let utmpx_s = buffer_to_String_noraw(&UTMPX_BUFFER2);
    eprintln!("UTMPX_BUFFER2: {}", utmpx_s);
    let entry = Utmpx::new(0, &FO_0, &UTMPX_BUFFER2, Some(UtmpxType::LinuxGnu)).unwrap();
    eprintln!("UTMPX2: {}", entry.to_String_raw());

    assert_eq!(entry.entry.ut_type(), 7, "ut_type");
    assert_eq!(entry.entry.ut_pid(), 13236, "ut_pid");

    let mut ut_line_expect: Vec<i8> = create_vec_from(
        &[b'p' as i8, b't' as i8, b's' as i8, b'/' as i8, b'0' as i8],
        size_of_val(&entry.entry.ut_line())
    );
    ut_line_expect.resize_with(size_of_val(&entry.entry.ut_line()), || 0);
    assert_eq!(entry.entry.ut_line(), ut_line_expect.as_slice(), "ut_line");

    assert_eq!(
        entry.entry.ut_id(),
        [b't' as i8, b's' as i8, b'/' as i8, b'0' as i8],
        "ut_id"
    );

    let ut_user_expect: Vec<i8> = create_vec_from(
        &[b'r' as i8, b'o' as i8, b'o' as i8, b't' as i8],
        size_of_val(&entry.entry.ut_user())
    );
    assert_eq!(entry.entry.ut_user(), ut_user_expect.as_slice(), "ut_user");

    let ut_host_expect: Vec<i8> = create_vec_from(
        &[
            b'1' as i8,
            b'9' as i8,
            b'2' as i8,
            b'.' as i8,
            b'1' as i8,
            b'6' as i8,
            b'8' as i8,
            b'.' as i8,
            b'1' as i8,
            b'.' as i8,
            b'4' as i8,
        ],
        size_of_val(&entry.entry.ut_host())
    );
    assert_eq!(entry.entry.ut_host(), ut_host_expect.as_slice(), "ut_host");
    assert_eq!(entry.entry.ut_exit_e_termination(), 7, "ut_exit.e_termination");
    assert_eq!(entry.entry.ut_exit_e_exit(), 3, "ut_exit.e_exit");
    assert_eq!(entry.entry.ut_session(), 5, "ut_session");
    assert_eq!(entry.entry.ut_tv_tv_sec(), 1577880002, "ut_tv.tv_sec");
    assert_eq!(entry.entry.ut_tv_tv_usec(), 123636, "ut_tv.tv_usec");
    assert_eq!(entry.entry.ut_addr_v6(), [0x2F7CA8C0, 0, 0, 0], "ut_addr_v6");
}

#[test]
fn test_Utmpx_helpers() {
    const BSZ20: BlockSz = 20;
    const BSZ_U: usize = BSZ20 as usize;
    assert_eq!(
        UTMPX2.blockoffset_begin(BSZ20),
        ((UTMPX2_FO2 as usize) / BSZ_U) as BlockOffset,
        "blockoffset_begin"
    );
    let sz_fo: FileOffset = linux_gnu_UTMPX_SZ as FileOffset;
    assert_eq!(
        UTMPX2.blockoffset_end(BSZ20),
        (((sz_fo + UTMPX2_FO2) as usize) / BSZ_U) as BlockOffset,
        "blockoffset_end"
    );
    assert_eq!(UTMPX2.fileoffset_begin(), UTMPX2_FO2, "fileoffset_begin");
    assert_eq!(UTMPX2.fileoffset_end(), sz_fo + UTMPX2_FO2, "fileoffset_end");
}

#[test]
fn test_Utmpx_dt() {
    assert_eq!(UTMPX2.dt(), &*UTMPX2_DT, "dt");
}

#[test]
fn test_Utmpx_as_bytes() {
    eprintln!("UTMPX2: {}", *UTMPX2_STRING_NORAW);
    let mut buffer: [u8; UTMPX_SZ_MAX * 2] = [0; UTMPX_SZ_MAX * 2];
    let info: InfoAsBytes = UTMPX2.as_bytes(&mut buffer);
     // make broad approximate asserts on returned values
    match info {
        InfoAsBytes::Ok(at, beg, end) => {
            assert_gt!(at, 100, "at");
            assert_gt!(beg, 100, "beg");
            assert_lt!(beg, end, "beg end");
            assert_lt!(end, 200, "end");
        }
        InfoAsBytes::Fail(at) => {
            panic!("ERROR: as_bytes failed: {:?}", at);
        }
    }
    assert_gt!(buffer.len(), 100, "as_bytes");
    // can it convert to a str?
    let s: &str = str::from_utf8(&buffer).unwrap();
    eprintln!("buffer: {}", s);
}
