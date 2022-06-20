// Data/datetime_tests.rs
//

use super::datetime::{
    FixedOffset,
    dt_pattern_has_year,
    dt_pattern_has_tz,
    str_datetime,
    DateTimeL,
    DateTimeL_Opt,
    DATETIME_PARSE_DATAS,
    DATETIME_PARSE_DATAS_VEC,
    Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    dt_pass_filters,
    dt_after_or_before,
    datetime_from_str_workaround_Issue660,
};

use crate::printer_debug::stack::{
    sn,
    sx,
};

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    assert_ge,
};

#[cfg(test)]
use std::str;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// built-in sanity check of the static DATETIME_PARSE_DATAS
/// can only check for coarse errors, cannot catch all errors
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
fn fo_to_fo0(dt_opt: &DateTimeL_Opt) -> DateTimeL_Opt {
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
    eprintln!("{}test_dt_pass_filters_fixedoffset2()", sn());

    fn DTL(s: &str) -> DateTimeL {
        let tzo = FixedOffset::west(3600 * 2);
        str_datetime(s, "%Y%m%dT%H%M%S", false, &tzo).unwrap()
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
    eprintln!("{}test_dt_pass_filters_fixedoffset2()", sx());
}

#[rustfmt::skip]
#[allow(non_snake_case)]
#[test]
fn test_dt_pass_filters_z() {
    eprintln!("{}test_dt_pass_filters_z()", sn());

    fn DTLz(s: &str) -> DateTimeL {
        let dummy = FixedOffset::east(0);
        str_datetime(s, "%Y%m%dT%H%M%S%z", true, &dummy).unwrap()
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
    eprintln!("{}test_dt_pass_filters_z()", sx());
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
        let result = dt_after_or_before(&dt, &da);
        assert_eq!(exp_result, result, "Expected {:?} Got {:?} for ({:?}, {:?})", exp_result, result, dt, da);
        eprintln!("dt_after_or_before(\n\t{:?},\n\t{:?}\n)\nreturned expected {:?}", dt, da, result);
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