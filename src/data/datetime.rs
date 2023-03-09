// src/data/datetime.rs

//! Functions to perform regular expression ("regex") searches on bytes and
//! transform matches to chrono [`DateTime`] instances.
//!
//! Parsing bytes and finding datetime strings requires:
//! 1. searching some slice of bytes from a [`Line`] for a regular expression
//!    match.
//! 2. using a [`DateTimeParseInstr`], attempting to transform the matched
//!    regular expression named capture groups into data passable to
//!    chrono [`DateTime::parse_from_str`] or [`NaiveDateTime::parse_from_str`].
//! 3. return chrono `DateTime` instances along with byte offsets of the found
//!    matches to a caller (who will presumably use it create a new
//!    [`Sysline`]).
//!
//! The most relevant documents to understand this file are:
//! - `chrono` crate [`strftime`] format.
//! - `regex` crate [Regular Expression syntax].
//!
//! The most relevant functions are:
//! - [`bytes_to_regex_to_datetime`] which calls
//! - [`captures_to_buffer_bytes`]
//!
//! The most relevant constant is [`DATETIME_PARSE_DATAS`].
//!
//! [`DATETIME_PARSE_DATAS`]: self::DATETIME_PARSE_DATAS
//! [`Line`]: crate::data::line::Line
//! [`Sysline`]: crate::data::sysline::Sysline
//! [`DateTime`]: https://docs.rs/chrono/0.4.22/chrono/struct.DateTime.html
//! [`DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.22/chrono/struct.DateTime.html#method.parse_from_str
//! [`NaiveDateTime::parse_from_str`]: https://docs.rs/chrono/0.4.22/chrono/naive/struct.NaiveDateTime.html#method.parse_from_str
//! [`strftime`]: https://docs.rs/chrono/0.4.22/chrono/format/strftime/index.html
//! [`DateTimeParseInstr`]: crate::data::datetime::DateTimeParseInstr
//! [Regular Expression syntax]: https://docs.rs/regex/1.6.0/regex/index.html#syntax
//! [`bytes_to_regex_to_datetime`]: crate::data::datetime::bytes_to_regex_to_datetime
//! [`captures_to_buffer_bytes`]: crate::data::datetime::captures_to_buffer_bytes

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

#[cfg(any(debug_assertions, test))]
use crate::debug::printers::{buffer_to_String_noraw, str_to_String_noraw};

#[doc(hidden)]
pub use crate::data::line::{LineIndex, Range_LineIndex};

use std::collections::BTreeMap;

#[cfg(any(debug_assertions, test))]
use std::collections::HashMap;

use std::fmt;

#[doc(hidden)]
pub use std::time::{SystemTime, UNIX_EPOCH};

extern crate arrayref;
use arrayref::array_ref;

extern crate chrono;
#[doc(hidden)]
pub use chrono::{
    DateTime,
    Datelike, // adds method `.year()` onto `DateTime`
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
use more_asserts::{assert_le, debug_assert_ge, debug_assert_le, debug_assert_lt};

extern crate regex;
use regex::bytes::Regex;

extern crate si_trace_print;
#[allow(unused_imports)]
use si_trace_print::{defn, defo, defx, defñ, den, deo, dex, deñ};

extern crate unroll;
use unroll::unroll_for_loops;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DateTime Regex Matching and strftime formatting
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A _Year_ in a date
pub type Year = i32;

/// Crate `chrono` [`strftime`] formatting pattern, passed to
/// chrono [`DateTime::parse_from_str`] or [`NaiveDateTime::parse_from_str`].
///
/// Specific `const` instances of `DateTimePattern_str` are hardcoded in
/// [`captures_to_buffer_bytes`].
///
/// [`strftime`]: https://docs.rs/chrono/0.4.22/chrono/format/strftime/index.html
/// [`DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.22/chrono/struct.DateTime.html#method.parse_from_str
/// [`NaiveDateTime::parse_from_str`]: https://docs.rs/chrono/0.4.22/chrono/naive/struct.NaiveDateTime.html#method.parse_from_str
/// [`captures_to_buffer_bytes`]: captures_to_buffer_bytes
pub type DateTimePattern_str = str;

/// Regular expression formatting pattern, passed to [`regex::bytes::Regex`].
///
/// [`regex::bytes::Regex`]: https://docs.rs/regex/1.6.0/regex/bytes/struct.Regex.html
pub type DateTimeRegex_str = str;

/// Regular expression capture group name, used within the regular expression and
/// for later retrieval via [`regex::captures.name`].
///
/// [`regex::captures.name`]: https://docs.rs/regex/1.6.0/regex/bytes/struct.Captures.html#method.name
pub type CaptureGroupName = str;

/// Regular expression capture group pattern, used within a [`RegexPattern`].
pub type CaptureGroupPattern = str;

/// A regular expression, passed to [`regex::bytes::Regex::captures`].
///
/// [`regex::bytes::Regex::captures`]: https://docs.rs/regex/1.6.0/regex/bytes/struct.Regex.html#method.captures
pub type RegexPattern = str;

/// The regular expression "class" used here, specifically for matching datetime substrings
/// within a [`&str`](str).
pub type DateTimeRegex = Regex;

/// A chrono [`DateTime`] type used in _s4lib_.
///
/// [`DateTime`]: https://docs.rs/chrono/0.4.22/chrono/struct.DateTime.html
// TODO: rename to `DateTimeS4`
pub type DateTimeL = DateTime<FixedOffset>;
pub type DateTimeLOpt = Option<DateTimeL>;

/// FixedOffset seconds
#[cfg(test)]
type fos = i32;

#[cfg(any(debug_assertions,test))]
lazy_static! {
    static ref FO_0: FixedOffset = FixedOffset::east_opt(0).unwrap();
}

#[cfg(test)]
pub(crate) const YEAR_FALLBACKDUMMY_VAL: i32 = 1972;
/// For datetimes missing a year, in some circumstances a filler year must be
/// used.
///
/// First leap year after Unix Epoch.
///
/// XXX: using leap year as a filler might help handle 'Feb 29' dates without a
///      year but it is not guaranteed. It depends on the file modified time
///      (i.e. [`blockreader.mtime()`](BlockReader)) being true.
const YEAR_FALLBACKDUMMY: &str = "1972";

/// Convert a `T` to a [`SystemTime`].
///
/// [`SystemTime`]: std::time::SystemTime
pub fn convert_to_systemtime<T>(epoch_seconds: T)
    -> SystemTime where u64: From<T>
{
    UNIX_EPOCH + std::time::Duration::from_secs(epoch_seconds.try_into().unwrap())
}

/// create a `DateTime`
///
/// wrapper for chrono DateTime creation function
#[cfg(test)]
pub fn ymdhms(
    fixedoffset: &FixedOffset,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
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

/// create a `DateTime` with milliseconds
///
/// wrapper for chrono DateTime creation function
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub fn ymdhmsl(
    fixedoffset: &FixedOffset,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
    milli: i64,
) -> DateTimeL {
    fixedoffset.with_ymd_and_hms(
        year,
        month,
        day,
        hour,
        min,
        sec,
    )
    .unwrap()
    + Duration::milliseconds(milli)
}

/// create a `DateTime` with microseconds
///
/// wrapper for chrono DateTime creation function
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub fn ymdhmsm(
    fixedoffset: &FixedOffset,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
    micro: i64,
) -> DateTimeL {
    fixedoffset.with_ymd_and_hms(
        year,
        month,
        day,
        hour,
        min,
        sec,
    )
    .unwrap()
    + Duration::microseconds(micro)
}

/// create a `DateTime` with nanoseconds
///
/// wrapper for chrono DateTime creation function
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub fn ymdhmsn(
    fixedoffset: &FixedOffset,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
    nano: i64,
) -> DateTimeL {
    fixedoffset
    .with_ymd_and_hms(
        year,
        month,
        day,
        hour,
        min,
        sec,
    )
    .unwrap()
    + Duration::nanoseconds(nano)
}

// abbreviate *y*ear *d*ummy
#[cfg(test)]
pub(crate) const YD: i32 = YEAR_FALLBACKDUMMY_VAL;

// Offset UTC/Z
#[cfg(test)]
const O_Z: fos = 0;
#[cfg(test)]
const O_0: fos = 0;
// Offset Local
// symbolic value for Local time, replaced at runtime
#[cfg(test)]
pub(crate) const O_L: fos = i32::max_value();
// Offset Minus (or West)
#[cfg(test)]
const O_M1: fos = -3600;
#[cfg(test)]
const O_M2: fos = -2 * 3600;
#[cfg(test)]
const O_M3: fos = -3 * 3600;
#[cfg(test)]
const O_M330: fos = -3 * 3600 - 30 * 60;
#[allow(dead_code)]
#[cfg(test)]
const O_M4: fos = -4 * 3600;
#[cfg(test)]
const O_M5: fos = -5 * 3600;
#[allow(dead_code)]
#[cfg(test)]
const O_M6: fos = -6 * 3600;
#[cfg(test)]
const O_M7: fos = -7 * 3600;
#[cfg(test)]
const O_M730: fos = -7 * 3600 - 30 * 60;
#[cfg(test)]
const O_M8: fos = -8 * 3600;
#[cfg(test)]
const O_M9: fos = -9 * 3600;
#[cfg(test)]
const O_M10: fos = -10 * 3600;
#[cfg(test)]
const O_M1030: fos = -10 * 3600 - 30 * 60;
#[cfg(test)]
const O_M11: fos = -11 * 3600;
#[cfg(test)]
const O_M1130: fos = -11 * 3600 - 30 * 60;
#[allow(dead_code)]
#[cfg(test)]
const O_M12: fos = -12 * 3600;
#[allow(dead_code)]
#[cfg(test)]
const O_M1230: fos = -12 * 3600 - 30 * 60;
// Offset Plus (or East)
#[cfg(test)]
const O_P1: fos = 3600;
#[cfg(test)]
const O_P130: fos = 3600 + 30 * 60;
#[allow(dead_code)]
#[cfg(test)]
const O_P2: fos = 2 * 3600;
#[cfg(test)]
const O_P3: fos = 3 * 3600;
#[allow(dead_code)]
#[cfg(test)]
const O_P4: fos = 4 * 3600;
#[cfg(test)]
const O_P5: fos = 5 * 3600;
#[cfg(test)]
const O_P6: fos = 6 * 3600;
#[allow(dead_code)]
#[cfg(test)]
const O_P7: fos = 7 * 3600;
#[cfg(test)]
const O_P8: fos = 8 * 3600;
#[cfg(test)]
const O_P9: fos = 9 * 3600;
#[cfg(test)]
const O_P945: fos = 9 * 3600 + 45 * 60;
#[cfg(test)]
const O_P10: fos = 10 * 3600;
#[cfg(test)]
const O_P11: fos = 11 * 3600;
#[cfg(test)]
const O_P12: fos = 12 * 3600;
#[cfg(test)]
const O_P1230: fos = 12 * 3600 + 30 * 60;
// misc. named timezones
#[cfg(test)]
const O_ALMT: fos = O_P6;
#[cfg(test)]
const O_CHUT: fos = O_P10;
#[cfg(test)]
const O_CIST: fos = O_M8;
#[cfg(test)]
const O_PDT: fos = O_M7;
#[cfg(test)]
const O_PETT: fos = O_P12;
#[cfg(test)]
const O_PET: fos = O_M5;
#[cfg(test)]
const O_PONT: fos = O_P11;
#[cfg(test)]
const O_PST: fos = O_M8;
#[cfg(test)]
const O_PWT: fos = O_P9;
#[cfg(test)]
const O_VLAT: fos = O_P10;
#[cfg(test)]
const O_WAT: fos = O_P1;
#[cfg(test)]
const O_WITA: fos = O_P8;
#[cfg(test)]
const O_WIT: fos = O_P9;
#[cfg(test)]
const O_WGST: fos = O_M2;
#[cfg(test)]
const O_WST: fos = O_P8;
#[cfg(test)]
const O_YAKT: fos = O_P9;
#[cfg(test)]
const O_YEKT: fos = O_P5;

/// tuple of arguments for function `ymdhmsn`
#[cfg(test)]
pub(crate) type ymdhmsn_args = (
    /* fixedoffset: */ fos,
    /* year: */ i32,
    /* month: */ u32,
    /* day: */ u32,
    /* hour: */ u32,
    /* min: */ u32,
    /* sec: */ u32,
    /* nano: */ i64,
);

#[cfg(test)]
pub(crate) const DUMMY_ARGS: ymdhmsn_args = (0, 1972, 1, 1, 0, 0, 0, 123456789);

/*
selective copy of chrono `strftime` specifier reference table
copied from https://docs.rs/chrono/0.4.22/chrono/format/strftime/index.html

DATE SPECIFIERS:

%Y  2001    The full proleptic Gregorian year, zero-padded to 4 digits.
%C  20      The proleptic Gregorian year divided by 100, zero-padded to 2 digits.
%y  01      The proleptic Gregorian year modulo 100, zero-padded to 2 digits.

%m  07      Month number (01–12), zero-padded to 2 digits.
%b  Jul     Abbreviated month name. Always 3 letters.
%B  July    Full month name. Also accepts corresponding abbreviation in parsing.

%d  08      Day number (01–31), zero-padded to 2 digits.
%e  8       Same as %d but space-padded. Same as %_d.

%a  Sun     Abbreviated weekday name. Always 3 letters.
%A  Sunday  Full weekday name. Also accepts corresponding abbreviation in parsing.
%w  0       Sunday = 0, Monday = 1, …, Saturday = 6.
%u  7       Monday = 1, Tuesday = 2, …, Sunday = 7. (ISO 8601)

TIME SPECIFIERS:

%H  00  Hour number (00–23), zero-padded to 2 digits.
%k  0   Same as %H but space-padded. Same as %_H.
%I  12  Hour number in 12-hour clocks (01–12), zero-padded to 2 digits.
%l  12  Same as %I but space-padded. Same as %_I.

%P  am  am or pm in 12-hour clocks.
%p  AM  AM or PM in 12-hour clocks.

%M  34  Minute number (00–59), zero-padded to 2 digits.

%S  60  Second number (00–60), zero-padded to 2 digits.

%f      026490000   The fractional seconds (in nanoseconds) since last whole second.
%.f     .026490     Similar to .%f but left-aligned. These all consume the leading dot.
%.3f    .026        Similar to .%f but left-aligned but fixed to a length of 3.
%.6f    .026490     Similar to .%f but left-aligned but fixed to a length of 6.
%.9f    .026490000  Similar to .%f but left-aligned but fixed to a length of 9.
%3f     026         Similar to %.3f but without the leading dot.
%6f     026490      Similar to %.6f but without the leading dot.
%9f     026490000   Similar to %.9f but without the leading dot.

TIME ZONE SPECIFIERS:

%Z  ACST    Local time zone name. Skips all non-whitespace characters during parsing.
%z  +0930   Offset from the local time to UTC (with UTC being +0000).
%:z +09:30  Same as %z but with a colon.
%#z +09     Parsing only: Same as %z but allows minutes to be missing or present.

%s  994518299   UNIX timestamp, the number of seconds since 1970-01-01 00:00 UTC.

SPECIAL SPECIFIERS:

%t  Literal tab (\t).
%n  Literal newline (\n).
%%  Literal percent sign.
*/

// TODO: [2022/10] Issue #26
//       refactor this `datetime.rs` to remove intermediary `DTP_*` variables
//       allow more flexible regex grouping and name declarations.

/// DateTime Format Specifier for a Year.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Year {
    /// %Y
    Y,
    /// %y
    y,
    /// none provided, must be filled.
    /// the associated `pattern` should use "%Y`
    _fill,
}

/// DateTime Format Specifier for a Month.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Month {
    /// %m, month numbers 00 to 12
    m,
    /// %b, month abbreviated to three characters.
    b,
    /// %B, month full name, transformed to form `%b` in
    /// function `month_bB_to_month_m_bytes` called by
    /// function `captures_to_buffer_bytes`
    B,
}

/// DateTime Format Specifier for a Day.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Day {
    /// `%d`, day number 01 to 31
    /// `%e`, day number 1 to 31
    /// `%d` (" 8" or "08") captured will be changed to `%e` ("08") in
    /// `fn captures_to_buffer_bytes`
    _e_or_d,
}

/// DateTime Format Specifier for an Hour.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Hour {
    /// %H, 24 hour, 00 to 23
    H,
    /// %k, 24 hour, 0 to 23
    k,
    /// %I, 12 hour, 01 to 12
    I,
    /// %l, 12 hour, 1 to 12
    l,
}

/// DateTime Format Specifier for a Minute.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Minute {
    /// %M, 00 to 59
    M,
}

/// DateTime Format Specifier for a Second.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Second {
    /// %S, 00 to 60
    S,
    /// fill with value `0`
    _fill,
}

/// DateTime Format Specifier for a Fractional or fractional second.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Fractional {
    /// %f, subsecond decimal digits
    f,
    /// none, will not be filled
    _none,
}

/// DateTime Format Specifier for a Timezone.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Tz {
    /// `%z` numeric timezone offset, e.g. `"+0930"`
    z,
    /// `%:z` numeric timezone offset with colon, e.g. `"+09:30"` ("zee colon")
    zc,
    /// `%#z`numeric timezone offset shortened, e.g. `"+09"` ("zee pound")
    zp,
    /// `%Z` named timezone offset, e.g. `"PST"`
    Z,
    /// none, must be filled.
    /// the associated `pattern` should use `%:z` (variable name substring `Zc`)
    /// as that is the form displayed by
    /// `chrono::FixedOffset::east(0).as_string().to_str()`
    _fill,
}

/// `DTFSSet`, "DateTime Format Specifer Set", is essentially instructions
/// to transcribe regex [`named capture groups`] to a
/// chrono [`strftime`]-ready string,
/// and ultimately a [`DateTimeL`] instance.
///
/// Given extracted regular expression named capture groups
/// `<year>`, `<month>`, `<day>`, etc. (see `CGN_` vars),
/// then what is the format of each such that the data can be readied and then
/// passed to [`chrono::DateTime::parse_from_str`]?
/// These are effectively mappings to receive extracting datetime substrings
/// in a [`&str`](str) then to rearrange those into order suitable for
/// [`captures_to_buffer_bytes`].
///
/// Given the following code for capturing and enumerating some named capture
/// groups:
/// ```rust
/// extern crate regex;
/// use regex::Regex;
/// extern crate chrono;
/// use chrono::{
///   NaiveDateTime,
///   NaiveDate,
/// };
/// fn main() {
///     let data = r"[2020/Mar/05 12:17:59.631000 PMDT] ../source3/smbd/oplock.c:1340(init_oplocks)";
///     let pattern = r"^\[(?P<year>[12][[:digit:]]{3})[ /\-]?(?P<month>(?i)01|02|03|04|05|06|07|08|09|10|11|12|jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec(?-i))[ /\-]?(?P<day>01|02|03|04|05|06|07|08|09|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31)[ T]?(?P<hour>00|01|02|03|04|05|06|07|08|09|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24)[:]?(?P<minute>[012345][[:digit:]])[:]?(?P<second>[0123456][[:digit:]])[\.,](?P<subsecond>[[:digit:]]{3,9})[[:blank:]](?P<tz>ACDT|ACST|ACT|ADT|AEDT|AEST|AET|AFT|AKDT|AKST|ALMT|AMST|AMT|ANAT|AQTT|ART|AST|AWST|AZOT|AZT|BIOT|BIT|BNT|BOT|BRST|BRT|BST|BTT|CAT|CCT|CDT|CEST|CET|CHOT|CHST|CHUT|CIST|CKT|CLST|CLT|COST|COT|CST|CT|CVT|CWST|CXT|DAVT|DDUT|DFT|EAST|EAT|ECT|EDT|EEST|EET|EGST|EGT|EST|ET|FET|FJT|FKST|FKT|FNT|GALT|GAMT|GET|GFT|GILT|GIT|GMT|GST|GYT|HAEC|HDT|HKT|HMT|HOVT|HST|ICT|IDLW|IDT|IOT|IRDT|IRKT|IRST|IST|JST|KALT|KGT|KOST|KRAT|KST|LHST|LINT|MAGT|MART|MAWT|MDT|MEST|MET|MHT|MIST|MIT|MMT|MSK|MST|MUT|MVT|MYT|NCT|NDT|NFT|NOVT|NPT|NST|NT|NUT|NZDT|NZST|OMST|ORAT|PDT|PETT|PET|PGT|PHOT|PHST|PHT|PKT|PMDT|PMST|PONT|PST|PWT|PYST|PYT|RET|ROTT|SAKT|SAMT|SAST|SBT|SCT|SDT|SGT|SLST|SRET|SRT|SST|SYOT|TAHT|TFT|THA|TJT|TKT|TLT|TMT|TOT|TRT|TVT|ULAT|UTC|UYST|UYT|UZT|VET|VLAT|VOLT|VOST|VUT|WAKT|WAST|WAT|WEST|WET|WGST|WGT|WIB|WITA|WIT|WST|YAKT|YEKT)[^[[:upper:]]]";
///     let re = Regex::new(pattern).unwrap();
///     let captures = match re.captures(data) {
///         Some(cap) => cap,
///         None => panic!("re.captures failed"),
///     };
///     for (i, name_opt) in re.capture_names().enumerate() {
///         let match_ = match captures.get(i) {
///             Some(m_) => m_,
///             None => {
///                 match name_opt {
///                     Some(name) => {
///                         eprintln!("{} {:?} None", i, name);
///                     },
///                     None => {
///                         eprintln!("{} None None", i);
///                     }
///                 }
///                 continue;
///             }
///         };
///         match name_opt {
///             Some(name) => {
///                 eprintln!("{} {:?} {:?}", i, name, match_.as_str());
///             },
///             None => {
///                 eprintln!("{} unnamed {:?}", i, match_.as_str());
///             }
///         }
///     }
/// }
/// ```
/// [(Rust Playground)],
///
/// should print:
/// ```text
/// index name        value
/// 0     unnamed     "[2020/Mar/05 12:17:59.631000 PMDT]"
/// 1     "year"      "2020"
/// 2     "month"     "Mar"
/// 3     "day"       "05"
/// 4     "hour"      "12"
/// 5     "minute"    "17"
/// 6     "second"    "59"
/// 7     "subsecond" "631000"
/// 8     "tz"        "PMDT"
/// ```
///
/// A `DTFSSset` provides "instructions" to transform and then pass those
/// string values to chrono `parse_from_str`.
///
/// The `DTFSSset` instance for this example should be:
///
/// ```ignore
/// DTFSSet {
///     year: DTFS_Year::Y,     // example value was `"2020"`
///     month: DTFS_Month::b,   // example value was `"Mar"`
///     day: DTFS_Day::_e_or_d, // example value was `"05"`
///     hour: DTFS_Hour::H,     // example value was `"12"`
///     minute: DTFS_Minute::M, // example value was `"17"`
///     second: DTFS_Second::S, // example value was `"59"`
///     fractional: DTFS_Fractional::_none, // example value did not have a fractional
///     tz: DTFS_Tz::_fill,     // example value did not have a timezone, it will be filled with the default, or fallback, timezone (which can be passed by the user via `--tz-offset`)
///     pattern: "%Y%m%dT%H%M%S%:z", // strftime specifier pattern, notice the %m ?
/// };
/// ```
///
/// Here is the tricky part: function `captures_to_buffer_bytes` transforms
/// some values. In the example case, value `"Mar"` is written to a buffer
/// as `"03"`. The timezone value was not captured, so the default
/// timezone offset value is written to the same buffer.
/// That buffer is passed to function `datetime_parse_from_str`
/// which, in this case, calls chrono [`DateTime::parse_from_str`] (
/// function `datetime_parse_from_str` might
/// call [`NaiveDateTime::parse_from_str`] in other cases).
///
/// The enum values `DTFS_*` are interdependent with the value of `pattern`.
/// The `pattern` is a chrono `strftime` specifier formatting string
/// passed to chrono `datetime_parse_from_str`.
///
/// ---
///
/// All `DTFSSet` instances are `const`.
///
/// All `DTFSSet.pattern` take from `const` declared variables `DTP_*`.
///
/// Strictly, there are 192 permutations of `DTFSSet`.
/// In practice, only a subset is encountered in real-life syslog files.
/// Furthermore, some regex capture data is modified to be only one type.
/// For example, capture group _day_ will capture pattern specifier for
/// `%e` (`" 8"`) and `%d` (`"08"`).
/// The captured data will be modified to strftime day format `%d`,
/// e.g. captured data `" 8"` becomes `"08"` before passing to `parse_from_str`.
///
/// Each `DTFSSet` is checked for internal consistency within test
/// `test_DATETIME_PARSE_DATAS_builtin` (as much as reasonably possible).
///
/// [`named capture groups`]: https://docs.rs/regex/1.6.0/regex/bytes/struct.Captures.html
/// [`chrono::DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.22/chrono/struct.DateTime.html#method.parse_from_str
/// [`DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.22/chrono/struct.DateTime.html#method.parse_from_str
/// [`NaiveDateTime::parse_from_str`]: https://docs.rs/chrono/0.4.22/chrono/naive/struct.NaiveDateTime.html#method.parse_from_str
/// [`strftime`]: https://docs.rs/chrono/0.4.22/chrono/format/strftime/index.html
/// [(Rust Playground)]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=00460112beb2a6d078d6bbba72557574
#[derive(Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct DTFSSet<'a> {
    pub year: DTFS_Year,
    pub month: DTFS_Month,
    pub day: DTFS_Day,
    pub hour: DTFS_Hour,
    pub minute: DTFS_Minute,
    pub second: DTFS_Second,
    pub fractional: DTFS_Fractional,
    pub tz: DTFS_Tz,
    /// strftime pattern passed to [`chrono::DateTime::parse_from_str`] or
    /// [`chrono::NaiveDateTime::parse_from_str`]
    /// in function [`datetime_parse_from_str`]. Directly relates to order of capture group extractions and `push_str`
    /// done in private `captures_to_buffer_bytes`.
    ///
    /// `pattern` is interdependent with other members.
    ///
    /// Tested in test `test_DATETIME_PARSE_DATAS_builtin`.
    ///
    /// [`chrono::DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.22/chrono/struct.DateTime.html#method.parse_from_str
    /// [`chrono::NaiveDateTime::parse_from_str`]: https://docs.rs/chrono/0.4.22/chrono/naive/struct.NaiveDate.html#method.parse_from_str
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
            DTFS_Tz::z | DTFS_Tz::zc | DTFS_Tz::zp | DTFS_Tz::Z => true,
            DTFS_Tz::_fill => false,
        }
    }
}

impl fmt::Debug for DTFSSet<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        let mut f_ = f.debug_struct("DTFSSet:");
        f_.field("year", &self.year)
            .field("month", &self.month)
            .field("day", &self.day)
            .field("hour", &self.hour)
            .field("second", &self.second)
            .field("fractional", &self.fractional)
            .field("tz", &self.tz)
            .field("pattern", &self.pattern)
            ;

        f_.finish()
    }
}

/// `Instr`uctions for `pars`ing from some unknown [`bytes`](u8) to a
/// [`regex::Regex.captures`] instance to a `&str` value that can be passed to
/// [`chrono::DateTime::parse_from_str`] or
/// [`chrono::NaiveDateTime::parse_from_str`].
///
/// An explanation of a `DateTimeParseInstr` instance:
///
/// 1. All `DateTimeParseInstr` instances are declared within the array
///   [`pub const DATETIME_PARSE_DATAS`].
/// 2. The `DateTimeParseInstr.regex_pattern` is a `&str` for regex matching some
///    line of text from the processed file.
/// 3. The `DateTimeParseInstr.dtfs` are like instructions for taking the
///    regex capture group values, `regex::Regex.captures`, and transforming
///    those into a single `&str` value that can be processed by
///    `chrono::DateTime::parse_from_str` or
///    `chrono::NaiveDateTime::parse_from_str`.
///    See [`DTFSSet`].
/// 4. The `DateTimeParseInstr.range_regex` is used to slice data provided by
///    a [`Line`].
///    Some lines can have many bytes, so this shortens the amount of time
///    the regex spends matching (regex matching is an resource expensive
///    operation).
///    Also, syslogs have a bias toward placing
///    the syslog datetime stamp at the front of the line. slicing the front
///    of the line, for example, the first 50 bytes, makes it less likely an
///    errant match would be made further into the syslog line. e.g. a syslog
///    message may include a datetime string unrelated to the datetime
///    of that syslog message.
/// 5. `DateTimeParseInstr.cgn_first` and `DateTimeParseInstr.cgn_last` are the
///    first and last regex capture groups within the
///    `DateTimeParseInstr.regex_pattern`. These are used to help determine
///    where a datetime substring occurred within the given line. For exampe,
///    given line `"INFO: 2019/01/22 07:55:38 hello!"`, the first regex named
///    capture group is the year, `<year>` (at `"2"`).
///    The year data begins at byte offset 5.
///    The last named capture group is the second, `<second>`.
///    The second data begins at byte offset 23 and, more importantly,
///    ends at byte offset 25 (one byte after `"8"`).
///    Later, in function `bytes_to_regex_to_datetime`, the offsets are
///    returned as a pair, `(Some(5, 25))`.
///    These offsets values are stored by the controlling
///    [`SyslineReader`], and later passed to a [`SyslinePrinter`] which
///    highlights the datetime substring within the line (if `--color` is
///    enabled).
///
/// A `DateTimeParseInstr` instance is declared with macro [`DTPD!`].
///
/// The values within a `DateTimeParseInstr` instance are mostly entirely
/// interdependent and tricky to declare correctly.
/// The test `test_DATETIME_PARSE_DATAS_builtin`
/// checks for as many irregularities as it can find.
/// The test `test_DATETIME_PARSE_DATAS_test_cases` processes entries in
/// array `DateTimeParseInstr._test_cases`. It checks that
/// a `DateTime` instance is returned, and does a few other checks.
///
/// [`regex::Regex.captures`]: https://docs.rs/regex/1.6.0/regex/bytes/struct.Captures.html
/// [`chrono::DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.22/chrono/struct.DateTime.html#method.parse_from_str
/// [`chrono::NaiveDateTime::parse_from_str`]: https://docs.rs/chrono/0.4.22/chrono/naive/struct.NaiveDateTime.html#method.parse_from_str
/// [chrono `strftime`]: https://docs.rs/chrono/0.4.22/chrono/format/strftime/index.html
/// [`pub const DATETIME_PARSE_DATAS`]: DATETIME_PARSE_DATAS
/// [`DTFSSet`]: DTFSSet
/// [`SyslineReader`]: crate::readers::syslinereader::SyslineReader
/// [`SyslinePrinter`]: crate::printer::printers::SyslinePrinter
/// [`Line`]: crate::data::line::Line
/// [`DTPD!`]: DTPD!
// TODO: [2022/10] each _test_cases entry should include an expected chrono
//       DateTime value that is compared to the processed value.
#[derive(Hash)]
pub struct DateTimeParseInstr<'a> {
    /// Regex pattern for [`captures`].
    ///
    /// [`captures`]: https://docs.rs/regex/1.6.0/regex/bytes/struct.Regex.html#method.captures
    pub regex_pattern: &'a DateTimeRegex_str,
    /// In what `strftime` form are the regex `regex_pattern` capture groups?
    pub dtfs: DTFSSet<'a>,
    /// Slice range of widest regex pattern match.
    ///
    /// This range is sliced from the [`Line`] and then a [`Regex`] match is
    /// attempted using it. It must be at least contain the datetime string to
    /// match. It may contain extra characters before or after the datetime
    /// (assuming the `regex_pattern` is correct).
    ///
    /// Attempting a `Regex` match on a smaller subset slice of a `Line`,
    /// instead of the entire `Line`, can significantly improve run-time
    /// performance.
    ///
    /// [`Line`]: crate::data::line::Line
    /// [`Regex`]: https://docs.rs/regex/1.6.0/regex/bytes/struct.Regex.html#method.captures
    pub range_regex: Range_LineIndex,
    /// Capture named group first (left-most) position in `regex_pattern`.
    pub cgn_first: &'a CaptureGroupName,
    /// Capture named group last (right-most) position in `regex_pattern`.
    pub cgn_last: &'a CaptureGroupName,
    /// Hardcoded self-test cases.
    #[cfg(test)]
    pub _test_cases: &'a [(LineIndex, LineIndex, ymdhmsn_args, &'a str)],
    /// Source code line number of declaration.
    /// Only to aid humans reviewing failing tests.
    pub _line_num: u32,
}

/// Declare a [`DateTimeParseInstr`] tuple more easily.
///
/// `$test_cases` are not compiled into the release build.
// TODO: [2023/01] replace name `DTPD` with `DTPI`
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
            range_regex: Range_LineIndex {
                start: $sib,
                end: $sie,
            },
            cgn_first: $cgn_first,
            cgn_last: $cgn_last,
            #[cfg(test)]
            _test_cases: $test_cases,
            _line_num: $line_num,
        }
    };
}
// Allow easy macro import via `use s4lib::data::datetime::DTPD;`
pub use DTPD;

// TODO: [2022/10] create macro for declaring individual test cases,
//       e.g. `DTPDT!`
//       adding `line()` to each test (print the line during test failure),
//       Also, instantiate a `DateTime` instance to compare the result.
//       Pass the datetime initial data as numbers, like
//       (2000, 1, 2, 0, 0, 55, 342), that progressively sets parts of the
//       `DateTime`. There is a macro in chrono that makes this easy.

/// Implement ordering traits to allow sorting collections of
/// `DateTimeParseInstr`.
///
/// Only used for tests.
impl Ord for DateTimeParseInstr<'_> {
    fn cmp(
        &self,
        other: &Self,
    ) -> std::cmp::Ordering {
        (self.regex_pattern, &self.dtfs).cmp(&(other.regex_pattern, &other.dtfs))
    }
}

impl PartialOrd for DateTimeParseInstr<'_> {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DateTimeParseInstr<'_> {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.regex_pattern == other.regex_pattern && self.dtfs == other.dtfs
    }
}

impl Eq for DateTimeParseInstr<'_> {}

impl fmt::Debug for DateTimeParseInstr<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        // regexp strings can be very long, truncate it
        const MAXLEN: usize = 20;
        let mut rp: String = String::with_capacity(MAXLEN + 5);
        rp.extend(
            self.regex_pattern
                .chars()
                .take(MAXLEN),
        );
        if self.regex_pattern.len() > MAXLEN {
            rp.push('…');
        }
        let mut f_ = f.debug_struct("DateTimeParseInstr:");
        f_.field("regex_pattern", &rp)
            .field("line", &self._line_num)
            .field("range_regex", &self.range_regex)
            .field("cgn_first", &self.cgn_first)
            .field("cgn_last", &self.cgn_last)
            .field("", &self.dtfs);

        f_.finish()
    }
}

// `strftime` patterns used in `DTFSSet!` declarations

// TODO: [2022/10/08] refactor for consistent naming of  `DTP_*` variables:
//       put 'Y' in front, so it matches
//       strftime specifier ordering within the value.
//       e.g. variable `DTP_BdHMSYz` has value `"%Y%m%dT%H%M%S%z"`, the `%Y`
//       is in front, so the variable should match the ordering, `DTP_YBdHMSz`.
//       a few less human brain cycles to grok the var.

// TODO: [2022/10/10] refactor for consistent naming of timezone in variables
//       names. Sometimes it is `DTP_YmdHMSzc` (notice `zc`) but then there
//       is `DTP_bdHMSYZc` (noticed `Zc`).

const DTP_YmdHMSzc: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
const DTP_YmdHMSz: &DateTimePattern_str = "%Y%m%dT%H%M%S%z";
const DTP_YmdHMSzp: &DateTimePattern_str = "%Y%m%dT%H%M%S%#z";
const DTP_YmdHMSfzc: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%:z";
const DTP_YmdHMSfz: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%z";
const DTP_YmdHMSfzp: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%#z";

/// no second, chrono will set to value 0
const DTP_mdHMYZc: &DateTimePattern_str = "%Y%m%dT%H%M%:z";

/// `%Y` `%:z` is filled, `%B` value transformed to `%m` value by [`captures_to_buffer_bytes`]
const DTP_BdHMS: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
/// `%Y` is filled, `%Z` transformed to `%:z`, `%B` value transformed to `%m` value by [`captures_to_buffer_bytes`]
const DTP_BdHMSZ: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
/// `%:z` is filled, `%B` value transformed to `%m` value by [`captures_to_buffer_bytes`]
const DTP_BdHMSY: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
/// `%Z` transformed to `%:z`, `%B` value transformed to `%m` value by [`captures_to_buffer_bytes`]
const DTP_BdHMSYZ: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
/// `%B` value transformed to `%m` value by [`captures_to_buffer_bytes`]
const DTP_BdHMSYzp: &DateTimePattern_str = "%Y%m%dT%H%M%S%#z";

/// `%b` value transformed to `%m` value by [`captures_to_buffer_bytes`]
const DTP_bdHMSYZc: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
/// `%b` value transformed to `%m` value by [`captures_to_buffer_bytes`]
const DTP_bdHMSyZc: &DateTimePattern_str = "%y%m%dT%H%M%S%:z";
/// `%b` value transformed to `%m` value by [`captures_to_buffer_bytes`]
const DTP_bdHMSYZp: &DateTimePattern_str = "%Y%m%dT%H%M%S%#z";
/// `%b` value transformed to `%m` value by [`captures_to_buffer_bytes`]
const DTP_bdHMSYZz: &DateTimePattern_str = "%Y%m%dT%H%M%S%z";

/// `%b` value transformed to `%m` value by [`captures_to_buffer_bytes`]
const DTP_bdHMSYfZc: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%:z";
/// `%b` value transformed to `%m` value by [`captures_to_buffer_bytes`]
const DTP_bdHMSYfZp: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%#z";
/// `%b` value transformed to `%m` value by [`captures_to_buffer_bytes`]
const DTP_bdHMSYfZz: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%z";

/// `%b` value transformed to `%m` value,
/// `%Z` transformed to `%:z` by [`captures_to_buffer_bytes`]
const DTP_bdHMSYZ: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
/// `%b` value transformed to `%m` value,
/// `%:z` filled by [`captures_to_buffer_bytes`]
const DTP_bdHMSY: &DateTimePattern_str = "%Y%m%dT%H%M%S%:z";
/// `%b` value transformed to `%m` value,
/// `%:z` filled by [`captures_to_buffer_bytes`]
const DTP_bdHMSYf: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%:z";

// The variable name represents what is available. The value represents it's rearranged form
// using in function `captures_to_buffer_bytes`.

const DTFSS_YmdHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    pattern: DTP_YmdHMSzc,
};
const DTFSS_YmdHMSz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::z,
    pattern: DTP_YmdHMSz,
};
const DTFSS_YmdHMSzc: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zc,
    pattern: DTP_YmdHMSzc,
};
const DTFSS_YmdHMSzp: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zp,
    pattern: DTP_YmdHMSzp,
};
const DTFSS_YmdHMSZ: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
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
    day: DTFS_Day::_e_or_d,
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
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::z,
    pattern: DTP_YmdHMSfz,
};
const DTFSS_YmdHMSfzc: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::zc,
    pattern: DTP_YmdHMSfzc,
};
const DTFSS_YmdHMSfzp: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::zp,
    pattern: DTP_YmdHMSfzp,
};
const DTFSS_YmdHMSfZ: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
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
    day: DTFS_Day::_e_or_d,
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
    day: DTFS_Day::_e_or_d,
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
    day: DTFS_Day::_e_or_d,
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
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::Z,
    pattern: DTP_BdHMSYZ,
};
const DTFSS_BdHMSYz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::z,
    pattern: DTP_BdHMSYZ,
};
const DTFSS_BdHMSYzc: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zc,
    pattern: DTP_BdHMSYZ,
};
const DTFSS_BdHMSYzp: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zp,
    pattern: DTP_BdHMSYzp,
};

const DTFSS_bdHMSY: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    pattern: DTP_bdHMSY,
};
const DTFSS_bdHMSYf: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::_fill,
    pattern: DTP_bdHMSYf,
};
const DTFSS_bdHMSYZ: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::Z,
    pattern: DTP_bdHMSYZ,
};
const DTFSS_bdHMSYz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::z,
    pattern: DTP_bdHMSYZz,
};
const DTFSS_bdHMSYzc: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zc,
    pattern: DTP_bdHMSYZc,
};
const DTFSS_bdHMSYzp: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zp,
    pattern: DTP_bdHMSYZp,
};
const DTFSS_bdHMSYfz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::z,
    pattern: DTP_bdHMSYfZz,
};
const DTFSS_bdHMSYfzc: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::zc,
    pattern: DTP_bdHMSYfZc,
};
const DTFSS_bdHMSYfzp: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::zp,
    pattern: DTP_bdHMSYfZp,
};

const DTFSS_YbdHMSzc: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zc,
    pattern: DTP_bdHMSYZc,
};
const DTFSS_YbdHMSzp: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zp,
    pattern: DTP_bdHMSYZp,
};
const DTFSS_YbdHMSz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::z,
    pattern: DTP_bdHMSYZz,
};
const DTFSS_YbdHMSZ: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::Z,
    pattern: DTP_bdHMSYZ,
};
const DTFSS_YbdHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    pattern: DTP_bdHMSYZc,
};

const DTFSS_ybdHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    pattern: DTP_bdHMSyZc,
};

const DTFSS_YmdHM: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::_fill,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    pattern: DTP_mdHMYZc,
};

// TODO: Issue #4 handle dmesg
// special case for `dmesg` syslog lines
//pub(crate) const DTFSS_u: DTFSSet = DTFSSet {
//    year: DTFS_Year::_fill,
//    month: DTFS_Month::m,
//    day: DTFS_Day::_e_or_d,
//    hour: DTFS_Hour::H,
//    minute: DTFS_Minute::M,
//    second: DTFS_Second::S,
//    fractional: DTFS_Fractional::_none,
//    tz: DTFS_Tz::_fill,
//    pattern: DTP_YmdHMSzc,
//};

/// to aid testing
// check `DTP_ALL` has all `DTP_` vars
//
//     grep -Fe ' DTP_' ./src/data/datetime.rs  | grep const | grep -oEe 'DTP_[[:alnum:]]+' | sed 's/$/,/'
//
#[doc(hidden)]
#[cfg(test)]
pub(crate) const DTP_ALL: &[&DateTimePattern_str] = &[
    DTP_YmdHMSzc,
    DTP_YmdHMSz,
    DTP_YmdHMSzp,
    DTP_YmdHMSfzc,
    DTP_YmdHMSfz,
    DTP_YmdHMSfzp,
    DTP_BdHMS,
    DTP_BdHMSZ,
    DTP_BdHMSY,
    DTP_BdHMSYZ,
    DTP_BdHMSYzp,
    DTP_BdHMS,
    DTP_BdHMSZ,
    DTP_BdHMSY,
    DTP_BdHMSYZ,
];

// `regex::Captures` capture group names

/// corresponds to `strftime` specifier `%Y`
const CGN_YEAR: &CaptureGroupName = "year";
/// corresponds to `strftime` specifier `%m`
const CGN_MONTH: &CaptureGroupName = "month";
/// corresponds to `strftime` specifier `%d`
const CGN_DAY: &CaptureGroupName = "day";
/// corresponds to `strftime` specifier `%a`
const CGN_DAYa: &CaptureGroupName = "dayIgnore";
/// corresponds to `strftime` specifier `%H`
const CGN_HOUR: &CaptureGroupName = "hour";
/// corresponds to `strftime` specifier `%M`
const CGN_MINUTE: &CaptureGroupName = "minute";
/// corresponds to `strftime` specifier `%S`
const CGN_SECOND: &CaptureGroupName = "second";
/// corresponds to `strftime` specifier `%f`
const CGN_FRACTIONAL: &CaptureGroupName = "fractional";
/// corresponds to `strftime` specifier `%Z`, `%z`, `%:z`, `%#z`
const CGN_TZ: &CaptureGroupName = "tz";
// special case: `dmesg` uptime
//const CGN_UPTIME: &CaptureGroupName = "uptime";

/// all capture group names, for testing
#[doc(hidden)]
#[cfg(test)]
pub(crate) const CGN_ALL: [&CaptureGroupName; 9] = [
    CGN_YEAR,
    CGN_MONTH,
    CGN_DAY,
    CGN_DAYa,
    CGN_HOUR,
    CGN_MINUTE,
    CGN_SECOND,
    CGN_FRACTIONAL,
    CGN_TZ,
    //CGN_UPTIME,
];

// saved rust playground for quick testing regex patterns
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=00460112beb2a6d078d6bbba72557574

// Names used in the upcoming capture group pattern variable values (`CGP_*`) *MUST*
// match the values of previous capture group name values (`CGN_*`).

/// Regex capture group pattern for `strftime` year specifier `%Y`, as
/// four decimal number characters.
pub const CGP_YEAR: &CaptureGroupPattern = r"(?P<year>[12][[:digit:]]{3})";
/// Regex capture group pattern for `strftime` year specifier `%y`, as
/// two decimal number characters.
pub const CGP_YEARy: &CaptureGroupPattern = r"(?P<year>[[:digit:]]{2})";
/// Regex capture group pattern for `strftime` month specifier `%m`,
/// month numbers `"01"` to `"12"`.
pub const CGP_MONTHm: &CaptureGroupPattern = r"(?P<month>01|02|03|04|05|06|07|08|09|10|11|12)";
/// Regex capture group pattern for `strftime` month specifier `%b`,
/// month name abbreviated to three characters, e.g. `Jan`.
pub const CGP_MONTHb: &CaptureGroupPattern = r"(?P<month>jan|Jan|JAN|feb|Feb|FEB|mar|Mar|MAR|apr|Apr|APR|may|May|MAY|jun|Jun|JUN|jul|Jul|JUL|aug|Aug|AUG|sep|Sep|SEP|oct|Oct|OCT|nov|Nov|NOV|dec|Dec|DEC)";
/// Regex capture group pattern for `strftime` month specifier `%B`,
/// month name long, e.g. `January`.
pub const CGP_MONTHB: &CaptureGroupPattern = r"(?P<month>january|January|JANUARY|february|February|FEBRUARY|march|March|MARCH|april|April|APRIL|may|May|MAY|june|June|JUNE|july|July|JULY|august|August|AUGUST|september|September|SEPTEMBER|october|October|OCTOBER|november|November|NOVEMBER|december|December|DECEMBER)";
/// Regex capture group pattern for `strftime` month specifier `%B` and `%b`,
/// e.g. `January` or `Jan`.
pub const CGP_MONTHBb: &CaptureGroupPattern = r"(?P<month>january|January|JANUARY|jan|Jan|JAN|february|February|FEBRUARY|feb|Feb|FEB|march|March|MARCH|mar|Mar|MAR|april|April|APRIL|apr|Apr|APR|may|May|MAY|june|June|JUNE|jun|Jun|JUN|july|July|JULY|jul|Jul|JUL|august|August|AUGUST|aug|Aug|AUG|september|September|SEPTEMBER|sep|Sep|SEP|october|October|OCTOBER|oct|Oct|OCT|november|November|NOVEMBER|nov|Nov|NOV|december|December|DECEMBER|dec|Dec|DEC)";
/// Regex capture group pattern for `strftime` day specifier `%d`,
/// number day of month with leading zero, e.g. `"02"` or `"31"`.
/// Regex capture group pattern for `strftime` day specifier `%e`,
/// number day of month, 1 to 31, e.g. `"2"` or `"31"`.
/// Transformed to equivalent `%d` form within function
/// `captures_to_buffer_bytes` (i.e. `'0'` is prepended if necessary).
pub const CGP_DAYde: &CaptureGroupPattern = r"(?P<day>01|02|03|04|05|06|07|08|09|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|1|2|3|4|5|6|7|8|9| 1| 2| 3| 4| 5| 6| 7| 8| 9)";
/// Regex capture group pattern for `strftime` day specifier `%a`,
/// named day of week, either long name or abbreviated three character name,
/// e.g. `"Mon"` or `"Monday"`.
pub const CGP_DAYa: &RegexPattern = r"(?P<dayIgnore>monday|Monday|MONDAY|mon|Mon|MON|tuesday|Tuesday|TUESDAY|tue|Tue|TUE|wednesday|Wednesday|WEDNESDAY|wed|Wed|WED|thursday|Thursday|THURSDAY|thu|Thu|THU|friday|Friday|FRIDAY|fri|Fri|FRI|saturday|Saturday|SATURDAY|sat|Sat|SAT|sunday|Sunday|SUNDAY|sun|Sun|SUN)";
/// Regex capture group pattern for `strftime` hour specifier `%H`, 00 to 24.
pub const CGP_HOUR: &CaptureGroupPattern =
    r"(?P<hour>00|01|02|03|04|05|06|07|08|09|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24)";
/// Regex capture group pattern for `strftime` hour specifier `%h`, 1 to 12.
pub const CGP_HOURh: &CaptureGroupPattern = r"(?P<hour>|1|2|3|4|5|6|7|8|9|10|11|12)";
/// Regex capture group pattern for `strftime` minute specifier `%M`, 00 to 59.
pub const CGP_MINUTE: &CaptureGroupPattern = r"(?P<minute>[012345][[:digit:]])";
/// Regex capture group pattern for `strftime` second specifier `%S`, 00 to 60.
/// Includes leap second "60".
pub const CGP_SECOND: &CaptureGroupPattern = r"(?P<second>[012345][[:digit:]]|60)";
/// Regex capture group pattern for `strftime` fractional specifier `%f`.
/// Matches all `strftime` specifiers `%f`, `%3f`, `%6f`, and `%9f`, a sequence
/// of decimal number characters.
///
/// Function `datetime_parse_from_str` will match with strftime specifier `%f`.
/// Function `captures_to_buffer_bytes` will fill a too short or too long
/// fractionals to 9 digits to match the correct precision.
/// For example, fractional data "123" is transformed to "123000000" in
/// function `captures_to_buffer_bytes`. Then it is parsed by
/// `datetime_parse_from_str` using `%f` specifier.
pub const CGP_FRACTIONAL: &CaptureGroupPattern = r"(?P<fractional>[[:digit:]]{1,9})";
/// Like [`CGP_FRACTIONAL`] but only matches 3 digits, `%3f`.
pub const CGP_FRACTIONAL3: &CaptureGroupPattern = r"(?P<fractional>[[:digit:]]{3})";
/// Regex capture group pattern for dmesg uptime fractional seconds in logs
//pub const CGP_UPTIME: &CaptureGroupPattern = r"(?P<uptime>[[:digit:]]{1,9}\.[[:digit:]]{3,9})";

/// for testing
#[doc(hidden)]
#[cfg(test)]
pub(crate) const CGP_MONTH_ALL: &[&CaptureGroupPattern] = &[
    CGP_MONTHm,
    CGP_MONTHb,
    CGP_MONTHB,
    CGP_MONTHBb,
];

/// for testing
#[doc(hidden)]
#[cfg(test)]
pub(crate) const CGP_DAY_ALL: &[&CaptureGroupPattern] = &[CGP_DAYde];

// Regarding timezone formatting, ISO 8601 allows Unicode "minus sign".
// See https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1114291504#Time_offsets_from_UTC
// Unicode "minus sign" will be replaced with ASCII "hyphen-minus" for
// processing by chrono `DateTime::parse_from_str`.
// See https://github.com/chronotope/chrono/issues/835

/// Unicode "minus sign"
const MINUS_SIGN: &[u8] = "−".as_bytes();
/// Unicode/ASCII "hyphen-minus"
const HYPHEN_MINUS: &[u8] = "-".as_bytes();

// The "|" operator will match in order. And pattern ordering matters.
// So patterns should attempt longer matches first, e.g.
//   Pattern "PETT" should occur before "PET"; "PETT|PET"

/// `strftime` specifier `%z` e.g. `"+0930"`
const CGP_TZz: &CaptureGroupPattern = r"(?P<tz>[\+\-−][012][[:digit:]]{3})";
/// `strftime` specifier `%:z` e.g. `"+09:30"`
const CGP_TZzc: &CaptureGroupPattern = r"(?P<tz>[\+\-−][012][[:digit:]]:[[:digit:]]{2})";
/// `strftime` specifier `%#z` e.g. `"+09"`
const CGP_TZzp: &CaptureGroupPattern = r"(?P<tz>[\+\-−][012][[:digit:]])";
/// `strftime` specifier `%Z` e.g. `"ACST"`, all lowercase also allowed
pub(crate) const CGP_TZZ: &CaptureGroupPattern = "(?P<tz>\
ACDT|ACST|ACT|ADT|AEDT|AEST|AET|AFT|AKDT|AKST|ALMT|AMST|AMT|ANAT|AQTT|ART|AST|AWST|AZOST|AZOT|AZT|BIOT|BIT|BNT|BOT|BRST|BRT|BST|BTT|CAT|CCT|CDT|CEST|CET|CHOST|CHOT|CHST|CHUT|CIST|CKT|CLST|CLT|COST|COT|CST|CT|CVT|CWST|CXT|DAVT|DDUT|DFT|EASST|EAST|EAT|ECT|EDT|EEST|EET|EGST|EGT|EST|ET|FET|FJT|FKST|FKT|FNT|GALT|GAMT|GET|GFT|GILT|GIT|GMT|GST|GYT|HAEC|HDT|HKT|HMT|HOVST|HOVT|HST|ICT|IDLW|IDT|IOT|IRDT|IRKT|IRST|IST|JST|KALT|KGT|KOST|KRAT|KST|LHST|LINT|MAGT|MART|MAWT|MDT|MEST|MET|MHT|MIST|MIT|MMT|MSK|MST|MUT|MVT|MYT|NCT|NDT|NFT|NOVT|NPT|NST|NT|NUT|NZDT|NZST|OMST|ORAT|PDT|PETT|PET|PGT|PHOT|PHST|PHT|PKT|PMDT|PMST|PONT|PST|PWT|PYST|PYT|RET|ROTT|SAKT|SAMT|SAST|SBT|SCT|SDT|SGT|SLST|SRET|SRT|SST|SYOT|TAHT|TFT|THA|TJT|TKT|TLT|TMT|TOT|TRT|TVT|ULAST|ULAT|UTC|UYST|UYT|UZT|VET|VLAT|VOLT|VOST|VUT|WAKT|WAST|WAT|WEST|WET|WGST|WGT|WIB|WITA|WIT|WST|YAKT|YEKT|ZULU|Z|\
acdt|acst|act|adt|aedt|aest|aet|aft|akdt|akst|almt|amst|amt|anat|aqtt|art|ast|awst|azost|azot|azt|biot|bit|bnt|bot|brst|brt|bst|btt|cat|cct|cdt|cest|cet|chost|chot|chst|chut|cist|ckt|clst|clt|cost|cot|cst|ct|cvt|cwst|cxt|davt|ddut|dft|easst|east|eat|ect|edt|eest|eet|egst|egt|est|et|fet|fjt|fkst|fkt|fnt|galt|gamt|get|gft|gilt|git|gmt|gst|gyt|haec|hdt|hkt|hmt|hovst|hovt|hst|ict|idlw|idt|iot|irdt|irkt|irst|ist|jst|kalt|kgt|kost|krat|kst|lhst|lint|magt|mart|mawt|mdt|mest|met|mht|mist|mit|mmt|msk|mst|mut|mvt|myt|nct|ndt|nft|novt|npt|nst|nt|nut|nzdt|nzst|omst|orat|pdt|pett|pet|pgt|phot|phst|pht|pkt|pmdt|pmst|pont|pst|pwt|pyst|pyt|ret|rott|sakt|samt|sast|sbt|sct|sdt|sgt|slst|sret|srt|sst|syot|taht|tft|tha|tjt|tkt|tlt|tmt|tot|trt|tvt|ulast|ulat|utc|uyst|uyt|uzt|vet|vlat|volt|vost|vut|wakt|wast|wat|west|wet|wgst|wgt|wib|wita|wit|wst|yakt|yekt|zulu|z\
)";

/// hardcoded listing of named timezone abbreviations
#[cfg(any(debug_assertions, test))]
pub(crate) const TZZ_LIST_UPPER: &[&str] = &[
    "ACDT", "ACST", "ACT", "ACWST", "ADT", "AEDT", "AEST", "AET", "AFT", "AKDT", "AKST", "ALMT", "AMST",
    "AMT", "ANAT", "AQTT", "ART", "AST", "AWST", "AZOST", "AZOT", "AZT", "BIOT", "BIT", "BNT", "BOT", "BRST",
    "BRT", "BST", "BTT", "CAT", "CCT", "CDT", "CEST", "CET", "CHADT", "CHAST", "CHOST", "CHOT", "CHST",
    "CHUT", "CIST", "CKT", "CLST", "CLT", "COST", "COT", "CST", "CT", "CVT", "CWST", "CXT", "DAVT", "DDUT",
    "DFT", "EASST", "EAST", "EAT", "ECT", "EDT", "EEST", "EET", "EGST", "EGT", "EST", "ET", "FET", "FJT",
    "FKST", "FKT", "FNT", "GALT", "GAMT", "GET", "GFT", "GILT", "GIT", "GMT", "GST", "GYT", "HAEC", "HDT",
    "HKT", "HMT", "HOVST", "HOVT", "HST", "ICT", "IDLW", "IDT", "IOT", "IRDT", "IRKT", "IRST", "IST", "JST",
    "KALT", "KGT", "KOST", "KRAT", "KST", "LHST", "LINT", "MAGT", "MART", "MAWT", "MDT", "MEST", "MET",
    "MHT", "MIST", "MIT", "MMT", "MSK", "MST", "MUT", "MVT", "MYT", "NCT", "NDT", "NFT", "NOVT", "NPT",
    "NST", "NT", "NUT", "NZDT", "NZST", "OMST", "ORAT", "PDT", "PETT", "PET", "PGT", "PHOT", "PHST", "PHT",
    "PKT", "PMDT", "PMST", "PONT", "PST", "PWT", "PYST", "PYT", "RET", "ROTT", "SAKT", "SAMT", "SAST", "SBT",
    "SCT", "SDT", "SGT", "SLST", "SRET", "SRT", "SST", "SYOT", "TAHT", "TFT", "THA", "TJT", "TKT", "TLT",
    "TMT", "TOT", "TRT", "TVT", "ULAST", "ULAT", "UTC", "UYST", "UYT", "UZT", "VET", "VLAT", "VOLT", "VOST",
    "VUT", "WAKT", "WAST", "WAT", "WEST", "WET", "WGST", "WGT", "WIB", "WITA", "WIT", "WST", "YAKT", "YEKT",
    "ZULU", "Z",
];

/// lowercase version of [`TZZ_LIST_UPPER`]
#[cfg(any(debug_assertions, test))]
pub(crate) const TZZ_LIST_LOWER: &[&str] = &[
    "acdt", "acst", "act", "acwst", "adt", "aedt", "aest", "aet", "aft", "akdt", "akst", "almt", "amst",
    "amt", "anat", "aqtt", "art", "ast", "awst", "azost", "azot", "azt", "biot", "bit", "bnt", "bot", "brst",
    "brt", "bst", "btt", "cat", "cct", "cdt", "cest", "cet", "chadt", "chast", "chost", "chot", "chst",
    "chut", "cist", "ckt", "clst", "clt", "cost", "cot", "cst", "ct", "cvt", "cwst", "cxt", "davt", "ddut",
    "dft", "easst", "east", "eat", "ect", "edt", "eest", "eet", "egst", "egt", "est", "et", "fet", "fjt",
    "fkst", "fkt", "fnt", "galt", "gamt", "get", "gft", "gilt", "git", "gmt", "gst", "gyt", "haec", "hdt",
    "hkt", "hmt", "hovst", "hovt", "hst", "ict", "idlw", "idt", "iot", "irdt", "irkt", "irst", "ist", "jst",
    "kalt", "kgt", "kost", "krat", "kst", "lhst", "lint", "magt", "mart", "mawt", "mdt", "mest", "met",
    "mht", "mist", "mit", "mmt", "msk", "mst", "mut", "mvt", "myt", "nct", "ndt", "nft", "novt", "npt",
    "nst", "nt", "nut", "nzdt", "nzst", "omst", "orat", "pdt", "pett", "pet", "pgt", "phot", "phst", "pht",
    "pkt", "pmdt", "pmst", "pont", "pst", "pwt", "pyst", "pyt", "ret", "rott", "sakt", "samt", "sast", "sbt",
    "sct", "sdt", "sgt", "slst", "sret", "srt", "sst", "syot", "taht", "tft", "tha", "tjt", "tkt", "tlt",
    "tmt", "tot", "trt", "tvt", "ulast", "ulat", "utc", "uyst", "uyt", "uzt", "vet", "vlat", "volt", "vost",
    "vut", "wakt", "wast", "wat", "west", "wet", "wgst", "wgt", "wib", "wita", "wit", "wst", "yakt", "yekt",
    "zulu", "z",
];

#[cfg(any(debug_assertions, test))]
lazy_static! {
    /// map lowercase `%Z` timezones (e.g. "pst") to uppercase (e.g. "PST")
    pub(crate) static ref TZZ_LOWER_TO_UPPER: HashMap<&'static str, &'static str> = {
        assert_eq!(TZZ_LIST_UPPER.len(), TZZ_LIST_LOWER.len());
        let mut map = HashMap::<&'static str, &'static str>::new();
        map.reserve(TZZ_LIST_LOWER.len());
        for (index, tz_lower) in TZZ_LIST_LOWER.iter().enumerate() {
            let tz_upper = TZZ_LIST_UPPER[index];
            map.insert(tz_lower, tz_upper);
        }

        map
    };
}

/// for testing
#[doc(hidden)]
#[cfg(test)]
pub(crate) const CGP_TZ_ALL: &[&CaptureGroupPattern] = &[
    CGP_TZz, CGP_TZzc, CGP_TZzp, CGP_TZZ,
];

/// no alphabetic or line end, helper to `CGP_TZZ`
const RP_NOALPHA: &RegexPattern = r"([^[[:alpha:]]]|$|^)";

/// no alphanumeric or line end, helper to `CGP_TZZ` and and `CGP_YEAR`
const RP_NOALNUM: &RegexPattern = r"([^[[:alnum:]]]|$|^)";

/// no alphanumeric plus minus or line end, helper to `CGP_TZZ` and `CGP_YEAR`
const RP_NOALNUMpm: &RegexPattern = r"([^[[:alnum:]]\+\-]|$|^)";

/// no numeric or line end, helper to `CGP_TZZ` and `CGP_YEAR`
const RP_NODIGIT: &RegexPattern = r"([^[[:digit:]]]|$|^)";

/// one or more digits
pub const RP_DIGITS: &RegexPattern = "[[:digit:]]+";

/// one to three digits
pub const RP_DIGITS3: &RegexPattern = r"[[:digit:]]{1,3}";

/// All named timezone abbreviations, maps all chrono strftime `%Z` values
/// (e.g. `"EDT"`) to equivalent `%:z` value (e.g. `"-04:00"`).
///
/// _Super Speedy Syslog Searcher_ attempts to be more lenient than chrono
/// about matching named abbreviated timezones, e.g. `"EDT"`.
/// Chrono provides `%Z` strftime specifier
/// yet rejects named timezones when passed to [`DateTime::parse_from_str`].
/// `TZZ_ALL` provides the necessary mapping.
///
/// However, due to duplicate timezone names, some valid timezone names
/// will result in the default timezone. For example, there are three named
/// timezones `"IST"` that refer to different timezone offsets. If `"IST"` is
/// parsed as a timezone in a sysline then the resultant value will be the
/// default timezone offset value, e.g. the value passed to `--tz-offset`.
/// See the opening paragraph in [_List of time zone abbreviations_].
///
/// The listing of timezone abbreviations and values can be scraped from
/// Wikipedia with this code snippet:
///
/// ```text
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
///
/// See also:
/// - Applicable tz offsets <https://en.wikipedia.org/wiki/List_of_UTC_offsets>
/// - Applicable tz abbreviations <https://en.wikipedia.org/wiki/List_of_time_zone_abbreviations>
///
/// [_List of time zone abbreviations_]: https://en.wikipedia.org/w/index.php?title=List_of_time_zone_abbreviations&oldid=1106679802
/// [`DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.22/chrono/format/strftime/#fn7

pub const TZZ_ALL: [(&str, &str); 420] = [
    // uppercase
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
    ("AZOST", "+00:00"),
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
    ("ZULU", "+00:00"),
    ("Z", "+00:00"),
    // lowercase
    ("acdt", "+10:30"),
    ("acst", "+09:30"),
    ("act", "-05:00"),
    ("act", "+08:00"),
    ("acwst", "+08:45"),
    ("adt", "-03:00"),
    ("aedt", "+11:00"),
    ("aest", "+10:00"),
    ("aet", "+11:00"),
    ("aft", "+04:30"),
    ("akdt", "-08:00"),
    ("akst", "-09:00"),
    ("almt", "+06:00"),
    ("amst", "-03:00"),
    ("amt", "-04:00"),
    ("amt", "+04:00"),
    ("anat", "+12:00"),
    ("aqtt", "+05:00"),
    ("art", "-03:00"),
    ("ast", "+03:00"),
    ("ast", "-04:00"),
    ("awst", "+08:00"),
    ("azost", "-00:00"),
    ("azot", "-01:00"),
    ("azt", "+04:00"),
    ("bnt", "+08:00"),
    ("biot", "+06:00"),
    ("bit", "-12:00"),
    ("bot", "-04:00"),
    ("brst", "-02:00"),
    ("brt", "-03:00"),
    ("bst", "+06:00"),
    ("bst", "+11:00"),
    ("bst", "+01:00"),
    ("btt", "+06:00"),
    ("cat", "+02:00"),
    ("cct", "+06:30"),
    ("cdt", "-05:00"),
    ("cdt", "-04:00"),
    ("cest", "+02:00"),
    ("cet", "+01:00"),
    ("chadt", "+13:45"),
    ("chast", "+12:45"),
    ("chot", "+08:00"),
    ("chost", "+09:00"),
    ("chst", "+10:00"),
    ("chut", "+10:00"),
    ("cist", "-08:00"),
    ("ckt", "-10:00"),
    ("clst", "-03:00"),
    ("clt", "-04:00"),
    ("cost", "-04:00"),
    ("cot", "-05:00"),
    ("cst", "-06:00"),
    ("cst", "+08:00"),
    ("cst", "-05:00"),
    ("ct", "-05:00"),
    ("cvt", "-01:00"),
    ("cwst", "+08:45"),
    ("cxt", "+07:00"),
    ("davt", "+07:00"),
    ("ddut", "+10:00"),
    ("dft", "+01:00"),
    ("easst", "-05:00"),
    ("east", "-06:00"),
    ("eat", "+03:00"),
    ("ect", "-04:00"),
    ("ect", "-05:00"),
    ("edt", "-04:00"),
    ("eest", "+03:00"),
    ("eet", "+02:00"),
    ("egst", "-00:00"),
    ("egt", "-01:00"),
    ("est", "-05:00"),
    ("et", "-04:00"),
    ("fet", "+03:00"),
    ("fjt", "+12:00"),
    ("fkst", "-03:00"),
    ("fkt", "-04:00"),
    ("fnt", "-02:00"),
    ("galt", "-06:00"),
    ("gamt", "-09:00"),
    ("get", "+04:00"),
    ("gft", "-03:00"),
    ("gilt", "+12:00"),
    ("git", "-09:00"),
    ("gmt", "-00:00"),
    ("gst", "-02:00"),
    ("gst", "+04:00"),
    ("gyt", "-04:00"),
    ("hdt", "-09:00"),
    ("haec", "+02:00"),
    ("hst", "-10:00"),
    ("hkt", "+08:00"),
    ("hmt", "+05:00"),
    ("hovst", "+08:00"),
    ("hovt", "+07:00"),
    ("ict", "+07:00"),
    ("idlw", "-12:00"),
    ("idt", "+03:00"),
    ("iot", "+03:00"),
    ("irdt", "+04:30"),
    ("irkt", "+08:00"),
    ("irst", "+03:30"),
    ("ist", "+05:30"),
    ("ist", "+01:00"),
    ("ist", "+02:00"),
    ("jst", "+09:00"),
    ("kalt", "+02:00"),
    ("kgt", "+06:00"),
    ("kost", "+11:00"),
    ("krat", "+07:00"),
    ("kst", "+09:00"),
    ("lhst", "+10:30"),
    ("lhst", "+11:00"),
    ("lint", "+14:00"),
    ("magt", "+12:00"),
    ("mart", "-09:30"),
    ("mawt", "+05:00"),
    ("mdt", "-06:00"),
    ("met", "+01:00"),
    ("mest", "+02:00"),
    ("mht", "+12:00"),
    ("mist", "+11:00"),
    ("mit", "-09:30"),
    ("mmt", "+06:30"),
    ("msk", "+03:00"),
    ("mst", "+08:00"),
    ("mst", "-07:00"),
    ("mut", "+04:00"),
    ("mvt", "+05:00"),
    ("myt", "+08:00"),
    ("nct", "+11:00"),
    ("ndt", "-02:30"),
    ("nft", "+11:00"),
    ("novt", "+07:00"),
    ("npt", "+05:45"),
    ("nst", "-03:30"),
    ("nt", "-03:30"),
    ("nut", "-11:00"),
    ("nzdt", "+13:00"),
    ("nzst", "+12:00"),
    ("omst", "+06:00"),
    ("orat", "+05:00"),
    ("pdt", "-07:00"),
    ("pet", "-05:00"),
    ("pett", "+12:00"),
    ("pgt", "+10:00"),
    ("phot", "+13:00"),
    ("pht", "+08:00"),
    ("phst", "+08:00"),
    ("pkt", "+05:00"),
    ("pmdt", "-02:00"),
    ("pmst", "-03:00"),
    ("pont", "+11:00"),
    ("pst", "-08:00"),
    ("pwt", "+09:00"),
    ("pyst", "-03:00"),
    ("pyt", "-04:00"),
    ("ret", "+04:00"),
    ("rott", "-03:00"),
    ("sakt", "+11:00"),
    ("samt", "+04:00"),
    ("sast", "+02:00"),
    ("sbt", "+11:00"),
    ("sct", "+04:00"),
    ("sdt", "-10:00"),
    ("sgt", "+08:00"),
    ("slst", "+05:30"),
    ("sret", "+11:00"),
    ("srt", "-03:00"),
    ("sst", "-11:00"),
    ("sst", "+08:00"),
    ("syot", "+03:00"),
    ("taht", "-10:00"),
    ("tha", "+07:00"),
    ("tft", "+05:00"),
    ("tjt", "+05:00"),
    ("tkt", "+13:00"),
    ("tlt", "+09:00"),
    ("tmt", "+05:00"),
    ("trt", "+03:00"),
    ("tot", "+13:00"),
    ("tvt", "+12:00"),
    ("ulast", "+09:00"),
    ("ulat", "+08:00"),
    ("utc", "-00:00"),
    ("uyst", "-02:00"),
    ("uyt", "-03:00"),
    ("uzt", "+05:00"),
    ("vet", "-04:00"),
    ("vlat", "+10:00"),
    ("volt", "+03:00"),
    ("vost", "+06:00"),
    ("vut", "+11:00"),
    ("wakt", "+12:00"),
    ("wast", "+02:00"),
    ("wat", "+01:00"),
    ("west", "+01:00"),
    ("wet", "-00:00"),
    ("wib", "+07:00"),
    ("wit", "+09:00"),
    ("wita", "+08:00"),
    ("wgst", "-02:00"),
    ("wgt", "-03:00"),
    ("wst", "+08:00"),
    ("yakt", "+09:00"),
    ("yekt", "+05:00"),
    ("zulu", "+00:00"),
    ("z", "+00:00"),
];

type Map_TZZ_to_TZz<'a> = BTreeMap<&'a str, &'a str>;

lazy_static! {
    /// Map of all `%Z` values, e.g. `"PST"` or `"pst"`,
    /// to the `%:z` value, e.g. `"-07:00"`.
    /// A key with an `is_empty()` value signifies an ambiguous timezone.
    /// Ambiguous timezones will use the provided fallback timezone offset
    /// (i.e. CLI option `--tz-offset`).
    ///
    /// Ambiguous timezone names have their values set to empty string,
    /// e.g. `"SST"` maps to `""`. See [Issue #59].
    ///
    /// [Issue #59]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/59
    // must be `pub` to allow access from `src/bin/bin.rs`
    pub static ref MAP_TZZ_TO_TZz: Map_TZZ_to_TZz<'static> = {
        let mut map = Map_TZZ_to_TZz::new();
        #[allow(non_snake_case)]
        for tzZ_zc in TZZ_ALL.iter() {
            if map.insert(tzZ_zc.0, tzZ_zc.1).is_some() {
                // duplicate key entries are set to empty string
                map.insert(tzZ_zc.0, "");
            }
        }

        map
    };
}

/// [`RegexPattern`] divider _date?_ `2020/01/01` or `2020-01-01` or
/// `2020 01 01` or `20200101`
const D_Dq: &RegexPattern = r"[ /\-]?";
/// [`RegexPattern`] divider _date_, `2020/01/01` or `2020-01-01` or
/// `2020 01 01`
const D_D: &RegexPattern = r"[ /\-]";
/// [`RegexPattern`] divider _time_, `20:30:00`
const D_T: &RegexPattern = "[:]?";
/// [`RegexPattern`] divider _time_ with extras, `20:30:00` or `20-00`
const D_Te: &RegexPattern = r"[:\-]?";
/// [`RegexPattern`] divider _day_ to _hour_, `2020/01/01T20:30:00`
const D_DHq: &RegexPattern = "[ T]?";
/// [`RegexPattern`] divider _day_ to _hour_ with dash, `2020:01:01-20:30:00`.
const D_DHdq: &RegexPattern = r"[ T\-]?";
/// [`RegexPattern`] divider _day_ to _hour_ with colon or dash,
/// `2020:01:01-20:30:00`.
const D_DHcdq: &RegexPattern = r"[ T\-:]?";
/// [`RegexPattern`] divider _day_ to _hour_ with colon or dash or underline,
/// `2020:01:01_20:30:00`.
const D_DHcdqu: &RegexPattern = r"[ T\-:_]?";
/// [`RegexPattern`] divider _fractional_, `2020/01/01T20:30:00,123456`
const D_SF: &RegexPattern = r"[\.,]";

/// [`RegexPattern`] dot or comma?
const RP_dcq: &RegexPattern = r"[\.,]?";
/// [`RegexPattern`] comma?
const RP_cq: &RegexPattern = "[,]?";
/// [`RegexPattern`] of commonly found syslog level names
///
/// References:
/// - <https://www.rfc-editor.org/rfc/rfc5427#section-3>
/// - <https://learningnetwork.cisco.com/s/article/syslog-severity-amp-level>
/// - <https://learningnetwork.cisco.com/s/feed/0D53i00000KsKHECA3>
/// - <https://success.trendmicro.com/dcx/s/solution/TP000086250>
const RP_LEVELS: &RegexPattern = r"((?i)DEBUG[[[:digit:]]]|DEBUG|INFO[[[:digit:]]]|INFO|ERROR[[[:digit:]]]|ERROR|ERR|TRACE[[[:digit:]]]|TRACE|WARN[[[:digit:]]]|WARN|WARNING|VERBOSE[[[:digit:]]]|VERBOSE|EMERGENCY|EMERG|NOTICE|CRIT|CRITICAL|ALERT[[[:digit:]]]|ALERT(?-i))";
/// [`RegexPattern`] blank
const RP_BLANK: &RegexPattern = "[[:blank:]]";
/// [`RegexPattern`] blank?
const RP_BLANKq: &RegexPattern = "[[:blank:]]?";
/// [`RegexPattern`] blank or end
const RP_BLANKe: &RegexPattern = "([[:blank:]]|$)";
/// [`RegexPattern`] blank, 1 or 2
const RP_BLANK12: &RegexPattern = r"[[:blank:]]{1,2}";
/// [`RegexPattern`] blank, 1 or 2?
const RP_BLANK12q: &RegexPattern = r"([[:blank:]]{1,2})?";
/// [`RegexPattern`] blanks
const RP_BLANKS: &RegexPattern = "[[:blank:]]+";
/// [`RegexPattern`] blanks?
const RP_BLANKSq: &RegexPattern = "[[:blank:]]*";
/// [`RegexPattern`] anything plus
const RP_ANYp: &RegexPattern = ".+";
/// [`RegexPattern`] left-side brackets
pub(crate) const RP_LB: &RegexPattern = r"[\[\(<{]";
/// [`RegexPattern`] right-side brackets
pub(crate) const RP_RB: &RegexPattern = r"[\]\)>}]";

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// the global list of built-in Datetime parsing "instructions"
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Index into the global [`DATETIME_PARSE_DATAS`]
pub type DateTimeParseInstrsIndex = usize;

/// A run-time created vector of [`DateTimeRegex`] instances that is a
/// counterpart to [`DATETIME_PARSE_DATAS`]
pub type DateTimeParseInstrsRegexVec = Vec<DateTimeRegex>;

/// Length of [`DATETIME_PARSE_DATAS`]
// XXX: do not forget to update `#[test_case()]` for test `test_DATETIME_PARSE_DATAS_test_cases`
//      in `datetime_tests.rs`. Should have test cases, `#[test_case(XX)]`, for values `0` to
//      `DATETIME_PARSE_DATAS_LEN-1`.
pub const DATETIME_PARSE_DATAS_LEN: usize = 104;

/// Built-in [`DateTimeParseInstr`] datetime parsing patterns.
///
/// These are all regular expression patterns that will be attempted on
/// each [`Line`] of a processed file.
///
/// Order of declaration matters: during initial parsing of a syslog file, all
/// of these regex patterns are attempted in order of their declaration.
/// Listing a general regex pattern before a specific regex pattern may result
/// in a loss of datetime information.
///
/// For example, given sysline
/// ```text
/// 2001-02-03T04:05:06 -1100 hello
/// ```
///
/// A regex that attempts to match from year to second (and not the timezone),
/// will match `"2001-02-03T04:05:06"`, dropping the timezone information.
/// This will result in a filler timezone being used which may not
/// be correct. Generally, more specific regex patterns should be listed before
/// general regex patterns.
///
/// Notice that local sequences of `DateTimeParseInstr`
/// generally match from more specific to more general
/// to no timezone. i.e. match attempt ordering is
/// `%:z` (`"-04:00"`), to `%z` (`"-0400"`),
/// to `%#z` (`"-04"`), to `%Z` (`"EDT"`),
/// to no timezone.
///
/// A drawback of this specific-to-general approach:
/// during [`SyslineReader`] initial reading stage,
/// it will try *all* the patterns (from index 0 of
/// `DATETIME_PARSE_DATAS` to wherever it finds a match).
/// So if a file has a datetime pattern that matches the last entry in
/// `DATETIME_PARSE_DATAS` then the `SyslineReader` will try *all*
/// the `DateTimeParseInstr` within `DATETIME_PARSE_DATAS` several times.
/// Once the controlling `SyslineReader` calls
/// [`dt_patterns_analysis`] for the last time, then only one `DateTimeParseInstr`
/// is tried for each `Line`.
/// But until `dt_patterns_analysis` is called, there will be many missed
/// matches, and regular expression matching uses a large amount of compute
/// and time resources.
///
/// [`SyslineReader`]: crate::readers::syslinereader::SyslineReader
/// [`dt_patterns_analysis`]: crate::readers::syslinereader::SyslineReader#method.dt_patterns_analysis
/// [`Line`]: crate::data::line::Line
// XXX: yet another rust playground for testing regex
//      https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=93a47ceb225bae73dfcbce2574b65e91
// TODO: [2023/01/14] add test of shortest possible match for all DTPD!
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
        concatcp!("^", RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_RB),
        DTFSS_YmdHMSf, 0, 40, CGN_YEAR, CGN_FRACTIONAL,
        &[
            (1, 24, (O_L, 2000, 1, 1, 0, 0, 1, 123000000), "[2000/01/01 00:00:01.123] ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 27, (O_L, 2000, 1, 1, 0, 0, 1, 123456000), "[2000/01/01 00:00:01.123456] ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 30, (O_L, 2000, 1, 1, 0, 0, 1, 123456789), "[2000/01/01 00:00:01.123456789] ../source3/smbd/oplock.c:1340(init_oplocks)"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZz, RP_RB),
        DTFSS_YmdHMSfz, 0, 40, CGN_YEAR, CGN_TZ,
        &[
            (1, 28, (O_M11, 2000, 1, 1, 0, 0, 2, 100000000), "(2000/01/01 00:00:02.1 -1100) ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 29, (O_M11, 2000, 1, 1, 0, 0, 2, 120000000), "(2000/01/01 00:00:02.12 -1100) ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 30, (O_M11, 2000, 1, 1, 0, 0, 2, 123000000), "(2000/01/01 00:00:02.123 -1100) ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 33, (O_M11, 2000, 1, 1, 0, 0, 2, 123456000), "(2000/01/01 00:00:02.123456 -1100) ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 36, (O_M11, 2000, 1, 1, 0, 0, 2, 123456789), "(2000/01/01 00:00:02.123456789 -1100) ../source3/smbd/oplock.c:1340(init_oplocks)"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZzc, RP_RB),
        DTFSS_YmdHMSfzc, 0, 40, CGN_YEAR, CGN_TZ,
        &[
            (1, 37, (O_M1130, 2000, 1, 1, 0, 0, 3, 123456789), r"{2000/01/01 00:00:03.123456789 -11:30} ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 31, (O_M1130, 2000, 1, 1, 0, 0, 3, 123000000), r"{2000/01/01 00:00:03.123 -11:30} ../source3/smbd/oplock.c:1340(init_oplocks)"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZzp, RP_RB),
        DTFSS_YmdHMSfzp, 0, 40, CGN_YEAR, CGN_TZ,
        &[
            (1, 34, (O_M11, 2000, 1, 1, 0, 0, 4, 123456789), "(2000/01/01 00:00:04.123456789 -11) ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 34, (O_M11, 2000, 1, 1, 0, 0, 4, 123456789), "(2000/01/01 00:00:04.123456789 -11)"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANK, CGP_TZZ, RP_RB),
        DTFSS_YmdHMSfZ, 0, 40, CGN_YEAR, CGN_TZ,
        &[
            (1, 35, (O_VLAT, 2000, 1, 1, 0, 0, 5, 123456789), "(2000/01/01 00:00:05.123456789 VLAT) ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 34, (O_WAT, 2000, 1, 1, 0, 0, 5, 123456789), "<2000/01/01 00:00:05.123456789 WAT> ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 34, (O_PST, 2000, 1, 1, 0, 0, 5, 123456789), "<2000/01/01 00:00:05.123456789 PST> ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 34, (O_PST, 2000, 1, 1, 0, 0, 5, 123456789), "<2000/01/01 00:00:05.123456789 pst> ../source3/smbd/oplock.c:1340(init_oplocks)"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, r"[,\.\| \t]", RP_BLANKSq, r"[[:word:]]{1,20}", RP_RB),
        DTFSS_YmdHMSf, 0, 40, CGN_YEAR, CGN_FRACTIONAL,
        &[
            (1, 27, (O_L, 2020, 3, 5, 12, 17, 59, 631000000), "[2020/03/05 12:17:59.631000, FOO] ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 27, (O_L, 2020, 3, 5, 12, 17, 59, 631000000), "[2020/03/05 12:17:59.631000, bubba] ../source3/smbd/oplock.c:1340(init_oplocks)"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/synology-DS6/opentftp.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     [22-Feb-17 21:24:20] Section [ALLOWED-CLIENTS] Invalid entry 192.168.0.0-192.168.0.255 in ini file, ignored
    //
    // The `"17"` is the shortened year and `"22"` the day of the month, see
    // source code
    //      https://sourceforge.net/projects/tftp-server/files/tftp%20server%20single%20port/opentftpspV1.66.tar.gz/download
    // file path
    //      opentftpspV1.66.tar.gz:opentftpspV1.66.tar:opentftp/opentftpd.cpp
    // function
    //     void logMess(request *req, MYBYTE logLevel)
    // line
    //     strftime(extbuff, sizeof(extbuff), "%d-%b-%y %X", ttm);
    //
    DTPD!(
        concatcp!("^", RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEARy, D_DHq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_RB),
        DTFSS_ybdHMS, 0, 40, CGN_DAY, CGN_SECOND,
        &[
            (1, 19, (O_L, 2017, 2, 22, 21, 24, 20, 0), "[22-Feb-17 21:24:20] Section [ALLOWED-CLIENTS] Invalid entry 192.168.0.0-192.168.0.255 in ini file, ignored")
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    // syslog messages using datetime format ISO 8601 / RFC 3339
    //
    //      <14>2023-01-01T15:00:36-08:00 (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received ⸨<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received⸩
    //      <29>2023-01-01T14:21:13-08:00 (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up ⸨<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up⸩
    //
    // Helpful reference https://ijmacd.github.io/rfc3339-iso8601/
    //
    // Highest value syslog priority is 191.
    // From https://github.com/rsyslog/rsyslog/blob/v8.2212.0/runtime/rsyslog.h#L197
    //
    //      #define LOG_MAXPRI 191  /* highest supported valid PRI value --> RFC3164, RFC5424 */
    //
    // an example with fractional seconds
    //
    //               1         2         3         4         5
    //     012345678901234567890123456789012345678901234567890
    //     <31>2023-01-06T14:35:00.506282-08:00 (host) (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826
    //     <128> 2023-01-06T14:35:00.506282871 -08:00[host]
    //
    // syslog format with fractional seconds
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Te, CGP_MINUTE, D_Te, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZzc, RP_NODIGIT),
        DTFSS_YmdHMSfzc, 0, 50, CGN_YEAR, CGN_TZ,
        &[
            (4, 36, (O_M8, 2023, 1, 6, 14, 35, 0, 506282000), "<31>2023-01-06T14:35:00.506282-08:00 (host) (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
            (6, 42, (O_M8, 2023, 1, 6, 14, 35, 0, 506282871), "<128> 2023-01-06T14:35:00.506282871 -08:00[host] (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Te, CGP_MINUTE, D_Te, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        DTFSS_YmdHMSfz, 0, 50, CGN_YEAR, CGN_TZ,
        &[
            (4, 35, (O_P8, 2023, 1, 6, 14, 35, 0, 506282000), "<31>2023-01-06T14:35:00.506282+0800 (host) (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
            (6, 41, (O_P8, 2023, 1, 6, 14, 35, 0, 506282871), "<128> 2023-01-06T14:35:00.506282871 +0800[host] (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Te, CGP_MINUTE, D_Te, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZzp, RP_NODIGIT),
        DTFSS_YmdHMSfzp, 0, 50, CGN_YEAR, CGN_TZ,
        &[
            (4, 33, (O_P8, 2023, 1, 6, 14, 35, 0, 506282000), "<31>2023-01-06T14:35:00.506282+08 (host) (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
            (6, 39, (O_P8, 2023, 1, 6, 14, 35, 0, 506282871), "<128> 2023-01-06T14:35:00.506282871 +08[host] (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Te, CGP_MINUTE, D_Te, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZZ, RP_NODIGIT),
        DTFSS_YmdHMSfZ, 0, 50, CGN_YEAR, CGN_TZ,
        &[
            (4, 34, (O_PST, 2023, 1, 6, 14, 35, 0, 506282000), "<31>2023-01-06T14:35:00.506282 PST (host) (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
            (6, 40, (O_WITA, 2023, 1, 6, 14, 35, 0, 506282871), "<128> 2023-01-06T14:35:00.506282871 WITA[host] (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Te, CGP_MINUTE, D_Te, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_NOALNUM),
        DTFSS_YmdHMSf, 0, 50, CGN_YEAR, CGN_FRACTIONAL,
        &[
            (4, 30, (O_L, 2023, 1, 6, 14, 35, 0, 506282000), "<31>2023-01-06T14:35:00.506282 (host) (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
            (6, 35, (O_L, 2023, 1, 6, 14, 35, 0, 506282871), "<128> 2023-01-06T14:35:00.506282871[host] (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
        ],
        line!(),
    ),
    // syslog format without fractional seconds
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzc),
        DTFSS_YmdHMSzc, 0, 46, CGN_YEAR, CGN_TZ,
        &[
            (4, 29, (O_M8, 2023, 2, 1, 15, 0, 36, 0), "<14>2023-02-01T15:00:36-08:00 (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received ⸨<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received⸩"),
            (4, 29, (O_0, 2023, 2, 1, 15, 0, 36, 0), "<14>2023-02-01T15:00:36+00:00 (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received ⸨<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received⸩"),
            (4, 30, (O_M8, 2023, 2, 1, 14, 21, 13, 0), "<29>2023-02-01T14:21:13 -08:00 (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up ⸨<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up⸩"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzp),
        DTFSS_YmdHMSzp, 0, 50, CGN_YEAR, CGN_TZ,
        &[
            (4, 26, (O_M8, 2023, 2, 1, 15, 0, 36, 0), "<14>2023-02-01T15:00:36-08 (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received ⸨<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received⸩"),
            (4, 27, (O_M8, 2023, 2, 1, 14, 21, 13, 0), "<29>2023-02-01T14:21:13 -08 (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up ⸨<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up⸩"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZZ, RP_NOALNUM),
        DTFSS_YmdHMSZ, 0, 50, CGN_YEAR, CGN_TZ,
        &[
            (4, 27, (O_PST, 2023, 2, 1, 15, 0, 36, 0), "<14>2023-02-01T15:00:36 PST (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received ⸨<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received⸩"),
            (4, 28, (O_CIST, 2023, 2, 1, 14, 21, 13, 0), "<29>2023-02-01T14:21:13 CIST (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up ⸨<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up⸩"),
            (4, 24, (O_Z, 2023, 2, 1, 14, 21, 13, 0), "<29>2023-02-01T14:21:13Z (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up ⸨<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up⸩"),
        ],
        line!(),
    ),
    //DTPD!(
    //    concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZZ, RP_NOALNUM),
    //    DTFSS_YmdHMSfZ, 0, 50, CGN_YEAR, CGN_TZ,
    //    &[
    //        (4, 31, DUMMY_ARGS, "<14>2023-01-01T15:00:36.172 PST (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received ⸨<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received⸩"),
    //        (4, 32, DUMMY_ARGS, "<29>2023-01-01T14:21:13.376 CIST (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up ⸨<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up⸩"),
    //        (4, 28, DUMMY_ARGS, "<29>2023-01-01T14:21:13.376Z (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up ⸨<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up⸩"),
    //    ],
    //    line!(),
    //),
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NOALNUM),
        DTFSS_YmdHMS, 0, 45, CGN_YEAR, CGN_SECOND,
        &[
            (4, 23, (O_L, 2023, 2, 1, 15, 0, 36, 0), "<14>2023-02-01T15:00:36 (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received ⸨<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received⸩"),
            (4, 23, (O_L, 2023, 2, 1, 14, 21, 13, 0), "<29>2023-02-01T14:21:13(HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up ⸨<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up⸩"),
            (5, 24, (O_L, 2023, 2, 1, 14, 21, 13, 0), "<191>2023-02-01T14:21:13 (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up ⸨<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up⸩"),
        ],
        line!(),
    ),
    //
    // TODO: [2023/01/09] not all TIMESTAMP patterns or other valid message formats are covered.
    //       see RFC 5424 https://www.rfc-editor.org/rfc/rfc5424.html#page-12
    //                    https://www.rfc-editor.org/rfc/rfc5424.html#section-6.5
    //                    https://www.rfc-editor.org/rfc/rfc5424.html#appendix-A.4
    //
    // Syslog also uses datetime stamp format RFC 3164
    // The contrived rsyslog messages are derived from the example in RFC 3164 (BSD Syslog Protocol)
    // https://www.rfc-editor.org/rfc/rfc3164.html#section-5.4
    //
    //      <14>Jan  1 15:00:36 2023 HOST dropbear[23732]: Exit (root): Disconnect received
    //      <29>Jan  1 14:21:13 2023 HOST netifd: Network device 'eth0' link is up
    //
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKSq, CGP_TZzc, RP_NODIGIT),
        DTFSS_BdHMSYzc, 0, 40, CGN_MONTH, CGN_TZ,
        &[
            (4, 31, (O_M2, 2023, 1, 1, 15, 0, 36, 0), "<14>Jan  1 15:00:36 2023 -02:00 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (4, 31, (O_M2, 2023, 1, 1, 15, 0, 36, 0), "<14>Jan 01 15:00:36 2023 -02:00 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 30, (O_M2, 2023, 1, 31, 14, 21, 13, 0), "<1>Jan 31 14:21:13 2023 -02:00 HOST netifd: Network device 'eth0' link is up"),
            (5, 32, (O_M2, 2023, 1, 31, 14, 21, 13, 0), "<135>Jan 31 14:21:13 2023 -02:00 HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKSq, CGP_TZzp, RP_NODIGIT),
        DTFSS_BdHMSYzp, 0, 40, CGN_MONTH, CGN_TZ,
        &[
            (4, 28, (O_M2, 2023, 1, 1, 15, 0, 36, 0), "<14>Jan  1 15:00:36 2023 -02 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (4, 28, (O_M2, 2023, 1, 1, 15, 0, 36, 0), "<14>Jan 01 15:00:36 2023 -02 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 27, (O_M2, 2023, 1, 31, 14, 21, 13, 0), "<1>Jan 31 14:21:13 2023 -02 HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKSq, CGP_TZZ, RP_NOALNUM),
        DTFSS_BdHMSYZ, 0, 40, CGN_MONTH, CGN_TZ,
        &[
            (4, 29, (O_WGST, 2023, 1, 1, 15, 0, 36, 0), "<14>Jan  1 15:00:36 2023 WGST HOST dropbear[23732]: Exit (root): Disconnect received"),
            (4, 29, (O_WGST, 2023, 1, 1, 15, 0, 36, 0), "<14>Jan 01 15:00:36 2023 WGST HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 28, (O_WGST, 2023, 1, 31, 14, 21, 13, 0), "<1>Jan 31 14:21:13 2023 WGST HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_NODIGIT),
        DTFSS_BdHMSY, 0, 35, CGN_MONTH, CGN_YEAR,
        &[
            (4, 24, (O_L, 2023, 1, 1, 15, 0, 36, 0), "<14>Jan  1 15:00:36 2023 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 23, (O_L, 2023, 1, 31, 14, 21, 13, 0), "<1>Jan 31 14:21:13 2023 HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    //
    //      <14>Jan  1 15:00:36 WIST 2023 HOST dropbear[23732]: Exit (root): Disconnect received
    //
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_TZzc, RP_BLANKS, CGP_YEAR, RP_NODIGIT),
        DTFSS_BdHMSYzc, 0, 40, CGN_MONTH, CGN_YEAR,
        &[
            (4, 31, (O_M2, 2023, 1, 1, 15, 0, 36, 0), "<14>Jan  1 15:00:36 -02:00 2023 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 30, (O_M2, 2023, 1, 31, 14, 21, 13, 0), "<1>Jan 31 14:21:13 -02:00 2023 HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_TZzp, RP_BLANKS, CGP_YEAR, RP_NODIGIT),
        DTFSS_BdHMSYzp, 0, 40, CGN_MONTH, CGN_YEAR,
        &[
            (4, 28, (O_M2, 2023, 1, 1, 15, 0, 36, 0), "<14>Jan  1 15:00:36 -02 2023 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 27, (O_M2, 2023, 1, 31, 14, 21, 13, 0), "<1>Jan 31 14:21:13 -02 2023 HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_TZZ, RP_BLANKS, CGP_YEAR, RP_NODIGIT),
        DTFSS_BdHMSYZ, 0, 40, CGN_MONTH, CGN_YEAR,
        &[
            (4, 29, (O_WGST, 2023, 1, 1, 15, 0, 36, 0), "<14>Jan  1 15:00:36 WGST 2023 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 28, (O_WGST, 2023, 1, 31, 14, 21, 13, 0), "<1>Jan 31 14:21:13 WGST 2023 HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    //
    //      <14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received
    //      <29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up
    //      <7>Mar 23 09:35:30 localhost DPKG [9332:<console>.<module>:2] debug info
    //
    DTPD!(
        concatcp!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKSq),
        DTFSS_BdHMS, 0, 30, CGN_MONTH, CGN_SECOND,
        &[
            (4, 19, (O_L, YD, 1, 1, 15, 0, 36, 0), "<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 18, (O_L, YD, 1, 31, 14, 21, 13, 0), "<1>Jan 31 14:21:13 HOST netifd: Network device 'eth0' link is up"),
            (3, 18, (O_L, YD, 3, 23, 9, 35, 30, 0), "<7>Mar 23 09:35:30 localhost DPKG [9332:<console>.<module>:2] debug info"),
            (5, 20, (O_L, YD, 3, 23, 9, 35, 30, 0), "<144>Mar 23 09:35:30 localhost DPKG [9332:<console>.<module>:2] debug info"),
            // example scraped from RFC 3164 (https://www.rfc-editor.org/rfc/rfc3164.html#section-5.4)
            (4, 19, (O_L, YD, 10, 11, 22, 14, 15, 0), "<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
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
        concatcp!("^((log|Log|LOG) (started|Started|STARTED|ended|Ended|ENDED))[:]?", RP_BLANKSq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, "(T|[[:blank:]]+)", CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND),
        DTFSS_YmdHMS, 0, 40, CGN_YEAR, CGN_SECOND,
        &[
            (13, 33, (O_L, 2022, 7, 14, 6, 48, 58, 0), "Log started: 2022-07-14  06:48:58\n(Reading database …"),
            (13, 32, (O_L, 2022, 7, 14, 6, 48, 58, 0), "Log started: 2022-07-14 06:48:58 Reading database ..."),
            (13, 32, (O_L, 2022, 7, 14, 6, 48, 58, 0), "Log started: 2022-07-14T06:48:58"),
            (11, 31, (O_L, 2022, 7, 14, 6, 49, 59, 0), "Log ended: 2022-07-14  06:49:59"),
            (11, 30, (O_L, 2022, 7, 14, 6, 49, 59, 0), "Log ended:\t2022-07-14\t06:49:59"),
        ],
        line!(),
    ),
    //
    // ---------------------------------------------------------------------------------------------
    // from file `logs/Windows10Pro/debug/mrt.log`
    // example with offset:
    //
    //               1         2         3         4          5         6         7         8         9
    //     01234567890123456789012345678901234567890012345678901234567890123456789012345678901234567890
    //     ---------------------------------------------------------------------------------------
    //     Microsoft Windows Malicious Software Removal Tool v5.83, (build 5.83.13532.1)
    //     Started On Thu Sep 10 10:08:35 2020
    //     ...
    //     Results Summary:
    //     ----------------
    //     No infection found.
    //     Successfully Submitted Heartbeat Report
    //     Microsoft Windows Malicious Software Removal Tool Finished On Tue Nov 10 18:54:47 2020
    //
    DTPD!(
        concatcp!("(Started On|Started on|started on|STARTED|Started|started|Finished On|Finished on|finished on|FINISHED|Finished|finished)[:]?", RP_BLANK, CGP_DAYa, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        DTFSS_YbdHMS, 0, 140, CGN_DAYa, CGN_YEAR,
        &[
            (11, 35, (O_L, 2020, 9, 10, 10, 8, 35, 0), "Started On Thu Sep 10 10:08:35 2020"),
            (11, 34, (O_L, 2020, 9, 1, 10, 8, 35, 0), "Started On Thu Sep 1 10:08:35 2020"),
            (11, 35, (O_L, 2020, 9, 1, 10, 8, 35, 0), "Started On Thu Sep  1 10:08:35 2020"),
            (62, 86, (O_L, 2020, 11, 10, 18, 54, 47, 0), "Microsoft Windows Malicious Software Removal Tool Finished On Tue Nov 10 18:54:47 2020"),
        ],
        line!(),
    ),
    //
    // ---------------------------------------------------------------------------------------------
    // from file `logs/Windows10Pro/comsetup.log`
    // example with offset:
    //
    //      COM+[12:24:34]: Setup started - [DATE:05,27,2020 TIME: 12:24 pm]
    //      COM+[12:24:34]: ********************************************************************************
    //      COM+[12:24:34]: Start CComMig::Discover
    //      COM+[12:24:34]: Return XML stream: <migXml xmlns=""><rules context="system"><include><objectSet></objectSet></include></rules></migXml>
    //      COM+[12:24:34]: End CComMig::Discover - Return 0x00000000
    //      COM+[12:24:38]: ********************************************************************************
    //      COM+[12:24:38]: Setup (COMMIG) finished - [DATE:05,27,2020 TIME: 12:24 pm]
    //
    // ---------------------------------------------------------------------------------------------
    // from file `logs/Windows10Pro/System32/wbem/WMIMigration.log`
    // example with offset:
    //
    //      (08/10/2019-01:46:44.0042) Filtering object "\\HOST\ROOT\CIMV2\mdm\dmmap:MDM_Policy_Config01_Location02" during apply
    //      (05/27/2020-12:25:43.0877) Total number of objects successfully migrated :2346, failed objects :16
    //
    DTPD!(
        concatcp!("^", RP_LB, CGP_MONTHm, D_D, CGP_DAYde, D_D, CGP_YEAR, D_DHdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_RB),
        DTFSS_YmdHMSf, 0, 40, CGN_MONTH, CGN_FRACTIONAL,
        &[
            (1, 25, (O_L, 2019, 8, 10, 1, 46, 44, 4200000), r#"(08/10/2019-01:46:44.0042) Filtering object "\\HOST\ROOT\CIMV2\mdm\dmmap:MDM_Policy_Config01_Location02" during apply"#),
            (1, 25, (O_L, 2020, 5, 27, 12, 25, 43, 87700000), "(05/27/2020-12:25:43.0877) Total number of objects successfully migrated :2346, failed objects :16"),
        ],
        line!(),
    ),
    //
    // ---------------------------------------------------------------------------------------------
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
    // from NTP statistics logging files `loopstats`, `clockstats`, `peerstats`, etc...
    //
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     59955 725.605 -0.002167105 47.876 0.012528010 1.558579 9
    //
    // according to https://docs.ntpsec.org/latest/monopt.html
    //
    //     59955 is the modified Julian Day number
    //     725.605 time of day (s) past midnight UTC
    //
    // other examples from http://www.ntp.org/ntpfaq/NTP-s-trouble.htm
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
        concatcp!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZZ, RP_NOALPHA),
        DTFSS_BdHMSYZ, 0, 40, CGN_MONTH, CGN_TZ,
        &[
            (0, 30, (O_PWT, 2000, 9, 3, 8, 10, 29, 0), "September 03 08:10:29 2000 PWT hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 24, (O_PWT, 2000, 1, 3, 1, 2, 4, 0), "Jan 03 01:02:04 2000 PWT 😀"),
            (0, 24, (O_PWT, 2000, 1, 3, 1, 2, 4, 0), "Jan  3 01:02:04 2000 PWT 😀"),
            (0, 23, (O_PWT, 2000, 1, 3, 1, 2, 4, 0), "Jan 3 01:02:04 2000 PWT 😀"),
            (0, 24, (O_PWT, 2000, 2, 29, 1, 2, 4, 0), "Feb 29 01:02:04 2000 pwt 😀"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZzc, RP_NOALPHA),
        DTFSS_BdHMSYzc, 0, 40, CGN_MONTH, CGN_TZ,
        &[
            (0, 33, (O_P3, 2000, 9, 3, 8, 10, 29, 0), "September 03 08:10:29 2000 +03:00 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 27, (O_P3, 2000, 1, 3, 1, 2, 4, 0), "Jan 03 01:02:04 2000 +03:00 😀"),
            (0, 26, (O_M730, 2000, 1, 3, 1, 2, 4, 0), "Jan 3 01:02:04 2000 -07:30 😀"),
            (0, 27, (O_M730, 2000, 1, 3, 1, 2, 4, 0), "Jan  3 01:02:04 2000 -07:30 😀"),
            (0, 27, (O_P3, 2000, 2, 29, 1, 2, 4, 0), "Feb 29 01:02:04 2000 +03:00 😀"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZz, RP_NOALPHA),
        DTFSS_BdHMSYz, 0, 40, CGN_MONTH, CGN_TZ,
        &[
            (0, 32, (O_P3, 2000, 9, 3, 8, 10, 29, 0), "September 03 08:10:29 2000 +0300 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 26, (O_P3, 2000, 1, 3, 1, 2, 4, 0), "Jan 03 01:02:04 2000 +0300 😀"),
            (0, 26, (O_P3, 2000, 1, 3, 1, 2, 4, 0), "Jan  3 01:02:04 2000 +0300 😀"),
            (0, 25, (O_M730, 2000, 1, 2, 3, 4, 5, 0), "Jan 2 03:04:05 2000 -0730 😀"),
            (0, 26, (O_P3, 2000, 2, 29, 1, 2, 4, 0), "Feb 29 01:02:04 2000 +0300 😀"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZzp, RP_NOALPHA),
        DTFSS_BdHMSYzp, 0, 40, CGN_MONTH, CGN_TZ,
        &[
            (0, 30, (O_P3, 2000, 9, 3, 8, 10, 29, 0), "September 03 08:10:29 2000 +03 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 24, (O_P3, 2000, 1, 5, 1, 2, 4, 0), "Jan 05 01:02:04 2000 +03 😀"),
            (0, 24, (O_P3, 2000, 2, 29, 1, 2, 4, 0), "Feb 29 01:02:04 2000 +03 😀"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_NOALNUM),
        DTFSS_BdHMSY, 0, 35, CGN_MONTH, CGN_YEAR,
        &[
            (0, 26, (O_L, 2000, 9, 3, 8, 10, 29, 0), "September 03 08:10:29 2000:hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 20, (O_L, 2000, 1, 2, 3, 4, 5, 0), "Jan 02 03:04:05 2000|😀"),
            (0, 19, (O_L, 2000, 1, 5, 1, 2, 4, 0), "Jan 5 01:02:04 2000 😀"),
            (0, 20, (O_L, 2000, 2, 29, 1, 2, 4, 0), "Feb 29 01:02:04 2000 😀"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_TZZ, RP_NOALPHA),
        DTFSS_BdHMSZ, 0, 35, CGN_MONTH, CGN_TZ,
        &[
            (0, 25, (O_PWT, YD, 9, 3, 8, 10, 29, 0), "September 03 08:10:29 PWT hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 19, (O_PWT, YD, 1, 2, 3, 4, 5, 0), "Jan 02 03:04:05 pwt 😀"),
            (0, 18, (O_PWT, YD, 1, 2, 3, 4, 5, 0), "Jan 2 03:04:05 PWT 😀"),
            (0, 19, (O_PWT, YD, 2, 29, 1, 2, 4, 0), "Feb 29 01:02:04 PWT 😀"),
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
        concatcp!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKSq),
        DTFSS_BdHMS, 0, 22, CGN_MONTH, CGN_SECOND,
        &[
            (0, 21, (O_L, YD, 9, 3, 8, 10, 29, 0), "September 03 08:10:29 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 15, (O_L, YD, 1, 2, 3, 4, 5, 0), "Jan 02 03:04:05 1900 😀"),
            (0, 14, (O_L, YD, 1, 2, 3, 4, 5, 0), "Jan 2 03:04:05 1900 😀"),
            (0, 19, (O_L, YD, 1, 3, 13, 47, 7, 0), "January 03 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119"),
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
    DTPD!(
        concatcp!("^", CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZz, RP_NODIGIT),
        DTFSS_BdHMSYz, 0, 45, CGN_DAYa, CGN_TZ,
        &[
            (0, 30, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "mon Jun 28 2022 01:51:12 +1230"),
            (0, 31, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "tue. Jun 28 2022 01:51:12 +1230"),
            (0, 31, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "Mon. Jun 28 2022 01:51:12 +1230"),
            (0, 30, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "Mon Jun 28 2022 01:51:12 +1230"),
            (0, 30, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), "Wed Jun 02 2022 01:51:12 +1230"),
            (0, 29, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), "thu Jun 2 2022 01:51:12 +1230"),
            (0, 33, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "Friday Jun 28 2022 01:51:12 +1230"),
            (0, 34, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "monday, Jun 28 2022 01:51:12 +1230"),
            (0, 30, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "Tue Jun 28 2022 01:51:12 +1230 FOOBAR"),
            (0, 31, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "Tue, Jun 28 2022 01:51:12 +1230"),
            (0, 35, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "Tuesday. Jun 28 2022 01:51:12 +1230 FOOBAR"),
            (0, 35, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "TUESDAY. Jun 28 2022 01:51:12 +1230 FOOBAR"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZzc, RP_NODIGIT),
        DTFSS_BdHMSYzc, 0, 45, CGN_DAYa, CGN_TZ,
        &[
            (0, 31, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "WED Jun 28 2022 01:51:12 +01:30"),
            (0, 32, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "Wed, Jun 28 2022 01:51:12 +01:30"),
            (0, 32, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "wed. Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (0, 32, (O_P130, 2022, 6, 2, 1, 51, 12, 0), "wed. Jun 02 2022 01:51:12 +01:30 FOOBAR"),
            (0, 31, (O_P130, 2022, 6, 2, 1, 51, 12, 0), "wed. Jun 2 2022 01:51:12 +01:30 FOOBAR"),
            (0, 37, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "Wednesday Jun 28 2022 01:51:12 +01:30"),
            (0, 38, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "Wednesday, Jun 28 2022 01:51:12 +01:30"),
            (0, 31, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "thu Jun 28 2022 01:51:12 +01:30 FOOBAR"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZzp, RP_NODIGIT),
        DTFSS_BdHMSYzp, 0, 45, CGN_DAYa, CGN_TZ,
        &[
            (0, 34, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "THURSDAY, Jun 28 2022 01:51:12 +01"),
            (0, 34, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "thursday, Jun 28 2022 01:51:12 +01"),
            (0, 29, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "fri. Jun 28 2022 01:51:12 +01 FOOBAR"),
            (0, 29, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "fri, Jun 28 2022 01:51:12 +01"),
            (0, 28, (O_P1, 2022, 6, 2, 1, 51, 12, 0), "fri, Jun 2 2022 01:51:12 +01"),
            (0, 31, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "FRIDAY Jun 28 2022 01:51:12 +01 FOOBAR"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZZ, RP_NOALPHA),
        DTFSS_BdHMSYZ, 0, 45, CGN_DAYa, CGN_TZ,
        &[
            (0, 34, (O_WIT, 2022, 6, 28, 1, 51, 12, 0), "Saturday, Jun 28 2022 01:51:12 WIT"),
            (0, 30, (O_WITA, 2022, 6, 28, 1, 51, 12, 0), "SAT, Jun 28 2022 01:51:12 WITA:FOOBAR"),
            (0, 29, (O_WST, 2022, 6, 28, 1, 51, 12, 0), "SAT. Jun 28 2022 01:51:12 WST FOOBAR"),
            (0, 29, (O_YAKT, 2022, 6, 28, 1, 51, 12, 0), "sun Jun 28 2022 01:51:12 YAKT"),
            (0, 28, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), "sun Jun 2 2022 01:51:12 YAKT"),
            (0, 32, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "sunday Jun 28 2022 01:51:12 YEKT FOOBAR"),
            (0, 32, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "sunday Jun 28 2022 01:51:12 yekt FOOBAR"),
            (0, 32, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "SUNDAY Jun 28 2022 01:51:12 YEKT FOOBAR"),
            (0, 33, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "SUNDAY, Jun 28 2022 01:51:12 YEKT FOOBAR"),
        ],
        line!(),
    ),
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
        concatcp!("^(start|Start|START|end|End|END)[- ]?(date|Date|DATE)", D_T, RP_BLANKSq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NODIGIT),
        DTFSS_YmdHMS, 0, 35, CGN_YEAR, CGN_SECOND,
        &[
            (12, 32, (O_L, 2022, 7, 18, 19, 35, 1, 0), "Start-Date: 2022-07-18  19:35:01\nCommandline: apt-get install -y gnupg2\nInstall: gnupg2:amd64 (2.2.27-3ubuntu2.1)\n"),
            (10, 30, (O_L, 2022, 7, 18, 19, 35, 2, 0), "End-Date: 2022-07-18  19:35:02\n"),
            (10, 30, (O_L, 2022, 7, 18, 19, 35, 3, 0), "End-Date: 2022-07-18  19:35:03"),
            (9, 29, (O_L, 2022, 7, 18, 19, 35, 4, 0), "End-Date:2022-07-18  19:35:04"),
            (9, 28, (O_L, 2022, 7, 18, 19, 35, 5, 0), "End Date:2022-07-18 19:35:05\n"),
            (9, 28, (O_L, 2022, 7, 18, 19, 35, 6, 0), "End-Date 2022-07-18 19:35:06\n"),
            (10, 29, (O_L, 2022, 7, 18, 19, 35, 7, 0), "END-DATE  2022-07-18 19:35:07 Foobar"),
            (10, 29, (O_L, 2022, 7, 18, 19, 35, 7, 0), "END DATE		2022-07-18 19:35:07	Foobar"),
            (9, 28, (O_L, 2022, 7, 18, 19, 35, 7, 0), "END-DATE	2022-07-18 19:35:07 Foobar"),
            (10, 29, (O_L, 2022, 7, 18, 19, 35, 7, 0), "END-DATE:	2022-07-18 19:35:07 Foobar"),
            (9, 28, (O_L, 2022, 7, 18, 19, 35, 8, 0), "end-date 2022-07-18T19:35:08 Foobar"),
            (14, 33, (O_L, 2022, 7, 18, 19, 35, 9, 0), "START-DATE:   2022-07-18 19:35:09\nCommandline: apt-get install -y gnupg2\n"),
            (11, 30, (O_L, 2022, 7, 18, 19, 35, 9, 0), "STARTDATE:	2022/07/18 19:35:09\nCommandline: apt-get install -y gnupg2\n"),
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
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/Ubuntu18/cups/error_log`
    // example with offset:
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     E [09/Aug/2019:00:09:01 -0700] Unable to open listen socket for address [v1.::1]:631 - Cannot assign requested address.
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
    // from file `./logs/synology-DS6/synoreport.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2017-05-14 04-00-07: -------------------- report start
    DTPD!(
        concatcp!(CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Te, CGP_MINUTE, D_Te, CGP_SECOND, ":", RP_BLANKe),
        DTFSS_YmdHMS, 0, 30, CGN_YEAR, CGN_SECOND,
        &[
            (0, 19, (O_L, 2017, 5, 14, 4, 0, 7, 0), "2017-05-14 04-00-07:"),
            (0, 19, (O_L, 2017, 5, 14, 4, 0, 8, 0), "2017-05-14 04-00-08: "),
            (0, 19, (O_L, 2017, 5, 14, 4, 0, 9, 0), "2017-05-14 04-00-09: -------------------- report start"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // matches of datetimes within a "guard" symbols.
    //
    // from file `./logs/Debian11/apache2/access.log`
    // example with offset:
    //
    //               1         2         3         4         5
    //     012345678901234567890123456789012345678901234567890
    //     192.168.0.172 - - [11/Oct/2022:00:10:26 +0000] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1"
    //
    // from file `./logs/Debian9/nginx/access.log`
    // example with offset:
    //
    //               1         2         3         4         5
    //     012345678901234567890123456789012345678901234567890
    //     192.168.0.8 - - [06/Mar/2020:06:30:43 -0800] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72"
    //
    DTPD!(
        concatcp!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz, RP_RB),
        DTFSS_bdHMSYz, 0, 300, CGN_DAY, CGN_TZ,
        &[
            (19, 45, (O_P1, 2022, 10, 11, 0, 10, 26, 0), r#"192.168.0.172 - - [11/Oct/2022:00:10:26 +0100] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 45, (O_P1, 2022, 10, 11, 0, 10, 26, 0), r#"192.168.0.172 - - {11/oct/2022 00:10:26 +0100} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 40, (O_P1, 2022, 10, 11, 0, 10, 26, 0), r#"192.168.0.172	<11-oct-2022 00:10:26+0100>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 43, (O_M8, 2020, 3, 7, 6, 30, 43, 0), r#"192.168.0.8 - - [07/Mar/2020:06:30:43 -0800] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZZ, RP_RB),
        DTFSS_bdHMSYZ, 0, 300, CGN_DAY, CGN_TZ,
        &[
            (19, 44, (O_CHUT, 2022, 10, 11, 0, 10, 26, 0), r#"192.168.0.172 - - [11/Oct/2022:00:10:26 CHUT] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 44, (O_CHUT, 2022, 10, 11, 0, 10, 26, 0), r#"192.168.0.172 - - {11/oct/2022 00:10:26 CHUT} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 40, (O_CHUT, 2022, 10, 11, 0, 10, 26, 0), r#"192.168.0.172	<11-oct-2022 00:10:26 CHUT>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 42, (O_CHUT, 2020, 3, 7, 6, 30, 43, 0), r#"192.168.0.8 - - [07/Mar/2020:06:30:43 CHUT] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzc, RP_RB),
        DTFSS_bdHMSYzc, 0, 300, CGN_DAY, CGN_TZ,
        &[
            (19, 46, (O_P1, 2022, 10, 11, 0, 10, 26, 0), r#"192.168.0.172 - - [11/Oct/2022:00:10:26 +01:00] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 46, (O_P1, 2022, 10, 11, 0, 10, 26, 0), r#"192.168.0.172 - - {11/oct/2022 00:10:26 +01:00} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 41, (O_P1, 2022, 10, 11, 0, 10, 26, 0), r#"192.168.0.172	<11-oct-2022 00:10:26+01:00>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 44, (O_M8, 2020, 3, 7, 6, 30, 43, 0), r#"192.168.0.8 - - [07/Mar/2020:06:30:43 -08:00] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzp, RP_RB),
        DTFSS_bdHMSYzp, 0, 300, CGN_DAY, CGN_TZ,
        &[
            (19, 43, (O_P1, 2022, 10, 11, 0, 10, 26, 0), r#"192.168.0.172 - - [11/Oct/2022:00:10:26 +01] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 43, (O_P1, 2022, 10, 11, 0, 10, 26, 0), r#"192.168.0.172 - - {11/oct/2022 00:10:26 +01} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 38, (O_P1, 2022, 10, 11, 0, 10, 26, 0), r#"192.168.0.172	<11-oct-2022 00:10:26+01>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 41, (O_M8, 2020, 3, 7, 6, 30, 43, 0), r#"192.168.0.8 - - [07/Mar/2020:06:30:43 -08] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    // prior patterns with fractionals 1-9
    DTPD!(
        concatcp!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZz, RP_RB),
        DTFSS_bdHMSYfz, 0, 300, CGN_DAY, CGN_TZ,
        &[
            (19, 49, (O_P1, 2022, 10, 11, 0, 10, 26, 123000000), r#"192.168.0.172 - - [11/Oct/2022:00:10:26.123 +0100] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 48, (O_P1, 2022, 10, 11, 0, 10, 26, 110000000), r#"192.168.0.172 - - {11/oct/2022 00:10:26.11 +0100} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 42, (O_P1, 2022, 10, 11, 0, 10, 26, 100000000), r#"192.168.0.172	<11-oct-2022 00:10:26.1+0100>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 53, (O_M8, 2020, 3, 7, 6, 30, 43, 123456789), r#"192.168.0.8 - - [07/Mar/2020:06:30:43.123456789 -0800] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZzc, RP_RB),
        DTFSS_bdHMSYfzc, 0, 300, CGN_DAY, CGN_TZ,
        &[
            (19, 50, (O_P1, 2022, 10, 11, 0, 10, 26, 123000000), r#"192.168.0.172 - - [11/Oct/2022:00:10:26.123 +01:00] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 49, (O_P1, 2022, 10, 11, 0, 10, 26, 110000000), r#"192.168.0.172 - - {11/oct/2022 00:10:26.11 +01:00} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 43, (O_P1, 2022, 10, 11, 0, 10, 26, 100000000), r#"192.168.0.172	<11-oct-2022 00:10:26.1+01:00>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 54, (O_M8, 2020, 3, 7, 6, 30, 43, 123456789), r#"192.168.0.8 - - [07/Mar/2020:06:30:43.123456789 -08:00] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZzp, RP_RB),
        DTFSS_bdHMSYfzp, 0, 300, CGN_DAY, CGN_TZ,
        &[
            (19, 47, (O_P1, 2022, 10, 11, 0, 10, 26, 123000000), r#"192.168.0.172 - - [11/Oct/2022:00:10:26.123 +01] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 46, (O_P1, 2022, 10, 11, 0, 10, 26, 110000000), r#"192.168.0.172 - - {11/oct/2022 00:10:26.11 +01} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 40, (O_P1, 2022, 10, 11, 0, 10, 26, 100000000), r#"192.168.0.172	<11-oct-2022 00:10:26.1+01>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 51, (O_M8, 2020, 3, 7, 6, 30, 43, 123456789), r#"192.168.0.8 - - [07/Mar/2020:06:30:43.123456789 -08] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKSq, RP_RB),
        DTFSS_bdHMSYf, 0, 300, CGN_DAY, CGN_FRACTIONAL,
        &[
            (19, 43, (O_L, 2022, 10, 11, 0, 10, 26, 123000000), r#"192.168.0.172 - - [11/Oct/2022:00:10:26.123] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 42, (O_L, 2022, 10, 11, 0, 10, 26, 110000000), r#"192.168.0.172 - - {11/oct/2022 00:10:26.11 } "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 37, (O_L, 2022, 10, 11, 0, 10, 26, 100000000), r#"192.168.0.172	<11-oct-2022 00:10:26.1        >	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 47, (O_L, 2020, 3, 7, 6, 30, 43, 123456789), r#"192.168.0.8 - - [07/Mar/2020:06:30:43.123456789] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    //
    // from file `./logs/Debian11/apache/error.log`
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     [Mon Oct 10 23:56:29.204202 2022] [mpm_event:notice] [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations
    //
    DTPD!(
        concatcp!(RP_LB, CGP_DAYa, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_BLANKS, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANK, CGP_YEAR, RP_RB),
        DTFSS_bdHMSYf, 0, 300, CGN_DAYa, CGN_YEAR,
        &[
            (1, 32, (O_L, 2022, 10, 10, 23, 56, 29, 204202000), "[Mon Oct 10 23:56:29.204202 2022] [mpm_event:notice] [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations"),
            (20, 51, (O_L, 2022, 10, 10, 23, 56, 29, 204202000), "[mpm_event:notice]	<Mon Oct 10	23:56:29.204202 2022> [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations"),
            (20, 48, (O_L, 2022, 10, 30, 23, 56, 29, 204000000), "[mpm_event:notice]	<sun Oct 30	23:56:29.204 2022> [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations"),
            (20, 54, (O_L, 2022, 10, 5, 23, 56, 29, 204948193), "[mpm_event:notice]	<WED oct 05	23:56:29.204948193 2022> [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(RP_LB, CGP_DAYa, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_BLANKS, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_RB),
        DTFSS_bdHMSY, 0, 300, CGN_DAYa, CGN_YEAR,
        &[
            (1, 25, (O_L, 2022, 10, 10, 23, 56, 29, 0), "[Mon Oct 10 23:56:29 2022] [mpm_event:notice] [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations"),
            (20, 44, (O_L, 2022, 10, 10, 23, 56, 29, 0), "[mpm_event:notice]	(Mon Oct 10	23:56:29 2022) [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // tomcat catalina stdout log format, from file `./logs/programs/tomcat/catalina.out`
    // example with offset:
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     08-Feb-2023 12:12:09.827 INFO [main] org.apache.coyote.AbstractProtocol.init Initializing ProtocolHandler ["http-nio2-0.0.0.0-8080"]
    //
    DTPD!(
        concatcp!("^", CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL3, RP_NOALNUMpm),
        DTFSS_bdHMSYf, 0, 30, CGN_DAY, CGN_FRACTIONAL,
        &[
            (0, 24, (O_L, 2023, 2, 8, 12, 13, 9, 827000000), r#"08-Feb-2023 12:13:09.827 INFO [main] org.apache.coyote.AbstractProtocol.init Initializing ProtocolHandler ["http-nio2-0.0.0.0-8080"]"#),
            (0, 24, (O_L, 2023, 2, 8, 12, 13, 20, 63000000), "08-Feb-2023 12:13:20.063 INFO [localhost-startStop-1] org.apache.jasper.servlet.TldScanner.scanJars At least one JAR was scanned for TLDs yet contained no TLDs. Enable debug logging for this logger for a complete list of JARs that were scanned but no TLDs were found in them. Skipping unneeded JARs during scanning can improve startup time and JSP compilation time.\nSLF4J: Class path contains multiple SLF4J bindings."),
        ],
        line!(),
    ),
    // TODO: add new `DTPD!` copied from prior `DTPD!` with varying timezones and more lenient fractional
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
    //
    // from file `./logs/synology-DS6/synolog/synobackup.log`
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     info	2017/02/21 21:50:48	SYSTEM:	[Local][Backup Task Backup1] Backup task started.
    //     err	2017/02/23 02:55:58	SYSTEM:	[Local][Backup Task Backup1] Exception occurred while backing up data. (Capacity at destination is insufficient.) [Path: /share4/usbshare/Backup1.hbk]
    //     err	2017/02/23 02:56:03	SYSTEM:	[Local][Backup Task Backup1] Failed to backup data.
    //     info	2017/02/24 02:30:04	SYSTEM:	[Local][Backup Task Backup1] Backup task started.
    //     warning	2017/02/24 03:43:57	SYSTEM:	[Local][Backup Task Backup1] Backup folder [Vol/DS] failed. (The backup source shared folder is encrypted and not mounted. Please mount the backup source shared folder and try again.)
    //
    // other examples:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     INFO: Thu Feb 20 00:59:59 2020 info
    //     ERROR: Thu Feb 20 00:59:59 2020 error
    //     DEBUG: Thu Feb 20 00:59:59 2020 debug
    //     VERBOSE: Thu Feb 20 00:59:59 2020 verbose
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     INFO: Sat Jan 01 2000 08:45:55 info
    //     WARN: Sat Jan 01 2000 08:45:55 warn
    //     ERROR: Sat Jan 01 2000 08:45:55 error
    //     DEBUG: Sun Jan 02 2000 21:00:00 debug
    //     VERBOSE: Sat Jan 01 2000 08:45:55 verbose
    //
    DTPD!(
        concatcp!("^", RP_LEVELS, "[:]?", RP_BLANKSq, CGP_DAYa, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzc, RP_NODIGIT),
        DTFSS_YbdHMSzc, 0, 60, CGN_DAYa, CGN_TZ,
        &[
            (7, 38, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "TRACE:	Sat Jan 01 2000 08:45:55 +09:00 TRACE: ⇥ ×1‼"),
            (8, 39, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "trace0:	sat jan 01 2000 08:45:55 +09:00 trace0: ⇥ ×1‼"),
            (9, 40, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "TRACE1:		Sat Jan 01 2000 08:45:55 +09:00 TRACE1: ⇥ ×2‼"),
            (8, 39, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "TRACE2:	Sat Jan 01 2000 08:45:55 +09:00 TRACE2: ⇥ ×1‼"),
            (7, 38, (O_P9, 2000, 1, 2, 21, 0, 0, 0), "DEBUG: Sun Jan 02 2000 21:00:00 +09:00 DEBUG:‼"),
            (7, 38, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "debug: sat jan 01 2000 08:45:55 +09:00 debug:‼"),
            (7, 38, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "DEBUG0 Sat Jan 01 2000 08:45:55 +09:00 debug0‼"),
            (8, 39, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "DEBUG9: Sat Jan 01 2000 08:45:55 +09:00 debug9:‼"),
            (6, 37, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "INFO: Sat Jan 01 2000 08:45:55 -09:00 info:‼"),
            (7, 38, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "INFO2: Sat Jan 01 2000 08:45:55 -09:00 info2:‼"),
            (9, 40, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "warning: Sat Jan 01 2000 08:45:55 -09:00 warning:‼"),
            (8, 39, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "warning sat jan 01 2000 08:45:55 -09:00 warning‼"),
            (8, 39, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "warning	Sat Jan 01 2000 08:45:55 -09:00 warning ⇥ ×1‼"),
            (8, 39, (O_M9, 2000, 1, 3, 23, 30, 59, 0), "WARNING	MON JAN 03 2000 23:30:59 -09:00 warning ⇥ ×1‼"),
            (9, 40, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "warning		Sat Jan 01 2000 08:45:55 -09:00 warning ⇥ ×2‼"),
            (6, 37, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "WARN: SAT JAN 01 2000 08:45:55 -09:00 warn:‼"),
            (7, 38, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "ERROR: Sat Jan 01 2000 08:45:55 -09:00 error:‼"),
            (5, 36, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "ERR: Sat Jan 01 2000 08:45:55 -09:00 err:‼"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LEVELS, "[:]?", RP_BLANKSq, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        DTFSS_YbdHMSz, 0, 60, CGN_DAYa, CGN_TZ,
        &[
            (7, 37, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "TRACE:	Sat Jan 01 2000 08:45:55 +0900 TRACE: ⇥ ×1‼"),
            (8, 38, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "trace0:	sat jan 01 2000 08:45:55 +0900	trace0: ⇥ ×1‼"),
            (9, 39, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "TRACE1:		Sat Jan 01 2000 08:45:55 +0900		TRACE1: ⇥ ×2‼"),
            (8, 38, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "TRACE2:	Sat Jan 01 2000 08:45:55 +0900	TRACE2: ⇥ ×1‼"),
            (7, 37, (O_P9, 2000, 1, 2, 21, 0, 0, 0), "DEBUG: Sun Jan 02 2000 21:00:00 +0900 DEBUG:‼"),
            (7, 37, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "debug: sat jan 01 2000 08:45:55 +0900 debug:‼"),
            (7, 37, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "DEBUG0 Sat Jan 01 2000 08:45:55 +0900 debug0‼"),
            (8, 38, (O_P9, 2000, 1, 1, 8, 45, 55, 0), "DEBUG9: Sat Jan 01 2000 08:45:55 +0900 debug9:‼"),
            (6, 36, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "INFO: Sat Jan 01 2000 08:45:55 -0900 info:‼"),
            (7, 37, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "INFO2: Sat Jan 01 2000 08:45:55 -0900 info2:‼"),
            (9, 39, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "warning: Sat Jan 01 2000 08:45:55 -0900 warning:‼"),
            (8, 38, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "warning sat jan 01 2000 08:45:55 -0900 warning‼"),
            (8, 38, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "warning	Sat Jan 01 2000 08:45:55 -0900	warning ⇥ ×1‼"),
            (8, 38, (O_M9, 2000, 1, 3, 23, 30, 59, 0), "WARNING	MON JAN 03 2000 23:30:59 -0900	warning ⇥ ×1‼"),
            (9, 39, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "warning		Sat Jan 01 2000 08:45:55 -0900		warning ⇥ ×2‼"),
            (6, 36, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "WARN: SAT JAN 01 2000 08:45:55 -0900 warn:‼"),
            (7, 37, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "ERROR: Sat Jan 01 2000 08:45:55 -0900 error:‼"),
            (5, 35, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "ERR: Sat Jan 01 2000 08:45:55 -0900 err:‼"),
            (5, 39, (O_M9, 2000, 1, 1, 8, 45, 55, 0), "ERR: Sat January 01 2000 08:45:55 -0900 err:‼"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LEVELS, "[:]?", RP_BLANKSq, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzp, RP_NODIGIT),
        DTFSS_YbdHMSzp, 0, 60, CGN_DAYa, CGN_TZ,
        &[
            (7, 35, (O_P9, 2000, 1, 31, 8, 45, 55, 0), "TRACE:	Sat Jan 31 2000 08:45:55 +09 TRACE: ⇥ ×1‼"),
            (8, 36, (O_P9, 2000, 1, 31, 8, 45, 55, 0), "trace0:	sat jan 31 2000 08:45:55 +09	trace0: ⇥ ×1‼"),
            (9, 37, (O_P9, 2000, 1, 31, 8, 45, 55, 0), "TRACE1:		Sat Jan 31 2000 08:45:55 +09		TRACE1: ⇥ ×2‼"),
            (8, 36, (O_P9, 2000, 1, 31, 8, 45, 55, 0), "TRACE2:	Sat Jan 31 2000 08:45:55 +09	TRACE2: ⇥ ×1‼"),
            (7, 35, (O_P9, 2000, 1, 2, 21, 0, 0, 0), "DEBUG: Sun Jan 02 2000 21:00:00 +09 DEBUG:‼"),
            (7, 35, (O_P9, 2000, 1, 31, 8, 45, 55, 0), "debug: sat jan 31 2000 08:45:55 +09 debug:‼"),
            (7, 35, (O_P9, 2000, 1, 31, 8, 45, 55, 0), "DEBUG0 Sat Jan 31 2000 08:45:55 +09 debug0‼"),
            (8, 36, (O_P9, 2000, 1, 31, 8, 45, 55, 0), "DEBUG9: Sat Jan 31 2000 08:45:55 +09 debug9:‼"),
            (6, 34, (O_M9, 2000, 1, 31, 8, 45, 55, 0), "INFO: Sat Jan 31 2000 08:45:55 -09 info:‼"),
            (7, 35, (O_M9, 2000, 1, 31, 8, 45, 55, 0), "INFO2: Sat Jan 31 2000 08:45:55 -09 info2:‼"),
            (9, 37, (O_M9, 2000, 1, 31, 8, 45, 55, 0), "warning: Sat Jan 31 2000 08:45:55 -09 warning:‼"),
            (8, 36, (O_M9, 2000, 1, 31, 8, 45, 55, 0), "warning sat jan 31 2000 08:45:55 -09 warning‼"),
            (8, 36, (O_M9, 2000, 1, 31, 8, 45, 55, 0), "warning	Sat Jan 31 2000 08:45:55 -09	warning ⇥ ×1‼"),
            (8, 36, (O_M9, 2000, 1, 3, 23, 30, 59, 0), "WARNING	MON JAN 03 2000 23:30:59 -09	warning ⇥ ×1‼"),
            (9, 37, (O_M9, 2000, 1, 31, 8, 45, 55, 0), "warning		Sat Jan 31 2000 08:45:55 -09		warning ⇥ ×2‼"),
            (6, 34, (O_M9, 2000, 1, 31, 8, 45, 55, 0), "WARN: SAT JAN 31 2000 08:45:55 -09 warn:‼"),
            (7, 35, (O_M9, 2000, 1, 31, 8, 45, 55, 0), "ERROR: Sat Jan 31 2000 08:45:55 -09 error:‼"),
            (5, 33, (O_M9, 2000, 1, 31, 8, 45, 55, 0), "ERR: sat jan 31 2000 08:45:55 -09 err:‼"),
            (4, 32, (O_M9, 2000, 1, 31, 8, 45, 55, 0), "err SAT JAN 31 2000 08:45:55 -09 err:‼"),
            (4, 36, (O_M9, 2000, 1, 31, 8, 45, 55, 0), "err SAT JANUARY 31 2000 08:45:55 -09 err:‼"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LEVELS, "[:]?", RP_BLANKSq, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZZ, RP_NOALPHA),
        DTFSS_YbdHMSZ, 0, 65, CGN_DAYa, CGN_TZ,
        &[
            (7, 35, (O_PST, 2000, 1, 31, 8, 45, 55, 0), "TRACE:	Sat Jan 31 2000 08:45:55 PST"),
            (7, 35, (O_PST, 2000, 1, 31, 8, 45, 55, 0), "TRACE:	Sat Jan 31 2000 08:45:55 PST TRACE: ⇥ ×1‼"),
            (8, 36, (O_PET, 2000, 1, 31, 8, 45, 55, 0), "trace0:	sat jan 31 2000 08:45:55 pet	trace0: ⇥ ×1‼"),
            (9, 38, (O_PETT, 2000, 1, 31, 8, 45, 55, 0), "TRACE1:		Sat Jan 31 2000 08:45:55 PETT	TRACE1: ⇥ ×2‼"),
            (8, 37, (O_WITA, 2000, 1, 31, 8, 45, 55, 0), "TRACE2:	Sat Jan 31 2000 08:45:55 WITA	TRACE2: ⇥ ×1‼"),
            (7, 36, (O_WITA, 2000, 1, 2, 21, 45, 55, 0), "DEBUG: Sun Jan 02 2000 21:45:55 WITA DEBUG:‼"),
            (7, 36, (O_WITA, 2000, 1, 31, 8, 45, 55, 0), "debug: sat jan 31 2000 08:45:55 wita debug:‼"),
            (7, 36, (O_WITA, 2000, 1, 31, 8, 45, 55, 0), "DEBUG0 Sat Jan 31 2000 08:45:55 WITA debug0‼"),
            (8, 37, (O_WITA, 2000, 1, 31, 8, 45, 55, 0), "DEBUG9: Sat Jan 31 2000 08:45:55 WITA debug9:‼"),
            (6, 35, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "INFO: Sat Jan 31 2000 08:45:55 PONT info:‼"),
            (7, 36, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "INFO2: Sat Jan 31 2000 08:45:55 PONT info2:‼"),
            (9, 38, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "warning: Sat Jan 31 2000 08:45:55 pont warning:‼"),
            (8, 37, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "warning sat jan 31 2000 08:45:55 pont warning‼"),
            (8, 37, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "warning	Sat Jan 31 2000 08:45:55 pont	warning ⇥ ×1‼"),
            (8, 37, (O_PONT, 2000, 1, 3, 23, 30, 59, 0), "WARNING	MON JAN 03 2000 23:30:59 PONT		warning ⇥ ×1‼"),
            (9, 38, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "warning		Sat Jan 31 2000 08:45:55 pont	warning ⇥ ×2‼"),
            (6, 35, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "WARN: SAT JAN 31 2000 08:45:55 PONT:warn:‼"),
            (7, 36, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "ERROR: SAT jan 31 2000 08:45:55 PONT|error:‼"),
            (5, 34, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "ERR: sat Jan 31 2000 08:45:55 PONT err:‼"),
            (5, 34, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "ERR: Sat jan 31 2000 08:45:55 PONT err:‼"),
            (5, 38, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "ERR: Sat january 31 2000 08:45:55 PONT err:‼"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LEVELS, "[:]?", RP_BLANKSq, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NODIGIT),
        DTFSS_YbdHMS, 0, 60, CGN_DAYa, CGN_SECOND,
        &[
            (7, 31, (O_L, 2000, 1, 31, 8, 45, 55, 0), "TRACE:	Sat Jan 31 2000 08:45:55 TRACE: ⇥ ×1‼"),
            (8, 32, (O_L, 2000, 1, 31, 8, 45, 55, 0), "trace0:	sat jan 31 2000 08:45:55	trace0: ⇥ ×1‼"),
            (9, 33, (O_L, 2000, 1, 31, 8, 45, 55, 0), "TRACE1:		Sat Jan 31 2000 08:45:55	TRACE1: ⇥ ×2‼"),
            (8, 32, (O_L, 2000, 1, 31, 8, 45, 55, 0), "TRACE2:	Sat Jan 31 2000 08:45:55	TRACE2: ⇥ ×1‼"),
            (7, 31, (O_L, 2000, 1, 2, 21, 0, 0, 0), "DEBUG: Sun Jan 02 2000 21:00:00 DEBUG:‼"),
            (7, 31, (O_L, 2000, 1, 31, 8, 45, 55, 0), "debug: sat jan 31 2000 08:45:55 debug:‼"),
            (7, 31, (O_L, 2000, 1, 31, 8, 45, 55, 0), "DEBUG0 Sat Jan 31 2000 08:45:55 debug0‼"),
            (8, 32, (O_L, 2000, 1, 31, 8, 45, 55, 0), "DEBUG9: Sat Jan 31 2000 08:45:55 debug9:‼"),
            (6, 30, (O_L, 2000, 1, 31, 8, 45, 55, 0), "INFO: Sat Jan 31 2000 08:45:55 info:‼"),
            (7, 31, (O_L, 2000, 1, 31, 8, 45, 55, 0), "INFO2: Sat Jan 31 2000 08:45:55 info2:‼"),
            (9, 33, (O_L, 2000, 1, 31, 8, 45, 55, 0), "warning: Sat Jan 31 2000 08:45:55 -09:00 warning:‼"),
            (8, 32, (O_L, 2000, 1, 31, 8, 45, 55, 0), "warning sat jan 31 2000 08:45:55 warning‼"),
            (8, 32, (O_L, 2000, 1, 31, 8, 45, 55, 0), "warning	Sat Jan 31 2000 08:45:55	warning ⇥ ×1‼"),
            (8, 32, (O_L, 2000, 1, 3, 23, 30, 59, 0), "WARNING	MON JAN 03 2000 23:30:59	warning ⇥ ×1‼"),
            (8, 31, (O_L, 2000, 1, 3, 23, 30, 59, 0), "WARNING	MON JAN 3 2000 23:30:59	warning ⇥ ×1‼"),
            (9, 33, (O_L, 2000, 1, 31, 8, 45, 55, 0), "warning		Sat Jan 31 2000 08:45:55		warning ⇥ ×2‼"),
            (6, 30, (O_L, 2000, 1, 31, 8, 45, 55, 0), "WARN: SAT JAN 31 2000 08:45:55 warn:‼"),
            (7, 31, (O_L, 2000, 1, 31, 8, 45, 55, 0), "ERROR: Sat Jan 31 2000 08:45:55 error:‼"),
            (5, 29, (O_L, 2000, 1, 31, 8, 45, 55, 0), "ERR: Sat Jan 31 2000 08:45:55 err:‼"),
            (5, 29, (O_L, 2000, 1, 31, 8, 45, 55, 0), "ERR: Sat JAN 31 2000 08:45:55 err:‼"),
            (4, 32, (O_L, 2000, 1, 31, 8, 45, 55, 0), "ERR Sat JANUARY 31 2000 08:45:55 err:‼"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Debian9/apport.log.1`
    //
    //               1         2         3         4         5         6
    //     0123456789012345678901234567890123456789012345678901234567890
    //     ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 -0700: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 -07:00: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 9) Thu Feb 20 00:59:59 2020: executable: /usr/lib/firefox/firefox (command line "/usr/lib/firefox/firefox"
    //
    DTPD!(
        concatcp!("^", RP_LEVELS, "[:]?", RP_ANYp, RP_BLANK, CGP_DAYa, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZzc, RP_NODIGIT),
        DTFSS_YbdHMSzc, 0, 120, CGN_DAYa, CGN_TZ,
        &[
            (22, 53, (O_M7, 2020, 2, 27, 0, 33, 59, 0), "ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 -07:00: called for pid 8581, signal 24, core limit 0, dump mode 1"),
            (27, 58, (O_M330, 2022, 8, 13, 8, 48, 3, 0), r#"ERROR: apport (pid 529343) Sat Aug 13 08:48:03 2022 -03:30: executable: [s4] (command line "./target/release/s4 -s -wp /dev")"#),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LEVELS, "[:]?", RP_ANYp, RP_BLANK, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZz, RP_NODIGIT),
        DTFSS_YbdHMSz, 0, 120, CGN_DAYa, CGN_TZ,
        &[
            (22, 52, (O_M7, 2020, 2, 27, 0, 33, 59, 0), "ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 -0700: called for pid 8581, signal 24, core limit 0, dump mode 1"),
            (27, 57, (O_M330, 2022, 8, 13, 8, 48, 3, 0), r#"ERROR: apport (pid 529343) Sat Aug 13 08:48:03 2022 -0330: executable: [s4] (command line "./target/release/s4 -s -wp /dev")"#),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LEVELS, "[:]?", RP_ANYp, RP_BLANK, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZzp, RP_NODIGIT),
        DTFSS_YbdHMSzp, 0, 120, CGN_DAYa, CGN_TZ,
        &[
            (22, 50, (O_M7, 2020, 2, 27, 0, 33, 59, 0), "ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 -07: called for pid 8581, signal 24, core limit 0, dump mode 1"),
            (27, 55, (O_M3, 2022, 8, 13, 8, 48, 3, 0), r#"ERROR: apport (pid 529343) Sat Aug 13 08:48:03 2022 -03: executable: [s4] (command line "./target/release/s4 -s -wp /dev")"#),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LEVELS, "[:]?", RP_ANYp, RP_BLANK, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZZ, RP_NOALPHA),
        DTFSS_YbdHMSZ, 0, 120, CGN_DAYa, CGN_TZ,
        &[
            (22, 51, (O_ALMT, 2020, 2, 27, 0, 33, 59, 0), "ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 ALMT called for pid 8581, signal 24, core limit 0, dump mode 1"),
            (27, 56, (O_ALMT, 2022, 8, 13, 8, 48, 3, 0), r#"ERROR: apport (pid 529343) Sat Aug 13 08:48:03 2022 ALMT: executable: [s4] (command line "./target/release/s4 -s -wp /dev")"#),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LEVELS, "[:]?", RP_ANYp, RP_BLANK, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_NOALNUM),
        DTFSS_YbdHMS, 0, 120, CGN_DAYa, CGN_YEAR,
        &[
            (22, 46, (O_L, 2020, 2, 27, 0, 33, 59, 0), "ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 called for pid 8581, signal 24, core limit 0, dump mode 1"),
            (27, 51, (O_L, 2022, 8, 13, 8, 48, 3, 0), r#"ERROR: apport (pid 529343) Sat Aug 13 08:48:03 2022: executable: [s4] (command line "./target/release/s4 -s -wp /dev")"#),
            (25, 49, (O_L, 2020, 2, 20, 0, 59, 59, 0), r#"ERROR: apport (pid 9359) Thu Feb 20 00:59:59 2020: executable: /usr/lib/firefox/firefox (command line "/usr/lib/firefox/firefox"#),
            (27, 51, (O_L, 2023, 1, 8, 12, 53, 11, 0), "ERROR: apport (pid 150689) Sun Jan  8 12:53:11 2023: called for pid 150672, signal 6, core limit 0, dump mode 1\n"),
        ],
        line!(),
    ),
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
    // general matches from beginning of line
    //
    DTPD!(
        concatcp!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        DTFSS_YmdHMSfz, 0, 50, CGN_YEAR, CGN_TZ,
        &[(0, 29, (O_M11, 2000, 1, 2, 0, 0, 2, 123000000), "2000/01/02 00:00:02.123 -1100 a")],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZzc, RP_NODIGIT),
        DTFSS_YmdHMSfzc, 0, 50, CGN_YEAR, CGN_TZ,
        &[(0, 33, (O_M1130, 2000, 1, 3, 0, 0, 3, 123456000), "2000/01/03 00:00:03.123456 -11:30 ab")],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZzp, RP_NODIGIT),
        DTFSS_YmdHMSfzp, 0, 50, CGN_YEAR, CGN_TZ,
        &[(0, 33, (O_M11, 2000, 1, 4, 0, 0, 4, 123456789), "2000/01/04 00:00:04,123456789 -11 abc")],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANK, CGP_TZZ, RP_NOALPHA),
        DTFSS_YmdHMSfZ, 0, 50, CGN_YEAR, CGN_TZ,
        &[(0, 34, (O_PETT, 2000, 1, 5, 0, 0, 5, 123456789), "2000/01/05 00:00:05.123456789 PETT abcd")],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_NODIGIT),
        DTFSS_YmdHMSf, 0, 50, CGN_YEAR, CGN_FRACTIONAL,
        &[(0, 29, (O_L, 2020, 1, 6, 0, 0, 26, 123456789), "2020-01-06 00:00:26.123456789 abcdefg")],
        line!(),
    ),
    //
    DTPD!(
        concatcp!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        DTFSS_YmdHMSz, 0, 50, CGN_YEAR, CGN_TZ,
        &[(0, 25, (O_M11, 2000, 1, 7, 0, 0, 2, 0), "2000/01/07T00:00:02 -1100 abcdefgh")],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzc, RP_NODIGIT),
        DTFSS_YmdHMSzc, 0, 50, CGN_YEAR, CGN_TZ,
        &[
            (0, 26, (O_M1130, 2000, 1, 8, 0, 0, 3, 0), "2000-01-08-00:00:03 -11:30 abcdefghi"),
            // ISO 8601, time extended format
            (0, 25, (O_M1, 2020, 1, 2, 3, 4, 5, 0), "2020-01-02T03:04:05-01:00 The standard uses the Gregorian calendar, which 'serves as an international standard for civil use'.[18]"),
            // ISO 8601, time basic format
            (0, 23, (O_M1, 2020, 1, 2, 3, 4, 5, 0), "2020-01-02T030405-01:00 ISO 8601:2004 fixes a reference calendar date to the Gregorian calendar of 20 May 1875 as the date the Convention du Mètre (Metre Convention) was signed in Paris (the explicit reference date was removed in ISO 8601-1:2019)."),
            // ISO 8601, time extended format
            (0, 23, (O_M1, 2020, 1, 2, 3, 4, 5, 0), "20200102T03:04:05-01:00 However, ISO calendar dates before the convention are still compatible with the Gregorian calendar all the way back to the official introduction of the Gregorian calendar on 15 October 1582."),
            // ISO 8601, time extended format
            (0, 23, (O_M1, 2020, 1, 2, 3, 4, 5, 0), "20200102T03:04:05-01:00 Calendar date representations are in the form shown in the adjacent box. [YYYY] indicates a four-digit year, 0000 through 9999. [MM] indicates a two-digit month of the year, 01 through 12. [DD] indicates a two-digit day of that month, 01 through 31."),
            // ISO 8601 / RFC 3339, time basic format
            (0, 21, (O_Z, 2020, 1, 2, 3, 4, 5, 0), "20200102T030405-00:00 IETF RFC 3339[43] defines a profile of ISO 8601 for use in Internet protocols and standards."),
            // ISO 8601 / RFC 3339, time extended format
            (0, 23, (O_Z, 2020, 1, 2, 3, 4, 5, 0), "20200102T03:04:05-00:00 RFC 3339 deviates from ISO 8601 in allowing a zero time zone offset to be specified as '-00:00;', which ISO 8601 forbids."),
            // ISO 8601, time extended format using Unicode "minus sign".
            //
            // Uses non-ASCII pattern in capture data.
            //
            // The data passed to chrono `DateTime::parse_from_str` is modified;
            // the Unicode "minus sign" is replaced with ASCII "hyphen-minus".
            // However, the bytes that would be written to stdout remain
            // unchanged (if this data had followed the full program path and
            // been processed by the `printer::printers::PrinterSysline`).
            // Hence, the offsets for `begin`, `end`, must account for Unicode
            // char "minus sign" (which is larger than typical 1-byte ASCII).
            (0, 27, (O_M1, 2020, 1, 2, 3, 4, 5, 0), "2020-01-02T03:04:05−01:00 To represent a negative offset, ISO 8601 specifies using a minus sign, (−)."),

        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzp, RP_NODIGIT),
        DTFSS_YmdHMSzp, 0, 50, CGN_YEAR, CGN_TZ,
        &[(0, 23, (O_M11, 2000, 1, 9, 0, 0, 4, 0), "2000/01/09 00:00:04 -11 abcdefghij")],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZZ, RP_NOALPHA),
        DTFSS_YmdHMSZ, 0, 50, CGN_YEAR, CGN_TZ,
        &[
            (0, 24, (O_VLAT, 2000, 1, 10, 0, 0, 5, 0), "2000/01/10T00:00:05 VLAT abcdefghijk"),
            (0, 23, (O_PST, 2000, 1, 10, 0, 0, 5, 0), "2000/01/10T00:00:05 pst abcdefghijk"),
            (0, 24, (O_VLAT, 2000, 1, 10, 0, 0, 5, 0), "2000/01/10T00:00:05 VLAT "),
            (0, 23, (O_PST, 2000, 1, 10, 0, 0, 5, 0), "2000/01/10T00:00:05 pst "),
            (0, 24, (O_VLAT, 2000, 1, 10, 0, 0, 5, 0), "2000/01/10T00:00:05 VLAT"),
            (0, 23, (O_PST, 2000, 1, 10, 0, 0, 5, 0), "2000/01/10T00:00:05 pst"),
        ],
        line!(),
    ),
    //
    DTPD!(
        concatcp!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NODIGIT),
        DTFSS_YmdHMS, 0, 50, CGN_YEAR, CGN_SECOND,
        &[
            (0, 19, (O_L, 2020, 1, 11, 0, 0, 26, 0), "2020-01-11 00:00:26 abcdefghijkl"),
            (0, 19, (O_L, 2020, 1, 11, 0, 0, 26, 0), "2020-01-11 00:00:26 pstxxxxxxxxx"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/synology-DS6/upstart/umount-root-fs.log`, Issue #44
    // example with offset:
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     Mon Dec 5 21:01:12 PST 2016 try umount root [1] times
    //     Wed Feb 28 14:58:07 PST 2018 try umount root [1] times
    //
    // CGP_DAY version
    //
    DTPD!(
        concatcp!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZZ, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        DTFSS_BdHMSYZ, 0, 45, CGN_DAYa, CGN_YEAR,
        &[
            (0, 27, (O_PST, 2016, 12, 5, 21, 1, 12, 0), "Mon Dec 5 21:01:12 PST 2016 try umount root [1] times"),
            (0, 28, (O_PST, 2016, 12, 5, 21, 1, 12, 0), "MON DEC  5 21:01:12 PST 2016 try umount root [1] times"),
            (0, 28, (O_PST, 2016, 12, 5, 21, 1, 12, 0), "MON DEC 05 21:01:12 PST 2016 try umount root [1] times"),
            (0, 28, (O_PST, 2016, 12, 5, 21, 1, 12, 0), "mon dec  5 21:01:12 pst 2016 try umount root [1] times"),
            (0, 31, (O_PST, 2016, 12, 5, 21, 1, 12, 0), "MONDAY dec  5 21:01:12 pst 2016 try umount root [1] times"),
            (0, 31, (O_PST, 2016, 12, 5, 21, 1, 12, 0), "MONDAY DEC  5 21:01:12 PST 2016 try umount root [1] times"),
            (0, 27, (O_PDT, 2017, 5, 8, 8, 33, 0, 0), "mon May 8 08:33:00 PDT 2017 try umount root [1] times"),
            (0, 28, (O_PST, 2018, 2, 28, 14, 58, 7, 0), "Wed Feb 28 14:58:07 PST 2018 try umount root [1] times"),
            (0, 34, (O_PST, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY Feb 28 14:58:07 PST 2018 try umount root [1] times"),
            (0, 39, (O_PST, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY February 28 14:58:07 PST 2018 try umount root [1] times"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZz, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        DTFSS_BdHMSYz, 0, 40, CGN_DAYa, CGN_YEAR,
        &[
            (0, 29, (O_Z, 2016, 12, 5, 21, 1, 12, 0), "Mon Dec 5 21:01:12 -0000 2016 try umount root [1] times"),
            (0, 30, (O_Z, 2016, 12, 5, 21, 1, 12, 0), "MON DEC  5 21:01:12 +0000 2016 try umount root [1] times"),
            (0, 30, (O_Z, 2016, 12, 5, 21, 1, 12, 0), "MON DEC 05 21:01:12 +0000 2016 try umount root [1] times"),
            (0, 30, (O_M1130, 2016, 12, 5, 21, 1, 12, 0), "mon dec  5 21:01:12 -1130 2016 try umount root [1] times"),
            (0, 29, (O_P945, 2017, 5, 8, 8, 33, 0, 0), "mon May 8 08:33:00 +0945 2017 try umount root [1] times"),
            (0, 32, (O_P945, 2017, 5, 8, 8, 33, 0, 0), "monday may 8 08:33:00 +0945 2017 try umount root [1] times"),
            (0, 30, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), "Wed Feb 28 14:58:07 -1030 2018 try umount root [1] times"),
            (0, 36, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY Feb 28 14:58:07 -1030 2018 try umount root [1] times"),
            (0, 41, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY February 28 14:58:07 -1030 2018 try umount root [1] times"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZzc, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        DTFSS_BdHMSYzc, 0, 45, CGN_DAYa, CGN_YEAR,
        &[
            (0, 30, (O_Z, 2016, 12, 5, 21, 1, 12, 0), "Mon Dec 5 21:01:12 -00:00 2016 try umount root [1] times"),
            (0, 31, (O_Z, 2016, 12, 5, 21, 1, 12, 0), "MON DEC  5 21:01:12 +00:00 2016 try umount root [1] times"),
            (0, 31, (O_Z, 2016, 12, 5, 21, 1, 12, 0), "MON DEC 05 21:01:12 +00:00 2016 try umount root [1] times"),
            (0, 31, (O_M1130, 2016, 12, 5, 21, 1, 12, 0), "mon dec  5 21:01:12 -11:30 2016 try umount root [1] times"),
            (0, 30, (O_P945, 2017, 5, 8, 8, 33, 0, 0), "mon May 8 08:33:00 +09:45 2017 try umount root [1] times"),
            (0, 33, (O_P945, 2017, 5, 8, 8, 33, 0, 0), "monday may 8 08:33:00 +09:45 2017 try umount root [1] times"),
            (0, 31, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), "Wed Feb 28 14:58:07 -10:30 2018 try umount root [1] times"),
            (0, 37, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY Feb 28 14:58:07 -10:30 2018 try umount root [1] times"),
            (0, 42, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY February 28 14:58:07 -10:30 2018 try umount root [1] times"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZzp, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        DTFSS_BdHMSYzp, 0, 45, CGN_DAYa, CGN_YEAR,
        &[
            (0, 27, (O_Z, 2016, 12, 5, 21, 1, 12, 0), "Mon Dec 5 21:01:12 -00 2016 try umount root [1] times"),
            (0, 28, (O_Z, 2016, 12, 5, 21, 1, 12, 0), "MON DEC  5 21:01:12 +00 2016 try umount root [1] times"),
            (0, 28, (O_Z, 2016, 12, 5, 21, 1, 12, 0), "MON DEC 05 21:01:12 +00 2016 try umount root [1] times"),
            (0, 28, (O_M11, 2016, 12, 5, 21, 1, 12, 0), "mon dec  5 21:01:12 -11 2016 try umount root [1] times"),
            (0, 33, (O_M11, 2016, 12, 5, 21, 1, 12, 0), "mon december  5 21:01:12 -11 2016 try umount root [1] times"),
            (0, 27, (O_P9, 2017, 5, 8, 8, 33, 0, 0), "mon May 8 08:33:00 +09 2017 try umount root [1] times"),
            (0, 27, (O_P9, 2017, 5, 8, 8, 33, 0, 0), "mon MAY 8 08:33:00 +09 2017 try umount root [1] times"),
            (0, 28, (O_M10, 2018, 2, 28, 14, 58, 7, 0), "Wed Feb 28 14:58:07 -10 2018 try umount root [1] times"),
            (0, 34, (O_M10, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY Feb 28 14:58:07 -10 2018 try umount root [1] times"),
            (0, 39, (O_M10, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY February 28 14:58:07 -10 2018 try umount root [1] times"),
            (0, 39, (O_M10, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY FEBRUARY 28 14:58:07 -10 2018 try umount root [1] times"),
        ],
        line!(),
    ),
    // TODO: add prior without any TZ
    //
    // from file `./logs/programs/proftpd/xferlog`, Issue #42
    // example with offset:
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     Sat Oct 03 11:26:12 2020 0 192.168.0.8 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c
    //     Sat Oct 03 11:26:12 2020 0 192.168.0.8 2323 /var/log/proftpd/proftpd.log b _ o r root ftp 0 * c
    //
    DTPD!(
        concatcp!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_NOALNUMpm),
        DTFSS_BdHMSY, 0, 40, CGN_DAYa, CGN_YEAR,
        &[
            (0, 23, (O_L, 2016, 12, 5, 21, 1, 12, 0), "Mon Dec 5 21:01:12 2016 try umount root [1] times"),
            (0, 24, (O_L, 2016, 12, 5, 21, 1, 13, 0), "MON DEC  5 21:01:13 2016 try umount root [1] times"),
            (0, 24, (O_L, 2016, 12, 5, 21, 1, 14, 0), "mon dec  5 21:01:14 2016 try umount root [1] times"),
            (0, 29, (O_L, 2016, 12, 5, 21, 1, 12, 0), "mon december  5 21:01:12 2016 try umount root [1] times"),
            (0, 23, (O_L, 2017, 5, 8, 8, 33, 0, 0), "mon May 8 08:33:00 2017 try umount root [1] times"),
            (0, 23, (O_L, 2017, 5, 8, 8, 33, 0, 0), "mon MAY 8 08:33:00 2017 try umount root [1] times"),
            (0, 24, (O_L, 2018, 2, 28, 14, 58, 7, 0), "Wed Feb 28 14:58:07 2018 try umount root [1] times"),
            (0, 30, (O_L, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY Feb 28 14:58:07 2018 try umount root [1] times"),
            (0, 35, (O_L, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY February 28 14:58:07 2018 try umount root [1] times"),
            (0, 35, (O_L, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY FEBRUARY 28 14:58:07 2018 try umount root [1] times"),
            (0, 24, (O_L, 2020, 10, 3, 11, 26, 12, 0), "Sat Oct  3 11:26:12 2020 0 192.168.0.8 2323 /var/log/proftpd/proftpd.log b _ o r root ftp 0 * c")
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // pacman log format, example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     [2019-03-01 16:56] [PACMAN] synchronizing package lists
    //
    DTPD!(
        concatcp!(r"^\[", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYde, D_DHq, CGP_HOUR, D_Te, CGP_MINUTE, r"\]"),
        DTFSS_YmdHM, 0, 20, CGN_YEAR, CGN_MINUTE,
        &[
            (1, 17, (O_L, 2019, 3, 1, 16, 56, 0, 0), "[2019-03-01 16:56] [PACMAN] synchronizing package lists"),
            (1, 17, (O_L, 2018, 5, 31, 12, 19, 0, 0), "[2018-05-31 12:19] [PACMAN] Running 'pacman -Syu --root /tmp/newmsys/msys64'"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // Windows 10 ReportingEvents.log format, example with offset:
    //
    //               1         2         3         4         5         6         7
    //     01234567890123456789012345678901234567890123456789012345678901234567890
    //     {5F45546A-691D-4519-810C-9B159EA7A24F}  2022-10-12 09:26:44:980-0700    1       181 [AGENT_INSTALLING_STARTED]  101     {ADF3720E-8453-44C7-82EF-F9F5DA2D8551}  1       0 Update;ScanForUpdates    Success Content Install Installation Started: Windows has started installing the following update: 9WZDNCRFJ364-MICROSOFT.SKYPEAPP      te2D3dMIjE2PeNSM.86.5.1.0.0.1.0
    //
    // very similar to next DTPD!, but with different second-to-fractional divider ":"
    //
    DTPD!(
        concatcp!(RP_NODIGIT, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, ":", CGP_FRACTIONAL3, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        DTFSS_YmdHMSfz, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
                (40, 68, (O_M7, 2022, 10, 12, 9, 26, 44, 980000000), r"{5F45546A-691D-4519-810C-9B159EA7A24F}  2022-10-12 09:26:44:980-0700    1       181 [AGENT_INSTALLING_STARTED]  101      {ADF3720E-8453-44C7-82EF-F9F5DA2D8551}  1       0 Update;ScanForUpdates    Success Content Download        Download succeeded.     te2D3dMIjE2PeNSM.86.3.1.0.0.85.0")
        ],
        line!(),
    ),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // general matches anywhere in the first 1024 bytes of the line
    //
    // these are most likely to match datetimes in the log *message*, i.e. a substring that happens
    // to be a datetime but is not the formal log timestamp. In other words, most likely to cause
    // errant matches. One reason they are declared last and so attempted last.
    //
    DTPD!(
        concatcp!(RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZz, RP_RB),
        DTFSS_YmdHMSfz, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            (1, 30, (O_M11, 2000, 1, 2, 5, 1, 32, 123000000), "<2000/01/02 05:01:32.123 -1100> a"),
            (1, 30, (O_M11, 2000, 1, 2, 5, 1, 32, 123000000), "{2000/01/02 05:01:32.123 -1100} a"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZzc, RP_RB),
        DTFSS_YmdHMSfzc, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            (11, 43, (O_M1130, 2000, 1, 3, 5, 2, 33, 123456000), "[LOGGER]  {2000/01/03 05:02:33.123456-11:30} ab"),
            (1, 34, (O_M1130, 2000, 1, 3, 5, 2, 33, 123456000), "<2000-01-03T05:02:33.123456 -11:30> ab"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZzp, RP_RB),
        DTFSS_YmdHMSfzp, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            (11, 44, (O_M11, 2000, 1, 4, 0, 3, 34, 123456789), "[LOGGER]  [2000/01/04 00:03:34,123456789 -11]"),
            (11, 44, (O_M11, 2000, 1, 4, 0, 3, 34, 123456789), "[LOGGER]  [2000/01/04 00:03:34.123456789 -11] abc"),
            (11, 43, (O_M11, 2000, 1, 4, 0, 3, 34, 123456789), "[LOGGER]  [2000/01/04T00:03:34,123456789-11]abc"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANK, CGP_TZZ, RP_RB),
        DTFSS_YmdHMSfZ, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), "[LOGGER]\t\t<2000/01/05 00:04:35.123456789 VLAT>:"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), "[LOGGER]  <2000/01/05 00:04:35.123456789 VLAT>"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), "[LOGGER]  <2000/01/05 00:04:35.123456789 VLAT> abcd"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), "[LOGGER]  <2000/01/05 00:04:35.123456789 VLAT>abcd"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), "[LOGGER]  <2000/01/05 00:04:35.123456789 VLAT>abcd"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), "[LOGGER]  [2000/01/05 00:04:35.123456789 VLAT] abcd"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), "[LOGGER]  [2000/01/05-00:04:35.123456789 VLAT]"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), "[LOGGER]  (2000/01/05T00:04:35.123456789 vlat) abcd"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_RB),
        DTFSS_YmdHMSf, 0, 1024, CGN_YEAR, CGN_FRACTIONAL,
        &[
            (11, 40, (O_L, 2020, 1, 6, 0, 5, 26, 123456789), "[LOGGER]  (2020-01-06 00:05:26.123456789) abcdefg"),
            (21, 50, (O_L, 2020, 1, 6, 0, 5, 26, 123456789), "[FOOBAR] (PID 2005) (2020-01-06 00:05:26.123456789) foobar!"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        DTFSS_YmdHMSfz, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            (0, 29, (O_M11, 2000, 1, 2, 7, 8, 32, 123000000), "2000/01/02 07:08:32.123 -1100 a"),
            (0, 29, (O_M11, 2000, 1, 2, 7, 8, 32, 123000000), "2000-01-02T07:08:32.123 -1100 a"),
            (0, 25, (O_M11, 2000, 1, 2, 7, 8, 32, 123000000), "20000102:070832.123 -1100 a"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZzc, RP_NODIGIT),
        DTFSS_YmdHMSfzc, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            (0, 33, (O_M1130, 2000, 1, 3, 0, 2, 3, 123456000), "2000/01/03 00:02:03.123456 -11:30 ab"),
            (1, 34, (O_M1130, 2000, 1, 3, 0, 2, 3, 123456000), "|2000/01/03:00:02:03.123456 -11:30|ab"),
            (1, 34, (O_M1130, 2000, 1, 3, 0, 2, 3, 123456000), "<2000/01/03T00:02:03.123456 -11:30 abc"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZzp, RP_NODIGIT),
        DTFSS_YmdHMSfzp, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            (0, 33, (O_M11, 2000, 1, 4, 0, 23, 24, 123456789), "2000/01/04 00:23:24,123456789 -11"),
            (0, 33, (O_M11, 2000, 1, 4, 0, 23, 24, 123456789), "2000/01/04 00:23:24,123456789 -11 abc"),
            (0, 33, (O_M11, 2000, 1, 4, 0, 23, 24, 123456789), "2000/01/04 00:23:24,123456789 -11_abc"),
            (1, 34, (O_M11, 2000, 1, 4, 0, 23, 24, 123456789), "|2000/01/04-00:23:24,123456789 -11|abc"),
            (1, 34, (O_M11, 2000, 1, 4, 0, 23, 24, 123456789), "[2000/01/04T00:23:24,123456789 -11] abc"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANK, CGP_TZZ, RP_NOALPHA),
        DTFSS_YmdHMSfZ, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            (0, 34, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), "2000/01/05 00:34:35.123456789 VLAT:"),
            (0, 34, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), "2000/01/05 00:34:35.123456789 VLAT"),
            (0, 34, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), "2000/01/05 00:34:35.123456789 VLAT abcd"),
            (0, 34, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), "2000/01/05:00:34:35.123456789 VLAT:abcd"),
            (1, 35, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), "|2000/01/05 00:34:35.123456789 VLAT|abcd"),
            (1, 35, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), ":2000/01/05 00:34:35.123456789 VLAT: abcd"),
            (1, 35, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), "[2000/01/05T00:34:35.123456789 VLAT]"),
            (0, 34, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), "2000/01/05-00:34:35.123456789 vlat abcd"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_NODIGIT),
        DTFSS_YmdHMSf, 0, 1024, CGN_YEAR, CGN_FRACTIONAL,
        &[
            (0, 29, (O_L, 2020, 1, 6, 0, 5, 26, 123456789), "2020-01-06 00:05:26.123456789 abcdefg"),
            (20, 49, (O_L, 2020, 1, 6, 0, 5, 26, 123456789), "[FOOBAR] (PID 2005) 2020-01-06 00:05:26.123456789 foobar!"),
        ],
        line!(),
    ),
    //
    // Synology OS `fsck/root.log`
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     20200307_202530 /sbin/e2fsck -pvf
    //
    DTPD!(
        concatcp!(CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        DTFSS_YmdHMSz, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            (0, 25, (O_M11, 2000, 1, 7, 0, 6, 2, 0), "2000/01/07T00:06:02 -1100 abcdefgh"),
            (1, 26, (O_M11, 2000, 1, 7, 0, 6, 2, 0), "[2000/01/07T00:06:02 -1100]	abcdefgh"),
            (0, 21, (O_M11, 2020, 3, 7, 20, 25, 30, 0), "20200307_202530 -1100 /sbin/e2fsck -pvf"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzc, RP_NODIGIT),
        DTFSS_YmdHMSzc, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            (0, 26, (O_M1130, 2000, 1, 8, 0, 7, 3, 0), "2000-01-08-00:07:03 -11:30 aabcdefghi"),
            (0, 26, (O_M1130, 2000, 1, 8, 0, 7, 3, 0), "2000-01-08-00:07:03 -11:30	aabcdefghi"),
            (1, 27, (O_M1130, 2000, 1, 8, 0, 7, 3, 0), "[2000-01-08-00:07:03 -11:30] aabcdefghi"),
            (0, 22, (O_M11, 2020, 3, 7, 20, 25, 30, 0), "20200307_202530 -11:00 /sbin/e2fsck -pvf"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzp, RP_NODIGIT),
        DTFSS_YmdHMSzp, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            (0, 23, (O_M11, 2000, 1, 9, 0, 8, 4, 0), "2000/01/09 00:08:04 -11 abcdefghij"),
            (1, 24, (O_M11, 2000, 1, 9, 0, 8, 4, 0), "[2000/01/09 00:08:04 -11] abcdefghij"),
            (0, 19, (O_M11, 2020, 3, 7, 20, 25, 30, 0), "20200307_202530 -11 /sbin/e2fsck -pvf"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZZ, RP_NOALPHA),
        DTFSS_YmdHMSZ, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            (0, 24, (O_VLAT, 2000, 1, 10, 0, 9, 5, 0), "2000/01/10T00:09:05 VLAT abcdefghijk"),
            (0, 24, (O_VLAT, 2000, 1, 10, 0, 9, 5, 0), "2000/01/10T00:09:05 VLAT_abcdefghijk"),
            (1, 25, (O_VLAT, 2000, 1, 10, 0, 9, 5, 0), "[2000/01/10T00:09:05 VLAT] abcdefghijk"),
            (1, 25, (O_VLAT, 2000, 1, 10, 0, 9, 5, 0), "[2000/01/10T00:09:05 VLAT] abcdefghijk"),
            (1, 25, (O_VLAT, 2000, 1, 10, 0, 9, 5, 0), "<2000/01/10T00:09:05 VLAT> abcdefghijk"),
            (0, 20, (O_VLAT, 2020, 3, 7, 20, 25, 30, 0), "20200307_202530 VLAT /sbin/e2fsck -pvf"),
        ],
        line!(),
    ),
    //
    /*
    DTPD!(
        concatcp!(CGP_MONTH, D_D, CGP_MONTHm, D_D, CGP_DAYde, " @", BLANKq, CGP_HOURh, D_T, CGP_MINUTE, RP_BLANKq, ),
        DTFSS_YmdHMSZ, 0, 1024, CGN_YEAR, CGN_TZ,
        &[
            "09/12/2022 @ 7:05am"
        ],
        line!(),
    ),
    */
    //
    DTPD!(
        concatcp!(CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NODIGIT),
        DTFSS_YmdHMS, 0, 1024, CGN_YEAR, CGN_SECOND,
        &[
            (0, 19, (O_L, 2020, 1, 11, 0, 10, 26, 0), "2020-01-11 00:10:26 abcdefghijkl"),
            (0, 15, (O_L, 2020, 3, 7, 20, 25, 30, 0), "20200307_202530:/sbin/e2fsck -pvf"),
        ],
        line!(),
    ),
    //
    DTPD!(
        concatcp!(CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZz, RP_NODIGIT),
        DTFSS_BdHMSYz, 0, 1024, CGN_DAYa, CGN_TZ,
        &[
            (8, 42, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tuesday Jun 28 2022 01:51:12 +1230"),
            (8, 38, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue Jun 28 2022 01:51:12 +1230 FOOBAR"),
            (8, 39, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue, Jun 28 2022 01:51:12 +1230"),
            (8, 39, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), "RSYSLOG Tue, Jun  2 2022 01:51:12 +1230"),
            (8, 39, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), "RSYSLOG Tue, Jun 02 2022 01:51:12 +1230"),
            (8, 38, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), "RSYSLOG Tue, Jun 2 2022 01:51:12 +1230"),
            (8, 39, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue. Jun 28 2022 01:51:12 +1230 FOOBAR"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZzc, RP_NODIGIT),
        DTFSS_BdHMSYzc, 0, 1024, CGN_DAYa, CGN_TZ,
        &[
            (3, 35, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "<7>Tue, Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (4, 36, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "<33>Tue, Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (28, 60, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "[SOME OTHER FIELD] BLARG<33>Tue, Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (1, 33, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "*Tue, Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (3, 35, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "***Tue, Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (11, 43, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "[RSYSLOG]: Tue, Jun 28 2022 01:51:12 +01:30"),
            (8, 40, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "[INFO]: Tue. Jun 28 2022 01:51:12 +01:30:FOOBAR"),
            (7, 38, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "[INFO]:Tue Jun 28 2022 01:51:12 +01:30<33>FOOBAR"),
            (6, 37, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "[INFO]Tue Jun 28 2022 01:51:12 +01:30FOOBAR"),
            (7, 38, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "{INFO} Tue Jun 28 2022 01:51:12 +01:30 FOOBAR"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZzp, RP_NODIGIT),
        DTFSS_BdHMSYzp, 0, 1024, CGN_DAYa, CGN_TZ,
        &[
            (8, 41, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "[DEBUG] Tuesday, Jun 28 2022 01:51:12 +01"),
            (9, 38, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "[TRACE1] Tue. Jun 28 2022 01:51:12 +01 FOOBAR"),
            (9, 38, (O_P1, 2022, 6, 2, 1, 51, 12, 0), "[TRACE1] Tue. Jun  2 2022 01:51:12 +01 FOOBAR"),
            (9, 38, (O_P1, 2022, 6, 2, 1, 51, 12, 0), "[TRACE1] Tue. Jun 02 2022 01:51:12 +01 FOOBAR"),
            (9, 37, (O_P1, 2022, 6, 2, 1, 51, 12, 0), "[TRACE1] Tue. Jun 2 2022 01:51:12 +01 FOOBAR"),
            (9, 38, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "[TRACE2] Tue, Jun 28 2022 01:51:12 +01"),
            (9, 37, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "[TRACE1] Tue Jun 28 2022 01:51:12 +01 FOOBAR"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZZ, RP_NOALPHA),
        DTFSS_BdHMSYZ, 0, 1024, CGN_DAYa, CGN_TZ,
        &[
            (6, 39, (O_WIT, 2022, 6, 28, 1, 51, 12, 0), "LOGGR Tuesday, Jun 28 2022 01:51:12 WIT"),
            (6, 36, (O_WITA, 2022, 6, 28, 1, 51, 12, 0), "LOGGR Tue, Jun 28 2022 01:51:12 WITA:FOOBAR"),
            (6, 35, (O_WST, 2022, 6, 28, 1, 51, 12, 0), "LOGGR Tue. Jun 28 2022 01:51:12 WST FOOBAR"),
            (8, 37, (O_YAKT, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue Jun 28 2022 01:51:12 YAKT"),
            (8, 37, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), "RSYSLOG Tue Jun  2 2022 01:51:12 YAKT"),
            (8, 37, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), "RSYSLOG Tue Jun 02 2022 01:51:12 YAKT"),
            (8, 36, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), "RSYSLOG Tue Jun 2 2022 01:51:12 YAKT"),
            (8, 37, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue Jun 28 2022 01:51:12 YEKT FOOBAR"),
            (8, 37, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue Jun 28 2022 01:51:12 yekt foobar"),
            (8, 38, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue June 28 2022 01:51:12 yekt foobar"),
            (8, 38, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue JUNE 28 2022 01:51:12 yekt foobar"),
            (8, 38, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue june 28 2022 01:51:12 yekt foobar"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NOALNUM),
        DTFSS_BdHMSY, 0, 1024, CGN_DAYa, CGN_SECOND,
        &[
            (6, 35, (O_L, 2022, 6, 28, 1, 51, 12, 0), "LOGGR Tuesday, Jun 28 2022 01:51:12 "),
            (6, 35, (O_L, 2022, 6, 28, 1, 51, 12, 0), "LOGGR Tuesday, Jun 28 2022 01:51:12"),
            (6, 31, (O_L, 2022, 6, 28, 1, 51, 12, 0), "LOGGR Tue, Jun 28 2022 01:51:12 FOOBAR"),
            (7, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), "blarg: Tue. Jun 28 2022 01:51:12 WST"),
            (8, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue Jun 28 2022 01:51:12[abc"),
            (8, 32, (O_L, 2022, 6, 2, 1, 51, 12, 0), "RSYSLOG Tue Jun  2 2022 01:51:12[abc"),
            (8, 32, (O_L, 2022, 6, 2, 1, 51, 12, 0), "RSYSLOG Tue Jun 02 2022 01:51:12;abc"),
            (8, 31, (O_L, 2022, 6, 2, 1, 51, 12, 0), "RSYSLOG Tue Jun 2 2022 01:51:12 YAKT"),
            (8, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue Jun 28 2022 01:51:12 YEKT FOOBAR"),
            (8, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue Jun 28 2022 01:51:12 foobar"),
            (8, 33, (O_L, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue June 28 2022 01:51:12               foobar"),
            (8, 33, (O_L, 2022, 6, 28, 1, 51, 12, 0), "RSYSLOG Tue JUNE 28 2022 01:51:12[YEKT]"),
            (6, 31, (O_L, 2022, 6, 28, 1, 51, 12, 0), "LOGGR|Tue june 28 2022 01:51:12|YEKT"),
        ],
        line!(),
    ),
    //
    DTPD!(
        concatcp!(CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_YEAR, RP_BLANK12q, CGP_TZz, RP_NODIGIT),
        DTFSS_BdHMSYz, 0, 1024, CGN_DAYa, CGN_TZ,
        &[
            (27, 57, (O_P1230, 2023, 1, 12, 22, 26, 47, 0), "ERROR: apport (pid 486722) Thu Jan 12 22:26:47 2023 +1230: called for pid 486450, signal 6, core limit 0, dump mode 1"),
            (8, 42, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tuesday Jun 28 01:51:12 2022 +1230"),
            (8, 38, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue Jun 28 01:51:12 2022 +1230 FOOBAR"),
            (8, 39, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue, Jun 28 01:51:12 2022 +1230"),
            (8, 39, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), "MESSAGE Tue, Jun  2 01:51:12 2022 +1230"),
            (8, 39, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), "MESSAGE Tue, Jun 02 01:51:12 2022 +1230"),
            (8, 38, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), "MESSAGE Tue, Jun 2 01:51:12 2022 +1230"),
            (8, 39, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue. Jun 28 01:51:12 2022 +1230 FOOBAR"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_YEAR, RP_BLANK12q, CGP_TZzc, RP_NODIGIT),
        DTFSS_BdHMSYzc, 0, 1024, CGN_DAYa, CGN_TZ,
        &[
            (27, 58, (O_P1230, 2023, 1, 12, 22, 26, 47, 0), "ERROR: apport (pid 486722) Thu Jan 12 22:26:47 2023 +12:30: called for pid 486450, signal 6, core limit 0, dump mode 1"),
            (3, 35, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "<7>Tue, Jun 28 01:51:12 2022 +01:30 FOOBAR"),
            (4, 36, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "<33>Tue, Jun 28 01:51:12 2022 +01:30 FOOBAR"),
            (28, 60, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "[SOME OTHER FIELD] BLARG<33>Tue, Jun 28 01:51:12 2022 +01:30 FOOBAR"),
            (1, 33, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "*Tue, Jun 28 01:51:12 2022 +01:30 FOOBAR"),
            (3, 35, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "***Tue, Jun 28 01:51:12 2022 +01:30 FOOBAR"),
            (11, 43, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "[MESSAGE]: Tue, Jun 28 01:51:12 2022 +01:30"),
            (8, 40, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "[INFO]: Tue. Jun 28 01:51:12 2022 +01:30:FOOBAR"),
            (7, 38, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "[INFO]:Tue Jun 28 01:51:12 2022 +01:30<33>FOOBAR"),
            (6, 37, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "[INFO]Tue Jun 28 01:51:12 2022 +01:30FOOBAR"),
            (7, 38, (O_P130, 2022, 6, 28, 1, 51, 12, 0), "{INFO} Tue Jun 28 01:51:12 2022 +01:30 FOOBAR"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_YEAR, RP_BLANK12q, CGP_TZzp, RP_NODIGIT),
        DTFSS_BdHMSYzp, 0, 1024, CGN_DAYa, CGN_TZ,
        &[
            (27, 55, (O_P12, 2023, 1, 12, 22, 26, 47, 0), "ERROR: apport (pid 486722) Thu Jan 12 22:26:47 2023 +12: called for pid 486450, signal 6, core limit 0, dump mode 1"),
            (8, 41, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "[DEBUG] Tuesday, Jun 28 01:51:12 2022 +01"),
            (9, 38, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "[TRACE1] Tue. Jun 28 01:51:12 2022 +01 FOOBAR"),
            (9, 38, (O_P1, 2022, 6, 2, 1, 51, 12, 0), "[TRACE1] Tue. Jun  2 01:51:12 2022 +01 FOOBAR"),
            (9, 38, (O_P1, 2022, 6, 2, 1, 51, 12, 0), "[TRACE1] Tue. Jun 02 01:51:12 2022 +01 FOOBAR"),
            (9, 37, (O_P1, 2022, 6, 2, 1, 51, 12, 0), "[TRACE1] Tue. Jun 2 01:51:12 2022 +01 FOOBAR"),
            (9, 38, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "[TRACE2] Tue, Jun 28 01:51:12 2022 +01"),
            (9, 37, (O_P1, 2022, 6, 28, 1, 51, 12, 0), "[TRACE1] Tue Jun 28 01:51:12 2022 +01 FOOBAR"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_YEAR, RP_BLANK12q, CGP_TZZ, RP_NOALPHA),
        DTFSS_BdHMSYZ, 0, 1024, CGN_DAYa, CGN_TZ,
        &[
            (27, 56, (O_WITA, 2023, 1, 12, 22, 26, 47, 0), "ERROR: apport (pid 486722) Thu Jan 12 22:26:47 2023 WITA: called for pid 486450, signal 6, core limit 0, dump mode 1"),
            (6, 39, (O_WIT, 2022, 6, 28, 1, 51, 12, 0), "MESSG Tuesday, Jun 28 01:51:12 2022 WIT"),
            (6, 36, (O_WITA, 2022, 6, 28, 1, 51, 12, 0), "MESSG Tue, Jun 28 01:51:12 2022 WITA:FOOBAR"),
            (6, 35, (O_WST, 2022, 6, 28, 1, 51, 12, 0), "MESSG Tue. Jun 28 01:51:12 2022 WST FOOBAR"),
            (8, 37, (O_YAKT, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue Jun 28 01:51:12 2022 YAKT"),
            (8, 37, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), "MESSAGE Tue Jun  2 01:51:12 2022 YAKT"),
            (8, 37, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), "MESSAGE Tue Jun 02 01:51:12 2022 YAKT"),
            (8, 36, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), "MESSAGE Tue Jun 2 01:51:12 2022 YAKT"),
            (8, 37, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue Jun 28 01:51:12 2022 YEKT FOOBAR"),
            (8, 37, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue Jun 28 01:51:12 2022 yekt foobar"),
            (8, 38, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue June 28 01:51:12 2022 yekt foobar"),
            (8, 38, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue JUNE 28 01:51:12 2022 yekt foobar"),
            (8, 38, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue june 28 01:51:12 2022 yekt foobar"),
        ],
        line!(),
    ),
    DTPD!(
        concatcp!(CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_YEAR, RP_NOALNUM),
        DTFSS_BdHMSY, 0, 1024, CGN_DAYa, CGN_YEAR,
        &[
            (27, 51, (O_L, 2023, 1, 12, 22, 26, 47, 0), "ERROR: apport (pid 486722) Thu Jan 12 22:26:47 2023: called for pid 486450, signal 6, core limit 0, dump mode 1"),
            (6, 35, (O_L, 2022, 6, 28, 1, 51, 12, 0), "MESSG Tuesday, Jun 28 01:51:12 2022 "),
            (6, 35, (O_L, 2022, 6, 28, 1, 51, 12, 0), "MESSG Tuesday, Jun 28 01:51:12 2022"),
            (6, 31, (O_L, 2022, 6, 28, 1, 51, 12, 0), "MESSG Tue, Jun 28 01:51:12 2022 FOOBAR"),
            (7, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), "messg: Tue. Jun 28 01:51:12 2022 WST"),
            (8, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue Jun 28 01:51:12 2022[abc"),
            (8, 32, (O_L, 2022, 6, 2, 1, 51, 12, 0), "MESSAGE Tue Jun  2 01:51:12 2022[abc"),
            (8, 32, (O_L, 2022, 6, 2, 1, 51, 12, 0), "MESSAGE Tue Jun 02 01:51:12 2022;abc"),
            (8, 31, (O_L, 2022, 6, 2, 1, 51, 12, 0), "MESSAGE Tue Jun 2 01:51:12 2022 YAKT"),
            (8, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue Jun 28 01:51:12 2022 YEKT FOOBAR"),
            (8, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue Jun 28 01:51:12 2022 foobar"),
            (8, 33, (O_L, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE Tue June 28 01:51:12 2022               foobar"),
            (8, 33, (O_L, 2022, 6, 28, 1, 51, 12, 0), "FOOBAR! Tue JUNE 28 01:51:12 2022[YEKT]"),
            (8, 33, (O_L, 2022, 6, 28, 1, 51, 12, 0), "MESSAGE|Tue JUNE 28 01:51:12 2022|YEKT|foobar!"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // file `FedoraRemix29/hawkeye.log`
    //
    //                1         2         3         4
    //      01234567890123456789012345678901234567890
    //      INFO Jun-16 14:09:58 === Started libdnf-0.31.0 ===
    //      DEBUG Jun-16 14:09:58 fetching rpmdb
    //
    DTPD!(
        concatcp!(CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NOALNUM),
        DTFSS_BdHMS, 0, 128, CGN_MONTH, CGN_SECOND,
        &[
            (5, 20, (O_L, YD, 6, 16, 14, 9, 58, 0), "INFO Jun-16 14:09:58 === Started libdnf-0.31.0 ==="),
            (6, 21, (O_L, YD, 6, 16, 14, 9, 58, 0), "DEBUG Jun-16 14:09:58 fetching rpmdb"),
        ],
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
    //     [    0.000000] kernel: Linux version 5.15.0-43-generic (build@lcy02-amd64-076) (gcc (Ubuntu 11.2.0-19ubuntu1) 11.2.0, GNU ld (GNU Binutils for Ubuntu) 2.38) #46-Ubuntu SMP Tue Jul 12 10:30:17 UTC 2022 (Ubuntu 5.15.0-43.46-generic 5.15.39)
    //     [    0.000000] kernel: Command line: BOOT_IMAGE=/boot/vmlinuz-5.15.0-43-generic root=UUID=136735fa-5cc1-470f-9359-ee736e42f844 ro console=tty1 console=ttyS0 net.ifnames=0 biosdevname=0
    //     [    0.000000] kernel: KERNEL supported cpus:
    //     [    0.000000] kernel:   Intel GenuineIntel
    //
    //DTPD!(
    //    concatcp!(r"^\[", RP_BLANKSq, CGP_UPTIME, r"\]", RP_BLANK),
    //    DTFSS_u, 0, 20, CGN_UPTIME, CGN_UPTIME,
    //    &[
    //        (0, 0, DUMMY_ARGS, "[    0.000000] kernel: KERNEL supported cpus:"),
    //        (0, 0, DUMMY_ARGS, "[    5.364159] kernel: ISO 9660 Extensions: RRIP_1991A"),
    //    ],
    //    line!(),
    //),
];

lazy_static! {
    /// Run-time created copy of [`DATETIME_PARSE_DATAS`] with compiled
    /// [`Regex`].
    ///
    /// [`Regex`]: https://docs.rs/regex/1.6.0/regex/bytes/struct.Regex.html
    // TODO: cost-savings: each compiled regex requires ≈133,000 Bytes on the heap according
    //       to `./tools/valgrind-massif.sh`. This is a lot of memory.
    //       An easy way to reduce baseline heap-use is drop the unused ones.
    //       That would need to occur after all file threads have passed the stage 1 blockzero analysis.
    pub(crate) static ref DATETIME_PARSE_DATAS_REGEX_VEC: DateTimeParseInstrsRegexVec = {
        defn!("init DATETIME_PARSE_DATAS_REGEX_VEC");
        let mut datas: DateTimeParseInstrsRegexVec = DateTimeParseInstrsRegexVec::with_capacity(
            DATETIME_PARSE_DATAS_LEN
        );
        for (_i, data) in DATETIME_PARSE_DATAS.iter().enumerate() {
            defo!("init RegEx {:?}", _i);
            datas.push(Regex::new(data.regex_pattern).unwrap());
        }
        defx!();

        datas
    };
}

// TODO: Issue #6 handle all Unicode whitespace.
//       This fn is essentially counteracting an errant call to
//       `std::string:trim` within `Local.datetime_from_str`.
//       `trim` removes "Unicode Derived Core Property White_Space".
//       This implementation handles three whitespace chars. There are
//       twenty-five whitespace chars according to
//       <https://en.wikipedia.org/wiki/Unicode_character_property#Whitespace>.
//
/// Match spaces at beginning and ending of `value`.
///
/// Workaround for chrono
/// [Issue #660](https://github.com/chronotope/chrono/issues/660).
#[allow(non_snake_case)]
pub fn datetime_from_str_workaround_Issue660(
    value: &str,
    pattern: &DateTimePattern_str,
) -> bool {
    const SPACES: &str = " ";
    const TABS: &str = "\t";
    const LINE_ENDS: &str = "\n\r";

    // match whitespace forwards from beginning
    let mut v_sc: u32 = 0; // `value` spaces count
    let mut v_tc: u32 = 0; // `value` tabs count
    let mut v_ec: u32 = 0; // `value` line ends count
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
    let mut p_sc: u32 = 0; // `pattern` space count
    let mut p_tc: u32 = 0; // `pattern` tab count
    let mut p_ec: u32 = 0; // `pattern` line ends count
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

/// Decoding [\[`u8`\]] bytes to a [`str`] takes a surprisingly long amount of
/// time, according to script `tools/flamegraph.sh`.
///
/// First check `u8` slice with custom simplistic checker that, in case of
/// complications, falls back to using higher-resource and more-precise checker
/// [`encoding_rs::mem::utf8_latin1_up_to`].
///
/// This uses built-in unsafe [`from_utf8_unchecked`].
///
/// See `benches/bench_decode_utf.rs` for comparison of `bytes` → `str`
/// decode strategies.
///
/// [\[`u8`\]]: u8
/// [`str`]: str
/// [`encoding_rs::mem::utf8_latin1_up_to`]: <https://docs.rs/encoding_rs/0.8.31/encoding_rs/mem/fn.utf8_latin1_up_to.html>
/// [`from_utf8_unchecked`]: std::str::from_utf8_unchecked
#[inline(always)]
pub fn u8_to_str(data: &[u8]) -> Option<&str> {
    let dts: &str;
    let mut fallback = false;
    // custom check for UTF8; fast but imperfect
    if !data.is_ascii() {
        fallback = true;
    }
    if fallback {
        // found non-ASCII, fallback to checking with `utf8_latin1_up_to`
        // which is a thorough check
        let va = encoding_rs::mem::utf8_latin1_up_to(data);
        if va != data.len() {
            // TODO: this needs a better resolution
            return None; // invalid UTF8
        }
    }
    unsafe {
        dts = std::str::from_utf8_unchecked(data);
    };
    Some(dts)
}

/// Convert a [`&str`] to a chrono [`Option<DateTime<FixedOffset>>`]
/// instance.
///
/// Compensate for a missing timezone.
///
/// - `data` to parse that has a datetime string
/// - strftime `pattern` to use for parsing
/// - `has_tz`, the `pattern` has a timezone (`%Z`, `%z`, etc.)?
/// - `tz_offset` fallback timezone offset when `!has_tz`
///
/// [`&str`]: str
/// [`Option<DateTime<FixedOffset>>`]: https://docs.rs/chrono/0.4.22/chrono/struct.DateTime.html#impl-DateTime%3CFixedOffset%3E
pub fn datetime_parse_from_str(
    data: &str,
    pattern: &DateTimePattern_str,
    has_tz: bool,
    tz_offset: &FixedOffset,
) -> DateTimeLOpt {
    defn!("(pattern {:?}, has_tz {}, tz_offset {:?}, data {:?})", pattern, has_tz, tz_offset, str_to_String_noraw(data));

    // saved rust playground for quick testing chrono `DateTime::parse_from_str`
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=e6f44c79dbb3d2c05c55ffba9bd91c76

    // if `has_tz` then create a `DateTime`.
    // else if `!has_tz` then create a `NaiveDateTime`, then convert that to `DateTime` with aid
    // of crate `chrono_tz`.
    if has_tz {
        match DateTime::parse_from_str(data, pattern) {
            Ok(val) => {
                defo!(
                    "DateTime::parse_from_str({:?}, {:?}) extrapolated DateTime {:?}",
                    str_to_String_noraw(data),
                    pattern,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `data`
                //       and `dt_pattern`
                if !datetime_from_str_workaround_Issue660(data, pattern) {
                    defn!("skip match due to chrono Issue #660");
                    return None;
                }
                defx!("return Some({:?})", val);

                Some(val)
            }
            Err(_err) => {
                defx!("DateTime::parse_from_str({:?}, {:?}) failed ParseError: {}", data, pattern, _err);

                None
            }
        }
    } else {
        // !has_tz
        // no timezone in `pattern` so first convert to a `NaiveDateTime` instance
        let dt_naive = match NaiveDateTime::parse_from_str(data, pattern) {
            Ok(val) => {
                defo!(
                    "NaiveDateTime.parse_from_str({:?}, {:?}) extrapolated NaiveDateTime {:?}",
                    str_to_String_noraw(data),
                    pattern,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `data`
                //       and `pattern`
                if !datetime_from_str_workaround_Issue660(data, pattern) {
                    defx!("skip match due to chrono Issue #660");
                    return None;
                }
                defx!("return {:?}", val);

                val
            }
            Err(_err) => {
                defx!("NaiveDateTime.parse_from_str({:?}, {:?}) failed ParseError: {}", data, pattern, _err);
                return None;
            }
        };
        // second convert the `NaiveDateTime` instance to `DateTime<FixedOffset>` instance
        match tz_offset
            .from_local_datetime(&dt_naive)
            .earliest()
        {
            Some(val) => {
                defo!(
                    "tz_offset.from_local_datetime({:?}).earliest() extrapolated NaiveDateTime {:?}",
                    dt_naive,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `data`
                //       and `pattern`
                if !datetime_from_str_workaround_Issue660(data, pattern) {
                    defx!("skip match due to chrono Issue #660, return None");
                    return None;
                }
                defx!("return {:?}", Some(val));

                Some(val)
            }
            None => {
                defx!("tz_offset.from_local_datetime({:?}, {:?}) returned None, return None", data, pattern);
                None
            }
        }
    }
}

/// Call [`datetime_parse_from_str`] with a `pattern` containing a timezone.
///
/// [`datetime_parse_from_str`]: datetime_parse_from_str
pub fn datetime_parse_from_str_w_tz(
    data: &str,
    pattern: &DateTimePattern_str,
) -> DateTimeLOpt {
    datetime_parse_from_str(
        data,
        pattern,
        true,
        &FixedOffset::east_opt(-9999).unwrap(),
    )
}

/// Data of interest from a set of [`regex::Captures`] for a datetime
/// substring found in a [`Line`].
///
/// - datetime substring begin index
/// - datetime substring end index
/// - datetime
///
/// [`Line`]: crate::data::line::Line
/// [`regex::Captures`]: https://docs.rs/regex/1.6.0/regex/bytes/struct.Captures.html
pub type CapturedDtData = (LineIndex, LineIndex, DateTimeL);

/// Macro helper to `captures_to_buffer_bytes`.
macro_rules! copy_capturegroup_to_buffer {
    (
        $name:ident,
        $captures:ident,
        $buffer:ident,
        $at:ident
    ) => {
        let len_: usize = $captures
            .name($name)
            .as_ref()
            .unwrap()
            .as_bytes()
            .len();
        defo!("copy_capturegroup_to_buffer! buffer[{:?}‥{:?}]", $at, $at + len_);
        $buffer[$at..$at + len_].copy_from_slice(
            $captures
                .name($name)
                .as_ref()
                .unwrap()
                .as_bytes(),
        );
        $at += len_;
    };
}

/// Macro helper to `captures_to_buffer_bytes`.
macro_rules! copy_slice_to_buffer {
    (
        $u8_slice:expr,
        $buffer:ident,
        $at:ident
    ) => {
        let len_: usize = $u8_slice.len();
        defo!("copy_slice_to_buffer! buffer[{:?}‥{:?}]", $at, $at + len_);
        $buffer[$at..$at + len_].copy_from_slice($u8_slice);
        $at += len_;
    };
}

/// Macro helper to `captures_to_buffer_bytes`.
macro_rules! copy_u8_to_buffer {
    (
        $u8_:expr,
        $buffer:ident,
        $at:ident
    ) => {
        defo!("copy_slice_to_buffer! buffer[{:?}] = {:?}", $at, $u8_);
        $buffer[$at] = $u8_;
        $at += 1;
    };
}

// Variables `const MONTH_` are helpers to [`month_bB_to_month_m_bytes`].
//
// TODO: replace `to_byte_array` with rust experimental feature `const_str_as_bytes`
//       https://doc.bccnsoft.com/docs/rust-1.36.0-docs-html/unstable-book/library-features/const-str-as-bytes.html#const_str_as_bytes
//       https://github.com/rust-lang/rust/issues/57563
//
// MONTH_XY_B_l, month XY as `%B` form, lowercase
// MONTH_XY_b_l, month XY as `%b` form, lowercase
// MONTH_XY_B_u, month XY as `%B` form, uppercase
// MONTH_XY_b_u, month XY as `%b` form, uppercase
// MONTH_XY_b_U, month XY as `%b` form, uppercase all

const MONTH_01_b_l: &[u8] = &to_byte_array!("jan");
const MONTH_01_b_u: &[u8] = &to_byte_array!("Jan");
const MONTH_01_b_U: &[u8] = &to_byte_array!("JAN");
const MONTH_01_B_l: &[u8] = &to_byte_array!("january");
const MONTH_01_B_u: &[u8] = &to_byte_array!("January");
const MONTH_01_B_U: &[u8] = &to_byte_array!("JANUARY");
const MONTH_01_m: &[u8] = &to_byte_array!("01");
const MONTH_02_b_l: &[u8] = &to_byte_array!("feb");
const MONTH_02_b_u: &[u8] = &to_byte_array!("Feb");
const MONTH_02_b_U: &[u8] = &to_byte_array!("FEB");
const MONTH_02_B_l: &[u8] = &to_byte_array!("february");
const MONTH_02_B_u: &[u8] = &to_byte_array!("February");
const MONTH_02_B_U: &[u8] = &to_byte_array!("FEBRUARY");
const MONTH_02_m: &[u8] = &to_byte_array!("02");
const MONTH_03_b_l: &[u8] = &to_byte_array!("mar");
const MONTH_03_b_u: &[u8] = &to_byte_array!("Mar");
const MONTH_03_b_U: &[u8] = &to_byte_array!("MAR");
const MONTH_03_B_l: &[u8] = &to_byte_array!("march");
const MONTH_03_B_u: &[u8] = &to_byte_array!("March");
const MONTH_03_B_U: &[u8] = &to_byte_array!("MARCH");
const MONTH_03_m: &[u8] = &to_byte_array!("03");
const MONTH_04_b_l: &[u8] = &to_byte_array!("apr");
const MONTH_04_b_u: &[u8] = &to_byte_array!("Apr");
const MONTH_04_b_U: &[u8] = &to_byte_array!("APR");
const MONTH_04_B_l: &[u8] = &to_byte_array!("april");
const MONTH_04_B_u: &[u8] = &to_byte_array!("April");
const MONTH_04_B_U: &[u8] = &to_byte_array!("APRIL");
const MONTH_04_m: &[u8] = &to_byte_array!("04");
const MONTH_05_b_l: &[u8] = &to_byte_array!("may");
const MONTH_05_b_u: &[u8] = &to_byte_array!("May");
const MONTH_05_b_U: &[u8] = &to_byte_array!("MAY");
#[allow(dead_code)]
const MONTH_05_B_l: &[u8] = &to_byte_array!("may"); // not used, defined for completeness
#[allow(dead_code)]
const MONTH_05_B_u: &[u8] = &to_byte_array!("May"); // not used, defined for completeness
#[allow(dead_code)]
const MONTH_05_B_U: &[u8] = &to_byte_array!("MAY"); // not used, defined for completeness
const MONTH_05_m: &[u8] = &to_byte_array!("05");
const MONTH_06_b_l: &[u8] = &to_byte_array!("jun");
const MONTH_06_b_u: &[u8] = &to_byte_array!("Jun");
const MONTH_06_b_U: &[u8] = &to_byte_array!("JUN");
const MONTH_06_B_l: &[u8] = &to_byte_array!("june");
const MONTH_06_B_u: &[u8] = &to_byte_array!("June");
const MONTH_06_B_U: &[u8] = &to_byte_array!("JUNE");
const MONTH_06_m: &[u8] = &to_byte_array!("06");
const MONTH_07_b_l: &[u8] = &to_byte_array!("jul");
const MONTH_07_b_u: &[u8] = &to_byte_array!("Jul");
const MONTH_07_b_U: &[u8] = &to_byte_array!("JUL");
const MONTH_07_B_l: &[u8] = &to_byte_array!("july");
const MONTH_07_B_u: &[u8] = &to_byte_array!("July");
const MONTH_07_B_U: &[u8] = &to_byte_array!("JULY");
const MONTH_07_m: &[u8] = &to_byte_array!("07");
const MONTH_08_b_l: &[u8] = &to_byte_array!("aug");
const MONTH_08_b_u: &[u8] = &to_byte_array!("Aug");
const MONTH_08_b_U: &[u8] = &to_byte_array!("AUG");
const MONTH_08_B_l: &[u8] = &to_byte_array!("august");
const MONTH_08_B_u: &[u8] = &to_byte_array!("August");
const MONTH_08_B_U: &[u8] = &to_byte_array!("AUGUST");
const MONTH_08_m: &[u8] = &to_byte_array!("08");
const MONTH_09_b_l: &[u8] = &to_byte_array!("sep");
const MONTH_09_b_u: &[u8] = &to_byte_array!("Sep");
const MONTH_09_b_U: &[u8] = &to_byte_array!("SEP");
const MONTH_09_B_l: &[u8] = &to_byte_array!("september");
const MONTH_09_B_u: &[u8] = &to_byte_array!("September");
const MONTH_09_B_U: &[u8] = &to_byte_array!("SEPTEMBER");
const MONTH_09_m: &[u8] = &to_byte_array!("09");
const MONTH_10_b_l: &[u8] = &to_byte_array!("oct");
const MONTH_10_b_u: &[u8] = &to_byte_array!("Oct");
const MONTH_10_b_U: &[u8] = &to_byte_array!("OCT");
const MONTH_10_B_l: &[u8] = &to_byte_array!("october");
const MONTH_10_B_u: &[u8] = &to_byte_array!("October");
const MONTH_10_B_U: &[u8] = &to_byte_array!("OCTOBER");
const MONTH_10_m: &[u8] = &to_byte_array!("10");
const MONTH_11_b_l: &[u8] = &to_byte_array!("nov");
const MONTH_11_b_u: &[u8] = &to_byte_array!("Nov");
const MONTH_11_b_U: &[u8] = &to_byte_array!("NOV");
const MONTH_11_B_l: &[u8] = &to_byte_array!("november");
const MONTH_11_B_u: &[u8] = &to_byte_array!("November");
const MONTH_11_B_U: &[u8] = &to_byte_array!("NOVEMBER");
const MONTH_11_m: &[u8] = &to_byte_array!("11");
const MONTH_12_b_l: &[u8] = &to_byte_array!("dec");
const MONTH_12_b_u: &[u8] = &to_byte_array!("Dec");
const MONTH_12_b_U: &[u8] = &to_byte_array!("DEC");
const MONTH_12_B_l: &[u8] = &to_byte_array!("december");
const MONTH_12_B_u: &[u8] = &to_byte_array!("December");
const MONTH_12_B_U: &[u8] = &to_byte_array!("DECEMBER");
const MONTH_12_m: &[u8] = &to_byte_array!("12");

/// Transform `%B`, `%b` (i.e. `"January"`, `"Jan"`) to `%m` (i.e. `"01"`).
///
/// Helper to [`captures_to_buffer_bytes`].
#[allow(non_snake_case)]
fn month_bB_to_month_m_bytes(
    data: &[u8],
    buffer: &mut [u8],
) {
    match data {
        // try *b* matches first; it is more common
        MONTH_01_b_l | MONTH_01_b_u | MONTH_01_b_U => buffer.copy_from_slice(MONTH_01_m),
        MONTH_02_b_l | MONTH_02_b_u | MONTH_02_b_U => buffer.copy_from_slice(MONTH_02_m),
        MONTH_03_b_l | MONTH_03_b_u | MONTH_03_b_U => buffer.copy_from_slice(MONTH_03_m),
        MONTH_04_b_l | MONTH_04_b_u | MONTH_04_b_U => buffer.copy_from_slice(MONTH_04_m),
        MONTH_05_b_l | MONTH_05_b_u | MONTH_05_b_U => buffer.copy_from_slice(MONTH_05_m),
        MONTH_06_b_l | MONTH_06_b_u | MONTH_06_b_U => buffer.copy_from_slice(MONTH_06_m),
        MONTH_07_b_l | MONTH_07_b_u | MONTH_07_b_U => buffer.copy_from_slice(MONTH_07_m),
        MONTH_08_b_l | MONTH_08_b_u | MONTH_08_b_U => buffer.copy_from_slice(MONTH_08_m),
        MONTH_09_b_l | MONTH_09_b_u | MONTH_09_b_U => buffer.copy_from_slice(MONTH_09_m),
        MONTH_10_b_l | MONTH_10_b_u | MONTH_10_b_U => buffer.copy_from_slice(MONTH_10_m),
        MONTH_11_b_l | MONTH_11_b_u | MONTH_11_b_U => buffer.copy_from_slice(MONTH_11_m),
        MONTH_12_b_l | MONTH_12_b_u | MONTH_12_b_U => buffer.copy_from_slice(MONTH_12_m),
        // then try *B* matches
        MONTH_01_B_l | MONTH_01_B_u | MONTH_01_B_U => buffer.copy_from_slice(MONTH_01_m),
        MONTH_02_B_l | MONTH_02_B_u | MONTH_02_B_U => buffer.copy_from_slice(MONTH_02_m),
        MONTH_03_B_l | MONTH_03_B_u | MONTH_03_B_U => buffer.copy_from_slice(MONTH_03_m),
        MONTH_04_B_l | MONTH_04_B_u | MONTH_04_B_U => buffer.copy_from_slice(MONTH_04_m),
        //MONTH_05_B_l | MONTH_05_B_u | MONTH_05_B_U => buffer.copy_from_slice(MONTH_05_m),
        MONTH_06_B_l | MONTH_06_B_u | MONTH_06_B_U => buffer.copy_from_slice(MONTH_06_m),
        MONTH_07_B_l | MONTH_07_B_u | MONTH_07_B_U => buffer.copy_from_slice(MONTH_07_m),
        MONTH_08_B_l | MONTH_08_B_u | MONTH_08_B_U => buffer.copy_from_slice(MONTH_08_m),
        MONTH_09_B_l | MONTH_09_B_u | MONTH_09_B_U => buffer.copy_from_slice(MONTH_09_m),
        MONTH_10_B_l | MONTH_10_B_u | MONTH_10_B_U => buffer.copy_from_slice(MONTH_10_m),
        MONTH_11_B_l | MONTH_11_B_u | MONTH_11_B_U => buffer.copy_from_slice(MONTH_11_m),
        MONTH_12_B_l | MONTH_12_B_u | MONTH_12_B_U => buffer.copy_from_slice(MONTH_12_m),
        data_ => {
            panic!("month_bB_to_month_m_bytes: unexpected month value {:?}", data_);
        }
    }
}

/// Put [`Captures`] into a `String` buffer in a particular order and
/// formatting.
///
/// This bridges the crate `regex` regular expression pattern strings,
/// [`DateTimeParseInstr::regex_pattern`], to crate `chrono` strftime strings,
/// [`DateTimeParseInstr::dt_pattern`].
///
/// Directly relates to datetime format `dt_pattern` values in
/// [`DATETIME_PARSE_DATAS`] which use `DTFSS_YmdHMS`, etc.
///
/// Transforms `%B` acceptable value to `%m` acceptable value.
///
/// Transforms `%e` acceptable value to `%d` acceptable value.
///
/// [`Captures`]: https://docs.rs/regex/1.6.0/regex/bytes/struct.Captures.html
/// [`DateTimeParseInstr::regex_pattern`]: crate::data::datetime::DateTimeParseInstr::regex_pattern
/// [`DateTimeParseInstr::dt_pattern`]: crate::data::datetime::DateTimeParseInstr::dt_pattern
#[inline(always)]
pub(crate) fn captures_to_buffer_bytes(
    buffer: &mut [u8],
    captures: &regex::bytes::Captures,
    year_opt: &Option<Year>,
    tz_offset: &FixedOffset,
    dtfs: &DTFSSet,
) -> usize {
    defn!("(…, …, year_opt {:?}, tz_offset {:?}, …)", year_opt, tz_offset);

    let mut at: usize = 0;

    // year
    defo!("process <year>…");
    match captures
        .name(CGN_YEAR)
        .as_ref()
    {
        Some(match_) => {
            copy_slice_to_buffer!(match_.as_bytes(), buffer, at);
        }
        None => {
            match year_opt {
                Some(year) => {
                    // TODO: 2022/07/11 cost-savings: pass in `Option<&[u8]>`, avoid creating `String`
                    let year_s: String = year.to_string();
                    debug_assert_eq!(year_s.len(), 4, "Bad year string {:?}", year_s);
                    defo!("using fallback year {:?}", year_s);
                    copy_slice_to_buffer!(year_s.as_bytes(), buffer, at);
                }
                None => {
                    defo!("using hardcoded dummy year {:?}", YEAR_FALLBACKDUMMY);
                    copy_slice_to_buffer!(YEAR_FALLBACKDUMMY.as_bytes(), buffer, at);
                }
            }
        }
    }
    // month
    defo!("process <month>…");
    match dtfs.month {
        DTFS_Month::b | DTFS_Month::B => {
            month_bB_to_month_m_bytes(
                captures
                    .name(CGN_MONTH)
                    .as_ref()
                    .unwrap()
                    .as_bytes(),
                &mut buffer[at..at + 2],
            );
            at += 2;
        }
        DTFS_Month::m => {
            copy_capturegroup_to_buffer!(CGN_MONTH, captures, buffer, at);
        }
    }
    // day
    defo!("process <day>…");
    match dtfs.day {
        DTFS_Day::_e_or_d => {
            let day: &[u8] = captures
                .name(CGN_DAY)
                .as_ref()
                .unwrap()
                .as_bytes();
            debug_assert_ge!(day.len(), 1, "bad named group 'day' data {:?}, expected data ge 1", day);
            debug_assert_le!(day.len(), 2, "bad named group 'day' data {:?}, expected data le 2", day);
            match day.len() {
                1 => {
                    // change day "8" (%e) to "08" (%d)
                    copy_u8_to_buffer!(b'0', buffer, at);
                    copy_u8_to_buffer!(day[0], buffer, at);
                }
                2 => {
                    match day[0] {
                        b' ' => {
                            // change day " 8" (%e) to "08" (%d)
                            copy_u8_to_buffer!(b'0', buffer, at);
                            copy_u8_to_buffer!(day[1], buffer, at);
                        }
                        _ => {
                            copy_slice_to_buffer!(day, buffer, at);
                        }
                    }
                }
                _ => {
                    panic!("bad day.len() {}", day.len());
                }
            }
        }
    }
    // Day pattern `%a` (`Monday`, 'Tue`, etc.) (capture group `CGN_DAYa`) is captured but not
    // passed along to chrono functions.

    // day-time divider
    defo!("process date-time divider…");
    copy_u8_to_buffer!(b'T', buffer, at);
    // hour
    defo!("process <hour>…");
    copy_capturegroup_to_buffer!(CGN_HOUR, captures, buffer, at);
    // minute
    defo!("process <minute>…");
    copy_capturegroup_to_buffer!(CGN_MINUTE, captures, buffer, at);
    // second
    defo!("process <second>…");
    match dtfs.second {
        DTFS_Second::S => {
            copy_capturegroup_to_buffer!(CGN_SECOND, captures, buffer, at);
        }
        DTFS_Second::_fill => {}
    }
    // fractional
    defo!("process <factional>…");
    match dtfs.fractional {
        DTFS_Fractional::f => {
            defo!("matched DTFS_Fractional::f");
            copy_u8_to_buffer!(b'.', buffer, at);
            let fractional: &[u8] = match captures.name(CGN_FRACTIONAL).as_ref() {
                Some(match_) => {
                    match_.as_bytes()
                }
                None => {
                    panic!("DTFSSet has set {:?} yet no fraction was captured", dtfs.fractional);
                }
            };
            let len = fractional.len();
            defo!("match len {:?}", len);
            match len {
                0 => {
                    copy_slice_to_buffer!(fractional, buffer, at);
                    copy_slice_to_buffer!(b"000000000", buffer, at);
                }
                1 => {
                    copy_slice_to_buffer!(fractional, buffer, at);
                    copy_slice_to_buffer!(b"00000000", buffer, at);
                }
                2 => {
                    copy_slice_to_buffer!(fractional, buffer, at);
                    copy_slice_to_buffer!(b"0000000", buffer, at);
                }
                3 => {
                    copy_slice_to_buffer!(fractional, buffer, at);
                    copy_slice_to_buffer!(b"000000", buffer, at);
                }
                4 => {
                    copy_slice_to_buffer!(fractional, buffer, at);
                    copy_slice_to_buffer!(b"00000", buffer, at);
                }
                5 => {
                    copy_slice_to_buffer!(fractional, buffer, at);
                    copy_slice_to_buffer!(b"0000", buffer, at);
                }
                6 => {
                    copy_slice_to_buffer!(fractional, buffer, at);
                    copy_slice_to_buffer!(b"000", buffer, at);
                }
                7 => {
                    copy_slice_to_buffer!(fractional, buffer, at);
                    copy_slice_to_buffer!(b"00", buffer, at);
                }
                8 => {
                    copy_slice_to_buffer!(fractional, buffer, at);
                    copy_slice_to_buffer!(b"0", buffer, at);
                }
                9 => {
                    copy_slice_to_buffer!(fractional, buffer, at);
                }
                10 | 11 | 12 => {
                    // fractional is too large, copy only left-most 9 chars
                    copy_slice_to_buffer!(&fractional[..9], buffer, at);
                }
                _ => {
                    // something is very wrong
                }
            }
        }
        DTFS_Fractional::_none => {}
    }
    // tz
    defo!("process <tz>…");
    match dtfs.tz {
        DTFS_Tz::_fill => {
            // TODO: cost-savings: pass pre-created TZ `&str`
            let tzs: String = tz_offset.to_string();
            copy_slice_to_buffer!(tzs.as_bytes(), buffer, at);
        }
        DTFS_Tz::z | DTFS_Tz::zc | DTFS_Tz::zp => {
            // for data passed to chrono `DateTime::parse_from_str`,
            // replace Unicode "minus sign" to ASCII "hyphen-minus"
            // see https://github.com/chronotope/chrono/issues/835
            let captureb = captures
                .name(CGN_TZ)
                .as_ref()
                .unwrap()
                .as_bytes();
            match captureb.starts_with(MINUS_SIGN) {
                true => {
                    defo!("found Unicode 'minus sign', transform to ASCII 'hyphen-minus'");
                    // found Unicode "minus sign", replace with ASCII
                    // "hyphen-minus"
                    copy_slice_to_buffer!(HYPHEN_MINUS, buffer, at);
                    // copy data remaining after Unicode "minus sign"
                    match std::str::from_utf8(&captureb) {
                        Ok(val) => {
                            match val.char_indices().nth(1) {
                                Some((offset, _)) => {
                                    copy_slice_to_buffer!(val[offset..].as_bytes(), buffer, at);
                                }
                                None => {
                                    // something is wrong with captured value
                                    // ignore it
                                }
                            }
                        }
                        Err(_err) => {
                            // something is wrong with captured value, ignore it
                        }
                    }
                }
                false => {
                    copy_slice_to_buffer!(captureb, buffer, at);
                }
            }
        }
        DTFS_Tz::Z => {
            #[allow(non_snake_case)]
            let tzZ: &str = u8_to_str(
                captures
                    .name(CGN_TZ)
                    .as_ref()
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
            match MAP_TZZ_TO_TZz.get_key_value(tzZ) {
                Some((_tz_abbr, tz_offset_val)) => {
                    match tz_offset_val.is_empty() {
                        true => {
                            // given an ambiguous timezone name, fallback to
                            // passed TZ offset
                            // TODO: cost-savings: pre-create the `tz_offset` entries as bytes
                            let tzs: String = tz_offset.to_string();
                            copy_slice_to_buffer!(tzs.as_bytes(), buffer, at);
                        }
                        false => {
                            // given an unambiguous timezone name, use
                            // associated offset
                            // TODO: cost-savings: pre-create the `tz_offset` entries as bytes
                            let tzs: String = tz_offset_val.to_string();
                            copy_slice_to_buffer!(tzs.as_bytes(), buffer, at);
                        }
                    }
                }
                None => {
                    // cannot find entry in MAP_TZZ_TO_TZz, use passed TZ offset
                    // TODO: cost-savings: pre-create the `tz_offset` entries as bytes
                    let tzs: String = tz_offset.to_string();
                    copy_slice_to_buffer!(tzs.as_bytes(), buffer, at);
                }
            }
        }
    }

    defx!("return {:?}", at);

    at
}

/// Run [`regex::Captures`] on the `data` then convert to a chrono
/// [`Option<DateTime<FixedOffset>>`] instance. Uses matching and pattern
/// information hardcoded in [`DATETIME_PARSE_DATAS`].
///
/// [`DATETIME_PARSE_DATAS`]: [DATETIME_PARSE_DATAS]
/// [`regex::Captures`]: https://docs.rs/regex/1.6.0/regex/bytes/struct.Regex.html#method.captures
/// [`Option<DateTime<FixedOffset>>`]: https://docs.rs/chrono/0.4.22/chrono/struct.DateTime.html#impl-DateTime%3CFixedOffset%3E
pub fn bytes_to_regex_to_datetime(
    data: &[u8],
    index: &DateTimeParseInstrsIndex,
    year_opt: &Option<Year>,
    tz_offset: &FixedOffset,
) -> Option<CapturedDtData> {
    defn!("(…, {:?}, {:?}, {:?})", index, year_opt, tz_offset);

    let regex_: &Regex = match DATETIME_PARSE_DATAS_REGEX_VEC.get(*index) {
        Some(val) => val,
        None => {
            panic!("requested DATETIME_PARSE_DATAS_REGEX_VEC.get({}), returned None. DATETIME_PARSE_DATAS_REGEX_VEC.len() {}", index, DATETIME_PARSE_DATAS_REGEX_VEC.len());
        }
    };

    let captures: regex::bytes::Captures = match regex_.captures(data) {
        None => {
            defx!("regex: no captures (returned None)");
            return None;
        }
        Some(captures) => {
            deo!("regex: captures.len() {}", captures.len());

            captures
        }
    };
    if cfg!(debug_assertions) {
        for (i, name_opt) in regex_
            .capture_names()
            .enumerate()
        {
            let _match: regex::bytes::Match = match captures.get(i) {
                Some(m_) => m_,
                None => {
                    match name_opt {
                        Some(_name) => {
                            deo!("regex captures: {:2} {:<10} None", i, _name);
                        }
                        None => {
                            deo!("regex captures: {:2} {:<10} None", i, "None");
                        }
                    }
                    continue;
                }
            };
            match name_opt {
                Some(_name) => {
                    deo!(
                        "regex captures: {:2} {:<10} {:?}",
                        i,
                        _name,
                        buffer_to_String_noraw(_match.as_bytes())
                    );
                }
                None => {
                    deo!(
                        "regex captures: {:2} {:<10} {:?}",
                        i,
                        "NO NAME",
                        buffer_to_String_noraw(_match.as_bytes())
                    );
                }
            }
        }
    }
    // sanity check
    debug_assert!(
        !captures
            .iter()
            .any(|x| x.is_none()),
        "a match in the regex::Captures was None"
    );

    let dtpd: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*index];
    // copy regex matches into a buffer with predictable ordering
    // this ordering relates to datetime format strings in `DATETIME_PARSE_DATAS`
    // TODO: [2022/06/26] cost-savings: avoid a `String` alloc by passing precreated buffer
    const BUFLEN: usize = 35;
    let mut buffer: [u8; BUFLEN] = [0; BUFLEN];
    let copiedn = captures_to_buffer_bytes(&mut buffer, &captures, year_opt, tz_offset, &dtpd.dtfs);

    // use the `dt_format` to parse the buffer of regex matches
    let buffer_s: &str = u8_to_str(&buffer[0..copiedn]).unwrap();
    let dt = match datetime_parse_from_str(buffer_s, dtpd.dtfs.pattern, dtpd.dtfs.has_tz(), tz_offset) {
        Some(dt_) => dt_,
        None => {
            defx!("return None; datetime_parse_from_str returned None");
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

    defx!("return Some({:?}, {:?}, {:?})", dt_beg, dt_end, dt);
    Some((dt_beg, dt_end, dt))
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DateTime comparisons
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Describe the result of comparing one [`DateTimeL`] to one DateTime Filter.
///
/// [`DateTimeL`]: crate::data::datetime::DateTimeL
#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq)]
pub enum Result_Filter_DateTime1 {
    /// like Skip
    Pass,
    OccursAtOrAfter,
    OccursBefore,
}

impl Result_Filter_DateTime1 {
    /// Returns `true` if the result is `OccursAfter`.
    #[inline(always)]
    pub const fn is_after(&self) -> bool {
        matches!(*self, Result_Filter_DateTime1::OccursAtOrAfter)
    }

    /// Returns `true` if the result is `OccursBefore`.
    #[inline(always)]
    pub const fn is_before(&self) -> bool {
        matches!(*self, Result_Filter_DateTime1::OccursBefore)
    }
}

/// Describe the result of comparing one [`DateTimeL`] to two DateTime Filters
/// `(after, before)`.
///
/// [`DateTimeL`]: crate::data::datetime::DateTimeL
#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq)]
pub enum Result_Filter_DateTime2 {
    /// like Pass
    InRange,
    /// like Fail
    BeforeRange,
    /// like Fail
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

/// Compare passed [`DateTimeL`] `dt` to the passed filter `dt_filter`.
///
/// If `dt` is at or after `dt_filter` then return [`OccursAtOrAfter`]<br/>
/// If `dt` is before `dt_filter` then return [`OccursBefore`]<br/>
/// Else return [`Pass`] (including if `dt_filter` is `None`)
///
/// [`OccursAtOrAfter`]: crate::data::datetime::Result_Filter_DateTime1
/// [`OccursBefore`]: crate::data::datetime::Result_Filter_DateTime1
/// [`Pass`]: crate::data::datetime::Result_Filter_DateTime1
pub fn dt_after_or_before(
    dt: &DateTimeL,
    dt_filter: &DateTimeLOpt,
) -> Result_Filter_DateTime1 {
    if dt_filter.is_none() {
        defñ!("return Result_Filter_DateTime1::Pass; (no dt filters)");
        return Result_Filter_DateTime1::Pass;
    }

    let dt_a = &dt_filter.unwrap();
    defn!("comparing dt datetime {:?} to filter datetime {:?}", dt, dt_a);
    if dt < dt_a {
        defx!("return Result_Filter_DateTime1::OccursBefore; (dt {:?} is before dt_filter {:?})", dt, dt_a);
        return Result_Filter_DateTime1::OccursBefore;
    }
    defx!(
        "return Result_Filter_DateTime1::OccursAtOrAfter; (dt {:?} is at or after dt_filter {:?})",
        dt,
        dt_a
    );

    Result_Filter_DateTime1::OccursAtOrAfter
}

/// How does the passed [`DateTimeL`], `dt`, pass the optional `DateTimeLOpt`
/// filter instances,
/// `dt_filter_after` and `dt_filter_before`?  Is `dt` before ([`BeforeRange`]),
/// after ([`AfterRange`]), or in between ([`InRange`])?
///
/// If both filters are `Some` and `dt: DateTimeL` is "between" the filters then
/// return `InRange`.<br/>
/// If before then return `BeforeRange`.<br/>
/// If after then return `AfterRange`.
///
/// If filter `dt_filter_after` is `Some` and `dt: DateTimeL` is after that
/// filter then return `InRange`.<br/>
/// If before then return `BeforeRange`.
///
/// If filter `dt_filter_before` is `Some` and `dt: DateTimeL` is before that
/// filter then return `InRange`.<br/>
/// If after then return `AfterRange`.
///
/// If both filters are `None` then return `InRange`.
///
/// Comparisons are "inclusive" i.e. `dt` == `dt_filter_after` will return
/// `InRange`.
///
/// [`AfterRange`]: crate::data::datetime::Result_Filter_DateTime2::AfterRange
/// [`BeforeRange`]: crate::data::datetime::Result_Filter_DateTime2::BeforeRange
/// [`InRange`]: crate::data::datetime::Result_Filter_DateTime2::InRange
pub fn dt_pass_filters(
    dt: &DateTimeL,
    dt_filter_after: &DateTimeLOpt,
    dt_filter_before: &DateTimeLOpt,
) -> Result_Filter_DateTime2 {
    defn!("({:?}, {:?}, {:?})", dt, dt_filter_after, dt_filter_before);
    // TODO: use `match` statement instead of `if` statements
    if dt_filter_after.is_none() && dt_filter_before.is_none() {
        defx!("return {:?}; (no dt filters)", Result_Filter_DateTime2::InRange);
        return Result_Filter_DateTime2::InRange;
    }
    if dt_filter_after.is_some() && dt_filter_before.is_some() {
        defo!(" after {:?}" , dt_filter_after.unwrap().with_timezone(&*FO_0));
        defo!("     <");
        defo!("    dt {:?}", dt.with_timezone(&*FO_0));
        defo!("     <");
        defo!("before {:?}", dt_filter_before.unwrap().with_timezone(&*FO_0));
        let da = &dt_filter_after.unwrap();
        let db = &dt_filter_before.unwrap();
        assert_le!(da, db, "Bad datetime range values filter_after {:?} {:?} filter_before", da, db);
        if dt < da {
            defx!("return {:?}", Result_Filter_DateTime2::BeforeRange);
            return Result_Filter_DateTime2::BeforeRange;
        }
        if db < dt {
            defx!("return {:?}", Result_Filter_DateTime2::AfterRange);
            return Result_Filter_DateTime2::AfterRange;
        }
        // assert da < dt && dt < db
        assert_le!(da, dt, "Unexpected range values da dt");
        assert_le!(dt, db, "Unexpected range values dt db");
        defx!("return Result_Filter_DateTime2::InRange");

        Result_Filter_DateTime2::InRange
    } else if dt_filter_after.is_some() {
        defo!("after {:?}", dt_filter_after.unwrap().with_timezone(&*FO_0));
        defo!("    <");
        defo!("   dt {:?}", dt.with_timezone(&*FO_0));
        let da = &dt_filter_after.unwrap();
        if dt < da {
            defx!("return {:?}", Result_Filter_DateTime2::BeforeRange);
            return Result_Filter_DateTime2::BeforeRange;
        }
        defx!("return Result_Filter_DateTime2::InRange");

        Result_Filter_DateTime2::InRange
    } else {
        defo!("before {:?}", dt_filter_before.unwrap().with_timezone(&*FO_0));
        defo!("     <");
        defo!("    dt {:?}", dt.with_timezone(&*FO_0));
        let db = &dt_filter_before.unwrap();
        if db < dt {
            defx!("return {:?}", Result_Filter_DateTime2::AfterRange);
            return Result_Filter_DateTime2::AfterRange;
        }
        defx!("return {:?}", Result_Filter_DateTime2::InRange);

        Result_Filter_DateTime2::InRange
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// other miscellaneous DateTime function helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a new [`DateTimeL`] instance that uses the passed `DateTimeL`
/// month, day, and time, combined with the passed `Year`.
///
/// In case of error, return a copy of the passed `DateTimeL`.
// TODO: errors should return `Error`
pub fn datetime_with_year(
    datetime: &DateTimeL,
    year: &Year,
) -> DateTimeL {
    match datetime.with_year(*year) {
        Some(datetime_) => datetime_,
        None => *datetime,
    }
}

/// Convert passed [`SystemTime`] to [`DateTimeL`] with passed [`FixedOffset`].
///
/// [`FixedOffset`]: https://docs.rs/chrono/0.4.22/chrono/offset/struct.FixedOffset.html
/// [`SystemTime`]: std::time::SystemTime
pub fn systemtime_to_datetime(
    fixedoffset: &FixedOffset,
    systemtime: &SystemTime,
) -> DateTimeL {
    // https://users.rust-lang.org/t/convert-std-time-systemtime-to-chrono-datetime-datetime/7684/6
    let dtu: DateTime<Utc> = (*systemtime).into();

    dtu.with_timezone(fixedoffset)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// search a slice quickly (loop unroll version)
// loop unrolled implementation of `slice.contains` for a byte slice and a hardcoded array
// benchmark `benches/bench_slice_contains.rs` demonstrates this is faster
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_2_2(
    slice_: &[u8; 2],
    search: &[u8; 2],
) -> bool {
    for i in 0..1 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_3_2(
    slice_: &[u8; 3],
    search: &[u8; 2],
) -> bool {
    for i in 0..2 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_4_2(
    slice_: &[u8; 4],
    search: &[u8; 2],
) -> bool {
    for i in 0..3 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_5_2(
    slice_: &[u8; 5],
    search: &[u8; 2],
) -> bool {
    for i in 0..4 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_6_2(
    slice_: &[u8; 6],
    search: &[u8; 2],
) -> bool {
    for i in 0..5 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_7_2(
    slice_: &[u8; 7],
    search: &[u8; 2],
) -> bool {
    for i in 0..6 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_8_2(
    slice_: &[u8; 8],
    search: &[u8; 2],
) -> bool {
    for i in 0..7 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_9_2(
    slice_: &[u8; 9],
    search: &[u8; 2],
) -> bool {
    for i in 0..8 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_10_2(
    slice_: &[u8; 10],
    search: &[u8; 2],
) -> bool {
    for i in 0..9 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_11_2(
    slice_: &[u8; 11],
    search: &[u8; 2],
) -> bool {
    for i in 0..10 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_12_2(
    slice_: &[u8; 12],
    search: &[u8; 2],
) -> bool {
    for i in 0..11 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_13_2(
    slice_: &[u8; 13],
    search: &[u8; 2],
) -> bool {
    for i in 0..12 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_14_2(
    slice_: &[u8; 14],
    search: &[u8; 2],
) -> bool {
    for i in 0..13 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_15_2(
    slice_: &[u8; 15],
    search: &[u8; 2],
) -> bool {
    for i in 0..14 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_16_2(
    slice_: &[u8; 16],
    search: &[u8; 2],
) -> bool {
    for i in 0..15 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_17_2(
    slice_: &[u8; 17],
    search: &[u8; 2],
) -> bool {
    for i in 0..16 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_18_2(
    slice_: &[u8; 18],
    search: &[u8; 2],
) -> bool {
    for i in 0..17 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_19_2(
    slice_: &[u8; 19],
    search: &[u8; 2],
) -> bool {
    for i in 0..18 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_20_2(
    slice_: &[u8; 20],
    search: &[u8; 2],
) -> bool {
    for i in 0..19 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_21_2(
    slice_: &[u8; 21],
    search: &[u8; 2],
) -> bool {
    for i in 0..20 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_22_2(
    slice_: &[u8; 22],
    search: &[u8; 2],
) -> bool {
    for i in 0..21 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_23_2(
    slice_: &[u8; 23],
    search: &[u8; 2],
) -> bool {
    for i in 0..22 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_24_2(
    slice_: &[u8; 24],
    search: &[u8; 2],
) -> bool {
    for i in 0..23 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_25_2(
    slice_: &[u8; 25],
    search: &[u8; 2],
) -> bool {
    for i in 0..24 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_26_2(
    slice_: &[u8; 26],
    search: &[u8; 2],
) -> bool {
    for i in 0..25 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_27_2(
    slice_: &[u8; 27],
    search: &[u8; 2],
) -> bool {
    for i in 0..26 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_28_2(
    slice_: &[u8; 28],
    search: &[u8; 2],
) -> bool {
    for i in 0..27 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_29_2(
    slice_: &[u8; 29],
    search: &[u8; 2],
) -> bool {
    for i in 0..28 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_30_2(
    slice_: &[u8; 30],
    search: &[u8; 2],
) -> bool {
    for i in 0..29 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_31_2(
    slice_: &[u8; 31],
    search: &[u8; 2],
) -> bool {
    for i in 0..30 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_32_2(
    slice_: &[u8; 32],
    search: &[u8; 2],
) -> bool {
    for i in 0..31 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_33_2(
    slice_: &[u8; 33],
    search: &[u8; 2],
) -> bool {
    for i in 0..32 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_34_2(
    slice_: &[u8; 34],
    search: &[u8; 2],
) -> bool {
    for i in 0..33 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_35_2(
    slice_: &[u8; 35],
    search: &[u8; 2],
) -> bool {
    for i in 0..34 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_36_2(
    slice_: &[u8; 36],
    search: &[u8; 2],
) -> bool {
    for i in 0..35 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_37_2(
    slice_: &[u8; 37],
    search: &[u8; 2],
) -> bool {
    for i in 0..36 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_38_2(
    slice_: &[u8; 38],
    search: &[u8; 2],
) -> bool {
    for i in 0..37 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_39_2(
    slice_: &[u8; 39],
    search: &[u8; 2],
) -> bool {
    for i in 0..38 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_40_2(
    slice_: &[u8; 40],
    search: &[u8; 2],
) -> bool {
    for i in 0..39 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_41_2(
    slice_: &[u8; 41],
    search: &[u8; 2],
) -> bool {
    for i in 0..40 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_42_2(
    slice_: &[u8; 42],
    search: &[u8; 2],
) -> bool {
    for i in 0..41 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_43_2(
    slice_: &[u8; 43],
    search: &[u8; 2],
) -> bool {
    for i in 0..42 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_44_2(
    slice_: &[u8; 44],
    search: &[u8; 2],
) -> bool {
    for i in 0..43 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_45_2(
    slice_: &[u8; 45],
    search: &[u8; 2],
) -> bool {
    for i in 0..44 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_46_2(
    slice_: &[u8; 46],
    search: &[u8; 2],
) -> bool {
    for i in 0..45 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_47_2(
    slice_: &[u8; 47],
    search: &[u8; 2],
) -> bool {
    for i in 0..46 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_48_2(
    slice_: &[u8; 48],
    search: &[u8; 2],
) -> bool {
    for i in 0..47 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_49_2(
    slice_: &[u8; 49],
    search: &[u8; 2],
) -> bool {
    for i in 0..48 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_50_2(
    slice_: &[u8; 50],
    search: &[u8; 2],
) -> bool {
    for i in 0..49 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_51_2(
    slice_: &[u8; 51],
    search: &[u8; 2],
) -> bool {
    for i in 0..50 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_52_2(
    slice_: &[u8; 52],
    search: &[u8; 2],
) -> bool {
    for i in 0..51 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_53_2(
    slice_: &[u8; 53],
    search: &[u8; 2],
) -> bool {
    for i in 0..52 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_54_2(
    slice_: &[u8; 54],
    search: &[u8; 2],
) -> bool {
    for i in 0..53 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_55_2(
    slice_: &[u8; 55],
    search: &[u8; 2],
) -> bool {
    for i in 0..54 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_56_2(
    slice_: &[u8; 56],
    search: &[u8; 2],
) -> bool {
    for i in 0..55 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_57_2(
    slice_: &[u8; 57],
    search: &[u8; 2],
) -> bool {
    for i in 0..56 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_58_2(
    slice_: &[u8; 58],
    search: &[u8; 2],
) -> bool {
    for i in 0..57 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_59_2(
    slice_: &[u8; 59],
    search: &[u8; 2],
) -> bool {
    for i in 0..58 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_60_2(
    slice_: &[u8; 60],
    search: &[u8; 2],
) -> bool {
    for i in 0..59 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_61_2(
    slice_: &[u8; 61],
    search: &[u8; 2],
) -> bool {
    for i in 0..60 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_62_2(
    slice_: &[u8; 62],
    search: &[u8; 2],
) -> bool {
    for i in 0..61 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_63_2(
    slice_: &[u8; 63],
    search: &[u8; 2],
) -> bool {
    for i in 0..62 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_64_2(
    slice_: &[u8; 64],
    search: &[u8; 2],
) -> bool {
    for i in 0..63 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_65_2(
    slice_: &[u8; 65],
    search: &[u8; 2],
) -> bool {
    for i in 0..64 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_66_2(
    slice_: &[u8; 66],
    search: &[u8; 2],
) -> bool {
    for i in 0..65 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_67_2(
    slice_: &[u8; 67],
    search: &[u8; 2],
) -> bool {
    for i in 0..66 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_68_2(
    slice_: &[u8; 68],
    search: &[u8; 2],
) -> bool {
    for i in 0..67 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_69_2(
    slice_: &[u8; 69],
    search: &[u8; 2],
) -> bool {
    for i in 0..68 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

/// Loop unrolled implementation of `slice.contains` for a byte slice and a
/// hardcoded array. Uses crate [`unroll`].
///
/// Hardcoded implementation for [`u8`] slices up to 69 length. Runs very fast.
/// Supports arbitrary length.
///
/// [`unroll`]: https://docs.rs/unroll/0.1.5/unroll/index.html
//
// slice index values for the `DTPD!` declarations can be reviewed with:
//
//     $ grep -Ee '^[[:space:]]+DTFSS_' -- src/data/datetime.rs  | sort -t ',' -n -k2 -k3 | column -t
//
#[inline(always)]
#[allow(non_snake_case)]
pub fn slice_contains_X_2(
    slice_: &[u8],
    search: &[u8; 2],
) -> bool {
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
        51 => slice_contains_51_2(array_ref!(slice_, 0, 51), search),
        52 => slice_contains_52_2(array_ref!(slice_, 0, 52), search),
        53 => slice_contains_53_2(array_ref!(slice_, 0, 53), search),
        54 => slice_contains_54_2(array_ref!(slice_, 0, 54), search),
        55 => slice_contains_55_2(array_ref!(slice_, 0, 55), search),
        56 => slice_contains_56_2(array_ref!(slice_, 0, 56), search),
        57 => slice_contains_57_2(array_ref!(slice_, 0, 57), search),
        58 => slice_contains_58_2(array_ref!(slice_, 0, 58), search),
        59 => slice_contains_59_2(array_ref!(slice_, 0, 59), search),
        60 => slice_contains_60_2(array_ref!(slice_, 0, 60), search),
        61 => slice_contains_61_2(array_ref!(slice_, 0, 61), search),
        62 => slice_contains_62_2(array_ref!(slice_, 0, 62), search),
        63 => slice_contains_63_2(array_ref!(slice_, 0, 63), search),
        64 => slice_contains_64_2(array_ref!(slice_, 0, 64), search),
        65 => slice_contains_65_2(array_ref!(slice_, 0, 65), search),
        66 => slice_contains_66_2(array_ref!(slice_, 0, 66), search),
        67 => slice_contains_67_2(array_ref!(slice_, 0, 67), search),
        68 => slice_contains_68_2(array_ref!(slice_, 0, 68), search),
        69 => slice_contains_69_2(array_ref!(slice_, 0, 69), search),
        _ => slice_
            .iter()
            .any(|&c| c == search[0] || c == search[1]),
    }
}
