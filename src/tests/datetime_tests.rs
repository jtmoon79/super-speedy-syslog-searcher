// src/tests/datetime_tests.rs
// … ≤ ≥ ≠ ≟

//! tests for `datetime.rs` functions

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::data::datetime::{
    FixedOffset,
    DateTimeRegex_str,
    DateTimePattern_str,
    datetime_parse_from_str,
    DateTimeL,
    DateTimeLOpt,
    DTFS_Tz,
    DTFSSet,
    DateTimeParseInstr,
    DATETIME_PARSE_DATAS,
    CGN_ALL,
    CGP_YEAR,
    CGP_MONTH_ALL,
    CGP_DAY_ALL,
    CGP_HOUR,
    CGP_MINUTE,
    CGP_SECOND,
    CGP_FRACTIONAL,
    CGP_TZ_ALL,
    DTP_ALL,
    RP_LB,
    RP_RB,
    bytes_to_regex_to_datetime,
    datetime_from_str_workaround_Issue660,
    Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    dt_pass_filters,
    dt_after_or_before,
    Year,
};

use crate::printer_debug::printers::buffer_to_String_noraw;

use std::collections::HashSet;

extern crate more_asserts;
use more_asserts::{
    assert_gt,
    assert_le,
    assert_lt,
};

extern crate si_trace_print;
use si_trace_print::{
    dpfn,
    dpfx,
};
use si_trace_print::stack::stack_offset_set;

use std::str;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// does regex pattern have a year?
pub fn regex_pattern_has_year(pattern: &DateTimeRegex_str) -> bool {
    pattern.contains(CGP_YEAR)
}

/// does regex pattern have a month?
pub fn regex_pattern_has_month(pattern: &DateTimeRegex_str) -> bool {
    for pat in CGP_MONTH_ALL.iter() {
        if pattern.contains(pat) {
            return true;
        }
    }

    false
}

/// does regex pattern have a day?
pub fn regex_pattern_has_day(pattern: &DateTimeRegex_str) -> bool {
    for pat in CGP_DAY_ALL.iter() {
        if pattern.contains(pat) {
            return true;
        }
    }

    false
}

/// does regex pattern have a hour?
pub fn regex_pattern_has_hour(pattern: &DateTimeRegex_str) -> bool {
    pattern.contains(CGP_HOUR)
}

/// does regex pattern have a minute?
pub fn regex_pattern_has_minute(pattern: &DateTimeRegex_str) -> bool {
    pattern.contains(CGP_MINUTE)
}

/// does regex pattern have a second?
pub fn regex_pattern_has_second(pattern: &DateTimeRegex_str) -> bool {
    pattern.contains(CGP_SECOND)
}

/// does regex pattern have a fractional second?
pub fn regex_pattern_has_fractional(pattern: &DateTimeRegex_str) -> bool {
    pattern.contains(CGP_FRACTIONAL)
}

/// does regex pattern have a timezone?
pub fn regex_pattern_has_tz(pattern: &DateTimeRegex_str) -> bool {
    for pat in CGP_TZ_ALL.iter() {
        if pattern.contains(pat) {
            return true;
        }
    }

    false
}

// chrono strftime formats https://docs.rs/chrono/latest/chrono/format/strftime/

/// does chrono strftime pattern have a year?
pub fn dt_pattern_has_year(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%Y")
    || pattern.contains("%y")
}

/// does chrono strftime pattern have a month?
pub fn dt_pattern_has_month(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%m")
    || pattern.contains("%b")
    || pattern.contains("%B")
    // do not use "%h"
}

/// does chrono strftime pattern have a month?
pub fn dt_pattern_has_day(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%d")
    || pattern.contains("%e")
}

/// does chrono strftime pattern have a month?
pub fn dt_pattern_has_hour(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%H")
    || pattern.contains("%_H")
    || pattern.contains("%k")
    || (pattern.contains("%I") && pattern.contains("%P"))
    || (pattern.contains("%I") && pattern.contains("%p"))
    || (pattern.contains("%l") && pattern.contains("%P"))
    || (pattern.contains("%l") && pattern.contains("%p"))
}

/// does chrono strftime pattern have a month?
pub fn dt_pattern_has_minute(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%M")
}

/// does chrono strftime pattern have a second?
pub fn dt_pattern_has_second(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%S")
}

/// does chrono strftime pattern have a second?
pub fn dt_pattern_has_fractional(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%f")
}

/// does chrono strftime pattern have a timezone?
pub fn dt_pattern_has_tz(pattern: &DateTimePattern_str) -> bool {
    // %Z is not put into `pattern`
    pattern.contains("%z")
    || pattern.contains("%:z")
    || pattern.contains("%#z")
}

/// does chrono strftime pattern have the fill timezone?
pub fn dt_pattern_has_tz_fill(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%:z")
}

// ripped from https://stackoverflow.com/a/46767732/471376
fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + std::hash::Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}

#[test]
fn test_DTP_ALL() {
    stack_offset_set(Some(2));
    for dt_format in DTP_ALL.iter() {
        assert!(dt_pattern_has_year(dt_format), "built-in dt_format missing year {:?}", dt_format);
        assert!(dt_pattern_has_month(dt_format), "built-in dt_format missing month {:?}", dt_format);
        assert!(dt_pattern_has_day(dt_format), "built-in dt_format missing day {:?}", dt_format);
        assert!(dt_pattern_has_hour(dt_format), "built-in dt_format missing hour {:?}", dt_format);
        assert!(dt_pattern_has_minute(dt_format), "built-in dt_format missing minute {:?}", dt_format);
        assert!(dt_pattern_has_second(dt_format), "built-in dt_format missing second {:?}", dt_format);
        assert!(dt_pattern_has_tz(dt_format), "built-in dt_format missing timezone {:?}", dt_format);
    }
}

/// santy check of the built-in `const DATETIME_PARSE_DATAS` values
/// does each `DateTimeParseInstr` parameter agree with other parameters?
#[test]
fn test_DATETIME_PARSE_DATAS_builtin() {
    stack_offset_set(Some(2));
    for dtpd in DATETIME_PARSE_DATAS.iter() {
        // check regex_range (arbitrary minimum)
        let regpat: &DateTimeRegex_str = dtpd.regex_pattern;
        let dtfs: &DTFSSet = &dtpd.dtfs;
        assert_lt!(dtpd.range_regex.start, dtpd.range_regex.end, "bad range_regex start {} end {}; declared at line {}", dtpd.range_regex.start, dtpd.range_regex.end, dtpd._line_num);
        assert_le!(12, regpat.len(), ".regex_pattern.len() {} too short; bad built-in DateTimeParseData {:?}; declared at line {}", regpat.len(), dtpd, dtpd._line_num);
        // check dt_pattern (arbitrary minimum)
        let dtpat: &DateTimePattern_str = dtfs.pattern;
        assert_le!(8, dtpat.len(), ".dt_pattern.len too short; bad built-in dt_pattern {:?}; declared at line {}", dtpat, dtpd._line_num);

        // check year
        if dtfs.has_year() {
            assert!(regex_pattern_has_year(regpat), "regex_pattern has not year {:?} but .year is true; declared at line {}", regpat, dtpd._line_num);
            //assert!(dt_pattern_has_year(dtpat), "dt_pattern has not year {:?} but .year is true; declared at line {}", dtpat, dtpd._line_num);
        } else {
            assert!(!regex_pattern_has_year(regpat), "regex_pattern has year {:?} but .year is false; declared at line {}", regpat, dtpd._line_num);
            // year ('%Y', etc.) is added to all `dt_pattern`, for `captures_to_buffer`
            //assert!(!dt_pattern_has_year(dtpat), "dt_pattern has year {:?} but .year is false; declared at line {}", dtpat, dtpd._line_num);
        }
        assert!(dt_pattern_has_year(dtpat), "dt_pattern does not have a year {:?}; declared at line {}", dtpat, dtpd._line_num);
        // check month
        assert!(regex_pattern_has_month(regpat), "regex_pattern has not month {:?}; declared at line {}", regpat, dtpd._line_num);
        assert!(dt_pattern_has_month(dtpat), "dt_pattern does not have a month {:?}; declared at line {}", dtpat, dtpd._line_num);
        // check day
        assert!(regex_pattern_has_day(regpat), "regex_pattern has not day {:?}; declared at line {}", regpat, dtpd._line_num);
        assert!(dt_pattern_has_day(dtpat), "dt_pattern does not have a day {:?}; declared at line {}", dtpat, dtpd._line_num);
        // check hour
        assert!(regex_pattern_has_hour(regpat), "regex_pattern has not hour {:?}; declared at line {}", regpat, dtpd._line_num);
        assert!(dt_pattern_has_hour(dtpat), "dt_pattern does not have a hour {:?}; declared at line {}", dtpat, dtpd._line_num);
        // check minute
        assert!(regex_pattern_has_minute(regpat), "regex_pattern has not minute {:?}; declared at line {}", regpat, dtpd._line_num);
        assert!(dt_pattern_has_minute(dtpat), "dt_pattern does not have a minute {:?}; declared at line {}", dtpat, dtpd._line_num);
        // check second
        assert!(regex_pattern_has_second(regpat), "regex_pattern has not second {:?}; declared at line {}", regpat, dtpd._line_num);
        assert!(dt_pattern_has_second(dtpat), "dt_pattern does not have a second {:?}; declared at line {}", dtpat, dtpd._line_num);
        // check fractional (optional but must agree)
        let rp_ss = regex_pattern_has_fractional(regpat);
        let dp_ss = dt_pattern_has_fractional(dtpat);
        assert_eq!(rp_ss, dp_ss, "regex pattern fractional {}, datetime pattern fractional {} (they must agree); declared at line {}", rp_ss, dp_ss, dtpd._line_num);
        // check timezone
        if dtfs.has_tz() {
            assert!(regex_pattern_has_tz(regpat), "regex_pattern has not timezone {:?} but tz is true; declared at line {}", regpat, dtpd._line_num);
            assert!(dt_pattern_has_tz(dtpat), "dt_pattern has not timezone {:?} but tz is true; declared at line {}", dtpat, dtpd._line_num);
        } else {
            assert!(!regex_pattern_has_tz(regpat), "regex_pattern has timezone {:?} but tz is false; declared at line {}", regpat, dtpd._line_num);
            // tz '%:z' is added to all `pattern` for `captures_to_buffer`
            assert!(dt_pattern_has_tz_fill(dtpat), "dt_pattern does not have fill timezone {:?}; declared at line {}", dtpat, dtpd._line_num);
            assert!(dtfs.tz == DTFS_Tz::_fill, "has_tz() so expected tz {:?} found {:?}; declared at line {}", dtfs.tz, DTFS_Tz::_fill, dtpd._line_num);
        }
        assert!(dt_pattern_has_tz(dtpat), "dt_pattern does not have a timezone {:?}; declared at line {}", dtpat, dtpd._line_num);
        // check test data
        assert_gt!(dtpd._test_cases.len(), 0, "No test data for dtpd declared at line {}", dtpd._line_num);
        for test_case in dtpd._test_cases {
            assert_lt!(test_case.0, test_case.1, "Bad test_case indexes {} {} for dtpd declared at line {}", test_case.0, test_case.1, dtpd._line_num);
        }
        // check cgn_first
        assert!(regpat.contains(dtpd.cgn_first), "cgn_first {:?} but not contained in regex {:?}; declared at line {}", dtpd.cgn_first, regpat, dtpd._line_num);
        assert!(CGN_ALL.iter().any(|x| x == &dtpd.cgn_first), "cgn_first {:?} not in CGN_ALL {:?}; declared at line {}", dtpd.cgn_first, &CGN_ALL, dtpd._line_num);
        let mut cgn_first_i: usize = usize::MAX;
        let mut cgn_first_s: &str = "";
        for cgn in CGN_ALL.iter() {
            let mut cgn_full = String::from('<');
            cgn_full.push_str(cgn);
            cgn_full.push('>');
            match regpat.find(cgn_full.as_str()) {
                Some(i) => {
                    if i < cgn_first_i {
                        cgn_first_s = cgn;
                        cgn_first_i = i;
                    }
                    eprintln!();
                }
                None => {},
            }
        }
        assert_eq!(cgn_first_s, dtpd.cgn_first, "cgn_first is {:?}, but analysis of the regexp found the first capture named group {:?}; declared at line {}", dtpd.cgn_first, cgn_first_s, dtpd._line_num);
        // check cgn_last
        assert!(regpat.contains(dtpd.cgn_last), "cgn_last {:?} but not contained in regex {:?}; declared at line {}", dtpd.cgn_last, regpat, dtpd._line_num);
        assert!(CGN_ALL.iter().any(|x| x == &dtpd.cgn_last), "cgn_last {:?} not in CGN_ALL {:?}; declared at line {}", dtpd.cgn_last, &CGN_ALL, dtpd._line_num);
        let mut cgn_last_i: usize = 0;
        let mut cgn_last_s: &str = "";
        for cgn in CGN_ALL.iter() {
            let mut cgn_full = String::from('<');
            cgn_full.push_str(cgn);
            cgn_full.push('>');
            match regpat.find(cgn_full.as_str()) {
                Some(i) => {
                    if i > cgn_last_i {
                        cgn_last_s = cgn;
                        cgn_last_i = i;
                    }
                }
                None => {},
            }
        }
        assert_eq!(cgn_last_s, dtpd.cgn_last, "cgn_last is {:?}, but analysis of the regexp found the last capture named group {:?}; declared at line {}", dtpd.cgn_last, cgn_last_s, dtpd._line_num);
        // check left-brackets and right-brackets are equally present and on correct sides
        match regpat.find(RP_LB) {
            Some(lb_i) => {
                let rb_i = match regpat.find(RP_RB) {
                    Some(i) => i,
                    None => {
                        panic!("regex pattern has RP_LB at {} but no RP_RB found; declared at line {}", lb_i, dtpd._line_num);
                    }
                };
                assert_lt!(lb_i, rb_i, "regex pattern has RP_LB (left bracket) at {}, RP_RB (right bracket) at {}; declared at line {}", lb_i, rb_i, dtpd._line_num);
            }
            None => {},
        }
        match regpat.find(RP_RB) {
            Some(_) => {
                match regpat.find(RP_LB) {
                    Some(_) => {},
                    None => {
                        panic!("regex pattern has RP_RB (right bracket) no RP_LB (left bracket) found; declared at line {}", dtpd._line_num);
                    }
                }
            }
            None => {},
        }
    }
    // check for duplicates
    let mut check: Vec<DateTimeParseInstr> = Vec::<DateTimeParseInstr>::from(DATETIME_PARSE_DATAS);
    let check_orig: Vec<DateTimeParseInstr> = Vec::<DateTimeParseInstr>::from(DATETIME_PARSE_DATAS);
    check.sort_unstable();
    check.dedup();
    if check.len() != DATETIME_PARSE_DATAS.len() {
        for (i, (co, cd)) in check_orig.iter().zip(check.iter()).enumerate() {
            eprintln!("entry {} {:?} {:?}", i, co, cd);
        }
        for (co, cd) in check_orig.iter().zip(check.iter()) {
            assert_eq!(co, cd, "entry {:?} appears to be a duplicate", co);
        }
    };
    assert_eq!(check.len(), DATETIME_PARSE_DATAS.len(), "the deduplicated DATETIME_PARSE_DATAS_VEC is different len than original; there are duplicates in DATETIME_PARSE_DATAS_VEC but the test could not determine which entry.");
    // another check for duplicates
    assert!(has_unique_elements(check), "DATETIME_PARSE_DATAS has repeat element(s); this should have been caught");
}

#[test]
fn test_DATETIME_PARSE_DATAS_test_cases() {
    stack_offset_set(Some(2));
    for (index, dtpd) in DATETIME_PARSE_DATAS.iter().enumerate() {
        eprintln!("Testing dtpd declared at line {} …", dtpd._line_num);
        eprintln!("  Regex Pattern   : {:?}", dtpd.regex_pattern);
        eprintln!("  DateTime Pattern: {:?}", dtpd.dtfs.pattern);
        for test_case in dtpd._test_cases {
            eprintln!("  Test Data       : {:?}", test_case);
            let dta = test_case.0;
            let dtb = test_case.1;
            let data = test_case.2.as_bytes();
            let tz = FixedOffset::east_opt(60 * 60).unwrap();
            let mut year_opt: Option<Year> = None;
            if ! dtpd.dtfs.has_year() {
                year_opt = Some(1980);
            }
            let s = buffer_to_String_noraw(data);
            match bytes_to_regex_to_datetime(data, &index, &year_opt,&tz) {
                Some(capdata) => {
                    eprintln!("Passed dtpd declared at line {} result {:?}, test data {:?}", dtpd._line_num, capdata, s);
                    let a = capdata.0;
                    let b = capdata.1;
                    assert_eq!(dta, a, "Expected datetime begin index {}, got {}, for dtpd at line {} with test data {:?}", dta, a, dtpd._line_num, s);
                    assert_eq!(dtb, b, "Expected datetime end index {}, got {}, for dtpd at line {} with test data {:?}", dtb, b, dtpd._line_num, s);
                },
                None => {
                    panic!("Failed dtpd declared at line {}\ntest data {:?}\nregex \"{}\"", dtpd._line_num, s, dtpd.regex_pattern);
                }
            }
        }
    }
}




//#[test]
#[allow(dead_code)]
/// Check that the built-in test data is caught by the same DTPD in which it is
/// declared.
fn _test_DATETIME_PARSE_DATAS_test_cases_indexing() {
    stack_offset_set(Some(2));
    let tz = FixedOffset::east_opt(60 * 60).unwrap();
    for (index, dtpd) in DATETIME_PARSE_DATAS.iter().enumerate() {
        eprintln!("Testing dtpd declared at line {} …", dtpd._line_num);
        eprintln!("  Regex Pattern   : {:?}", dtpd.regex_pattern);
        eprintln!("  DateTime Pattern: {:?}", dtpd.dtfs.pattern);
        for test_case in dtpd._test_cases {
            eprintln!("  Test Data       : {:?}", test_case);
            let data = test_case.2.as_bytes();
            let mut year_opt: Option<Year> = None;
            if ! dtpd.dtfs.has_year() {
                year_opt = Some(1980);
            }
            for (index_, dtpd_) in DATETIME_PARSE_DATAS.iter().enumerate() {
                if index_ > index {
                    break;
                }
            }
        }
    }
}

#[test]
fn test_datetime_from_str_workaround_Issue660() {
    assert!(datetime_from_str_workaround_Issue660("", ""));
    assert!(datetime_from_str_workaround_Issue660("a", ""));
    assert!(datetime_from_str_workaround_Issue660("", "a"));
    assert!(!datetime_from_str_workaround_Issue660(" ", ""));
    assert!(!datetime_from_str_workaround_Issue660("", " "));
    assert!(datetime_from_str_workaround_Issue660(" ", " "));
    assert!(datetime_from_str_workaround_Issue660(" a", " a"));
    assert!(!datetime_from_str_workaround_Issue660(" a", "  a"));
    assert!(!datetime_from_str_workaround_Issue660("  a", " a"));
    assert!(!datetime_from_str_workaround_Issue660("  a", "   a"));
    assert!(!datetime_from_str_workaround_Issue660("a", "   a"));
    assert!(!datetime_from_str_workaround_Issue660("  a", "a"));
    assert!(datetime_from_str_workaround_Issue660("a ", "a "));
    assert!(datetime_from_str_workaround_Issue660("a  ", "a  "));
    assert!(datetime_from_str_workaround_Issue660(" a ", " a "));
    assert!(datetime_from_str_workaround_Issue660(" a  ", " a  "));
    assert!(datetime_from_str_workaround_Issue660("   a  ", "   a  "));
    assert!(datetime_from_str_workaround_Issue660("   a  ", "   a  "));
    assert!(!datetime_from_str_workaround_Issue660("   a  ", "   a   "));
    assert!(!datetime_from_str_workaround_Issue660("   a   ", "   a  "));
    assert!(!datetime_from_str_workaround_Issue660("   a   ", " a  "));
    assert!(!datetime_from_str_workaround_Issue660("a   ", " a  "));

    assert!(!datetime_from_str_workaround_Issue660(" \t", " "));
    assert!(!datetime_from_str_workaround_Issue660(" ", "\t "));
    assert!(datetime_from_str_workaround_Issue660(" \t", "\t "));
    assert!(!datetime_from_str_workaround_Issue660("\t ", "\t a\t"));

    assert!(datetime_from_str_workaround_Issue660(" \n\t", " \n\t"));
    assert!(datetime_from_str_workaround_Issue660(" \n\t", " \t\n"));
    assert!(datetime_from_str_workaround_Issue660(" \n\ta", " \t\n"));
    assert!(datetime_from_str_workaround_Issue660(" \n\t", " \t\na"));
    assert!(datetime_from_str_workaround_Issue660(" \n", " \n"));
    assert!(datetime_from_str_workaround_Issue660(" \n", "\n "));
    assert!(datetime_from_str_workaround_Issue660(" \n", "\r "));
    assert!(datetime_from_str_workaround_Issue660(" \n", " \n"));
    assert!(!datetime_from_str_workaround_Issue660("\t a", "\t a\t\n"));
    assert!(!datetime_from_str_workaround_Issue660("\t\n a\n", "\t\n a\t\n"));
}

/// FixedOffset to FixedOffset==0 (UTC)
/// testing helper
fn fo_to_fo0(dt_opt: &DateTimeLOpt) -> DateTimeLOpt {
    #[allow(clippy::manual_map)]
    match dt_opt {
        Some(dt) => { Some(dt.with_timezone(&FixedOffset::east(0))) },
        None => None,
    }
}

/// basic test of `SyslineReader.sysline_pass_filters`
#[rustfmt::skip]
#[allow(non_snake_case)]
#[test]
fn test_dt_pass_filters_fixedoffset2() {
    dpfn!();

    fn DTL(s: &str) -> DateTimeL {
        let tzo = FixedOffset::west(3600 * 2);
        datetime_parse_from_str(s, "%Y%m%dT%H%M%S", false, &tzo).unwrap()
    }

    for (da, dt, db, exp_result) in [
        (
            Some(DTL("20000101T010105")),
            DTL("20000101T010106"),
            Some(DTL("20000101T010107")),
            Result_Filter_DateTime2::InRange,
        ),
        (
            Some(DTL("20000101T010107")),
            DTL("20000101T010106"),
            Some(DTL("20000101T010108")),
            Result_Filter_DateTime2::BeforeRange,
        ),
        (
            Some(DTL("20000101T010101")),
            DTL("20000101T010106"),
            Some(DTL("20000101T010102")),
            Result_Filter_DateTime2::AfterRange,
        ),
        (
            Some(DTL("20000101T010101")),
            DTL("20000101T010106"),
            None,
            Result_Filter_DateTime2::InRange
        ),
        (
            Some(DTL("20000101T010102")),
            DTL("20000101T010101"),
            None,
            Result_Filter_DateTime2::BeforeRange,
        ),
        (
            Some(DTL("20000101T010101")),
            DTL("20000101T010101"),
            None,
            Result_Filter_DateTime2::InRange
        ),
        (
            None,
            DTL("20000101T010101"),
            Some(DTL("20000101T010106")),
            Result_Filter_DateTime2::InRange
        ),
        (
            None,
            DTL("20000101T010101"),
            Some(DTL("20000101T010100")),
            Result_Filter_DateTime2::AfterRange,
        ),
        (
            None,
            DTL("20000101T010101"),
            Some(DTL("20000101T010101")),
            Result_Filter_DateTime2::InRange
        ),
    ] {
        let result = dt_pass_filters(&dt, &da, &db);
        assert_eq!(exp_result, result, "Expected {:?} Got {:?} for {:?} among dt_pass_filters({:?}, {:?})", exp_result, result, dt, da, db);
        eprintln!("dt_pass_filters(\n\t{:?},\n\t{:?},\n\t{:?}\n)\nreturned expected {:?}", dt, da, db, result);
    }
    dpfx!();
}

#[rustfmt::skip]
#[allow(non_snake_case)]
#[test]
fn test_dt_pass_filters_z() {
    dpfn!();

    fn DTLz(s: &str) -> DateTimeL {
        let tz_dummy = FixedOffset::east(0);
        datetime_parse_from_str(s, "%Y%m%dT%H%M%S%z", true, &tz_dummy).unwrap()
    }

    for (da, dt, db, exp_result) in [
        (   // same TZ
            Some(DTLz("20000101T010105-0100")),
            DTLz("20000101T010106-0100"),
            Some(DTLz("20000101T010107-0100")),
            Result_Filter_DateTime2::InRange,
        ),
        (   // differing TZ
            Some(DTLz("20000101T020115+0200")),
            DTLz("20000101T010116+0100"),
            Some(DTLz("20000101T030117+0300")),
            Result_Filter_DateTime2::InRange,
        ),
        (   // same TZ
            Some(DTLz("20000101T010107-0200")),
            DTLz("20000101T010106-0200"),
            Some(DTLz("20000101T010108-0200")),
            Result_Filter_DateTime2::BeforeRange,
        ),
        (   // differing TZ
            Some(DTLz("20000101T010117+0100")),
            DTLz("20000101T020116+0200"),
            Some(DTLz("20000101T030118+0300")),
            Result_Filter_DateTime2::BeforeRange,
        ),
        (   // same TZ
            Some(DTLz("20000101T010101-0300")),
            DTLz("20000101T010106-0300"),
            Some(DTLz("20000101T010102-0300")),
            Result_Filter_DateTime2::AfterRange,
        ),
        (   // same TZ
            Some(DTLz("20000101T010101-0400")),
            DTLz("20000101T010106-0400"),
            None,
            Result_Filter_DateTime2::InRange
        ),
        (   // differing TZ
            Some(DTLz("20000101T030101+0300")),
            DTLz("20000101T010106-0100"),
            None,
            Result_Filter_DateTime2::InRange
        ),
        (   // same TZ
            Some(DTLz("20000101T010102-0500")),
            DTLz("20000101T010101-0500"),
            None,
            Result_Filter_DateTime2::BeforeRange,
        ),
        (   // differing TZ
            Some(DTLz("20000101T113102+0900")),
            DTLz("20000101T011101-0000"),
            None,
            Result_Filter_DateTime2::BeforeRange,
        ),
        (   // same TZ
            Some(DTLz("20000101T010101-0600")),
            DTLz("20000101T010101-0600"),
            None,
            Result_Filter_DateTime2::InRange
        ),
        (   // same TZ
            None,
            DTLz("20000101T010101-0700"),
            Some(DTLz("20000101T010106-0700")),
            Result_Filter_DateTime2::InRange
        ),
        (   // same TZ
            None,
            DTLz("20000101T010101-0800"),
            Some(DTLz("20000101T010100-0800")),
            Result_Filter_DateTime2::AfterRange,
        ),
        (   // same TZ
            None,
            DTLz("20000101T010101-0900"),
            Some(DTLz("20000101T010101-0900")),
            Result_Filter_DateTime2::InRange
        ),
    ] {
        let result = dt_pass_filters(&dt, &da, &db);
        // assert error message includes UTC datetimes for easier grok
        let dt0 = fo_to_fo0(&Some(dt)).unwrap();
        let da0 = fo_to_fo0(&da);
        let db0 = fo_to_fo0(&db);
        assert_eq!(exp_result, result, "
Expected {:?}
Got      {:?}
For                  {:?}
dt_pass_filters({:?}, {:?})
For                  {:?}
dt_pass_filters({:?}, {:?})
", exp_result, result, dt, da, db, dt0, da0, db0);
        eprintln!("dt_pass_filters(\n\t{:?},\n\t{:?},\n\t{:?}\n)\nreturned expected {:?}", dt, da, db, result);
    }
    dpfx!();
}

/// basic test of `SyslineReader.dt_after_or_before`
/// TODO: add tests with TZ
#[allow(non_snake_case)]
#[test]
fn test_dt_after_or_before() {
    dpfn!();

    fn DTL(s: &str) -> DateTimeL {
        let tzo = FixedOffset::west(3600 * 8);
        datetime_parse_from_str(s, "%Y%m%dT%H%M%S", false, &tzo).unwrap()
    }

    for (dt, da, exp_result) in [
        (DTL("20000101T010106"), None, Result_Filter_DateTime1::Pass),
        (DTL("20000101T010101"), Some(DTL("20000101T010103")), Result_Filter_DateTime1::OccursBefore),
        (DTL("20000101T010100"), Some(DTL("20000101T010100")), Result_Filter_DateTime1::OccursAtOrAfter),
        (DTL("20000101T010109"), Some(DTL("20000101T010108")), Result_Filter_DateTime1::OccursAtOrAfter),
    ] {
        let result = dt_after_or_before(&dt, &da);
        assert_eq!(exp_result, result, "Expected {:?} Got {:?} for ({:?}, {:?})", exp_result, result, dt, da);
        eprintln!("dt_after_or_before(\n\t{:?},\n\t{:?}\n)\nreturned expected {:?}", dt, da, result);
    }
    dpfx!();
}
