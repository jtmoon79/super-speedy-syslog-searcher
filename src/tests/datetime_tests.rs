// src/tests/datetime_tests.rs
// … ≤ ≥ ≠ ≟

//! tests for `datetime.rs` functions

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::tests::common::{TZO_0, TZO_E1, TZO_W8};

use crate::data::datetime::{
    bytes_to_regex_to_datetime, datetime_from_str_workaround_Issue660, datetime_parse_from_str,
    dt_after_or_before, dt_pass_filters, DTFSSet, DTFS_Tz,
    DateTimeL, DateTimeLOpt, Duration, FixedOffset,
    DateTimeParseInstr, DateTimePattern_str, DateTimeRegex_str,
    Result_Filter_DateTime1, Result_Filter_DateTime2, TimeZone, Year,
    DATETIME_PARSE_DATAS_LEN, DATETIME_PARSE_DATAS,
    CGP_HOUR, CGP_MINUTE, CGP_SECOND, CGP_FRACTIONAL, CGP_FRACTIONAL3,
    CGP_MONTH_ALL, CGN_ALL, CGP_DAY_ALL, CGP_YEAR, CGP_YEARy,
    CGP_TZZ, CGP_TZ_ALL,
    TZZ_LIST_LOWER, TZZ_LIST_UPPER, TZZ_LOWER_TO_UPPER, MAP_TZZ_TO_TZz,
    RP_LB, RP_RB,
    DTP_ALL,
};

use crate::debug::printers::buffer_to_String_noraw;

use bstr::ByteSlice;

extern crate chrono;
#[allow(unused_imports)]
use chrono::{Datelike, Timelike}; // for `with_nanosecond()` and others

extern crate lazy_static;
use lazy_static::lazy_static;

use std::collections::HashSet;

extern crate more_asserts;
use more_asserts::{assert_gt, assert_le, assert_lt};

extern crate regex;
extern crate si_trace_print;
use si_trace_print::stack::stack_offset_set;
use si_trace_print::{dpfn, dpfx};

use std::str;

extern crate test_case;
use test_case::test_case;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// wrapper for chrono DateTime creation function
pub fn ymdhms(
    fixedoffset: &FixedOffset,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32
) -> DateTimeL {
    fixedoffset.with_ymd_and_hms(
        year,
        month,
        day,
        hour,
        min,
        sec,
    ).unwrap()
}

/// wrapper for chrono DateTime creation function
pub fn ymdhmsn(
    fixedoffset: &FixedOffset,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
    nano: i64
) -> DateTimeL {
    fixedoffset
    .with_ymd_and_hms(
        year,
        month,
        day,
        hour,
        min,
        sec
    )
    .unwrap()
    + Duration::nanoseconds(nano)
}

/// wrapper for chrono DateTime creation function
pub fn ymdhmsm(
    fixedoffset: &FixedOffset,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
    micro: i64
) -> DateTimeL {
    fixedoffset.with_ymd_and_hms(
        year,
        month,
        day,
        hour,
        min,
        sec
    )
    .unwrap()
    + Duration::microseconds(micro)
}

/// does regex pattern have a year?
pub fn regex_pattern_has_year(pattern: &DateTimeRegex_str) -> bool {
    pattern.contains(CGP_YEAR) || pattern.contains(CGP_YEARy)
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
    pattern.contains(CGP_FRACTIONAL) || pattern.contains(CGP_FRACTIONAL3)
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
    pattern.contains("%Y") || pattern.contains("%y")
}

/// does chrono strftime pattern have a month?
pub fn dt_pattern_has_month(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%m") || pattern.contains("%b") || pattern.contains("%B")
    // do not use "%h"
}

/// does chrono strftime pattern have a month?
pub fn dt_pattern_has_day(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%d") || pattern.contains("%e")
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
    pattern.contains("%z") || pattern.contains("%:z") || pattern.contains("%#z")
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
    iter.into_iter()
        .all(move |x| uniq.insert(x))
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
    // check for duplicates
    let mut check: Vec<DateTimeParseInstr> = Vec::<DateTimeParseInstr>::from(DATETIME_PARSE_DATAS);
    let check_orig: Vec<DateTimeParseInstr> = Vec::<DateTimeParseInstr>::from(DATETIME_PARSE_DATAS);
    check.sort_unstable();
    check.dedup();
    if check.len() != DATETIME_PARSE_DATAS.len() {
        for (i, (co, cd)) in check_orig
            .iter()
            .zip(check.iter())
            .enumerate()
        {
            eprintln!("entry {} {:?} {:?}", i, co, cd);
        }
        for (co, cd) in check_orig
            .iter()
            .zip(check.iter())
        {
            assert_eq!(co, cd, "entry {:?} appears to be a duplicate", co);
        }
    };
    assert_eq!(check.len(), DATETIME_PARSE_DATAS.len(), "the deduplicated DATETIME_PARSE_DATAS_VEC is different len than original; there are duplicates in DATETIME_PARSE_DATAS_VEC but the test could not determine which entry.");
    // another check for duplicates
    assert!(
        has_unique_elements(check),
        "DATETIME_PARSE_DATAS has repeat element(s); this should have been caught"
    );
}

/// a crude way to help the developer not forget about updating the
/// hardcoded generated test cases in the proceeeding test function
/// `test_DATETIME_PARSE_DATAS_test_cases`
#[test]
fn test_DATETIME_PARSE_DATAS_test_cases_has_all_test_cases() {
    assert_eq!(
        // THIS NUMBER SHOULD MATCH `DATETIME_PARSE_DATAS_LEN`
        //
        // IF YOU CHANGE THIS NUMBER THEN ALSO UPDATE THE GENERATED TEST CASES
        // FOR `test_DATETIME_PARSE_DATAS_test_cases` BELOW! THOSE TESTS SHOULD
        // BE FROM ZERO TO ONE LESS THAN THIS NUMBER
        94,
        DATETIME_PARSE_DATAS.len(),
        "Did you update?\n\n    #[test_case({0})]\n    fn test_DATETIME_PARSE_DATAS_test_cases()\n\nShould be one less than DATETIME_PARSE_DATAS_LEN {0}\n\n",
        DATETIME_PARSE_DATAS_LEN
    );
}

/// match the regexp built-in test cases for all entries in `DATETIME_PARSE_DATAS`
// XXX: how to generate these test_cases from 0 to DATETIME_PARSE_DATAS_LEN?
//      until that is determined, run this shell snippet from the project root directory
//
//           for i in $(seq 0 $(($(grep -m1 -Fe 'DATETIME_PARSE_DATAS_LEN:' -- ./src/data/datetime.rs | grep -Eoe '[[:digit:]]+') - 1))); do echo '#[test_case('${i}')]'; done
//
//      See feature request https://github.com/frondeus/test-case/issues/111
//
#[test_case(0)]
#[test_case(1)]
#[test_case(2)]
#[test_case(3)]
#[test_case(4)]
#[test_case(5)]
#[test_case(6)]
#[test_case(7)]
#[test_case(8)]
#[test_case(9)]
#[test_case(10)]
#[test_case(11)]
#[test_case(12)]
#[test_case(13)]
#[test_case(14)]
#[test_case(15)]
#[test_case(16)]
#[test_case(17)]
#[test_case(18)]
#[test_case(19)]
#[test_case(20)]
#[test_case(21)]
#[test_case(22)]
#[test_case(23)]
#[test_case(24)]
#[test_case(25)]
#[test_case(26)]
#[test_case(27)]
#[test_case(28)]
#[test_case(29)]
#[test_case(30)]
#[test_case(31)]
#[test_case(32)]
#[test_case(33)]
#[test_case(34)]
#[test_case(35)]
#[test_case(36)]
#[test_case(37)]
#[test_case(38)]
#[test_case(39)]
#[test_case(40)]
#[test_case(41)]
#[test_case(42)]
#[test_case(43)]
#[test_case(44)]
#[test_case(45)]
#[test_case(46)]
#[test_case(47)]
#[test_case(48)]
#[test_case(49)]
#[test_case(50)]
#[test_case(51)]
#[test_case(52)]
#[test_case(53)]
#[test_case(54)]
#[test_case(55)]
#[test_case(56)]
#[test_case(57)]
#[test_case(58)]
#[test_case(59)]
#[test_case(60)]
#[test_case(61)]
#[test_case(62)]
#[test_case(63)]
#[test_case(64)]
#[test_case(65)]
#[test_case(66)]
#[test_case(67)]
#[test_case(68)]
#[test_case(69)]
#[test_case(70)]
#[test_case(71)]
#[test_case(72)]
#[test_case(73)]
#[test_case(74)]
#[test_case(75)]
#[test_case(76)]
#[test_case(77)]
#[test_case(78)]
#[test_case(79)]
#[test_case(80)]
#[test_case(81)]
#[test_case(82)]
#[test_case(83)]
#[test_case(84)]
#[test_case(85)]
#[test_case(86)]
#[test_case(87)]
#[test_case(88)]
#[test_case(89)]
#[test_case(90)]
#[test_case(91)]
#[test_case(92)]
#[test_case(93)]
fn test_DATETIME_PARSE_DATAS_test_cases(index: usize) {
    stack_offset_set(Some(2));

    let dtpd = &DATETIME_PARSE_DATAS[index];

    eprintln!("Testing dtpd declared at line {} …", dtpd._line_num);

    //
    // cross-check as much as possible
    //
    eprintln!("Check all variable settings …");
    // check regex_range (arbitrary minimum)
    let regpat: &DateTimeRegex_str = dtpd.regex_pattern;
    let dtfs: &DTFSSet = &dtpd.dtfs;
    assert_lt!(
        dtpd.range_regex.start,
        dtpd.range_regex.end,
        "bad range_regex start {} end {}; declared at line {}",
        dtpd.range_regex.start,
        dtpd.range_regex.end,
        dtpd._line_num
    );
    assert_le!(
        12,
        regpat.len(),
        ".regex_pattern.len() {} too short; bad built-in DateTimeParseData {:?}; declared at line {}",
        regpat.len(),
        dtpd,
        dtpd._line_num
    );
    // check dt_pattern (arbitrary minimum)
    let dtpat: &DateTimePattern_str = dtfs.pattern;
    assert_le!(
        8,
        dtpat.len(),
        ".dt_pattern.len too short; bad built-in dt_pattern {:?}; declared at line {}",
        dtpat,
        dtpd._line_num
    );
    // while the pattern could intentionally start with "^" and start past 0,
    // it is most likely a user error
    if dtpat.starts_with('^') {
        assert_eq!(dtpd.range_regex.start, 0, "Pattern user beginning of line yet range starts at {:?}, expected start at 0", dtpd.range_regex.start);
    }

    // check year
    if dtfs.has_year() {
        assert!(
            regex_pattern_has_year(regpat),
            "regex_pattern has not year {:?} but .year is true; declared at line {}",
            regpat,
            dtpd._line_num
        );
        //assert!(dt_pattern_has_year(dtpat), "dt_pattern has not year {:?} but .year is true; declared at line {}", dtpat, dtpd._line_num);
    } else {
        assert!(
            !regex_pattern_has_year(regpat),
            "regex_pattern has year {:?} but .year is false; declared at line {}",
            regpat,
            dtpd._line_num
        );
        // year ('%Y', etc.) is added to all `dt_pattern`, for `captures_to_buffer`
        //assert!(!dt_pattern_has_year(dtpat), "dt_pattern has year {:?} but .year is false; declared at line {}", dtpat, dtpd._line_num);
    }
    assert!(
        dt_pattern_has_year(dtpat),
        "dt_pattern does not have a year {:?}; declared at line {}",
        dtpat,
        dtpd._line_num
    );
    // check month
    assert!(
        regex_pattern_has_month(regpat),
        "regex_pattern has not month {:?}; declared at line {}",
        regpat,
        dtpd._line_num
    );
    assert!(
        dt_pattern_has_month(dtpat),
        "dt_pattern does not have a month {:?}; declared at line {}",
        dtpat,
        dtpd._line_num
    );
    // check day
    assert!(
        regex_pattern_has_day(regpat),
        "regex_pattern has not day {:?}; declared at line {}",
        regpat,
        dtpd._line_num
    );
    assert!(
        dt_pattern_has_day(dtpat),
        "dt_pattern does not have a day {:?}; declared at line {}",
        dtpat,
        dtpd._line_num
    );
    // check hour
    assert!(
        regex_pattern_has_hour(regpat),
        "regex_pattern has not hour {:?}; declared at line {}",
        regpat,
        dtpd._line_num
    );
    assert!(
        dt_pattern_has_hour(dtpat),
        "dt_pattern does not have a hour {:?}; declared at line {}",
        dtpat,
        dtpd._line_num
    );
    // check minute
    assert!(
        regex_pattern_has_minute(regpat),
        "regex_pattern has not minute {:?}; declared at line {}",
        regpat,
        dtpd._line_num
    );
    assert!(
        dt_pattern_has_minute(dtpat),
        "dt_pattern does not have a minute {:?}; declared at line {}",
        dtpat,
        dtpd._line_num
    );
    // check second (optional but must agree)
    let rp_ss = regex_pattern_has_second(regpat);
    let dp_ss = dt_pattern_has_second(dtpat);
    assert_eq!(
        rp_ss, dp_ss,
        "regex_pattern has second {}, datetime pattern has second {}; they must agree; declared at line {}\n  regex pattern: {:?}\n  dt_pattern {:?}\n",
        rp_ss, dp_ss,
        dtpd._line_num,
        regpat,
        dtpat,
    );
    // check fractional (optional but must agree)
    let rp_ss = regex_pattern_has_fractional(regpat);
    let dp_ss = dt_pattern_has_fractional(dtpat);
    assert_eq!(
        rp_ss, dp_ss,
        "regex_pattern has fractional {}, datetime pattern has fractional {}; they must agree; declared at line {}\n  regex pattern: {:?}\n  dt_pattern {:?}\n",
        rp_ss, dp_ss,
        dtpd._line_num,
        regpat,
        dtpat,
    );
    // check timezone
    if dtfs.has_tz() {
        assert!(
            regex_pattern_has_tz(regpat),
            "regex_pattern has not timezone {:?} but tz is true; declared at line {}",
            regpat,
            dtpd._line_num
        );
        assert!(
            dt_pattern_has_tz(dtpat),
            "dt_pattern has not timezone {:?} but tz is true; declared at line {}",
            dtpat,
            dtpd._line_num
        );
    } else {
        assert!(
            !regex_pattern_has_tz(regpat),
            "regex_pattern has timezone {:?} but tz is false; declared at line {}",
            regpat,
            dtpd._line_num
        );
        // tz '%:z' is added to all `pattern` for `captures_to_buffer`
        assert!(
            dt_pattern_has_tz_fill(dtpat),
            "dt_pattern does not have fill timezone {:?}; declared at line {}",
            dtpat,
            dtpd._line_num
        );
        assert!(
            dtfs.tz == DTFS_Tz::_fill,
            "has_tz() so expected tz {:?} found {:?}; declared at line {}",
            dtfs.tz,
            DTFS_Tz::_fill,
            dtpd._line_num
        );
    }
    assert!(
        dt_pattern_has_tz(dtpat),
        "dt_pattern does not have a timezone {:?}; declared at line {}",
        dtpat,
        dtpd._line_num
    );
    // check test data
    assert_gt!(dtpd._test_cases.len(), 0, "No test data for dtpd declared at line {}", dtpd._line_num);
    for test_case_ in dtpd._test_cases {
        assert_lt!(
            test_case_.0,
            test_case_.1,
            "Bad test_case indexes {} {} for dtpd declared at line {}",
            test_case_.0,
            test_case_.1,
            dtpd._line_num
        );
    }
    // check cgn_first
    assert!(
        regpat.contains(dtpd.cgn_first),
        "cgn_first {:?} but not contained in regex {:?}; declared at line {}",
        dtpd.cgn_first,
        regpat,
        dtpd._line_num
    );
    assert!(
        CGN_ALL
            .iter()
            .any(|x| x == &dtpd.cgn_first),
        "cgn_first {:?} not in CGN_ALL {:?}; declared at line {}",
        dtpd.cgn_first,
        &CGN_ALL,
        dtpd._line_num
    );
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
            None => {}
        }
    }
    assert_eq!(cgn_first_s, dtpd.cgn_first, "cgn_first is {:?}, but analysis of the regexp found the first capture named group {:?}; declared at line {}", dtpd.cgn_first, cgn_first_s, dtpd._line_num);
    // check cgn_last
    assert!(
        regpat.contains(dtpd.cgn_last),
        "cgn_last {:?} but not contained in regex {:?}; declared at line {}",
        dtpd.cgn_last,
        regpat,
        dtpd._line_num
    );
    assert!(
        CGN_ALL
            .iter()
            .any(|x| x == &dtpd.cgn_last),
        "cgn_last {:?} not in CGN_ALL {:?}; declared at line {}",
        dtpd.cgn_last,
        &CGN_ALL,
        dtpd._line_num
    );
    let mut cgn_last_i: usize = 0;
    let mut cgn_last_s: &str = "";
    for cgn in CGN_ALL.iter() {
        let mut cgn_full = String::from('<');
        cgn_full.push_str(cgn);
        cgn_full.push('>');
        if let Some(i) = regpat.find(cgn_full.as_str()) {
            if i > cgn_last_i {
                cgn_last_s = cgn;
                cgn_last_i = i;
            }
        }
    }
    assert_eq!(cgn_last_s, dtpd.cgn_last, "cgn_last is {:?}, but analysis of the regexp found the last capture named group {:?}; declared at line {}", dtpd.cgn_last, cgn_last_s, dtpd._line_num);
    // check left-brackets and right-brackets are equally present and on correct sides
    match regpat.find(RP_LB) {
        Some(lb_i) => {
            let rb_i = match regpat.find(RP_RB) {
                Some(i) => i,
                None => {
                    panic!(
                        "regex pattern has RP_LB at {} but no RP_RB found; declared at line {}",
                        lb_i, dtpd._line_num
                    );
                }
            };
            assert_lt!(lb_i, rb_i, "regex pattern has RP_LB (left bracket) at {}, RP_RB (right bracket) at {}; declared at line {}", lb_i, rb_i, dtpd._line_num);
        }
        None => {}
    }
    match regpat.find(RP_RB) {
        Some(_) => match regpat.find(RP_LB) {
            Some(_) => {}
            None => {
                panic!("regex pattern has RP_RB (right bracket) no RP_LB (left bracket) found; declared at line {}", dtpd._line_num);
            }
        },
        None => {}
    }

    //
    // regex matching tests
    //
    eprintln!("Test Regex self-tests …");
    eprintln!("  Regex Pattern   : {:?}", dtpd.regex_pattern);
    eprintln!("  DateTime Pattern: {:?}", dtpd.dtfs.pattern);
    for test_case_ in dtpd._test_cases {
        eprintln!("  Test Data       : {:?}", test_case_);
        let dta = test_case_.0;
        let dtb = test_case_.1;
        assert_lt!(dta, dtb, "bad indexes");
        let data = test_case_.2.as_bytes();
        eprintln!("  Test Data[{:2},{:2}]: {:?}", dta, dtb, &data[dta..dtb].as_bstr());
        let tz = *TZO_E1;
        let mut year_opt: Option<Year> = None;
        if !dtpd.dtfs.has_year() {
            year_opt = Some(1980);
        }
        let s = buffer_to_String_noraw(data);
        match bytes_to_regex_to_datetime(data, &index, &year_opt, &tz) {
            Some(capdata) => {
                eprintln!(
                    "Passed dtpd declared at line {} result {:?}, test data {:?}",
                    dtpd._line_num, capdata, s
                );
                let a = capdata.0;
                let b = capdata.1;
                assert_lt!(a, b, "bad a {} b {}", a, b);
                assert_eq!(
                    (dta, dtb), (a, b),
                    "For dtpd at line {:?} unexpected index returned\n  test data {:?}\n  expect {:?} {:?}\n  actual {:?} {:?}\n",
                    dtpd._line_num, s, (dta, dtb), &s.as_str()[dta..dtb], (a, b), &s.as_str()[a..b],
                );
            }
            None => {
                panic!(
                    "Failed dtpd declared at line {}\ntest data {:?}\nregex \"{}\"",
                    dtpd._line_num, s, dtpd.regex_pattern
                );
            }
        }
    }
}

#[test]
/// check of structures containing timezone names and timezone values
fn test_Map_TZ_names() {
    let regex = regex::Regex::new(CGP_TZZ).unwrap();
    assert_eq!(TZZ_LIST_UPPER.len(), TZZ_LIST_LOWER.len(), "TZZ_LIST_UPPER len {} != {} TZZ_LIST_LOWER len", TZZ_LIST_UPPER.len(), TZZ_LIST_LOWER.len());
    for up in TZZ_LIST_UPPER {
        assert!(MAP_TZZ_TO_TZz.contains_key(up), "Named timezone {:?} not found in MAP_TZZ_TO_TZz", up);
    }
    for lo in TZZ_LIST_LOWER {
        let up = lo.to_ascii_uppercase();
        assert!(MAP_TZZ_TO_TZz.contains_key(up.as_str()), "Named timezone {:?} (lower {:?}) not found in MAP_TZZ_TO_TZz", up, lo);
    }
    // tz_name example "PST"
    // tz_val example "-07:00"
    for (tz_name, tz_val) in MAP_TZZ_TO_TZz.iter() {
        if ! tz_val.is_empty() {
            assert!(tz_val.starts_with('+') || tz_val.starts_with('-'), "Bad timezone value starts_with {:?} for entry {:?}", tz_val, tz_name);
            assert!(tz_val.ends_with(":00") || tz_val.ends_with(":30") || tz_val.ends_with(":45"), "Bad timezone value ends_with {:?} for entry {:?}", tz_val, tz_name);
            assert!(tz_val.contains(':'), "Bad timezone value {:?} not contains ':' for entry {:?}", tz_val, tz_name);
            assert_eq!(tz_val.len(), 6, "Bad timezone value {:?} length {:?} for entry {:?}", tz_val, tz_val.len(), tz_name);
        } // empty value means the name is ambiguous
        assert!(TZZ_LIST_UPPER.contains(tz_name) || TZZ_LIST_LOWER.contains(tz_name), "Named timezone {:?} not in TZZ_LIST_UPPER or TZZ_LIST_LOWER", tz_name);
    }
    for (index, tz_upper) in TZZ_LIST_UPPER.iter().enumerate() {
        let tz_lower = TZZ_LIST_LOWER[index];
        let tz_lower_to_upper = tz_lower.to_ascii_uppercase();
        assert_eq!(
            tz_upper, &tz_lower_to_upper.as_str(),
            "TZZ_LIST_UPPER[{}]={:?} != TZZ_LIST_LOWER[{}]={:?} ({:?})",
            index, tz_upper, index, tz_lower, tz_lower_to_upper,
        );
    }
    for (lo, up) in TZZ_LOWER_TO_UPPER.iter() {
        assert!(regex.is_match(lo), "Key {:?} from TZZ_LOWER_TO_UPPER not matched by regex CGP_TZZ", lo);
        assert!(regex.is_match(up), "Value {:?} from TZZ_LOWER_TO_UPPER not matched by regex CGP_TZZ", up);
    }
    for (tz_name, _tz_val) in MAP_TZZ_TO_TZz.iter() {
        assert!(regex.is_match(tz_name), "Key {:?} from MAP_TZZ_TO_TZz not matched by regex CGP_TZZ", tz_name);
        assert!(
            TZZ_LIST_UPPER.contains(tz_name) != TZZ_LIST_LOWER.contains(tz_name),
            "Key {:?} from MAP_TZZ_TO_TZz {} in TZZ_LIST_UPPER, {} in TZZ_LIST_LOWER",
            tz_name,
            if TZZ_LIST_UPPER.contains(tz_name) { "is" } else { "not" },
            if TZZ_LIST_LOWER.contains(tz_name) { "is" } else { "not" },
        );
    }
}

//#[test]
#[allow(dead_code)]
/// Check that the built-in test data is caught by the same DTPD in which it is
/// declared.
fn _test_DATETIME_PARSE_DATAS_test_cases_indexing() {
    stack_offset_set(Some(2));
    let _tz = *TZO_E1;
    for (index, dtpd) in DATETIME_PARSE_DATAS
        .iter()
        .enumerate()
    {
        eprintln!("Testing dtpd declared at line {} …", dtpd._line_num);
        eprintln!("  Regex Pattern   : {:?}", dtpd.regex_pattern);
        eprintln!("  DateTime Pattern: {:?}", dtpd.dtfs.pattern);
        for test_case in dtpd._test_cases {
            eprintln!("  Test Data       : {:?}", test_case);
            let _data = test_case.2.as_bytes();
            let mut _year_opt: Option<Year> = None;
            if !dtpd.dtfs.has_year() {
                _year_opt = Some(1980);
            }
            for (index_, _dtpd) in DATETIME_PARSE_DATAS
                .iter()
                .enumerate()
            {
                if index_ > index {
                    break;
                }
            }
        }
    }
}

lazy_static! {
    static ref FO_UTC: FixedOffset = *TZO_0;
    static ref FO_W8: FixedOffset = FixedOffset::west_opt(60 * 60 * 8).unwrap();
    static ref FO_E10: FixedOffset = FixedOffset::east_opt(60 * 60 * 10).unwrap();
}

#[test_case(
    "20000101T000000", "%Y%m%dT%H%M%S", false, &FO_UTC,
    Some(ymdhms(&FO_UTC, 2000, 1, 1, 0, 0, 0));
    "20000101T000000 %Y%m%dT%H%M%S no_tz"
)]
#[test_case(
    "20000101T000000 ", "%Y%m%dT%H%M%S", false, &FO_UTC, None;
    "20000101T000000  %Y%m%dT%H%M%S no_tz (extra space data)"
)]
#[test_case(
    "20000101T000000", "%Y%m%dT%H%M%S ", false, &FO_UTC, None;
    "20000101T000000 %Y%m%dT%H%M%S  no_tz (extra space pattern)"
)]
#[test_case(
    "20000101T000000,123", "%Y%m%dT%H%M%S,%3f", false, &FO_UTC,
    Some(ymdhmsm(&FO_UTC, 2000, 1, 1, 0, 0, 0, 123000));
    "20000101T000000,123 %Y%m%dT%H%M%S,%3f no_tz"
)]
#[test_case(
    "20000101T000000,123 -0800", "%Y%m%dT%H%M%S,%3f %:z", true, &FO_W8,
    Some(ymdhmsm(&FO_W8, 2000, 1, 1, 0, 0, 0, 123000));
    "20000101T000000,123 -0800 %Y%m%dT%H%M%S,%3f %:z has_tz"
)]
#[test_case(
    "20000101T000000,123 +1000", "%Y%m%dT%H%M%S,%3f %:z", true, &FO_E10,
    Some(ymdhmsm(&FO_E10, 2000, 1, 1, 0, 0, 0, 123000));
    "20000101T000000,123 +1000 %Y%m%dT%H%M%S,%3f %:z has_tz"
)]
#[test_case(
    "20000101T000000,123 +1000", "%Y%m%dT%H%M%S,%f %:z", true, &FO_E10,
    Some(ymdhmsn(&FO_E10, 2000, 1, 1, 0, 0, 0, 123));
    "20000101T000000,123 +1000 %Y%m%dT%H%M%S,%f %:z has_tz"
)]
#[test_case(
    "20000101T000000 +1000 ", "%Y%m%dT%H%M%S %:z", true, &FO_E10, None;
    "20000101T000000 +1000  %Y%m%dT%H%M%S %:z has_tz (extra space data)"
)]
#[test_case(
    "20000101T000000 +1000", " %Y%m%dT%H%M%S %:z", true, &FO_E10, None;
    "20000101T000000 +1000  %Y%m%dT%H%M%S %:z has_tz (extra space pattern)"
)]
#[test_case(
    "20000101T000000,123", "%Y%m%dT%H%M%S,%3f %:z", true, &FO_E10, None;
    "20000101T000000,123 %Y%m%dT%H%M%S,%3f %:z has_tz (None)"
)]
#[test_case(
    "20000101T000000,123", "%Y%m%dT%H%M%S,%3f %:z", false, &FO_E10, None;
    "20000101T000000,123 %Y%m%dT%H%M%S,%3f %:z no_tz (None)"
)]
fn test_datetime_parse_from_str(
    data: &str,
    pattern: &str,
    has_tz: bool,
    tz_offset: &FixedOffset,
    expect_dt: Option<DateTimeL>,
) {
    match datetime_parse_from_str(data, pattern, has_tz, tz_offset) {
        Some(dt) => {
            assert!(expect_dt.is_some(), "\nExpected None\nReceived {:?}\n", expect_dt);
            let e_dt = expect_dt.unwrap();
            assert_eq!(dt, e_dt, "\nExpected {:?}\nReceived {:?}\n", e_dt, dt);
        }
        None => {
            assert!(expect_dt.is_none(), "\nExpected {:?}\nReceived None\n", expect_dt);
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
        Some(dt) => Some(dt.with_timezone(&*TZO_0)),
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
        let tzo = FixedOffset::west_opt(3600 * 2).unwrap();
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
        let tz_dummy = *TZO_0;
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
        let tzo = *TZO_W8;
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
