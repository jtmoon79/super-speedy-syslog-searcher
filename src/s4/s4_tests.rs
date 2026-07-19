// src/s4/s4_tests.rs

//! Tests for `s4.rs`

use std::collections::HashMap;

use ::s4lib::common::{
    FIXEDOFFSET0,
};
use ::s4lib::data::datetime::{
    ymdhms,
    ymdhmsl,
    ymdhmsm,
    DateTimeL,
    DateTimeLOpt,
    DateTimePattern_string,
    FixedOffset,
    MAP_TZZ_TO_TZz,
};

use crate::s4::{
    cli_process_tz_offset,
    cli_parser_prepend_dt_format,
    process_dt,
    DUR_OFFSET_TYPE,
    DurationSetType,
    CLI_OPT_PREPEND_FMT,
    CLI_FILTER_PATTERNS,
    CLI_FILTER_PATTERNS_COUNT,
    EXACT_HMS,
    T_NOW_YEAR,
    T_NOW_MONTH,
    T_NOW_DAY,
    UTC_NOW,
    LOCAL_NOW,
    LOCAL_NOW_YEAR,
    LOCAL_NOW_MONTH,
    LOCAL_NOW_DAY,
    M0130_NOW,
    string_wdhms_to_duration,
    unescape,
};

use ::chrono::{
    Datelike,
    Duration,
    TimeZone,
};
use ::lazy_static::lazy_static;
use ::si_trace_print::stack::stack_offset_set;
#[allow(unused_imports)]
use ::si_trace_print::{
    def1n,
    def1o,
    def1x,
    def1ñ,
    defn,
    defo,
    defx,
    defñ,
    deo,
};
use ::test_case::{
    test_case,
    test_matrix,
};

/// shorter name
const FO0: FixedOffset = FIXEDOFFSET0;

// XXX: these are defined in tests/common.rs but importing that fails
//      unexpectedly
const FO_E1: FixedOffset = FixedOffset::east_opt(3600).unwrap();
const FO_E8: FixedOffset = FixedOffset::east_opt(28800).unwrap();
const FO_W8: FixedOffset = FixedOffset::west_opt(28800).unwrap();

#[test]
fn test_select_remove_reregister_and_signal() {
    let (signal_send, signal_recv) = crossbeam_channel::bounded::<()>(1);
    let (data_send, data_recv) = crossbeam_channel::bounded::<usize>(1);
    let mut select = crossbeam_channel::Select::new();
    let signal_index = select.recv(&signal_recv);
    let data_index = select.recv(&data_recv);
    let mut index_pathid = HashMap::from([(data_index, 7_u64)]);
    let mut pathid_index = HashMap::from([(7_u64, data_index)]);

    data_send.send(42).unwrap();
    let operation = select.select();
    assert_eq!(operation.index(), data_index);
    assert_eq!(operation.recv(&data_recv), Ok(42));

    select.remove(data_index);
    index_pathid.remove(&data_index);
    pathid_index.remove(&7_u64);
    let data_index_reregistered = select.recv(&data_recv);
    assert_ne!(data_index_reregistered, data_index);
    index_pathid.insert(data_index_reregistered, 7_u64);
    pathid_index.insert(7_u64, data_index_reregistered);
    assert_eq!(index_pathid.get(&data_index_reregistered), Some(&7_u64));
    assert_eq!(pathid_index.get(&7_u64), Some(&data_index_reregistered));

    signal_send.try_send(()).unwrap();
    let operation = select.select();
    assert_eq!(operation.index(), signal_index);
    assert_eq!(operation.recv(&signal_recv), Ok(()));
}

lazy_static! {
    /// 1970-01-01T01:00:00+01:00
    pub static ref DT_0_E1: DateTimeL = ymdhms(&FO_E1, 0, 0, 0, 1, 0, 0);
}

#[test_case("+00", FO0; "+00 east(0)")]
#[test_case("+0000", FO0; "+0000 east(0)")]
#[test_case("+00:00", FO0; "+00:00 east(0)")]
#[test_case("+00:01", FixedOffset::east_opt(60).unwrap(); "+00:01 east(60)")]
#[test_case("+01:00", FixedOffset::east_opt(3600).unwrap(); "+01:00 east(3600) A")]
#[test_case("-01:00", FixedOffset::east_opt(-3600).unwrap(); "-01:00 east(-3600) B")]
#[test_case("+02:00", FixedOffset::east_opt(7200).unwrap(); "+02:00 east(7200)")]
#[test_case("+02:30", FixedOffset::east_opt(9000).unwrap(); "+02:30 east(9000)")]
#[test_case("+02:35", FixedOffset::east_opt(9300).unwrap(); "+02:30 east(9300)")]
#[test_case("+23:00", FixedOffset::east_opt(82800).unwrap(); "+23:00 east(82800)")]
#[test_case("gmt", FO0; "GMT (0)")]
#[test_case("UTC", FO0; "UTC east(0)")]
#[test_case("Z", FO0; "Z (0)")]
#[test_case("vlat", FixedOffset::east_opt(36000).unwrap(); "vlat east(36000)")]
#[test_case("IDLW", FixedOffset::east_opt(-43200).unwrap(); "IDLW east(-43200)")]
fn test_cli_process_tz_offset(
    in_: &str,
    out_fo: FixedOffset,
) {
    let input: String = String::from(in_);
    let result = cli_process_tz_offset(&input);
    match result {
        Ok(fo) => {
            assert_eq!(out_fo, fo, "cli_process_tz_offset returned FixedOffset {fo:?}, expected {out_fo:?}");
        }
        Err(err) => {
            panic!("Error {err}");
        }
    }
}

#[test_case("")]
#[test_case("abc")]
#[test_case(CLI_OPT_PREPEND_FMT)]
#[test_case("%Y%Y%Y%Y%Y%Y%Y%%%%")]
fn test_cli_parser_prepend_dt_format(input: &str) {
    assert!(cli_parser_prepend_dt_format(input).is_ok());
}

const FO_M0130: &FixedOffset = &FixedOffset::east_opt(-5400).unwrap();
const FO_M0100: &FixedOffset = &FixedOffset::east_opt(-3600).unwrap();

#[test_case(
    Some(String::from("2000-01-02T03:04:05")),
    FO0,
    Some(FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap());
    "2000-01-02T03:04:05"
)]
#[test_case(
    Some(String::from("2000-01-02T03:04:05.678")),
    FO0,
    Some(ymdhmsl(&FO0, 2000, 1, 2, 3, 4, 5, 678));
    "2000-01-02T03:04:05.678"
)]
#[test_case(
    Some(String::from("2000-01-02T03:04:05.678901")),
    FO0,
    Some(ymdhmsm(&FO0, 2000, 1, 2, 3, 4, 5, 678901));
    "2000-01-02T03:04:05.678901"
)]
#[test_case(
    Some(String::from("2000-01-02T03:04:05.678901-01")),
    FO0,
    Some(ymdhmsm(FO_M0100, 2000, 1, 2, 3, 4, 5, 678901));
    "2000-01-02T03:04:05.678901-01"
)]
#[test_case(
    Some(String::from("2000-01-02T03:04:05.678901-0100")),
    FO0,
    Some(ymdhmsm(FO_M0100, 2000, 1, 2, 3, 4, 5, 678901));
    "2000-01-02T03:04:05.678901-0100"
)]
#[test_case(
    Some(String::from("2000-01-02T03:04:05.678901-01:00")),
    FO0,
    Some(ymdhmsm(FO_M0100, 2000, 1, 2, 3, 4, 5, 678901));
    "2000-01-02T03:04:05.678901-01:00"
)]
#[test_case(
    Some(String::from("2000-01-02T03:04:05.678901 -01:00")),
    FO0,
    Some(ymdhmsm(FO_M0100, 2000, 1, 2, 3, 4, 5, 678901));
    "2000-01-02T03:04:05.678901 -01:00_"
)]
#[test_case(
    Some(String::from("2000-01-02T03:04:05.678901 AZOT")),
    FO0,
    Some(ymdhmsm(FO_M0100, 2000, 1, 2, 3, 4, 5, 678901));
    "2000-01-02T03:04:05.678901 AZOT"
)]
#[test_case(
    Some(String::from("+946782245")),
    FO0,
    Some(FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap());
    "+946782245"
)]
#[test_case(
    Some(String::from("2000-01-02T03:04:05 -0100")),
    FO0,
    Some(FO_M0100.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap());
    "2000-01-02T03:04:05 -0100"
)]
#[test_case(
    Some(String::from("2000-01-02T03:04:05PDT")),
    FO0,
    Some(FixedOffset::east_opt(-25200).unwrap().with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap());
    "2000-01-02T03:04:05PDT"
)]
#[test_case(
    // bad timezone
    Some(String::from("2000-01-02T03:04:05FOOO")),
    FO0,
    None;
    "2000-01-02T03:04:05FOOO"
)]
#[test_case(
    Some(String::from("2000/01/02 03:04:05")),
    FO0,
    Some(FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap());
    "2000-01-02T03:04:05 (no TZ)"
)]
#[test_case(
    Some(String::from("2000/01/02 03:04:05.678")),
    FO0,
    Some(ymdhmsl(&FO0, 2000, 1, 2, 3, 4, 5, 678));
    "2000-01-02 03:04:05.678"
)]
#[test_case(
    Some(String::from("2000/01/02 03:04:05.678901")),
    FO0,
    Some(ymdhmsm(&FO0, 2000, 1, 2, 3, 4, 5, 678901));
    "2000-01-02 03:04:05.678901"
)]
#[test_case(
    Some(String::from("2000/01/02 03:04:05.678901 -01")),
    FO0,
    Some(ymdhmsm(FO_M0100, 2000, 1, 2, 3, 4, 5, 678901));
    "2000-01-02 03:04:05.678901 -01"
)]
#[test_case(
    Some(String::from("2000/01/02 03:04:05.678901 -01:30")),
    FO0,
    Some(ymdhmsm(FO_M0130, 2000, 1, 2, 3, 4, 5, 678901));
    "2000-01-02 03:04:05.678901 -01:30"
)]
#[test_case(
    Some(String::from("2000/01/02 03:04:05.678901 -0130")),
    FO0,
    Some(ymdhmsm(FO_M0130, 2000, 1, 2, 3, 4, 5, 678901));
    "2000-01-02 03:04:05.678901 -0130"
)]
#[test_case(
    Some(String::from("2026-01-02")),
    *FO_M0130,
    Some(ymdhms(FO_M0130, 2026, 1, 2, 0, 0, 0));
    "2026-01-02 YMD only A"
)]
#[test_case(
    Some(String::from("2026/01/02")),
    *FO_M0130,
    Some(ymdhms(FO_M0130, 2026, 1, 2, 0, 0, 0));
    "2026/01/02 YMD only B"
)]    
#[test_case(
    Some(String::from("2026-01-02 1")),
    *FO_M0130,
    None;
    "2026-01-02 1 YMD failure"
)]
#[test_case(
    Some(String::from("01-02")),
    *FO_M0130,
    Some(ymdhms(FO_M0130, LOCAL_NOW_YEAR.with(|y| *y), 1, 2, 0, 0, 0));
    "01-02 MD only"
)]
#[test_case(
    Some(String::from("01-02 1")),
    *FO_M0130,
    None;
    "01-02 1 MD failure"
)]
#[test_case(
    Some(String::from("23:55")),
    *FO_M0130,
    Some(ymdhms(
        &FO_M0130,
        LOCAL_NOW_YEAR.with(|y| *y),
        LOCAL_NOW_MONTH.with(|m| *m),
        LOCAL_NOW_DAY.with(|d| *d),
        23,
        55,
        0
    ));
    "23:55 HM only"
)]
#[test_case(
    Some(String::from("23:555")),
    *FO_M0130,
    None;
    "23:555 HM failure"
)]
#[test_case(
    Some(String::from("23:55+")),
    *FO_M0130,
    None;
    "23:55p HM failure"
)]
#[test_case(
    Some(String::from("23:55@")),
    *FO_M0130,
    None;
    "23:55a HM failure"
)]
#[test_case(
    Some(String::from("23:55:59")),
    *FO_M0130,
    Some(ymdhms(
        &FO_M0130,
        LOCAL_NOW_YEAR.with(|y| *y),
        LOCAL_NOW_MONTH.with(|m| *m),
        LOCAL_NOW_DAY.with(|d| *d),
        23,
        55,
        59
    ));
    "23:55:59 HMS only"
)]
pub (crate) fn test_process_dt(
    dts: Option<String>,
    tz_offset: FixedOffset,
    expect: DateTimeLOpt,
) {
    defn!("test_process_dt({:?}, {:?}, {:?})", dts, tz_offset, expect);
    let utc_now = UTC_NOW.with(|utc_now| *utc_now);
    defo!("utc_now: {:?}", utc_now);
    let local_now = LOCAL_NOW.with(|local_now| *local_now);
    defo!("local_now: {:?}", local_now);
    let m0130_now = M0130_NOW.with(|m0130_now| *m0130_now);
    defo!("m0130_now: {:?}", m0130_now);
    let dt = process_dt(&dts, &tz_offset, &None, &utc_now);
    assert_eq!(
        dt, expect,
        "\nexpect {expect:?}\nactual {dt:?}\nfor process_dt({dts:?}, {tz_offset:?}, &None, UTC_NOW: {utc_now:?})",
    );
    defx!();
}

#[test_case(
    Some(String::from("@+1s")),
    FO0,
    FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    Some(FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 6).unwrap());
    "add 1s"
)]
#[test_case(
    Some(String::from("@-1s")),
    FO0,
    FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    Some(FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 4).unwrap());
    "sub 1s"
)]
#[test_case(
    Some(String::from("@-1d!13:44:55")),
    FO0,
    FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    Some(FO0.with_ymd_and_hms(2000, 1, 1, 13, 44, 55).unwrap());
    "sub 1d clock override 13 44 55"
)]
#[test_case(
    Some(String::from("@+1w1d!13:44")),
    FO0,
    FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    Some(FO0.with_ymd_and_hms(2000, 1, 10, 13, 44, 0).unwrap());
    "add 1w1d clock override 13 44"
)]
#[test_case(
    Some(String::from("@-1d!13")),
    FO0,
    FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    Some(FO0.with_ymd_and_hms(2000, 1, 1, 13, 0, 0).unwrap());
    "sub 1d clock override 13"
)]
#[test_case(
    Some(String::from("@+4h1d")),
    FO0,
    FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    Some(FO0.with_ymd_and_hms(2000, 1, 3, 7, 4, 5).unwrap());
    "add 4h1d"
)]
#[test_case(
    Some(String::from("@+4h1d")),
    FixedOffset::east_opt(-3630).unwrap(),
    FixedOffset::east_opt(-3630).unwrap().with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    Some(FixedOffset::east_opt(-3630).unwrap().with_ymd_and_hms(2000, 1, 3, 7, 4, 5).unwrap());
    "add 4h1d offset -3600"
)]
#[test_case(
    Some(String::from("@-1d!3")),
    FO0,
    FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    None;
    "bad override hour3"
)]
#[test_case(
    Some(String::from("@-1d!02:")),
    FO0,
    FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    None;
    "bad override hour2 colon"
)]
#[test_case(
    Some(String::from("@-1d!09:5")),
    FO0,
    FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    None;
    "bad override hour9 colon 5"
)]
fn test_process_dt_other(
    dts: Option<String>,
    tz_offset: FixedOffset,
    dt_other: DateTimeL,
    expect: DateTimeLOpt,
) {
    stack_offset_set(None);
    let dt = process_dt(
        &dts,
        &tz_offset,
        &Some(dt_other),
        &UTC_NOW.with(|utc_now| *utc_now),
    );
    assert_eq!(dt, expect,
        "\nexpect {expect:?}\nactual {dt:?}\nfor process_dt({dts:?}, {tz_offset:?}, Some({dt_other:?}), UTC_NOW: {:?})",
        UTC_NOW.with(|utc_now| *utc_now),
    );
}

/// helper to print patterns at index for humans debugging stuff.
/// run with:
/// `cargo test tests::s4::test_cli_filter_patterns_print_indexes -- --nocapture`
#[test]
fn test_cli_filter_patterns_print_indexes() {
    stack_offset_set(None);
    defn!();
    for i in 0..CLI_FILTER_PATTERNS_COUNT {
        let dtf_pattern = &CLI_FILTER_PATTERNS[i];
        defo!(
            "CLI_FILTER_PATTERNS[{i}] pattern: {:?}", dtf_pattern.pattern,
        );
    }
    defx!();
}

#[test_matrix(0..95)]  // last matrix value must be CLI_FILTER_PATTERNS_COUNT
fn test_cli_filter_patterns_test_cases(index: usize) {
    stack_offset_set(None);
    defn!("test_cli_filter_patterns_test_cases index: {}", index);
    let local_now = LOCAL_NOW.with(|local_now| *local_now);
    let dtf_pattern = &CLI_FILTER_PATTERNS[index];
    for (input_, dt_data_expect) in dtf_pattern._test_cases.iter() {
        defo!(
            "test_cli_filter_patterns_test_cases index: {}, pattern: {:?}, input: {:?}",
            index,
            dtf_pattern.pattern,
            input_,
        );
        let (
            fo,
            mut y,
            mut m,
            mut d,
            h,
            min,
            s,
            frac6_micro,
        ) = *dt_data_expect;
        if y == T_NOW_YEAR {
            y = local_now.year();
        }
        if m == T_NOW_MONTH {
            m = local_now.month();
        }
        if d == T_NOW_DAY {
            d = local_now.day();
        }
        let dt_expect = ymdhmsm(
            &fo,
            y,
            m,
            d,
            h,
            min,
            s,
            frac6_micro as i64,
        );
        let result: DateTimeLOpt = process_dt(
            &Some(String::from(*input_)),
            &fo,
            &None,
            &UTC_NOW.with(|utc_now| *utc_now),
        );
        assert_eq!(
            result,
            Some(dt_expect),
            "\npattern {:?}\ninput {:?}\nexpect {:?}\nactual {:?}\nfor pattern on line {}",
            dtf_pattern.pattern,
            input_,
            dt_expect,
            result,
            dtf_pattern._line_num,
        );
    }
    defx!("test_cli_filter_patterns_test_cases index: {} passed", index);
}

#[test]
fn test_cli_filter_patterns_static() {
    #[allow(non_snake_case)]
    for dtf_pattern in CLI_FILTER_PATTERNS.iter() {
        let pattern: DateTimePattern_string = DateTimePattern_string::from(dtf_pattern.pattern);
        // timezone / fixedoffset
        if dtf_pattern.has_named_tz {
            assert!(pattern.contains("%Z"));
            for (input, _) in dtf_pattern._test_cases.iter() {
                let mut tz_name_found: bool = false;
                for (tz_name, _) in &MAP_TZZ_TO_TZz {
                    if input.contains(tz_name) {
                        tz_name_found = true;
                        break;
                    }
                }
                assert!(
                    tz_name_found,
                    "input {:?} must contain a named timezone because has_named_tz is true; line {}",
                    input,
                    dtf_pattern._line_num,
                );
            }
        }
        if dtf_pattern.add_tz {
            assert!(
                !pattern.contains("%z")
                && !pattern.contains("%:z")
                && !pattern.contains("%::z")
                && !pattern.contains("%Z"),
                "pattern {pattern:?} should not contain timezone specifiers because add_tz is true; line {}",
                dtf_pattern._line_num,
            );
        }
        // year
        if dtf_pattern.add_date_y {
            assert!(
                !pattern.contains("%Y") && !pattern.contains("%y"),
                "pattern {pattern:?} should not contain year specifiers because add_date_y is true; line {}",
                dtf_pattern._line_num,
            );
        }
        // month
        if dtf_pattern.add_date_m {
            assert!(
                !pattern.contains("%m") && !pattern.contains("%b") && !pattern.contains("%B"),
                "pattern {pattern:?} should not contain month specifiers because add_date_m is true; line {}",
                dtf_pattern._line_num,
            );
        }
        // day
        if dtf_pattern.add_date_d {
            assert!(
                !pattern.contains("%d") && !pattern.contains("%e"),
                "pattern {pattern:?} should not contain day specifiers because add_date_d is true; line {}",
                dtf_pattern._line_num,
            );
        }
        // hour
        if dtf_pattern.add_time_h {
            assert!(
                !pattern.contains("%H")
                && !pattern.contains("%I")
                && !pattern.contains("%k")
                && !pattern.contains("%l"),
                "pattern {pattern:?} should not contain hour specifiers because add_time_h is true; line {}",
                dtf_pattern._line_num,
            );
        }
        // minute
        if dtf_pattern.add_time_m {
            assert!(
                !pattern.contains("%M"),
                "pattern {pattern:?} should not contain minute specifiers because add_time_m is true; line {}",
                dtf_pattern._line_num,
            );
        }
        // second
        if dtf_pattern.add_time_s {
            assert!(
                !pattern.contains("%S"),
                "pattern {pattern:?} should not contain second specifier because add_time_s is true; line {}",
                dtf_pattern._line_num,
            );
            assert!(
                !pattern.contains("%s"),
                "pattern {pattern:?} should not contain timestamp specifier because add_time_s is true; line {}",
                dtf_pattern._line_num,
            );
        }
        // fractional
    }
}

pub(crate) const NOW: DUR_OFFSET_TYPE = DUR_OFFSET_TYPE::Now;
pub(crate) const OTHER: DUR_OFFSET_TYPE = DUR_OFFSET_TYPE::Other;

const EN: EXACT_HMS = EXACT_HMS::None;

const fn d_s(val: i64) -> Duration {
    Duration::try_seconds(val).unwrap()
}

const fn d_m(val: i64) -> Duration {
    Duration::try_minutes(val).unwrap()
}

const fn d_h(val: i64) -> Duration {
    Duration::try_hours(val).unwrap()
}

const fn d_d(val: i64) -> Duration {
    Duration::try_days(val).unwrap()
}

const fn d_w(val: i64) -> Duration {
    Duration::try_weeks(val).unwrap()
}

const N: Option<FixedOffset> = None;

// bad values
#[test_case(String::from(""), None)]
#[test_case(String::from("1s"), None; "1s")]
#[test_case(String::from("@1s"), None; "at_1s")]
#[test_case(String::from("+1z"), None; "plus_1z")]
// good values
#[test_case(String::from("-0s"), Some((d_s(0), NOW, EN, N)))]
#[test_case(String::from("@+0s"), Some((d_s(0), OTHER, EN, N)))]
#[test_case(String::from("-1s"), Some((d_s(-1), NOW, EN, N)); "minus_1s")]
#[test_case(String::from("+1s"), Some((d_s(1), NOW, EN, N)); "plus_1s")]
#[test_case(String::from("+1m"), Some((d_m(1), NOW, EN, N)); "plus_1m")]
#[test_case(String::from("+1h"), Some((d_h(1), NOW, EN, N)); "plus_1h")]
#[test_case(String::from("+1d"), Some((d_d(1), NOW, EN, N)); "plus_1d")]
#[test_case(String::from("+1w"), Some((d_w(1), NOW, EN, N)); "plus_1w")]
#[test_case(String::from("+1w!13"), Some((d_w(1), NOW, EXACT_HMS::HMS(13, 0, 0), N)); "plus_1w!13")]
#[test_case(String::from("@-1s"), Some((d_s(-1), OTHER, EN, N)); "at_minus_1s")]
#[test_case(String::from("@+1s"), Some((d_s(1), OTHER, EN, N)); "at_plus_1s")]
#[test_case(String::from("@+9876s"), Some((d_s(9876), OTHER, EN, N)); "other_plus_9876")]
#[test_case(String::from("@-9876s"), Some((d_s(-9876), OTHER, EN, N)); "other_minus_9876")]
#[test_case(String::from("-9876s"), Some((d_s(-9876), NOW, EN, N)); "now_minus_9876")]
#[test_case(String::from("-3h"), Some((d_h(-3), NOW, EN, N)))]
#[test_case(String::from("-4d"), Some((d_d(-4), NOW, EN, N)))]
#[test_case(String::from("-5w"), Some((d_w(-5), NOW, EN, N)))]
#[test_case(String::from("@+5w"), Some((d_w(5), OTHER, EN, N)))]
#[test_case(String::from("-2m1s"), Some((d_m(-2) + d_s(-1), NOW, EN, N)); "minus_2m1s")]
#[test_case(String::from("-2d1h"), Some((d_d(-2) + d_h(-1), NOW, EN, N)); "minus_2d1h")]
#[test_case(String::from("+2d1h"), Some((d_d(2) + d_h(1), NOW, EN, N)); "plus_2d1h")]
#[test_case(String::from("@+2d1h"), Some((d_d(2) + d_h(1), OTHER, EN, N)); "at_plus_2d1h")]
// "reverse" order should not matter
#[test_case(String::from("-1h2d"), Some((d_h(-1) + d_d(-2), NOW, EN, N)); "minus_1h2d")]
#[test_case(String::from("-4w3d2m1s"), Some((d_w(-4) + d_d(-3) + d_m(-2) + d_s(-1), NOW, EN, N)); "minus_4w3d2m1s")]
// "mixed" order should not matter
#[test_case(String::from("-3d4w1s2m"), Some((d_w(-4) + d_d(-3) + d_m(-2) + d_s(-1), NOW, EN, N)))]
// repeat values; only last is used
#[test_case(String::from("-6w5w4w"), Some((d_w(-4), NOW, EN, N)))]
// repeat values; only last is used
#[test_case(String::from("-5w4w3d2m1s"), Some((d_w(-4) + d_d(-3) + d_m(-2) + d_s(-1), NOW, EN, N)))]
// repeat values; only last is used
#[test_case(String::from("-6w5w4w3d2m1s"), Some((d_w(-4) + d_d(-3) + d_m(-2) + d_s(-1), NOW, EN, N)))]
#[test_case(String::from("+1d1w!12:33"), Some((d_d(1) + d_w(1), NOW, EXACT_HMS::HMS(12, 33, 0), N)); "plus_1d_12_33")]
#[test_case(String::from("+1d!01:02:03"), Some((d_d(1), NOW, EXACT_HMS::HMS(1, 2, 3), N)); "plus_1d_01_02_03")]
// with timezone offset bad
#[test_case(String::from("+1d!01:02:03 PST"), None; "plus_1d_01_02_03 PST")]
#[test_case(String::from("+1d!01:02:03 9"), None; "plus_1d_01_02_03 9")]
#[test_case(String::from("+1d!01:02:03 -08A"), None; "plus_1d_01_02_03 sW08A")]
// with timezone offset good
#[test_case(String::from("+1d!01:02:03+08:00"), Some((d_d(1), NOW, EXACT_HMS::HMS(1, 2, 3), Some(FO_E8))); "plus_1d_01_02_03 E08:00")]
#[test_case(String::from("+1d!01:02:03-08:00"), Some((d_d(1), NOW, EXACT_HMS::HMS(1, 2, 3), Some(FO_W8))); "plus_1d_01_02_03 W08:00")]
#[test_case(String::from("+1d!01:02:03-08"), Some((d_d(1), NOW, EXACT_HMS::HMS(1, 2, 3), Some(FO_W8))); "plus_1d_01_02_03 W08")]
#[test_case(String::from("+1d!01:02:03 -08"), Some((d_d(1), NOW, EXACT_HMS::HMS(1, 2, 3), Some(FO_W8))); "plus_1d_01_02_03 sW08")]
fn test_string_wdhms_to_duration(
    input: String,
    expect: DurationSetType,
) {
    let actual = string_wdhms_to_duration(&input);
    assert_eq!(actual, expect);
}

#[test_case("", Some(""))]
#[test_case("a", Some("a"))]
#[test_case("abc", Some("abc"))]
#[test_case(r"\t", Some("\t"))]
#[test_case(r"\v", Some("\u{0B}"))]
#[test_case(r"\e", Some("\u{1B}"))]
#[test_case(r"\0", Some("\u{00}"))]
#[test_case(r"-\0-", Some("-\u{00}-"); "dash null dash")]
#[test_case(r":\t|", Some(":\t|"); "colon tab vertical pipe")]
#[test_case(r":\t\\|", Some(":\t\\|"); "colon tab escape vertical pipe")]
#[test_case(r"\\\t", Some("\\\t"); "escape tab")]
#[test_case(r"\\t", Some("\\t"); "escape t")]
#[test_case(r"\", None)]
#[test_case(r"\X", None)]
#[test_case(r"\x0", None)]
#[test_case(r"\x00\", None)]
#[test_case(r"\x0Z", None)]
#[test_case(r"\xZ0", None)]
#[test_case(r"\x00", Some("\0"); "hex escape 00")]
#[test_case(r"\x000", Some("\00"); "hex escape 00 0")]
#[test_case(r"\x3B", Some(";"); "hex escape 3B semicolon")]
#[test_case(r"\x3BZ", Some(";Z"); "hex escape 3B semicolon Z")]
#[test_case(r"A\x3B", Some("A;"); "A hex escape 3B semicolon")]
#[test_case(r"A\x3BZ", Some("A;Z"); "A hex escape 3B semicolon Z")]
#[test_case(r"A\x3BC\x3AZ", Some("A;C:Z"); "A hex escape 3B semicolon C hex escape 3A colon Z")]
#[test_case(unescape::BACKSLASH_ESCAPE_SEQUENCES0, Some("\0"); "BACKSLASH_ESCAPE_SEQUENCES0")]
#[test_case(unescape::BACKSLASH_ESCAPE_SEQUENCES1, Some("\u{07}"); "BACKSLASH_ESCAPE_SEQUENCES1")]
#[test_case(unescape::BACKSLASH_ESCAPE_SEQUENCES2, Some("\u{08}"); "BACKSLASH_ESCAPE_SEQUENCES2")]
#[test_case(unescape::BACKSLASH_ESCAPE_SEQUENCES3, Some("\u{1B}"); "BACKSLASH_ESCAPE_SEQUENCES3")]
#[test_case(unescape::BACKSLASH_ESCAPE_SEQUENCES4, Some("\u{0C}"); "BACKSLASH_ESCAPE_SEQUENCES4")]
#[test_case(unescape::BACKSLASH_ESCAPE_SEQUENCES5, Some("\n"); "BACKSLASH_ESCAPE_SEQUENCES5")]
#[test_case(unescape::BACKSLASH_ESCAPE_SEQUENCES6, Some("\r"); "BACKSLASH_ESCAPE_SEQUENCES6")]
#[test_case(unescape::BACKSLASH_ESCAPE_SEQUENCES7, Some("\\"); "BACKSLASH_ESCAPE_SEQUENCES7")]
#[test_case(unescape::BACKSLASH_ESCAPE_SEQUENCES8, Some("\t"); "BACKSLASH_ESCAPE_SEQUENCES8")]
#[test_case(unescape::BACKSLASH_ESCAPE_SEQUENCES9, Some("\u{0B}"); "BACKSLASH_ESCAPE_SEQUENCES9")]
fn test_unescape_str(
    input: &str,
    expect: Option<&str>,
) {
    let result = unescape::unescape_str(input);
    match (result, expect) {
        (Ok(actual_s), Some(expect_s)) => {
            assert_eq!(
                actual_s, expect_s,
                "Input: {input:?}\nExpected {expect_s:?}\nActual   {actual_s:?}\n");
        }
        (Ok(actual_s), None) => {
            panic!("Expected Error, got {actual_s:?}, input {input:?}");
        }
        (Err(err), Some(_)) => {
            panic!("Got Error {err:?} for input {input:?}, expected Ok");
        }
        (Err(_), None) => {}
    }
}
