// src/tests/fixedstruct_tests.rs
// …

//! tests for `fixedstruct.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::ffi::CString;
use std::str; // for `from_utf8`

use ::chrono::FixedOffset;
use ::lazy_static::lazy_static;
use ::more_asserts::{
    assert_gt,
    assert_lt,
};
#[allow(unused_imports)]
use ::si_trace_print::{
    defn,
    defo,
    defx,
    defñ,
};
use ::test_case::test_case;

use crate::common::FileOffset;
use crate::data::datetime::{
    ymdhms,
    ymdhmsm,
    DateTimeL,
    DateTimeLOpt,
};
use crate::data::fixedstruct::{
    buffer_to_fixedstructptr,
    convert_datetime_tvpair,
    convert_tvpair_to_datetime,
    freebsd_x8664,
    linux_arm64aarch64,
    linux_x86,
    netbsd_x8632,
    netbsd_x8664,
    openbsd_x86,
    tv_pair_type,
    tv_sec_type,
    tv_usec_type,
    FixedStruct,
    FixedStructDynPtr,
    FixedStructType,
    InfoAsBytes,
    Score,
    ENTRY_SZ_MAX,
};
use crate::readers::blockreader::{
    BlockOffset,
    BlockSz,
};
use crate::tests::common::{
    FO_0,
    FREEBSD_X8664_UTMPX_BUFFER1,
    LINUX_ARM64AARCH64_LASTLOG_BUFFER1,
    LINUX_ARM64AARCH64_UTMPX_BUFFER1,
    LINUX_X86_ACCT_BUFFER1,
    LINUX_X86_ACCT_V3_BUFFER1,
    LINUX_X86_LASTLOG_BUFFER1,
    LINUX_X86_UTMPX_BUFFER1,
    LINUX_X86_UTMPX_BUFFER2,
    LINUX_X86_UTMPX_BUFFER_00,
    LINUX_X86_UTMPX_BUFFER_FF,
    NETBSD_X8632_ACCT_BUFFER1,
    NETBSD_X8632_LASTLOGX_BUFFER1,
    NETBSD_X8632_UTMPX_BUFFER1,
    NETBSD_X8664_LASTLOGX_BUFFER1,
    NETBSD_X8664_LASTLOG_BUFFER1,
    NETBSD_X8664_UTMPX_BUFFER1,
    NETBSD_X8664_UTMP_BUFFER1,
    OPENBSD_X86_LASTLOG_BUFFER1,
    OPENBSD_X86_UTMP_BUFFER1,
};

/// fileoffset for `UTMPX2`
const UTMPX2_FO2: FileOffset = linux_x86::UTMPX_SZ_FO;

lazy_static! {
    static ref UTMPX2_PTR: FixedStructDynPtr = {
        buffer_to_fixedstructptr(
            &LINUX_X86_UTMPX_BUFFER2,
            FixedStructType::Fs_Linux_x86_Utmpx,
        ).unwrap()
    };
    static ref UTMPX2: FixedStruct = {
        FixedStruct::new(
            UTMPX2_FO2,
            &FO_0,
            &LINUX_X86_UTMPX_BUFFER2,
            FixedStructType::Fs_Linux_x86_Utmpx,
        ).unwrap()
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

    static ref LINUX_X86_LASTLOG_1ENTRY_PTR: FixedStructDynPtr = {
        buffer_to_fixedstructptr(
            &LINUX_X86_LASTLOG_BUFFER1,
            FixedStructType::Fs_Linux_x86_Lastlog,
        ).unwrap()
    };
}

#[test_case(
    ymdhmsm(&FO_0, 2030, 1, 2, 0, 12, 13, 999999), 1893543133, 999999
)]
#[test_case(
    ymdhmsm(&FO_0, 2023, 2, 28, 6, 41, 15, 1345), 1677566475, 1345
)]
#[test_case(
    ymdhms(&FO_0, 1970, 1, 1, 0, 0, 0), 0, 0
)]
fn test_convert_convert_datetime_tvpair(
    dt: DateTimeL,
    expect_tv_sec: tv_sec_type,
    expect_tv_usec: tv_usec_type,
) {
    defn!("test_convert_datetime_tvpair({})", &dt);
    let tv_pair_ = convert_datetime_tvpair(&dt);
    assert_eq!(
        tv_pair_.0, expect_tv_sec,
        "tv_sec actual {} expected {}", tv_pair_.0, expect_tv_sec
    );
    assert_eq!(
        tv_pair_.1, expect_tv_usec,
        "tv_usec actual {} expected {}", tv_pair_.1, expect_tv_usec
    );
    defx!();
}

#[test_case(
    1893543133, 999999, FO_0,
    Some(ymdhmsm(&FO_0, 2030, 1, 2, 0, 12, 13, 999999)))
]
#[test_case(
    1677566475, 1345, FO_0,
    Some(ymdhmsm(&FO_0, 2023, 2, 28, 6, 41, 15, 1345)))
]
#[test_case(
    0, 0, FO_0,
    Some(ymdhms(&FO_0, 1970, 1, 1, 0, 0, 0)))
]
#[test_case(tv_sec_type::MAX, tv_usec_type::MAX, FO_0, None)]
#[test_case(tv_sec_type::MIN, tv_usec_type::MIN, FO_0, None)]
fn test_convert_tvpair_to_datetime(
    tv_sec: tv_sec_type,
    tv_usec: tv_usec_type,
    fo: FixedOffset,
    expect_dt: DateTimeLOpt,
) {
    defn!("test_convert_tvpair_to_datetime(tv_sec = {}, tv_usec = {}, …)", tv_sec, tv_usec);
    let dt_actual = convert_tvpair_to_datetime(
        tv_pair_type(tv_sec, tv_usec),
        &fo,
    );
    defo!("dt_actual = {:?}", dt_actual);
    match expect_dt {
        Some(expect_val) => {
            let d_val = dt_actual.unwrap();
            assert_eq!(d_val, expect_val,
                "convert_tvpair_to_datetime returned {:?}, expected {:?}",
                d_val,
                expect_val,
            );
        },
        None => {
            assert!(dt_actual.is_err(),
                "convert_tvpair_to_datetime result {:?}, expected Err",
                dt_actual,
            );
        },
    }
    defx!();
}

#[test]
fn test_fixedstructptr_new_00() {
    if buffer_to_fixedstructptr(
        &[0; linux_x86::UTMPX_SZ],
        FixedStructType::Fs_Linux_x86_Utmpx,
    ).is_some() {
        panic!("passed 0x00 bytes, should have failed");
    };
}

#[test]
fn test_fixedstructptr_new_FF() {
    if buffer_to_fixedstructptr(
        &[0xFF; linux_x86::UTMPX_SZ],
        FixedStructType::Fs_Linux_x86_Utmpx,
    ).is_some() {
        panic!("passed 0xFF bytes, should have failed");
    };
}

#[test]
fn test_fixedstructptr_new_toosmall() {
    if buffer_to_fixedstructptr(
        &[0; 1],
        FixedStructType::Fs_Linux_x86_Utmpx,
    ).is_some() {
        panic!("passed 1 byte, should have failed");
    }
}

#[test]
fn test_FixedStruct_new_00() {
    if FixedStruct::new(
        0,
        &FO_0,
        &[0; linux_x86::UTMPX_SZ],
        FixedStructType::Fs_Linux_x86_Utmpx,
    ).is_ok() {
        panic!("passed 0x00 bytes, should have failed to create FixedStruct");
    }
}

#[test]
fn test_FixedStruct_new_FF() {
    if FixedStruct::new(
        0,
        &FO_0,
        &[0xFF; linux_x86::UTMPX_SZ],
        FixedStructType::Fs_Linux_x86_Utmpx,
    ).is_ok() {
        panic!("passed 0xFF bytes, should have failed to create FixedStruct");
    }
}

#[test]
fn test_FixedStruct_helpers() {
    const BSZ20: BlockSz = 20;
    const BSZ_U: usize = BSZ20 as usize;
    assert_eq!(
        UTMPX2.blockoffset_begin(BSZ20),
        ((UTMPX2_FO2 as usize) / BSZ_U) as BlockOffset,
        "blockoffset_begin"
    );
    let sz_fo: FileOffset = linux_x86::UTMPX_SZ as FileOffset;
    assert_eq!(
        UTMPX2.blockoffset_end(BSZ20),
        (((sz_fo + UTMPX2_FO2) as usize) / BSZ_U) as BlockOffset,
        "blockoffset_end"
    );
    assert_eq!(UTMPX2.fileoffset_begin(), UTMPX2_FO2, "fileoffset_begin");
    assert_eq!(UTMPX2.fileoffset_end(), sz_fo + UTMPX2_FO2, "fileoffset_end");
    assert_eq!(UTMPX2.tv_pair(), &tv_pair_type(1577880002, 123636), "tv_pair");
}

#[test]
fn test_FixedStruct_dt() {
    assert_eq!(UTMPX2.dt(), &*UTMPX2_DT, "dt");
}

#[test]
fn test_FixedStruct_as_bytes() {
    eprintln!("UTMPX2: {}", *UTMPX2_STRING_NORAW);
    let mut buffer: [u8; ENTRY_SZ_MAX * 2] = [0; ENTRY_SZ_MAX * 2];
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

#[test]
fn test_buffer_to_fixedstructptr_00() {
    let entry = buffer_to_fixedstructptr(
        &LINUX_X86_UTMPX_BUFFER_00,
        FixedStructType::Fs_Linux_x86_Utmpx,
    );
    assert!(entry.is_none(), "buffer_to_fixedstructptr 0x00");
}

#[test]
fn test_buffer_to_fixedstructptr_FF() {
    let entry = buffer_to_fixedstructptr(
        &LINUX_X86_UTMPX_BUFFER_FF,
        FixedStructType::Fs_Linux_x86_Utmpx,
    );
    assert!(entry.is_none(), "buffer_to_fixedstructptr 0xFF");
}

#[test_case(&FREEBSD_X8664_UTMPX_BUFFER1, FixedStructType::Fs_Freebsd_x8664_Utmpx)]
#[test_case(&LINUX_ARM64AARCH64_LASTLOG_BUFFER1, FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog)]
#[test_case(&LINUX_ARM64AARCH64_UTMPX_BUFFER1, FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx)]
#[test_case(&LINUX_X86_ACCT_BUFFER1, FixedStructType::Fs_Linux_x86_Acct)]
#[test_case(&LINUX_X86_ACCT_V3_BUFFER1, FixedStructType::Fs_Linux_x86_Acct_v3)]
#[test_case(&LINUX_X86_LASTLOG_BUFFER1, FixedStructType::Fs_Linux_x86_Lastlog)]
#[test_case(&LINUX_X86_UTMPX_BUFFER1, FixedStructType::Fs_Linux_x86_Utmpx)]
#[test_case(&NETBSD_X8632_ACCT_BUFFER1, FixedStructType::Fs_Netbsd_x8632_Acct)]
#[test_case(&NETBSD_X8632_LASTLOGX_BUFFER1, FixedStructType::Fs_Netbsd_x8632_Lastlogx)]
#[test_case(&NETBSD_X8632_UTMPX_BUFFER1, FixedStructType::Fs_Netbsd_x8632_Utmpx)]
#[test_case(&NETBSD_X8664_LASTLOG_BUFFER1, FixedStructType::Fs_Netbsd_x8664_Lastlog)]
#[test_case(&NETBSD_X8664_LASTLOGX_BUFFER1, FixedStructType::Fs_Netbsd_x8664_Lastlogx)]
#[test_case(&NETBSD_X8664_UTMP_BUFFER1, FixedStructType::Fs_Netbsd_x8664_Utmp)]
#[test_case(&NETBSD_X8664_UTMPX_BUFFER1, FixedStructType::Fs_Netbsd_x8664_Utmpx)]
#[test_case(&OPENBSD_X86_LASTLOG_BUFFER1, FixedStructType::Fs_Openbsd_x86_Lastlog)]
#[test_case(&OPENBSD_X86_UTMP_BUFFER1, FixedStructType::Fs_Openbsd_x86_Utmp)]
fn test_buffer_to_fixedstructptr(
    buffer: &[u8],
    fixedstructtype: FixedStructType,
) {
    let entry = buffer_to_fixedstructptr(
        buffer,
        fixedstructtype,
    );
    assert!(
        entry.is_some(),
        "buffer_to_fixedstructptr failed to create {:?}, buffer size {}",
        fixedstructtype, buffer.len(),
    );
}

#[test_case(&FREEBSD_X8664_UTMPX_BUFFER1, FixedStructType::Fs_Freebsd_x8664_Utmpx, 0, -22)]
#[test_case(&FREEBSD_X8664_UTMPX_BUFFER1, FixedStructType::Fs_Freebsd_x8664_Utmpx, 10, -12)]
#[test_case(&LINUX_ARM64AARCH64_LASTLOG_BUFFER1, FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog, 0, 66)]
#[test_case(&LINUX_ARM64AARCH64_UTMPX_BUFFER1, FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx, 0, 122)]
#[test_case(&LINUX_X86_ACCT_BUFFER1, FixedStructType::Fs_Linux_x86_Acct, 0, 48)]
#[test_case(&LINUX_X86_ACCT_V3_BUFFER1, FixedStructType::Fs_Linux_x86_Acct_v3, 0, -75)]
#[test_case(&LINUX_X86_LASTLOG_BUFFER1, FixedStructType::Fs_Linux_x86_Lastlog, 0, 60)]
#[test_case(&LINUX_X86_UTMPX_BUFFER1, FixedStructType::Fs_Linux_x86_Utmpx, 0, 129)]
#[test_case(&NETBSD_X8632_ACCT_BUFFER1, FixedStructType::Fs_Netbsd_x8632_Acct, 0, 58)]
#[test_case(&NETBSD_X8632_LASTLOGX_BUFFER1, FixedStructType::Fs_Netbsd_x8632_Lastlogx, 0, 72)]
#[test_case(&NETBSD_X8632_UTMPX_BUFFER1, FixedStructType::Fs_Netbsd_x8632_Utmpx, 0, 140)]
#[test_case(&NETBSD_X8664_LASTLOG_BUFFER1, FixedStructType::Fs_Netbsd_x8664_Lastlog, 0, 62)]
#[test_case(&NETBSD_X8664_LASTLOGX_BUFFER1, FixedStructType::Fs_Netbsd_x8664_Lastlogx, 0, -20)]
#[test_case(&NETBSD_X8664_UTMP_BUFFER1, FixedStructType::Fs_Netbsd_x8664_Utmp, 0, 81)]
#[test_case(&NETBSD_X8664_UTMPX_BUFFER1, FixedStructType::Fs_Netbsd_x8664_Utmpx, 0, 135)]
#[test_case(&OPENBSD_X86_LASTLOG_BUFFER1, FixedStructType::Fs_Openbsd_x86_Lastlog, 0, 72)]
#[test_case(&OPENBSD_X86_UTMP_BUFFER1, FixedStructType::Fs_Openbsd_x86_Utmp, 0, 81)]
fn test_score_fixedstruct(
    buffer: &[u8],
    fixedstructtype: FixedStructType,
    bonus: Score,
    score_expect: Score,
) {
    let ptr = buffer_to_fixedstructptr(
        buffer,
        fixedstructtype,
    ).unwrap();
    let score = FixedStruct::score_fixedstruct(&ptr, bonus);
    assert_eq!(
        score, score_expect,
        "score_fixedstruct, got {}, expected {}, for {:?}",
        score, score_expect, fixedstructtype,
    );
}

// individual FixedStruct tests

#[test]
fn test_freebsd_x8664_utmpx() {
    // TODO: [2024/03/10] need valid data for `FREEBSD_X8664_UTMPX_BUFFER1`
    let entry = buffer_to_fixedstructptr(
        &FREEBSD_X8664_UTMPX_BUFFER1,
        FixedStructType::Fs_Freebsd_x8664_Utmpx,
    ).unwrap();
    let utmpx: &freebsd_x8664::utmpx = entry.as_freebsd_x8664_utmpx();

    assert_eq!(utmpx.ut_type, 3, "ut_type");
    assert_eq!(utmpx.ut_user(), CString::new("A").unwrap().as_c_str(), "ut_user");

    eprintln!("freebsd_x8664::utmpx: {:?}", utmpx);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, -22, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_linux_arm64aarch64_lastlog() {
    let entry = buffer_to_fixedstructptr(
        &LINUX_ARM64AARCH64_LASTLOG_BUFFER1,
        FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog,
    ).unwrap();
    let lastlog: &linux_arm64aarch64::lastlog = entry.as_linux_arm64aarch64_lastlog();
    assert_eq!(lastlog.ll_time, 1708204125, "ll_time");
    assert_eq!(lastlog.ll_line(), CString::new("pts/0").unwrap().as_c_str(), "ll_line");
    assert_eq!(lastlog.ll_host(), CString::new("67.184.33.88").unwrap().as_c_str(), "ll_host");

    eprintln!("linux_arm64aarch64::lastlog: {:?}", lastlog);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 66, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_linux_arm64aarch64_utmpx() {
    let entry = buffer_to_fixedstructptr(
        &LINUX_ARM64AARCH64_UTMPX_BUFFER1,
        FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx,
    ).unwrap();
    let utmpx: &linux_arm64aarch64::utmpx = entry.as_linux_arm64aarch64_utmpx();
    assert_eq!(utmpx.ut_type, 2, "ut_type");
    assert_eq!(utmpx.ut_pid, 0, "ut_pid");
    assert_eq!(utmpx.ut_line(), CString::new("~").unwrap().as_c_str(), "ut_line");
    assert_eq!(utmpx.ut_id, [b'~' as i8, b'~' as i8, 0, 0], "ut_id");
    assert_eq!(utmpx.ut_user(), CString::new("reboot").unwrap().as_c_str(), "ut_user");
    assert_eq!(utmpx.ut_host(), CString::new("6.1.63-current-rockchip64").unwrap().as_c_str(), "ut_host");
    assert_eq!(utmpx.ut_exit, 0, "ut_exit");
    assert_eq!(utmpx.ut_session, 0, "ut_session");
    assert_eq!(utmpx.ut_tv.tv_sec, 1702248364, "ut_tv.tv_sec");
    assert_eq!(utmpx.ut_tv.tv_usec, 847969, "ut_tv.tv_usec");
    assert_eq!(utmpx.ut_addr_v6, [0, 0, 0, 0], "ut_addr_v6");

    eprintln!("linux_arm64aarch64::utmpx: {:?}", utmpx);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 122, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_linux_x86_acct() {
    let entry = buffer_to_fixedstructptr(
        &LINUX_X86_ACCT_BUFFER1,
        FixedStructType::Fs_Linux_x86_Acct,
    ).unwrap();
    let acct: &linux_x86::acct = entry.as_linux_x86_acct();
    assert_eq!(acct.ac_flag, 2, "ac_flag");
    assert_eq!(acct.ac_uid, 0, "ac_uid");
    assert_eq!(acct.ac_gid, 0, "ac_gid");
    assert_eq!(acct.ac_tty, 0, "ac_tty");
    assert_eq!(acct.ac_btime, 1709389501, "ac_btime");
    assert_eq!(acct.ac_utime, 0, "ac_utime");
    assert_eq!(acct.ac_stime, 0, "ac_stime");
    assert_eq!(acct.ac_etime, 0, "ac_etime");
    assert_eq!(acct.ac_mem, 2776, "ac_mem");
    assert_eq!(acct.ac_io, 0, "ac_io");
    assert_eq!(acct.ac_rw, 0, "ac_rw");
    assert_eq!(acct.ac_minflt, 189, "ac_minflt");
    assert_eq!(acct.ac_majflt, 0, "ac_majflt");
    assert_eq!(acct.ac_swaps, 0, "ac_swaps");
    assert_eq!(acct.ac_exitcode, 0, "ac_exitcode");
    assert_eq!(acct.ac_comm(), CString::new("accton").unwrap().as_c_str(), "ac_comm");

    eprintln!("linux_x86::acct: {:?}", acct);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 48, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();

    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_linux_86_acct_v3() {
    // TODO: [2024/03/10] `LINUX_X86_ACCT_V3_BUFFER1` is zero data
    let entry = buffer_to_fixedstructptr(
        &LINUX_X86_ACCT_V3_BUFFER1,
        FixedStructType::Fs_Linux_x86_Acct_v3,
    ).unwrap();
    let acct: &linux_x86::acct_v3 = entry.as_linux_x86_acct_v3();
    
    assert_eq!(acct.ac_flag, 1, "ac_flag");
    assert_eq!(acct.ac_uid, 0, "ac_uid");

    eprintln!("linux_x86::acct_v3: {:?}", acct);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, -75, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_linux_86_lastlog() {
    let entry = buffer_to_fixedstructptr(
        &LINUX_X86_LASTLOG_BUFFER1,
        FixedStructType::Fs_Linux_x86_Lastlog,
    ).unwrap();
    let lastlog: &linux_x86::lastlog = entry.as_linux_x86_lastlog();
    assert_eq!(lastlog.ll_time, 1702627821, "ll_time");
    assert_eq!(lastlog.ll_line(), CString::new("pts/1").unwrap().as_c_str(), "ll_line");
    assert_eq!(lastlog.ll_host(), CString::new("localhost").unwrap().as_c_str(), "ll_host");

    eprintln!("linux_x86::lastlog: {:?}", lastlog);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 60, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_linux_x86_utmpx() {
    let entry = buffer_to_fixedstructptr(
        &LINUX_X86_UTMPX_BUFFER1,
        FixedStructType::Fs_Linux_x86_Utmpx,
    ).unwrap();
    let utmpx: &linux_x86::utmpx = entry.as_linux_x86_utmpx();
    assert_eq!(utmpx.ut_type, 5, "ut_type");
    assert_eq!(utmpx.ut_pid, 41908, "ut_pid");
    assert_eq!(utmpx.ut_line(), CString::new("pts/1").unwrap().as_c_str(), "ut_line");
    assert_eq!(utmpx.ut_id, [b't' as i8, b's' as i8, b'/' as i8, b'1' as i8], "ut_id");
    assert_eq!(utmpx.ut_user(), CString::new("admin").unwrap().as_c_str(), "ut_user");
    assert_eq!(utmpx.ut_host(), CString::new("192.168.1.5").unwrap().as_c_str(), "ut_host");
    assert_eq!(utmpx.ut_exit.e_termination, 7, "ut_exit.e_termination");
    assert_eq!(utmpx.ut_exit.e_exit, 1, "ut_exit.e_exit");
    assert_eq!(utmpx.ut_session, 0, "ut_session");
    assert_eq!(utmpx.ut_tv.tv_sec, 1577880000, "ut_tv.tv_sec");
    assert_eq!(utmpx.ut_tv.tv_usec, 0, "ut_tv.tv_usec");
    assert_eq!(utmpx.ut_addr_v6, [0x2F7CA8D0, 0, 0, 0], "ut_addr_v6");

    eprintln!("linux_x86::utmpx: {:?}", utmpx);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 129, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_netbsd_x8632_acct() {
    let entry = buffer_to_fixedstructptr(
        &NETBSD_X8632_ACCT_BUFFER1,
        FixedStructType::Fs_Netbsd_x8632_Acct,
    ).unwrap();
    let acct: &netbsd_x8632::acct = entry.as_netbsd_x8632_acct();
    assert_eq!(acct.ac_comm(), CString::new("accton").unwrap().as_c_str(), "ac_comm");
    let ac_utime = acct.ac_utime;
    let ac_stime = acct.ac_stime;
    let ac_etime = acct.ac_etime;
    let ac_btime = acct.ac_btime;
    let ac_uid = acct.ac_uid;
    let ac_gid = acct.ac_gid;
    let ac_mem = acct.ac_mem;
    let ac_io = acct.ac_io;
    let ac_tty = acct.ac_tty;
    let ac_flag = acct.ac_flag;
    assert_eq!(ac_utime, 0, "ac_utime");
    assert_eq!(ac_stime, 0, "ac_stime");
    assert_eq!(ac_etime, 0, "ac_etime");
    assert_eq!(ac_btime, 1710109648, "ac_btime");
    assert_eq!(ac_uid, 0, "ac_uid");
    assert_eq!(ac_gid, 0, "ac_gid");
    assert_eq!(ac_mem, 0, "ac_mem");
    assert_eq!(ac_io, 0, "ac_io");
    assert_eq!(ac_tty, 1282, "ac_tty"); // XXX: suspicious vale
    assert_eq!(ac_flag, 2, "ac_flag");

    eprintln!("netbsd_x8632::acct: {:?}", acct);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 58, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_netbsd_x8632_lastlogx() {
    let entry = buffer_to_fixedstructptr(
        &NETBSD_X8632_LASTLOGX_BUFFER1,
        FixedStructType::Fs_Netbsd_x8632_Lastlogx,
    ).unwrap();
    let lastlogx: &netbsd_x8632::lastlogx = entry.as_netbsd_x8632_lastlogx();
    let tv_sec = lastlogx.ll_tv.tv_sec;
    let tv_usec = lastlogx.ll_tv.tv_usec;
    assert_eq!(tv_sec, 1708848961, "ll_tv.tv_sec");
    assert_eq!(tv_usec, 276757, "ll_tv.tv_usec");
    assert_eq!(lastlogx.ll_line(), CString::new("pts/2").unwrap().as_c_str(), "ll_line");
    assert_eq!(lastlogx.ll_host(), CString::new("192.168.100.254").unwrap().as_c_str(), "ll_host");
    const LL_SS: [u8; netbsd_x8632::UTX_SSSIZE] = [
        0x10, 0x02, 0xc0, 0xe4, 0xc0, 0xa8, 0x7c, 0xb4, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc8, 0x20, 0x73, 0xb3,
        0x10, 0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x50,
        0xe4, 0xb3, 0xc8, 0x20, 0x73, 0xb3, 0x30, 0x00, 0x00, 0x00,
        0x74, 0x20, 0x73, 0xb3, 0x42, 0xc7, 0xcd, 0xb3, 0xf5, 0x2c,
        0xcf, 0xb3, 0x30, 0x00, 0x00, 0x00, 0x40, 0x16, 0x74, 0xb3,
        0x00, 0x50, 0xe4, 0xb3, 0x01, 0x00, 0x00, 0x00, 0x66, 0xc9,
        0xce, 0xb3, 0x68, 0x57, 0xb7, 0xbf, 0x98, 0x57, 0xb7, 0xbf,
        0x30, 0x00, 0x00, 0x00, 0x00, 0xb3, 0x6d, 0xb3, 0x01, 0x00,
        0x00, 0x00, 0x9f, 0x54, 0xcf, 0xb3, 0x00, 0x50, 0xe4, 0xb3,
        0x05, 0x00, 0x00, 0x00, 0xd1, 0x55, 0xcf, 0xb3, 0x2d, 0xaa,
        0xce, 0xb3, 0x84, 0x20, 0x73, 0xb3, 0x00, 0x01, 0x00, 0x00,
        0x00, 0xb4, 0x6d, 0xb3, 0x30, 0x00, 0x00, 0x00,
    ];
    assert_eq!(lastlogx.ll_ss, LL_SS, "ll_ss");

    eprintln!("netbsd_x8632::lastlogx: {:?}", lastlogx);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 72, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_netbsd_x8632_utmpx() {
    let entry = buffer_to_fixedstructptr(
        &NETBSD_X8632_UTMPX_BUFFER1,
        FixedStructType::Fs_Netbsd_x8632_Utmpx,
    ).unwrap();
    let utmpx: &netbsd_x8632::utmpx = entry.as_netbsd_x8632_utmpx();
    assert_eq!(utmpx.ut_name(), CString::new("/usr/libexec/getty").unwrap().as_c_str(), "ut_name");
    // BUG: `ut_id` and goes to it's end, need to fix the wrapper functions
    //      see Issue #MNO
    assert_eq!(utmpx.ut_id(), CString::new("sttyconstty").unwrap().as_c_str(), "ut_id");
    assert_eq!(utmpx.ut_line(), CString::new("constty").unwrap().as_c_str(), "ut_line");
    assert_eq!(utmpx.ut_host(), CString::new("").unwrap().as_c_str(), "ut_host");
    assert_eq!(utmpx.ut_session, 2, "ut_session");
    assert_eq!(utmpx.ut_type, 6, "ut_type");
    assert_eq!(utmpx.ut_pid, 651, "ut_pid");
    assert_eq!(utmpx.ut_exit.e_termination, 0, "ut_exit.e_termination");
    assert_eq!(utmpx.ut_exit.e_exit, 0, "ut_exit.e_exit");
    let tv_sec = utmpx.ut_tv.tv_sec;
    assert_eq!(tv_sec, 1708848925, "ut_tv.tv_sec");
    let tv_usec = utmpx.ut_tv.tv_usec;
    assert_eq!(tv_usec, 460597, "ut_tv.tv_usec");

    eprintln!("netbsd_x8632::utmpx: {:?}", utmpx);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 140, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_netbsd_x8664_lastlog() {
    let entry = buffer_to_fixedstructptr(
        &NETBSD_X8664_LASTLOG_BUFFER1,
        FixedStructType::Fs_Netbsd_x8664_Lastlog,
    ).unwrap();
    let lastlog: &netbsd_x8664::lastlog = entry.as_netbsd_x8664_lastlog();
    let tv_sec = lastlog.ll_time;
    assert_eq!(tv_sec, 1708850203, "ll_time");
    assert_eq!(lastlog.ll_line(), CString::new("pts/2").unwrap().as_c_str(), "ll_line");
    assert_eq!(lastlog.ll_host(), CString::new("192.168.100.254").unwrap().as_c_str(), "ll_host");

    eprintln!("netbsd_x8664::lastlog: {:?}", lastlog);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 62, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_netbsd_x8664_lastlogx() {
    // TODO: [2024/03/10] `NETBSD_X8664_LASTLOGX_BUFFER1` is filler data
    //       see Issue #243 <https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/243>
    let entry = buffer_to_fixedstructptr(
        &NETBSD_X8664_LASTLOGX_BUFFER1,
        FixedStructType::Fs_Netbsd_x8664_Lastlogx,
    ).unwrap();
    let lastlogx: &netbsd_x8664::lastlogx = entry.as_netbsd_x8664_lastlogx();



    eprintln!("netbsd_x8664::lastlogx: {:?}", lastlogx);
}

#[test]
fn test_netbsd_x8664_utmp() {
    let entry = buffer_to_fixedstructptr(
        &NETBSD_X8664_UTMP_BUFFER1,
        FixedStructType::Fs_Netbsd_x8664_Utmp,
    ).unwrap();
    let utmp: &netbsd_x8664::utmp = entry.as_netbsd_x8664_utmp();
    assert_eq!(utmp.ut_line(), CString::new("pts/2").unwrap().as_c_str(), "ut_line");
    assert_eq!(utmp.ut_name(), CString::new("root").unwrap().as_c_str(), "ut_name");
    assert_eq!(utmp.ut_host(), CString::new("192.168.100.254").unwrap().as_c_str(), "ut_host");
    assert_eq!(utmp.ut_time, 1708850203, "ut_time");

    eprintln!("netbsd_x8664::utmp: {:?}", utmp);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 81, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_netbsd_x8664_utmpx() {
    let entry = buffer_to_fixedstructptr(
        &NETBSD_X8664_UTMPX_BUFFER1,
        FixedStructType::Fs_Netbsd_x8664_Utmpx,
    ).unwrap();
    let utmpx: &netbsd_x8664::utmpx = entry.as_netbsd_x8664_utmpx();
    assert_eq!(utmpx.ut_user(), CString::new("root").unwrap().as_c_str(), "ut_user");
    // BUG: see Issue #MNO
    assert_eq!(utmpx.ut_id(), CString::new("ts/2pts/2").unwrap().as_c_str(), "ut_id");
    assert_eq!(utmpx.ut_line(), CString::new("pts/2").unwrap().as_c_str(), "ut_line");
    assert_eq!(utmpx.ut_host(), CString::new("192.168.100.254").unwrap().as_c_str(), "ut_host");
    assert_eq!(utmpx.ut_session, 0, "ut_session");
    assert_eq!(utmpx.ut_type, 7, "ut_type");
    assert_eq!(utmpx.ut_pid, 201, "ut_pid");
    assert_eq!(utmpx.ut_exit.e_termination, 0, "ut_exit.e_termination");
    assert_eq!(utmpx.ut_exit.e_exit, 0, "ut_exit.e_exit");
    assert_eq!(utmpx.ut_tv.tv_sec, 1708850203, "ut_tv.tv_sec");
    assert_eq!(utmpx.ut_tv.tv_usec, 794868, "ut_tv.tv_usec");

    eprintln!("netbsd_x8664::utmpx: {:?}", utmpx);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 135, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}


#[test]
fn test_openbsd_x86_lastlog() {
    let entry = buffer_to_fixedstructptr(
        &OPENBSD_X86_LASTLOG_BUFFER1,
        FixedStructType::Fs_Openbsd_x86_Lastlog,
    ).unwrap();
    let lastlog: &openbsd_x86::lastlog = entry.as_openbsd_x86_lastlog();
    assert_eq!(lastlog.ll_time, 1706487506, "ll_time");
    assert_eq!(lastlog.ll_line(), CString::new("ttyp0").unwrap().as_c_str(), "ll_line");
    assert_eq!(lastlog.ll_host(), CString::new("192.168.100.254").unwrap().as_c_str(), "ll_host");

    eprintln!("openbsd_x86::lastlog: {:?}", lastlog);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 72, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

#[test]
fn test_openbsd_x86_utmp() {
    let entry = buffer_to_fixedstructptr(
        &OPENBSD_X86_UTMP_BUFFER1,
        FixedStructType::Fs_Openbsd_x86_Utmp,
    ).unwrap();
    let utmp: &openbsd_x86::utmp = entry.as_openbsd_x86_utmp();
    assert_eq!(utmp.ut_line(), CString::new("ttyp0").unwrap().as_c_str(), "ut_line");
    assert_eq!(utmp.ut_name(), CString::new("root").unwrap().as_c_str(), "ut_name");
    assert_eq!(utmp.ut_host(), CString::new("192.168.100.254").unwrap().as_c_str(), "ut_host");
    assert_eq!(utmp.ut_time, 1706487506, "ut_time");

    eprintln!("openbsd_x86::utmp: {:?}", utmp);

    let score = FixedStruct::score_fixedstruct(&entry, 0);
    assert_eq!(score, 81, "score");

    let fs = FixedStruct::from_fixedstructptr(
        0,
        &FO_0,
        entry,
    ).unwrap();
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    _ = fs.as_bytes(&mut buffer);
}

