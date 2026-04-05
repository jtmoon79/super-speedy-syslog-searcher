// src/s4/s4_tests.rs

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
use ::s4lib::readers::blockreader::{
    BlockSz,
};

use crate::s4::{
    cli_parse_blocksz,
    cli_process_blocksz,
    cli_process_tz_offset,
    cli_parser_prepend_dt_format,
    process_dt,
    DUR_OFFSET_TYPE,
    CLI_OPT_PREPEND_FMT,
    CLI_FILTER_PATTERNS,
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
    //Timelike,
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
use ::test_case::test_case;

/// shorter name
const FO0: FixedOffset = FIXEDOFFSET0;

// XXX: these are defined in tests/common.rs but importing that fails
//      unexpectedly
//const FO0: FixedOffset = FixedOffset::east_opt(0).unwrap();
const FO_E1: FixedOffset = FixedOffset::east_opt(3600).unwrap();

lazy_static! {
    /// 1970-01-01T01:00:00+01:00
    pub static ref DT_0_E1: DateTimeL = ymdhms(&FO_E1, 0, 0, 0, 1, 0, 0);
}

#[test_case("500", true)]
#[test_case("0x2", true)]
#[test_case("0x4", true)]
#[test_case("0xFFFFFF", true)]
#[test_case("BAD_BLOCKSZ_VALUE", false)]
#[test_case("", false)]
fn test_cli_parse_blocksz(
    blocksz_str: &str,
    is_ok: bool,
) {
    match is_ok {
        true => assert!(cli_parse_blocksz(blocksz_str).is_ok()),
        false => assert!(!cli_parse_blocksz(blocksz_str).is_ok()),
    }
}

#[test_case(
    "0b10101010101",
    Some(0b10101010101)
)]
#[test_case("0o44", Some(0o44))]
#[test_case("00500", Some(500))]
#[test_case("500", Some(500))]
#[test_case("0x4", Some(0x4))]
#[test_case("0xFFFFFF", Some(0xFFFFFF))]
#[test_case("BAD_BLOCKSZ_VALUE", None)]
#[test_case("", None)]
fn test_cli_process_blocksz(
    blocksz_str: &str,
    expect_: Option<BlockSz>,
) {
    match expect_ {
        Some(val_exp) => {
            let val_ret = cli_process_blocksz(&String::from(blocksz_str)).unwrap();
            assert_eq!(val_ret, val_exp);
        }
        None => {
            let ret = cli_process_blocksz(&String::from(blocksz_str));
            assert!(
                ret.is_err(),
                "Expected an Error for cli_process_blocksz({:?}), instead got {:?}",
                blocksz_str,
                ret
            );
        }
    }
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
            assert_eq!(out_fo, fo, "cli_process_tz_offset returned FixedOffset {:?}, expected {:?}", fo, out_fo);
        }
        Err(err) => {
            panic!("Error {}", err);
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
    "2000-01-02T03:04:05 add 1s"
)]
#[test_case(
    Some(String::from("@-1s")),
    FO0,
    FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    Some(FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 4).unwrap());
    "2000-01-02T03:04:04 add 1s"
)]
#[test_case(
    Some(String::from("@+4h1d")),
    FO0,
    FO0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    Some(FO0.with_ymd_and_hms(2000, 1, 3, 7, 4, 5).unwrap());
    "2000-01-02T03:04:05 sub 4h1d"
)]
#[test_case(
    Some(String::from("@+4h1d")),
    FixedOffset::east_opt(-3630).unwrap(),
    FixedOffset::east_opt(-3630).unwrap().with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
    Some(FixedOffset::east_opt(-3630).unwrap().with_ymd_and_hms(2000, 1, 3, 7, 4, 5).unwrap());
    "2000-01-02T03:04:05 sub 4h1d offset -3600"
)]
fn test_process_dt_other(
    dts: Option<String>,
    tz_offset: FixedOffset,
    dt_other: DateTimeL,
    expect: DateTimeLOpt,
) {
    let dt = process_dt(
        &dts,
        &tz_offset,
        &Some(dt_other),
        &UTC_NOW.with(|utc_now| *utc_now),
    );
    assert_eq!(dt, expect);
}

/// helper to print patterns at index for humans debugging stuff.
/// run with:
/// `cargo test tests::s4::test_cli_filter_patterns_print_indexes -- --nocapture`
#[test]
fn test_cli_filter_patterns_print_indexes() {
    stack_offset_set(None);
    defn!();
    for i in 0..CLI_FILTER_PATTERNS.len() {
        let dtf_pattern = &CLI_FILTER_PATTERNS[i];
        defo!(
            "CLI_FILTER_PATTERNS[{i}] pattern: {:?}", dtf_pattern.pattern,
        );
    }
    defx!();
}

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
// test_case indexes must be up to CLI_FILTER_PATTERNS.len() - 1 (CLI_FILTER_PATTERNS_COUNT - 1)
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

#[test_case(String::from(""), None)]
#[test_case(String::from("1s"), None; "1s")]
#[test_case(String::from("@1s"), None; "at_1s")]
#[test_case(String::from("-0s"), Some((Duration::try_seconds(0).unwrap(), NOW)))]
#[test_case(String::from("@+0s"), Some((Duration::try_seconds(0).unwrap(), OTHER)))]
#[test_case(String::from("-1s"), Some((Duration::try_seconds(-1).unwrap(), NOW)); "minus_1s")]
#[test_case(String::from("+1s"), Some((Duration::try_seconds(1).unwrap(), NOW)); "plus_1s")]
#[test_case(String::from("@-1s"), Some((Duration::try_seconds(-1).unwrap(), OTHER)); "at_minus_1s")]
#[test_case(String::from("@+1s"), Some((Duration::try_seconds(1).unwrap(), OTHER)); "at_plus_1s")]
#[test_case(String::from("@+9876s"), Some((Duration::try_seconds(9876).unwrap(), OTHER)); "other_plus_9876")]
#[test_case(String::from("@-9876s"), Some((Duration::try_seconds(-9876).unwrap(), OTHER)); "other_minus_9876")]
#[test_case(String::from("-9876s"), Some((Duration::try_seconds(-9876).unwrap(), NOW)); "now_minus_9876")]
#[test_case(String::from("-3h"), Some((Duration::try_hours(-3).unwrap(), NOW)))]
#[test_case(String::from("-4d"), Some((Duration::try_days(-4).unwrap(), NOW)))]
#[test_case(String::from("-5w"), Some((Duration::try_weeks(-5).unwrap(), NOW)))]
#[test_case(String::from("@+5w"), Some((Duration::try_weeks(5).unwrap(), OTHER)))]
#[test_case(String::from("-2m1s"), Some((Duration::try_seconds(-1).unwrap() + Duration::try_minutes(-2).unwrap(), NOW)); "minus_2m1s")]
#[test_case(String::from("-2d1h"), Some((Duration::try_hours(-1).unwrap() + Duration::try_days(-2).unwrap(), NOW)); "minus_2d1h")]
#[test_case(String::from("+2d1h"), Some((Duration::try_hours(1).unwrap() + Duration::try_days(2).unwrap(), NOW)); "plus_2d1h")]
#[test_case(String::from("@+2d1h"), Some((Duration::try_hours(1).unwrap() + Duration::try_days(2).unwrap(), OTHER)); "at_plus_2d1h")]
// "reverse" order should not matter
#[test_case(String::from("-1h2d"), Some((Duration::try_hours(-1).unwrap() + Duration::try_days(-2).unwrap(), NOW)); "minus_1h2d")]
#[test_case(String::from("-4w3d2m1s"), Some((Duration::try_seconds(-1).unwrap() + Duration::try_minutes(-2).unwrap() + Duration::try_days(-3).unwrap() + Duration::try_weeks(-4).unwrap(), NOW)))]
// "mixed" order should not matter
#[test_case(String::from("-3d4w1s2m"), Some((Duration::try_seconds(-1).unwrap() + Duration::try_minutes(-2).unwrap() + Duration::try_days(-3).unwrap() + Duration::try_weeks(-4).unwrap(), NOW)))]
// repeat values; only last is used
#[test_case(String::from("-6w5w4w"), Some((Duration::try_weeks(-4).unwrap(), NOW)))]
// repeat values; only last is used
#[test_case(String::from("-5w4w3d2m1s"), Some((Duration::try_seconds(-1).unwrap() + Duration::try_minutes(-2).unwrap() + Duration::try_days(-3).unwrap() + Duration::try_weeks(-4).unwrap(), NOW)))]
// repeat values; only last is used
#[test_case(String::from("-6w5w4w3d2m1s"), Some((Duration::try_seconds(-1).unwrap() + Duration::try_minutes(-2).unwrap() + Duration::try_days(-3).unwrap() + Duration::try_weeks(-4).unwrap(), NOW)))]
fn test_string_wdhms_to_duration(
    input: String,
    expect: Option<(Duration, DUR_OFFSET_TYPE)>,
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
fn test_unescape_str(
    input: &str,
    expect: Option<&str>,
) {
    let result = unescape::unescape_str(input);
    match (result, expect) {
        (Ok(actual_s), Some(expect_s)) => {
            assert_eq!(actual_s, expect_s, "\nExpected {:?}\nActual   {:?}\n", expect_s, actual_s);
        }
        (Ok(actual_s), None) => {
            panic!("Expected Error, got {:?}", actual_s);
        }
        (Err(err), Some(_)) => {
            panic!("Got Error {:?}", err);
        }
        (Err(_), None) => {}
    }
}
