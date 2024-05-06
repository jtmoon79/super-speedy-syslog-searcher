// src/tests/syslinereader_tests.rs

//! tests for `syslinereader.rs`

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::too_many_arguments)]

use crate::common::{
    CharSz,
    Count,
    FileOffset,
    FPath,
    ResultS3,
};
use crate::data::line::{
    LinePart,
    Line,
    LineP,
    LineIndex
};
use crate::readers::blockreader::{
    Block,
    BlockIndex,
    BlockP,
    BlockSz,
};
use crate::readers::filepreprocessor::{
    fpath_to_filetype,
    PathToFiletypeResult,
};
use crate::readers::helpers::{fill, randomize};
use crate::readers::syslinereader::{
    DateTimeParseDatasIndexes,
    ResultS3SyslineFind,
    SyslineReader,
    SummarySyslineReader,
    ResultFindDateTime,
};
use crate::data::datetime::{
    datetime_parse_from_str,
    DateTimeL,
    DateTimeLOpt,
    DateTimeParseInstr,
    DateTimePattern_str,
    FixedOffset,
    TimeZone,
    Year,
    DateTimeParseInstrsIndex,
    DATETIME_PARSE_DATAS,
    DATETIME_PARSE_DATAS_LEN,
    ymdhmsn,
    O_L,
};
use crate::tests::datetime_tests::dt_pattern_has_tz;
use crate::debug::helpers::{
    create_temp_file,
    create_temp_file_bytes,
    ntf_fpath,
    NamedTempFile,
};
use crate::debug::printers::str_to_String_noraw;
#[allow(unused_imports)]
use crate::tests::common::{
    eprint_file,
    NTF_GZ_1BYTE_FPATH,
    NTF_GZ_8BYTE_FPATH,
    NTF_GZ_EMPTY_FPATH,
    NTF_LZ4_8BYTE_FPATH,
    NTF_XZ_1BYTE_FPATH,
    NTF_XZ_8BYTE_FPATH,
    NTF_LOG_EMPTY_FPATH,
    NTF_NL_1_PATH,
    NTF_NL_2_PATH,
    NTF_NL_3_PATH,
    NTF_NL_4_PATH,
    NTF_NL_5_PATH,
    NTF_SYSLINE_1_PATH,
    NTF_SYSLINE_1_SYSLINE1_DT,
    NTF_SYSLINE_2_PATH,
    NTF_SYSLINE_2_SYSLINE1_DT,
    NTF_SYSLINE_2_SYSLINE2_DT,
    NTF_TAR_0BYTE_FILEA_FPATH,
    NTF_TAR_1BYTE_FILEA_FPATH,
    NTF_TAR_1BYTE_FPATH,
    NTF_TAR_8BYTE_FILEA_FPATH,
    NTF_WNL_1_PATH,
    FO_0,
    FO_L,
    FO_P5,
    FO_P8,
    FO_M5,
    FO_M8,
};

use std::io::Write; // for `flush()`
use std::str;

use ::bstr::ByteSlice;
use ::const_format::concatcp;
use ::lazy_static::lazy_static;
#[allow(unused_imports)]
use ::more_asserts::{assert_gt, assert_le};
#[allow(unused_imports)]
use ::si_trace_print::{defn, defo, defx, deo, stack::stack_offset_set};
use ::test_case::test_case;


/// check the return type of `find_sysline` to this dummy approximation of `ResultS3SyslineFind`
pub type ResultS3SyslineFind_Test = ResultS3<(), std::io::Error>;

const FOUND: ResultS3SyslineFind_Test = ResultS3SyslineFind_Test::Found(());
const DONE: ResultS3SyslineFind_Test = ResultS3SyslineFind_Test::Done;

/// Helper to wrap the match and panic checks
fn new_SyslineReader(
    path: &FPath,
    blocksz: BlockSz,
    tzo: FixedOffset,
) -> SyslineReader {
    stack_offset_set(Some(2));
    let result = fpath_to_filetype(path, true);
    let filetype = match result {
        PathToFiletypeResult::Filetype(ft) => ft,
        PathToFiletypeResult::Archive(..) => {
            panic!("ERROR: fpath_to_filetype({:?}) returned an PathToFiletypeResult::Archive", path);
        }
    };
    match SyslineReader::new(path.clone(), filetype, blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({:?}, {:?}, {:?}) failed {}", path, blocksz, tzo, err);
        }
    }
}

#[test]
fn test_new_SyslineReader_1() {
    new_SyslineReader(&NTF_LOG_EMPTY_FPATH, 1024, *FO_P8);
}

#[test]
#[should_panic]
fn test_new_SyslineReader_2_bad_path_panics() {
    new_SyslineReader(&FPath::from("THIS/PATH_DOES/NOT///EXIST!!!"), 1024, *FO_P8);
}

// ----
// NTF5

/// - `FileOffset` is the input
/// - second parameter is expected result enum
/// - third parameter is the expected sysline string value
type TestFindSyslineCheck<'a> = (FileOffset, ResultS3SyslineFind_Test, &'a str);
type TestFindSyslineChecks<'a> = Vec<TestFindSyslineCheck<'a>>;

const NTF5_DATA_LINE0: &str =
    "[20200113-11:03:06] [DEBUG] Testing if xrdp can listen on 0.0.0.0 port 3389.\n";
const NTF5_DATA_LINE1: &str = "[20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
CLOSED!\n";
const NTF5_DATA_LINE2: &str = "[20200113-11:03:08] [INFO ] starting xrdp with pid 23198\n";
const NTF5_DATA_LINE3: &str = "[20200113-11:13:59] [DEBUG] Certification found
    FOUND CERTIFICATE!\n";
const NTF5_DATA_LINE4: &str = "[20200113-11:13:59] [DEBUG] Certification complete.\n";

const NTF5_DATA: &str =
    concatcp!(NTF5_DATA_LINE0, NTF5_DATA_LINE1, NTF5_DATA_LINE2, NTF5_DATA_LINE3, NTF5_DATA_LINE4,);

const NTF5_DATA_LINE0_OFFSET: usize = 0;
const NTF5_DATA_LINE1_OFFSET: usize = NTF5_DATA_LINE0
    .as_bytes()
    .len();
const NTF5_DATA_LINE2_OFFSET: usize = NTF5_DATA_LINE1_OFFSET
    + NTF5_DATA_LINE1
        .as_bytes()
        .len();
const NTF5_DATA_LINE3_OFFSET: usize = NTF5_DATA_LINE2_OFFSET
    + NTF5_DATA_LINE2
        .as_bytes()
        .len();
const NTF5_DATA_LINE4_OFFSET: usize = NTF5_DATA_LINE3_OFFSET
    + NTF5_DATA_LINE3
        .as_bytes()
        .len();

lazy_static! {
    static ref NTF5: NamedTempFile = create_temp_file(NTF5_DATA);
    static ref NTF5_PATH: FPath = ntf_fpath(&NTF5);
}

/// basic test of `SyslineReader.find_sysline`
fn impl_find_sysline(
    cache: bool,
    blocksz: BlockSz,
    checks: TestFindSyslineChecks,
) {
    stack_offset_set(Some(2));
    defn!("(cache {:?}, blocksz {:?})", cache, blocksz);
    let mut slr = new_SyslineReader(&NTF5_PATH, blocksz, *FO_P8);
    if !cache {
        slr.LRU_cache_disable();
    }
    for (fo, result_expect, sline_expect) in checks.iter() {
        defo!("slr.find_sysline({})", fo);
        let result_actual = slr.find_sysline(*fo);
        assert_results4(fo, result_expect, &result_actual);
        match result_actual {
            ResultS3SyslineFind::Found((_fo, slp)) => {
                let slp_string = (*slp).to_String();
                defo!("slr.find_sysline({}) Found", fo);
                defo!("expected {:?}", sline_expect);
                defo!("actual   {:?}", &slp_string.as_str());
                assert_eq!(
                    sline_expect,
                    &slp_string.as_str(),
                    "\nExpected {:?}\nActual   {:?}\n",
                    sline_expect,
                    &slp_string.as_str()
                );
            }
            ResultS3SyslineFind::Done => {
                defo!("slr.find_sysline({}) Done", fo);
            }
            ResultS3SyslineFind::Err(err) => {
                panic!("ERROR: impl_find_sysline: slr.find_sysline({}) returned Err({})", fo, err);
            }
        }
    }

    defx!();
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
fn test_find_sysline_A0(
    cache: bool,
    blocksz: BlockSz,
) {
    let checks: TestFindSyslineChecks = vec![
        (NTF5_DATA_LINE0_OFFSET as FileOffset, FOUND, NTF5_DATA_LINE0),
        (NTF5_DATA_LINE1_OFFSET as FileOffset, FOUND, NTF5_DATA_LINE1),
        (NTF5_DATA_LINE2_OFFSET as FileOffset, FOUND, NTF5_DATA_LINE2),
        (NTF5_DATA_LINE3_OFFSET as FileOffset, FOUND, NTF5_DATA_LINE3),
        (NTF5_DATA_LINE4_OFFSET as FileOffset, FOUND, NTF5_DATA_LINE4),
    ];
    impl_find_sysline(cache, blocksz, checks);
}

// -------
// helpers

/// Helper for common operations in `test_find_datetime_in_line`
/// and `test_parse_datetime_in_line_cached__no_cache`
fn helper_extract_dtpi_info(
    dtpi: &DateTimeParseInstr,
    index: DateTimeParseInstrsIndex,
    test_case_index: usize,
) -> (
    LineIndex,
    LineIndex,
    BlockP,
    Line,
    DateTimeParseDatasIndexes,
    Year,
    FixedOffset,
    String,
    DateTimeL,
    String,
)
{
    let (a_expect, b_expect, zymdhmsn, data) = dtpi._test_cases.iter().nth(test_case_index).unwrap();
    let block: Block = Block::from(data.as_bytes());
    let blockp: BlockP = BlockP::new(block);
    let linepart = LinePart::new(
        blockp.clone(),
        0,
        data.len() as BlockIndex,
        0,
        0,
        data.len() as BlockSz,
    );
    let line = Line::new_from_linepart(linepart);
    let mut indexes: DateTimeParseDatasIndexes = DateTimeParseDatasIndexes::new();
    indexes.push(index);
    let year = zymdhmsn.1 as Year;
    let fo: FixedOffset = match zymdhmsn.0 {
        O_L => *FO_L,
        val if val < 0 => FixedOffset::west_opt(-val).unwrap(),
        val if val >= 0 => FixedOffset::east_opt(val).unwrap(),
        val => panic!("bad offset value {:?}", val),
    };
    let fo_string: String = fo.to_string();
    let dt_expect: DateTimeL = ymdhmsn(
        &fo,
        year,
        zymdhmsn.2,
        zymdhmsn.3,
        zymdhmsn.4,
        zymdhmsn.5,
        zymdhmsn.6,
        zymdhmsn.7,
    );

    (
        *a_expect,
        *b_expect,
        blockp,
        line,
        indexes,
        year,
        fo,
        fo_string,
        dt_expect,
        dt_expect.to_string(),
    )
}

/// test `SyslineReader::find_datetime_in_line`
#[test]
fn test_find_datetime_in_line() {
    let mut regex_captures_attempted: Count = 0;
    let mut get_boxptrs_singleptr: Count = 0;
    let mut get_boxptrs_doubleptr: Count = 0;
    let mut get_boxptrs_multiptr: Count = 0;
    let mut ezcheck12_hit: Count = 0;
    let mut ezcheck12_miss: Count = 0;
    let mut ezcheck12_hit_max: LineIndex = 0;
    let mut ezcheckd2_hit: Count = 0;
    let mut ezcheckd2_miss: Count = 0;
    let mut ezcheckd2_hit_max: LineIndex = 0;
    let mut ezcheck12d2_hit: Count = 0;
    let mut ezcheck12d2_miss: Count = 0;
    let mut ezcheck12d2_hit_max: LineIndex = 0;

    for (i, dtpi) in DATETIME_PARSE_DATAS
        .iter()
        .enumerate()
    {
        let mut j: usize = 0;
        while j < dtpi._test_cases.len() {
            defo!("test case {:?} {:?} (line num {})", i, j, dtpi._line_num);
            let (
                a_expect,
                b_expect,
                _blockp,
                line,
                indexes,
                year,
                fo,
                fo_string,
                dt_expect,
                dt_expect_string,
            ) = helper_extract_dtpi_info(dtpi, i, j);
            // assert `SyslineReader::find_datetime_in_line`
            defo!("SyslineReader::find_datetime_in_line(...)");
            match SyslineReader::find_datetime_in_line(
                &line,
                &indexes,
                1 as CharSz,
                &Some(year),
                &fo,
                &fo_string,
                &mut get_boxptrs_singleptr,
                &mut get_boxptrs_doubleptr,
                &mut get_boxptrs_multiptr,
                &mut regex_captures_attempted,
                &mut ezcheck12_hit,
                &mut ezcheck12_miss,
                &mut ezcheck12_hit_max,
                &mut ezcheckd2_hit,
                &mut ezcheckd2_miss,
                &mut ezcheckd2_hit_max,
                &mut ezcheck12d2_hit,
                &mut ezcheck12d2_miss,
                &mut ezcheck12d2_hit_max,
                &FPath::from("DUMMY PATH"),
            ) {
                ResultFindDateTime::Ok((a_actual, b_actual, dt_actual, index)) => {
                    assert_eq!(
                        a_expect,
                        a_actual,
                        "\nfind_datetime_in_line() a\nExpected {:?}\nActual   {:?}\n",
                        a_expect,
                        a_actual,
                    );
                    assert_eq!(
                        b_expect,
                        b_actual,
                        "\nfind_datetime_in_line() b\nExpected {:?}\nActual   {:?}\n",
                        b_expect,
                        b_actual,
                    );
                    let dt_actual_string = dt_actual.to_string();
                    assert_eq!(
                        dt_expect,
                        dt_actual,
                        "\nfind_datetime_in_line() dt\nExpected {:?}\nActual   {:?}\n",
                        dt_expect_string,
                        dt_actual_string,
                    );
                    assert_eq!(
                        i,
                        index,
                        "\nfind_datetime_in_line() index\nExpected {:?}\nActual   {:?}\n",
                        i,
                        index,
                    );
                }
                ResultFindDateTime::Err(err) => {
                    panic!("returned Error; failed to match test line at index {:?}, case {:?}; Error {:?}", i, j, err);
                }
            }
            eprintln!();
            j += 1;
        }
    }
}

/// test `SyslineReader.parse_datetime_in_line_cached`
#[test]
fn test_parse_datetime_in_line_cached__no_cache() {
    let mut slr = new_SyslineReader(&*NTF_LOG_EMPTY_FPATH, 0x100, *FO_L);
    slr.LRU_cache_disable();

    for (i, dtpi) in DATETIME_PARSE_DATAS
        .iter()
        .enumerate()
    {
        let mut j: usize = 0;
        while j < dtpi._test_cases.len() {
            defo!("test case {:?} {:?} (line num {})", i, j, dtpi._line_num);
            let (
                a_expect,
                b_expect,
                blockp,
                line,
                _indexes,
                year,
                _fo,
                _fo_string,
                dt_expect,
                dt_expect_string,
            ) = helper_extract_dtpi_info(dtpi, i, j);
            let linep = LineP::new(line);
            slr.dt_patterns_reset();
            slr.dt_patterns_indexes_refresh();
            slr.dt_patterns_update(i);
            // this assert repeats `test_dt_pattern_index_max_count` which is fine
            assert_eq!(
                slr.dt_pattern_index_max_count(), i,
                "unexpected dt_pattern_index_max_count, expected {}", i
            );
            // assert `SyslineReader::parse_datetime_in_line_cached`
            defo!("slr.parse_datetime_in_line_cached(...)");
            match slr.parse_datetime_in_line_cached(
                &linep,
                1 as CharSz,
                &Some(year)
            ) {
                ResultFindDateTime::Ok((a_actual, b_actual, dt_actual, index)) => {
                    eprintln!("data {:?}", &(*blockp).as_bstr());
                    let slice_ = &(*blockp)[a_actual..b_actual];
                    eprintln!("data datetime substring [{}..{}] {:?}", a_actual, b_actual, slice_.as_bstr());
                    assert_eq!(
                        a_expect,
                        a_actual,
                        "\nparse_datetime_in_line_cached() a\nExpected {:?}\nActual   {:?}\n",
                        a_expect,
                        a_actual,
                    );
                    assert_eq!(
                        b_expect,
                        b_actual,
                        "\nparse_datetime_in_line_cached() b\nExpected {:?}\nActual   {:?}\n",
                        b_expect,
                        b_actual,
                    );
                    let dt_actual_string = dt_actual.to_string();
                    assert_eq!(
                        dt_expect,
                        dt_actual,
                        "\nparse_datetime_in_line_cached() dt\nExpected {:?}\nActual   {:?}\n",
                        dt_expect_string,
                        dt_actual_string,
                    );
                    assert_eq!(
                        i,
                        index,
                        "\nfind_datetime_in_line() index\nExpected {:?}\nActual   {:?}\n",
                        i,
                        index,
                    );
                }
                ResultFindDateTime::Err(err) => {
                    panic!("returned Error; failed to match test line at index {:?}, case {:?}; Error {:?}", i, j, err);
                }
            }
            eprintln!();
            j += 1;
        }
    }
}

/// test `SyslineReader.test_dt_pattern_index_max_count`
#[test]
fn test_dt_pattern_index_max_count() {
    let mut slr = new_SyslineReader(&*NTF_LOG_EMPTY_FPATH, 0x100, *FO_L);
    let mut i: usize = 0;
    while i < DATETIME_PARSE_DATAS_LEN {
        slr.dt_patterns_reset();
        slr.dt_patterns_indexes_refresh();
        slr.dt_patterns_update(i);
        let c = slr.dt_pattern_index_max_count();
        assert_eq!(
            c, i,
            "unexpected dt_pattern_index_max_count {}, expected {}", c, i
        );
        i += 1;
    }
}

/// - First `FileOffset` input to `find_sysline_at_datetime_filter`
/// - Second `&str` input to `datetime_parse_from_str`
///   The datetime `str` transformed to `DateTimeL` and then passed to
///   `syslinereader.find_sysline_at_datetime_filter(FileOffset, Some(DateTimeL))`.
///
/// - Third `ResultS3SyslineFind_Test` is the expected return.
/// - Fourth (last) `str` is the expected sysline data, in `str` form, returned (this is the tested
///   comparison).
type TestFindSyslineAtDatetimeFilterCheck<'a> = (FileOffset, &'a str, ResultS3SyslineFind_Test, &'a str);
type TestFindSyslineAtDatetimeFilterChecks<'a> = Vec<TestFindSyslineAtDatetimeFilterCheck<'a>>;

/// underlying test code for `SyslineReader.find_datetime_in_line`
/// called by other functions `test_find_sysline_at_datetime_filterX`
fn impl_test_find_sysline_at_datetime_filter(
    ntf: &NamedTempFile,
    dt_pattern: &DateTimePattern_str,
    cache: bool,
    blocksz: BlockSz,
    checks: TestFindSyslineAtDatetimeFilterChecks,
) {
    defn!("(…, {:?}, {}, …)", dt_pattern, blocksz);

    let path = ntf_fpath(ntf);
    let tzo: FixedOffset = *FO_M8;
    let mut slr = new_SyslineReader(&path, blocksz, tzo);
    if !cache {
        slr.LRU_cache_disable();
    }
    for (fo1, dts, result_expect, sline_expect) in checks.iter() {
        // TODO: add `has_tz` to `checks`
        let has_tz = dt_pattern_has_tz(dt_pattern);
        defo!(
            "datetime_parse_from_str({:?}, {:?}, {:?}, {:?})",
            str_to_String_noraw(dts),
            dt_pattern,
            has_tz,
            &tzo
        );
        let dt = match datetime_parse_from_str(dts, dt_pattern, has_tz, &tzo) {
            Some(val) => val,
            None => {
                panic!("ERROR: datetime_from_str({:?}, {:?}) returned None", dts, dt_pattern);
            }
        };
        let sline_expect_noraw = str_to_String_noraw(sline_expect);
        defo!("find_sysline_at_datetime_filter({}, {:?})", fo1, dt);
        let result = slr.find_sysline_at_datetime_filter(*fo1, &Some(dt));
        assert_results4(fo1, result_expect, &result);
        match result {
            ResultS3SyslineFind::Found(val) => {
                let sline = val.1.to_String();
                let sline_noraw = str_to_String_noraw(sline.as_str());
                defo!("expected: {:?}", sline_expect_noraw);
                defo!("returned: {:?}", sline_noraw);
                let sline_expect_string = String::from(*sline_expect);
                assert_eq!(
                    sline, sline_expect_string,
                    "Expected {:?} == {:?} but it is not!",
                    sline_noraw, sline_expect_noraw
                );
                defo!(
                    "Check PASSED SyslineReader().find_sysline_at_datetime_filter({}, {:?}) == {:?}",
                    fo1, dts, sline_noraw
                );
            }
            ResultS3SyslineFind::Done => {}
            ResultS3SyslineFind::Err(err) => {
                panic!("During test unexpected result Error {}", err);
            }
        }
    }

    defx!();
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
// blah, this is a lot of work…
const NTF26_DATA_DT13: &str = "2020-01-01 00:00:13";
const NTF26_DATA_LINE13n: &str = "2020-01-01 00:00:13abcdefghijklm\n";
const NTF26_DATA_DT24: &str = "2020-01-01 00:00:24";
const NTF26_DATA_LINE24n: &str = "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n";
const NTF26_DATA_DT25: &str = "2020-01-01 00:00:25";
const NTF26_DATA_LINE25n: &str = "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n";
const NTF26_DATA_DT26: &str = "2020-01-01 00:00:26";
const NTF26_DATA_LINE26n: &str = "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n";

lazy_static! {
    static ref NTF26: NamedTempFile = {
        create_temp_file(NTF26_DATA)
    };
    static ref NTF26_PATH: FPath = {
        ntf_fpath(&NTF26)
    };

    /// exhaustive checks (one for every sysline)
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
            (0, NTF26_DATA_DT13, FOUND, NTF26_DATA_LINE13n),
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
            //(99999, NTF26_DATA_DT26, DONE, ""),
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
            (337, NTF26_DATA_DT13, FOUND, NTF26_DATA_LINE13n),
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
/// This function does a copy of a `ResultS3SyslineFind_Test`, dropping any particular `Error` for
/// a generic one.
fn copy_ResultS3(result: &ResultS3SyslineFind_Test) -> ResultS3SyslineFind_Test {
    match result {
        ResultS3SyslineFind_Test::Found(_) => FOUND,
        ResultS3SyslineFind_Test::Done => DONE,
        ResultS3SyslineFind_Test::Err(_err) =>
        // use a generic filler error; particulars are not important for testing
        {
            ResultS3SyslineFind_Test::Err(std::io::Error::new(std::io::ErrorKind::Other, "FILLER"))
        }
    }
}

fn NTF26_checks_copy() -> TestFindSyslineAtDatetimeFilterChecks<'static> {
    // must manually copy `NTF26_checks`
    let mut checks = TestFindSyslineAtDatetimeFilterChecks::with_capacity(NTF26_checks.len());
    for check in NTF26_checks.iter() {
        let result_expect = copy_ResultS3(&check.2);
        checks.push((check.0, check.1, result_expect, check.3))
    }

    checks
}

fn NTF26_checksx_copy() -> TestFindSyslineAtDatetimeFilterChecks<'static> {
    // must manually copy `NTF26_checksx`
    let mut checks = TestFindSyslineAtDatetimeFilterChecks::with_capacity(NTF26_checksx.len());
    for check in NTF26_checksx.iter() {
        let result_expect = copy_ResultS3(&check.2);
        checks.push((check.0, check.1, result_expect, check.3))
    }

    checks
}

/// wrapper for `impl_test_find_sysline_at_datetime_filter`
fn impl_test_find_sysline_at_datetime_filter_NTF26(
    cache: bool,
    blocksz: BlockSz,
    checks: Option<TestFindSyslineAtDatetimeFilterChecks>,
) {
    stack_offset_set(Some(1));
    defn!();

    // if passed `checks` then use that, otherwise use a copy of the static `NTF26_checks`
    let checks_ = match checks {
        Some(checks__) => checks__,
        None => NTF26_checks_copy(),
    };
    impl_test_find_sysline_at_datetime_filter(&NTF26, NTF26_DATETIME_FORMAT, cache, blocksz, checks_);
    defx!();
}

// XXX: are these different BlockSz tests necessary? are not these adequately tested by
//      other lower-level tests?

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
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
fn test_find_sysline_at_datetime_filter_NTF26(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(cache, blocksz, None);
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
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
fn test_find_sysline_at_datetime_filter_NTF26_checksx(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(cache, blocksz, Some(NTF26_checksx_copy()));
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 64; "cache_64")]
#[test_case(false, 64; "nocache_64")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n)])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 128; "cache_128")]
#[test_case(false, 128; "nocache_128")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_a(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n)])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_b(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n)])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_c(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(0, NTF26_DATA_DT3, FOUND, NTF26_DATA_LINE3n)])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_d(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(0, NTF26_DATA_DT4, FOUND, NTF26_DATA_LINE4n)])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_e(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:05",
            FOUND,
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_f(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:06",
            FOUND,
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_g(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:07",
            FOUND,
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_h(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:08",
            FOUND,
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_i(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:09",
            FOUND,
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_j(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:10",
            FOUND,
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_k(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:11",
            FOUND,
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_l(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:12",
            FOUND,
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_m(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:13",
            FOUND,
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_n(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:14",
            FOUND,
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_o(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:15",
            FOUND,
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_p(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:16",
            FOUND,
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_q(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:17",
            FOUND,
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_r(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:18",
            FOUND,
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_s(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:19",
            FOUND,
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_t(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:20",
            FOUND,
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_u(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:21",
            FOUND,
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_v(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:22",
            FOUND,
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_w(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            0,
            "2020-01-01 00:00:23",
            FOUND,
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_x(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(0, NTF26_DATA_DT24, FOUND, NTF26_DATA_LINE24n)])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_y(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n)])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_0_z(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n)])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_a(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(19, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n)])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_b(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(40, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n)])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_c(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(62, NTF26_DATA_DT3, FOUND, NTF26_DATA_LINE3n)])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_d(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(85, NTF26_DATA_DT4, FOUND, NTF26_DATA_LINE4n)])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_e(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            109,
            "2020-01-01 00:00:05",
            FOUND,
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_f(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            134,
            "2020-01-01 00:00:06",
            FOUND,
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_g(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            160,
            "2020-01-01 00:00:07",
            FOUND,
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_h(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            187,
            "2020-01-01 00:00:08",
            FOUND,
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_i(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            215,
            "2020-01-01 00:00:09",
            FOUND,
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_j(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            244,
            "2020-01-01 00:00:10",
            FOUND,
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_k(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            274,
            "2020-01-01 00:00:11",
            FOUND,
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_l(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            305,
            "2020-01-01 00:00:12",
            FOUND,
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_m(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            337,
            "2020-01-01 00:00:13",
            FOUND,
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_n(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            370,
            "2020-01-01 00:00:14",
            FOUND,
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_o(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            404,
            "2020-01-01 00:00:15",
            FOUND,
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_p(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            439,
            "2020-01-01 00:00:16",
            FOUND,
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_q(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            475,
            "2020-01-01 00:00:17",
            FOUND,
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_r(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            512,
            "2020-01-01 00:00:18",
            FOUND,
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_s(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            550,
            "2020-01-01 00:00:19",
            FOUND,
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_t(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            589,
            "2020-01-01 00:00:20",
            FOUND,
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_u(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            629,
            "2020-01-01 00:00:21",
            FOUND,
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_v(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            670,
            "2020-01-01 00:00:22",
            FOUND,
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_w(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            712,
            "2020-01-01 00:00:23",
            FOUND,
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_x(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            755,
            NTF26_DATA_DT24,
            FOUND,
            NTF26_DATA_LINE24n,
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_y(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            799,
            NTF26_DATA_DT25,
            FOUND,
            NTF26_DATA_LINE25n,
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_x_z(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([(
            844,
            NTF26_DATA_DT26,
            FOUND,
            NTF26_DATA_LINE26n,
        )])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_2_z_(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_2_y_(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_2_x_(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT24, FOUND, NTF26_DATA_LINE24n),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_2_m_(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_2_za(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_2_ya(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_2_xa(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT24, FOUND, NTF26_DATA_LINE24n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_2_ma(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3____(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3__ab(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3__az(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3__bd(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT4, FOUND, NTF26_DATA_LINE4n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3__ml(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:12", FOUND, "2020-01-01 00:00:12abcdefghijkl\n"),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3__my(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3__mz(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3__m_(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, "2020-01-01 00:00:13", FOUND, "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_aaa(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_abc(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT3, FOUND, NTF26_DATA_LINE3n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_aba(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_abn(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, "2020-01-01 00:00:14", FOUND, "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_aby(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_abz(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_aaz(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_byo(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, "2020-01-01 00:00:15", FOUND, "2020-01-01 00:00:15abcdefghijklmno\n"),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_zaa(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_zbc(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT3, FOUND, NTF26_DATA_LINE3n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_zba(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_zbn(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, "2020-01-01 00:00:14", FOUND, "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_zby(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_zbz(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_zaz(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_yaa(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_ybc(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT3, FOUND, NTF26_DATA_LINE3n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_yba(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_ybn(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, "2020-01-01 00:00:14", FOUND, "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_yby(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_ybz(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test_case(true, 2; "cache_2")]
#[test_case(false, 2; "nocache_2")]
#[test_case(true, 4; "cache_4")]
#[test_case(false, 4; "nocache_4")]
#[test_case(true, 6; "cache_6")]
#[test_case(false, 6; "nocache_6")]
#[test_case(true, 8; "cache_8")]
#[test_case(false, 8; "nocache_8")]
#[test_case(true, 16; "cache_16")]
#[test_case(false, 16; "nocache_16")]
#[test_case(true, 32; "cache_32")]
#[test_case(false, 32; "nocache_32")]
#[test_case(true, 1024; "cache_1024")]
#[test_case(false, 1024; "nocache_1024")]
fn test_find_sysline_at_datetime_filter_checks_NTF26_3_yaz(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_at_datetime_filter_NTF26(
        cache,
        blocksz,
        Some(TestFindSyslineAtDatetimeFilterChecks::from([
            (0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
        ])),
    );
}

// -------------------------------------------------------------------------------------------------

/// - First `FileOffset` input to `find_sysline_between_datetime_filters`
/// - Second `&str` input to `datetime_parse_from_str`
/// - Third `&str` input to `datetime_parse_from_str`
///   Those datetime `str` transformed to `DateTimeL` and then passed to
///   `syslinereader.find_sysline_between_datetime_filters(FileOffset, Some(DateTimeL), Some(DateTimeL))`.
/// - Fourth `ResultS3SyslineFind_Test` is the expected return.
/// - Fifth (last) `str` is the expected sysline data, in `str` form, returned (this is the tested
///   comparison).
type TestFindSyslineBetweenDatetimeFilterCheck<'a> =
    (FileOffset, &'a str, &'a str, ResultS3SyslineFind_Test, &'a str);
type TestFindSyslineBetweenDatetimeFilterChecks<'a> = Vec<TestFindSyslineBetweenDatetimeFilterCheck<'a>>;

/// test `syslinereader.find_sysline_between_datetime_filters`
///
/// similar to `impl_test_find_sysline_at_datetime_filter`
#[allow(non_snake_case)]
fn impl_test_find_sysline_between_datetime_filter(
    ntf: &NamedTempFile,
    dt_pattern: &DateTimePattern_str,
    cache: bool,
    blocksz: BlockSz,
    checks: TestFindSyslineBetweenDatetimeFilterChecks,
) {
    defn!("(…, {:?}, {}, {}, …)", dt_pattern, cache, blocksz);

    let path = ntf_fpath(ntf);
    let tzo: FixedOffset = *FO_M8;
    let mut slr = new_SyslineReader(&path, blocksz, tzo);
    if !cache {
        slr.LRU_cache_disable();
    }
    for (fo1, dts_a, dts_b, result_expect, sline_expect) in checks.iter() {
        // TODO: add `has_tz` to `checks`
        let has_tz = dt_pattern_has_tz(dt_pattern);
        defo!(
            "datetime_parse_from_str({:?}, {:?}, {:?}, {:?})",
            str_to_String_noraw(dts_a),
            dt_pattern,
            has_tz,
            &tzo
        );
        let dt_a = match datetime_parse_from_str(dts_a, dt_pattern, has_tz, &tzo) {
            Some(val) => val,
            None => {
                panic!("ERROR: datetime_from_str({:?}, {:?}) returned None", dts_a, dt_pattern);
            }
        };

        let has_tz = dt_pattern_has_tz(dt_pattern);
        defo!(
            "datetime_parse_from_str({:?}, {:?}, {:?}, {:?})",
            str_to_String_noraw(dts_b),
            dt_pattern,
            has_tz,
            &tzo
        );
        let dt_b = match datetime_parse_from_str(dts_b, dt_pattern, has_tz, &tzo) {
            Some(val) => val,
            None => {
                panic!("ERROR: datetime_from_str({:?}, {:?}) returned None", dts_b, dt_pattern);
            }
        };

        let sline_expect_noraw = str_to_String_noraw(sline_expect);
        defo!("find_sysline_between_datetime_filters({}, {:?}, {:?})", fo1, dt_a, dt_b);
        let result = slr.find_sysline_between_datetime_filters(*fo1, &Some(dt_a), &Some(dt_b));
        assert_results4(fo1, result_expect, &result);
        match result {
            ResultS3SyslineFind::Found(val) => {
                let sline = val.1.to_String();
                let sline_noraw = str_to_String_noraw(sline.as_str());
                defo!("expected: {:?}", sline_expect_noraw);
                defo!("returned: {:?}", sline_noraw);
                let sline_expect_string = String::from(*sline_expect);
                assert_eq!(
                    sline, sline_expect_string,
                    "Expected {:?} == {:?} but it is not!",
                    sline_noraw, sline_expect_noraw
                );
                defo!(
                    "Check PASSED SyslineReader().find_sysline_between_datetime_filters({}, {:?}, {:?}) == {:?}",
                    fo1, dts_a, dts_b, sline_noraw
                );
            }
            ResultS3SyslineFind::Done => {}
            ResultS3SyslineFind::Err(err) => {
                panic!("During test unexpected result Error {}", err);
            }
        }
    }

    defx!();
}

lazy_static! {
    /// many checks, not exhaustive
    static ref NTF26B_checks: TestFindSyslineBetweenDatetimeFilterChecks<'static> = {
        Vec::from([
            (0, NTF26_DATA_DT0, NTF26_DATA_DT0, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT0, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT0, NTF26_DATA_DT13, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT0, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT0, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT1, NTF26_DATA_DT1, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, NTF26_DATA_DT13, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, NTF26_DATA_DT2, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT2, NTF26_DATA_DT3, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT2, NTF26_DATA_DT13, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT2, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT2, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT13, NTF26_DATA_DT13, FOUND, NTF26_DATA_LINE13n),
            (0, NTF26_DATA_DT13, NTF26_DATA_DT24, FOUND, NTF26_DATA_LINE13n),
            (0, NTF26_DATA_DT24, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE24n),
            (0, NTF26_DATA_DT24, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE24n),
            (0, NTF26_DATA_DT25, NTF26_DATA_DT25, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT25, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT26, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT26, NTF26_DATA_DT26, FOUND, NTF26_DATA_LINE26n),
            //(99999, NTF26_DATA_DT26, NTF26_DATA_DT26, DONE, ""),
        ])
    };
}

fn NTF26B_checks_copy() -> TestFindSyslineBetweenDatetimeFilterChecks<'static> {
    // must manually copy `NTF26_checks`
    let mut checks = TestFindSyslineBetweenDatetimeFilterChecks::with_capacity(NTF26B_checks.len());
    for check in NTF26B_checks.iter() {
        let result_expect = copy_ResultS3(&check.3);
        checks.push((check.0, check.1, check.2, result_expect, check.4))
    }

    checks
}

/// wrapper for  `impl_test_find_sysline_between_datetime_filter`
fn impl_test_find_sysline_between_datetime_filter_NTF26B(
    cache: bool,
    blocksz: BlockSz,
    checks: Option<TestFindSyslineBetweenDatetimeFilterChecks>,
) {
    stack_offset_set(Some(1));
    defn!();

    // if passed `checks` then use that, otherwise use a copy of the static `NTF26_checks`
    let checks_ = match checks {
        Some(checks__) => checks__,
        None => NTF26B_checks_copy(),
    };
    impl_test_find_sysline_between_datetime_filter(&NTF26, NTF26_DATETIME_FORMAT, cache, blocksz, checks_);
    defx!();
}

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
fn test_find_sysline_between_datetime_filter(
    cache: bool,
    blocksz: BlockSz,
) {
    impl_test_find_sysline_between_datetime_filter_NTF26B(cache, blocksz, None);
}

type TestSyslineReaderCheck<'a> = (&'a str, FileOffset);
type TestSyslineReaderChecks<'a> = Vec<(&'a str, FileOffset)>;

/// basic linear test of `SyslineReader::find_sysline`
#[allow(non_snake_case)]
fn impl_test_SyslineReader_find_sysline(
    path: &FPath,
    blocksz: BlockSz,
    fileoffset: FileOffset,
    checks: &TestSyslineReaderChecks,
) {
    stack_offset_set(Some(2));
    defn!("({:?}, {}, {})", path, blocksz, fileoffset);
    eprint_file(path);
    let tzo: FixedOffset = *FO_M8;
    let mut slr = new_SyslineReader(path, blocksz, tzo);

    let mut fo1: FileOffset = fileoffset;
    let mut check_i: usize = 0;
    loop {
        let result = slr.find_sysline(fo1);
        //let done = result.is_done() || result.is_eof();
        let done = result.is_done();
        match result {
            ResultS3SyslineFind::Found((fo, slp)) => {
                defo!("slr.find_sysline({}) returned Found({}, @{:p})", fo1, fo, &*slp);
                defo!(
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
                defo!(
                    "find_sysline({}); check {} expect ({:?}, {:?})",
                    fo1,
                    check_i,
                    checks[check_i].1,
                    checks[check_i].0
                );
                // check slp.String
                let check_String = checks[check_i].0.to_string();
                let actual_String = (*slp).to_String();
                assert_eq!(
                    check_String, actual_String,
                    "\nexpected string value     {:?}\nfind_sysline({:?}) returned {:?}\n",
                    check_String, fo1, actual_String
                );
                // check fileoffset
                let check_fo = checks[check_i].1;
                assert_eq!(
                    check_fo, fo,
                    "expected fileoffset {}, but find_sysline returned fileoffset {} for check {}",
                    check_fo, fo, check_i
                );
            }
            ResultS3SyslineFind::Done => {
                defo!("slr.find_sysline({}) returned Done", fo1);
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                defo!("slr.find_sysline({}) returned Err({})", fo1, err);
                panic!("ERROR: {}", err);
            }
        }
        check_i += 1;
        if done {
            break;
        }
    }
    assert_eq!(
        checks.len(),
        check_i,
        "expected {} Sysline checks but only {} Sysline checks were done",
        checks.len(),
        check_i
    );

    defo!("Found {} Lines, {} Syslines", slr.count_lines_processed(), slr.count_syslines_stored());
    defx!();
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
    static ref NTF_A1: NamedTempFile = create_temp_file(test_data_file_A1_dt6);
    static ref NTF_A1_path: FPath = ntf_fpath(&NTF_A1);
}

#[test]
fn test_find_sysline_A1_dt6_4_0_() {
    let checks = TestSyslineReaderChecks::from(test_data_file_A1_dt6_checks);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 4, 0, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_0_() {
    let checks = TestSyslineReaderChecks::from(test_data_file_A1_dt6_checks);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 0, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_1_() {
    let checks = TestSyslineReaderChecks::from(&test_data_file_A1_dt6_checks[1..]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 40, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_2_() {
    let checks = TestSyslineReaderChecks::from(&test_data_file_A1_dt6_checks[2..]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 62, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_3_() {
    let checks = TestSyslineReaderChecks::from(&test_data_file_A1_dt6_checks[3..]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 85, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_4_() {
    let checks = TestSyslineReaderChecks::from(&test_data_file_A1_dt6_checks[4..]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 86, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_X_beforeend() {
    let checks = TestSyslineReaderChecks::from([]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 132, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_X_pastend() {
    let checks = TestSyslineReaderChecks::from([]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 135, &checks);
}

#[test]
fn test_find_sysline_A1_dt6_128_X9999() {
    let checks = TestSyslineReaderChecks::from([]);
    impl_test_SyslineReader_find_sysline(&NTF_A1_path, 128, 9999, &checks);
}

// -----------------------------------------------------------------------------

/// Helper to assert `find_sysline` return enum
fn assert_results4(
    fo: &FileOffset,
    result_expect: &ResultS3SyslineFind_Test,
    result_actual: &ResultS3SyslineFind,
) {
    let actual: String = format!("{}", result_actual);
    match result_expect {
        ResultS3SyslineFind_Test::Found(_) => {
            assert!(
                matches!(result_actual, ResultS3SyslineFind::Found(_)),
                "\n  for find_sysline({})\n  Expected Found\n  Actual {:?}\n",
                fo,
                actual,
            );
        }
        ResultS3SyslineFind_Test::Done => {
            assert!(
                matches!(result_actual, ResultS3SyslineFind::Done),
                "\n  for find_sysline({})\n  Expected Done\n  Actual {:?}\n",
                fo,
                actual,
            );
        }
        _ => {
            panic!("Unexpected result_expect");
        }
    }
}

// -----------------------------------------------------------------------------

type TestSyslineReaderAnyInputCheck<'a> = (FileOffset, ResultS3SyslineFind_Test, FileOffset, &'a str);
type TestSyslineReaderAnyInputChecks<'a> = Vec<TestSyslineReaderAnyInputCheck<'a>>;

/// test of `SyslineReader::find_sysline` with test-specified fileoffset searches
#[allow(non_snake_case)]
fn impl_test_findsysline(
    path: &FPath,
    blocksz: BlockSz,
    cache: bool,
    input_checks: &[TestSyslineReaderAnyInputCheck],
) {
    stack_offset_set(Some(2));
    defn!("({:?}, {})", path, blocksz);
    eprint_file(path);
    let tzo: FixedOffset = *FO_M8;
    let mut slr = new_SyslineReader(path, blocksz, tzo);
    if !cache {
        slr.LRU_cache_disable();
    }

    let mut check_i: usize = 0;
    let mut done_analysis = false;
    for (input_fo, expect_result, expect_fo, expect_val) in input_checks.iter() {
        let result = slr.find_sysline(*input_fo);
        assert_results4(input_fo, expect_result, &result);
        match result {
            ResultS3SyslineFind::Found((fo, slp)) => {
                defo!("slr.find_sysline({}) returned Found({}, @{:p})", input_fo, fo, &*slp);
                defo!(
                    "FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    fo,
                    &(*slp),
                    slp.count_lines(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );

                let actual_String = (*slp).to_String();
                let expect_String = String::from(*expect_val);
                defo!("find_sysline({}); check {}", input_fo, check_i);
                assert_eq!(
                    expect_String, actual_String,
                    "\nexpected string value     {:?}\nfind_sysline({:?}) returned {:?}\n",
                    expect_String, input_fo, actual_String,
                );
                assert_eq!(
                    &fo, expect_fo,
                    "mismatched fileoffsets returned by file_sysline({}); expected {}, actual {}, filesz {}",
                    *input_fo, expect_fo, fo, slr.filesz(),
                );

                if !done_analysis {
                    defo!("force pattern analysis based on first Found");
                    done_analysis = true;
                    assert!(slr.dt_patterns_analysis(), "dt_patterns_analysis() failed");
                }
            }
            ResultS3SyslineFind::Done => {
                defo!("slr.find_sysline({}) returned Done", input_fo);
            }
            ResultS3SyslineFind::Err(err) => {
                defo!("slr.find_sysline({}) returned Err({})", input_fo, err);
                panic!("ERROR: {}", err);
            }
        }
        check_i += 1;
        eprintln!("\n\n");
    }
    assert_eq!(
        input_checks.len(),
        check_i,
        "expected {} Sysline checks but only {} Sysline checks were done",
        input_checks.len(),
        check_i
    );

    defo!("Found {} Lines, {} Syslines", slr.count_lines_processed(), slr.count_syslines_stored());

    let cache_actual = slr.LRU_cache_disable();
    assert_eq!(cache, cache_actual, "Expected cache_enabled to be {}, it was {}", cache, cache_actual);

    defx!("({:?}, {})", &path, blocksz);
}

const test_data_A2_dt6_sysline0: &str = "2000-01-01 00:00:00\n";
const test_data_A2_dt6_sysline1: &str = "2000-01-01 00:00:01a\n";
const test_data_A2_dt6_sysline2: &str = "2000-01-01 00:00:02ab\n";
const test_data_A2_dt6_sysline3: &str = "2000-01-01 00:00:03abc\n";
const test_data_A2_dt6_sysline4: &str = "2000-01-01 00:00:04abcd\n";
const test_data_A2_dt6_sysline5: &str = "2000-01-01 00:00:05abcde";

const test_data_A2_dt6: &str = concatcp!(
    test_data_A2_dt6_sysline0,
    test_data_A2_dt6_sysline1,
    test_data_A2_dt6_sysline2,
    test_data_A2_dt6_sysline3,
    test_data_A2_dt6_sysline4,
    test_data_A2_dt6_sysline5,
);

// remember that `.len()` starts at one, and `FileOffset` starts at zero

const test_data_A2_dt6_sysline0_end: FileOffset = test_data_A2_dt6_sysline0.len() as FileOffset;
const test_data_A2_dt6_sysline1_end: FileOffset = test_data_A2_dt6_sysline1.len() as FileOffset + test_data_A2_dt6_sysline0_end;
const test_data_A2_dt6_sysline2_end: FileOffset = test_data_A2_dt6_sysline2.len() as FileOffset + test_data_A2_dt6_sysline1_end;
const test_data_A2_dt6_sysline3_end: FileOffset = test_data_A2_dt6_sysline3.len() as FileOffset + test_data_A2_dt6_sysline2_end;
const test_data_A2_dt6_sysline4_end: FileOffset = test_data_A2_dt6_sysline4.len() as FileOffset + test_data_A2_dt6_sysline3_end;
const test_data_A2_dt6_sysline5_end: FileOffset = test_data_A2_dt6_sysline5.len() as FileOffset + test_data_A2_dt6_sysline4_end;

type TestDataA2Dt6Checks = [TestSyslineReaderAnyInputCheck<'static>; 141];
type TestDataA2Dt6ChecksVec = Vec<TestSyslineReaderAnyInputCheck<'static>>;

const test_data_A2_dt6_checks_many: TestDataA2Dt6Checks = [
    (0, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (1, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (2, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (3, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (4, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (5, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (6, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (7, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (8, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (9, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (10, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (11, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (12, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (13, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (14, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (15, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (16, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (17, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (18, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (19, FOUND, test_data_A2_dt6_sysline0_end, test_data_A2_dt6_sysline0),
    (20, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (21, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (22, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (23, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (24, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (25, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (26, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (27, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (28, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (29, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (30, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (31, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (32, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (33, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (34, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (35, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (36, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (37, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (38, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (39, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (40, FOUND, test_data_A2_dt6_sysline1_end, test_data_A2_dt6_sysline1),
    (41, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (42, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (43, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (44, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (45, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (46, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (47, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (48, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (49, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (50, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (51, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (52, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (53, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (54, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (55, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (56, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (57, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (58, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (59, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (60, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (61, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (62, FOUND, test_data_A2_dt6_sysline2_end, test_data_A2_dt6_sysline2),
    (63, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (64, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (65, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (66, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (67, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (68, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (69, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (70, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (71, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (72, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (73, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (74, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (75, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (76, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (77, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (78, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (79, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (80, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (81, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (82, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (83, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (84, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (85, FOUND, test_data_A2_dt6_sysline3_end, test_data_A2_dt6_sysline3),
    (86, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (87, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (88, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (89, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (90, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (91, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (92, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (93, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (94, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (95, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (96, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (97, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (98, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (99, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (100, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (101, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (102, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (103, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (104, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (105, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (106, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (107, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (108, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (109, FOUND, test_data_A2_dt6_sysline4_end, test_data_A2_dt6_sysline4),
    (110, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (111, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (112, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (113, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (114, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (115, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (116, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (117, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (118, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (119, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (120, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (121, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (122, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (123, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (124, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (125, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (126, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (127, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (128, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (129, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (130, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (131, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (132, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (133, FOUND, test_data_A2_dt6_sysline5_end, test_data_A2_dt6_sysline5),
    (134, DONE, 0, ""),
    (135, DONE, 0, ""),
    (136, DONE, 0, ""),
    (137, DONE, 0, ""),
    (138, DONE, 0, ""),
    (139, DONE, 0, ""),
    (140, DONE, 0, ""),
];

lazy_static! {
    static ref test_data_A2_dt6_ntf: NamedTempFile = create_temp_file(test_data_A2_dt6);
    static ref test_data_A2_dt6_ntf_path: FPath = ntf_fpath(&test_data_A2_dt6_ntf);
}

fn copy_TestDataA2Dt6Checks(checks: &TestDataA2Dt6Checks) -> TestDataA2Dt6ChecksVec {
    let mut data = TestDataA2Dt6ChecksVec::new();
    // PitA to copy because std::io::Error does not copy or clone
    // discussed in https://github.com/rust-lang-deprecated/failure/issues/148
    for check in checks {
        let result = match &check.1 {
            ResultS3::Found(_) => FOUND,
            ResultS3::Done => DONE,
            ResultS3::Err(_) => ResultS3SyslineFind_Test::Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "dummy error",
                )
            ),
        };
        data.push((check.0, result, check.2, check.3));
    }

    data
}

#[test_case(true, false, false, 2, &test_data_A2_dt6_checks_many; "cache_2")]
#[test_case(false, false, false, 2, &test_data_A2_dt6_checks_many; "nocache_2")]
#[test_case(true, true, false, 2, &test_data_A2_dt6_checks_many; "cache_2_reverse")]
#[test_case(false, true, false, 2, &test_data_A2_dt6_checks_many; "nocache_2_reverse")]
#[test_case(true, false, true, 2, &test_data_A2_dt6_checks_many; "cache_2_swapends")]
#[test_case(false, false, true, 2, &test_data_A2_dt6_checks_many; "nocache_2_swapends")]
#[test_case(true, true, true, 2, &test_data_A2_dt6_checks_many; "cache_2_reverse_swapends")]
#[test_case(false, true, true, 2, &test_data_A2_dt6_checks_many; "nocache_2_reverse_swapends")]
#[test_case(true, false, false, 4, &test_data_A2_dt6_checks_many; "cache_4")]
#[test_case(false, false, false, 4, &test_data_A2_dt6_checks_many; "nocache_4")]
#[test_case(true, true, false, 4, &test_data_A2_dt6_checks_many; "cache_4_reverse")]
#[test_case(false, true, false, 4, &test_data_A2_dt6_checks_many; "nocache_4_reverse")]
#[test_case(true, false, false, 0xF, &test_data_A2_dt6_checks_many; "cache_0xF")]
#[test_case(false, false, false, 0xF, &test_data_A2_dt6_checks_many; "nocache_0xF")]
#[test_case(true, true, false, 0xF, &test_data_A2_dt6_checks_many; "cache_0xF_reverse")]
#[test_case(false, true, false, 0xF, &test_data_A2_dt6_checks_many; "nocache_0xF_reverse")]
#[test_case(true, false, false, 0xFF, &test_data_A2_dt6_checks_many; "cache_0xFF")]
#[test_case(false, false, false, 0xFF, &test_data_A2_dt6_checks_many; "nocache_0xFF")]
#[test_case(true, false, true, 0xFF, &test_data_A2_dt6_checks_many; "cache_0xFF_swapends")]
#[test_case(false, false, true, 0xFF, &test_data_A2_dt6_checks_many; "nocache_0xFF_swapends")]
#[test_case(true, true, false, 0xFF, &test_data_A2_dt6_checks_many; "cache_0xFF_reverse")]
#[test_case(false, true, false, 0xFF, &test_data_A2_dt6_checks_many; "nocache_0xFF_reverse")]
#[test_case(true, true, true, 0xFF, &test_data_A2_dt6_checks_many; "cache_0xFF_reverse_swapends")]
#[test_case(false, true, true, 0xFF, &test_data_A2_dt6_checks_many; "nocache_0xFF_reverse_swapends")]
#[test_case(true, false, false, 0x1FF, &test_data_A2_dt6_checks_many; "cache_0x1FF")]
#[test_case(false, false, false, 0x1FF, &test_data_A2_dt6_checks_many; "nocache_0x1FF")]
#[test_case(true, false, true, 0x1FF, &test_data_A2_dt6_checks_many; "cache_0x1FF_swapends")]
#[test_case(false, false, true, 0x1FF, &test_data_A2_dt6_checks_many; "nocache_0x1FF_swapends")]
#[test_case(true, true, false, 0x1FF, &test_data_A2_dt6_checks_many; "cache_0x1FF_reverse")]
#[test_case(false, true, false, 0x1FF, &test_data_A2_dt6_checks_many; "nocache_0x1FF_reverse")]
#[test_case(true, true, true, 0x1FF, &test_data_A2_dt6_checks_many; "cache_0x1FF_reverse_swapends")]
#[test_case(false, true, true, 0x1FF, &test_data_A2_dt6_checks_many; "nocache_0x1FF_reverse_swapends")]
fn test_find_sysline_A2_dt6(
    cache: bool,
    rev_checks: bool,
    swap_ends: bool,
    blocksz: BlockSz,
    checks: &TestDataA2Dt6Checks,
) {
    match std::io::stdout().flush() {
        Ok(_) => (),
        Err(err) => panic!("stdout flush failed: {}", err),
    };
    match std::io::stderr().flush() {
        Ok(_) => (),
        Err(err) => panic!("stderr flush failed: {}", err),
    };
    defn!("cache {}, rev_checks {}, swap_ends {}, blocksz 0x{:02X}",
        cache, rev_checks, swap_ends, blocksz);
    let mut checks_: TestDataA2Dt6ChecksVec;
    match (rev_checks, swap_ends) {
        (true, true) => {
            checks_ = copy_TestDataA2Dt6Checks(checks);
            checks_.reverse();
            // swap every other
            for at in (0..(checks_.len() / 2)).step_by(2) {
                let at_end = checks_.len() - at - 1;
                checks_.swap(at, at_end);
            }
        }
        (true, false) => {
            checks_ = copy_TestDataA2Dt6Checks(checks);
            checks_.reverse();
        }
        (false, true) => {
            checks_ = copy_TestDataA2Dt6Checks(checks);
            // swap every other
            for at in (0..(checks_.len() / 2)).step_by(2) {
                let at_end = checks_.len() - at - 1;
                checks_.swap(at, at_end);
            }
        }
        (false, false) => {
            defo!("checks {}", checks.len());
            for check in checks.iter() {
                defo!("check({:?} {:?} {:?})", check.0, check.1, check.2);
            }
            impl_test_findsysline(&test_data_A2_dt6_ntf_path, blocksz, cache, checks);
            return;
        }
    };
    defo!("checks {}", checks_.len(),);
    for (i, check) in checks_.iter().enumerate() {
        defo!("check[{}]({:?} {:?} {:?})", i, check.0, check.1, check.2);
    }
    impl_test_findsysline(&test_data_A2_dt6_ntf_path, blocksz, cache, &checks_);
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
fn test_find_sysline_B_dt0_0() {
    let ntf = create_temp_file(test_data_file_B_dt0);
    let path = ntf_fpath(&ntf);
    let checks = TestSyslineReaderChecks::from(test_data_file_B_dt0_checks);
    impl_test_SyslineReader_find_sysline(&path, 128, 0, &checks);
}

#[test]
fn test_find_sysline_B_dt0_3() {
    let ntf = create_temp_file(test_data_file_B_dt0);
    let path = ntf_fpath(&ntf);
    let checks = TestSyslineReaderChecks::from(test_data_file_B_dt0_checks);
    impl_test_SyslineReader_find_sysline(&path, 128, 3, &checks);
}

// -------------------------------------------------------------------------------------------------

const test_data_file_C_dt6: &str = "\
[DEBUG] 2000-01-01 00:00:00
[DEBUG] 2000-01-01 00:00:01a
[DEBUG] 2000-01-01 00:00:02ab
[DEBUG] 2000-01-01 00:00:03abc
[DEBUG] 2000-01-01 00:00:04abcd
[DEBUG] 2000-01-01 00:00:05abcde";

const test_data_file_C_dt6_checks: [TestSyslineReaderCheck; 6] = [
    ("[DEBUG] 2000-01-01 00:00:00\n", 28),
    ("[DEBUG] 2000-01-01 00:00:01a\n", 57),
    ("[DEBUG] 2000-01-01 00:00:02ab\n", 87),
    ("[DEBUG] 2000-01-01 00:00:03abc\n", 118),
    ("[DEBUG] 2000-01-01 00:00:04abcd\n", 150),
    ("[DEBUG] 2000-01-01 00:00:05abcde", 182),
];

lazy_static! {
    static ref test_SyslineReader_C_ntf: NamedTempFile = create_temp_file(test_data_file_C_dt6);
    static ref test_SyslineReader_C_ntf_path: FPath = ntf_fpath(&test_SyslineReader_C_ntf);
}

#[test]
fn test_find_sysline_C_dt6_0() {
    let checks = TestSyslineReaderChecks::from(test_data_file_C_dt6_checks);
    impl_test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 0, &checks);
}

#[test]
fn test_find_sysline_C_dt6_3() {
    let checks = TestSyslineReaderChecks::from(test_data_file_C_dt6_checks);
    impl_test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 3, &checks);
}

#[test]
fn test_find_sysline_C_dt6_27() {
    let checks = TestSyslineReaderChecks::from(test_data_file_C_dt6_checks);
    impl_test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 27, &checks);
}

#[test]
fn test_find_sysline_C_dt6_28_1__() {
    let checks = TestSyslineReaderChecks::from(&test_data_file_C_dt6_checks[1..]);
    impl_test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 28, &checks);
}

#[test]
fn test_find_sysline_C_dt6_29_1__() {
    let checks = TestSyslineReaderChecks::from(&test_data_file_C_dt6_checks[1..]);
    impl_test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 29, &checks);
}

#[test]
fn test_find_sysline_D_invalid1() {
    let data_invalid1: [u8; 1] = [0xFF];
    let date_checks1: TestSyslineReaderChecks = TestSyslineReaderChecks::from([]);
    let ntf = create_temp_file_bytes(&data_invalid1);
    let path = ntf_fpath(&ntf);
    impl_test_SyslineReader_find_sysline(&path, 128, 0, &date_checks1);
}

// -------------------------------------------------------------------------------------------------

const test_data_file_E_dt6_sysline0: &str = "2001-01-01 00:00:00 _\n2001-02-31 00:00:01 😩\n";
const test_data_file_E_dt6_sysline1: &str = "2001-03-01 00:00:02 😀😁\n";
const test_data_file_E_dt6_sysline2: &str = "2001-04-01 00:00:03 😀😁😂\n";

// notice the second line, first sysline, is an invalid date that will pass regex match
const test_data_file_E_dt6: &str = concatcp!(
    test_data_file_E_dt6_sysline0,
    test_data_file_E_dt6_sysline1,
    test_data_file_E_dt6_sysline2,
    "2001-05-01 00:00:04 😀😁😂😃\n",
    "2001-06-01 00:00:05 😀😁😂😃😄",
);

const test_data_file_E_dt6_sysline0_end: FileOffset = test_data_file_E_dt6_sysline0.len() as FileOffset;
const test_data_file_E_dt6_sysline1_end: FileOffset = test_data_file_E_dt6_sysline0_end + test_data_file_E_dt6_sysline1.len() as FileOffset;
const test_data_file_E_dt6_sysline2_end: FileOffset = test_data_file_E_dt6_sysline1_end + test_data_file_E_dt6_sysline2.len() as FileOffset;

lazy_static! {
    static ref test_SyslineReader_E_ntf: NamedTempFile = create_temp_file(test_data_file_E_dt6);
    static ref test_SyslineReader_E_ntf_path: FPath = ntf_fpath(&test_SyslineReader_E_ntf);
}

#[test]
fn test_find_sysline_E_dt6_0() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(0, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0)]
    );
    impl_test_findsysline(&test_SyslineReader_E_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_E_dt6_1() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(1, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0)]
    );
    impl_test_findsysline(&test_SyslineReader_E_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_E_dt6_22() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(22, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0)]
    );
    impl_test_findsysline(&test_SyslineReader_E_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_E_dt6_42() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(42, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0)]
    );
    impl_test_findsysline(&test_SyslineReader_E_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_E_dt6_43() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (43, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0)
        ]
    );
    impl_test_findsysline(&test_SyslineReader_E_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_E_dt6_44() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(44, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0)]
    );
    impl_test_findsysline(&test_SyslineReader_E_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_E_dt6_75() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(75, FOUND, test_data_file_E_dt6_sysline1_end, test_data_file_E_dt6_sysline1)]
    );
    impl_test_findsysline(&test_SyslineReader_E_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_E_dt6_76() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(76, FOUND, test_data_file_E_dt6_sysline2_end, test_data_file_E_dt6_sysline2)]
    );
    impl_test_findsysline(&test_SyslineReader_E_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_E_dt6_0______78() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (0, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0),
        (1, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0),
        (21, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0),
        (22, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0),
        (23, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0),
        (24, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0),
        (42, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0),
        (43, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0),
        (44, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0),
        (45, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0),
        (46, FOUND, test_data_file_E_dt6_sysline0_end, test_data_file_E_dt6_sysline0),
        (47, FOUND, test_data_file_E_dt6_sysline1_end, test_data_file_E_dt6_sysline1),
        (48, FOUND, test_data_file_E_dt6_sysline1_end, test_data_file_E_dt6_sysline1),
        (49, FOUND, test_data_file_E_dt6_sysline1_end, test_data_file_E_dt6_sysline1),
        (70, FOUND, test_data_file_E_dt6_sysline1_end, test_data_file_E_dt6_sysline1),
        (71, FOUND, test_data_file_E_dt6_sysline1_end, test_data_file_E_dt6_sysline1),
        (72, FOUND, test_data_file_E_dt6_sysline1_end, test_data_file_E_dt6_sysline1),
        (73, FOUND, test_data_file_E_dt6_sysline1_end, test_data_file_E_dt6_sysline1),
        (74, FOUND, test_data_file_E_dt6_sysline1_end, test_data_file_E_dt6_sysline1),
        (75, FOUND, test_data_file_E_dt6_sysline1_end, test_data_file_E_dt6_sysline1),
        (76, FOUND, test_data_file_E_dt6_sysline2_end, test_data_file_E_dt6_sysline2),
        (77, FOUND, test_data_file_E_dt6_sysline2_end, test_data_file_E_dt6_sysline2),
        (78, FOUND, test_data_file_E_dt6_sysline2_end, test_data_file_E_dt6_sysline2),
    ]);
    impl_test_findsysline(&test_SyslineReader_E_ntf_path, 4, false, &checks);
}

// -------------------------------------------------------------------------------------------------

const test_data_file_F_dt6_sysline0: &str = "2001-01-01 00:00:00 _\n";
const test_data_file_F_dt6_sysline1: &str = "2001-02-01 00:00:01 😀\n";
const test_data_file_F_dt6_sysline2: &str = "2001-03-01 00:00:02 😀😁\n2001-04-31 00:00:03 😫😫😫\n";
const test_data_file_F_dt6_sysline3: &str = "2001-05-01 00:00:04 😀😁😂😃\n";

// notice the fourth line, third sysline, is an invalid date that will pass regex match
const test_data_file_F_dt6: &str = concatcp!(
    test_data_file_F_dt6_sysline0,
    test_data_file_F_dt6_sysline1,
    test_data_file_F_dt6_sysline2,
    test_data_file_F_dt6_sysline3,
    "2001-05-01 00:00:04 😀😁😂😃\n",
    "2001-06-01 00:00:05 😀😁😂😃😄"
);

// remember that `.len()` starts at one, and `FileOffset` starts at zero

const test_data_file_F_dt6_sysline0_end: FileOffset = test_data_file_F_dt6_sysline0.len() as FileOffset;
const test_data_file_F_dt6_sysline1_end: FileOffset = test_data_file_F_dt6_sysline0_end + test_data_file_F_dt6_sysline1.len() as FileOffset;
const test_data_file_F_dt6_sysline2_end: FileOffset = test_data_file_F_dt6_sysline1_end + test_data_file_F_dt6_sysline2.len() as FileOffset;
const test_data_file_F_dt6_sysline3_end: FileOffset = test_data_file_F_dt6_sysline2_end + test_data_file_F_dt6_sysline3.len() as FileOffset;

lazy_static! {
    static ref test_SyslineReader_F_ntf: NamedTempFile = create_temp_file(test_data_file_F_dt6);
    static ref test_SyslineReader_F_ntf_path: FPath = ntf_fpath(&test_SyslineReader_F_ntf);
}

#[test]
fn test_find_sysline_F_dt6_45() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(45, FOUND, test_data_file_F_dt6_sysline1_end, test_data_file_F_dt6_sysline1)]
    );
    impl_test_findsysline(&test_SyslineReader_F_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_F_dt6_46() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(46, FOUND, test_data_file_F_dt6_sysline1_end, test_data_file_F_dt6_sysline1)]
    );
    impl_test_findsysline(&test_SyslineReader_F_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_F_dt6_47() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(47, FOUND, test_data_file_F_dt6_sysline2_end, test_data_file_F_dt6_sysline2)]
    );
    impl_test_findsysline(&test_SyslineReader_F_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_F_dt6_sysline2_sysline3_108_109() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (108, FOUND, test_data_file_F_dt6_sysline2_end, test_data_file_F_dt6_sysline2),
        (109, FOUND, test_data_file_F_dt6_sysline3_end, test_data_file_F_dt6_sysline3),
    ]);
    impl_test_findsysline(&test_SyslineReader_F_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_F_dt6_sysline2_sysline3_107_110() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (107, FOUND, test_data_file_F_dt6_sysline2_end, test_data_file_F_dt6_sysline2),
        (110, FOUND, test_data_file_F_dt6_sysline3_end, test_data_file_F_dt6_sysline3),
    ]);
    impl_test_findsysline(&test_SyslineReader_F_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_F_dt6_sysline2_sysline3_108_110() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (108, FOUND, test_data_file_F_dt6_sysline2_end, test_data_file_F_dt6_sysline2),
        (110, FOUND, test_data_file_F_dt6_sysline3_end, test_data_file_F_dt6_sysline3),
    ]);
    impl_test_findsysline(&test_SyslineReader_F_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_F_dt6_sysline3_sysline2_109_108() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (109, FOUND, test_data_file_F_dt6_sysline3_end, test_data_file_F_dt6_sysline3),
        (108, FOUND, test_data_file_F_dt6_sysline2_end, test_data_file_F_dt6_sysline2),
    ]);
    impl_test_findsysline(&test_SyslineReader_F_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_F_dt6_sysline3_sysline2_110_107() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (110, FOUND, test_data_file_F_dt6_sysline3_end, test_data_file_F_dt6_sysline3),
        (107, FOUND, test_data_file_F_dt6_sysline2_end, test_data_file_F_dt6_sysline2),
    ]);
    impl_test_findsysline(&test_SyslineReader_F_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_F_dt6_sysline3_sysline2_109_107() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (109, FOUND, test_data_file_F_dt6_sysline3_end, test_data_file_F_dt6_sysline3),
        (107, FOUND, test_data_file_F_dt6_sysline2_end, test_data_file_F_dt6_sysline2),
    ]);
    impl_test_findsysline(&test_SyslineReader_F_ntf_path, 4, false, &checks);
}

// -------------------------------------------------------------------------------------------------

// the `test_data_file_G_dt4_sysline_` is an invalid date that will pass regex match
// so not a sysline
const test_data_file_G_dt4_line0: &str = "2001-02-31 00:00:00 a\n";
const test_data_file_G_dt4_sysline1: &str = "2001-03-01 00:00:01 b\n";
const test_data_file_G_dt4_sysline2: &str = "2001-04-01 00:00:02 c\n";
const test_data_file_G_dt4_sysline3: &str = "2001-05-01 00:00:03 d";

/// notice the first line is an invalid date that will pass regex match
const test_data_file_G_dt4: &str = concatcp!(
    test_data_file_G_dt4_line0,
    test_data_file_G_dt4_sysline1,
    test_data_file_G_dt4_sysline2,
    test_data_file_G_dt4_sysline3,
);

// remember that `.len()` starts at one, and `FileOffset` starts at zero

const test_data_file_G_dt4_line0_end: FileOffset = test_data_file_G_dt4_line0.len() as FileOffset;
const test_data_file_G_dt4_sysline1_end: FileOffset = test_data_file_G_dt4_line0_end + test_data_file_G_dt4_sysline1.len() as FileOffset;
const test_data_file_G_dt4_sysline2_end: FileOffset = test_data_file_G_dt4_sysline1_end + test_data_file_G_dt4_sysline2.len() as FileOffset;
const test_data_file_G_dt4_sysline3_end: FileOffset = test_data_file_G_dt4_sysline2_end + test_data_file_G_dt4_sysline3.len() as FileOffset;

lazy_static! {
    static ref test_SyslineReader_G_ntf: NamedTempFile = create_temp_file(test_data_file_G_dt4);
    static ref test_SyslineReader_G_ntf_path: FPath = ntf_fpath(&test_SyslineReader_G_ntf);
}

#[test]
fn test_find_sysline_G_dt4_0() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(0, FOUND, test_data_file_G_dt4_sysline1_end, test_data_file_G_dt4_sysline1)]
    );
    impl_test_findsysline(&test_SyslineReader_G_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_G_dt4_42_42() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [
            (42, FOUND, test_data_file_G_dt4_sysline1_end, test_data_file_G_dt4_sysline1),
            (42, FOUND, test_data_file_G_dt4_sysline1_end, test_data_file_G_dt4_sysline1),
        ]
    );
    impl_test_findsysline(&test_SyslineReader_G_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_G_dt4_43() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(43, FOUND, test_data_file_G_dt4_sysline1_end, test_data_file_G_dt4_sysline1)]
    );
    impl_test_findsysline(&test_SyslineReader_G_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_G_dt4_44() {
    let checks = TestSyslineReaderAnyInputChecks::from(
        [(44, FOUND, test_data_file_G_dt4_sysline2_end, test_data_file_G_dt4_sysline2)]
    );
    impl_test_findsysline(&test_SyslineReader_G_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_G_dt4_1_43_44() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (1, FOUND, test_data_file_G_dt4_sysline1_end, test_data_file_G_dt4_sysline1),
        (43, FOUND, test_data_file_G_dt4_sysline1_end, test_data_file_G_dt4_sysline1),
        (44, FOUND, test_data_file_G_dt4_sysline2_end, test_data_file_G_dt4_sysline2),
    ]);
    impl_test_findsysline(&test_SyslineReader_G_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_G_dt4_44_43() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (44, FOUND, test_data_file_G_dt4_sysline2_end, test_data_file_G_dt4_sysline2),
        (43, FOUND, test_data_file_G_dt4_sysline1_end, test_data_file_G_dt4_sysline1),
    ]);
    impl_test_findsysline(&test_SyslineReader_G_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_G_dt4_44_45_42_43() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (44, FOUND, test_data_file_G_dt4_sysline2_end, test_data_file_G_dt4_sysline2),
        (45, FOUND, test_data_file_G_dt4_sysline2_end, test_data_file_G_dt4_sysline2),
        (42, FOUND, test_data_file_G_dt4_sysline1_end, test_data_file_G_dt4_sysline1),
        (43, FOUND, test_data_file_G_dt4_sysline1_end, test_data_file_G_dt4_sysline1),
    ]);
    impl_test_findsysline(&test_SyslineReader_G_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_G_dt4_43_e11_e12_e21() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (43, FOUND, test_data_file_G_dt4_sysline1_end, test_data_file_G_dt4_sysline1),
        (test_data_file_G_dt4_sysline1_end, FOUND, test_data_file_G_dt4_sysline2_end, test_data_file_G_dt4_sysline2),
        (test_data_file_G_dt4_sysline1_end + 2, FOUND, test_data_file_G_dt4_sysline2_end, test_data_file_G_dt4_sysline2),
        (test_data_file_G_dt4_sysline2_end - 1, FOUND, test_data_file_G_dt4_sysline2_end, test_data_file_G_dt4_sysline2),
    ]);
    impl_test_findsysline(&test_SyslineReader_G_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_G_dt4_66() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (66, FOUND, test_data_file_G_dt4_sysline3_end, test_data_file_G_dt4_sysline3),
    ]);
    impl_test_findsysline(&test_SyslineReader_G_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_G_dt4_86_87() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (86, FOUND, test_data_file_G_dt4_sysline3_end, test_data_file_G_dt4_sysline3),
        (87, DONE, 0, ""),
    ]);
    impl_test_findsysline(&test_SyslineReader_G_ntf_path, 4, false, &checks);
}

// -------------------------------------------------------------------------------------------------

const test_data_file_H_dt4_sysline0: &str = "2001-02-01 00:00:01 a\n";
const test_data_file_H_dt4_sysline1: &str = "2001-03-01 00:00:02 b\n";
const test_data_file_H_dt4_sysline2: &str = "2001-04-01 00:00:03 c\n";
const test_data_file_H_dt4_sysline3: &str = "2001-05-01 00:00:04 d";

const test_data_file_H_dt4: &str = concatcp!(
    test_data_file_H_dt4_sysline0,
    test_data_file_H_dt4_sysline1,
    test_data_file_H_dt4_sysline2,
    test_data_file_H_dt4_sysline3,
);

const test_data_file_H_dt4_sysline0_end: FileOffset = test_data_file_H_dt4_sysline0.len() as FileOffset;
const test_data_file_H_dt4_sysline1_end: FileOffset = test_data_file_H_dt4_sysline0_end + test_data_file_H_dt4_sysline1.len() as FileOffset;
const test_data_file_H_dt4_sysline2_end: FileOffset = test_data_file_H_dt4_sysline1_end + test_data_file_H_dt4_sysline2.len() as FileOffset;
const test_data_file_H_dt4_sysline3_end: FileOffset = test_data_file_H_dt4_sysline2_end + test_data_file_H_dt4_sysline3.len() as FileOffset;

lazy_static! {
    static ref test_SyslineReader_H_ntf: NamedTempFile = create_temp_file(test_data_file_H_dt4);
    static ref test_SyslineReader_H_ntf_path: FPath = ntf_fpath(&test_SyslineReader_H_ntf);
}

#[test]
fn test_find_sysline_H_dt4_sysline0() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (0, FOUND, test_data_file_H_dt4_sysline0_end, test_data_file_H_dt4_sysline0),
        (1, FOUND, test_data_file_H_dt4_sysline0_end, test_data_file_H_dt4_sysline0),
        (2, FOUND, test_data_file_H_dt4_sysline0_end, test_data_file_H_dt4_sysline0),
        (3, FOUND, test_data_file_H_dt4_sysline0_end, test_data_file_H_dt4_sysline0),
        (0, FOUND, test_data_file_H_dt4_sysline0_end, test_data_file_H_dt4_sysline0),
        (10, FOUND, test_data_file_H_dt4_sysline0_end, test_data_file_H_dt4_sysline0),
        (20, FOUND, test_data_file_H_dt4_sysline0_end, test_data_file_H_dt4_sysline0),
        (21, FOUND, test_data_file_H_dt4_sysline0_end, test_data_file_H_dt4_sysline0),
    ]);
    impl_test_findsysline(&test_SyslineReader_H_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_H_dt4_sysline1() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (22, FOUND, test_data_file_H_dt4_sysline1_end, test_data_file_H_dt4_sysline1),
        (22, FOUND, test_data_file_H_dt4_sysline1_end, test_data_file_H_dt4_sysline1),
        (22, FOUND, test_data_file_H_dt4_sysline1_end, test_data_file_H_dt4_sysline1),
        (43, FOUND, test_data_file_H_dt4_sysline1_end, test_data_file_H_dt4_sysline1),
    ]);
    impl_test_findsysline(&test_SyslineReader_H_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_H_dt4_sysline3_Found_Done() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (86, FOUND, test_data_file_H_dt4_sysline3_end, test_data_file_H_dt4_sysline3),
        (85, FOUND, test_data_file_H_dt4_sysline3_end, test_data_file_H_dt4_sysline3),
        (87, DONE, 0, ""),
        (88, DONE, 0, ""),
        (87, DONE, 0, ""),
        (66, FOUND, test_data_file_H_dt4_sysline3_end, test_data_file_H_dt4_sysline3),
        (88, DONE, 0, ""),
        (86, FOUND, test_data_file_H_dt4_sysline3_end, test_data_file_H_dt4_sysline3),
    ]);
    impl_test_findsysline(&test_SyslineReader_H_ntf_path, 4, false, &checks);
}

#[test]
fn test_find_sysline_H_dt4_Done() {
    let checks = TestSyslineReaderAnyInputChecks::from([
        (87, DONE, 0, ""),
        (88, DONE, 0, ""),
        (87, DONE, 0, ""),
    ]);
    impl_test_findsysline(&test_SyslineReader_H_ntf_path, 4, false, &checks);
}

// -----------------------------------------------------------------------------

#[test_case(true; "cache")]
#[test_case(false; "nocache")]
fn test_remove_sysline(cache: bool) {
    let mut slr = new_SyslineReader(&NTF26_PATH, 128, *FO_P8);
    if !cache {
        slr.LRU_cache_disable();
    }
    let mut fo: FileOffset = 0;
    // read all syslines in file in order
    loop {
        let fo2 = fo;
        let result = slr.find_sysline(fo);
        match result {
            ResultS3SyslineFind::Found((fo_, _syslinep)) => {
                fo = fo_;
            }
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                panic!("ERROR: {}", err);
            }
        }
        // remove the previous sysline
        assert!(slr.remove_sysline(fo2));
    }

    let cache_actual = slr.LRU_cache_disable();
    assert_eq!(cache, cache_actual, "Expected cache_enabled to be {}, it was {}", cache, cache_actual);
}

#[test_case(true; "cache")]
#[test_case(false; "nocache")]
fn test_clear_syslines(cache: bool) {
    let mut slr = new_SyslineReader(&NTF26_PATH, 128, *FO_P8);
    if !cache {
        slr.LRU_cache_disable();
    }
    let mut fo: FileOffset = 0;
    // read all syslines in file in order
    loop {
        let result = slr.find_sysline(fo);
        match result {
            ResultS3SyslineFind::Found((fo_, _syslinep)) => {
                fo = fo_;
            }
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                panic!("ERROR: {}", err);
            }
        }
        slr.clear_syslines();
    }
    // try three more times just for fun
    slr.clear_syslines();
    slr.clear_syslines();
    slr.clear_syslines();

    let cache_actual = slr.LRU_cache_disable();
    assert_eq!(cache, cache_actual, "Expected cache_enabled to be {}, it was {}", cache, cache_actual);
}

// -----------------------------------------------------------------------------

// test `syslinereader.datetime_parse_data`

#[test]
fn test_datetime_parse_data() {
    let mut slr = new_SyslineReader(&NTF26_PATH, 128, *FO_P8);
    let mut fo: FileOffset = 0;
    loop {
        let result = slr.find_sysline(fo);
        match result {
            ResultS3SyslineFind::Found((fo_, _syslinep)) => {
                fo = fo_;
            }
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                panic!("ERROR: {}", err);
            }
        }
        // asserting on the returned `DateTimeParseInstr` is very tedious
        // so just make sure it at least returns
        let _dtpd: &DateTimeParseInstr = slr.datetime_parse_data();
    }
}

// -------------------------------------------------------------------------------------------------

/// basic test of `SyslineReader::find_sysline`
/// read all file offsets but randomly
///
/// TODO: [2021/09] this test was hastily designed for human review. Redesign it for automatic review.
#[allow(non_snake_case)]
fn impl_test_find_sysline_rand(
    path: &FPath,
    blocksz: BlockSz,
) {
    defn!("({:?}, {})", path, blocksz);
    let tzo8: FixedOffset = *FO_M8;
    let mut slr = new_SyslineReader(path, blocksz, tzo8);
    defo!("SyslineReader: {:?}", slr);
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(slr.filesz() as usize);
    fill(&mut offsets_rand);
    defo!("offsets_rand: {:?}", offsets_rand);
    randomize(&mut offsets_rand);
    defo!("offsets_rand: {:?}", offsets_rand);

    for fo1 in offsets_rand {
        let result = slr.find_sysline(fo1);
        #[allow(clippy::single_match)]
        match result {
            ResultS3SyslineFind::Err(err) => {
                panic!("slr.find_sysline({}) returned Err({})", fo1, err);
            }
            _ => {}
        }
    }
    defx!();
}

#[test]
fn test_find_sysline_rand__zero__2() {
    impl_test_find_sysline_rand(&NTF_LOG_EMPTY_FPATH, 2);
}

#[test]
fn test_find_sysline_rand__test0_nlx1__2() {
    impl_test_find_sysline_rand(&NTF_NL_1_PATH, 2);
}

#[test]
fn test_find_sysline_rand__test0_nlx1__4() {
    impl_test_find_sysline_rand(&NTF_NL_1_PATH, 4);
}

#[test]
fn test_find_sysline_rand__test0_nlx1__8() {
    impl_test_find_sysline_rand(&NTF_NL_1_PATH, 8);
}

#[test]
fn test_find_sysline_rand__test0_nlx1_Win__2() {
    impl_test_find_sysline_rand(&NTF_WNL_1_PATH, 2);
}

#[test]
fn test_find_sysline_rand__test0_nlx1_Win__4() {
    impl_test_find_sysline_rand(&NTF_WNL_1_PATH, 4);
}

#[test]
fn test_find_sysline_rand__test0_nlx1_Win__8() {
    impl_test_find_sysline_rand(&NTF_WNL_1_PATH, 8);
}

#[test]
fn test_find_sysline_rand__test0_nlx2__4() {
    impl_test_find_sysline_rand(&NTF_NL_2_PATH, 4);
}

#[test]
fn test_find_sysline_rand__basic_dt1__4() {
    impl_test_find_sysline_rand(&FPath::from("./logs/other/tests/basic-dt1.log"), 4);
}

#[test]
fn test_find_sysline_rand__dtf5_6c__4() {
    impl_test_find_sysline_rand(&FPath::from("./logs/other/tests/dtf5-6c.log"), 4);
}

#[test]
fn test_find_sysline_rand__dtf5_6c__8() {
    impl_test_find_sysline_rand(&FPath::from("./logs/other/tests/dtf5-6c.log"), 8);
}

// -----------------------------------------------------------------------------

type CheckFindSyslineInBlock = Vec<(FileOffset, ResultS3SyslineFind_Test, String)>;

/// test `syslinereader.find_sysline_in_block`
#[allow(non_snake_case)]
fn impl_test_find_sysline_in_block(
    path: &FPath,
    cache: bool,
    checks: CheckFindSyslineInBlock,
    blocksz: BlockSz,
) {
    defn!("({:?}, {}, {})", path, cache, blocksz);
    let tzo = FixedOffset::west_opt(3600 * 2).unwrap();
    let mut slr = new_SyslineReader(path, blocksz, tzo);
    if !cache {
        slr.LRU_cache_disable();
    }
    defo!("SyslineReader {:?}", slr);

    for (fo_input, result_expect, value_expect) in checks.iter() {
        let result = slr.find_sysline_in_block(*fo_input);
        assert_results4(fo_input, result_expect, &result.0);
        match result.0 {
            ResultS3SyslineFind::Found((_fo, slp)) => {
                let value_actual: String = (*slp).to_String();
                assert_eq!(
                    value_expect, &value_actual,
                    "find_sysline_in_block({})\nExpected {:?}\nActual {:?}",
                    fo_input, value_expect, value_actual,
                );
            }
            ResultS3SyslineFind::Done => {
                // self-check
                assert_eq!(value_expect, &String::from(""), "bad test check value {:?}", value_expect,);
            }
            ResultS3SyslineFind::Err(err) => {
                panic!("ERROR: find_sysline_in_block({}) returned Error {:?}", fo_input, err);
            }
        }
    }

    defx!();
}

#[test_case(true; "cache")]
#[test_case(false; "nocache")]
fn test_find_sysline_in_block__empty0(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![];
    impl_test_find_sysline_in_block(&NTF_LOG_EMPTY_FPATH, cache, checks, 2);
}

#[test_case(true; "cache")]
#[test_case(false; "nocache")]
fn test_find_sysline_in_block__empty1(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![(0, DONE, String::from(""))];
    impl_test_find_sysline_in_block(&NTF_NL_1_PATH, cache, checks, 2);
}

#[test_case(true; "cache")]
#[test_case(false; "nocache")]
fn test_find_sysline_in_block__empty2(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![
        (0, DONE, String::from("")),
        (1, DONE, String::from("")),
    ];

    impl_test_find_sysline_in_block(&NTF_NL_2_PATH, cache, checks, 4);
}

#[test_case(true; "cache")]
#[test_case(false; "nocache")]
fn test_find_sysline_in_block__empty4(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![
        (0, DONE, String::from("")),
        (1, DONE, String::from("")),
        (2, DONE, String::from("")),
        (3, DONE, String::from("")),
    ];

    impl_test_find_sysline_in_block(&NTF_NL_4_PATH, cache, checks, 4);
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
#[test_case(false; "nocache")]
fn test_find_sysline_in_block__A3__1_4(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![
        (0, DONE, String::from("")),
        (1, DONE, String::from("")),
        (2, DONE, String::from("")),
        (3, DONE, String::from("")),
        (0xFFFF, DONE, String::from("")),
    ];

    impl_test_find_sysline_in_block(&NTF_A3_1_PATH, cache, checks, 4);
}

#[test_case(true; "cache")]
#[test_case(false; "nocache")]
fn test_find_sysline_in_block__A3__12_4(cache: bool) {
    let checks: CheckFindSyslineInBlock = vec![
        (0, DONE, String::from("")),
        (1, DONE, String::from("")),
        (2, DONE, String::from("")),
        (3, DONE, String::from("")),
        (0xFFFF, DONE, String::from("")),
    ];

    impl_test_find_sysline_in_block(&NTF_A3_12_PATH, cache, checks, 4);
}

#[test_case(true; "cache")]
#[test_case(false; "nocache")]
fn test_find_sysline_in_block__A3__1_LINE1LEN(cache: bool) {
    let fo: FileOffset = *NTF_A3_DATA_LINE1_BYTES_LEN as FileOffset;
    defo!("fo {:?}", fo);
    // use blocks of the same size as the first line in the test file
    let bsz: BlockSz = *NTF_A3_DATA_LINE1_BYTES_LEN_EVEN as BlockSz;
    defo!("bsz {:?}", bsz);
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

    impl_test_find_sysline_in_block(&NTF_A3_1_PATH, cache, checks, bsz);
}

#[test_case(true; "cache")]
#[test_case(false; "nocache")]
fn test_find_sysline_in_block__A3__12_LINE1LEN(cache: bool) {
    let fo: FileOffset = *NTF_A3_DATA_LINE1_BYTES_LEN as FileOffset;
    let fo12: FileOffset = (*NTF_A3_DATA_LINE1_BYTES_LEN + *NTF_A3_DATA_LINE2_BYTES_LEN) as FileOffset;
    defo!("fo {:?}", fo);
    // use blocks of the same size as the first line in the test file
    let bsz: BlockSz = *NTF_A3_DATA_LINE1_BYTES_LEN_EVEN as BlockSz;
    defo!("bsz {:?}", bsz);
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

    impl_test_find_sysline_in_block(&NTF_A3_12_PATH, cache, checks, bsz);
}

#[test_case(true; "cache")]
#[test_case(false; "nocache")]
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

    impl_test_find_sysline_in_block(&NTF_A3_1_PATH, cache, checks, 0xFFFF);
}

#[test_case(true; "cache")]
#[test_case(false; "nocache")]
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

    impl_test_find_sysline_in_block(&NTF_A3_12_PATH, cache, checks, 0xFFFF);
}

// -----------------------------------------------------------------------------

const HOUR: i32 = 3600;

#[test]
fn test_datetime_parse_from_str__good_without_tz1() {
    // good without timezone
    let dts1 = "2000-01-01 00:01:01";
    let p1 = "%Y-%m-%d %H:%M:%S";
    let dt1 = datetime_parse_from_str(dts1, p1, false, &FO_P8).unwrap();
    let answer1 = FO_P8
        .with_ymd_and_hms(2000, 1, 1, 0, 1, 1)
        .unwrap();
    assert_eq!(dt1, answer1);
}

#[test]
fn test_datetime_parse_from_str__2_good_without_tz() {
    // good without timezone
    let dts1 = "2000-01-01 00:02:01";
    let p1 = "%Y-%m-%d %H:%M:%S";
    let dt1 = datetime_parse_from_str(dts1, p1, false, &FO_P5).unwrap();
    let answer1 = FO_P5
        .with_ymd_and_hms(2000, 1, 1, 0, 2, 1)
        .unwrap();
    assert_eq!(dt1, answer1);
}

#[test]
fn test_datetime_parse_from_str__3_good_with_tz() {
    // good with timezone
    let dts2 = "2000-01-01 00:00:02 -0100";
    let p2 = "%Y-%m-%d %H:%M:%S %z";
    let dt2 = datetime_parse_from_str(dts2, p2, true, &FO_P8).unwrap();
    let answer2 = FixedOffset::west_opt(HOUR)
        .unwrap()
        .with_ymd_and_hms(2000, 1, 1, 0, 0, 2)
        .unwrap();
    assert_eq!(dt2, answer2);
}

#[test]
fn test_datetime_parse_from_str__4_bad_with_tz() {
    // bad with timezone
    let dts3 = "2000-01-01 00:00:03 BADD";
    let p3 = "%Y-%m-%d %H:%M:%S %z";
    let dt3 = datetime_parse_from_str(dts3, p3, true, &FO_P8);
    assert_eq!(dt3, None);
}

#[test]
fn test_datetime_parse_from_str__5_bad_without_tz() {
    // bad without timezone
    let dts4 = "2000-01-01 00:00:XX";
    let p4 = "%Y-%m-%d %H:%M:%S";
    let dt4 = datetime_parse_from_str(dts4, p4, false, &FO_P8);
    assert_eq!(dt4, None);
}

// -----------------------------------------------------------------------------

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
    defn!();
    let vec0 = Vec::<DateTimeL>::with_capacity(0);
    let val = datetime_soonest2(&vec0);
    assert!(val.is_none());
    let tzo: FixedOffset = *FO_M8;

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

    let dt1_b =
        datetime_parse_from_str("2001-01-01T12:00:00-0100", "%Y-%m-%dT%H:%M:%S%z", true, &tzo).unwrap();
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

    defx!();
}

/// test `SyslineReader::summary` before doing any processing
#[test_case(&NTF_LOG_EMPTY_FPATH, 64, *FO_M5)]
//#[test_case(&NTF_TAR_1BYTE_FILEA_FPATH, 64, *FO_P8)]
#[test_case(&NTF_GZ_EMPTY_FPATH, 64, *FO_P8)]
#[test_case(&NTF_GZ_1BYTE_FPATH, 64, *FO_P8)]
#[test_case(&NTF_XZ_1BYTE_FPATH, 64, *FO_P8)]
fn test_SyslineReader_summary_empty(
    path: &FPath,
    blocksz: BlockSz,
    fixedoffset: FixedOffset,
) {
    let syslinereader = new_SyslineReader(
        path,
        blocksz,
        fixedoffset,
    );
    _ = syslinereader.summary();
}

/// index of `DTPD` that has `has_year4() && has_d2()` and qualifies for
/// EZCHECK12D2
const INDEX_12D2: usize = 0;

/// index of `DTPD` that has `!has_year4()` and only qualifies for EZCHECKD2
const INDEX_D2: usize = 6;

/// value that does not match EZCHECK12D2 (no digits)
const VALUE_NO_12D2: &[u8] = b"abcdefghijklmnopqrstuvwxyz".as_slice();
const VALUE_NO_12D2_LEN: usize = VALUE_NO_12D2.len();
const VALUE_NO_12D2_LEN1: usize = VALUE_NO_12D2_LEN - 1;

/// value that only matches EZCHECK12 (`1`)
const VALUE_12: &[u8] = b"abcdefghijkl_1_pqrstuvwxyz".as_slice();
const VALUE_12_LEN: usize = VALUE_12.len();
const VALUE_12_LEN1: usize = VALUE_12_LEN - 1;

/// value that only matches EZCHECKD2 (`90`)
const VALUE_D2: &[u8] = b"abcdefghijklmno_90_tuvwxyz".as_slice();

// test cases empty
#[test_case(
    INDEX_12D2,
    b"".as_slice(),
    0, 0, 0, // *_min
    0, 0, 0, // ezcheck12
    0, 0, 0, // ezcheckd2
    1, 0, 0, // ezcheck12d2
    true;
    "empty A"
)]
#[test_case(
    INDEX_D2,
    b"".as_slice(),
    0, 0, 0, // *_min
    0, 0, 0, // ezcheck12
    1, 0, 0, // ezcheckd2
    0, 0, 0, // ezcheck12d2
    true;
    "empty B"
)]
// test cases VALUE_NO_12D2
#[test_case(
    INDEX_12D2,
    VALUE_NO_12D2,
    0, 0, VALUE_NO_12D2_LEN1, // *_min
    0, 0, 0, // ezcheck12
    0, 0, 0, // ezcheckd2
    1, 0, VALUE_NO_12D2_LEN1, // ezcheck12d2
    true;
    "VALUE_NO_12D2 C"
)]
#[test_case(
    INDEX_D2,
    VALUE_NO_12D2,
    0, VALUE_NO_12D2_LEN1, 0, // *_min
    0, 0, 0, // ezcheck12
    1, 0, VALUE_NO_12D2_LEN1, // ezcheckd2
    0, 0, 0, // ezcheck12d2
    true;
    "VALUE_NO_12D2 D"
)]
// test cases EZCHECK12
#[test_case(
    INDEX_12D2,
    VALUE_12,
    0, 0, 0, // *_min
    0, 0, 0, // ezcheck12
    0, 0, 0, // ezcheckd2
    0, 1, 0, // ezcheck12d2
    false;
    "EZCHECK12 E"
)]
#[test_case(
    INDEX_D2,
    VALUE_12,
    0, VALUE_12_LEN1, 0,
    0, 0, 0, // ezcheck12
    1, 0, VALUE_12_LEN1, // ezcheckd2
    0, 0, 0, // ezcheck12d2
    true;
    "EZCHECK12 F"
)]
// test cases VALUE_D2
#[test_case(
    INDEX_12D2,
    VALUE_D2,
    0, 0, 0, // *_min
    0, 0, 0, // ezcheck12
    0, 0, 0, // ezcheckd2
    0, 1, 0, // ezcheck12d2
    false;
    "EZCHECKD2 G"
)]
#[test_case(
    INDEX_D2,
    VALUE_D2,
    0, 0, 0, // *_min
    0, 0, 0, // ezcheck12
    0, 1, 0, // ezcheckd2
    0, 0, 0, // ezcheck12d2
    false;
    "EZCHECKD2 H"
)]
fn test_ezcheck_slice(
    index: usize,
    slice_: &[u8],
    expect_ezcheck12_min: LineIndex,
    expect_ezcheckd2_min: LineIndex,
    expect_ezcheck12d2_min: LineIndex,
    expect_ezcheck12_hit: Count,
    expect_ezcheck12_miss: Count,
    expect_ezcheck12_hit_max: LineIndex,
    expect_ezcheckd2_hit: Count,
    expect_ezcheckd2_miss: Count,
    expect_ezcheckd2_hit_max: LineIndex,
    expect_ezcheck12d2_hit: Count,
    expect_ezcheck12d2_miss: Count,
    expect_ezcheck12d2_hit_max: LineIndex,
    expect_result: bool,
) {
    assert_le!(index, DATETIME_PARSE_DATAS_LEN, "bad index {}", index);
    eprintln!("test_ezcheck_slice: index: {:?}", index);
    let dtpd = &DATETIME_PARSE_DATAS[index];
    let mut ezcheck12_min: LineIndex = 0;
    let mut ezcheckd2_min: LineIndex = 0;
    let mut ezcheck12d2_min: LineIndex = 0;
    let mut ezcheck12_hit: Count = 0;
    let mut ezcheck12_miss: Count = 0;
    let mut ezcheck12_hit_max: LineIndex = 0;
    let mut ezcheckd2_hit: Count = 0;
    let mut ezcheckd2_miss: Count = 0;
    let mut ezcheckd2_hit_max: LineIndex = 0;
    let mut ezcheck12d2_hit: Count = 0;
    let mut ezcheck12d2_miss: Count = 0;
    let mut ezcheck12d2_hit_max: LineIndex = 0;
    eprintln!("test_ezcheck_slice: slice: {:?}", slice_.as_bstr());
    eprintln!("test_ezcheck_slice: dtpd.has_year4: {:?}, dtpd.has_d2: {:?}",
        dtpd.has_year4(), dtpd.has_d2());
    let result = SyslineReader::ezcheck_slice(
        dtpd,
        slice_,
        1,
        &mut ezcheck12_min,
        &mut ezcheckd2_min,
        &mut ezcheck12d2_min,
        &mut ezcheck12_hit,
        &mut ezcheck12_miss,
        &mut ezcheck12_hit_max,
        &mut ezcheckd2_hit,
        &mut ezcheckd2_miss,
        &mut ezcheckd2_hit_max,
        &mut ezcheck12d2_hit,
        &mut ezcheck12d2_miss,
        &mut ezcheck12d2_hit_max,
    );
    assert_eq!(result, expect_result, "ezcheck_slice result");
    assert_eq!(
        (ezcheck12_min, ezcheckd2_min, ezcheck12d2_min),
        (expect_ezcheck12_min, expect_ezcheckd2_min, expect_ezcheck12d2_min),
        "\n(ezcheck12_min, ezcheckd2_min, ezcheck12d2_min)\n(expect_ezcheck12_min, expect_ezcheckd2_min, expect_ezcheck12d2_min)\n"
    );
    assert_eq!(
        (ezcheck12_hit, ezcheck12_miss, ezcheck12_hit_max),
        (expect_ezcheck12_hit, expect_ezcheck12_miss, expect_ezcheck12_hit_max),
        "\n(ezcheck12_hit, ezcheck12_miss, ezcheck12_hit_max)\n(expect_ezcheck12_hit, expect_ezcheck12_miss, expect_ezcheck12_hit_max)\n"
    );
    assert_eq!(
        (ezcheckd2_hit, ezcheckd2_miss, ezcheckd2_hit_max),
        (expect_ezcheckd2_hit, expect_ezcheckd2_miss, expect_ezcheckd2_hit_max),
        "\n(ezcheckd2_hit, ezcheckd2_miss, ezcheckd2_hit_max)\n(expect_ezcheckd2_hit, expect_ezcheckd2_miss, expect_ezcheckd2_hit_max)\n"
    );
    assert_eq!(
        (ezcheck12d2_hit, ezcheck12d2_miss, ezcheck12d2_hit_max),
        (expect_ezcheck12d2_hit, expect_ezcheck12d2_miss, expect_ezcheck12d2_hit_max),
        "\n(ezcheck12d2_hit, ezcheck12d2_miss, ezcheck12d2_hit_max)\n(expect_ezcheck12d2_hit, expect_ezcheck12d2_miss, expect_ezcheck12d2_hit_max)\n"
    );
}

#[test_case(
    &*NTF_GZ_8BYTE_FPATH,
    0x60,
    *FO_0,
    true,
    None,
	None,
    0,
    0,
    0,
    0,
    0,
    1,
    0,
    1,
    0,
    None,
    None,
    0,
    1,
    1,
    0,
    2,
    0,
    0,
    308,
    0,
    0,
    0,
    0,
    0,
    16,
    0,
    7,
    292,
    0,
    7;
    "NTF_GZ_8BYTE_FPATH"
)]
#[test_case(
    &*NTF_LZ4_8BYTE_FPATH,
    0x60,
    *FO_0,
    true,
    None,
	None,
    0,
    0,
    0,
    0,
    0,
    1,
    0,
    1,
    0,
    None,
    None,
    0,
    1,
    1,
    0,
    2,
    0,
    0,
    308,
    0,
    0,
    0,
    0,
    0,
    16,
    0,
    7,
    292,
    0,
    7;
    "NTF_LZ4_8BYTE_FPATH"
)]
#[test_case(
    &*NTF_XZ_8BYTE_FPATH,
    0x60,
    *FO_0,
    true,
    None,
	None,
    0,
    0,
    0,
    0,
    0,
    1,
    0,
    1,
    0,
    None,
    None,
    0,
    1,
    1,
    0,
    2,
    0,
    0,
    308,
    0,
    0,
    0,
    0,
    0,
    16,
    0,
    7,
    292,
    0,
    7;
    "NTF_XZ_8BYTE_FPATH"
)]
#[test_case(
    &*NTF_SYSLINE_1_PATH,
    0x20,
    *FO_0,
    true,
    None,
	None,
    0,
    0,
    1,
    1,
    0,
    2,
    0,
    2,
    1,
    Some(*NTF_SYSLINE_1_SYSLINE1_DT),
    Some(*NTF_SYSLINE_1_SYSLINE1_DT),
    0,
    2,
    2,
    0,
    1,
    0,
    76,
    4,
    72,
    0,
    0,
    0,
    0,
    0,
    4,
    0,
    0,
    72,
    0;
    "NTF_SYSLINE_1_PATH (cache)"
)]
#[test_case(
    &*NTF_SYSLINE_1_PATH,
    0x20,
    *FO_0,
    false,
    None,
	None,
    0,
    0,
    1,
    1,
    0,
    2,
    0,
    2,
    1,
    Some(*NTF_SYSLINE_1_SYSLINE1_DT),
    Some(*NTF_SYSLINE_1_SYSLINE1_DT),
    0,
    0,
    0,
    0,
    0,
    0,
    76,
    4,
    72,
    0,
    0,
    0,
    0,
    0,
    4,
    0,
    0,
    72,
    0;
    "NTF_SYSLINE_1_PATH (no cache)"
)]
#[test_case(
    &*NTF_SYSLINE_2_PATH,
    0x8,
    *FO_0,
    true,
    None,
	None,
    0,
    0,
    2,
    2,
    0,
    3,
    0,
    3,
    2,
    Some(*NTF_SYSLINE_2_SYSLINE1_DT),
    Some(*NTF_SYSLINE_2_SYSLINE2_DT),
    0,
    3,
    3,
    1,
    2,
    0,
    77,
    0,
    0,
    77,
    0,
    0,
    0,
    0,
    4,
    0,
    0,
    73,
    0;
    "NTF_SYSLINE_2_PATH"
)]
// TODO: [2024/03/10] copy design of similar function
//       `fixedstructreader_tests.rs:test_FixedStructReader_summary`
fn test_syslinereadersummary(
    path: &FPath,
    blocksz: BlockSz,
    tzo: FixedOffset,
    cache: bool,
    dt_filter_after: DateTimeLOpt,
    dt_filter_before: DateTimeLOpt,
    syslinereader_drop_sysline_ok: Count,
    syslinereader_drop_sysline_errors: Count,
    syslinereader_syslines: Count,
    syslinereader_syslines_stored_highest: usize,
    syslinereader_syslines_hit: Count,
    syslinereader_syslines_miss: Count,
    syslinereader_syslines_by_range_hit: Count,
    syslinereader_syslines_by_range_miss: Count,
    syslinereader_syslines_by_range_put: Count,
    syslinereader_datetime_first: DateTimeLOpt,
    syslinereader_datetime_last: DateTimeLOpt,
    syslinereader_find_sysline_lru_cache_hit: Count,
    syslinereader_find_sysline_lru_cache_miss: Count,
    syslinereader_find_sysline_lru_cache_put: Count,
    syslinereader_parse_datetime_in_line_lru_cache_hit: Count,
    syslinereader_parse_datetime_in_line_lru_cache_miss: Count,
    syslinereader_parse_datetime_in_line_lru_cache_put: Count,
    syslinereader_regex_captures_attempted: Count,
    syslinereader_get_boxptrs_singleptr: Count,
    syslinereader_get_boxptrs_doubleptr: Count,
    syslinereader_get_boxptrs_multiptr: Count,
    syslinereader_ezcheck12_hit: Count,
    syslinereader_ezcheck12_miss: Count,
    syslinereader_ezcheck12_hit_max: LineIndex,
    syslinereader_ezcheckd2_hit: Count,
    syslinereader_ezcheckd2_miss: Count,
    syslinereader_ezcheckd2_hit_max: LineIndex,
    syslinereader_ezcheck12d2_hit: Count,
    syslinereader_ezcheck12d2_miss: Count,
    syslinereader_ezcheck12d2_hit_max: LineIndex,
) {
    eprintln!(
        "test_syslinereadersummary(path={:?}, blocksz={}, tzo={:?}, cache={}, dt_filter_after={:?}, dt_filter_before={:?})",
        path, blocksz, tzo, cache, dt_filter_after, dt_filter_before
    );
    let mut slr = new_SyslineReader(path, blocksz, tzo);
    if !cache {
        slr.LRU_cache_disable();
    }
    let mut fo: FileOffset = 0;
    loop {
        let result = slr.find_sysline_between_datetime_filters(fo, &dt_filter_after, &dt_filter_before);
        match result {
            ResultS3SyslineFind::Found((fo_, _syslinep)) => {
                fo = fo_;
            }
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                panic!("During test unexpected result Error {}", err);
            }
        }
    }
    let summary: SummarySyslineReader = slr.summary();
    eprintln!("\nsummary: {:?}", summary);
    eprintln!("
    syslinereader_drop_sysline_ok: {} {},
    syslinereader_drop_sysline_errors: {} {},
    syslinereader_syslines: {} {},
    syslinereader_syslines_stored_highest: {} {},
    syslinereader_syslines_hit: {} {},
    syslinereader_syslines_miss: {} {},
    syslinereader_syslines_by_range_hit: {} {},
    syslinereader_syslines_by_range_miss: {} {},
    syslinereader_syslines_by_range_put: {} {},
    syslinereader_datetime_first: {:?} {:?},
    syslinereader_datetime_last: {:?} {:?},
    syslinereader_find_sysline_lru_cache_hit: {} {},
    syslinereader_find_sysline_lru_cache_miss: {} {},
    syslinereader_find_sysline_lru_cache_put: {} {},
    syslinereader_parse_datetime_in_line_lru_cache_hit: {} {},
    syslinereader_parse_datetime_in_line_lru_cache_miss: {} {},
    syslinereader_parse_datetime_in_line_lru_cache_put: {} {},
    syslinereader_regex_captures_attempted: {} {},
    syslinereader_get_boxptrs_singleptr: {} {},
    syslinereader_get_boxptrs_doubleptr: {} {},
    syslinereader_get_boxptrs_multiptr: {} {},
    syslinereader_ezcheck12_hit: {} {},
    syslinereader_ezcheck12_miss: {} {},
    syslinereader_ezcheck12_hit_max: {} {},
    syslinereader_ezcheckd2_hit: {} {},
    syslinereader_ezcheckd2_miss: {} {},
    syslinereader_ezcheckd2_hit_max: {} {},
    syslinereader_ezcheck12d2_hit: {} {},
    syslinereader_ezcheck12d2_miss: {} {},
    syslinereader_ezcheck12d2_hit_max: {} {},
",
        syslinereader_drop_sysline_ok, summary.syslinereader_drop_sysline_ok,
        syslinereader_drop_sysline_errors, summary.syslinereader_drop_sysline_errors,
        syslinereader_syslines, summary.syslinereader_syslines,
        syslinereader_syslines_stored_highest, summary.syslinereader_syslines_stored_highest,
        syslinereader_syslines_hit, summary.syslinereader_syslines_hit,
        syslinereader_syslines_miss, summary.syslinereader_syslines_miss,
        syslinereader_syslines_by_range_hit, summary.syslinereader_syslines_by_range_hit,
        syslinereader_syslines_by_range_miss, summary.syslinereader_syslines_by_range_miss,
        syslinereader_syslines_by_range_put, summary.syslinereader_syslines_by_range_put,
        syslinereader_datetime_first, summary.syslinereader_datetime_first,
        syslinereader_datetime_last, summary.syslinereader_datetime_last,
        syslinereader_find_sysline_lru_cache_hit, summary.syslinereader_find_sysline_lru_cache_hit,
        syslinereader_find_sysline_lru_cache_miss, summary.syslinereader_find_sysline_lru_cache_miss,
        syslinereader_find_sysline_lru_cache_put, summary.syslinereader_find_sysline_lru_cache_put,
        syslinereader_parse_datetime_in_line_lru_cache_hit, summary.syslinereader_parse_datetime_in_line_lru_cache_hit,
        syslinereader_parse_datetime_in_line_lru_cache_miss, summary.syslinereader_parse_datetime_in_line_lru_cache_miss,
        syslinereader_parse_datetime_in_line_lru_cache_put, summary.syslinereader_parse_datetime_in_line_lru_cache_put,
        syslinereader_regex_captures_attempted, summary.syslinereader_regex_captures_attempted,
        syslinereader_get_boxptrs_singleptr, summary.syslinereader_get_boxptrs_singleptr,
        syslinereader_get_boxptrs_doubleptr, summary.syslinereader_get_boxptrs_doubleptr,
        syslinereader_get_boxptrs_multiptr, summary.syslinereader_get_boxptrs_multiptr,
        syslinereader_ezcheck12_hit, summary.syslinereader_ezcheck12_hit,
        syslinereader_ezcheck12_miss, summary.syslinereader_ezcheck12_miss,
        syslinereader_ezcheck12_hit_max, summary.syslinereader_ezcheck12_hit_max,
        syslinereader_ezcheckd2_hit, summary.syslinereader_ezcheckd2_hit,
        syslinereader_ezcheckd2_miss, summary.syslinereader_ezcheckd2_miss,
        syslinereader_ezcheckd2_hit_max, summary.syslinereader_ezcheckd2_hit_max,
        syslinereader_ezcheck12d2_hit, summary.syslinereader_ezcheck12d2_hit,
        syslinereader_ezcheck12d2_miss, summary.syslinereader_ezcheck12d2_miss,
        syslinereader_ezcheck12d2_hit_max, summary.syslinereader_ezcheck12d2_hit_max,
    );
    assert_eq!(
        syslinereader_drop_sysline_ok,
        summary.syslinereader_drop_sysline_ok,
        "syslinereader_drop_sysline_ok 1"
    );
    assert_eq!(
        syslinereader_drop_sysline_errors,
        summary.syslinereader_drop_sysline_errors,
        "syslinereader_drop_sysline_errors 2"
    );
    assert_eq!(
        syslinereader_syslines,
        summary.syslinereader_syslines,
        "syslinereader_syslines 3"
    );
    assert_eq!(
        syslinereader_syslines_stored_highest,
        summary.syslinereader_syslines_stored_highest,
        "syslinereader_syslines_stored_highest 4"
    );
    assert_eq!(
        syslinereader_syslines_hit,
        summary.syslinereader_syslines_hit,
        "syslinereader_syslines_hit 5"
    );
    assert_eq!(
        syslinereader_syslines_miss,
        summary.syslinereader_syslines_miss,
        "syslinereader_syslines_miss 6"
    );
    assert_eq!(
        syslinereader_syslines_by_range_hit,
        summary.syslinereader_syslines_by_range_hit,
        "syslinereader_syslines_by_range_hit 7"
    );
    assert_eq!(
        syslinereader_syslines_by_range_miss,
        summary.syslinereader_syslines_by_range_miss,
        "syslinereader_syslines_by_range_miss 8"
    );
    assert_eq!(
        syslinereader_syslines_by_range_put,
        summary.syslinereader_syslines_by_range_put,
        "syslinereader_syslines_by_range_put 9"
    );
    assert_eq!(
        syslinereader_datetime_first,
        summary.syslinereader_datetime_first,
        "syslinereader_datetime_first 10"
    );
    assert_eq!(
        syslinereader_datetime_last,
        summary.syslinereader_datetime_last,
        "syslinereader_datetime_last 11"
    );
    assert_eq!(
        syslinereader_find_sysline_lru_cache_hit,
        summary.syslinereader_find_sysline_lru_cache_hit,
        "syslinereader_find_sysline_lru_cache_hit 12"
    );
    assert_eq!(
        syslinereader_find_sysline_lru_cache_miss,
        summary.syslinereader_find_sysline_lru_cache_miss,
        "syslinereader_find_sysline_lru_cache_miss 13"
    );
    assert_eq!(
        syslinereader_find_sysline_lru_cache_put,
        summary.syslinereader_find_sysline_lru_cache_put,
        "syslinereader_find_sysline_lru_cache_put 14"
    );
    assert_eq!(
        syslinereader_parse_datetime_in_line_lru_cache_hit,
        summary.syslinereader_parse_datetime_in_line_lru_cache_hit,
        "syslinereader_parse_datetime_in_line_lru_cache_hit 15"
    );
    assert_eq!(
        syslinereader_parse_datetime_in_line_lru_cache_miss,
        summary.syslinereader_parse_datetime_in_line_lru_cache_miss,
        "syslinereader_parse_datetime_in_line_lru_cache_miss 16"
    );
    assert_eq!(
        syslinereader_parse_datetime_in_line_lru_cache_put,
        summary.syslinereader_parse_datetime_in_line_lru_cache_put,
        "syslinereader_parse_datetime_in_line_lru_cache_put 17"
    );
    assert_eq!(
        syslinereader_regex_captures_attempted,
        summary.syslinereader_regex_captures_attempted,
        "syslinereader_regex_captures_attempted 18"
    );
    assert_eq!(
        syslinereader_get_boxptrs_singleptr,
        summary.syslinereader_get_boxptrs_singleptr,
        "syslinereader_get_boxptrs_singleptr 19"
    );
    assert_eq!(
        syslinereader_get_boxptrs_doubleptr,
        summary.syslinereader_get_boxptrs_doubleptr,
        "syslinereader_get_boxptrs_doubleptr 20"
    );
    assert_eq!(
        syslinereader_get_boxptrs_multiptr,
        summary.syslinereader_get_boxptrs_multiptr,
        "syslinereader_get_boxptrs_multiptr 21"
    );
    assert_eq!(
        syslinereader_ezcheck12_hit,
        summary.syslinereader_ezcheck12_hit,
        "syslinereader_ezcheck12_hit 22"
    );
    assert_eq!(
        syslinereader_ezcheck12_miss,
        summary.syslinereader_ezcheck12_miss,
        "syslinereader_ezcheck12_miss 23"
    );
    assert_eq!(
        syslinereader_ezcheck12_hit_max,
        summary.syslinereader_ezcheck12_hit_max,
        "syslinereader_ezcheck12_hit_max 24"
    );
    assert_eq!(
        syslinereader_ezcheckd2_hit,
        summary.syslinereader_ezcheckd2_hit,
        "syslinereader_ezcheckd2_hit 25"
    );
    assert_eq!(
        syslinereader_ezcheckd2_miss,
        summary.syslinereader_ezcheckd2_miss,
        "syslinereader_ezcheckd2_miss 26"
    );
    assert_eq!(
        syslinereader_ezcheckd2_hit_max,
        summary.syslinereader_ezcheckd2_hit_max,
        "syslinereader_ezcheckd2_hit_max 27"
    );
    assert_eq!(
        syslinereader_ezcheck12d2_hit,
        summary.syslinereader_ezcheck12d2_hit,
        "syslinereader_ezcheck12d2_hit 28"
    );
    assert_eq!(
        syslinereader_ezcheck12d2_miss,
        summary.syslinereader_ezcheck12d2_miss,
        "syslinereader_ezcheck12d2_miss 29"
    );
    assert_eq!(
        syslinereader_ezcheck12d2_hit_max,
        summary.syslinereader_ezcheck12d2_hit_max,
        "syslinereader_ezcheck12d2_hit_max 30"
    );
}
