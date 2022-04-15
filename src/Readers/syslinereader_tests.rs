// Readers/syslinereader_tests.rs
//

use crate::common::{
    FileOffset,
};

#[cfg(test)]
use crate::common::{
    FPath,
    Path,
};

#[cfg(test)]
use crate::Readers::blockreader::{
    BlockSz,
};

//#[cfg(test)]
//use crate::Readers::linereader::{
    //LineIndex,
    //enum_BoxPtrs,
//};

#[cfg(test)]
use crate::Readers::syslinereader::{
    randomize,
    fill,
    dt_pattern_has_year,
    dt_pattern_has_tz,
    DATETIME_PARSE_DATAS,
    DATETIME_PARSE_DATAS_VEC,
    DateTimeL,
    DateTimePattern,
    //DateTimePattern_str,
    Result_Filter_DateTime1,
    str_datetime,
    //slice_contains_X_2,
};

#[cfg(any(debug_assertions,test))]
use crate::Readers::syslinereader::{
    DateTimeL_Opt,
    Result_Filter_DateTime2,
    SyslineP,
    SyslineReader,
    ResultS4_SyslineFind,
};

#[cfg(test)]
use crate::dbgpr::helpers::{
    NamedTempFile,
    create_temp_file,
    create_temp_file_bytes,
};

#[cfg(test)]
use crate::dbgpr::printers::{
    //byte_to_char_noraw,
    buffer_to_String_noraw,
    str_to_String_noraw,
    file_to_String_noraw,
};

#[cfg(any(debug_assertions,test))]
use crate::dbgpr::printers::{
    write_stdout,
    print_colored_stdout,
    //print_colored_stderr,
    Color,
};

#[cfg(test)]
use crate::dbgpr::stack::{
    stack_offset_set,
};

#[cfg(any(debug_assertions,test))]
use crate::dbgpr::stack::{
    sn,
    //snx,
    so,
    sx,
};

extern crate chrono;
#[cfg(test)]
use chrono::{
    //DateTime,
    FixedOffset,
    Local,
    //Offset,
    //NaiveDateTime,
    TimeZone,
    //Utc
};

extern crate more_asserts;
#[cfg(test)]
use more_asserts::{
    //assert_eq,
    assert_le,
    assert_lt,
    assert_ge,
    //debug_assert_le,
    //debug_assert_lt,
    //debug_assert_ge,
    //debug_assert_gt,
};

//#[cfg(any(debug_assertions,test))]
//use std::result::Result;
#[cfg(test)]
use std::str;

extern crate lazy_static;
#[cfg(test)]
use lazy_static::lazy_static;

/// built-in sanity check of the static DATETIME_PARSE_DATAS
/// can only check for coarse errors, cannot check catch all errors
#[test]
fn test_DATETIME_PARSE_DATAS() {
    for dtpd in DATETIME_PARSE_DATAS_VEC.iter() {
        assert_lt!(dtpd.sib, dtpd.sie, "dtpd.sib < dtpd.sie failed; bad build-in DateTimeParseData {:?}", dtpd);
        assert_lt!(dtpd.siba, dtpd.siea, "dtpd.siba < dtpd.siea failed; bad build-in DateTimeParseData {:?}", dtpd);
        assert_le!(dtpd.sib, dtpd.siba, "dtpd.sib ≤ dtpd.siba failed; bad build-in DateTimeParseData {:?}", dtpd);
        assert_ge!(dtpd.sie, dtpd.siea, "dtpd.sie ≥ dtpd.siea failed; bad build-in DateTimeParseData {:?}", dtpd);
        let len_ = dtpd.pattern.len();
        // XXX: arbitrary minimum
        assert_le!(6, len_, ".pattern.len too short; bad build-in DateTimeParseData {:?}", dtpd);
        let diff_ = dtpd.sie - dtpd.sib;
        let diff_dt = dtpd.siea - dtpd.siba;
        assert_ge!(diff_, diff_dt, "len(.sib,.sie) ≥ len(.siba,.siea) failed; bad build-in DateTimeParseData {:?}", dtpd);
        assert_ge!(diff_, len_, "len(.sib,.sie) ≥ .dtp.len() failed; bad build-in DateTimeParseData {:?}", dtpd);
        //assert_le!(diff_dt, len_, "len(.3,.4) ≤ .0.len() failed; bad build-in DateTimeParseData {:?}", dtpd);
        if dtpd.year {
            assert!(dt_pattern_has_year(&dtpd.pattern), "pattern has not year {:?} but .year is true", dtpd.pattern);
        } else {
            assert!(!dt_pattern_has_year(&dtpd.pattern), "pattern has year {:?} but .year is false", dtpd.pattern);
        }
        if dtpd.tz {
            assert!(dt_pattern_has_tz(&dtpd.pattern), "pattern has not timezone {:?} but tz is true", dtpd.pattern);
        } else {
            assert!(!dt_pattern_has_tz(&dtpd.pattern), "pattern has timezone {:?} but tz is false", dtpd.pattern);
        }
    }
    // check for duplicates
    let mut check = DATETIME_PARSE_DATAS_VEC.clone();
    check.sort_unstable();
    let check_orig = check.clone();
    check.dedup();
    //let check: DateTime_Parse_Datas_vec = DATETIME_PARSE_DATAS.into_iter().unique().collect();
    if check.len() != DATETIME_PARSE_DATAS.len() {
        for (i, (co, cd)) in check_orig.iter().zip(check.iter()).enumerate() {
            eprintln!("entry {} {:?} {:?}", i, co, cd);
        }
        for (co, cd) in check_orig.iter().zip(check.iter()) {
            assert_eq!(co, cd, "entry {:?} appears to be a duplicate", co);
        }
    };
    assert_eq!(check.len(), DATETIME_PARSE_DATAS.len(), "the deduplicated DATETIME_PARSE_DATAS_VEC is different len than original; there are duplicates in DATETIME_PARSE_DATAS_VEC but the test could not determine which entry.");
}

#[test]
fn test_datetime_from_str_workaround_Issue660() {
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("", ""));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("a", ""));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("", "a"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660(" ", ""));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("", " "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" ", " "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" a", " a"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660(" a", "  a"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("  a", " a"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("  a", "   a"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("a", "   a"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("  a", "a"));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("a ", "a "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("a  ", "a  "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" a ", " a "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" a  ", " a  "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("   a  ", "   a  "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("   a  ", "   a  "));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("   a  ", "   a   "));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("   a   ", "   a  "));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("   a   ", " a  "));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("a   ", " a  "));

    assert!(!SyslineReader::datetime_from_str_workaround_Issue660(" \t", " "));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660(" ", "\t "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \t", "\t "));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("\t ", "\t a\t"));

    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n\t", " \n\t"));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n\t", " \t\n"));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n\ta", " \t\n"));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n\t", " \t\na"));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n", " \n"));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n", "\n "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n", "\r "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n", " \n"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("\t a", "\t a\t\n"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("\t\n a\n", "\t\n a\t\n"));
}

/// basic test of `SyslineReader.find_datetime_in_line`
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_find_datetime_in_line_by_block(blocksz: BlockSz) {
    eprintln!("{}_test_find_datetime_in_line_by_block()", sn());

    let ntf1 = create_temp_file(
        "\
[20200113-11:03:06] [DEBUG] Testing if xrdp can listen on 0.0.0.0 port 3389.
[20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
CLOSED!
[20200113-11:03:08] [INFO ] starting xrdp with pid 23198
[20200113-11:03:08] [INFO ] listening to port 3389 on 0.0.0.0
[20200113-11:13:59] [INFO ] Socket 12: AF_INET6 connection received from ::ffff:127.0.0.1 port 55426
[20200113-11:13:59] [DEBUG] Closed socket 12 (AF_INET6 ::ffff:127.0.0.1 port 3389)
[20200113-11:13:59] [DEBUG] Closed socket 11 (AF_INET6 :: port 3389)
[20200113-11:13:59] [INFO ] Using default X.509 certificate: /etc/xrdp/cert.pem
[20200113-11:13:59] [INFO ] Using default X.509 key file: /etc/xrdp/key.pem
[20200113-11:13:59] [DEBUG] read private key file /etc/xrdp/key.pem
[20200113-11:13:59] [DEBUG] Certification found
    FOUND CERTIFICATE!
[20200113-11:13:59] [DEBUG] Certification complete.
",
    );
    let path = String::from(ntf1.path().to_str().unwrap());

    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = match SyslineReader::new(&path, blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({:?}, {}) failed {}", &path, blocksz, err);
        }
    };

    let mut fo1: FileOffset = 0;
    loop {
        let result = slr.find_sysline(fo1);
        let done = result.is_done() || result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                /*
                eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Found|Found_EOF({}, @{:p})", so(), fo1, fo, &*slp);
                eprintln!(
                    "{}test_find_datetime_in_line: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                */
                print_slp(&slp);
                fo1 = fo;
            }
            ResultS4_SyslineFind::Done => {
                //eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Done", so(), fo1);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Err({})", so(), fo1, err);
                panic!("ERROR: test_find_datetime_in_line: slr.find_sysline({}) returned Err({})", fo1, err);
            }
        }
        if done {
            break;
        }
    }

    eprintln!("{}_test_find_datetime_in_line_by_block()", sx());
}

#[test]
fn test_find_datetime_in_line_by_block2() {
    _test_find_datetime_in_line_by_block(2);
}

#[test]
fn test_find_datetime_in_line_by_block4() {
    _test_find_datetime_in_line_by_block(4);
}

#[test]
fn test_find_datetime_in_line_by_block8() {
    _test_find_datetime_in_line_by_block(8);
}

#[test]
fn test_find_datetime_in_line_by_block256() {
    _test_find_datetime_in_line_by_block(256);
}

#[cfg(test)]
type _test_find_sysline_at_datetime_filter_Checks<'a> = Vec<(FileOffset, &'a str, &'a str)>;

/// underlying test code for `SyslineReader.find_datetime_in_line`
/// called by other functions `test_find_sysline_at_datetime_filterX`
#[cfg(test)]
fn __test_find_sysline_at_datetime_filter(
    file_content: String, dt_pattern: DateTimePattern, blocksz: BlockSz,
    checks: _test_find_sysline_at_datetime_filter_Checks,
) {
    eprintln!("{}__test_find_sysline_at_datetime_filter(…, {:?}, {}, …)", sn(), dt_pattern, blocksz);

    let ntf1 = create_temp_file(file_content.as_str());
    let path = String::from(ntf1.path().to_str().unwrap());
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = match SyslineReader::new(&path, blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({:?}, {}) failed {}", &path, blocksz, err);
        }
    };
    for (fo1, dts, sline_expect) in checks.iter() {
        //let dt = match Local.datetime_from_str(dts, dt_pattern.as_str()) {
        // TODO: add `has_tz` to `checks`, remove this
        let tzo = FixedOffset::west(3600 * 8);
        let has_tz = dt_pattern_has_tz(&dt_pattern.as_str());
        eprintln!("{}str_datetime({:?}, {:?}, {:?}, {:?})", so(), str_to_String_noraw(dts), dt_pattern, has_tz, &tzo);
        let dt = match str_datetime(dts, &dt_pattern.as_str(), has_tz, &tzo) {
            Some(val) => val,
            None => {
                panic!("ERROR: datetime_from_str({:?}, {:?}) returned None", dts, dt_pattern);
            }
        };
        let sline_expect_noraw = str_to_String_noraw(sline_expect);
        eprintln!("{}find_sysline_at_datetime_filter({}, {:?})", so(), fo1, dt);
        let result = slr.find_sysline_at_datetime_filter(*fo1, &Some(dt));
        match result {
            ResultS4_SyslineFind::Found(val) | ResultS4_SyslineFind::Found_EOF(val) => {
                let sline = val.1.to_String();
                let sline_noraw = str_to_String_noraw(sline.as_str());
                eprintln!("\nexpected: {:?}", sline_expect_noraw);
                eprintln!("returned: {:?}\n", sline_noraw);
                //print_colored(Color::Yellow, format!("expected: {}\n", sline_expect_noraw).as_bytes());
                //print_colored(Color::Yellow, format!("returned: {}\n", sline_noraw).as_bytes());
                assert_eq!(
                    sline,
                    String::from(*sline_expect),
                    "Expected {:?} == {:?} but it is not!",
                    sline_noraw,
                    sline_expect_noraw
                );
                //eprintln!("{}Check PASSED {:?}", so(), sline_noraw);
                #[allow(clippy::match_single_binding)]
                match print_colored_stdout(
                    Color::Green,
                    format!(
                        "Check PASSED SyslineReader().find_sysline_at_datetime_filter({} {:?}) == {:?}\n",
                        fo1, dts, sline_noraw
                    )
                    .as_bytes(),
                ) {
                    _ => {},
                };
            }
            ResultS4_SyslineFind::Done => {
                panic!("During test unexpected result Done");
            }
            ResultS4_SyslineFind::Err(err) => {
                panic!("During test unexpected result Error {}", err);
            }
        }
    }

    eprintln!("{}_test_find_sysline_at_datetime_filter(…)", sx());
}

// TODO: [2022/03/16] create test cases with varying sets of Checks passed-in, current setup is always
//       clean, sequential series of checks from file_offset 0.
// TODO: BUG: [2022/03/15] why are these checks done in random order? The tests pass but run
//       in a confusing manner. Run `cargo test` to see.
/// basic test of `SyslineReader.find_datetime_in_line`
#[cfg(test)]
fn _test_find_sysline_at_datetime_filter(
    blocksz: BlockSz, checks: Option<_test_find_sysline_at_datetime_filter_Checks>,
) {
    stack_offset_set(None);
    eprintln!("{}_test_find_sysline_at_datetime_filter()", sn());
    let dt_fmt1: DateTimePattern = String::from("%Y-%m-%d %H:%M:%S");
    let file_content1 = String::from(
        "\
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
",
    );
    let checks0: _test_find_sysline_at_datetime_filter_Checks = Vec::from([
        (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
        (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        (0, "2020-01-01 00:00:04", "2020-01-01 00:00:04abcd\n"),
        (0, "2020-01-01 00:00:05", "2020-01-01 00:00:05abcde\n"),
        (0, "2020-01-01 00:00:06", "2020-01-01 00:00:06abcdef\n"),
        (0, "2020-01-01 00:00:07", "2020-01-01 00:00:07abcdefg\n"),
        (0, "2020-01-01 00:00:08", "2020-01-01 00:00:08abcdefgh\n"),
        (0, "2020-01-01 00:00:09", "2020-01-01 00:00:09abcdefghi\n"),
        (0, "2020-01-01 00:00:10", "2020-01-01 00:00:10abcdefghij\n"),
        (0, "2020-01-01 00:00:11", "2020-01-01 00:00:11abcdefghijk\n"),
        (0, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
        (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
        (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        (0, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
        (0, "2020-01-01 00:00:16", "2020-01-01 00:00:16abcdefghijklmnop\n"),
        (0, "2020-01-01 00:00:17", "2020-01-01 00:00:17abcdefghijklmnopq\n"),
        (0, "2020-01-01 00:00:18", "2020-01-01 00:00:18abcdefghijklmnopqr\n"),
        (0, "2020-01-01 00:00:19", "2020-01-01 00:00:19abcdefghijklmnopqrs\n"),
        (0, "2020-01-01 00:00:20", "2020-01-01 00:00:20abcdefghijklmnopqrst\n"),
        (0, "2020-01-01 00:00:21", "2020-01-01 00:00:21abcdefghijklmnopqrstu\n"),
        (0, "2020-01-01 00:00:22", "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n"),
        (0, "2020-01-01 00:00:23", "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n"),
        (0, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
        (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
    ]);

    let _checksx: _test_find_sysline_at_datetime_filter_Checks = Vec::from([
        (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        (19, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        (40, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
        (62, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        (85, "2020-01-01 00:00:04", "2020-01-01 00:00:04abcd\n"),
        (109, "2020-01-01 00:00:05", "2020-01-01 00:00:05abcde\n"),
        (134, "2020-01-01 00:00:06", "2020-01-01 00:00:06abcdef\n"),
        (162, "2020-01-01 00:00:07", "2020-01-01 00:00:07abcdefg\n"),
        (187, "2020-01-01 00:00:08", "2020-01-01 00:00:08abcdefgh\n"),
        (215, "2020-01-01 00:00:09", "2020-01-01 00:00:09abcdefghi\n"),
        (244, "2020-01-01 00:00:10", "2020-01-01 00:00:10abcdefghij\n"),
        (274, "2020-01-01 00:00:11", "2020-01-01 00:00:11abcdefghijk\n"),
        (305, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
        (337, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
        (370, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        (404, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
        (439, "2020-01-01 00:00:16", "2020-01-01 00:00:16abcdefghijklmnop\n"),
        (475, "2020-01-01 00:00:17", "2020-01-01 00:00:17abcdefghijklmnopq\n"),
        (512, "2020-01-01 00:00:18", "2020-01-01 00:00:18abcdefghijklmnopqr\n"),
        (550, "2020-01-01 00:00:19", "2020-01-01 00:00:19abcdefghijklmnopqrs\n"),
        (589, "2020-01-01 00:00:20", "2020-01-01 00:00:20abcdefghijklmnopqrst\n"),
        (629, "2020-01-01 00:00:21", "2020-01-01 00:00:21abcdefghijklmnopqrstu\n"),
        (670, "2020-01-01 00:00:22", "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n"),
        (712, "2020-01-01 00:00:23", "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n"),
        (755, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
        (799, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        (844, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
    ]);

    let checks_: _test_find_sysline_at_datetime_filter_Checks = checks.unwrap_or(checks0);
    __test_find_sysline_at_datetime_filter(file_content1, dt_fmt1, blocksz, checks_);
    eprintln!("{}_test_find_sysline_at_datetime_filter()", sx());
}

// XXX: are these different BlockSz tests necessary? are not these adequately tested by
//      other lower-level tests?

#[test]
fn test_find_sysline_at_datetime_filter_4() {
    _test_find_sysline_at_datetime_filter(4, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_8() {
    _test_find_sysline_at_datetime_filter(8, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_16() {
    _test_find_sysline_at_datetime_filter(16, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_32() {
    _test_find_sysline_at_datetime_filter(32, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_64() {
    _test_find_sysline_at_datetime_filter(64, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_128() {
    _test_find_sysline_at_datetime_filter(128, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_256() {
    _test_find_sysline_at_datetime_filter(256, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_512() {
    _test_find_sysline_at_datetime_filter(512, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_1024() {
    _test_find_sysline_at_datetime_filter(1024, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_2056() {
    _test_find_sysline_at_datetime_filter(2056, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_() {
    _test_find_sysline_at_datetime_filter(64,Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:00",
            "2020-01-01 00:00:00\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_a() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:01",
            "2020-01-01 00:00:01a\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_b() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:02",
            "2020-01-01 00:00:02ab\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_c() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:03",
            "2020-01-01 00:00:03abc\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_d() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:04",
            "2020-01-01 00:00:04abcd\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_e() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:05",
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_f() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:06",
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_g() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:07",
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_h() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:08",
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_i() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:09",
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_j() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:10",
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_k() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:11",
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_l() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:12",
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_m() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:13",
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_n() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:14",
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_o() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:15",
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_p() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:16",
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_q() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:17",
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_r() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:18",
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_s() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:19",
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_t() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:20",
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_u() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:21",
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_v() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:22",
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_w() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:23",
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_x() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:24",
            "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_y() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:25",
            "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_z() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:26",
            "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_a() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            19,
            "2020-01-01 00:00:01",
            "2020-01-01 00:00:01a\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_b() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            40,
            "2020-01-01 00:00:02",
            "2020-01-01 00:00:02ab\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_c() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            62,
            "2020-01-01 00:00:03",
            "2020-01-01 00:00:03abc\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_d() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            85,
            "2020-01-01 00:00:04",
            "2020-01-01 00:00:04abcd\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_e() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            109,
            "2020-01-01 00:00:05",
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_f() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            134,
            "2020-01-01 00:00:06",
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_g() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            160,
            "2020-01-01 00:00:07",
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_h() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            187,
            "2020-01-01 00:00:08",
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_i() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            215,
            "2020-01-01 00:00:09",
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_j() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            244,
            "2020-01-01 00:00:10",
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_k() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            274,
            "2020-01-01 00:00:11",
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_l() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            305,
            "2020-01-01 00:00:12",
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_m() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            337,
            "2020-01-01 00:00:13",
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_n() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            370,
            "2020-01-01 00:00:14",
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_o() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            404,
            "2020-01-01 00:00:15",
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_p() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            439,
            "2020-01-01 00:00:16",
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_q() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            475,
            "2020-01-01 00:00:17",
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_r() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            512,
            "2020-01-01 00:00:18",
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_s() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            550,
            "2020-01-01 00:00:19",
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_t() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            589,
            "2020-01-01 00:00:20",
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_u() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            629,
            "2020-01-01 00:00:21",
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_v() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            670,
            "2020-01-01 00:00:22",
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_w() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            712,
            "2020-01-01 00:00:23",
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_x() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            755,
            "2020-01-01 00:00:24",
            "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_y() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            799,
            "2020-01-01 00:00:25",
            "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_z() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            844,
            "2020-01-01 00:00:26",
            "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_z_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_y_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_x_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_m_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_za() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_ya() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_xa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_ma() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3____() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__ab() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__az() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__bd() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:04", "2020-01-01 00:00:04abcd\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__ml() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__my() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__mz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__m_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aaa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abc() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aba() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abn() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aby() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aaz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_byo() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zaa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbc() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zba() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbn() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zby() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zaz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybc() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yba() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybn() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yby() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

// TODO: [2022/03/18] create one wrapper test test_find_sysline_at_datetime_checks_ that takes some
//        vec of test-input-output, and does all possible permutations.

/// basic test of `SyslineReader.sysline_pass_filters`
/// TODO: add tests with TZ
#[allow(non_snake_case)]
#[test]
fn test_sysline_pass_filters() {
    eprintln!("{}test_sysline_pass_filters()", sn());

    fn DTL(s: &str) -> DateTimeL {
        //return DateTimeL.datetime_from_str(s, "%Y%m%dT%H%M%S").unwrap();
        let tzo = FixedOffset::west(3600 * 8);
        str_datetime(s, "%Y%m%dT%H%M%S", false, &tzo).unwrap()
    }

    for (da, dt, db, exp_result) in [
        (
            Some(DTL("20000101T010105")),
            DTL("20000101T010106"),
            Some(DTL("20000101T010107")),
            Result_Filter_DateTime2::OccursInRange,
        ),
        (
            Some(DTL("20000101T010107")),
            DTL("20000101T010106"),
            Some(DTL("20000101T010108")),
            Result_Filter_DateTime2::OccursBeforeRange,
        ),
        (
            Some(DTL("20000101T010101")),
            DTL("20000101T010106"),
            Some(DTL("20000101T010102")),
            Result_Filter_DateTime2::OccursAfterRange,
        ),
        (Some(DTL("20000101T010101")), DTL("20000101T010106"), None, Result_Filter_DateTime2::OccursInRange),
        (
            Some(DTL("20000101T010102")),
            DTL("20000101T010101"),
            None,
            Result_Filter_DateTime2::OccursBeforeRange,
        ),
        (Some(DTL("20000101T010101")), DTL("20000101T010101"), None, Result_Filter_DateTime2::OccursInRange),
        (None, DTL("20000101T010101"), Some(DTL("20000101T010106")), Result_Filter_DateTime2::OccursInRange),
        (
            None,
            DTL("20000101T010101"),
            Some(DTL("20000101T010100")),
            Result_Filter_DateTime2::OccursAfterRange,
        ),
        (None, DTL("20000101T010101"), Some(DTL("20000101T010101")), Result_Filter_DateTime2::OccursInRange),
    ] {
        let result = SyslineReader::dt_pass_filters(&dt, &da, &db);
        assert_eq!(exp_result, result, "Expected {:?} Got {:?} for ({:?}, {:?}, {:?})", exp_result, result, dt, da, db);
        eprintln!("SyslineReader::dt_pass_filters(\n\t{:?},\n\t{:?},\n\t{:?}\n)\nreturned expected {:?}", dt, da, db, result);
        /*
        #[allow(unused_must_use)]
        #[allow(clippy::match_single_binding)]
        match print_colored_stdout(
            Color::Green,
            format!("{}({:?}, {:?}, {:?}) returned expected {:?}\n", so(), dt, da, db, result).as_bytes(),
        ) {
            _ => {},
        }
        */
    }
    eprintln!("{}test_sysline_pass_filters()", sx());
}

/// basic test of `SyslineReader.dt_after_or_before`
/// TODO: add tests with TZ
#[allow(non_snake_case)]
#[test]
fn test_dt_after_or_before() {
    eprintln!("{}test_dt_after_or_before()", sn());

    fn DTL(s: &str) -> DateTimeL {
        let tzo = FixedOffset::west(3600 * 8);
        str_datetime(s, "%Y%m%dT%H%M%S", false, &tzo).unwrap()
    }

    for (dt, da, exp_result) in [
        (DTL("20000101T010106"), None, Result_Filter_DateTime1::Pass),
        (DTL("20000101T010101"), Some(DTL("20000101T010103")), Result_Filter_DateTime1::OccursBefore),
        (DTL("20000101T010100"), Some(DTL("20000101T010100")), Result_Filter_DateTime1::OccursAtOrAfter),
        (DTL("20000101T010109"), Some(DTL("20000101T010108")), Result_Filter_DateTime1::OccursAtOrAfter),
    ] {
        let result = SyslineReader::dt_after_or_before(&dt, &da);
        assert_eq!(exp_result, result, "Expected {:?} Got {:?} for ({:?}, {:?})", exp_result, result, dt, da);
        eprintln!("SyslineReader::dt_after_or_before(\n\t{:?},\n\t{:?}\n)\nreturned expected {:?}", dt, da, result);
        /*
        #[allow(unused_must_use)]
        #[allow(clippy::match_single_binding)]
        match print_colored_stdout(
            Color::Green,
            format!("{}({:?}, {:?}) returned expected {:?}\n", so(), dt, da, result).as_bytes(),
        ) {
            _ => {},
        }
        */
    }
    eprintln!("{}test_dt_after_or_before()", sx());
}

/// testing helper
/// if debug then print with color
/// else print efficiently
/// XXX: does not handle multi-byte
/// BUG: if `(*slp).dt_beg` or `(*slp).dt_end` are within multi-byte encoded character
///      then this will panic. e.g. Sysline with underlying "2000-01-01 00:00:00\n".to_String_noraw()
///      will return "2000-01-01 00:00:00␊". Which will panic:
///          panicked at 'byte index 20 is not a char boundary; it is inside '␊' (bytes 19..22) of `2000-01-01 00:00:00␊`'
///      However, this function is only an intermediary development helper. Can this problem have a
///      brute-force workaround. 
#[allow(dead_code)]
#[cfg(any(debug_assertions,test))]
fn print_slp(slp: &SyslineP) {
    if cfg!(debug_assertions) {
        let out = (*slp).to_String_noraw();
        // XXX: presumes single-byte character encoding, does not handle multi-byte encoding
        /*
        eprintln!("{}print_slp: to_String_noraw() {:?} dt_beg {} dt_end {} len {}", so(), out, split_ab, (*slp).dt_end, (*slp).len());
        eprintln!("{}print_slp: out.chars():", so());
        for (c_n, c_) in out.chars().enumerate() {
            eprintln!("{}print_slp:              char {} {:?}", so(), c_n, c_);
        }
        eprintln!("{}print_slp: out.bytes():", so());
        for (b_n, b_) in out.bytes().enumerate() {
            eprintln!("{}print_slp:              byte {} {:?}", so(), b_n, b_);
        }
        */
        let a = &out[..(*slp).dt_beg];
        match print_colored_stdout(Color::Green, a.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: print_colored a returned error {}", err);
            }
        };
        let b = &out[(*slp).dt_beg..(*slp).dt_end];
        match print_colored_stdout(Color::Yellow, b.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: print_colored b returned error {}", err);
            }
        };
        let c = &out[(*slp).dt_end..];
        match print_colored_stdout(Color::Green, c.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: print_colored c returned error {}", err);
            }
        };
        println!();
    } else {
        //(*slp_).print(true);
        let slices = (*slp).get_slices();
        for slice in slices.iter() {
            write_stdout(slice);
        }
    }
}

#[cfg(test)]
type _test_SyslineReader_check<'a> = (&'a str, FileOffset);

#[cfg(test)]
type _test_SyslineReader_checks<'a> = Vec<(&'a str, FileOffset)>;

/// basic test of SyslineReader things
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader(path: &Path, blocksz: BlockSz, fileoffset: FileOffset, checks: &_test_SyslineReader_checks) {
    eprintln!("{}test_SyslineReader({:?}, {})", sn(), &path, blocksz);
    let fpath: FPath = path.to_str().unwrap_or("").to_string();
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = match SyslineReader::new(&fpath, blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({:?}, {}) failed {}", fpath, blocksz, err);
        }
    };
    eprintln!("{}test_SyslineReader: {:?}", so(), slr);

    let mut fo1: FileOffset = fileoffset;
    let mut check_i: usize = 0;
    loop {
        let result = slr.find_sysline(fo1);
        let done = result.is_done() || result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) => {
                eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Found({}, @{:p})", so(), fo1, fo, &*slp);
                eprintln!(
                    "{}test_SyslineReader: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print_slp(&slp);
                assert!(!slr.is_sysline_last(&slp), "returned Found yet this Sysline is last! Should have returned Found_EOF or is this Sysline not last?");
                fo1 = fo;

                eprintln!("{}test_SyslineReader: check {}", so(), check_i);
                // check slp.String
                let check_String = checks[check_i].0.to_string();
                let actual_String = (*slp).to_String();
                assert_eq!(check_String, actual_String,"\nexpected string value {:?}\nfind_sysline returned {:?}", check_String, actual_String);
                // check fileoffset
                let check_fo = checks[check_i].1;
                assert_eq!(check_fo, fo, "expected fileoffset {}, but find_sysline returned fileoffset {} for check {}", check_fo, fo, check_i);
            }
            ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Found_EOF({}, @{:p})", so(), fo1, fo, &*slp);
                eprintln!(
                    "{}test_SyslineReader: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print_slp(&slp);
                assert!(slr.is_sysline_last(&slp), "returned Found_EOF yet this Sysline is not last!");
                fo1 = fo;

                eprintln!("{}test_SyslineReader: check {}", so(), check_i);
                // check slp.String
                let check_String = checks[check_i].0.to_string();
                let actual_String = (*slp).to_String();
                assert_eq!(check_String, actual_String,"\nexpected string value {:?}\nfind_sysline returned {:?}", check_String, actual_String);
                // check fileoffset
                let check_fo = checks[check_i].1;
                assert_eq!(check_fo, fo, "expected fileoffset {}, but find_sysline returned fileoffset {} for check {}", check_fo, fo, check_i);
            }
            ResultS4_SyslineFind::Done => {
                eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Done", so(), fo1);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Err({})", so(), fo1, err);
                panic!("ERROR: {}", err);
            }
        }
        check_i += 1;
        if done {
            break;
        }
    }
    assert_eq!(checks.len(), check_i, "expected {} Sysline checks but only {} Sysline checks were done", checks.len(), check_i);

    eprintln!("{}test_SyslineReader: Found {} Lines, {} Syslines", so(), slr.linereader.count(), slr.syslines.len());
    eprintln!("{}test_SyslineReader({:?}, {})", sx(), &path, blocksz);
}

#[allow(non_upper_case_globals)]
#[cfg(test)]
static test_data_file_A_dt6: &str = "\
2000-01-01 00:00:00
2000-01-01 00:00:01a
2000-01-01 00:00:02ab
2000-01-01 00:00:03abc
2000-01-01 00:00:04abcd
2000-01-01 00:00:05abcde";

#[allow(non_upper_case_globals)]
#[cfg(test)]
static test_data_file_A_dt6_checks: [_test_SyslineReader_check; 6] = [
    ("2000-01-01 00:00:00\n", 20),
    ("2000-01-01 00:00:01a\n", 41),
    ("2000-01-01 00:00:02ab\n", 63),
    ("2000-01-01 00:00:03abc\n", 86),
    ("2000-01-01 00:00:04abcd\n", 110),
    ("2000-01-01 00:00:05abcde", 134),
];

#[cfg(test)]
lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref test_SyslineReader_A_ntf: NamedTempFile = {
        create_temp_file(test_data_file_A_dt6)
    };
}

#[test]
fn test_SyslineReader_A_dt6_128_0_()
{
    let checks = _test_SyslineReader_checks::from(test_data_file_A_dt6_checks);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 0, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_1_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[1..]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 1, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_2_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[2..]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 40, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_3_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[3..]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 62, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_4_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[4..]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 85, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_5_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[5..]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 86, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_X_beforeend()
{
    let checks = _test_SyslineReader_checks::from([]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 132, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_X_pastend()
{
    let checks = _test_SyslineReader_checks::from([]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 135, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_X9999()
{
    let checks = _test_SyslineReader_checks::from([]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 9999, &checks);
}

// LAST WORKING HERE 2022/03/19 21:11:23 getting these tests test_SyslineReader_A_dt6* to work.
// After that, add *at least* one more data set.
//  see test_data_file_dt5
// then extraploate more tests for test_SyslineReader_w_filtering*

#[allow(non_upper_case_globals)]
#[cfg(test)]
static test_data_file_B_dt0: &str = "
foo
bar
";

#[allow(non_upper_case_globals)]
#[cfg(test)]
static test_data_file_B_dt0_checks: [_test_SyslineReader_check; 0] = [];

#[test]
fn test_SyslineReader_B_dt0_0()
{
    let ntf = create_temp_file(test_data_file_B_dt0);
    let checks = _test_SyslineReader_checks::from(test_data_file_B_dt0_checks);
    test_SyslineReader(ntf.path(), 128, 0, &checks);
}

#[test]
fn test_SyslineReader_B_dt0_3()
{
    let ntf = create_temp_file(test_data_file_B_dt0);
    let checks = _test_SyslineReader_checks::from(test_data_file_B_dt0_checks);
    test_SyslineReader(ntf.path(), 128, 3, &checks);
}

#[allow(non_upper_case_globals)]
#[cfg(test)]
static _test_data_file_C_dt6: &str = "\
[DEBUG] 2000-01-01 00:00:00
[DEBUG] 2000-01-01 00:00:01a
[DEBUG] 2000-01-01 00:00:02ab
[DEBUG] 2000-01-01 00:00:03abc
[DEBUG] 2000-01-01 00:00:04abcd
[DEBUG] 2000-01-01 00:00:05abcde";

#[allow(non_upper_case_globals)]
#[cfg(test)]
static _test_data_file_C_dt6_checks: [_test_SyslineReader_check; 6] = [
    ("[DEBUG] 2000-01-01 00:00:00\n", 28),
    ("[DEBUG] 2000-01-01 00:00:01a\n", 57),
    ("[DEBUG] 2000-01-01 00:00:02ab\n", 87),
    ("[DEBUG] 2000-01-01 00:00:03abc\n", 118),
    ("[DEBUG] 2000-01-01 00:00:04abcd\n", 150),
    ("[DEBUG] 2000-01-01 00:00:05abcde", 182),
];

#[cfg(test)]
lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref test_SyslineReader_C_ntf: NamedTempFile = {
        create_temp_file(_test_data_file_C_dt6)
    };
}

#[test]
fn test_SyslineReader_C_dt6_0()
{
    let checks = _test_SyslineReader_checks::from(_test_data_file_C_dt6_checks);
    test_SyslineReader(test_SyslineReader_C_ntf.path(), 128, 0, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_1()
{
    let checks = _test_SyslineReader_checks::from(&_test_data_file_C_dt6_checks[1..]);
    test_SyslineReader(test_SyslineReader_C_ntf.path(), 128, 3, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_2a()
{
    let checks = _test_SyslineReader_checks::from(&_test_data_file_C_dt6_checks[1..]);
    test_SyslineReader(test_SyslineReader_C_ntf.path(), 128, 27, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_2b()
{
    let checks = _test_SyslineReader_checks::from(&_test_data_file_C_dt6_checks[2..]);
    test_SyslineReader(test_SyslineReader_C_ntf.path(), 128, 28, &checks);
}

#[test]
fn test_SyslineReader_D_invalid1()
{
    let data_invalid1: [u8; 1] = [ 0xFF ];
    let date_checks1: _test_SyslineReader_checks = _test_SyslineReader_checks::from([]);
    let ntf1: NamedTempFile = create_temp_file_bytes(&data_invalid1);
    test_SyslineReader(ntf1.path(), 128, 0, &date_checks1);
}

/// basic test of SyslineReader things
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_w_filtering_1(
    path: &FPath, blocksz: BlockSz, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    eprintln!(
        "{}test_SyslineReader_w_filtering_1({:?}, {}, {:?}, {:?})",
        sn(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );

    if cfg!(debug_assertions) {
        let s1 = file_to_String_noraw(path);
        #[allow(unused_must_use)]
        #[allow(clippy::match_single_binding)]
        match print_colored_stdout(Color::Yellow, s1.as_bytes()) { _ => {}, };
        println!();
    }

    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = match SyslineReader::new(path, blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({}, {}) failed {}", path, blocksz, err);
        }
    };
    eprintln!("{}{:?}", so(), slr);

    let mut fo1: FileOffset = 0;
    let filesz = slr.filesz();
    while fo1 < filesz {
        eprintln!("{}slr.find_sysline_at_datetime_filter({}, {:?})", so(), fo1, filter_dt_after_opt);
        let result = slr.find_sysline_at_datetime_filter(fo1, filter_dt_after_opt);
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                eprintln!(
                    "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Found({}, @{:p})",
                    so(),
                    fo1,
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                    fo,
                    &*slp
                );
                eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print!("FileOffset {:3} {:?} '", fo1, filter_dt_after_opt);
                let snippet = slr
                    .linereader
                    .blockreader
                    ._vec_from(fo1, std::cmp::min(fo1 + 40, filesz));
                #[allow(clippy::match_single_binding)]
                match print_colored_stdout(Color::Yellow, buffer_to_String_noraw(snippet.as_slice()).as_bytes())
                     { _ => {}, };
                print!("' ");
                //print_slp(&slp);
                let slices = (*slp).get_slices();
                for slice in slices.iter() {
                    #[allow(clippy::match_single_binding)]
                    match print_colored_stdout(Color::Green, slice) { _ => {}, };
                }
                println!();
            }
            ResultS4_SyslineFind::Done => {
                eprintln!(
                    "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Done",
                    so(),
                    fo1,
                    filter_dt_after_opt,
                    filter_dt_before_opt
                );
            }
            ResultS4_SyslineFind::Err(err) => {
                eprintln!(
                    "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Err({})",
                    so(),
                    fo1,
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                    err
                );
                panic!(
                    "ERROR: find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Err({})",
                    fo1,
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                    err,
                );
            }
        }
        fo1 += 1;
        eprintln!("\n");
    }

    eprintln!("{}Found {} Lines, {} Syslines", so(), slr.linereader.count(), slr.syslines.len());
    eprintln!(
        "{}test_SyslineReader_w_filtering_1({:?}, {}, {:?}, {:?})",
        sx(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
}

// TODO: add test cases for `test_SyslineReader_w_filtering_1`

/// print the filtered syslines for a SyslineReader
/// quick debug helper
#[allow(dead_code)]
#[cfg(any(debug_assertions,test))]
fn process_SyslineReader(
    slr: &mut SyslineReader, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    eprintln!("{}process_SyslineReader({:?}, {:?}, {:?})", sn(), slr, filter_dt_after_opt, filter_dt_before_opt,);
    let mut fo1: FileOffset = 0;
    let mut search_more = true;
    eprintln!("{}slr.find_sysline_at_datetime_filter({}, {:?})", so(), fo1, filter_dt_after_opt);
    let result = slr.find_sysline_at_datetime_filter(fo1, filter_dt_after_opt);
    match result {
        ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
            eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Found|Found_EOF({}, @{:p})",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt,
                fo,
                &*slp
            );
            eprintln!(
                "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                so(),
                fo,
                &(*slp),
                slp.lines.len(),
                (*slp).len(),
                (*slp).to_String_noraw(),
            );
            fo1 = fo;
            print_slp(&slp);
        }
        ResultS4_SyslineFind::Done => {
            eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Done",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt
            );
            search_more = false;
        }
        ResultS4_SyslineFind::Err(err) => {
            eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Err({})",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt,
                err
            );
            panic!("ERROR: {}", err);
            search_more = false;
        }
    }
    if !search_more {
        eprintln!("{}! search_more", so());
        eprintln!("{}process_SyslineReader(…)", sx());
        return;
    }
    let mut fo2: FileOffset = fo1;
    loop {
        let result = slr.find_sysline(fo2);
        let eof = result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                if eof {
                    eprintln!("{}slr.find_sysline({}) returned Found_EOF({}, @{:p})", so(), fo2, fo, &*slp);
                } else {
                    eprintln!("{}slr.find_sysline({}) returned Found({}, @{:p})", so(), fo2, fo, &*slp);
                }
                fo2 = fo;
                eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                eprintln!(
                    "{}sysline_pass_filters({:?}, {:?}, {:?})",
                    so(),
                    (*slp).dt,
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                );
                match SyslineReader::sysline_pass_filters(&slp, filter_dt_after_opt, filter_dt_before_opt) {
                    Result_Filter_DateTime2::OccursBeforeRange | Result_Filter_DateTime2::OccursAfterRange => {
                        eprintln!(
                            "{}sysline_pass_filters returned not Result_Filter_DateTime2::OccursInRange; continue!",
                            so()
                        );
                        continue;
                    }
                    Result_Filter_DateTime2::OccursInRange => {
                        print_slp(&slp);
                        if eof {
                            assert!(slr.is_sysline_last(&slp), "returned Found_EOF yet this Sysline is not last!?");
                        } else {
                            assert!(!slr.is_sysline_last(&slp), "returned Found yet this Sysline is last!? Should have returned Found_EOF or this Sysline is really not last.");
                        }
                    }
                }
            }
            ResultS4_SyslineFind::Done => {
                eprintln!("{}slr.find_sysline({}) returned Done", so(), fo2);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                eprintln!("{}slr.find_sysline({}) returned Err({})", so(), fo2, err);
                panic!("ERROR: {}", err);
                break;
            }
        }
    }
    eprintln!("{}process_SyslineReader({:?}, …)", sx(), slr.path());
}

/// quick debug helper
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_SyslineReader_process_file<'a>(
    path: &'a FPath, blocksz: BlockSz, filter_dt_after_opt: &'a DateTimeL_Opt, filter_dt_before_opt: &'a DateTimeL_Opt,
) -> Option<Box<SyslineReader<'a>>> {
    eprintln!(
        "{}process_file({:?}, {}, {:?}, {:?})",
        sn(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
    let tzo8 = FixedOffset::west(3600 * 8);
    let slr = match SyslineReader::new(path, blocksz, tzo8) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({}, {}) failed {}", path, blocksz, err);
        }
    };
    eprintln!("{}{:?}", so(), slr);
    eprintln!("{}process_file(…)", sx());

    Some(Box::new(slr))
}

/// basic test of SyslineReader things
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_w_filtering_2(
    path: &FPath, blocksz: BlockSz, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    eprintln!(
        "{}test_SyslineReader_w_filtering_2({:?}, {}, {:?}, {:?})",
        sn(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
    let slr_opt = _test_SyslineReader_process_file(path, blocksz, filter_dt_after_opt, filter_dt_before_opt);
    if slr_opt.is_some() {
        let slr = &slr_opt.unwrap();
        eprintln!("{}Found {} Lines, {} Syslines", so(), slr.linereader.count(), slr.syslines.len());
    }
    eprintln!("{}test_SyslineReader_w_filtering_2(…)", sx());
}

// TODO: add test cases for test_SyslineReader_w_filtering_2

/// basic test of SyslineReader things
/// process multiple files
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_w_filtering_3(
    paths: &Vec<String>, blocksz: BlockSz, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    eprintln!(
        "{}test_SyslineReader_w_filtering_3({:?}, {}, {:?}, {:?})",
        sn(),
        &paths,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );

    let mut slrs = Vec::<SyslineReader>::with_capacity(paths.len());
    for path in paths.iter() {
        let tzo8 = FixedOffset::west(3600 * 8);
        eprintln!("{}SyslineReader::new({:?}, {}, {:?})", so(), path, blocksz, tzo8);
        let slr = match SyslineReader::new(path, blocksz, tzo8) {
            Ok(val) => val,
            Err(err) => {
                panic!("ERROR: SyslineReader::new({:?}, {}) failed {}", path, blocksz, err);
            }
        };
        eprintln!("{}{:?}", so(), slr);
        slrs.push(slr)
    }
    for slr in slrs.iter_mut() {
        process_SyslineReader(slr, filter_dt_after_opt, filter_dt_before_opt);
        println!();
    }
    eprintln!("{}test_SyslineReader_w_filtering_3(…)", sx());
}

// TODO: add test cases for `test_SyslineReader_w_filtering_3`

/// basic test of SyslineReader things
/// read all file offsets but randomly
/// TODO: this test was hastily designed for human review. Redesign it for automatic review.
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_SyslineReader_rand(path_: &FPath, blocksz: BlockSz) {
    eprintln!("{}test_SyslineReader_rand({:?}, {})", sn(), &path_, blocksz);
    let tzo8 = FixedOffset::west(3600 * 8);
    let mut slr1 = match SyslineReader::new(path_, blocksz, tzo8) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({}, {}, ...) failed {}", path_, blocksz, err);
        }
    };
    eprintln!("{}SyslineReader {:?}", so(), slr1);
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(slr1.filesz() as usize);
    fill(&mut offsets_rand);
    eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);
    randomize(&mut offsets_rand);
    eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);

    for fo1 in offsets_rand {
        let result = slr1.find_sysline(fo1);
        #[allow(clippy::single_match)]
        match result {
            ResultS4_SyslineFind::Err(err) => {
                eprintln!("{}slr1.find_sysline({}) returned Err({})", so(), fo1, err);
                panic!("slr1.find_sysline({}) returned Err({})", fo1, err);
            }
            _ => {}
        }
    }
    // should print the file as-is and not be affected by random reads
    slr1.print_all(true);
    eprintln!("\n{}{:?}", so(), slr1);
    eprintln!("{}test_SyslineReader_rand(…)", sx());
}

#[test]
fn test_SyslineReader_rand__zero__2() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/zero.log"), 2);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1__2() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1.log"), 2);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1.log"), 4);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1__8() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1.log"), 8);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1_Win__2() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1_Win.log"), 2);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1_Win__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1_Win.log"), 4);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1_Win__8() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1_Win.log"), 8);
}

#[test]
fn test_SyslineReader_rand__test0_nlx2__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx2.log"), 4);
}

#[test]
fn test_SyslineReader_rand__basic_dt1__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/basic-dt1.log"), 4);
}

#[test]
fn test_SyslineReader_rand__dtf5_6c__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/dtf5-6c.log"), 4);
}

#[test]
fn test_SyslineReader_rand__dtf5_6c__8() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/dtf5-6c.log"), 8);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_str_datetime() {
    let hour = 3600;
    let tzo8 = FixedOffset::west(3600 * 8);
    let tzo5 = FixedOffset::east(3600 * 5);

    // good without timezone
    let dts1 = "2000-01-01 00:00:01";
    let p1 = "%Y-%m-%d %H:%M:%S";
    let dt1 = str_datetime(dts1, p1, false, &tzo8).unwrap();
    let answer1 = Local.ymd(2000, 1, 1).and_hms(0, 0, 1);
    assert_eq!(dt1, answer1);

    // good without timezone
    let dts1 = "2000-01-01 00:00:01";
    let p1 = "%Y-%m-%d %H:%M:%S";
    let dt1 = str_datetime(dts1, p1, false, &tzo5).unwrap();
    let answer1 = tzo5.ymd(2000, 1, 1).and_hms(0, 0, 1);
    assert_eq!(dt1, answer1);

    // good with timezone
    let dts2 = "2000-01-01 00:00:02 -0100";
    let p2 = "%Y-%m-%d %H:%M:%S %z";
    let dt2 = str_datetime(dts2, p2, true, &tzo8).unwrap();
    let answer2 = FixedOffset::west(1 * hour).ymd(2000, 1, 1).and_hms(0, 0, 2);
    assert_eq!(dt2, answer2);

    // bad with timezone
    let dts3 = "2000-01-01 00:00:03 BADD";
    let p3 = "%Y-%m-%d %H:%M:%S %z";
    let dt3 = str_datetime(dts3, p3, true, &tzo8);
    assert_eq!(dt3, None);

    // bad without timezone
    let dts4 = "2000-01-01 00:00:XX";
    let p4 = "%Y-%m-%d %H:%M:%S";
    let dt4 = str_datetime(dts4, p4, false, &tzo8);
    assert_eq!(dt4, None);
}

/// given the vector of `DateTimeL`, return the vector index and value of the soonest
/// (minimum) value within a `Some`
/// If the vector is empty then return `None`
#[cfg(test)]
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
    eprintln!("{}test_datetime_soonest2()", sn());
    let vec0 = Vec::<DateTimeL>::with_capacity(0);
    let val = datetime_soonest2(&vec0);
    assert!(val.is_none());
    let tzo = FixedOffset::west(3600 * 8);

    let dt1_a = str_datetime("2001-01-01T12:00:00", "%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
    let vec1: Vec<DateTimeL> = vec![dt1_a];
    let (i_, dt_) = match datetime_soonest2(&vec1) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None1a");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_a);

    let dt1_b = str_datetime("2001-01-01T12:00:00-0100", "%Y-%m-%dT%H:%M:%S%z", true, &tzo).unwrap();
    let vec1: Vec<DateTimeL> = vec![dt1_b];
    let (i_, dt_) = match datetime_soonest2(&vec1) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None1b");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_b);

    let dt2_a = str_datetime("2002-01-01T11:00:00", "%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
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

    let dt3 = str_datetime("2000-01-01T12:00:00", "%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
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

    eprintln!("{}test_datetime_soonest2()", sx());
}

