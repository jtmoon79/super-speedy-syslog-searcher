// src/data/datetime.rs
//
// the most relevant documents to understand this file:
// `chrono` crate `strftime` format:
// https://docs.rs/chrono/latest/chrono/format/strftime/index.html
// `regex` crate patterns
// https://docs.rs/regex/latest/regex/
//

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

#[cfg(any(debug_assertions,test))]
use crate::printer_debug::printers::{
    buffer_to_String_noraw,
    str_to_String_noraw,
};

#[allow(unused_imports)]
use crate::printer_debug::printers::{
    dpo,
    dpn,
    dpx,
    dpnx,
    dpof,
    dpnf,
    dpxf,
    dpnxf,
};

pub use crate::data::line::{
    LineIndex,
    Range_LineIndex,
};

use std::collections::BTreeMap;
use std::fmt;
pub use std::time::SystemTime;

extern crate arrayref;
use arrayref::array_ref;

extern crate chrono;
pub use chrono::{
    Date,
    DateTime,
    Datelike,  // adds method `.year()` onto `DateTime`
    Duration,
    FixedOffset,
    Local,
    LocalResult,
    NaiveDateTime,
    NaiveTime,
    Offset,
    TimeZone,
    Utc,
};

extern crate const_format;
use const_format::concatcp;

extern crate const_str;
use const_str::to_byte_array;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate more_asserts;
use more_asserts::{
    assert_ge,
    assert_le,
    assert_lt,
    debug_assert_ge,
    debug_assert_le,
    debug_assert_lt,
};

extern crate regex;
use regex::bytes::Regex;

extern crate unroll;
use unroll::unroll_for_loops;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DateTime Regex Matching and strftime formatting
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub type Year = i32;
/// crate `chrono` `strftime` formatting pattern, passed to `chrono::datetime_from_str`.
/// Specific `const` instances of `DateTimePattern_str` are hardcoded in
/// `fn captures_to_buffer_bytes`.
pub type DateTimePattern_str = str;
/// regular expression formatting pattern, passed to `regex::bytes::Regex`
pub type DateTimeRegex_str = str;
/// regular expression capture group name, used within the regular expression and
/// for later retreival via `regex::captures.name()`
pub type CaptureGroupName = str;
/// regular expression capture group pattern, used within the regular expression
pub type CaptureGroupPattern = str;
/// a regular expression
pub type RegexPattern = str;
/// the regular expression "class" used here, specifically for matching datetime substrings within
/// a `&str`
pub type DateTimeRegex = Regex;
/// the chrono DateTime type used here
// TODO: rename to `DateTimeS4`
pub type DateTimeL = DateTime<FixedOffset>;
pub type DateTimeLOpt = Option<DateTimeL>;

/// for datetimes missing a year, in some circumstances a filler year must be used.
///
/// first leap year after Unix Epoch.
///
/// XXX: using leap year as a filler might help handle 'Feb 29' dates without a year
///      but it is not guaranteed. It depends on the file modified time
///      (i.e. `blockreader.mtime()`) being true
const YEAR_FALLBACKDUMMY: &str = "1972";

const DATETIME_FALLBACKDUMMY: &str = "19720229T120000";

/// DateTime Format Specifier for Year
/// follows chrono `strftime` formatting
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Year {
    /// %Y
    Y,
    /// %y
    y,
    /// none provided, must be filled
    /// the associated `pattern` should use "%Y`
    _fill,
}

/// DateTime Format Specifier for Month
/// follows chrono `strftime` formatting
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Month {
    /// %m
    m,
    /// %b
    b,
    /// %B - transformed in `fn captures_to_buffer_bytes`
    B,
}

/// DateTime Format Specifier for Day
/// follows chrono `strftime` formatting
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Day {
    /// %d
    d,
    /// %e
    e,
    /// %d (" 8" or "08") captured but must be changed to %d ("08")
    _e_to_d,
}

/// DateTime Format Specifier for Hour
/// follows chrono `strftime` formatting
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Hour {
    /// %H
    H,
    /// %k
    k,
    /// %I
    I,
    /// %l
    l,
}

/// DateTime Format Specifier for Minute
/// follows chrono `strftime` formatting
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Minute {
    /// %M
    M,
}

/// DateTime Format Specifier for Minute
/// follows chrono `strftime` formatting
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Second {
    /// %S
    S,
}

/// DateTime Format Specifier for Fractional or fractional second
/// follows chrono `strftime` formatting
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Fractional {
    /// %f
    f,
    /// none, will not be filled
    _none,
}

/// DateTime Format Specifier for Timezone
/// follows chrono `strftime` formatting
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Tz {
    /// %z +0930
    z,
    /// %:z +09:30
    cz,
    /// %#z +09
    pz,
    /// %Z PST
    Z,
    /// none, must be filled
    /// the associated `pattern` should use "%:z` as that is the form displayed
    /// by `chrono::FixedOffset::east(0).as_string().to_str()`
    _fill,
}

/// `DTFSSet` is essentially instructions to transcribe regex captures to a chrono `strftime`-ready
/// string. Given extracted regular expression groups "year", "day", etc. (see `CGN_*` vars),
/// then what is the format of each such that the data can be readied and then passed
/// to `chrono::DateTime::parse_from_str` (strftime format)?
///
/// Strictly, there are 192 permutations. In practice, only a subset is encountered in real-life
/// syslog files.
/// Furthermore, some regex capture data is modified to be only one type. For example,
/// capture group "day" will capture patterns for %e (" 8") and %d ("08"). The captured data will be
/// modified to strftime day format "%d", e.g. data " 8" ("%e") becomes "08" ("%d").
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct DTFSSet<'a> {
    pub year: DTFS_Year,
    pub month: DTFS_Month,
    pub day: DTFS_Day,
    pub hour: DTFS_Hour,
    pub minute: DTFS_Minute,
    pub second: DTFS_Second,
    pub fractional: DTFS_Fractional,
    pub tz: DTFS_Tz,
    /// strftime pattern passed to `chrono::DateTime::parse_from_str` or `chrono::NaiveDateTime::parse_from_str`
    /// in function `datetime_parse_from_str`. Directly relates to order of capture group extractions and `push_str`
    /// done in `captures_to_buffer_bytes`.
    ///
    /// `pattern` is interdependent with other members. Tested in `test_DATETIME_PARSE_DATAS_builtin`.
    pub pattern: &'a DateTimePattern_str,
}

impl DTFSSet<'_> {
    pub fn has_year(&self) -> bool {
        match self.year {
            DTFS_Year::Y | DTFS_Year::y => true,
            DTFS_Year::_fill => false,
        }
    }
    pub fn has_tz(&self) -> bool {
        match self.tz {
            DTFS_Tz::z | DTFS_Tz::cz | DTFS_Tz::pz | DTFS_Tz::Z => true,
            DTFS_Tz::_fill => false,
        }
    }
}


/// `Instr`uctions for `pars`ing from an unknown `str` to
/// `regex::Regex().captures()` instance to `fn chrono::DateTime::parse_from_str`.
///
/// The settings is entirely interdependent. Tested in `test_DATETIME_PARSE_DATAS_builtin`.
#[derive(Hash)]
pub struct DateTimeParseInstr<'a> {
    // regex pattern for `captures`
    pub regex_pattern: &'a DateTimeRegex_str,
    /// in what strftime form are the regex `regex_pattern` capture groups?
    pub dtfs: DTFSSet<'a>,
    /// slice range of widest regex pattern match
    ///
    /// This is range is sliced from the `Line` and then a `Regex` match is attempted using it.
    /// It must be at least contain the datetime string to match. It may contain extra characters
    /// before or after the datetime (assuming the `regex_pattern` is correct).
    pub range_regex: Range_LineIndex,
    /// capture named group first (left-most) position in regex
    pub cgn_first: &'a CaptureGroupName,
    /// capture named group last (right-most) position in regex
    pub cgn_last: &'a CaptureGroupName,
    /// hardcoded self-test cases
    #[cfg(any(debug_assertions,test))]
    pub _test_cases: &'a [&'a str],
    /// line number of declaration, to aid debugging
    pub _line_num: u32,
}

/// declare a `DateTimeParseInstr` tuple more easily
#[macro_export]
macro_rules! DTPD {
    (
        $dtr:expr,
        $dtfs:expr,
        $sib:literal,
        $sie:literal,
        $cgn_first:ident,
        $cgn_last:ident,
        $test_cases:expr,
        $line_num:expr,
    ) => {
        DateTimeParseInstr {
            regex_pattern: $dtr,
            dtfs: $dtfs,
            range_regex: Range_LineIndex { start: $sib, end: $sie },
            cgn_first: $cgn_first,
            cgn_last: $cgn_last,
            #[cfg(any(debug_assertions,test))]
            _test_cases: $test_cases,
            _line_num: $line_num,
        }
    }
}
// allow easy macro import via `use s4lib::data::datetime::DTPD;`
pub use DTPD;

// implement ordering traits to allow sorting collections of `DateTimeParseInstr`
// only used for tests

impl Ord for DateTimeParseInstr<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            self.regex_pattern,
            &self.dtfs,
        ).cmp(
            &(
                other.regex_pattern,
                &other.dtfs,
            )
        )
    }
}

impl PartialOrd for DateTimeParseInstr<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DateTimeParseInstr<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.regex_pattern == other.regex_pattern
        && self.dtfs == other.dtfs
    }
}

impl Eq for DateTimeParseInstr<'_> {}

impl fmt::Debug for DateTimeParseInstr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // regexp strings can be very long, truncate it
        const MAXLEN: usize = 20;
        let mut rp: String = String::with_capacity(MAXLEN + 5);
        rp.extend(self.regex_pattern.chars().take(MAXLEN));
        if self.regex_pattern.len() > MAXLEN {
            rp.push('…');
        }
        let mut f_ = f.debug_struct("DateTimeParseInstr:");
        f_.field("regex_pattern", &rp)
            .field("range_regex", &self.range_regex)
            .field("dtfs", &self.dtfs)
            .field("cgn_first", &self.cgn_first)
            .field("cgn_last", &self.cgn_last)
            .field("cgn_last", &self.cgn_last);
        //if cfg!(debug_assertions) || cfg!(test) {
            f_.field("line", &self._line_num);
        //}

        f_.finish()
    }

}

// `strftime` patterns used in `DTPD!` declarations

const DTP_YmdHMSzc: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
const DTP_YmdHMSz: &DateTimePattern_str = "%Y%m%dT%H%M%S%z";
const DTP_YmdHMSzp: &DateTimePattern_str = "%Y%m%dT%H%M%S%#z";
const DTP_YmdHMSfzc: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%:z";
const DTP_YmdHMSfz: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%z";
const DTP_YmdHMSfzp: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%#z";

/// `%Z` is mapped to %z by `captures_to_buffer_bytes`
const DTP_YmdHMSfZ: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%z";

const DTP_YbdHMSz: &DateTimePattern_str = "%Y%b%dT%H%M%S%z";
const DTP_YbdHMScz: &DateTimePattern_str = "%Y%b%dT%H%M%S%:z";
const DTP_YBdHMSz: &DateTimePattern_str = "%Y%B%dT%H%M%S%z";
/// `%:z` is filled by `captures_to_buffer_bytes`
const DTP_YbdHMS: &DateTimePattern_str = "%Y%b%dT%H%M%S%:z";
/// `%:z` is filled by `captures_to_buffer_bytes`
const DTP_YBdHMS: &DateTimePattern_str = "%Y%B%dT%H%M%S%:z";
/// `%:z` is filled by `captures_to_buffer_bytes`
const DTP_YbeHMS: &DateTimePattern_str = "%Y%b%eT%H%M%S%:z";
/// `%:z` is filled by `captures_to_buffer_bytes`
const DTP_YBeHMS: &DateTimePattern_str = "%Y%B%eT%H%M%S%:z";

/// `%Y` `%:z` is filled by `captures_to_buffer_bytes`
const DTP_beHMS: &DateTimePattern_str = "%Y%b%eT%H%M%S%:z";

/// `%Y` `%:z` is filled, `%B` value transformed to `%m` value by `captures_to_buffer_bytes`
const DTP_BdHMS: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
/// `%Y` is filled, `%Z` tranformed to `%:z`, `%B` value transformed to `%m` value by `captures_to_buffer_bytes`
const DTP_BdHMSZ: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
/// `%:z` is filled, `%B` value transformed to `%m` value by `captures_to_buffer_bytes`
const DTP_BdHMSY: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
///  `%Z` tranformed to `%:z`, `%B` value transformed to `%m` value by `captures_to_buffer_bytes`
const DTP_BdHMSYZ: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
/// `%Y` `%:z` is filled, `%B` value transformed to `%m` value by `captures_to_buffer_bytes`
const DTP_BeHMS: &DateTimePattern_str = "%Y%m%eT%H%M%S%:z";
/// `%Y` is filled, `%Z` tranformed to `%:z`, `%B` value transformed to `%m` value by `captures_to_buffer_bytes`
const DTP_BeHMSZ: &DateTimePattern_str = "%Y%m%eT%H%M%S%:z";
/// `%:z` is filled, `%B` value transformed to `%m` value by `captures_to_buffer_bytes`
const DTP_BeHMSY: &DateTimePattern_str = "%Y%m%eT%H%M%S%:z";
///  `%Z` tranformed to `%:z`, `%B` value transformed to `%m` value by `captures_to_buffer_bytes`
const DTP_BeHMSYZ: &DateTimePattern_str = "%Y%m%eT%H%M%S%:z";
/// `%Y` `%:z` is filled, `%B` value transformed to `%m` value by `captures_to_buffer_bytes`
const DTP_bdHMS: &DateTimePattern_str = "%Y%b%dT%H%M%S%:z";

// chrono `strftime` formatting strings used in `fn datetime_parse_from_str`.
// `DTF` is "DateTime Format"
//
// These are effectively mappings to receive extracting datetime substrings in a `&str`
// then to rearrange those into order suitable for `fn captures_to_buffer_bytes`.
//
// The variable name represents what is available. The value represents it's rearranged form
// using in `fn captures_to_buffer_bytes`.

pub(crate) const DTFSS_YmdHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    pattern: DTP_YmdHMSzc,
};
pub(crate) const DTFSS_YmdHMSz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::z,
    pattern: DTP_YmdHMSz,
};
pub(crate) const DTFSS_YmdHMScz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::cz,
    pattern: DTP_YmdHMSzc,
};
pub(crate) const DTFSS_YmdHMSpz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::pz,
    pattern: DTP_YmdHMSzp,
};
pub(crate) const DTFSS_YmdHMSZ: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::Z,
    pattern: DTP_YmdHMSz,
};

const DTFSS_YmdHMSf: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::_fill,
    pattern: DTP_YmdHMSfzc,
};
const DTFSS_YmdHMSfz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::z,
    pattern: DTP_YmdHMSfz,
};
const DTFSS_YmdHMSfcz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::cz,
    pattern: DTP_YmdHMSfzc,
};
const DTFSS_YmdHMSfpz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::pz,
    pattern: DTP_YmdHMSfzp,
};
const DTFSS_YmdHMSfZ: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::Z,
    pattern: DTP_YmdHMSfzc,
};

const DTFSS_BdHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::_fill,
    month: DTFS_Month::B,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    pattern: DTP_BdHMS,
};
const DTFSS_BdHMSZ: DTFSSet = DTFSSet {
    year: DTFS_Year::_fill,
    month: DTFS_Month::B,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::Z,
    pattern: DTP_BdHMSZ,
};
const DTFSS_BdHMSY: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::B,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    pattern: DTP_BdHMSY,
};
const DTFSS_BdHMSYZ: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::B,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::Z,
    pattern: DTP_BdHMSYZ,
};

const DTFSS_BeHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::_fill,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_to_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    pattern: DTP_BdHMS,
};
const DTFSS_BeHMSZ: DTFSSet = DTFSSet {
    year: DTFS_Year::_fill,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_to_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::Z,
    pattern: DTP_BdHMSZ,
};
const DTFSS_BeHMSY: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_to_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    pattern: DTP_BdHMSY,
};
const DTFSS_BeHMSYZ: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_to_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::Z,
    pattern: DTP_BdHMSYZ,
};

/// specail case: `dmesg` syslog lines
pub(crate) const DTFSS_u: DTFSSet = DTFSSet {
    year: DTFS_Year::_fill,
    month: DTFS_Month::m,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    pattern: DTP_YmdHMSzc,
};


/// to aid testing
#[cfg(any(debug_assertions,test))]
pub(crate) const _DTF_ALL: &[&DateTimePattern_str] = &[
    //DTP_YmdHMS,
    //DTP_YmdHMSz,
    //DTP_YmdHMScz,
    //DTP_YmdHMSpz,
    //DTP_YmdHMSf,
    //DTP_YmdHMSfz,
    //DTP_YmdHMSfcz,
    //DTP_YmdHMSfpz,
    DTP_YmdHMSfZ,
    DTP_YbdHMSz,
    DTP_YbdHMScz,
    DTP_YBdHMSz,
    DTP_YbdHMS,
    DTP_YBdHMS,
    DTP_YbeHMS,
    DTP_YBeHMS,
    DTP_BdHMS,
    DTP_BeHMS,
    DTP_bdHMS,
    DTP_beHMS,
];

// `regex::Captures` capture group names

/// corresponds to strftime `%Y`
const CGN_YEAR: &CaptureGroupName = "year";
/// corresponds to strftime `%m`
const CGN_MONTH: &CaptureGroupName = "month";
/// corresponds to strftime `%d`
const CGN_DAY: &CaptureGroupName = "day";
/// corresponds to strftime `%H`
const CGN_HOUR: &CaptureGroupName = "hour";
/// corresponds to strftime `%M`
const CGN_MINUTE: &CaptureGroupName = "minute";
/// corresponds to strftime `%S`
const CGN_SECOND: &CaptureGroupName = "second";
/// corresponds to strftime `%f`
const CGN_FRACTIONAL: &CaptureGroupName = "fractional";
/// corresponds to strftime `%Z` `%z` `%:z` `%#z`
const CGN_TZ: &CaptureGroupName = "tz";
// special case: `dmesg` uptime
//const CGN_UPTIME: &CaptureGroupName = "uptime";

/// all capture group names, for testing
#[cfg(any(debug_assertions,test))]
pub(crate) const _CGN_ALL: [&CaptureGroupName; 8] = [
    CGN_YEAR,
    CGN_MONTH,
    CGN_DAY,
    CGN_HOUR,
    CGN_MINUTE,
    CGN_SECOND,
    CGN_FRACTIONAL,
    CGN_TZ,
    //CGN_UPTIME,
];

// saved rust playground for quick testing patterns
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=00460112beb2a6d078d6bbba72557574

/// regex capture group pattern for strftime year
pub const CGP_YEAR: &CaptureGroupPattern = r"(?P<year>[12]\d{3})";
/// regex capture group pattern for strftime month `%m`
pub const CGP_MONTHm: &CaptureGroupPattern = r"(?P<month>01|02|03|04|05|06|07|08|09|10|11|12)";
/// regex capture group pattern for strftime month `%b`
pub const CGP_MONTHb: &CaptureGroupPattern = r"(?P<month>(?i)Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec(?-i))";
/// regex capture group pattern for strftime month `%B`
pub const CGP_MONTHB: &CaptureGroupPattern = r"(?P<month>(?i)January|February|March|April|May|June|July|August|September|October|November|December(?-i))";
/// regex capture group pattern for strftime month `%B` and `%b`
pub const CGP_MONTHBb: &CaptureGroupPattern = r"(?P<month>(?i)January|Jan|February|Feb|March|Mar|April|Apr|May|June|Jun|July|Jul|August|Aug|September|Sep|October|Oct|November|Nov|December|Dec(?-i))";
/// regex capture group pattern for strftime day `%d`
pub const CGP_DAYd: &CaptureGroupPattern = r"(?P<day>01|02|03|04|05|06|07|08|09|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31)";
/// regex capture group pattern for strftime day `%e`
pub const CGP_DAYe: &CaptureGroupPattern = r"(?P<day>1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31)";
/// regex capture group pattern for strftime hour `%H`
pub const CGP_HOUR: &CaptureGroupPattern = r"(?P<hour>00|01|02|03|04|05|06|07|08|09|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24)";
/// regex capture group pattern for strftime minute `%M`
pub const CGP_MINUTE: &CaptureGroupPattern = r"(?P<minute>[012345]\d)";
/// regex capture group pattern for strftime second `%S`, includes leap second "60"
pub const CGP_SECOND: &CaptureGroupPattern = r"(?P<second>[012345]\d|60)";
/// regex capture group pattern for strftime fractional `%f`,
/// all strftime patterns %f %3f %6f %9f
pub const CGP_FRACTIONAL: &CaptureGroupPattern = r"(?P<fractional>\d{3,9})";
// uptime fractional seconds, seen in `dmesg` logs
//pub const CGP_UPTIME: &CaptureGroupPattern = r"(?P<uptime>\d{1,5}\.\d{3,9})";

/// for help in testing only
#[cfg(any(debug_assertions,test))]
pub const _CGP_MONTH_ALL: &[&CaptureGroupPattern] = &[
    CGP_MONTHm,
    CGP_MONTHb,
    CGP_MONTHB,
    CGP_MONTHBb,
];

/// for help in testing only
#[cfg(any(debug_assertions,test))]
pub const _CGP_DAY_ALL: &[&CaptureGroupPattern] = &[
    CGP_DAYd,
    CGP_DAYe,
];
// Applicable tz offsets https://en.wikipedia.org/wiki/List_of_UTC_offsets
// Applicable tz abbreviations https://en.wikipedia.org/wiki/List_of_time_zone_abbreviations
// chrono strftime https://docs.rs/chrono/latest/chrono/format/strftime/index.html
//
/// strftime `%z` e.g. `+0930`
const CGP_TZz: &CaptureGroupPattern = r"(?P<tz>[\+\-][01]\d{3})";
/// strftime `%:z` e.g. `+09:30`
const CGP_TZcz: &CaptureGroupPattern = r"(?P<tz>[\+\-][01]\d:\d\d)";
/// strftime `%#z` e.g. `+09`
const CGP_TZpz: &CaptureGroupPattern = r"(?P<tz>[\+\-][01]\d)";
/// strftime `%Z` e.g. `ACST`
const CGP_TZZ: &CaptureGroupPattern = r"(?P<tz>ACDT|ACST|ACT|ADT|AEDT|AEST|AET|AFT|AKDT|AKST|ALMT|AMST|AMT|ANAT|AQTT|ART|AST|AWST|AZOT|AZT|BIOT|BIT|BNT|BOT|BRST|BRT|BST|BTT|CAT|CCT|CDT|CEST|CET|CHOT|CHST|CHUT|CIST|CKT|CLST|CLT|COST|COT|CST|CT|CVT|CWST|CXT|DAVT|DDUT|DFT|EAST|EAT|ECT|EDT|EEST|EET|EGST|EGT|EST|ET|FET|FJT|FKST|FKT|FNT|GALT|GAMT|GET|GFT|GILT|GIT|GMT|GST|GYT|HAEC|HDT|HKT|HMT|HOVT|HST|ICT|IDLW|IDT|IOT|IRDT|IRKT|IRST|IST|JST|KALT|KGT|KOST|KRAT|KST|LHST|LINT|MAGT|MART|MAWT|MDT|MEST|MET|MHT|MIST|MIT|MMT|MSK|MST|MUT|MVT|MYT|NCT|NDT|NFT|NOVT|NPT|NST|NT|NUT|NZDT|NZST|OMST|ORAT|PDT|PET|PETT|PGT|PHOT|PHST|PHT|PKT|PMDT|PMST|PONT|PST|PWT|PYST|PYT|RET|ROTT|SAKT|SAMT|SAST|SBT|SCT|SDT|SGT|SLST|SRET|SRT|SST|SYOT|TAHT|TFT|THA|TJT|TKT|TLT|TMT|TOT|TRT|TVT|ULAT|UTC|UYST|UYT|UZT|VET|VLAT|VOLT|VOST|VUT|WAKT|WAST|WAT|WEST|WET|WGST|WGT|WIB|WIT|WITA|WST|YAKT|YEKT)";
/// for help in testing only
#[cfg(any(debug_assertions,test))]
pub const _CGP_TZ_ALL: &[&CaptureGroupPattern] = &[
    CGP_TZz,
    CGP_TZcz,
    CGP_TZpz,
    CGP_TZZ,
];

/// no uppercase, helper to `CGP_TZZ`
const RP_NOUPPER: &RegexPattern = r"([^[[:upper:]]]|$)";
/// timezone abbreviation, not followed by uppercase
const CGP_TZZn: &CaptureGroupPattern = concatcp!(CGP_TZZ, RP_NOUPPER);

const NOENTRY: &str = "";

/// all timezone abbreviations, maps all strftime "%Z" to strftime "%:z".
///
/// attempts to be more lenient than chrono
/// https://docs.rs/chrono/latest/chrono/format/strftime/#fn7
///
///
/// latest listing of timezone abbreviations can be retrieved by:
///
/// ```no_run
/// $ curl "https://en.wikipedia.org/wiki/List_of_time_zone_abbreviations" \
///     | grep -Ee '^<td>[[:upper:]]{2,4}</td>' \
///     | grep -oEe '[[:upper:]]{2,4}' \
///     | sort \
///     | uniq \
///     | sed -Ee ':a;N;$!ba;s/\n/|/g'
///
/// $ curl "https://en.wikipedia.org/wiki/List_of_time_zone_abbreviations" \
///     | rg -or '$1 $2' -e '^<td>([[:upper:]]{2,5})</td>' -e '^<td data-sort-value.*>UTC(.*)</a>' \
///     | sed -e '/^$/d' \
///     | rg -r '("$1", ' -e '^([[:upper:]]{2,5})' -C5 \
///     | rg -r '"$1"), ' -e '^[[:blank:]]*([[:print:]−±+]*[[:digit:]]{1,4}.*$)' -C5 \
///     | rg -r '"$1:00"' -e '"(.?[[:digit:]][[:digit:]])"' -C5 \
///     | sed -e 's/\n"/"/g' -e 'N;s/\n/ /' -e 's/−/-/g' -e 's/±/-/g' \
///     | tr -s ' '
/// ```

pub const TZZ_ALL: [(&str, &str); 208] = [
    ("ACDT", "+10:30"),
    ("ACST", "+09:30"),
    ("ACT", "-05:00"),
    ("ACT", "+08:00"),
    ("ACWST", "+08:45"),
    ("ADT", "-03:00"),
    ("AEDT", "+11:00"),
    ("AEST", "+10:00"),
    ("AET", "+11:00"),
    ("AFT", "+04:30"),
    ("AKDT", "-08:00"),
    ("AKST", "-09:00"),
    ("ALMT", "+06:00"),
    ("AMST", "-03:00"),
    ("AMT", "-04:00"),
    ("AMT", "+04:00"),
    ("ANAT", "+12:00"),
    ("AQTT", "+05:00"),
    ("ART", "-03:00"),
    ("AST", "+03:00"),
    ("AST", "-04:00"),
    ("AWST", "+08:00"),
    ("AZOST", "-00:00"),
    ("AZOT", "-01:00"),
    ("AZT", "+04:00"),
    ("BNT", "+08:00"),
    ("BIOT", "+06:00"),
    ("BIT", "-12:00"),
    ("BOT", "-04:00"),
    ("BRST", "-02:00"),
    ("BRT", "-03:00"),
    ("BST", "+06:00"),
    ("BST", "+11:00"),
    ("BST", "+01:00"),
    ("BTT", "+06:00"),
    ("CAT", "+02:00"),
    ("CCT", "+06:30"),
    ("CDT", "-05:00"),
    ("CDT", "-04:00"),
    ("CEST", "+02:00"),
    ("CET", "+01:00"),
    ("CHADT", "+13:45"),
    ("CHAST", "+12:45"),
    ("CHOT", "+08:00"),
    ("CHOST", "+09:00"),
    ("CHST", "+10:00"),
    ("CHUT", "+10:00"),
    ("CIST", "-08:00"),
    ("CKT", "-10:00"),
    ("CLST", "-03:00"),
    ("CLT", "-04:00"),
    ("COST", "-04:00"),
    ("COT", "-05:00"),
    ("CST", "-06:00"),
    ("CST", "+08:00"),
    ("CST", "-05:00"),
    ("CT", "-05:00"),
    ("CVT", "-01:00"),
    ("CWST", "+08:45"),
    ("CXT", "+07:00"),
    ("DAVT", "+07:00"),
    ("DDUT", "+10:00"),
    ("DFT", "+01:00"),
    ("EASST", "-05:00"),
    ("EAST", "-06:00"),
    ("EAT", "+03:00"),
    ("ECT", "-04:00"),
    ("ECT", "-05:00"),
    ("EDT", "-04:00"),
    ("EEST", "+03:00"),
    ("EET", "+02:00"),
    ("EGST", "-00:00"),
    ("EGT", "-01:00"),
    ("EST", "-05:00"),
    ("ET", "-04:00"),
    ("FET", "+03:00"),
    ("FJT", "+12:00"),
    ("FKST", "-03:00"),
    ("FKT", "-04:00"),
    ("FNT", "-02:00"),
    ("GALT", "-06:00"),
    ("GAMT", "-09:00"),
    ("GET", "+04:00"),
    ("GFT", "-03:00"),
    ("GILT", "+12:00"),
    ("GIT", "-09:00"),
    ("GMT", "-00:00"),
    ("GST", "-02:00"),
    ("GST", "+04:00"),
    ("GYT", "-04:00"),
    ("HDT", "-09:00"),
    ("HAEC", "+02:00"),
    ("HST", "-10:00"),
    ("HKT", "+08:00"),
    ("HMT", "+05:00"),
    ("HOVST", "+08:00"),
    ("HOVT", "+07:00"),
    ("ICT", "+07:00"),
    ("IDLW", "-12:00"),
    ("IDT", "+03:00"),
    ("IOT", "+03:00"),
    ("IRDT", "+04:30"),
    ("IRKT", "+08:00"),
    ("IRST", "+03:30"),
    ("IST", "+05:30"),
    ("IST", "+01:00"),
    ("IST", "+02:00"),
    ("JST", "+09:00"),
    ("KALT", "+02:00"),
    ("KGT", "+06:00"),
    ("KOST", "+11:00"),
    ("KRAT", "+07:00"),
    ("KST", "+09:00"),
    ("LHST", "+10:30"),
    ("LHST", "+11:00"),
    ("LINT", "+14:00"),
    ("MAGT", "+12:00"),
    ("MART", "-09:30"),
    ("MAWT", "+05:00"),
    ("MDT", "-06:00"),
    ("MET", "+01:00"),
    ("MEST", "+02:00"),
    ("MHT", "+12:00"),
    ("MIST", "+11:00"),
    ("MIT", "-09:30"),
    ("MMT", "+06:30"),
    ("MSK", "+03:00"),
    ("MST", "+08:00"),
    ("MST", "-07:00"),
    ("MUT", "+04:00"),
    ("MVT", "+05:00"),
    ("MYT", "+08:00"),
    ("NCT", "+11:00"),
    ("NDT", "-02:30"),
    ("NFT", "+11:00"),
    ("NOVT", "+07:00"),
    ("NPT", "+05:45"),
    ("NST", "-03:30"),
    ("NT", "-03:30"),
    ("NUT", "-11:00"),
    ("NZDT", "+13:00"),
    ("NZST", "+12:00"),
    ("OMST", "+06:00"),
    ("ORAT", "+05:00"),
    ("PDT", "-07:00"),
    ("PET", "-05:00"),
    ("PETT", "+12:00"),
    ("PGT", "+10:00"),
    ("PHOT", "+13:00"),
    ("PHT", "+08:00"),
    ("PHST", "+08:00"),
    ("PKT", "+05:00"),
    ("PMDT", "-02:00"),
    ("PMST", "-03:00"),
    ("PONT", "+11:00"),
    ("PST", "-08:00"),
    ("PWT", "+09:00"),
    ("PYST", "-03:00"),
    ("PYT", "-04:00"),
    ("RET", "+04:00"),
    ("ROTT", "-03:00"),
    ("SAKT", "+11:00"),
    ("SAMT", "+04:00"),
    ("SAST", "+02:00"),
    ("SBT", "+11:00"),
    ("SCT", "+04:00"),
    ("SDT", "-10:00"),
    ("SGT", "+08:00"),
    ("SLST", "+05:30"),
    ("SRET", "+11:00"),
    ("SRT", "-03:00"),
    ("SST", "-11:00"),
    ("SST", "+08:00"),
    ("SYOT", "+03:00"),
    ("TAHT", "-10:00"),
    ("THA", "+07:00"),
    ("TFT", "+05:00"),
    ("TJT", "+05:00"),
    ("TKT", "+13:00"),
    ("TLT", "+09:00"),
    ("TMT", "+05:00"),
    ("TRT", "+03:00"),
    ("TOT", "+13:00"),
    ("TVT", "+12:00"),
    ("ULAST", "+09:00"),
    ("ULAT", "+08:00"),
    ("UTC", "-00:00"),
    ("UYST", "-02:00"),
    ("UYT", "-03:00"),
    ("UZT", "+05:00"),
    ("VET", "-04:00"),
    ("VLAT", "+10:00"),
    ("VOLT", "+03:00"),
    ("VOST", "+06:00"),
    ("VUT", "+11:00"),
    ("WAKT", "+12:00"),
    ("WAST", "+02:00"),
    ("WAT", "+01:00"),
    ("WEST", "+01:00"),
    ("WET", "-00:00"),
    ("WIB", "+07:00"),
    ("WIT", "+09:00"),
    ("WITA", "+08:00"),
    ("WGST", "-02:00"),
    ("WGT", "-03:00"),
    ("WST", "+08:00"),
    ("YAKT", "+09:00"),
    ("YEKT", "+05:00"),
];
type Map_TZZ_to_TZz<'a> = BTreeMap<&'a str, &'a str>;
lazy_static!{
    static ref MAP_TZZ_TO_TZz: Map_TZZ_to_TZz<'static> = {
        let mut map_ = Map_TZZ_to_TZz::new();
        #[allow(non_snake_case)]
        for tzZ in TZZ_ALL.iter() {
            if let Some(_) = map_.insert(tzZ.0, tzZ.1) {
                // duplicate key entries are set to blank
                map_.insert(tzZ.0, NOENTRY);
            }
        }
        map_
    };
}

/// regexp divider date, 2020/01/01
const D_D: &RegexPattern = r"[ /\-]?";
/// regexp divider time, 20:30:00
const D_T: &RegexPattern = r"[:]?";
/// regexp divider day hour, 2020/01/01T20:30:00
const D_DH: &RegexPattern = r"[ T]?";
/// regexp divider day hour with colon, 2020:01:01:20:30:00
const _D_DHc: &RegexPattern = r"[ T:]?";
/// regexp divider day hour with dash, 2020:01:01-20:30:00
const _D_DHd: &RegexPattern = r"[ T\-]?";
/// regexp divider day hour with colon or dash, 2020:01:01-20:30:00
const D_DHcd: &RegexPattern = r"[ T\-:]?";
/// regexp divider fractional, 2020/01/01T20:30:00,123456
const D_SF: &RegexPattern = r"[\.,]";

/// commonly found syslog level names
const _RP_LEVELS: &RegexPattern = r"((?i)DEBUG|INFO|ERR|ERROR|TRACE|WARN|WARNING|VERBOSE(?-i))";
/// regex blank
const RP_BLANK: &RegexPattern = r"[[:blank:]]";
/// regex blank?
const RP_BLANKq: &RegexPattern = r"[[:blank:]]?";
/// regex blanks
const RP_BLANKS: &RegexPattern = r"[[:blank:]]+";
/// regex blanks?
const RP_BLANKSq: &RegexPattern = r"[[:blank:]]*";
/// regex blank or line end?
const RP_BLANKSqEnd: &RegexPattern = r"([[:blank:]]?|$)";
/// left-side brackets
pub(crate) const RP_LB: &RegexPattern = r"[\[\(<{]";
/// right-side brackets
pub(crate) const RP_RB: &RegexPattern = r"[\]\)>}]";

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// the global list of built-in Datetime parsing "instructions"
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// index into the global `DATETIME_PARSE_DATAS`.
pub type DateTimeParseInstrsIndex = usize;
/// a run-time created vector of `Regex` instances that is a counterpart to `DATETIME_PARSE_DATAS`.
pub type DateTimeParseInstrsRegexVec = Vec<DateTimeRegex>;

pub const DATETIME_PARSE_DATAS_LEN: usize = 36;

/// built-in `const DateTimeParseInstr` datetime parsing patterns.
///
/// These are all regexp patterns that will be attempted on processed files.
///
/// Order of declaration matters: during initial parsing of a syslog file, all of these
/// regex patterns are attempted. Listing a general regex pattern before a specific regex pattern
/// may result in loss of datetime information.
///
/// For example, given sysline
///
///     2001-02-03T04:05:06 -1100 hello
///
/// A regex that attempts to match the year to the second will not capture the timezone. This will
/// result in a filler timezone being used which may not be correct.
/// Generally, more specific regex patterns should be listed before general regex patterns.
///
/// Notice the "with timezone" versions of `DateTimeParseInstr` are often listed before the same
/// `DateTimeParseInstr` "without".
///
/// A drawback to specific-to-general approach: during `SyslineReader` initial reading stage, it
/// will try *all* the patterns (from index 0 to whereever it finds a match). So if a file has a
/// very general pattern (like it only matches the last listed `DateTimeParseInstr` here) then
/// the `SyslineReader` will try *all* the `DateTimeParseInstr` within `DATETIME_PARSE_DATAS`
/// several times (until `SyslineReader` is satisfied it has found the definitive pattern).
/// The many missed matches use a lot of resources and time.
///
pub const DATETIME_PARSE_DATAS: [DateTimeParseInstr; DATETIME_PARSE_DATAS_LEN] = [
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/Ubuntu18/xrdp.log`
    // example with offset:
    //               1
    //     01234567890123456789
    //     [20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
    //
    // from file `./logs/Ubuntu18/samba/log.10.1.1.2` (multi-line)
    // example with offset:
    //               1         2         3
    //     0123456789012345678901234567890
    //     [2020/03/05 12:17:59.631000,  3] ../source3/smbd/oplock.c:1340(init_oplocks)
    //        init_oplocks: initializing messages.
    //     [2000/01/01 00:00:04.123456] ../source3/smbd/oplock.c:1340(init_oplocks)
    //
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_RB),
        DTFSS_YmdHMSf, 0, 40, CGN_YEAR, CGN_FRACTIONAL,
        &["[2000/01/01 00:00:01.123] ../source3/smbd/oplock.c:1340(init_oplocks)"],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZz, RP_RB),
        DTFSS_YmdHMSfz, 0, 40, CGN_YEAR, CGN_TZ,
        &["(2000/01/01 00:00:02.123456 -1100) ../source3/smbd/oplock.c:1340(init_oplocks)"],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZcz, RP_RB),
        DTFSS_YmdHMSfcz, 0, 40, CGN_YEAR, CGN_TZ,
        &[r"{2000/01/01 00:00:03.123456789 -11:30} ../source3/smbd/oplock.c:1340(init_oplocks)"],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZpz, RP_RB),
        DTFSS_YmdHMSfpz, 0, 40, CGN_YEAR, CGN_TZ,
        &[r"(2000/01/01 00:00:04.123456789 -11) ../source3/smbd/oplock.c:1340(init_oplocks)"],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANK, CGP_TZZ, RP_RB),
        DTFSS_YmdHMSfZ, 0, 40, CGN_YEAR, CGN_TZ,
        &[r"(2000/01/01 00:00:05.123456789 VLAT) ../source3/smbd/oplock.c:1340(init_oplocks)"],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, r"[,\.\| \t]", RP_BLANKSq, r"[[:word:]]{1,20}", RP_RB),
        DTFSS_YmdHMSf, 0, 40, CGN_YEAR, CGN_FRACTIONAL,
        &["[2020/03/05 12:17:59.631000, FOO] ../source3/smbd/oplock.c:1340(init_oplocks)"],
        line!(),
    ),
    // from file `/var/log/unattended-upgrades/unattended-upgrades-dpkg.log`
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     Log started: 2022-07-14  06:48:58
    //     (Reading database ...
    //     Preparing to unpack .../linux-tools-common_5.15.0-41.44_all.deb ...
    //     Unpacking linux-tools-common (5.15.0-41.44) over (5.15.0-40.43) ...
    //     Setting up linux-tools-common (5.15.0-41.44) ...
    //     Processing triggers for man-db (2.10.2-1) ...
    //     NEEDRESTART-VER: 3.5
    //     NEEDRESTART-KCUR: 5.10.102.1-microsoft-standard-WSL2
    //     NEEDRESTART-KSTA: 0
    //     Log ended: 2022-07-14  06:49:02
    //
    DTPD!(
        concatcp!(r"^(Log started:|Log ended:)", RP_BLANKSq, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, r"[ T]*", CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND),
        DTFSS_YmdHMS, 0, 40, CGN_YEAR, CGN_SECOND,
        &["Log started: 2022-07-14  06:48:58\n(Reading database ..."],
        line!(),
    ),
    //
    // from file `./logs/Ubuntu18/vmware-installer.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2019-05-06 11:24:34,074] Successfully loaded GTK libraries.
    //
    // ---------------------------------------------------------------------------------------------
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     [ERROR] 2000-01-02 12:33:01 -1200 1
    //     [WARNING] 2000-01-02 12:33:02 -1130 22
    //     [INFO] 2000-01-02 12:33:03 +1100 333
    //     [VERBOSE] 2000-01-02T12:33:04 -1030 4444
    //     [TRACE] 2000-01-02T12:33:05 -1000 55555
    //
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/synology/usbcopyd.log`
    //
    // example with offset:
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     2017-05-24T19:14:38-07:00 hostname1 usb-copy-starter
    //
    // ---------------------------------------------------------------------------------------------
    //
    // prescripted datetime+tz
    //
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-02 12:33:05 -0400 foo
    //     2000-01-02 12:33:05 -04:00 foo
    //     2000-01-02T12:33:05 -0400 foo
    //     2000-01-02T12:33:05 -04:00 foo
    //
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-02 12:33:05,123 -0400 foo
    //     2000-01-02 12:33:05,123 -04:00 foo
    //     2000-01-02T12:33:05,123 -0400 foo
    //     2000-01-02T12:33:05,123 -04:00 foo
    //
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-02 12:33:05.123456 foo
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     2000-01-02 12:33:05 foo
    //     2000-01-02 12:33:05 foo
    //     2000-01-02T12:33:05 foo
    //     2000-01-02T12:33:05 foo
    //
    // ---------------------------------------------------------------------------------------------
    //               1         2         3
    //     0123456789012345678901234567890
    //     [ERROR] 2000-01-02 12:33:01 1
    //     [WARNING] 2000-01-02T12:33:02 22
    //     [INFO] 2000-01-02T12:33:03 333
    //     [VERBOSE] 2000-01-02 12:33:04 4444
    //     [TRACE] 2000-01-02 12:33:05 55555
    //
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/Ubuntu18/vmware/hostd-62.log`
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     2019-07-26T10:40:29.682-07:00 info hostd[03210] [Originator@6876 sub=Default] Current working directory: /usr/bin
    //
    DTPD!(
        concatcp!(r"^", CGP_MONTHBb, RP_BLANKS, CGP_DAYe, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZZn),
        DTFSS_BeHMSYZ, 0, 28, CGN_MONTH, CGN_TZ,
        &[
            "September  3 08:10:29 2000 PWT hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode",
            "Jan 1 01:00:00 2000 PWT 😀",
            "Jan 11 01:00:00 2000 PWT 😀",
            "Feb 29 01:00:00 2000 PWT 😀",
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_MONTHBb, RP_BLANKS, CGP_DAYd, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZZn),
        DTFSS_BdHMSYZ, 0, 28, CGN_MONTH, CGN_TZ,
        &[
            "September 03 08:10:29 2000 PWT hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode",
            "Jan 01 01:00:00 2000 PWT 😀",
            "Jan 11 01:00:00 2000 PWT 😀",
            "Feb 29 01:00:00 2000 PWT 😀",
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_MONTHBb, RP_BLANKS, CGP_DAYe, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR),
        DTFSS_BeHMSY, 0, 28, CGN_MONTH, CGN_YEAR,
        &[
            "September  3 08:10:29 2000 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode",
            "Jan 1 01:00:00 2000 😀",
            "Jan 11 01:00:00 2000 😀",
            "Feb 29 01:00:00 2000 😀",
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_MONTHBb, RP_BLANKS, CGP_DAYd, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR),
        DTFSS_BdHMSY, 0, 28, CGN_MONTH, CGN_YEAR,
        &[
            "September 03 08:10:29 2000 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode",
            "Jan 01 01:00:00 2000 😀",
            "Jan 11 01:00:00 2000 😀",
            "Feb 29 01:00:00 2000 😀",
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_MONTHBb, RP_BLANKS, CGP_DAYe, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_TZZn),
        DTFSS_BeHMSZ, 0, 28, CGN_MONTH, CGN_TZ,
        &[
            "September  3 08:10:29 PWT hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode",
            "Jan 1 01:00:00 PWT 😀",
            "Jan 11 01:00:00 PWT 😀",
            "Feb 29 01:00:00 PWT 😀",
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_MONTHBb, RP_BLANKS, CGP_DAYd, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_TZZn),
        DTFSS_BdHMSZ, 0, 28, CGN_MONTH, CGN_TZ,
        &[
            "September 03 08:10:29 PWT hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode",
            "Jan 01 01:00:00 PWT 😀",
            "Jan 11 01:00:00 PWT 😀",
            "Feb 29 01:00:00 PWT 😀",
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/Ubuntu18/kernel.log`, no year
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     Mar  9 08:10:29 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode
    //     Mar 09 08:10:29 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode
    //
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     Sep 03 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119
    //     September  3 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119
    //     September 3 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119
    //
    DTPD!(
        concatcp!(r"^", CGP_MONTHBb, RP_BLANKS, CGP_DAYe, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKSq),
        DTFSS_BeHMS, 0, 22, CGN_MONTH, CGN_SECOND,
        &[
            "September  3 08:10:29 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode",
            "Jan 1 01:00:00 1900 😀",
            "January  3 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119",
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_MONTHBb, RP_BLANKS, CGP_DAYd, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKSq),
        DTFSS_BdHMS, 0, 22, CGN_MONTH, CGN_SECOND,
        &[
            "September 03 08:10:29 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode",
            "Jan 01 01:00:00 1900 😀",
            "January 03 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119",
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    // from file `/var/log/aptitude`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     ===============================================================================
    //
    //     Aptitude 0.8.13: log report
    //     Tue, Jun 28 2022 01:51:12 +0000
    //
    //       IMPORTANT: this log only lists intended actions; actions which fail
    //       due to dpkg problems may not be completed.
    //
    //     Will install 1 packages, and remove 0 packages.
    //     4833 kB of disk space will be used
    //     ========================================
    //     [HOLD, DEPENDENCIES] libnss-systemd:amd64 249.11-0ubuntu3.1
    //     [HOLD, DEPENDENCIES] libpam-systemd:amd64 249.11-0ubuntu3.1
    //     [HOLD, DEPENDENCIES] libsystemd0:amd64 249.11-0ubuntu3.1
    //     [HOLD, DEPENDENCIES] libudev1:amd64 249.11-0ubuntu3.1
    //     [HOLD, DEPENDENCIES] systemd:amd64 249.11-0ubuntu3.1
    //     [HOLD, DEPENDENCIES] systemd-sysv:amd64 249.11-0ubuntu3.1
    //     [HOLD, DEPENDENCIES] systemd-timesyncd:amd64 249.11-0ubuntu3.1
    //     [HOLD, DEPENDENCIES] udev:amd64 249.11-0ubuntu3.1
    //     [INSTALL] p7zip-full:amd64 16.02+dfsg-8
    //     ========================================
    //
    //     Log complete.
    //
    //     ===============================================================================
    //
    // TODO: add DTPD for `aptitude` log report
    //
    // ---------------------------------------------------------------------------------------------
    // from file `/var/log/apt/history.log`
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     Start-Date: 2022-07-18  19:34:46
    //     Commandline: apt-get install -y gnupg2
    //     Install: gnupg2:amd64 (2.2.27-3ubuntu2.1)
    //     End-Date: 2022-07-18  19:35:04
    //     Start-Date: 2022-07-31  19:13:42
    //     Commandline: apt-get -qq install -y ca-certificates gnupg2 apt-utils apt-transport-https curl
    //     Install: apt-transport-https:amd64 (2.4.6)
    //     Upgrade: apt:amd64 (2.4.5, 2.4.6), libapt-pkg6.0:amd64 (2.4.5, 2.4.6), apt-utils:amd64 (2.4.5, 2.4.6)
    //
    DTPD!(
        concatcp!("^([SS][Tt][Aa][Rr][Tt]|[Ee][Nn][Dd])-[Dd][Aa][Tt][Ee]", "[:]?", RP_BLANKSq, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKSqEnd),
        DTFSS_YmdHMS, 0, 35, CGN_YEAR, CGN_SECOND,
        &[
            "Start-Date: 2022-07-18  19:34:01\nCommandline: apt-get install -y gnupg2\nInstall: gnupg2:amd64 (2.2.27-3ubuntu2.1)\n",
            "End-Date: 2022-07-18  19:35:02\n",
            "End-Date: 2022-07-18  19:35:03",
            "End-Date:2022-07-18  19:35:04",
            "End-Date:2022-07-18 19:35:05\n",
            "End-Date 2022-07-18 19:35:06\n",
            "END-DATE  2022-07-18 19:35:07 Foobar",
            "end-date 2022-07-18 19:35:08 Foobar",
            "START-DATE:   2022-07-18 19:35:09\nCommandline: apt-get install -y gnupg2\n",
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------    // from file `./logs/debian9/alternatives.log`
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890123456789
    //     update-alternatives 2020-02-03 13:56:07: run with --install /usr/bin/jjs jjs /usr/lib/jvm/java-11-openjdk-amd64/bin/jjs 1111
    //
    //(" %Y-%m-%d %H:%M:%S: ", true, false, false, 19, 41, 20, 39),
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/Ubuntu18/cups/error_log`
    // example with offset:
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     E [09/Aug/2019:00:09:01 -0700] Unable to open listen socket for address [v1.::1]:631 - Cannot assign requested address.
    //
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/other/archives/proftpd/xferlog`
    // example with offset:
    //
    //               1         2
    //     0123456789012345678901234
    //     Sat Oct 03 11:26:12 2020 0 192.168.1.1 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c
    //
    // XXX: ignore the leading Day Of Week substring
    //
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/OpenSUSE15/zypper.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2019-05-23 16:53:43 <1> trenker(24689) [zypper] main.cc(main):74 ===== Hi, me zypper 1.14.27
    //
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/synology/synoupdate.log`
    // example with offset:
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     2016/12/05 21:34:43	Start of the update…
    //
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/synology/synobackup.log`
    // example with offset:
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     err	2020/03/01 09:06:32	SYSTEM: [Network][Hyper Backup Task 1] Failed to start backup task.
    //     info	2017/02/21 23:01:59	admin: Setting of backup task [Local Storage 1] was created
    //     warning	2020/02/24 03:00:20	SYSTEM:  Scheduled backup had been skipped
    //
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     2020-01-02 12:33:59.001 xyz
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/debian9/apport.log.1`
    // example with offset:
    //
    //               1         2         3         4         5
    //     012345678901234567890123456789012345678901234567890
    //     ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 -0700: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 -07:00: called for pid 8581, signal 24, core limit 0, dump mode 1
    //
    // ---------------------------------------------------------------------------------------------
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     INFO: Thu Feb 20 00:59:59 2020 info
    //     ERROR: Thu Feb 20 00:59:59 2020 error
    //     DEBUG: Thu Feb 20 00:59:59 2020 debug
    //     VERBOSE: Thu Feb 20 00:59:59 2020 verbose
    //
    // ---------------------------------------------------------------------------------------------
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     INFO: Sat Jan 01 2000 08:00:00 info
    //     WARN: Sat Jan 01 2000 08:00:00 warn
    //     ERROR: Sat Jan 01 2000 08:00:00 error
    //     DEBUG: Sat Jan 01 2000 08:00:00 debug
    //     VERBOSE: Sat Jan 01 2000 08:00:00 verbose
    //
    // ---------------------------------------------------------------------------------------------
    // TODO: [2022/03/24] add timestamp formats seen at https://www.unixtimestamp.com/index.php
    // ---------------------------------------------------------------------------------------------
    // general matches from start
    //
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZz),
        DTFSS_YmdHMSfz, 0, 50, CGN_YEAR, CGN_TZ,
        &["2000/01/02 00:00:02.123 -1100 a"],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZcz),
        DTFSS_YmdHMSfcz, 0, 50, CGN_YEAR, CGN_TZ,
        &[r"2000/01/03 00:00:03.123456 -11:30 ab"],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZpz),
        DTFSS_YmdHMSfpz, 0, 50, CGN_YEAR, CGN_TZ,
        &[r"2000/01/04 00:00:04,123456789 -11 abc"],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANK, CGP_TZZn),
        DTFSS_YmdHMSfZ, 0, 50, CGN_YEAR, CGN_TZ,
        &[r"2000/01/05 00:00:05.123456789 VLAT abcd"],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL),
        DTFSS_YmdHMSf, 0, 50, CGN_YEAR, CGN_FRACTIONAL,
        &["2020-01-06 00:00:26.123456789 abcdefg"],
        line!(),
    ),
    //
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz),
        DTFSS_YmdHMSz, 0, 50, CGN_YEAR, CGN_TZ,
        &["2000/01/07T00:00:02 -1100 abcdefgh"],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZcz),
        DTFSS_YmdHMScz, 0, 50, CGN_YEAR, CGN_TZ,
        &[r"2000-01-08-00:00:03 -11:30 abcdefghi"],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZpz),
        DTFSS_YmdHMSpz, 0, 50, CGN_YEAR, CGN_TZ,
        &[r"2000/01/09 00:00:04 -11 abcdefghij"],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZZn),
        DTFSS_YmdHMSZ, 0, 50, CGN_YEAR, CGN_TZ,
        &[r"2000/01/10T00:00:05 VLAT abcdefghijk"],
        line!(),
    ),
    //
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND),
        DTFSS_YmdHMS, 0, 50, CGN_YEAR, CGN_SECOND,
        &["2020-01-11 00:00:26 abcdefghijkl"],
        line!(),
    ),
    //
    // general matches anywhere in the first 1024 bytes of the line
    //
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZz),
        DTFSS_YmdHMSfz, 0, 1024, CGN_YEAR, CGN_TZ,
        &["2000/01/02 00:01:02.123 -1100 a"],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZcz),
        DTFSS_YmdHMSfcz, 0, 1024, CGN_YEAR, CGN_TZ,
        &[r"2000/01/03 00:02:03.123456 -11:30 ab"],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZpz),
        DTFSS_YmdHMSfpz, 0, 1024, CGN_YEAR, CGN_TZ,
        &[r"2000/01/04 00:03:04,123456789 -11 abc"],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANK, CGP_TZZn),
        DTFSS_YmdHMSfZ, 0, 1024, CGN_YEAR, CGN_TZ,
        &[r"2000/01/05 00:04:05.123456789 VLAT abcd"],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL),
        DTFSS_YmdHMSf, 0, 1024, CGN_YEAR, CGN_FRACTIONAL,
        &["2020-01-06 00:05:26.123456789 abcdefg"],
        line!(),
    ),
    //
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz),
        DTFSS_YmdHMSz, 0, 1024, CGN_YEAR, CGN_TZ,
        &["2000/01/07T00:06:02 -1100 abcdefgh"],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZcz),
        DTFSS_YmdHMScz, 0, 1024, CGN_YEAR, CGN_TZ,
        &[r"2000-01-08-00:07:03 -11:30 aabcdefghi"],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZpz),
        DTFSS_YmdHMSpz, 0, 1024, CGN_YEAR, CGN_TZ,
        &[r"2000/01/09 00:08:04 -11 abcdefghij"],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZZn),
        DTFSS_YmdHMSZ, 0, 1024, CGN_YEAR, CGN_TZ,
        &[r"2000/01/10T00:09:05 VLAT abcdefghijk"],
        line!(),
    ),
    //
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND),
        DTFSS_YmdHMS, 0, 1024, CGN_YEAR, CGN_SECOND,
        &["2020-01-11 00:10:26 abcdefghijkl"],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // TODO: Issue #4 handle dmesg
    //
    // dmesg format, example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     [    0.000000] kernel: Linux version 5.15.0-43-generic (buildd@lcy02-amd64-076) (gcc (Ubuntu 11.2.0-19ubuntu1) 11.2.0, GNU ld (GNU Binutils for Ubuntu) 2.38) #46-Ubuntu SMP Tue Jul 12 10:30:17 UTC 2022 (Ubuntu 5.15.0-43.46-generic 5.15.39)
    //     [    0.000000] kernel: Command line: BOOT_IMAGE=/boot/vmlinuz-5.15.0-43-generic root=UUID=136735fa-5cc1-470f-9359-ee736e42f844 ro console=tty1 console=ttyS0 net.ifnames=0 biosdevname=0
    //     [    0.000000] kernel: KERNEL supported cpus:
    //     [    0.000000] kernel:   Intel GenuineIntel
    //
    //DTPD!(
    //    concatcp!("^[", RP_BLANKSq, CGP_UPTIME, "]", RP_BLANK),
    //    DTFSS_u, 0, 20, CGN_UPTIME, CGN_UPTIME,
    //    &[
    //        "[    0.000000] kernel: KERNEL supported cpus:",
    //        "[    5.364159] kernel: ISO 9660 Extensions: RRIP_1991A",
    //    ],
    //    line!(),
    //),
];

lazy_static! {
    // `Regex::new` runs at run-time, create this vector on-demand
    pub(crate) static ref DATETIME_PARSE_DATAS_REGEX_VEC: DateTimeParseInstrsRegexVec =
        DATETIME_PARSE_DATAS.iter().map(
            |x| Regex::new(x.regex_pattern).unwrap()
        ).collect();
}

// TODO: Issue #6 handle all Unicode whitespace.
//       This fn is essentially counteracting an errant call to `std::string:trim`
//       within `Local.datetime_from_str`.
//       `trim` removes "Unicode Derived Core Property White_Space".
//       This implementation handles three whitespace chars. There are twenty-five whitespace
//       chars according to
//       https://en.wikipedia.org/wiki/Unicode_character_property#Whitespace
/// workaround for chrono Issue #660 https://github.com/chronotope/chrono/issues/660
/// match spaces at beginning and ending of inputs
#[allow(non_snake_case)]
pub fn datetime_from_str_workaround_Issue660(value: &str, pattern: &DateTimePattern_str) -> bool {
    const SPACES: &str = " ";
    const TABS: &str = "\t";
    const LINE_ENDS: &str = "\n\r";

    // match whitespace forwards from beginning
    let mut v_sc: u32 = 0;  // `value` spaces count
    let mut v_tc: u32 = 0;  // `value` tabs count
    let mut v_ec: u32 = 0;  // `value` line ends count
    let mut v_brk: bool = false;
    for v_ in value.chars() {
        if SPACES.contains(v_) {
            v_sc += 1;
        } else if TABS.contains(v_) {
            v_tc += 1;
        } else if LINE_ENDS.contains(v_) {
            v_ec += 1;
        } else {
            v_brk = true;
            break;
        }
    }
    let mut p_sc: u32 = 0;  // `pattern` space count
    let mut p_tc: u32 = 0;  // `pattern` tab count
    let mut p_ec: u32 = 0;  // `pattern` line ends count
    let mut p_brk: bool = false;
    for p_ in pattern.chars() {
        if SPACES.contains(p_) {
            p_sc += 1;
        } else if TABS.contains(p_) {
            p_tc += 1;
        } else if LINE_ENDS.contains(p_) {
            p_ec += 1;
        } else {
            p_brk = true;
            break;
        }
    }
    if v_sc != p_sc || v_tc != p_tc || v_ec != p_ec {
        return false;
    }

    // match whitespace backwards from ending
    v_sc = 0;
    v_tc = 0;
    v_ec = 0;
    if v_brk {
        for v_ in value.chars().rev() {
            if SPACES.contains(v_) {
                v_sc += 1;
            } else if TABS.contains(v_) {
                v_tc += 1;
            } else if LINE_ENDS.contains(v_) {
                v_ec += 1;
            } else {
                break;
            }
        }
    }
    p_sc = 0;
    p_tc = 0;
    p_ec = 0;
    if p_brk {
        for p_ in pattern.chars().rev() {
            if SPACES.contains(p_) {
                p_sc += 1;
            } else if TABS.contains(p_) {
                p_tc += 1;
            } else if LINE_ENDS.contains(p_) {
                p_ec += 1;
            } else {
                break;
            }
        }
    }
    if v_sc != p_sc || v_tc != p_tc || v_ec != p_ec {
        return false;
    }

    true
}

/// decoding `[u8]` bytes to a `str` takes a surprising amount of time, according to `tools/flamegraph.sh`.
/// first check `u8` slice with custom simplistic checker that, in case of complications,
/// falls back to using higher-resource and more-precise checker `encoding_rs::mem::utf8_latin1_up_to`.
/// this uses built-in unsafe `str::from_utf8_unchecked`.
/// See `benches/bench_decode_utf.rs` for comparison of bytes->str decode strategies
#[inline(always)]
pub fn u8_to_str(data: &[u8]) -> Option<&str> {
    let dts: &str;
    let mut fallback = false;
    // custom check for UTF8; fast but imperfect
    if ! data.is_ascii() {
        fallback = true;
    }
    if fallback {
        // found non-ASCII, fallback to checking with `utf8_latin1_up_to` which is a thorough check
        let va = encoding_rs::mem::utf8_latin1_up_to(data);
        if va != data.len() {
            // TODO: this needs a better resolution
            return None;  // invalid UTF8
        }
    }
    unsafe {
        dts = std::str::from_utf8_unchecked(data);
    };
    Some(dts)
}

/// convert a `&str` to a chrono `Option<DateTime<FixedOffset>>` instance.
///
/// compensate for a missing timezone.
///
/// - `data` to parse that has a datetime string
/// - strftime `pattern` to use for parsing
/// - `has_tz`, the `pattern` has a timezone (`%Z`, `%z`, etc.)
/// - `tz_offset` fallback timezone offset when `!has_tz`
pub fn datetime_parse_from_str(
    data: &str,
    pattern: &DateTimePattern_str,
    has_tz: bool,
    tz_offset: &FixedOffset,
) -> DateTimeLOpt {
    dpnf!("(pattern {:?}, tz_offset {:?}, data {:?})", pattern, tz_offset, str_to_String_noraw(data));

    // if `has_tz` then create a `DateTime`.
    // else if `!has_tz` then create a `NaiveDateTime`, then convert that to `DateTime` with aid
    // of crate `chrono_tz`.
    if has_tz {
        match DateTime::parse_from_str(data, pattern) {
            Ok(val) => {
                dpof!(
                    "DateTime::parse_from_str({:?}, {:?}) extrapolated DateTime {:?}",
                    str_to_String_noraw(data),
                    pattern,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `data`
                //       and `dt_pattern`
                if !datetime_from_str_workaround_Issue660(data, pattern) {
                    dpnf!("skip match due to chrono Issue #660");
                    return None;
                }
                dpxf!("return Some({:?})", val);

                Some(val)
            }
            Err(_err) => {
                dpxf!("DateTime::parse_from_str({:?}, {:?}) failed ParseError: {}", data, pattern, _err);

                None
            }
        }
    } else {  // !has_tz
        // no timezone in `pattern` so first convert to a `NaiveDateTime` instance
        let dt_naive = match NaiveDateTime::parse_from_str(data, pattern) {
            Ok(val) => {
                dpof!(
                    "NaiveDateTime.parse_from_str({:?}, {:?}) extrapolated NaiveDateTime {:?}",
                    str_to_String_noraw(data),
                    pattern,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `data`
                //       and `pattern`
                if !datetime_from_str_workaround_Issue660(data, pattern) {
                    dpxf!("skip match due to chrono Issue #660");
                    return None;
                }
                dpxf!("return {:?}", val);

                val
            }
            Err(_err) => {
                dpxf!("NaiveDateTime.parse_from_str({:?}, {:?}) failed ParseError: {}", data, pattern, _err);
                return None;
            }
        };
        // second convert the `NaiveDateTime` instance to `DateTime<FixedOffset>` instance
        match tz_offset.from_local_datetime(&dt_naive).earliest() {
            Some(val) => {
                dpof!(
                    "tz_offset.from_local_datetime({:?}).earliest() extrapolated NaiveDateTime {:?}",
                    dt_naive,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `data`
                //       and `pattern`
                if !datetime_from_str_workaround_Issue660(data, pattern) {
                    dpxf!("skip match due to chrono Issue #660, return None");
                    return None;
                }
                dpxf!("return {:?}", Some(val));

                Some(val)
            }
            None => {
                dpxf!("tz_offset.from_local_datetime({:?}, {:?}) returned None, return None", data, pattern);
                None
            }
        }
    }
}

/// data of interest from a set of `regex::Captures` for a datetime substring found in a `Line`
///
/// - datetime substring begin index
/// - datetime substring end index
/// - datetime
pub type CapturedDtData = (LineIndex, LineIndex, DateTimeL);

/// helper to `captures_to_buffer_bytes`
macro_rules! copy_capturegroup_to_buffer {
    (
        $name:ident,
        $captures:ident,
        $buffer:ident,
        $at:ident
    ) => {
        let len_: usize = $captures.name($name).as_ref().unwrap().as_bytes().len();
        dpo!("bytes_to_regex_to_datetime:copy_capturegroup_to_buffer! buffer[{:?}‥{:?}]", $at, $at+len_);
        $buffer[$at..$at+len_].copy_from_slice($captures.name($name).as_ref().unwrap().as_bytes());
        $at += len_;
    }
}

/// helper to `captures_to_buffer_bytes`
macro_rules! copy_slice_to_buffer {
    (
        $u8_slice:expr,
        $buffer:ident,
        $at:ident
    ) => {
        let len_: usize = $u8_slice.len();
        dpo!("bytes_to_regex_to_datetime:copy_slice_to_buffer! buffer[{:?}‥{:?}]", $at, $at+len_);
        $buffer[$at..$at+len_].copy_from_slice($u8_slice);
        $at += len_;
    }
}


/// helper to `captures_to_buffer_bytes`
macro_rules! copy_u8_to_buffer {
    (
        $u8_:expr,
        $buffer:ident,
        $at:ident
    ) => {
        dpo!("bytes_to_regex_to_datetime:copy_slice_to_buffer! buffer[{:?}] = {:?}", $at, $u8_);
        $buffer[$at] = $u8_;
        $at += 1;
    }
}

// variables `const MONTH_` are helpers to `fn month_bB_to_month_m_bytes`
//
// TODO: replace `to_byte_array` with rust experimental feature `const_str_as_bytes`
//       https://doc.bccnsoft.com/docs/rust-1.36.0-docs-html/unstable-book/library-features/const-str-as-bytes.html#const_str_as_bytes
//       https://github.com/rust-lang/rust/issues/57563
//

const MONTH_01_B_l: &[u8] = &to_byte_array!("january");
const MONTH_01_b_l: &[u8] = &to_byte_array!("jan");
const MONTH_01_B_u: &[u8] = &to_byte_array!("January");
const MONTH_01_b_u: &[u8] = &to_byte_array!("Jan");
const MONTH_01_m: &[u8] = &to_byte_array!("01");
const MONTH_02_B_l: &[u8] = &to_byte_array!("february");
const MONTH_02_b_l: &[u8] = &to_byte_array!("feb");
const MONTH_02_B_u: &[u8] = &to_byte_array!("February");
const MONTH_02_b_u: &[u8] = &to_byte_array!("Feb");
const MONTH_02_m: &[u8] = &to_byte_array!("02");
const MONTH_03_B_l: &[u8] = &to_byte_array!("march");
const MONTH_03_b_l: &[u8] = &to_byte_array!("mar");
const MONTH_03_B_u: &[u8] = &to_byte_array!("March");
const MONTH_03_b_u: &[u8] = &to_byte_array!("Mar");
const MONTH_03_m: &[u8] = &to_byte_array!("03");
const MONTH_04_B_l: &[u8] = &to_byte_array!("april");
const MONTH_04_b_l: &[u8] = &to_byte_array!("apr");
const MONTH_04_B_u: &[u8] = &to_byte_array!("April");
const MONTH_04_b_u: &[u8] = &to_byte_array!("Apr");
const MONTH_04_m: &[u8] = &to_byte_array!("04");
const MONTH_05_B_l: &[u8] = &to_byte_array!("may");
const _MONTH_05_b_l: &[u8] = &to_byte_array!("may");  // not used, defined for completeness
const MONTH_05_B_u: &[u8] = &to_byte_array!("May");
const _MONTH_05_b_u: &[u8] = &to_byte_array!("May");  // not used, defined for completeness
const MONTH_05_m: &[u8] = &to_byte_array!("05");
const MONTH_06_B_l: &[u8] = &to_byte_array!("june");
const MONTH_06_b_l: &[u8] = &to_byte_array!("jun");
const MONTH_06_B_u: &[u8] = &to_byte_array!("June");
const MONTH_06_b_u: &[u8] = &to_byte_array!("Jun");
const MONTH_06_m: &[u8] = &to_byte_array!("06");
const MONTH_07_B_l: &[u8] = &to_byte_array!("july");
const MONTH_07_b_l: &[u8] = &to_byte_array!("jul");
const MONTH_07_B_u: &[u8] = &to_byte_array!("July");
const MONTH_07_b_u: &[u8] = &to_byte_array!("Jul");
const MONTH_07_m: &[u8] = &to_byte_array!("07");
const MONTH_08_B_l: &[u8] = &to_byte_array!("august");
const MONTH_08_b_l: &[u8] = &to_byte_array!("aug");
const MONTH_08_B_u: &[u8] = &to_byte_array!("August");
const MONTH_08_b_u: &[u8] = &to_byte_array!("Aug");
const MONTH_08_m: &[u8] = &to_byte_array!("08");
const MONTH_09_B_l: &[u8] = &to_byte_array!("september");
const MONTH_09_b_l: &[u8] = &to_byte_array!("sep");
const MONTH_09_B_u: &[u8] = &to_byte_array!("September");
const MONTH_09_b_u: &[u8] = &to_byte_array!("Sep");
const MONTH_09_m: &[u8] = &to_byte_array!("09");
const MONTH_10_B_l: &[u8] = &to_byte_array!("october");
const MONTH_10_b_l: &[u8] = &to_byte_array!("oct");
const MONTH_10_B_u: &[u8] = &to_byte_array!("October");
const MONTH_10_b_u: &[u8] = &to_byte_array!("Oct");
const MONTH_10_m: &[u8] = &to_byte_array!("10");
const MONTH_11_B_l: &[u8] = &to_byte_array!("november");
const MONTH_11_b_l: &[u8] = &to_byte_array!("nov");
const MONTH_11_B_u: &[u8] = &to_byte_array!("November");
const MONTH_11_b_u: &[u8] = &to_byte_array!("Nov");
const MONTH_11_m: &[u8] = &to_byte_array!("11");
const MONTH_12_B_l: &[u8] = &to_byte_array!("december");
const MONTH_12_b_l: &[u8] = &to_byte_array!("dec");
const MONTH_12_B_u: &[u8] = &to_byte_array!("December");
const MONTH_12_b_u: &[u8] = &to_byte_array!("Dec");
const MONTH_12_m: &[u8] = &to_byte_array!("12");

/// helper to `captures_to_buffer_bytes`
///
/// transform `%B`, `%b` (i.e. "January", "Jan") to `%m` (i.e. "01")
#[allow(non_snake_case)]
fn month_bB_to_month_m_bytes(data: &[u8], buffer: &mut [u8]) {
    match data {
        MONTH_01_B_l | MONTH_01_b_l | MONTH_01_B_u | MONTH_01_b_u
            => buffer.copy_from_slice(MONTH_01_m),
        MONTH_02_B_l | MONTH_02_b_l | MONTH_02_B_u | MONTH_02_b_u
            => buffer.copy_from_slice(MONTH_02_m),
        MONTH_03_B_l | MONTH_03_b_l | MONTH_03_B_u | MONTH_03_b_u
            => buffer.copy_from_slice(MONTH_03_m),
        MONTH_04_B_l | MONTH_04_b_l | MONTH_04_B_u | MONTH_04_b_u
            => buffer.copy_from_slice(MONTH_04_m),
        //MONTH_05_B_l | MONTH_05_b_l | MONTH_05_B_u | MONTH_05_b_u
        MONTH_05_B_l | MONTH_05_B_u
            => buffer.copy_from_slice(MONTH_05_m),
        MONTH_06_B_l | MONTH_06_b_l | MONTH_06_B_u | MONTH_06_b_u
            => buffer.copy_from_slice(MONTH_06_m),
        MONTH_07_B_l | MONTH_07_b_l | MONTH_07_B_u | MONTH_07_b_u
            => buffer.copy_from_slice(MONTH_07_m),
        MONTH_08_B_l | MONTH_08_b_l | MONTH_08_B_u | MONTH_08_b_u
            => buffer.copy_from_slice(MONTH_08_m),
        MONTH_09_B_l | MONTH_09_b_l | MONTH_09_B_u | MONTH_09_b_u
            => buffer.copy_from_slice(MONTH_09_m),
        MONTH_10_B_l | MONTH_10_b_l | MONTH_10_B_u | MONTH_10_b_u
            => buffer.copy_from_slice(MONTH_10_m),
        MONTH_11_B_l | MONTH_11_b_l | MONTH_11_B_u | MONTH_11_b_u
            => buffer.copy_from_slice(MONTH_11_m),
        MONTH_12_B_l | MONTH_12_b_l | MONTH_12_B_u | MONTH_12_b_u
            => buffer.copy_from_slice(MONTH_12_m),
        data_ => {
            panic!("month_bB_to_month_m_bytes: unexpected month value {:?}", data_);
            //debug_assert_le!(data_.len(), 2, "month_bB_to_month_m_bytes passed bad data; len {}; {:?}", data_.len(), data_);
            //buffer.copy_from_slice(data_)
        }
    }
}

/// Put `Captures` into a `String` buffer in a particular order and formatting. This bridges the
/// `DateTimeParseInstr::regex_pattern` to `DateTimeParseInstr::dt_pattern`.
///
/// Directly relates to datetime format `dt_pattern` values in `DATETIME_PARSE_DATAS`
/// which use `DTFSS_YmdHMS`, etc.
///
/// transforms `%B` acceptable value to `%m` acceptable value.
///
/// transforms `%e` acceptable value to `%d` acceptable value.
///
#[inline(always)]
fn captures_to_buffer_bytes(
    buffer: &mut[u8],
    captures: &regex::bytes::Captures,
    year_opt: &Option<Year>,
    tz_offset: &FixedOffset,
    dtfs: &DTFSSet,
) -> usize {
    dpnf!("(…, …, year_opt {:?}, tz_offset {:?}, …)", year_opt, tz_offset);

    let mut at: usize = 0;

    // year
    match captures.name(CGN_YEAR).as_ref() {
        Some(match_) => {
            copy_slice_to_buffer!(match_.as_bytes(), buffer, at);
        }
        None => {
            match year_opt {
                Some(year) => {
                    // TODO: 2022/07/11 cost-savings: pass in `Option<&[u8]>`, avoid creating `String`
                    let year_s: String = year.to_string();
                    debug_assert_eq!(year_s.len(), 4, "Bad year string {:?}", year_s);
                    dpof!("using fallback year {:?}", year_s);
                    copy_slice_to_buffer!(year_s.as_bytes(), buffer, at);
                }
                None => {
                    dpof!("using hardcoded dummy year {:?}", YEAR_FALLBACKDUMMY);
                    copy_slice_to_buffer!(YEAR_FALLBACKDUMMY.as_bytes(), buffer, at);
                }
            }
        }
    }
    // month
    match dtfs.month {
        DTFS_Month::b | DTFS_Month::B => {
            month_bB_to_month_m_bytes(
                captures.name(CGN_MONTH).as_ref().unwrap().as_bytes(),
                &mut buffer[at..at+2]
            );
            at += 2;
        }
        DTFS_Month::m => {
            copy_capturegroup_to_buffer!(CGN_MONTH, captures, buffer, at);
        }
    }
    // day
    match dtfs.day {
        DTFS_Day::d => {
            copy_capturegroup_to_buffer!(CGN_DAY, captures, buffer, at);
        }
        DTFS_Day::_e_to_d => {
            let day: &[u8] = captures.name(CGN_DAY).as_ref().unwrap().as_bytes();
            debug_assert_ge!(day.len(), 1, "bad named group 'day' data {:?}, expected data ge 1", day);
            debug_assert_le!(day.len(), 2, "bad named group 'day' data {:?}, expected data le 2", day);
            match day.len() {
                1 => {
                    // change day "8" to "08"
                    copy_u8_to_buffer!('0' as u8, buffer, at);
                    copy_u8_to_buffer!(day[0], buffer, at);
                }
                2 => {
                    debug_assert_ne!(day[0], ' ' as u8, "bad value for _e_to_d {:?} {:?}", day, String::from_utf8_lossy(day));
                    copy_slice_to_buffer!(day, buffer, at);
                }
                _ => {
                    panic!("bad day.len() {}", day.len());
                }
            }
        }
        DTFS_Day::e => {
            panic!("Do not use DTFS_Day::e in a DTFS");
        }
    }
    copy_u8_to_buffer!('T' as u8, buffer, at);
    // hour
    copy_capturegroup_to_buffer!(CGN_HOUR, captures, buffer, at);
    // minute
    copy_capturegroup_to_buffer!(CGN_MINUTE, captures, buffer, at);
    // second
    copy_capturegroup_to_buffer!(CGN_SECOND, captures, buffer, at);
    // fractional
    match dtfs.fractional {
        DTFS_Fractional::f => {
            copy_u8_to_buffer!('.' as u8, buffer, at);
            copy_capturegroup_to_buffer!(CGN_FRACTIONAL, captures, buffer, at);
        }
        DTFS_Fractional::_none => {}
    }
    // tz
    match dtfs.tz {
        DTFS_Tz::_fill => {
            // TODO: cost-savings: pass pre-created TZ `&str`
            let tzs: String = tz_offset.to_string();
            copy_slice_to_buffer!(tzs.as_bytes(), buffer, at);
        }
        DTFS_Tz::z | DTFS_Tz::cz | DTFS_Tz::pz => {
            copy_capturegroup_to_buffer!(CGN_TZ, captures, buffer, at);
        }
        DTFS_Tz::Z => {
            #[allow(non_snake_case)]
            let tzZ: &str = u8_to_str(captures.name(CGN_TZ).as_ref().unwrap().as_bytes()).unwrap();
            match MAP_TZZ_TO_TZz.get_key_value(tzZ) {
                Some((_tz_abbr, tz_offset_)) => {
                    // TODO: cost-savings: pre-create the `tz_offset` entries as bytes
                    let tzs: String = tz_offset_.to_string();
                    copy_slice_to_buffer!(tzs.as_bytes(), buffer, at);
                }
                None => {
                    // cannot find entry in MAP_TZZ_TO_TZz, fill with passed TZ
                    // TODO: cost-savings: pre-create the `tz_offset` entries as bytes
                    let tzs: String = tz_offset.to_string();
                    copy_slice_to_buffer!(tzs.as_bytes(), buffer, at);
                }
            }

        }
    }

    dpxf!("return {:?}", at);

    at
}

/// run `regex::Captures` on the `data` then convert to a chrono
/// `Option<DateTime<FixedOffset>>` instance. Uses matching and pattern information
/// hardcoded in `DATETIME_PARSE_DATAS_REGEX` and `DATETIME_PARSE_DATAS`.
pub fn bytes_to_regex_to_datetime(
    data: &[u8],
    index: &DateTimeParseInstrsIndex,
    year_opt: &Option<Year>,
    tz_offset: &FixedOffset,
) -> Option<CapturedDtData> {
    dpnf!("(…, {:?}, {:?}, {:?})", index, year_opt, tz_offset);

    let regex_: &Regex = match DATETIME_PARSE_DATAS_REGEX_VEC.get(*index) {
        Some(val) => val,
        None => {
            panic!("requested DATETIME_PARSE_DATAS_REGEX_VEC.get({}), returned None. DATETIME_PARSE_DATAS_REGEX_VEC.len() {}", index, DATETIME_PARSE_DATAS_REGEX_VEC.len());
        }
    };

    let captures: regex::bytes::Captures = match regex_.captures(data) {
        None => {
            dpxf!("regex: no captures (returned None)");
            return None;
        }
        Some(captures) => {
            dpo!("regex: captures.len() {}", captures.len());

            captures
        }
    };
    if cfg!(debug_assertions) {
        for (i, name_opt) in regex_.capture_names().enumerate() {
            let _match: regex::bytes::Match = match captures.get(i) {
                Some(m_) => m_,
                None => {
                    match name_opt {
                        Some(_name) => {
                            dpo!("regex captures: {:2} {:<10} None", i, _name);
                        },
                        None => {
                            dpo!("regex captures: {:2} {:<10} None", i, "None");
                        }
                    }
                    continue;
                }
            };
            match name_opt {
                Some(name) => {
                    dpo!("regex captures: {:2} {:<10} {:?}", i, name, buffer_to_String_noraw(_match.as_bytes()));
                },
                None => {
                    dpo!("regex captures: {:2} {:<10} {:?}", i, "NO NAME", buffer_to_String_noraw(_match.as_bytes()));
                }
            }
        }
    }
    // sanity check
    debug_assert!(!captures.iter().any(|x| x.is_none()), "a match in the regex::Captures was None");

    let dtpd: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*index];
    // copy regex matches into a buffer with predictable ordering
    // this ordering relates to datetime format strings in `DATETIME_PARSE_DATAS`
    // TODO: [2022/06/26] cost-savings: avoid a `String` alloc by passing precreated buffer
    const BUFLEN: usize = 35;
    let mut buffer: [u8; BUFLEN] = [0; BUFLEN];
    let copiedn = captures_to_buffer_bytes(&mut buffer, &captures, year_opt, tz_offset, &dtpd.dtfs);

    // use the `dt_format` to parse the buffer of regex matches
    let buffer_s: &str = u8_to_str(&buffer[0..copiedn]).unwrap();
    let dt = match datetime_parse_from_str(
        buffer_s,
        dtpd.dtfs.pattern,
        dtpd.dtfs.has_tz(),
        tz_offset,
    ) {
        Some(dt_) => dt_,
        None => {
            dpxf!("return None; datetime_parse_from_str returned None");
            return None;
        }
    };

    // derive the `LineIndex` bounds of the datetime substring within `data`
    // TODO: cost-savings: only track dt_first dt_last if using `--color`
    let dt_beg: LineIndex = match captures.name(dtpd.cgn_first) {
        Some(match_) => match_.start() as LineIndex,
        None => 0,
    };
    let dt_end: LineIndex = match captures.name(dtpd.cgn_last) {
        Some(match_) => match_.end() as LineIndex,
        None => 0,
    };
    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}, index {}", dt_beg, dt_end, index);

    dpxf!("return Some({:?}, {:?}, {:?})", dt_beg, dt_end, dt);
    Some((dt_beg, dt_end, dt))
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DateTime comparisons
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// describe the result of comparing one DateTime to one DateTime Filter
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Result_Filter_DateTime1 {
    Pass,
    OccursAtOrAfter,
    OccursBefore,
}

impl Result_Filter_DateTime1 {
    /// Returns `true` if the result is [`OccursAfter`].
    #[inline(always)]
    pub const fn is_after(&self) -> bool {
        matches!(*self, Result_Filter_DateTime1::OccursAtOrAfter)
    }

    /// Returns `true` if the result is [`OccursBefore`].
    #[inline(always)]
    pub const fn is_before(&self) -> bool {
        matches!(*self, Result_Filter_DateTime1::OccursBefore)
    }
}

/// describe the result of comparing one DateTime to two DateTime Filters
/// `(after, before)`
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Result_Filter_DateTime2 {
    /// PASS
    InRange,
    /// FAIL
    BeforeRange,
    /// FAIL
    AfterRange,
}

impl Result_Filter_DateTime2 {
    #[inline(always)]
    pub const fn is_pass(&self) -> bool {
        matches!(*self, Result_Filter_DateTime2::InRange)
    }

    #[inline(always)]
    pub const fn is_fail(&self) -> bool {
        matches!(*self, Result_Filter_DateTime2::AfterRange | Result_Filter_DateTime2::BeforeRange)
    }
}

/// if `dt` is at or after `dt_filter` then return `OccursAtOrAfter`
/// if `dt` is before `dt_filter` then return `OccursBefore`
/// else return `Pass` (including if `dt_filter` is `None`)
pub fn dt_after_or_before(dt: &DateTimeL, dt_filter: &DateTimeLOpt) -> Result_Filter_DateTime1 {
    if dt_filter.is_none() {
        dpnxf!("return Result_Filter_DateTime1::Pass; (no dt filters)");
        return Result_Filter_DateTime1::Pass;
    }

    let dt_a = &dt_filter.unwrap();
    dpnf!("comparing dt datetime {:?} to filter datetime {:?}", dt, dt_a);
    if dt < dt_a {
        dpxf!("return Result_Filter_DateTime1::OccursBefore; (dt {:?} is before dt_filter {:?})", dt, dt_a);
        return Result_Filter_DateTime1::OccursBefore;
    }
    dpxf!("return Result_Filter_DateTime1::OccursAtOrAfter; (dt {:?} is at or after dt_filter {:?})", dt, dt_a);

    Result_Filter_DateTime1::OccursAtOrAfter
}

/// If both filters are `Some` and `dt` is "between" the filters then return `InRange`.
/// If before then return `BeforeRange`.
/// If after then return `AfterRange`.
///
/// If filter `dt_filter_after` is `Some` and `dt` is after that filter then
/// return `InRange`. If before then return `BeforeRange`.
///
/// If filter `dt_filter_before` is `Some` and `dt` is before that filter then
/// return `InRange`. If after then return `AfterRange`.
///
/// If both filters are `None` then return `InRange`.
///
/// Comparisons are "inclusive" i.e. `dt` == `dt_filter_after` will return `InRange`
pub fn dt_pass_filters(
    dt: &DateTimeL, dt_filter_after: &DateTimeLOpt, dt_filter_before: &DateTimeLOpt,
) -> Result_Filter_DateTime2 {
    dpnf!("({:?}, {:?}, {:?})", dt, dt_filter_after, dt_filter_before);
    if dt_filter_after.is_none() && dt_filter_before.is_none() {
        dpxf!("return {:?}; (no dt filters)", Result_Filter_DateTime2::InRange);
        return Result_Filter_DateTime2::InRange;
    }
    if dt_filter_after.is_some() && dt_filter_before.is_some() {
        dpof!(
            "comparing datetime dt_filter_after {:?} < {:?} dt < {:?} dt_fiter_before ???",
            &dt_filter_after.unwrap(),
            dt,
            &dt_filter_before.unwrap()
        );
        let da = &dt_filter_after.unwrap();
        let db = &dt_filter_before.unwrap();
        assert_le!(da, db, "Bad datetime range values filter_after {:?} {:?} filter_before", da, db);
        if dt < da {
            dpxf!("return {:?}", Result_Filter_DateTime2::BeforeRange);
            return Result_Filter_DateTime2::BeforeRange;
        }
        if db < dt {
            dpxf!("return {:?}", Result_Filter_DateTime2::AfterRange);
            return Result_Filter_DateTime2::AfterRange;
        }
        // assert da < dt && dt < db
        assert_le!(da, dt, "Unexpected range values da dt");
        assert_le!(dt, db, "Unexpected range values dt db");
        dpxf!("return Result_Filter_DateTime2::InRange");

        Result_Filter_DateTime2::InRange
    } else if dt_filter_after.is_some() {
        dpof!("comparing datetime dt_filter_after {:?} < {:?} dt ???", &dt_filter_after.unwrap(), dt);
        let da = &dt_filter_after.unwrap();
        if dt < da {
            dpxf!("return {:?}", Result_Filter_DateTime2::BeforeRange);
            return Result_Filter_DateTime2::BeforeRange;
        }
        dpxf!("return Result_Filter_DateTime2::InRange");

        Result_Filter_DateTime2::InRange
    } else {
        dpof!("comparing datetime dt {:?} < {:?} dt_filter_before ???", dt, &dt_filter_before.unwrap());
        let db = &dt_filter_before.unwrap();
        if db < dt {
            dpxf!("return {:?}", Result_Filter_DateTime2::AfterRange);
            return Result_Filter_DateTime2::AfterRange;
        }
        dpxf!("return {:?}", Result_Filter_DateTime2::InRange);

        Result_Filter_DateTime2::InRange
    }
}


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// other miscellaneous DateTime function helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// create a new `DateTimeL` instance that use the passed `datetime` month, day, and time, and the
/// passed `year`.
///
/// In case of error, return a copy of the passed `datetime`.
pub fn datetime_with_year(datetime: &DateTimeL, year: &Year) -> DateTimeL {
    match datetime.with_year(*year) {
        Some(datetime_) => {
            datetime_
        }
        None => {
            datetime.clone()
        }
    }
}

/// convert passed `SystemTime` to `DatetimeL` with passed  `fixedoffset`
pub fn systemtime_to_datetime(fixedoffset: &FixedOffset, systemtime: &SystemTime) -> DateTimeL {
    // https://users.rust-lang.org/t/convert-std-time-systemtime-to-chrono-datetime-datetime/7684/6
    let dtu: DateTime<Utc> = systemtime.clone().into();

    dtu.with_timezone(fixedoffset)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// search a slice quickly (loop unroll version)
// loop unrolled implementation of `slice.contains` for a byte slice and a hardcorded array
// benchmark `benches/bench_slice_contains.rs` demonstrates this is faster
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_2_2(slice_: &[u8; 2], search: &[u8; 2]) -> bool {
    for i in 0..1 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_3_2(slice_: &[u8; 3], search: &[u8; 2]) -> bool {
    for i in 0..2 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_4_2(slice_: &[u8; 4], search: &[u8; 2]) -> bool {
    for i in 0..3 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_5_2(slice_: &[u8; 5], search: &[u8; 2]) -> bool {
    for i in 0..4 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_6_2(slice_: &[u8; 6], search: &[u8; 2]) -> bool {
    for i in 0..5 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_7_2(slice_: &[u8; 7], search: &[u8; 2]) -> bool {
    for i in 0..6 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_8_2(slice_: &[u8; 8], search: &[u8; 2]) -> bool {
    for i in 0..7 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_9_2(slice_: &[u8; 9], search: &[u8; 2]) -> bool {
    for i in 0..8 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_10_2(slice_: &[u8; 10], search: &[u8; 2]) -> bool {
    for i in 0..9 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_11_2(slice_: &[u8; 11], search: &[u8; 2]) -> bool {
    for i in 0..10 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_12_2(slice_: &[u8; 12], search: &[u8; 2]) -> bool {
    for i in 0..11 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_13_2(slice_: &[u8; 13], search: &[u8; 2]) -> bool {
    for i in 0..12 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_14_2(slice_: &[u8; 14], search: &[u8; 2]) -> bool {
    for i in 0..13 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_15_2(slice_: &[u8; 15], search: &[u8; 2]) -> bool {
    for i in 0..14 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_16_2(slice_: &[u8; 16], search: &[u8; 2]) -> bool {
    for i in 0..15 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_17_2(slice_: &[u8; 17], search: &[u8; 2]) -> bool {
    for i in 0..16 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_18_2(slice_: &[u8; 18], search: &[u8; 2]) -> bool {
    for i in 0..17 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_19_2(slice_: &[u8; 19], search: &[u8; 2]) -> bool {
    for i in 0..18 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_20_2(slice_: &[u8; 20], search: &[u8; 2]) -> bool {
    for i in 0..19 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_21_2(slice_: &[u8; 21], search: &[u8; 2]) -> bool {
    for i in 0..20 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_22_2(slice_: &[u8; 22], search: &[u8; 2]) -> bool {
    for i in 0..21 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_23_2(slice_: &[u8; 23], search: &[u8; 2]) -> bool {
    for i in 0..22 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_24_2(slice_: &[u8; 24], search: &[u8; 2]) -> bool {
    for i in 0..23 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_25_2(slice_: &[u8; 25], search: &[u8; 2]) -> bool {
    for i in 0..24 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_26_2(slice_: &[u8; 26], search: &[u8; 2]) -> bool {
    for i in 0..25 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_27_2(slice_: &[u8; 27], search: &[u8; 2]) -> bool {
    for i in 0..26 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_28_2(slice_: &[u8; 28], search: &[u8; 2]) -> bool {
    for i in 0..27 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_29_2(slice_: &[u8; 29], search: &[u8; 2]) -> bool {
    for i in 0..28 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_30_2(slice_: &[u8; 30], search: &[u8; 2]) -> bool {
    for i in 0..29 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_31_2(slice_: &[u8; 31], search: &[u8; 2]) -> bool {
    for i in 0..30 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_32_2(slice_: &[u8; 32], search: &[u8; 2]) -> bool {
    for i in 0..31 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_33_2(slice_: &[u8; 33], search: &[u8; 2]) -> bool {
    for i in 0..32 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_34_2(slice_: &[u8; 34], search: &[u8; 2]) -> bool {
    for i in 0..33 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_35_2(slice_: &[u8; 35], search: &[u8; 2]) -> bool {
    for i in 0..34 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_36_2(slice_: &[u8; 36], search: &[u8; 2]) -> bool {
    for i in 0..35 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_37_2(slice_: &[u8; 37], search: &[u8; 2]) -> bool {
    for i in 0..36 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_38_2(slice_: &[u8; 38], search: &[u8; 2]) -> bool {
    for i in 0..37 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_39_2(slice_: &[u8; 39], search: &[u8; 2]) -> bool {
    for i in 0..38 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_40_2(slice_: &[u8; 40], search: &[u8; 2]) -> bool {
    for i in 0..39 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_41_2(slice_: &[u8; 41], search: &[u8; 2]) -> bool {
    for i in 0..40 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_42_2(slice_: &[u8; 42], search: &[u8; 2]) -> bool {
    for i in 0..41 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_43_2(slice_: &[u8; 43], search: &[u8; 2]) -> bool {
    for i in 0..42 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_44_2(slice_: &[u8; 44], search: &[u8; 2]) -> bool {
    for i in 0..43 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_45_2(slice_: &[u8; 45], search: &[u8; 2]) -> bool {
    for i in 0..44 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_46_2(slice_: &[u8; 46], search: &[u8; 2]) -> bool {
    for i in 0..45 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_47_2(slice_: &[u8; 47], search: &[u8; 2]) -> bool {
    for i in 0..46 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_48_2(slice_: &[u8; 48], search: &[u8; 2]) -> bool {
    for i in 0..47 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_49_2(slice_: &[u8; 49], search: &[u8; 2]) -> bool {
    for i in 0..48 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_50_2(slice_: &[u8; 50], search: &[u8; 2]) -> bool {
    for i in 0..49 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

/// loop unrolled implementation of `slice.contains` for a byte slice and a hardcorded array.
/// Uses crate `unroll`.
///
/// Hardcoded implementation for `u8` slices up to 50 length. Runs very fast.
/// Supports arbitrary length.
#[inline(always)]
#[allow(non_snake_case)]
pub fn slice_contains_X_2(slice_: &[u8], search: &[u8; 2]) -> bool {
    match slice_.len() {
        2 => slice_contains_2_2(array_ref!(slice_, 0, 2), search),
        3 => slice_contains_3_2(array_ref!(slice_, 0, 3), search),
        4 => slice_contains_4_2(array_ref!(slice_, 0, 4), search),
        5 => slice_contains_5_2(array_ref!(slice_, 0, 5), search),
        6 => slice_contains_6_2(array_ref!(slice_, 0, 6), search),
        7 => slice_contains_7_2(array_ref!(slice_, 0, 7), search),
        8 => slice_contains_8_2(array_ref!(slice_, 0, 8), search),
        9 => slice_contains_9_2(array_ref!(slice_, 0, 9), search),
        10 => slice_contains_10_2(array_ref!(slice_, 0, 10), search),
        11 => slice_contains_11_2(array_ref!(slice_, 0, 11), search),
        12 => slice_contains_12_2(array_ref!(slice_, 0, 12), search),
        13 => slice_contains_13_2(array_ref!(slice_, 0, 13), search),
        14 => slice_contains_14_2(array_ref!(slice_, 0, 14), search),
        15 => slice_contains_15_2(array_ref!(slice_, 0, 15), search),
        16 => slice_contains_16_2(array_ref!(slice_, 0, 16), search),
        17 => slice_contains_17_2(array_ref!(slice_, 0, 17), search),
        18 => slice_contains_18_2(array_ref!(slice_, 0, 18), search),
        19 => slice_contains_19_2(array_ref!(slice_, 0, 19), search),
        20 => slice_contains_20_2(array_ref!(slice_, 0, 20), search),
        21 => slice_contains_21_2(array_ref!(slice_, 0, 21), search),
        22 => slice_contains_22_2(array_ref!(slice_, 0, 22), search),
        23 => slice_contains_23_2(array_ref!(slice_, 0, 23), search),
        24 => slice_contains_24_2(array_ref!(slice_, 0, 24), search),
        25 => slice_contains_25_2(array_ref!(slice_, 0, 25), search),
        26 => slice_contains_26_2(array_ref!(slice_, 0, 26), search),
        27 => slice_contains_27_2(array_ref!(slice_, 0, 27), search),
        28 => slice_contains_28_2(array_ref!(slice_, 0, 28), search),
        29 => slice_contains_29_2(array_ref!(slice_, 0, 29), search),
        30 => slice_contains_30_2(array_ref!(slice_, 0, 30), search),
        31 => slice_contains_31_2(array_ref!(slice_, 0, 31), search),
        32 => slice_contains_32_2(array_ref!(slice_, 0, 32), search),
        33 => slice_contains_33_2(array_ref!(slice_, 0, 33), search),
        34 => slice_contains_34_2(array_ref!(slice_, 0, 34), search),
        35 => slice_contains_35_2(array_ref!(slice_, 0, 35), search),
        36 => slice_contains_36_2(array_ref!(slice_, 0, 36), search),
        37 => slice_contains_37_2(array_ref!(slice_, 0, 37), search),
        38 => slice_contains_38_2(array_ref!(slice_, 0, 38), search),
        39 => slice_contains_39_2(array_ref!(slice_, 0, 39), search),
        40 => slice_contains_40_2(array_ref!(slice_, 0, 40), search),
        41 => slice_contains_41_2(array_ref!(slice_, 0, 41), search),
        42 => slice_contains_42_2(array_ref!(slice_, 0, 42), search),
        43 => slice_contains_43_2(array_ref!(slice_, 0, 43), search),
        44 => slice_contains_44_2(array_ref!(slice_, 0, 44), search),
        45 => slice_contains_45_2(array_ref!(slice_, 0, 45), search),
        46 => slice_contains_46_2(array_ref!(slice_, 0, 46), search),
        47 => slice_contains_47_2(array_ref!(slice_, 0, 47), search),
        48 => slice_contains_48_2(array_ref!(slice_, 0, 48), search),
        49 => slice_contains_49_2(array_ref!(slice_, 0, 49), search),
        50 => slice_contains_50_2(array_ref!(slice_, 0, 50), search),
        _ => {
            slice_.iter().any(|&c| c == search[0] || c == search[1])
        }
    }
}
