// src/tests/syslinereader_tests.rs
//

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::common::{
    FPath,
    ResultS4,
};

use crate::readers::blockreader::{
    FileOffset,
    BlockSz,
};

use crate::readers::filepreprocessor::{
    fpath_to_filetype_mimeguess,
};

use crate::readers::helpers::{
    randomize,
    fill,
};

use crate::readers::syslinereader::{
    SyslineReader,
    ResultS4SyslineFind,
};

use crate::data::datetime::{
    // chrono imports
    TimeZone,
    FixedOffset,
    //
    DateTimeL,
    DateTimeLOpt,
    DateTimePattern_str,
    Result_Filter_DateTime2,
    datetime_parse_from_str,
};

use crate::tests::datetime_tests::{
    dt_pattern_has_tz,
};

use crate::tests::common::{
    eprint_file,
};

use crate::printer_debug::helpers::{
    NamedTempFile,
    create_temp_file,
    create_temp_file_bytes,
    ntf_fpath,
};

use crate::printer_debug::printers::{
    str_to_String_noraw,
    dpo,
    dpnf,
    dpof,
    dpxf,
};

use crate::printer_debug::stack::{
    stack_offset_set,
};

#[allow(unused_imports)]
use crate::tests::common::{
    NTF_LOG_EMPTY_FPATH,
    NTF_NL_1_PATH,
    NTF_NL_2_PATH,
    NTF_NL_3_PATH,
    NTF_NL_4_PATH,
    NTF_NL_5_PATH,
    NTF_WNL_1_PATH,
    NTF_GZ_EMPTY_FPATH,
    NTF_GZ_1BYTE_FPATH,
    NTF_GZ_8BYTE_FPATH,
    NTF_TAR_0BYTE_FILEA_FPATH,
    NTF_TAR_1BYTE_FPATH,
    NTF_TAR_1BYTE_FILEA_FPATH,
    NTF_TAR_8BYTE_FILEA_FPATH,
};

use std::str;

extern crate const_format;
use const_format::concatcp;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate test_case;
use test_case::test_case;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// check the return type of `find_sysline` to this dummy approximation of `ResultS4SyslineFind`
#[allow(non_camel_case_types)]
pub type ResultS4SyslineFind_Test = ResultS4<(), std::io::Error>;

const FOUND: ResultS4SyslineFind_Test = ResultS4SyslineFind_Test::Found(());
const DONE: ResultS4SyslineFind_Test = ResultS4SyslineFind_Test::Done;

/// helper to wrap the match and panic checks
fn new_SyslineReader(path: &FPath, blocksz: BlockSz, tzo: FixedOffset) -> SyslineReader {
    stack_offset_set(Some(2));
    let (filetype, _mimeguess) = fpath_to_filetype_mimeguess(path);
    match SyslineReader::new(path.clone(), filetype, blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({:?}, {:?}, {:?}) failed {}", path, blocksz, tzo, err);
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// - `FileOffset` is the input
/// - second parameter is expected result enum
/// - third parameter is the expected sysline string value
type TestFindSyslineCheck<'a> = (FileOffset, ResultS4SyslineFind_Test, &'a str);
type TestFindSyslineChecks<'a> = Vec<TestFindSyslineCheck<'a>>;

const NTF5_DATA_LINE0: &str = "[20200113-11:03:06] [DEBUG] Testing if xrdp can listen on 0.0.0.0 port 3389.\n";
const NTF5_DATA_LINE1: &str = "[20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
CLOSED!\n";
const NTF5_DATA_LINE2: &str = "[20200113-11:03:08] [INFO ] starting xrdp with pid 23198\n";
const NTF5_DATA_LINE3: &str = "[20200113-11:13:59] [DEBUG] Certification found
    FOUND CERTIFICATE!\n";
const NTF5_DATA_LINE4: &str = "[20200113-11:13:59] [DEBUG] Certification complete.\n";

const NTF5_DATA: &str = concatcp!(
    NTF5_DATA_LINE0,
    NTF5_DATA_LINE1,
    NTF5_DATA_LINE2,
    NTF5_DATA_LINE3,
    NTF5_DATA_LINE4,
);

const NTF5_DATA_LINE0_OFFSET: usize = 0;
const NTF5_DATA_LINE1_OFFSET: usize = NTF5_DATA_LINE0.as_bytes().len();
const NTF5_DATA_LINE2_OFFSET: usize = NTF5_DATA_LINE1_OFFSET + NTF5_DATA_LINE1.as_bytes().len();
const NTF5_DATA_LINE3_OFFSET: usize = NTF5_DATA_LINE2_OFFSET + NTF5_DATA_LINE2.as_bytes().len();
const NTF5_DATA_LINE4_OFFSET: usize = NTF5_DATA_LINE3_OFFSET + NTF5_DATA_LINE3.as_bytes().len();

lazy_static! {
    static ref NTF5: NamedTempFile = {
        create_temp_file(&NTF5_DATA)
    };
    static ref NTF5_PATH: FPath = {
        ntf_fpath(&NTF5)
    };
    static ref TZO_E8: FixedOffset = {
        FixedOffset::west(3600 * 8)
    };
}

/// basic test of `SyslineReader.find_sysline`
fn impl_find_sysline(
    cache: bool,
    blocksz: BlockSz,
    checks: TestFindSyslineChecks,
) {
    stack_offset_set(Some(2));
    dpnf!("(cache {:?}, blocksz {:?})", cache, blocksz);
    let mut slr = new_SyslineReader(&NTF5_PATH, blocksz, *TZO_E8);
    if cache {
        slr.LRU_cache_disable();
    }
    for (fo, result_expect, sline_expect) in checks.iter() {
        dpof!("slr.find_sysline({})", fo);
        let result_actual = slr.find_sysline(*fo);
        assert_results4(fo, result_expect, &result_actual);
        match result_actual {
            ResultS4SyslineFind::Found((_fo, slp)) => {
                let slp_string = (*slp).to_String();
                dpof!("slr.find_sysline({}) Found", fo);
                dpof!("expected {:?}", sline_expect);
                dpof!("actual   {:?}", &slp_string.as_str());
                assert_eq!(sline_expect, &slp_string.as_str(), "\nExpected {:?}\nActual   {:?}\n", sline_expect, &slp_string.as_str());
            }
            ResultS4SyslineFind::Done => {
                dpof!("slr.find_sysline({}) Done", fo);
            }
            ResultS4SyslineFind::Err(err) => {
                panic!("ERROR: impl_find_sysline: slr.find_sysline({}) returned Err({})", fo, err);
            }
        }
    }

    dpxf!();
}

#[test_case(true, 0x2 ; "cache_0x2")]
#[test_case(false, 0x2 ; "nocache_0x2")]
#[test_case(true, 0x4 ; "cache_0x4")]
#[test_case(false, 0x4 ; "nocache_0x4")]
#[test_case(true, 0xF ; "cache_0xF")]
#[test_case(false, 0xF ; "nocache_0xF")]
#[test_case(true, 0xFF ; "cache_0xFF")]
#[test_case(false, 0xFF ; "nocache_0xFF")]
#[test_case(true, 0xFFFF ; "cache_0xFFFF")]
#[test_case(false, 0xFFFF ; "nocache_0xFFFF")]
fn test_find_sysline_A0(cache: bool, blocksz: BlockSz) {
    let checks: TestFindSyslineChecks = vec![
        (NTF5_DATA_LINE0_OFFSET as FileOffset, FOUND, NTF5_DATA_LINE0),
        (NTF5_DATA_LINE1_OFFSET as FileOffset, FOUND, NTF5_DATA_LINE1),
        (NTF5_DATA_LINE2_OFFSET as FileOffset, FOUND, NTF5_DATA_LINE2),
        (NTF5_DATA_LINE3_OFFSET as FileOffset, FOUND, NTF5_DATA_LINE3),
        (NTF5_DATA_LINE4_OFFSET as FileOffset, FOUND, NTF5_DATA_LINE4),
    ];
    impl_find_sysline(cache, blocksz, checks);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// `FileOffset` and first datetime `str` are inputs.
/// The datetime `str` transformed to `DateTimeL` and then passed to
/// `syslinereader.find_sysline_at_datetime_filter(FileOffset, Some(DateTimeL))`.
///
/// The third `ResultS4SyslineFind_Test` is the expected return.
/// The last `str` is the expected sysline data, in `str` form, returned (this is the tested
/// comparison).
type TestFindSyslineAtDatetimeFilterCheck<'a> = (FileOffset, &'a str, ResultS4SyslineFind_Test, &'a str);
type TestFindSyslineAtDatetimeFilterChecks<'a> = Vec<TestFindSyslineAtDatetimeFilterCheck<'a>>;

/// underlying test code for `SyslineReader.find_datetime_in_line`
/// called by other functions `test_find_sysline_at_datetime_filterX`
// TODO: also check return type, e.g. also allow checking for `Done`
fn impl_test_find_sysline_at_datetime_filter(
    ntf: &NamedTempFile,
    dt_pattern: &DateTimePattern_str,
    cache: bool,
    blocksz: BlockSz,
    checks: TestFindSyslineAtDatetimeFilterChecks,
) {
    dpnf!("(…, {:?}, {}, …)", dt_pattern, blocksz);

    let path = ntf_fpath(&ntf);
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = new_SyslineReader(&path, blocksz, tzo);
    if !cache {
        slr.LRU_cache_disable();
    }
    for (fo1, dts, result_expect, sline_expect) in checks.iter() {
        // TODO: add `has_tz` to `checks`
        let has_tz = dt_pattern_has_tz(&dt_pattern);
        dpo!("datetime_parse_from_str({:?}, {:?}, {:?}, {:?})", str_to_String_noraw(dts), dt_pattern, has_tz, &tzo);
        let dt = match datetime_parse_from_str(dts, dt_pattern, has_tz, &tzo) {
            Some(val) => val,
            None => {
                panic!("ERROR: datetime_from_str({:?}, {:?}) returned None", dts, dt_pattern);
            }
        };
        let sline_expect_noraw = str_to_String_noraw(sline_expect);
        dpo!("find_sysline_at_datetime_filter({}, {:?})", fo1, dt);
        let result = slr.find_sysline_at_datetime_filter(*fo1, &Some(dt));
        assert_results4(fo1, result_expect, &result);
        match result {
            ResultS4SyslineFind::Found(val)
            => {
                let sline = val.1.to_String();
                let sline_noraw = str_to_String_noraw(sline.as_str());
                eprintln!("\nexpected: {:?}", sline_expect_noraw);
                eprintln!("returned: {:?}\n", sline_noraw);
                let sline_expect_string = String::from(*sline_expect);
                assert_eq!(
                    sline,
                    sline_expect_string,
                    "Expected {:?} == {:?} but it is not!",
                    sline_noraw,
                    sline_expect_noraw
                );
                eprintln!(
                    "Check PASSED SyslineReader().find_sysline_at_datetime_filter({}, {:?}) == {:?}",
                    fo1, dts, sline_noraw
                );
            }
            ResultS4SyslineFind::Done => {}
            ResultS4SyslineFind::Err(err) => {
                panic!("During test unexpected result Error {}", err);
            }
        }
    }

    dpxf!("impl_test_find_sysline_at_datetime_filter(…)");
}

// -------------------------------------------------------------------------------------------------

const NTF26_DATETIME_FORMAT: &DateTimePattern_str = "%Y-%m-%d %H:%M:%S";

const NTF26_DATA: &str = "\
2020-01-01 00:00:00
2020-01-01 00:00:01a
2020-01-01 00:00:02ab
2020-01-01 00:00:03abc
2020-01-01 00:00:04abcd
2020-01-01 00:00:05abcde
2020-01-01 00:00:06abcdef
2020-01-01 00:00:07abcdefg
2020-01-01 00:00:08abcdefgh
2020-01-01 00:00:09abcdefghi
2020-01-01 00:00:10abcdefghij
2020-01-01 00:00:11abcdefghijk
2020-01-01 00:00:12abcdefghijkl
2020-01-01 00:00:13abcdefghijklm
2020-01-01 00:00:14abcdefghijklmn
2020-01-01 00:00:15abcdefghijklmno
2020-01-01 00:00:16abcdefghijklmnop
2020-01-01 00:00:17abcdefghijklmnopq
2020-01-01 00:00:18abcdefghijklmnopqr
2020-01-01 00:00:19abcdefghijklmnopqrs
2020-01-01 00:00:20abcdefghijklmnopqrst
2020-01-01 00:00:21abcdefghijklmnopqrstu
2020-01-01 00:00:22abcdefghijklmnopqrstuv
2020-01-01 00:00:23abcdefghijklmnopqrstuvw
2020-01-01 00:00:24abcdefghijklmnopqrstuvwx
2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy
2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz
";

const NTF26_DATA_DT0: &str = "2020-01-01 00:00:00";
const NTF26_DATA_LINE0n: &str = "2020-01-01 00:00:00\n";
const NTF26_DATA_DT1: &str = "2020-01-01 00:00:01";
const NTF26_DATA_LINE1n: &str = "2020-01-01 00:00:01a\n";
const NTF26_DATA_DT2: &str = "2020-01-01 00:00:02";
const NTF26_DATA_LINE2n: &str = "2020-01-01 00:00:02ab\n";
const NTF26_DATA_DT3: &str = "2020-01-01 00:00:03";
const NTF26_DATA_LINE3n: &str = "2020-01-01 00:00:03abc\n";
const NTF26_DATA_DT4: &str = "2020-01-01 00:00:04";
const NTF26_DATA_LINE4n: &str = "2020-01-01 00:00:04abcd\n";
// blah, this is a lot of work...
const NTF26_DATA_DT24: &str = "2020-01-01 00:00:24";
const NTF26_DATA_LINE24n: &str = "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n";
const NTF26_DATA_DT25: &str = "2020-01-01 00:00:25";
const NTF26_DATA_LINE25n: &str = "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n";
const NTF26_DATA_DT26: &str = "2020-01-01 00:00:26";
const NTF26_DATA_LINE26n: &str = "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n";

lazy_static! {
    static ref NTF26: NamedTempFile = {
        create_temp_file(&NTF26_DATA)
    };
    static ref NTF26_PATH: FPath = {
        ntf_fpath(&NTF26)
    };

    static ref NTF26_checks: TestFindSyslineAtDatetimeFilterChecks<'static> = {
        Vec::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT3, FOUND, NTF26_DATA_LINE3n),
            (0, NTF26_DATA_DT4, FOUND, NTF26_DATA_LINE4n),
            (0, "2020-01-01 00:00:05", FOUND, "2020-01-01 00:00:05abcde\n"),
            (0, "2020-01-01 00:00:06", FOUND, "2020-01-01 00:00:06abcdef\n"),
            (0, "2020-01-01 00:00:07", FOUND, "2020-01-01 00:00:07abcdefg\n"),
            (0, "2020-01-01 00:00:08", FOUND, "2020-01-01 00:00:08abcdefgh\n"),
            (0, "2020-01-01 00:00:09", FOUND, "2020-01-01 00:00:09abcdefghi\n"),
            (0, "2020-01-01 00:00:10", FOUND, "2020-01-01 00:00:10abcdefghij\n"),
            (0, "2020-01-01 00:00:11", FOUND, "2020-01-01 00:00:11abcdefghijk\n"),
            (0, "2020-01-01 00:00:12", FOUND, "2020-01-01 00:00:12abcdefghijkl\n"),
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:14", FOUND, "2020-01-01 00:00:14abcdefghijklmn\n"),
            (0, "2020-01-01 00:00:15", FOUND, "2020-01-01 00:00:15abcdefghijklmno\n"),
            (0, "2020-01-01 00:00:16", FOUND, "2020-01-01 00:00:16abcdefghijklmnop\n"),
            (0, "2020-01-01 00:00:17", FOUND, "2020-01-01 00:00:17abcdefghijklmnopq\n"),
            (0, "2020-01-01 00:00:18", FOUND, "2020-01-01 00:00:18abcdefghijklmnopqr\n"),
            (0, "2020-01-01 00:00:19", FOUND, "2020-01-01 00:00:19abcdefghijklmnopqrs\n"),
            (0, "2020-01-01 00:00:20", FOUND, "2020-01-01 00:00:20abcdefghijklmnopqrst\n"),
            (0, "2020-01-01 00:00:21", FOUND, "2020-01-01 00:00:21abcdefghijklmnopqrstu\n"),
            (0, "2020-01-01 00:00:22", FOUND, "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n"),
            (0, "2020-01-01 00:00:23", FOUND, "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n"),
            (0, NTF26_DATA_DT24, FOUND, NTF26_DATA_LINE24n),
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])
    };

    static ref NTF26_checksx: TestFindSyslineAtDatetimeFilterChecks<'static> = {
        Vec::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (19, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (40, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (62, NTF26_DATA_DT3, FOUND, NTF26_DATA_LINE3n),
            (85, NTF26_DATA_DT4, FOUND, NTF26_DATA_LINE4n),
            (109, "2020-01-01 00:00:05", FOUND, "2020-01-01 00:00:05abcde\n"),
            (134, "2020-01-01 00:00:06", FOUND, "2020-01-01 00:00:06abcdef\n"),
            (162, "2020-01-01 00:00:07", FOUND, "2020-01-01 00:00:07abcdefg\n"),
            (187, "2020-01-01 00:00:08", FOUND, "2020-01-01 00:00:08abcdefgh\n"),
            (215, "2020-01-01 00:00:09", FOUND, "2020-01-01 00:00:09abcdefghi\n"),
            (244, "2020-01-01 00:00:10", FOUND, "2020-01-01 00:00:10abcdefghij\n"),
            (274, "2020-01-01 00:00:11", FOUND, "2020-01-01 00:00:11abcdefghijk\n"),
            (305, "2020-01-01 00:00:12", FOUND, "2020-01-01 00:00:12abcdefghijkl\n"),
            (337, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (370, "2020-01-01 00:00:14", FOUND, "2020-01-01 00:00:14abcdefghijklmn\n"),
            (404, "2020-01-01 00:00:15", FOUND, "2020-01-01 00:00:15abcdefghijklmno\n"),
            (439, "2020-01-01 00:00:16", FOUND, "2020-01-01 00:00:16abcdefghijklmnop\n"),
            (475, "2020-01-01 00:00:17", FOUND, "2020-01-01 00:00:17abcdefghijklmnopq\n"),
            (512, "2020-01-01 00:00:18", FOUND, "2020-01-01 00:00:18abcdefghijklmnopqr\n"),
            (550, "2020-01-01 00:00:19", FOUND, "2020-01-01 00:00:19abcdefghijklmnopqrs\n"),
            (589, "2020-01-01 00:00:20", FOUND, "2020-01-01 00:00:20abcdefghijklmnopqrst\n"),
            (629, "2020-01-01 00:00:21", FOUND, "2020-01-01 00:00:21abcdefghijklmnopqrstu\n"),
            (670, "2020-01-01 00:00:22", FOUND, "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n"),
            (712, "2020-01-01 00:00:23", FOUND, "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n"),
            (755, NTF26_DATA_DT24, FOUND, NTF26_DATA_LINE24n),
            (799, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (844, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])
    };
}

// TODO: [2022/03/16] create test cases with varying sets of Checks passed-in, current setup is always
//       clean, sequential series of checks from file_offset 0.

/// a `std::io::Error` does not implement `Copy` or `Clone` which means `std::io::Result`
/// also does not.
/// This function does a copy of a `ResultS4SyslineFind_Test`, dropping any particular `Error` for
/// a generic one.
fn copy_ResultS4(result: &ResultS4SyslineFind_Test) -> ResultS4SyslineFind_Test {
    match result {
        ResultS4SyslineFind_Test::Found(_) => FOUND,
        ResultS4SyslineFind_Test::Done => DONE,
        ResultS4SyslineFind_Test::Err(_err) =>
            // use a generic filler error; particulars are not important for testing
            ResultS4SyslineFind_Test::Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "FILLER",
                )
            ),
    }
}

/// basic test of `SyslineReader.find_datetime_in_line`
fn impl_test_find_sysline_at_datetime_filter1(
    cache: bool,
    blocksz: BlockSz,
    checks: Option<TestFindSyslineAtDatetimeFilterChecks>,
) {
    stack_offset_set(None);
    dpnf!();

    // if passed `checks` then use that, otherwise use a copy of the static `NTF26_checks`
    let checks_ = match checks {
        Some(checks_) => checks_,
        None => {
            // must manually copy `NTF26_checks`
            let mut checks_ = TestFindSyslineAtDatetimeFilterChecks::with_capacity(NTF26_checks.len());
            for check in NTF26_checks.iter() {
                let result_expect = copy_ResultS4(&check.2);
                checks_.push(
                    (check.0, check.1, result_expect, check.3)
                )
            }

            checks_
        }
    };
    impl_test_find_sysline_at_datetime_filter(
        &NTF26,
        NTF26_DATETIME_FORMAT,
        cache,
        blocksz,
        checks_
    );
    dpxf!();
}

// XXX: are these different BlockSz tests necessary? are not these adequately tested by
//      other lower-level tests?

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 64; "cache_64")]
#[test_case(false, 64; "nocache_64")]
#[test_case(true, 128; "cache_128")]
#[test_case(false, 128; "nocache_128")]
#[test_case(true, 256; "cache_256")]
#[test_case(false, 256; "nocache_256")]
#[test_case(true, 512; "cache_512")]
#[test_case(false, 512; "nocache_512")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
#[test_case(true, 2056; "cache_2056")]
#[test_case(false, 2056; "nocache_2056")]
fn test_find_sysline_at_datetime_filter(cache: bool, blocksz: BlockSz) {
    impl_test_find_sysline_at_datetime_filter1(cache, 4, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            NTF26_DATA_DT0,
            FOUND,
            NTF26_DATA_LINE0n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_a() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            NTF26_DATA_DT1,
            FOUND,
            NTF26_DATA_LINE1n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_b() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            NTF26_DATA_DT2,
            FOUND,
            NTF26_DATA_LINE2n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_c() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            NTF26_DATA_DT3,
            FOUND,
            NTF26_DATA_LINE3n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_d() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            NTF26_DATA_DT4,
            FOUND,
            NTF26_DATA_LINE4n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_e() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:05",
            FOUND,
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_f() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:06",
            FOUND,
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_g() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:07",
            FOUND,
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_h() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:08",
            FOUND,
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_i() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:09",
            FOUND,
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_j() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:10",
            FOUND,
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_k() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:11",
            FOUND,
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_l() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:12",
            FOUND,
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_m() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:13",
            FOUND,
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_n() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:14",
            FOUND,
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_o() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:15",
            FOUND,
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_p() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:16",
            FOUND,
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_q() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:17",
            FOUND,
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_r() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:18",
            FOUND,
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_s() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:19",
            FOUND,
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_t() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:20",
            FOUND,
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_u() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:21",
            FOUND,
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_v() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:22",
            FOUND,
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_w() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:23",
            FOUND,
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_x() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            NTF26_DATA_DT24,
            FOUND,
            NTF26_DATA_LINE24n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_y() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            NTF26_DATA_DT25,
            FOUND,
            NTF26_DATA_LINE25n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_z() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            NTF26_DATA_DT26,
            FOUND,
            NTF26_DATA_LINE26n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_a() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            19,
            NTF26_DATA_DT1,
            FOUND,
            NTF26_DATA_LINE1n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_b() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            40,
            NTF26_DATA_DT2,
            FOUND,
            NTF26_DATA_LINE2n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_c() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            62,
            NTF26_DATA_DT3,
            FOUND,
            NTF26_DATA_LINE3n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_d() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            85,
            NTF26_DATA_DT4,
            FOUND,
            NTF26_DATA_LINE4n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_e() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            109,
            "2020-01-01 00:00:05",
            FOUND,
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_f() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            134,
            "2020-01-01 00:00:06",
            FOUND,
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_g() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            160,
            "2020-01-01 00:00:07",
            FOUND,
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_h() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            187,
            "2020-01-01 00:00:08",
            FOUND,
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_i() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            215,
            "2020-01-01 00:00:09",
            FOUND,
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_j() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            244,
            "2020-01-01 00:00:10",
            FOUND,
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_k() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            274,
            "2020-01-01 00:00:11",
            FOUND,
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_l() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            305,
            "2020-01-01 00:00:12",
            FOUND,
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_m() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            337,
            "2020-01-01 00:00:13",
            FOUND,
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_n() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            370,
            "2020-01-01 00:00:14",
            FOUND,
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_o() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            404,
            "2020-01-01 00:00:15",
            FOUND,
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_p() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            439,
            "2020-01-01 00:00:16",
            FOUND,
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_q() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            475,
            "2020-01-01 00:00:17",
            FOUND,
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_r() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            512,
            "2020-01-01 00:00:18",
            FOUND,
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_s() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            550,
            "2020-01-01 00:00:19",
            FOUND,
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_t() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            589,
            "2020-01-01 00:00:20",
            FOUND,
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_u() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            629,
            "2020-01-01 00:00:21",
            FOUND,
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_v() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            670,
            "2020-01-01 00:00:22",
            FOUND,
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_w() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            712,
            "2020-01-01 00:00:23",
            FOUND,
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_x() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            755,
            NTF26_DATA_DT24,
            FOUND,
            NTF26_DATA_LINE24n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_y() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            799,
            NTF26_DATA_DT25,
            FOUND,
            NTF26_DATA_LINE25n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_z() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            844,
            NTF26_DATA_DT26,
            FOUND,
            NTF26_DATA_LINE26n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_z_() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_y_() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_x_() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT24, FOUND, NTF26_DATA_LINE24n),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_m_() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_za() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_ya() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_xa() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT24, FOUND, NTF26_DATA_LINE24n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_ma() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3____() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__ab() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__az() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__bd() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT4, FOUND, NTF26_DATA_LINE4n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__ml() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:12", FOUND, "2020-01-01 00:00:12abcdefghijkl\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__my() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__mz() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__m_() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aaa() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abc() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT3, FOUND, NTF26_DATA_LINE3n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aba() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abn() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, "2020-01-01 00:00:14", FOUND, "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aby() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abz() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aaz() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_byo() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, "2020-01-01 00:00:15", FOUND, "2020-01-01 00:00:15abcdefghijklmno\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zaa() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbc() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT3, FOUND, NTF26_DATA_LINE3n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zba() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbn() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, "2020-01-01 00:00:14", FOUND, "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zby() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbz() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zaz() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaa() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybc() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT3, FOUND, NTF26_DATA_LINE3n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yba() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybn() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, "2020-01-01 00:00:14", FOUND, "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yby() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybz() {
    impl_test_find_sysline_at_datetime_filter1(
        true,
        64,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

fn test_find_sysline_at_datetime_filter_checks_3_yaz(cache: bool, blocksz: BlockSz) {
    impl_test_find_sysline_at_datetime_filter1(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz__cache__2() {
    test_find_sysline_at_datetime_filter_checks_3_yaz(true, 2);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz__cache__8() {
    test_find_sysline_at_datetime_filter_checks_3_yaz(true, 8);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz__cache__16() {
    test_find_sysline_at_datetime_filter_checks_3_yaz(true, 16);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz__cache__32() {
    test_find_sysline_at_datetime_filter_checks_3_yaz(true, 32);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz__cache__64() {
    test_find_sysline_at_datetime_filter_checks_3_yaz(true, 64);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz__nocache__2() {
    test_find_sysline_at_datetime_filter_checks_3_yaz(false, 2);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz__nocache__8() {
    test_find_sysline_at_datetime_filter_checks_3_yaz(false, 8);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz__nocache__16() {
    test_find_sysline_at_datetime_filter_checks_3_yaz(false, 16);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz__nocache__32() {
    test_find_sysline_at_datetime_filter_checks_3_yaz(false, 32);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz__nocache__64() {
    test_find_sysline_at_datetime_filter_checks_3_yaz(false, 64);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz__nocache__128() {
    test_find_sysline_at_datetime_filter_checks_3_yaz(false, 128);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz__nocache__256() {
    test_find_sysline_at_datetime_filter_checks_3_yaz(false, 256);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// TODO: [2022/03/18] create one wrapper test test_find_sysline_at_datetime_checks_ that takes some
//        vec of test-input-output, and does all possible permutations.

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

type TestSyslineReaderCheck<'a> = (&'a str, FileOffset);
type TestSyslineReaderChecks<'a> = Vec<(&'a str, FileOffset)>;

/// basic linear test of `SyslineReader::find_sysline`
#[allow(non_snake_case)]
fn impl_test_SyslineReader_find_sysline(
    path: &FPath,
    blocksz: BlockSz,
    fileoffset: FileOffset,
    checks: &TestSyslineReaderChecks
) {
    stack_offset_set(Some(2));
    dpnf!("({:?}, {}, {})", path, blocksz, fileoffset);
    eprint_file(path);
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = new_SyslineReader(path, blocksz, tzo);

    let mut fo1: FileOffset = fileoffset;
    let mut check_i: usize = 0;
    loop {
        let result = slr.find_sysline(fo1);
        //let done = result.is_done() || result.is_eof();
        let done = result.is_done();
        match result {
            ResultS4SyslineFind::Found((fo, slp)) => {
                dpof!("slr.find_sysline({}) returned Found({}, @{:p})", fo1, fo, &*slp);
                dpof!(
                    "FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    fo,
                    &(*slp),
                    slp.count_lines(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                fo1 = fo;

                if checks.is_empty() {
                    continue;
                }
                dpof!("find_sysline({}); check {} expect ({:?}, {:?})", fo1, check_i, checks[check_i].1, checks[check_i].0);
                // check slp.String
                let check_String = checks[check_i].0.to_string();
                let actual_String = (*slp).to_String();
                assert_eq!(check_String, actual_String,"\nexpected string value     {:?}\nfind_sysline({:?}) returned {:?}\n", check_String, fo1, actual_String);
                // check fileoffset
                let check_fo = checks[check_i].1;
                assert_eq!(check_fo, fo, "expected fileoffset {}, but find_sysline returned fileoffset {} for check {}", check_fo, fo, check_i);
            }
            ResultS4SyslineFind::Done => {
                dpof!("slr.find_sysline({}) returned Done", fo1);
                break;
            }
            ResultS4SyslineFind::Err(err) => {
                dpof!("slr.find_sysline({}) returned Err({})", fo1, err);
                panic!("ERROR: {}", err);
            }
        }
        check_i += 1;
        if done {
            break;
        }
    }
    assert_eq!(checks.len(), check_i, "expected {} Sysline checks but only {} Sysline checks were done", checks.len(), check_i);

    dpof!("Found {} Lines, {} Syslines", slr.count_lines_processed(), slr.count_syslines_stored());
    dpxf!();
}

const test_data_file_A1_dt6: &str = "\
2000-01-01 00:00:00
2000-01-01 00:00:01a
2000-01-01 00:00:02ab
2000-01-01 00:00:03abc
2000-01-01 00:00:04abcd
2000-01-01 00:00:05abcde";

const test_data_file_A1_dt6_checks: [TestSyslineReaderCheck; 6] = [
    ("2000-01-01 00:00:00\n", 20),
    ("2000-01-01 00:00:01a\n", 41),
    ("2000-01-01 00:00:02ab\n", 63),
    ("2000-01-01 00:00:03abc\n", 86),
    ("2000-01-01 00:00:04abcd\n", 110),
    ("2000-01-01 00:00:05abcde", 134),
];

lazy_static! {
    static ref NTF_A1: NamedTempFile = {
        create_temp_file(test_data_file_A1_dt6)
    };
    static ref NTF_A1_path: FPath = {
        ntf_fpath(&NTF_A1)
    };
}

#[test]
fn test_find_sysline_A1_dt6_4_0_()
{
    let checks = TestSyslineReaderChecks::from(test_data_file_A1_dt6_checks);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 4, 0, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_0_()
{
    let checks = TestSyslineReaderChecks::from(test_data_file_A1_dt6_checks);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 0, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_1_()
{
    let checks = TestSyslineReaderChecks::from(&test_data_file_A1_dt6_checks[1..]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 40, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_2_()
{
    let checks = TestSyslineReaderChecks::from(&test_data_file_A1_dt6_checks[2..]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 62, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_3_()
{
    let checks = TestSyslineReaderChecks::from(&test_data_file_A1_dt6_checks[3..]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 85, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_4_()
{
    let checks = TestSyslineReaderChecks::from(&test_data_file_A1_dt6_checks[4..]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 86, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_X_beforeend()
{
    let checks = TestSyslineReaderChecks::from([]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 132, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_X_pastend()
{
    let checks = TestSyslineReaderChecks::from([]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 135, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_X9999()
{
    let checks = TestSyslineReaderChecks::from([]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 9999, &checks);
}

// -------------------------------------------------------------------------------------------------

/// helper to assert `find_sysline` return enum
fn assert_results4 (
    fo: &FileOffset,
    result_expect: &ResultS4SyslineFind_Test,
    result_actual: &ResultS4SyslineFind
) {
    let actual: String = format!("{}", result_actual);
    match result_expect {
        ResultS4SyslineFind_Test::Found(_) => {
            assert!(matches!(result_actual, ResultS4SyslineFind::Found(_)), "Expected Found, Actual {} for find_sysline({})", actual, fo);
        },
        ResultS4SyslineFind_Test::Done => {
            assert!(matches!(result_actual, ResultS4SyslineFind::Done), "Expected Done, Actual {} for find_sysline({})", actual, fo);
        },
        _ => {
            panic!("Unexpected result_expect");
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

type TestSyslineReaderAnyInputCheck<'a> = (FileOffset, ResultS4SyslineFind_Test, &'a str);
type TestSyslineReaderAnyInputChecks<'a> = Vec<(FileOffset, ResultS4SyslineFind_Test, &'a str)>;

/// test of `SyslineReader::find_sysline` with test-specified fileoffset searches
#[allow(non_snake_case)]
fn imp_test_findsysline_any_input_checks(
    path: &FPath,
    blocksz: BlockSz,
    lru_cache_enable: bool,
    input_checks: &[TestSyslineReaderAnyInputCheck],
) {
    stack_offset_set(Some(2));
    dpnf!("({:?}, {})", path, blocksz);
    eprint_file(path);
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = new_SyslineReader(path, blocksz, tzo);
    if ! lru_cache_enable {
        slr.LRU_cache_disable();
    }

    let mut check_i: usize = 0;
    let mut first_loop = true;
    for (input_fo, expect_result, expect_val) in input_checks.iter() {
        let result = slr.find_sysline(*input_fo);
        assert_results4(input_fo, expect_result, &result);
        match result {
            ResultS4SyslineFind::Found((_fo, slp)) => {
                dpof!("slr.find_sysline({}) returned Found({}, @{:p})", input_fo, _fo, &*slp);
                dpof!(
                    "FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    _fo,
                    &(*slp),
                    slp.count_lines(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );

                let actual_String = (*slp).to_String();
                let expect_String = String::from(*expect_val);
                dpof!("find_sysline({}); check {}", input_fo, check_i);
                assert_eq!(expect_String, actual_String,"\nexpected string value     {:?}\nfind_sysline({:?}) returned {:?}\n", expect_String, input_fo, actual_String);
            }
            ResultS4SyslineFind::Done => {
                dpof!("slr.find_sysline({}) returned Done", input_fo);
            }
            ResultS4SyslineFind::Err(err) => {
                dpof!("slr.find_sysline({}) returned Err({})", input_fo, err);
                panic!("ERROR: {}", err);
            }
        }
        check_i += 1;
        if first_loop {
            // force pattern analysis
            slr.dt_patterns_analysis();
            first_loop = false;
        }
    }
    assert_eq!(input_checks.len(), check_i, "expected {} Sysline checks but only {} Sysline checks were done", input_checks.len(), check_i);

    dpof!("Found {} Lines, {} Syslines", slr.count_lines_processed(), slr.count_syslines_stored());
    dpxf!("({:?}, {})", &path, blocksz);
}

const test_data_A2_dt6: &str = "\
2000-01-01 00:00:00
2000-01-01 00:00:01a
2000-01-01 00:00:02ab
2000-01-01 00:00:03abc
2000-01-01 00:00:04abcd
2000-01-01 00:00:05abcde";

const test_data_A2_dt6_sysline0: &str = "2000-01-01 00:00:00\n";
const test_data_A2_dt6_sysline1: &str = "2000-01-01 00:00:01a\n";
const test_data_A2_dt6_sysline2: &str = "2000-01-01 00:00:02ab\n";
const test_data_A2_dt6_sysline3: &str = "2000-01-01 00:00:03abc\n";
const test_data_A2_dt6_sysline4: &str = "2000-01-01 00:00:04abcd\n";
const test_data_A2_dt6_sysline5: &str = "2000-01-01 00:00:05abcde";

type TestDataA2Dt6Checks = [TestSyslineReaderAnyInputCheck<'static>; 50];

const test_data_A2_dt6_checks_many: TestDataA2Dt6Checks = [
    (0, FOUND, test_data_A2_dt6_sysline0),
    (1, FOUND, test_data_A2_dt6_sysline0),
    (2, FOUND, test_data_A2_dt6_sysline0),
    (3, FOUND, test_data_A2_dt6_sysline0),
    (4, FOUND, test_data_A2_dt6_sysline0),
    (5, FOUND, test_data_A2_dt6_sysline0),
    (6, FOUND, test_data_A2_dt6_sysline0),
    (19, FOUND, test_data_A2_dt6_sysline0),
    (20, FOUND, test_data_A2_dt6_sysline1),
    (21, FOUND, test_data_A2_dt6_sysline1),
    (22, FOUND, test_data_A2_dt6_sysline1),
    (23, FOUND, test_data_A2_dt6_sysline1),
    (24, FOUND, test_data_A2_dt6_sysline1),
    (25, FOUND, test_data_A2_dt6_sysline1),
    (40, FOUND, test_data_A2_dt6_sysline1),
    (41, FOUND, test_data_A2_dt6_sysline2),
    (42, FOUND, test_data_A2_dt6_sysline2),
    (43, FOUND, test_data_A2_dt6_sysline2),
    (44, FOUND, test_data_A2_dt6_sysline2),
    (45, FOUND, test_data_A2_dt6_sysline2),
    (46, FOUND, test_data_A2_dt6_sysline2),
    (47, FOUND, test_data_A2_dt6_sysline2),
    (61, FOUND, test_data_A2_dt6_sysline2),
    (62, FOUND, test_data_A2_dt6_sysline2),
    (63, FOUND, test_data_A2_dt6_sysline3),
    (64, FOUND, test_data_A2_dt6_sysline3),
    (65, FOUND, test_data_A2_dt6_sysline3),
    (66, FOUND, test_data_A2_dt6_sysline3),
    (67, FOUND, test_data_A2_dt6_sysline3),
    (84, FOUND, test_data_A2_dt6_sysline3),
    (85, FOUND, test_data_A2_dt6_sysline3),
    (86, FOUND, test_data_A2_dt6_sysline4),
    (87, FOUND, test_data_A2_dt6_sysline4),
    (88, FOUND, test_data_A2_dt6_sysline4),
    (89, FOUND, test_data_A2_dt6_sysline4),
    (90, FOUND, test_data_A2_dt6_sysline4),
    (108, FOUND, test_data_A2_dt6_sysline4),
    (109, FOUND, test_data_A2_dt6_sysline4),
    (110, FOUND, test_data_A2_dt6_sysline5),
    (111, FOUND, test_data_A2_dt6_sysline5),
    (112, FOUND, test_data_A2_dt6_sysline5),
    (113, FOUND, test_data_A2_dt6_sysline5),
    (114, FOUND, test_data_A2_dt6_sysline5),
    (134, DONE, ""),
    (135, DONE, ""),
    (136, DONE, ""),
    (137, DONE, ""),
    (138, DONE, ""),
    (139, DONE, ""),
    (140, DONE, ""),
];

/// reverse order `test_data_A2_dt6_checks_many`
const test_data_A2_dt6_checks_many_rev: TestDataA2Dt6Checks = [
    (140, DONE, ""),
    (139, DONE, ""),
    (138, DONE, ""),
    (137, DONE, ""),
    (136, DONE, ""),
    (135, DONE, ""),
    (134, DONE, ""),
    (114, FOUND, test_data_A2_dt6_sysline5),
    (113, FOUND, test_data_A2_dt6_sysline5),
    (112, FOUND, test_data_A2_dt6_sysline5),
    (111, FOUND, test_data_A2_dt6_sysline5),
    (110, FOUND, test_data_A2_dt6_sysline5),
    (109, FOUND, test_data_A2_dt6_sysline4),
    (108, FOUND, test_data_A2_dt6_sysline4),
    (90, FOUND, test_data_A2_dt6_sysline4),
    (89, FOUND, test_data_A2_dt6_sysline4),
    (88, FOUND, test_data_A2_dt6_sysline4),
    (87, FOUND, test_data_A2_dt6_sysline4),
    (86, FOUND, test_data_A2_dt6_sysline4),
    (85, FOUND, test_data_A2_dt6_sysline3),
    (84, FOUND, test_data_A2_dt6_sysline3),
    (67, FOUND, test_data_A2_dt6_sysline3),
    (66, FOUND, test_data_A2_dt6_sysline3),
    (65, FOUND, test_data_A2_dt6_sysline3),
    (64, FOUND, test_data_A2_dt6_sysline3),
    (63, FOUND, test_data_A2_dt6_sysline3),
    (62, FOUND, test_data_A2_dt6_sysline2),
    (61, FOUND, test_data_A2_dt6_sysline2),
    (47, FOUND, test_data_A2_dt6_sysline2),
    (46, FOUND, test_data_A2_dt6_sysline2),
    (45, FOUND, test_data_A2_dt6_sysline2),
    (44, FOUND, test_data_A2_dt6_sysline2),
    (43, FOUND, test_data_A2_dt6_sysline2),
    (42, FOUND, test_data_A2_dt6_sysline2),
    (41, FOUND, test_data_A2_dt6_sysline2),
    (40, FOUND, test_data_A2_dt6_sysline1),
    (25, FOUND, test_data_A2_dt6_sysline1),
    (24, FOUND, test_data_A2_dt6_sysline1),
    (23, FOUND, test_data_A2_dt6_sysline1),
    (22, FOUND, test_data_A2_dt6_sysline1),
    (21, FOUND, test_data_A2_dt6_sysline1),
    (20, FOUND, test_data_A2_dt6_sysline1),
    (19, FOUND, test_data_A2_dt6_sysline0),
    (6, FOUND, test_data_A2_dt6_sysline0),
    (5, FOUND, test_data_A2_dt6_sysline0),
    (4, FOUND, test_data_A2_dt6_sysline0),
    (3, FOUND, test_data_A2_dt6_sysline0),
    (2, FOUND, test_data_A2_dt6_sysline0),
    (1, FOUND, test_data_A2_dt6_sysline0),
    (0, FOUND, test_data_A2_dt6_sysline0),
];

lazy_static! {
    static ref test_data_A2_dt6_ntf: NamedTempFile = {
        create_temp_file(test_data_A2_dt6)
    };
    static ref test_data_A2_dt6_ntf_path: FPath = {
        ntf_fpath(&test_data_A2_dt6_ntf)
    };
}

#[test_case(true, 2, &test_data_A2_dt6_checks_many; "cache_2")]
#[test_case(false, 2, &test_data_A2_dt6_checks_many; "nocache_2")]
#[test_case(true, 2, &test_data_A2_dt6_checks_many_rev; "cache_2_reverse")]
#[test_case(false, 2, &test_data_A2_dt6_checks_many_rev; "nocache_2_reverse")]
#[test_case(true, 4, &test_data_A2_dt6_checks_many; "cache_4")]
#[test_case(false, 4, &test_data_A2_dt6_checks_many; "nocache_4")]
#[test_case(true, 4, &test_data_A2_dt6_checks_many_rev; "cache_4_reverse")]
#[test_case(false, 4, &test_data_A2_dt6_checks_many_rev; "nocache_4_reverse")]
#[test_case(true, 0xF, &test_data_A2_dt6_checks_many; "cache_0xF")]
#[test_case(false, 0xF, &test_data_A2_dt6_checks_many; "nocache_0xF")]
#[test_case(true, 0xF, &test_data_A2_dt6_checks_many_rev; "cache_0xF_reverse")]
#[test_case(false, 0xF, &test_data_A2_dt6_checks_many_rev; "nocache_0xF_reverse")]
#[test_case(true, 0xFF, &test_data_A2_dt6_checks_many; "cache_0xFF")]
#[test_case(false, 0xFF, &test_data_A2_dt6_checks_many; "nocache_0xFF")]
#[test_case(true, 0xFF, &test_data_A2_dt6_checks_many_rev; "cache_0xFF_reverse")]
#[test_case(false, 0xFF, &test_data_A2_dt6_checks_many_rev; "nocache_0xFF_reverse")]
#[test_case(true, 0x1FF, &test_data_A2_dt6_checks_many; "cache_0x1FF")]
#[test_case(false, 0x1FF, &test_data_A2_dt6_checks_many; "nocache_0x1FF")]
#[test_case(true, 0x1FF, &test_data_A2_dt6_checks_many_rev; "cache_0x1FF_reverse")]
#[test_case(false, 0x1FF, &test_data_A2_dt6_checks_many_rev; "nocache_0x1FF_reverse")]
fn test_find_sysline_A2_dt6(
    cache: bool,
    blocksz: BlockSz,
    checks: &TestDataA2Dt6Checks
) {
    imp_test_findsysline_any_input_checks(
        &test_data_A2_dt6_ntf_path,
        blocksz,
        cache,
        checks
    );
}

// -------------------------------------------------------------------------------------------------

#[allow(non_upper_case_globals)]
const test_data_file_B_dt0: &str = "
foo
bar
";

#[allow(non_upper_case_globals)]
const test_data_file_B_dt0_checks: [TestSyslineReaderCheck; 0] = [];

#[test]
fn test_find_sysline_B_dt0_0()
{
    let ntf = create_temp_file(test_data_file_B_dt0);
    let path = ntf_fpath(&ntf);
    let checks = TestSyslineReaderChecks::from(test_data_file_B_dt0_checks);
    impl_test_SyslineReader_find_sysline(&path, 128, 0, &checks);
}

#[test]
fn test_find_sysline_B_dt0_3()
{
    let ntf = create_temp_file(test_data_file_B_dt0);
    let path = ntf_fpath(&ntf);
    let checks = TestSyslineReaderChecks::from(test_data_file_B_dt0_checks);
    impl_test_SyslineReader_find_sysline(&path, 128, 3, &checks);
}

// -------------------------------------------------------------------------------------------------

#[allow(non_upper_case_globals)]
const test_data_file_C_dt6: &str = "\
[DEBUG] 2000-01-01 00:00:00
[DEBUG] 2000-01-01 00:00:01a
[DEBUG] 2000-01-01 00:00:02ab
[DEBUG] 2000-01-01 00:00:03abc
[DEBUG] 2000-01-01 00:00:04abcd
[DEBUG] 2000-01-01 00:00:05abcde";

#[allow(non_upper_case_globals)]
const test_data_file_C_dt6_checks: [TestSyslineReaderCheck; 6] = [
    ("[DEBUG] 2000-01-01 00:00:00\n", 28),
    ("[DEBUG] 2000-01-01 00:00:01a\n", 57),
    ("[DEBUG] 2000-01-01 00:00:02ab\n", 87),
    ("[DEBUG] 2000-01-01 00:00:03abc\n", 118),
    ("[DEBUG] 2000-01-01 00:00:04abcd\n", 150),
    ("[DEBUG] 2000-01-01 00:00:05abcde", 182),
];

lazy_static! {
    static ref test_SyslineReader_C_ntf: NamedTempFile =
        create_temp_file(test_data_file_C_dt6);
    static ref test_SyslineReader_C_ntf_path: FPath =
        ntf_fpath(&test_SyslineReader_C_ntf);
}

#[test]
fn test_find_sysline_C_dt6_0()
{
    let checks = TestSyslineReaderChecks::from(test_data_file_C_dt6_checks);
    impl_test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 0, &checks);
}

#[test]
fn test_find_sysline_C_dt6_3()
{
    let checks = TestSyslineReaderChecks::from(test_data_file_C_dt6_checks);
    impl_test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 3, &checks);
}

#[test]
fn test_find_sysline_C_dt6_27()
{
    let checks = TestSyslineReaderChecks::from(test_data_file_C_dt6_checks);
    impl_test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 27, &checks);
}

#[test]
fn test_find_sysline_C_dt6_28_1__()
{
    let checks = TestSyslineReaderChecks::from(&test_data_file_C_dt6_checks[1..]);
    impl_test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 28, &checks);
}

#[test]
fn test_find_sysline_C_dt6_29_1__()
{
    let checks = TestSyslineReaderChecks::from(&test_data_file_C_dt6_checks[1..]);
    impl_test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 29, &checks);
}

#[test]
fn test_find_sysline_D_invalid1()
{
    let data_invalid1: [u8; 1] = [ 0xFF ];
    let date_checks1: TestSyslineReaderChecks = TestSyslineReaderChecks::from([]);
    let ntf = create_temp_file_bytes(&data_invalid1);
    let path = ntf_fpath(&ntf);
    impl_test_SyslineReader_find_sysline(&path, 128, 0, &date_checks1);
}

// -------------------------------------------------------------------------------------------------

/// notice the second line is an invalid date that will pass regex match
#[allow(non_upper_case_globals)]
const test_data_file_E_dt6: &str = "\
2001-01-01 00:00:00 _
2001-02-31 00:00:01 😩
2001-03-01 00:00:02 😀😁
2001-04-01 00:00:03 😀😁😂
2001-05-01 00:00:04 😀😁😂😃
2001-06-01 00:00:05 😀😁😂😃😄";

const test_data_file_E_dt6_sysline0: &str = "2001-01-01 00:00:00 _\n2001-02-31 00:00:01 😩\n";
const test_data_file_E_dt6_sysline1: &str = "2001-03-01 00:00:02 😀😁\n";
const test_data_file_E_dt6_sysline2: &str = "2001-04-01 00:00:03 😀😁😂\n";

lazy_static! {
    static ref test_SyslineReader_E_ntf: NamedTempFile =
        create_temp_file(test_data_file_E_dt6);
    static ref test_SyslineReader_E_ntf_path: FPath =
        ntf_fpath(&test_SyslineReader_E_ntf);
}

#[test]
fn test_find_sysline_E_dt6_0()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (0, FOUND, test_data_file_E_dt6_sysline0),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_E_dt6_1()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (1, FOUND, test_data_file_E_dt6_sysline0),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_E_dt6_22()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (22, FOUND, test_data_file_E_dt6_sysline0),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_E_dt6_42()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (42, FOUND, test_data_file_E_dt6_sysline0),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_E_dt6_43()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (43, FOUND, test_data_file_E_dt6_sysline0),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_E_dt6_44()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (44, FOUND, test_data_file_E_dt6_sysline0),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_E_dt6_75()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (75, FOUND, test_data_file_E_dt6_sysline1),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_E_dt6_76()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (76, FOUND, test_data_file_E_dt6_sysline2),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_E_dt6_0______78()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (0, FOUND, test_data_file_E_dt6_sysline0),
            (1, FOUND, test_data_file_E_dt6_sysline0),
            (21, FOUND, test_data_file_E_dt6_sysline0),
            (22, FOUND, test_data_file_E_dt6_sysline0),
            (23, FOUND, test_data_file_E_dt6_sysline0),
            (24, FOUND, test_data_file_E_dt6_sysline0),
            (42, FOUND, test_data_file_E_dt6_sysline0),
            (43, FOUND, test_data_file_E_dt6_sysline0),
            (44, FOUND, test_data_file_E_dt6_sysline0),
            (45, FOUND, test_data_file_E_dt6_sysline0),
            (46, FOUND, test_data_file_E_dt6_sysline0),
            (47, FOUND, test_data_file_E_dt6_sysline1),
            (48, FOUND, test_data_file_E_dt6_sysline1),
            (49, FOUND, test_data_file_E_dt6_sysline1),
            (70, FOUND, test_data_file_E_dt6_sysline1),
            (71, FOUND, test_data_file_E_dt6_sysline1),
            (72, FOUND, test_data_file_E_dt6_sysline1),
            (73, FOUND, test_data_file_E_dt6_sysline1),
            (74, FOUND, test_data_file_E_dt6_sysline1),
            (75, FOUND, test_data_file_E_dt6_sysline1),
            (76, FOUND, test_data_file_E_dt6_sysline2),
            (77, FOUND, test_data_file_E_dt6_sysline2),
            (78, FOUND, test_data_file_E_dt6_sysline2),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

// -------------------------------------------------------------------------------------------------

/// notice the fourth line is an invalid date that will pass regex match
#[allow(non_upper_case_globals)]
const _test_data_file_F_dt6: &str = "\
2001-01-01 00:00:00 _
2001-02-01 00:00:01 😀
2001-03-01 00:00:02 😀😁
2001-04-31 00:00:03 😫😫😫
2001-05-01 00:00:04 😀😁😂😃
2001-06-01 00:00:05 😀😁😂😃😄";

const test_data_file_F_dt6_sysline0: &str = "2001-01-01 00:00:00 _\n";
const test_data_file_F_dt6_sysline1: &str = "2001-02-01 00:00:01 😀\n";
const test_data_file_F_dt6_sysline2: &str = "2001-03-01 00:00:02 😀😁\n2001-04-31 00:00:03 😫😫😫\n";
const test_data_file_F_dt6_sysline3: &str = "2001-05-01 00:00:04 😀😁😂😃\n";

lazy_static! {
    static ref test_SyslineReader_F_ntf: NamedTempFile =
        create_temp_file(_test_data_file_F_dt6);
    static ref test_SyslineReader_F_ntf_path: FPath =
        ntf_fpath(&test_SyslineReader_F_ntf);
}

#[test]
fn test_find_sysline_F_dt6_45()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (45, FOUND, test_data_file_F_dt6_sysline1),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_F_dt6_46()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (46, FOUND, test_data_file_F_dt6_sysline1),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_F_dt6_47()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (47, FOUND, test_data_file_F_dt6_sysline2),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_F_dt6_sysline2_sysline3_108_109()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (108, FOUND, test_data_file_F_dt6_sysline2),
            (109, FOUND, test_data_file_F_dt6_sysline3),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_F_dt6_sysline2_sysline3_107_110()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (107, FOUND, test_data_file_F_dt6_sysline2),
            (110, FOUND, test_data_file_F_dt6_sysline3),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_F_dt6_sysline2_sysline3_108_110()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (108, FOUND, test_data_file_F_dt6_sysline2),
            (110, FOUND, test_data_file_F_dt6_sysline3),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_F_dt6_sysline3_sysline2_109_108()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (109, FOUND, test_data_file_F_dt6_sysline3),
            (108, FOUND, test_data_file_F_dt6_sysline2),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_F_dt6_sysline3_sysline2_110_107()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (110, FOUND, test_data_file_F_dt6_sysline3),
            (107, FOUND, test_data_file_F_dt6_sysline2),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_F_dt6_sysline3_sysline2_109_107()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (109, FOUND, test_data_file_F_dt6_sysline3),
            (107, FOUND, test_data_file_F_dt6_sysline2),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

// -------------------------------------------------------------------------------------------------

/// notice the first line is an invalid date that will pass regex match
#[allow(non_upper_case_globals)]
const test_data_file_G_dt4: &str = "\
2001-02-31 00:00:01 a
2001-03-01 00:00:02 b
2001-04-01 00:00:03 c
2001-05-01 00:00:04 d";

const test_data_file_G_dt4_sysline0: &str = "2001-03-01 00:00:02 b\n";
const test_data_file_G_dt4_sysline1: &str = "2001-04-01 00:00:03 c\n";
const test_data_file_G_dt4_sysline2: &str = "2001-05-01 00:00:04 d";

lazy_static! {
    static ref test_SyslineReader_G_ntf: NamedTempFile =
        create_temp_file(test_data_file_G_dt4);
    static ref test_SyslineReader_G_ntf_path: FPath =
        ntf_fpath(&test_SyslineReader_G_ntf);
}

#[test]
fn test_find_sysline_G_dt4_0()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (0, FOUND, test_data_file_G_dt4_sysline0),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_G_dt4_42()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (42, FOUND, test_data_file_G_dt4_sysline0),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_G_dt4_43()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (43, FOUND, test_data_file_G_dt4_sysline0),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_G_dt4_44()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (44, FOUND, test_data_file_G_dt4_sysline1),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_G_dt4_sysline0_sysline1_43_44()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (43, FOUND, test_data_file_G_dt4_sysline0),
            (44, FOUND, test_data_file_G_dt4_sysline1),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_G_dt4_sysline1_sysline0_44_43()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (44, FOUND, test_data_file_G_dt4_sysline1),
            (43, FOUND, test_data_file_G_dt4_sysline0),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_G_dt4_sysline1_sysline0_44_45_42_43()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (44, FOUND, test_data_file_G_dt4_sysline1),
            (45, FOUND, test_data_file_G_dt4_sysline1),
            (42, FOUND, test_data_file_G_dt4_sysline0),
            (43, FOUND, test_data_file_G_dt4_sysline0),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

// -------------------------------------------------------------------------------------------------

#[allow(non_upper_case_globals)]
const test_data_file_H_dt4: &str = "\
2001-02-01 00:00:01 a
2001-03-01 00:00:02 b
2001-04-01 00:00:03 c
2001-05-01 00:00:04 d";

const test_data_file_H_dt4_sysline0: &str = "2001-02-01 00:00:01 a\n";
const test_data_file_H_dt4_sysline1: &str = "2001-03-01 00:00:02 b\n";
const test_data_file_H_dt4_sysline2: &str = "2001-04-01 00:00:03 c\n";
const test_data_file_H_dt4_sysline3: &str = "2001-05-01 00:00:04 d";

lazy_static! {
    static ref test_SyslineReader_H_ntf: NamedTempFile =
        create_temp_file(test_data_file_H_dt4);
    static ref test_SyslineReader_H_ntf_path: FPath =
        ntf_fpath(&test_SyslineReader_H_ntf);
}

#[test]
fn test_find_sysline_H_dt4_sysline0()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (0, FOUND, test_data_file_H_dt4_sysline0),
            (1, FOUND, test_data_file_H_dt4_sysline0),
            (2, FOUND, test_data_file_H_dt4_sysline0),
            (3, FOUND, test_data_file_H_dt4_sysline0),
            (0, FOUND, test_data_file_H_dt4_sysline0),
            (10, FOUND, test_data_file_H_dt4_sysline0),
            (20, FOUND, test_data_file_H_dt4_sysline0),
            (21, FOUND, test_data_file_H_dt4_sysline0),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_H_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_H_dt4_sysline1()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (22, FOUND, test_data_file_H_dt4_sysline1),
            (22, FOUND, test_data_file_H_dt4_sysline1),
            (22, FOUND, test_data_file_H_dt4_sysline1),
            (43, FOUND, test_data_file_H_dt4_sysline1),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_H_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_H_dt4_sysline3_Found_Done()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (86, FOUND, test_data_file_H_dt4_sysline3),
            (85, FOUND, test_data_file_H_dt4_sysline3),
            (87, DONE, ""),
            (88, DONE, ""),
            (87, DONE, ""),
            (66, FOUND, test_data_file_H_dt4_sysline3),
            (88, DONE, ""),
            (86, FOUND, test_data_file_H_dt4_sysline3),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_H_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_find_sysline_H_dt4_Done()
{
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (87, DONE, ""),
            (88, DONE, ""),
            (87, DONE, ""),
        ]
    );
    imp_test_findsysline_any_input_checks(
        &test_SyslineReader_H_ntf_path,
        4,
        false,
        &checks
    );
}

// TODO: test `clear_syslines` and `remove_sysline`

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// print the filtered syslines for a SyslineReader
/// quick debug helper for manual reviews
fn process_SyslineReader(
    slr: &mut SyslineReader, filter_dt_after_opt: &DateTimeLOpt, filter_dt_before_opt: &DateTimeLOpt,
) {
    dpnf!("({:?}, {:?}, {:?})", slr, filter_dt_after_opt, filter_dt_before_opt,);
    let mut fo1: FileOffset = 0;
    let mut search_more = true;
    dpof!("slr.find_sysline_at_datetime_filter({}, {:?})", fo1, filter_dt_after_opt);
    let result = slr.find_sysline_at_datetime_filter(fo1, filter_dt_after_opt);
    match result {
        ResultS4SyslineFind::Found((fo, slp))
        => {
            dpof!(
                "slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Found({}, @{:p})",
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt,
                fo,
                &*slp
            );
            dpof!(
                "FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                fo,
                &(*slp),
                slp.count_lines(),
                (*slp).len(),
                (*slp).to_String_noraw(),
            );
            fo1 = fo;
        }
        ResultS4SyslineFind::Done => {
            dpof!(
                "slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Done",
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt
            );
            search_more = false;
        }
        ResultS4SyslineFind::Err(err) => {
            dpof!(
                "slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Err({})",
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt,
                err
            );
            //search_more = false;
            panic!("ERROR: {}", err);
        }
    }
    if !search_more {
        dpof!("!search_more");
        dpxf!();
        return;
    }
    let mut fo2: FileOffset = fo1;
    loop {
        let result = slr.find_sysline(fo2);
        match result {
            ResultS4SyslineFind::Found((fo, slp))
            => {
                dpof!("slr.find_sysline({}) returned Found({}, @{:p})", fo2, fo, &*slp);
                fo2 = fo;
                dpof!(
                    "FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    fo,
                    &(*slp),
                    slp.count_lines(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                dpof!(
                    "sysline_pass_filters({:?}, {:?}, {:?})",
                    (*slp).dt(),
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                );
                match SyslineReader::sysline_pass_filters(&slp, filter_dt_after_opt, filter_dt_before_opt) {
                    Result_Filter_DateTime2::BeforeRange | Result_Filter_DateTime2::AfterRange => {
                        dpof!("sysline_pass_filters returned not Result_Filter_DateTime2::InRange; continue!");
                        continue;
                    }
                    Result_Filter_DateTime2::InRange => {}
                }
            }
            ResultS4SyslineFind::Done => {
                dpof!("slr.find_sysline({}) returned Done", fo2);
                break;
            }
            ResultS4SyslineFind::Err(err) => {
                dpof!("slr.find_sysline({}) returned Err({})", fo2, err);
                panic!("ERROR: {}", err);
                //break;
            }
        }
    }
    dpxf!("process_SyslineReader({:?}, …)", slr.path());
}

/// quick debug helper
#[allow(non_snake_case)]
fn test_SyslineReader_process_file(
    path: &FPath,
    blocksz: BlockSz,
    filter_dt_after_opt: &DateTimeLOpt,
    filter_dt_before_opt: &DateTimeLOpt,
) -> Option<Box<SyslineReader>> {
    dpnf!(
        "({:?}, {}, {:?}, {:?})",
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
    let tzo8 = FixedOffset::west(3600 * 8);
    let slr: SyslineReader = new_SyslineReader(
        path,
        blocksz,
        tzo8,
    );
    dpof!("{:?}", slr);
    dpxf!();

    Some(Box::new(slr))
}

/// basic test of SyslineReader things
#[allow(non_snake_case)]
fn test_SyslineReader_w_filtering_2(
    path: &FPath, blocksz: BlockSz, filter_dt_after_opt: &DateTimeLOpt, filter_dt_before_opt: &DateTimeLOpt,
) {
    dpnf!(
        "({:?}, {}, {:?}, {:?})",
        path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
    let slr_opt = test_SyslineReader_process_file(path, blocksz, filter_dt_after_opt, filter_dt_before_opt);
    if slr_opt.is_some() {
        let slr = &slr_opt.unwrap();
        dpof!("Found {} Lines, {} Syslines", slr.count_lines_processed(), slr.count_syslines_stored());
    }
    dpxf!();
}

// TODO: add test cases for test_SyslineReader_w_filtering_2

// -------------------------------------------------------------------------------------------------

/// basic test of SyslineReader things
/// process multiple files
#[allow(non_snake_case)]
fn test_SyslineReader_w_filtering_3(
    paths: &Vec<String>,
    blocksz: BlockSz,
    filter_dt_after_opt: &DateTimeLOpt,
    filter_dt_before_opt: &DateTimeLOpt,
) {
    dpnf!(
        "({:?}, {}, {:?}, {:?})",
        &paths,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );

    let mut slrs = Vec::<SyslineReader>::with_capacity(paths.len());
    for path in paths.iter() {
        let tzo8 = FixedOffset::west(3600 * 8);
        dpof!("SyslineReader::new({:?}, {}, {:?})", path, blocksz, tzo8);
        let slr = new_SyslineReader(path, blocksz, tzo8);
        dpof!("{:?}", slr);
        slrs.push(slr)
    }
    for slr in slrs.iter_mut() {
        process_SyslineReader(slr, filter_dt_after_opt, filter_dt_before_opt);
        eprintln!();
    }
    dpxf!();
}

// TODO: add test cases for `test_SyslineReader_w_filtering_3`

// -------------------------------------------------------------------------------------------------

/// basic test of `SyslineReader::find_sysline`
/// read all file offsets but randomly
///
/// TODO: [2021/09] this test was hastily designed for human review. Redesign it for automatic review.
#[allow(non_snake_case)]
fn test_find_sysline_rand(path: &FPath, blocksz: BlockSz) {
    dpnf!("({:?}, {})", path, blocksz);
    let tzo8 = FixedOffset::west(3600 * 8);
    let mut slr = new_SyslineReader(path, blocksz, tzo8);
    dpof!("SyslineReader: {:?}", slr);
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(slr.filesz() as usize);
    fill(&mut offsets_rand);
    dpof!("offsets_rand: {:?}", offsets_rand);
    randomize(&mut offsets_rand);
    dpof!("offsets_rand: {:?}", offsets_rand);

    for fo1 in offsets_rand {
        let result = slr.find_sysline(fo1);
        #[allow(clippy::single_match)]
        match result {
            ResultS4SyslineFind::Err(err) => {
                panic!("slr.find_sysline({}) returned Err({})", fo1, err);
            }
            _ => {}
        }
    }
    dpxf!();
}

#[test]
fn test_find_sysline_rand__zero__2() {
    test_find_sysline_rand(&NTF_LOG_EMPTY_FPATH, 2);
}

#[test]
fn test_find_sysline_rand__test0_nlx1__2() {
    test_find_sysline_rand(&NTF_NL_1_PATH, 2);
}

#[test]
fn test_find_sysline_rand__test0_nlx1__4() {
    test_find_sysline_rand(&NTF_NL_1_PATH, 4);
}

#[test]
fn test_find_sysline_rand__test0_nlx1__8() {
    test_find_sysline_rand(&NTF_NL_1_PATH, 8);
}

#[test]
fn test_find_sysline_rand__test0_nlx1_Win__2() {
    test_find_sysline_rand(&NTF_WNL_1_PATH, 2);
}

#[test]
fn test_find_sysline_rand__test0_nlx1_Win__4() {
    test_find_sysline_rand(&NTF_WNL_1_PATH, 4);
}

#[test]
fn test_find_sysline_rand__test0_nlx1_Win__8() {
    test_find_sysline_rand(&NTF_WNL_1_PATH, 8);
}

#[test]
fn test_find_sysline_rand__test0_nlx2__4() {
    test_find_sysline_rand(&NTF_NL_2_PATH, 4);
}

#[test]
fn test_find_sysline_rand__basic_dt1__4() {
    test_find_sysline_rand(&FPath::from("./logs/other/tests/basic-dt1.log"), 4);
}

#[test]
fn test_find_sysline_rand__dtf5_6c__4() {
    test_find_sysline_rand(&FPath::from("./logs/other/tests/dtf5-6c.log"), 4);
}

#[test]
fn test_find_sysline_rand__dtf5_6c__8() {
    test_find_sysline_rand(&FPath::from("./logs/other/tests/dtf5-6c.log"), 8);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

type CheckFindSyslineInBlock = Vec::<(FileOffset, ResultS4SyslineFind_Test, String)>;

/// test `SyslineReader::find_sysline_in_block`, cache turned off
#[allow(non_snake_case)]
fn test_find_sysline_in_block(
    path: &FPath,
    cache: bool,
    checks: CheckFindSyslineInBlock,
    blocksz: BlockSz,
) {
    dpnf!("({:?}, {}, {})", path, cache, blocksz);
    let tzo = FixedOffset::west(3600 * 2);
    let mut slr = new_SyslineReader(path, blocksz, tzo);
    if !cache {
        slr.LRU_cache_disable();
    }
    dpof!("SyslineReader {:?}", slr);

    for (fo_input, result_expect, value_expect) in checks.iter() {
        let result = slr.find_sysline_in_block(*fo_input);
        assert_results4(fo_input, result_expect, &result);
        match result {
            ResultS4SyslineFind::Found((_fo, slp)) => {
                let value_actual: String = (*slp).to_String();
                assert_eq!(
                    value_expect, &value_actual,
                    "find_sysline_in_block({})\nExpected {:?}\nActual {:?}",
                    fo_input, value_expect, value_actual,
                );
            },
            ResultS4SyslineFind::Done => {
                // self-check
                assert_eq!(
                    value_expect, &String::from(""), "bad test check value {:?}", value_expect,
                );
            },
            ResultS4SyslineFind::Err(err) => {
                panic!("ERROR: find_sysline_in_block({}) returned Error {:?}", fo_input, err);
            },
        }
    }

    dpxf!();
}

#[test_case(true; "cache")]
#[test_case(true; "nocache")]
fn test_find_sysline_in_block__empty0(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![];
    test_find_sysline_in_block(&NTF_LOG_EMPTY_FPATH, cache, checks, 2);
}

#[test_case(true; "cache")]
#[test_case(true; "nocache")]
fn test_find_sysline_in_block__empty1(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![
        (0, DONE, String::from("")),
    ];
    test_find_sysline_in_block(&NTF_NL_1_PATH, cache, checks, 2);
}

#[test_case(true; "cache")]
#[test_case(true; "nocache")]
fn test_find_sysline_in_block__empty2(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![
        (0, DONE, String::from("")),
        (1, DONE, String::from("")),
    ];

    test_find_sysline_in_block(&NTF_NL_2_PATH, cache, checks, 4);
}

#[test_case(true; "cache")]
#[test_case(true; "nocache")]
fn test_find_sysline_in_block__empty4(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![
        (0, DONE, String::from("")),
        (1, DONE, String::from("")),
        (2, DONE, String::from("")),
        (3, DONE, String::from("")),
    ];

    test_find_sysline_in_block(&NTF_NL_4_PATH, cache, checks, 4);
}

const NTF_A3_DATA_LINE1: &str = "2000-01-01 00:00:00\n";
const NTF_A3_DATA_LINE2: &str = "2000-01-02 00:00:00\n";
const NTF_A3_DATA_LINE12: &str = concatcp!(NTF_A3_DATA_LINE1, NTF_A3_DATA_LINE2);

lazy_static! {
    static ref NTF_A3_DATA_LINE1_STRING: String = String::from(NTF_A3_DATA_LINE1);
    static ref NTF_A3_DATA_LINE2_STRING: String = String::from(NTF_A3_DATA_LINE2);

    static ref NTF_A3_DATA_LINE1_BYTES_LEN: usize = NTF_A3_DATA_LINE1_STRING.as_bytes().len();
    /// length `NTF_A3_DATA_LINE1_STRING` to nearest even number
    static ref NTF_A3_DATA_LINE1_BYTES_LEN_EVEN: usize = {
        let len_ = NTF_A3_DATA_LINE1_STRING.as_bytes().len();
        if len_ % 2 == 0 {
            len_
        } else {
            len_ + 1
        }
    };
    static ref NTF_A3_DATA_LINE2_BYTES_LEN: usize = NTF_A3_DATA_LINE2_STRING.as_bytes().len();

    static ref NTF_A3_1: NamedTempFile = create_temp_file(NTF_A3_DATA_LINE1);
    static ref NTF_A3_1_PATH: FPath = ntf_fpath(&NTF_A3_1);

    static ref NTF_A3_12: NamedTempFile = create_temp_file(NTF_A3_DATA_LINE12);
    static ref NTF_A3_12_PATH: FPath = ntf_fpath(&NTF_A3_12);
}

#[test_case(true; "cache")]
#[test_case(true; "nocache")]
fn test_find_sysline_in_block__A3__1_4(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![
        (0, DONE, String::from("")),
        (1, DONE, String::from("")),
        (2, DONE, String::from("")),
        (3, DONE, String::from("")),
        (0xFFFF, DONE, String::from("")),
    ];

    test_find_sysline_in_block(&NTF_A3_1_PATH, cache, checks, 4);
}

#[test_case(true; "cache")]
#[test_case(true; "nocache")]
fn test_find_sysline_in_block__A3__12_4(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![
        (0, DONE, String::from("")),
        (1, DONE, String::from("")),
        (2, DONE, String::from("")),
        (3, DONE, String::from("")),
        (0xFFFF, DONE, String::from("")),
    ];

    test_find_sysline_in_block(&NTF_A3_12_PATH, cache, checks, 4);
}

#[test_case(true; "cache")]
#[test_case(true; "nocache")]
fn test_find_sysline_in_block__A3__1_LINE1LEN(cache: bool) {
    let fo: FileOffset = *NTF_A3_DATA_LINE1_BYTES_LEN as FileOffset;
    dpof!("fo {:?}", fo);
    // use blocks of the same size as the first line in the test file
    let bsz: BlockSz = *NTF_A3_DATA_LINE1_BYTES_LEN_EVEN as BlockSz;
    dpof!("bsz {:?}", bsz);
    let checks: CheckFindSyslineInBlock = vec![
        (0, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (1, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (2, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (fo - 2, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (fo - 1, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (fo, DONE, String::from("")),
        (fo + 1, DONE, String::from("")),
        (0xFFFF, DONE, String::from("")),
    ];

    test_find_sysline_in_block(&NTF_A3_1_PATH, cache, checks, bsz);
}

#[test_case(true; "cache")]
#[test_case(true; "nocache")]
fn test_find_sysline_in_block__A3__12_LINE1LEN(cache: bool) {
    let fo: FileOffset = *NTF_A3_DATA_LINE1_BYTES_LEN as FileOffset;
    let fo12: FileOffset = (*NTF_A3_DATA_LINE1_BYTES_LEN + *NTF_A3_DATA_LINE2_BYTES_LEN) as FileOffset;
    dpof!("fo {:?}", fo);
    // use blocks of the same size as the first line in the test file
    let bsz: BlockSz = *NTF_A3_DATA_LINE1_BYTES_LEN_EVEN as BlockSz;
    dpof!("bsz {:?}", bsz);
    let checks: CheckFindSyslineInBlock = vec![
        (0, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (1, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (2, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (fo - 2, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (fo - 1, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (fo, FOUND, NTF_A3_DATA_LINE2_STRING.clone()),
        (fo + 1, FOUND, NTF_A3_DATA_LINE2_STRING.clone()),
        (fo12 - 2, FOUND, NTF_A3_DATA_LINE2_STRING.clone()),
        (fo12 - 1, FOUND, NTF_A3_DATA_LINE2_STRING.clone()),
        (fo12, DONE, String::from("")),
        (fo12 + 1, DONE, String::from("")),
        (0xFFFF, DONE, String::from("")),
    ];

    test_find_sysline_in_block(&NTF_A3_12_PATH, cache, checks, bsz);
}

#[test_case(true; "cache")]
#[test_case(true; "nocache")]
fn test_find_sysline_in_block__A3__1_FFFF(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![
        (0, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (1, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (2, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (3, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (17, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (18, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (19, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (20, DONE, String::from("")),
        (21, DONE, String::from("")),
    ];

    test_find_sysline_in_block(&NTF_A3_1_PATH, cache, checks, 0xFFFF);
}

#[test_case(true; "cache")]
#[test_case(true; "nocache")]
fn test_find_sysline_in_block__A3__12_FFFF(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![
        (0, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (1, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (2, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (3, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (17, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (18, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (19, FOUND, NTF_A3_DATA_LINE1_STRING.clone()),
        (20, FOUND, NTF_A3_DATA_LINE2_STRING.clone()),
        (21, FOUND, NTF_A3_DATA_LINE2_STRING.clone()),
        (22, FOUND, NTF_A3_DATA_LINE2_STRING.clone()),
        (39, FOUND, NTF_A3_DATA_LINE2_STRING.clone()),
        (40, DONE, String::from("")),
    ];

    test_find_sysline_in_block(&NTF_A3_12_PATH, cache, checks, 0xFFFF);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const HOUR: i32 = 3600;
lazy_static!{
    static ref TZO8: FixedOffset = FixedOffset::west(3600 * 8);
    static ref TZO5: FixedOffset = FixedOffset::east(3600 * 5);
}

#[test]
fn test_datetime_parse_from_str__good_without_tz1() {
    // good without timezone
    let dts1 = "2000-01-01 00:01:01";
    let p1 = "%Y-%m-%d %H:%M:%S";
    let dt1 = datetime_parse_from_str(dts1, p1, false, &TZO8).unwrap();
    let answer1 = TZO8.ymd(2000, 1, 1).and_hms(0, 1, 1);
    assert_eq!(dt1, answer1);
}

#[test]
fn test_datetime_parse_from_str__2_good_without_tz() {
    // good without timezone
    let dts1 = "2000-01-01 00:02:01";
    let p1 = "%Y-%m-%d %H:%M:%S";
    let dt1 = datetime_parse_from_str(dts1, p1, false, &TZO5).unwrap();
    let answer1 = TZO5.ymd(2000, 1, 1).and_hms(0, 2, 1);
    assert_eq!(dt1, answer1);
}

#[test]
fn test_datetime_parse_from_str__3_good_with_tz() {
    // good with timezone
    let dts2 = "2000-01-01 00:00:02 -0100";
    let p2 = "%Y-%m-%d %H:%M:%S %z";
    let dt2 = datetime_parse_from_str(dts2, p2, true, &TZO8).unwrap();
    let answer2 = FixedOffset::west(HOUR).ymd(2000, 1, 1).and_hms(0, 0, 2);
    assert_eq!(dt2, answer2);
}

#[test]
fn test_datetime_parse_from_str__4_bad_with_tz() {
    // bad with timezone
    let dts3 = "2000-01-01 00:00:03 BADD";
    let p3 = "%Y-%m-%d %H:%M:%S %z";
    let dt3 = datetime_parse_from_str(dts3, p3, true, &TZO8);
    assert_eq!(dt3, None);
}

#[test]
fn test_datetime_parse_from_str__5_bad_without_tz() {
    // bad without timezone
    let dts4 = "2000-01-01 00:00:XX";
    let p4 = "%Y-%m-%d %H:%M:%S";
    let dt4 = datetime_parse_from_str(dts4, p4, false, &TZO8);
    assert_eq!(dt4, None);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// given the vector of `DateTimeL`, return the vector index and value of the soonest
/// (minimum) value within a `Some`
/// If the vector is empty then return `None`
#[allow(clippy::ptr_arg)]
fn datetime_soonest2(vec_dt: &Vec<DateTimeL>) -> Option<(usize, DateTimeL)> {
    if vec_dt.is_empty() {
        return None;
    }

    let mut index: usize = 0;
    for (index_, _) in vec_dt.iter().enumerate() {
        if vec_dt[index_] < vec_dt[index] {
            index = index_;
        }
    }

    Some((index, vec_dt[index]))
}

/// test function `datetime_soonest2`
#[test]
fn test_datetime_soonest2() {
    dpnf!();
    let vec0 = Vec::<DateTimeL>::with_capacity(0);
    let val = datetime_soonest2(&vec0);
    assert!(val.is_none());
    let tzo = FixedOffset::west(3600 * 8);

    let dt1_a = datetime_parse_from_str("2001-01-01T12:00:00", "%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
    let vec1: Vec<DateTimeL> = vec![dt1_a];
    let (i_, dt_) = match datetime_soonest2(&vec1) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None1a");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_a);

    let dt1_b = datetime_parse_from_str("2001-01-01T12:00:00-0100", "%Y-%m-%dT%H:%M:%S%z", true, &tzo).unwrap();
    let vec1: Vec<DateTimeL> = vec![dt1_b];
    let (i_, dt_) = match datetime_soonest2(&vec1) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None1b");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_b);

    let dt2_a = datetime_parse_from_str("2002-01-01T11:00:00", "%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
    let vec2a: Vec<DateTimeL> = vec![dt1_a, dt2_a];
    let (i_, dt_) = match datetime_soonest2(&vec2a) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None2a");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_a);

    let vec2b: Vec<DateTimeL> = vec![dt2_a, dt1_a];
    let (i_, dt_) = match datetime_soonest2(&vec2b) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None2b");
        }
    };
    assert_eq!(i_, 1);
    assert_eq!(dt_, dt1_a);

    let dt3 = datetime_parse_from_str("2000-01-01T12:00:00", "%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
    let vec3a: Vec<DateTimeL> = vec![dt1_a, dt2_a, dt3];
    let (i_, dt_) = match datetime_soonest2(&vec3a) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None3a");
        }
    };
    assert_eq!(i_, 2);
    assert_eq!(dt_, dt3);

    let vec3b: Vec<DateTimeL> = vec![dt1_a, dt3, dt2_a];
    let (i_, dt_) = match datetime_soonest2(&vec3b) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None3b");
        }
    };
    assert_eq!(i_, 1);
    assert_eq!(dt_, dt3);

    let vec3c: Vec<DateTimeL> = vec![dt3, dt1_a, dt2_a];
    let (i_, dt_) = match datetime_soonest2(&vec3c) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None3c");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt3);

    dpxf!();
}
