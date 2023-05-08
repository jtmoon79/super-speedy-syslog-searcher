// src/tests/datetime_tests.rs
// … ≤ ≥ ≠ ≟

//! tests for `datetime.rs` functions

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::tests::common::{
    FO_0,
    FO_P1,
    FO_M7,
    FO_M8,
    FO_E10,
    FO_L,
    FO_L_STR,
    FO_W8,
    FO_Z,
};
use crate::data::datetime::{
    LineIndex,
    ymdhms,
    ymdhmsn,
    ymdhmsm,
    ymdhmsn_args,
    DUMMY_ARGS,
    O_L,
    YEAR_FALLBACKDUMMY_VAL,
    bytes_to_regex_to_datetime,
    datetime_from_str_workaround_Issue660,
    datetime_parse_from_str,
    dt_after_or_before,
    dt_pass_filters,
    DTFSSet,
    DTFS_Tz,
    DTFS_Year,
    DTFS_Epoch,
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
    DateTimeParseInstr,
    DateTimePattern_str,
    DateTimeRegex_str,
    Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    Year,
    DATETIME_PARSE_DATAS_LEN,
    DATETIME_PARSE_DATAS,
    CGP_HOUR_ALL,
    CGP_MINUTE,
    CGP_SECOND,
    CGP_FRACTIONAL,
    CGP_FRACTIONAL3,
    CGP_MONTH_ALL,
    CGN_ALL,
    CGP_DAY_ALL,
    CGP_YEAR,
    CGP_YEARy,
    CGP_TZZ,
    CGP_TZ_ALL,
    CGP_EPOCH,
    MAP_TZZ_TO_TZz,
    RP_LB,
    RP_RB,
    DTP_ALL,
    slice_contains_X_2,
    slice_contains_D2,
    slice_contains_12_D2,
};
use crate::debug::printers::buffer_to_String_noraw;

use std::collections::HashSet;
use std::str;

// for `with_nanosecond()`, `year()`, and others
#[allow(unused_imports)]
use ::chrono::{Datelike, Timelike};
use ::bstr::ByteSlice;
use ::more_asserts::{assert_gt, assert_le, assert_lt};
use ::regex;
use ::si_trace_print::stack::stack_offset_set;
use ::si_trace_print::{defn, defo, defx};
use ::test_case::test_case;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
    for pat in CGP_HOUR_ALL.iter() {
        if pattern.contains(pat) {
            return true;
        }
    }

    false
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

/// does regex pattern have a epoch?
pub fn regex_pattern_has_epoch(pattern: &DateTimeRegex_str) -> bool {
    pattern.contains(CGP_EPOCH)
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

/// does chrono strftime pattern have a epoch?
pub fn dt_pattern_has_epoch(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%s")
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
/// hardcoded generated test cases in the proceeding test function
/// `test_DATETIME_PARSE_DATAS_test_cases`
#[test]
fn test_DATETIME_PARSE_DATAS_test_cases_has_all_test_cases() {
    assert_eq!(
        // THIS NUMBER SHOULD MATCH `DATETIME_PARSE_DATAS_LEN`
        //
        // IF YOU CHANGE THIS NUMBER THEN ALSO UPDATE THE GENERATED TEST CASES
        // FOR `test_DATETIME_PARSE_DATAS_test_cases` BELOW! THOSE TESTS SHOULD
        // BE FROM ZERO TO ONE LESS THAN THIS NUMBER
        122,
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
#[test_case(94)]
#[test_case(95)]
#[test_case(96)]
#[test_case(97)]
#[test_case(98)]
#[test_case(99)]
#[test_case(100)]
#[test_case(101)]
#[test_case(102)]
#[test_case(103)]
#[test_case(104)]
#[test_case(105)]
#[test_case(106)]
#[test_case(107)]
#[test_case(108)]
#[test_case(109)]
#[test_case(110)]
#[test_case(111)]
#[test_case(112)]
#[test_case(113)]
#[test_case(114)]
#[test_case(115)]
#[test_case(116)]
#[test_case(117)]
#[test_case(118)]
#[test_case(119)]
#[test_case(120)]
#[test_case(121)]
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
    let mut dt_pat_min: usize = 8;
    match dtfs.epoch {
        DTFS_Epoch::s => dt_pat_min = 2,
        DTFS_Epoch::_none => {}
    }
    assert_le!(
        dt_pat_min,
        dtpat.len(),
        ".dt_pattern.len too short, less than {}; bad built-in dt_pattern {:?}; declared at line {}",
        dt_pat_min,
        dtpat,
        dtpd._line_num
    );
    // while the pattern could intentionally start with "^" and start past 0,
    // it is most likely a user error
    if dtpat.starts_with('^') {
        assert_eq!(dtpd.range_regex.start, 0, "Pattern user beginning of line yet range starts at {:?}, expected start at 0", dtpd.range_regex.start);
    }

    // check year
    eprintln!("Check year {:?}…", dtfs.year);
    if dtfs.has_year() {
        assert!(
            regex_pattern_has_year(regpat),
            "regex_pattern does not have year {:?} for {:?}; declared at line {}",
            regpat,
            dtfs.year,
            dtpd._line_num
        );
        assert!(
            dt_pattern_has_year(dtpat),
            "dt_pattern does not have year {:?} for {:?}; declared at line {}",
            dtpat,
            dtfs.year,
            dtpd._line_num
        );
    } else {
        assert!(
            !regex_pattern_has_year(regpat),
            "regex_pattern has year {:?} for {:?}; declared at line {}",
            regpat,
            dtfs.year,
            dtpd._line_num
        );
        assert!(
            !dt_pattern_has_year(dtpat) || dtfs.year == DTFS_Year::_fill,
            "dt_pattern has year {:?} for {:?}; declared at line {}",
            dtpat,
            dtfs.year,
            dtpd._line_num
        );
    }
    // check month
    eprintln!("Check month {:?}…", dtfs.month);
    if dtfs.has_month() {
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
    } else {
        assert!(
            !regex_pattern_has_month(regpat),
            "regex_pattern has month {:?} but month is {:?}; declared at line {}",
            regpat,
            dtfs.month,
            dtpd._line_num
        );
        assert!(
            !dt_pattern_has_month(dtpat),
            "dt_pattern has month {:?} but month is {:?}; declared at line {}",
            dtpat,
            dtfs.month,
            dtpd._line_num
        );
    }
    // check day
    eprintln!("Check day {:?}…", dtfs.day);
    if dtfs.has_day() {
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
    } else {
        assert!(
            !regex_pattern_has_day(regpat),
            "regex_pattern has day {:?} but day is {:?}; declared at line {}",
            regpat,
            dtfs.day,
            dtpd._line_num
        );
        assert!(
            !dt_pattern_has_day(dtpat),
            "dt_pattern has day {:?} but day is {:?}; declared at line {}",
            dtpat,
            dtfs.day,
            dtpd._line_num
        );
    }
    // check hour
    eprintln!("Check hour {:?}…", dtfs.hour);
    if dtfs.has_hour() {
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
    } else {
        assert!(
            !regex_pattern_has_hour(regpat),
            "regex_pattern has hour {:?} but hour is {:?}; declared at line {}",
            regpat,
            dtfs.hour,
            dtpd._line_num
        );
        assert!(
            !dt_pattern_has_hour(dtpat),
            "dt_pattern has hour {:?} but hour is {:?}; declared at line {}",
            dtpat,
            dtfs.hour,
            dtpd._line_num
        );
    }
    // check minute
    eprintln!("Check minute {:?}…", dtfs.minute);
    if dtfs.has_minute() {
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
    } else {
        assert!(
            !regex_pattern_has_minute(regpat),
            "regex_pattern has minute {:?} but minute is {:?}; declared at line {}",
            regpat,
            dtfs.minute,
            dtpd._line_num
        );
        assert!(
            !dt_pattern_has_minute(dtpat),
            "dt_pattern has minute {:?} but minute is {:?}; declared at line {}",
            dtpat,
            dtfs.minute,
            dtpd._line_num
        );
    }
    // check second
    eprintln!("Check second {:?}…", dtfs.second);
    let rp_ss = regex_pattern_has_second(regpat);
    let dp_ss = dt_pattern_has_second(dtpat);
    assert_eq!(
        rp_ss, dp_ss,
        "regex_pattern has second {}, datetime pattern has second {}; they must agree; second {:?}, declared at line {}\n  regex pattern: {:?}\n  dt_pattern {:?}\n",
        rp_ss, dp_ss,
        dtfs.second,
        dtpd._line_num,
        regpat,
        dtpat,
    );
    // check fractional
    eprintln!("Check fractional {:?}…", dtfs.fractional);
    let rp_ss = regex_pattern_has_fractional(regpat);
    let dp_ss = dt_pattern_has_fractional(dtpat);
    assert_eq!(
        rp_ss, dp_ss,
        "regex_pattern has fractional {}, datetime pattern has fractional {}; they must agree; fractional {:?}, declared at line {}\n  regex pattern: {:?}\n  dt_pattern {:?}\n",
        rp_ss, dp_ss,
        dtfs.fractional,
        dtpd._line_num,
        regpat,
        dtpat,
    );
    // check timezone
    eprintln!("Check timezone {:?}…", dtfs.tz);
    if dtfs.has_tz() {
        assert!(
            regex_pattern_has_tz(regpat),
            "regex_pattern has not timezone {:?} but tz is {:?}; declared at line {}",
            regpat,
            dtfs.tz,
            dtpd._line_num
        );
        assert!(
            dt_pattern_has_tz(dtpat),
            "dt_pattern has not timezone {:?} but tz is {:?}; declared at line {}",
            dtpat,
            dtfs.tz,
            dtpd._line_num
        );
    } else {
        assert!(
            !regex_pattern_has_tz(regpat),
            "regex_pattern has timezone {:?} but tz is {:?}; declared at line {}",
            regpat,
            dtfs.tz,
            dtpd._line_num
        );
        if dtfs.tz == DTFS_Tz::_fill {
            assert!(
                dt_pattern_has_tz_fill(dtpat),
                "dt_pattern does not have fill timezone {:?} for tz {:?}; declared at line {}",
                dtpat,
                dtfs.tz,
                dtpd._line_num
            );
        } else {
            assert!(
                ! dt_pattern_has_tz_fill(dtpat),
                "dt_pattern has fill timezone {:?} for tz {:?}; declared at line {}",
                dtpat,
                dtfs.tz,
                dtpd._line_num
            );
        }
    }
    // check epoch
    eprintln!("Check epoch {:?}…", dtfs.epoch);
    let rp_ss = regex_pattern_has_epoch(regpat);
    let dp_ss = dt_pattern_has_epoch(dtpat);
    assert_eq!(
        rp_ss, dp_ss,
        "regex_pattern has epoch {}, datetime pattern has epoch {}; they must agree; declared at line {}\n  regex pattern: {:?}\n  dt_pattern {:?}\n",
        rp_ss, dp_ss,
        dtpd._line_num,
        regpat,
        dtpat,
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
    eprintln!("  Regex Pattern     : {:?}", dtpd.regex_pattern);
    eprintln!("  DateTime Pattern  : {:?}", dtpd.dtfs.pattern);
    for test_case_ in dtpd._test_cases {
        eprintln!("  Test Data all          : {:?}", test_case_);
        let data = test_case_.3.as_bytes();
        eprintln!("  Test Data              : {:?}", test_case_.3);
        let slice_a: usize = std::cmp::min(dtpd.range_regex.start, data.len());
        let slice_b: usize = std::cmp::min(dtpd.range_regex.end, data.len());
        let slice_ = &data[slice_a..slice_b];
        eprintln!("  Test Data slice [{:2},{:2}]: {:?}", dtpd.range_regex.start, dtpd.range_regex.end, slice_.as_bstr());
        let dta: LineIndex = test_case_.0;
        let dtb: LineIndex = test_case_.1;
        assert_lt!(dta, dtb, "bad indexes");
        eprintln!("  Test Data expect[{:2},{:2}]: {:?}", dta, dtb, &data[dta..dtb].as_bstr());
        let mut year_opt: Option<Year> = None;
        if !dtpd.dtfs.has_year() {
            year_opt = Some(YEAR_FALLBACKDUMMY_VAL);
        }
        let s = buffer_to_String_noraw(data);
        match bytes_to_regex_to_datetime(slice_, &index, &year_opt, &FO_L, &FO_L_STR) {
            Some(capdata) => {
                eprintln!(
                    "Passed dtpd declared at line {} result {:?}, test data {:?}",
                    dtpd._line_num, capdata, s
                );
                let a: LineIndex = capdata.0;
                let b: LineIndex = capdata.1;
                assert_lt!(a, b, "bad a {} b {}", a, b);
                // verify indexes returned by the regex
                let s_a_b = buffer_to_String_noraw(data[a..b].as_bstr());
                let s_dta_dtb = buffer_to_String_noraw(data[dta..dtb].as_bstr());
                assert_eq!(
                    (dta, dtb), (a, b),
                    "For dtpd at line {:?} unexpected index returned\n  test data {:?}\n  expect {:?} {:?}\n  actual {:?} {:?}\n",
                    dtpd._line_num, s, (dta, dtb), &s_dta_dtb, (a, b), &s_a_b,
                );
                let ymdhmsn_args_: ymdhmsn_args = test_case_.2;
                if ymdhmsn_args_ != DUMMY_ARGS {
                    // verify datetime processed
                    let fo: FixedOffset = match ymdhmsn_args_.0 {
                        O_L => *FO_L,
                        val if val < 0 => FixedOffset::west_opt(-val).unwrap(),
                        val if val >= 0 => FixedOffset::east_opt(val).unwrap(),
                        val => panic!("bad offset value {:?}", val),
                    };
                    let dt: DateTimeL = ymdhmsn(
                        &fo,
                        ymdhmsn_args_.1,
                        ymdhmsn_args_.2,
                        ymdhmsn_args_.3,
                        ymdhmsn_args_.4,
                        ymdhmsn_args_.5,
                        ymdhmsn_args_.6,
                        ymdhmsn_args_.7,
                    );
                    assert_eq!(
                        dt, capdata.2,
                        "For dtpd at line {:?} unexpected datetime returned\n  test data {:?}\n  expect {:?}\n  actual {:?}\n",
                        dtpd._line_num, s, dt, capdata.2,
                    );
                }
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
    // tz_name example "PST" or "pst"
    // tz_val example "-07:00"
    for (tz_name, tz_val) in MAP_TZZ_TO_TZz.entries()  {
        let tz_name_u = tz_name.to_ascii_uppercase();
        let tz_name_l = tz_name.to_ascii_lowercase();
        assert!(tz_name == &tz_name_u.as_str() || tz_name == &tz_name_l.as_str(),
            "Bad timezone name {:?} not all uppercase or all lowercase", tz_name
        );
        assert!(MAP_TZZ_TO_TZz.contains_key(&tz_name_u),
            "Key {:?} as uppercase {:?} not found in MAP_TZZ_TO_TZz", tz_name, tz_name_u
        );
        assert!(MAP_TZZ_TO_TZz.contains_key(&tz_name_l),
            "Key {:?} as lowercase {:?} not found in MAP_TZZ_TO_TZz", tz_name, tz_name_l
        );
        assert!(regex.is_match(tz_name), "Key {:?} from MAP_TZZ_TO_TZz not matched by CGP_TZZ Regex", tz_name);
        let captures = regex.captures(tz_name).unwrap();
        assert_eq!(captures.len(), 2, "CGP_TZZ Regex captured {:?} != 2 expected", captures.len());
        let tz_name_captured = captures.get(1).unwrap().as_str();
        assert_eq!(&tz_name_captured, tz_name, "CGP_TZZ Regex captured {:?} != {:?} expected", tz_name_captured, tz_name);
        assert!(CGP_TZZ.contains(tz_name), "CGP_TZZ does not contain name {:?} from MAP_TZZ_TO_TZz", tz_name);
        if ! tz_val.is_empty() {
            assert_eq!(tz_val.len(), 6, "Bad timezone value {:?} length {:?} for entry {:?}", tz_val, tz_val.len(), tz_name);
            assert!("+-".contains(tz_val.chars().nth(0).unwrap()), "Bad timezone value starts_with {:?} for entry {:?}", tz_val, tz_name);
            assert!("01".contains(tz_val.chars().nth(1).unwrap()), "Bad timezone value {:?} for entry {:?}", tz_val, tz_name);
            assert!("0123456789".contains(tz_val.chars().nth(2).unwrap()), "Bad timezone value {:?} for entry {:?}", tz_val, tz_name);
            assert!(":".contains(tz_val.chars().nth(3).unwrap()), "Bad timezone value {:?} for entry {:?}", tz_val, tz_name);
            assert!(tz_val.ends_with(":00") || tz_val.ends_with(":30") || tz_val.ends_with(":45"), "Bad timezone value ends_with {:?} for entry {:?}", tz_val, tz_name);
            assert!(tz_val.contains(':'), "Bad timezone value {:?} not contains ':' for entry {:?}", tz_val, tz_name);
        } else {
            // empty value means the name is ambiguous
            let tz_val_u = MAP_TZZ_TO_TZz.get(&tz_name_u.as_str()).unwrap();
            let tz_val_l = MAP_TZZ_TO_TZz.get(&tz_name_l.as_str()).unwrap();
            assert!(tz_val_u.is_empty(), "Ambiguous timezone name {:?} has uppercase version {:?} that is not empty {:?}", tz_name, tz_name_u, tz_val_u);
            assert!(tz_val_l.is_empty(), "Ambiguous timezone name {:?} has lowercase version {:?} that is not empty {:?}", tz_name, tz_name_l, tz_val_l);
        }
    }
    let start = CGP_TZZ.find('>');
    assert!(start.is_some(), "CGP_TZZ does not contain start '>'");
    let end = CGP_TZZ.find(')');
    assert!(end.is_some(), "CGP_TZZ does not contain end ')'");
    for val in CGP_TZZ[start.unwrap() + 1..end.unwrap()].split('|') {
        assert!(MAP_TZZ_TO_TZz.contains_key(val), "Substring {:?} from regex CGP_TZZ not found in MAP_TZZ_TO_TZz", val);
    }
}

//#[test]
#[allow(dead_code)]
/// Check that the built-in test data is caught by the same DTPD in which it is
/// declared.
fn _test_DATETIME_PARSE_DATAS_test_cases_indexing() {
    stack_offset_set(Some(2));
    let _tz = *FO_P1;
    for (index, dtpd) in DATETIME_PARSE_DATAS
        .iter()
        .enumerate()
    {
        eprintln!("Testing dtpd declared at line {} …", dtpd._line_num);
        eprintln!("  Regex Pattern   : {:?}", dtpd.regex_pattern);
        eprintln!("  DateTime Pattern: {:?}", dtpd.dtfs.pattern);
        for test_case in dtpd._test_cases {
            eprintln!("  Test Data       : {:?}", test_case);
            let _data = test_case.3.as_bytes();
            let mut _year_opt: Option<Year> = None;
            if !dtpd.dtfs.has_year() {
                _year_opt = Some(YEAR_FALLBACKDUMMY_VAL);
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

#[test_case(
    "20000101T000000", "%Y%m%dT%H%M%S", false, &FO_Z,
    Some(ymdhms(&FO_Z, 2000, 1, 1, 0, 0, 0));
    "20000101T000000 %Y%m%dT%H%M%S no_tz"
)]
#[test_case(
    "20000101T000000 ", "%Y%m%dT%H%M%S", false, &FO_Z, None;
    "20000101T000000  %Y%m%dT%H%M%S no_tz (extra space data)"
)]
#[test_case(
    "20000101T000000", "%Y%m%dT%H%M%S ", false, &FO_Z, None;
    "20000101T000000 %Y%m%dT%H%M%S  no_tz (extra space pattern)"
)]
#[test_case(
    "20000101T000000,123", "%Y%m%dT%H%M%S,%3f", false, &FO_Z,
    Some(ymdhmsm(&FO_Z, 2000, 1, 1, 0, 0, 0, 123000));
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
        Some(dt) => Some(dt.with_timezone(&FO_0)),
        None => None,
    }
}

/// basic test of `SyslineReader.sysline_pass_filters`
#[rustfmt::skip]
#[allow(non_snake_case)]
#[test]
fn test_dt_pass_filters_fixedoffset2() {
    defn!();

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
        defo!("dt_pass_filters(\n\t{:?},\n\t{:?},\n\t{:?}\n)\nreturned expected {:?}", dt, da, db, result);
    }
    defx!();
}

#[rustfmt::skip]
#[allow(non_snake_case)]
#[test]
fn test_dt_pass_filters_z() {
    defn!();

    fn DTLz(s: &str) -> DateTimeL {
        let tz_dummy = *FO_0;
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
        defo!("dt_pass_filters(\n\t{:?},\n\t{:?},\n\t{:?}\n)\nreturned expected {:?}", dt, da, db, result);
    }
    defx!();
}

/// basic test of `SyslineReader.dt_after_or_before`
#[allow(non_snake_case)]
#[test]
fn test_dt_after_or_before() {
    defn!();

    fn DTL(s: &str) -> DateTimeL {
        let tz_offset = *FO_M8;
        datetime_parse_from_str(s, "%Y%m%dT%H%M%S", false, &tz_offset).unwrap()
    }

    fn DTLz(s: &str, tz_offset: &FixedOffset) -> DateTimeL {
        datetime_parse_from_str(s, "%Y%m%dT%H%M%S%z", true, &tz_offset).unwrap()
    }

    for (dt, da, exp_result) in [
        (DTL("20000101T010106"), None, Result_Filter_DateTime1::Pass),
        (DTL("20000101T010101"), Some(DTL("20000101T010103")), Result_Filter_DateTime1::OccursBefore),
        (DTL("20000101T010100"), Some(DTL("20000101T010100")), Result_Filter_DateTime1::OccursAtOrAfter),
        (DTL("20000101T010109"), Some(DTL("20000101T010108")), Result_Filter_DateTime1::OccursAtOrAfter),
        (DTLz("20000101T010106+0100", &FO_P1), None, Result_Filter_DateTime1::Pass),
        (
            DTLz("20000101T010101+0100", &FO_P1),
            Some(DTLz("20000101T010103-0700", &FO_M7)),
            Result_Filter_DateTime1::OccursBefore
        ),
    ] {
        let result = dt_after_or_before(&dt, &da);
        assert_eq!(exp_result, result, "Expected {:?} Got {:?} for ({:?}, {:?})", exp_result, result, dt, da);
        defo!("dt_after_or_before(\n\t{:?},\n\t{:?}\n)\nreturned expected {:?}", dt, da, result);
    }
    defx!();
}

#[test_case(b"", b"xy", false)]
#[test_case(b"a", b"xy", false)]
#[test_case(b"ab", b"xy", false)]
#[test_case(b"abc", b"xy", false)]
#[test_case(b"abcd", b"xy", false)]
#[test_case(b"abcde", b"xy", false)]
#[test_case(b"abcdef", b"xy", false)]
#[test_case(b"ax", b"xy", true)]
#[test_case(b"xa", b"xy", true)]
#[test_case(b"xbc", b"xy", true)]
#[test_case(b"axc", b"xy", true)]
#[test_case(b"abx", b"xy", true)]
#[test_case(b"xbcd", b"xy", true)]
#[test_case(b"axcd", b"xy", true)]
#[test_case(b"abxd", b"xy", true)]
#[test_case(b"abcx", b"xy", true)]
#[test_case(b"xbcde", b"xy", true)]
#[test_case(b"axcde", b"xy", true)]
#[test_case(b"abxde", b"xy", true)]
#[test_case(b"abcxe", b"xy", true)]
#[test_case(b"abcdx", b"xy", true)]
#[test_case(b"xbcdef", b"xy", true)]
#[test_case(b"axcdef", b"xy", true)]
#[test_case(b"abxdef", b"xy", true)]
#[test_case(b"abcxef", b"xy", true)]
#[test_case(b"abcdxf", b"xy", true)]
#[test_case(b"abcdex", b"xy", true)]
#[test_case(b"ay", b"xy", true)]
#[test_case(b"ya", b"xy", true)]
#[test_case(b"ybc", b"xy", true)]
#[test_case(b"ayc", b"xy", true)]
#[test_case(b"aby", b"xy", true)]
#[test_case(b"ybcd", b"xy", true)]
#[test_case(b"aycd", b"xy", true)]
#[test_case(b"abyd", b"xy", true)]
#[test_case(b"abcy", b"xy", true)]
#[test_case(b"ybcde", b"xy", true)]
#[test_case(b"aycde", b"xy", true)]
#[test_case(b"abyde", b"xy", true)]
#[test_case(b"abcye", b"xy", true)]
#[test_case(b"abcdy", b"xy", true)]
#[test_case(b"ybcdef", b"xy", true)]
#[test_case(b"aycdef", b"xy", true)]
#[test_case(b"abydef", b"xy", true)]
#[test_case(b"abcyef", b"xy", true)]
#[test_case(b"abcdyf", b"xy", true)]
#[test_case(b"abcdey", b"xy", true)]
#[test_case(b"ax", b"xx", true)]
#[test_case(b"xx", b"xx", true)]
#[test_case(
    b"abcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrst",
    b"12",
    false
)]
#[test_case(
    b"1bcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrst",
    b"12",
    true
)]
#[test_case(
    b"2bcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrst",
    b"12",
    true
)]
#[test_case(
    b"abcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrs1",
    b"12",
    true
)]
#[test_case(
    b"abcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrstabcdefghijklmnopqrs2",
    b"12",
    true
)]
fn test_slice_contains_X_2(data: &[u8], search: &[u8; 2], expect: bool) {
    let actual = slice_contains_X_2(data, search);
    assert_eq!(expect, actual);
}

#[test_case(b"", false)]
#[test_case(b"a", false)]
#[test_case(b"ab", false)]
#[test_case(b"abc", false)]
#[test_case(b"abcd", false)]
#[test_case(b"1", false)]
#[test_case(b"1a", false)]
#[test_case(b"a1", false)]
#[test_case(b"1bc", false)]
#[test_case(b"a1c", false)]
#[test_case(b"ab1", false)]
#[test_case(b"1bcd", false)]
#[test_case(b"a1cd", false)]
#[test_case(b"ab1d", false)]
#[test_case(b"abc1", false)]
#[test_case(b"12", true)]
#[test_case(b"12c", true)]
#[test_case(b"a12", true)]
#[test_case(b"1a2", false)]
#[test_case(b"12cd", true)]
#[test_case(b"a12d", true)]
#[test_case(b"ab12", true)]
#[test_case(b"1b2d", false)]
#[test_case(b"1bc2", false)]
#[test_case(b"a1c2", false)]
fn test_slice_contains_D2(data: &[u8], expect: bool) {
    let actual = slice_contains_D2(data);
    assert_eq!(expect, actual);
}

#[test_case(b"", false)]
#[test_case(b"a", false)]
#[test_case(b"1", true)]
#[test_case(b"3", false)]
#[test_case(b"ab", false)]
#[test_case(b"3a", false)]
#[test_case(b"a3", false)]
#[test_case(b"1a", true)]
#[test_case(b"a2", true)]
#[test_case(b"12", true)]
#[test_case(b"34", true)]
#[test_case(b"abc", false)]
#[test_case(b"3bc", false)]
#[test_case(b"a3c", false)]
#[test_case(b"ab3", false)]
#[test_case(b"2bc", true)]
#[test_case(b"a2c", true)]
#[test_case(b"ab2", true)]
#[test_case(b"12c", true)]
#[test_case(b"a12", true)]
#[test_case(b"1a2", true)]
#[test_case(b"34c", true)]
#[test_case(b"a34", true)]
#[test_case(b"3a4", false)]
#[test_case(b"abcd", false)]
#[test_case(b"3bcd", false)]
#[test_case(b"a3cd", false)]
#[test_case(b"ab3d", false)]
#[test_case(b"abc3", false)]
#[test_case(b"1bcd", true)]
#[test_case(b"a1cd", true)]
#[test_case(b"ab1d", true)]
#[test_case(b"abc1", true)]
#[test_case(b"12cd", true)]
#[test_case(b"a12d", true)]
#[test_case(b"ab12", true)]
#[test_case(b"34cd", true)]
#[test_case(b"a34d", true)]
#[test_case(b"ab34", true)]
#[test_case(b"3b4d", false)]
#[test_case(b"3bc4", false)]
#[test_case(b"a3c4", false)]
fn test_slice_contains_12_D2(data: &[u8], expect: bool) {
    let actual = slice_contains_12_D2(data);
    assert_eq!(actual, expect);
}
