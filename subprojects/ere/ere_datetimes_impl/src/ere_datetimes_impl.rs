// ere_datetimes_impl.rs

//! Definitions of all `ere` compile-time regular expressions.
//! Declared in it's own rust project so it might be rebuilt less often
//! during development.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::collections::{
    HashMap,
    HashSet,
};
use std::fmt;
#[doc(hidden)]
pub use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use ::const_str::concat;
#[allow(unused_imports)]
use ::si_trace_print::{
    defn,
    defo,
    defx,
    defñ,
    den,
    deo,
    dex,
    deñ,
};

use ::ere_automator_procmacro::{
    counter,
    counter_last,
};

pub use ::ere::regex;
pub use ::ere_automator_procmacro::new_ere_regex;

/// Are all regular expressions built?
pub const REGEX_ALL_COMPILED: bool = {
    if cfg!(regex = "ALL") {
        true
    } else {
        false
    }
};

// DateTime Regex Matching and strftime formatting

// XXX: duplicates Line.rs
pub type LineIndex = usize;
// XXX: duplicates Line.rs
pub type RangeLineIndex = std::ops::Range<LineIndex>;

pub type RegexId = u16;

/// A local implementation of `ere::Span`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SpanS4 {
    /// The start offset of the span, inclusive.
    pub start: usize,
    /// The end offset of the span, exclusive.
    pub end: usize,
}

impl SpanS4 {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub const fn as_range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

/// Index into the global [`GROUP_NAMES`] array.
// TODO: use nutype and constrain the range of this type to be `0..GROUP_NAMES_LEN`
pub type GroupsIndex = usize;

/// Information about a match.
/// The wrapper function that calls the `ere` regular expression will return
/// `vec<MatchType>`.
// TODO: rename this to `MatchData`
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MatchType {
    /// The start and end offsets of the match in an associated haystack.
    span: SpanS4,
    /// Index into the global [`GROUP_NAMES`] array for the capture group name
    /// of this match.
    /// Easier to store a number versus a `&str`; no lifetime declaration
    /// pollution.
    group_index: GroupsIndex,
}
pub type MatchesType = Vec<MatchType>;
/// Returned by wrapper function that calls the `ere` regular expression.
pub type MatchesTypeOpt = Option<MatchesType>;
/// The function signature of the wrapper function that calls the `ere` regular
/// expression.
pub type RegexFn = fn(&[u8]) -> MatchesTypeOpt;

impl MatchType {
    pub const fn new(span: SpanS4, group_index: GroupsIndex) -> Self {
        Self { span, group_index }
    }

    /// Index into associated haystack, start of the match, inclusive
    #[inline(always)]
    pub const fn start(&self) -> usize {
        self.span.start
    }

    /// Index into associated haystack, end of the match, exlusive
    #[inline(always)]
    pub const fn end(&self) -> usize {
        self.span.end
    }

    /// Index into associated haystack, span of the match, [...)
    #[inline(always)]
    pub const fn span(&self) -> &SpanS4 {
        &self.span
    }

    /// Index into [`CGN_ALL`], maps to match name
    #[inline(always)]
    pub const fn group_index(&self) -> GroupsIndex {
        self.group_index
    }
}

pub const CHARSZ: usize = 1;

#[macro_export]
macro_rules! SpanS4_from_ptrs {
    (
        $group_str:expr,
        $haystack:expr
    ) => {
        SpanS4::new(
            (($group_str.as_ptr() as usize - $haystack.as_ptr() as usize) / CHARSZ),
            (($group_str.as_ptr() as usize - $haystack.as_ptr() as usize + $group_str.len()) / CHARSZ),
        )
    }
}

/// An _Uptime_ in a date, e.g. from `[    1.000043] kernel: Linux starting...`,
/// the seconds parts, e.g. the `1`.
/// This the default format for dmesg-style log files.
pub type Uptime = i32;

/// Crate `chrono` [`strftime`] formatting pattern, passed to
/// chrono [`DateTime::parse_from_str`] or [`NaiveDateTime::parse_from_str`].
///
/// Specific `const` instances of `DateTimePattern_str` are hardcoded in
/// [`captures_to_buffer_bytes`].
///
/// [`strftime`]: https://docs.rs/chrono/0.4.38/chrono/format/strftime/index.html
/// [`DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/struct.DateTime.html#method.parse_from_str
/// [`NaiveDateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/naive/struct.NaiveDateTime.html#method.parse_from_str
pub type DateTimePattern_str = str;

/// Analogous to [`DateTimePattern_str`], but for `String` instances.
pub type DateTimePattern_string = String;

/// Regular expression formatting pattern, passed to [`regex::bytes::Regex`].
///
/// [`regex::bytes::Regex`]: https://docs.rs/regex/1.10.5/regex/bytes/struct.Regex.html
pub type DateTimeRegex_str = str;

/// Regular expression capture group name, used within the regular expression
/// and for later retrieval via [`regex::captures.name`].
///
/// [`regex::captures.name`]: https://docs.rs/regex/1.10.5/regex/bytes/struct.Captures.html#method.name
pub type CaptureGroupName = str;

/// Regular expression capture group pattern, used within a [`RegexPattern`].
pub type CaptureGroupPattern = str;

/// A regular expression, passed to [`regex::bytes::Regex::captures`].
///
/// [`regex::bytes::Regex::captures`]: https://docs.rs/regex/1.10.5/regex/bytes/struct.Regex.html#method.captures
pub type RegexPattern = str;

pub type HaystackType = [u8];

/// FixedOffset seconds
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub type fos = i32;

#[allow(dead_code)]
pub const YEAR_FALLBACKDUMMY_VAL: i32 = 1972;
/// For datetimes missing a year, in some circumstances a filler year must be
/// used.
///
/// First leap year after Unix Epoch.
///
/// XXX: using leap year as a filler might help handle 'Feb 29' dates without a
///      year but it is not guaranteed. It depends on the file modified time
///      (i.e. [`blockreader.mtime()`](BlockReader)) being true.
#[allow(dead_code)]
pub const YEAR_FALLBACKDUMMY: &str = "1972";

/// abbreviate *y*ear *d*ummy
#[allow(dead_code)]
pub const YD: i32 = YEAR_FALLBACKDUMMY_VAL;

// Offset UTC/Z
#[allow(dead_code)]
pub const O_Z: fos = 0;
#[allow(dead_code)]
pub const O_0: fos = 0;
// Offset Local
// symbolic value for Local time, replaced at runtime
#[allow(dead_code)]
pub const O_L: fos = i32::MAX;
// Offset Minus (or West)
#[allow(dead_code)]
pub const O_M1: fos = -3600;
#[allow(dead_code)]
pub const O_M1_30: fos = -3600 - 30 * 60;
#[allow(dead_code)]
pub const O_M2: fos = -2 * 3600;
#[allow(dead_code)]
pub const O_M3: fos = -3 * 3600;
#[allow(dead_code)]
pub const O_M330: fos = -3 * 3600 - 30 * 60;
#[allow(dead_code)]
#[allow(dead_code)]
pub const O_M4: fos = -4 * 3600;
#[allow(dead_code)]
pub const O_M5: fos = -5 * 3600;
#[allow(dead_code)]
#[allow(dead_code)]
pub const O_M6: fos = -6 * 3600;
#[allow(dead_code)]
pub const O_M7: fos = -7 * 3600;
#[allow(dead_code)]
pub const O_M730: fos = -7 * 3600 - 30 * 60;
#[allow(dead_code)]
pub const O_M8: fos = -8 * 3600;
#[allow(dead_code)]
pub const O_M9: fos = -9 * 3600;
#[allow(dead_code)]
pub const O_M10: fos = -10 * 3600;
#[allow(dead_code)]
pub const O_M1030: fos = -10 * 3600 - 30 * 60;
#[allow(dead_code)]
pub const O_M11: fos = -11 * 3600;
#[allow(dead_code)]
pub const O_M1130: fos = -11 * 3600 - 30 * 60;
#[allow(dead_code)]
#[allow(dead_code)]
pub const O_M12: fos = -12 * 3600;
#[allow(dead_code)]
#[allow(dead_code)]
pub const O_M1230: fos = -12 * 3600 - 30 * 60;
// Offset Plus (or East)
#[allow(dead_code)]
pub const O_P1: fos = 3600;
#[allow(dead_code)]
pub const O_P1_30: fos = 3600 + 30 * 60;
#[allow(dead_code)]
#[allow(dead_code)]
pub const O_P2: fos = 2 * 3600;
#[allow(dead_code)]
pub const O_P3: fos = 3 * 3600;
#[allow(dead_code)]
#[allow(dead_code)]
pub const O_P4: fos = 4 * 3600;
#[allow(dead_code)]
pub const O_P5: fos = 5 * 3600;
#[allow(dead_code)]
pub const O_P6: fos = 6 * 3600;
#[allow(dead_code)]
#[allow(dead_code)]
pub const O_P7: fos = 7 * 3600;
#[allow(dead_code)]
pub const O_P8: fos = 8 * 3600;
#[allow(dead_code)]
pub const O_P9: fos = 9 * 3600;
#[allow(dead_code)]
pub const O_P945: fos = 9 * 3600 + 45 * 60;
#[allow(dead_code)]
pub const O_P10: fos = 10 * 3600;
#[allow(dead_code)]
pub const O_P11: fos = 11 * 3600;
#[allow(dead_code)]
pub const O_P12: fos = 12 * 3600;
#[allow(dead_code)]
pub const O_P1230: fos = 12 * 3600 + 30 * 60;
// misc. named timezones
#[allow(dead_code)]
pub const O_ALMT: fos = O_P6;
#[allow(dead_code)]
pub const O_CHUT: fos = O_P10;
#[allow(dead_code)]
pub const O_CIST: fos = O_M8;
#[allow(dead_code)]
pub const O_EDT: fos = O_M4;
#[allow(dead_code)]
pub const O_CAT: fos = O_P2;
#[allow(dead_code)]
pub const O_PDT: fos = O_M7;
#[allow(dead_code)]
pub const O_PETT: fos = O_P12;
#[allow(dead_code)]
pub const O_PET: fos = O_M5;
#[allow(dead_code)]
pub const O_PONT: fos = O_P11;
#[allow(dead_code)]
pub const O_PST: fos = O_M8;
#[allow(dead_code)]
pub const O_PWT: fos = O_P9;
#[allow(dead_code)]
pub const O_VLAT: fos = O_P10;
#[allow(dead_code)]
pub const O_WAT: fos = O_P1;
#[allow(dead_code)]
pub const O_WITA: fos = O_P8;
#[allow(dead_code)]
pub const O_WIT: fos = O_P9;
#[allow(dead_code)]
pub const O_WGST: fos = O_M2;
#[allow(dead_code)]
pub const O_WST: fos = O_P8;
#[allow(dead_code)]
pub const O_YAKT: fos = O_P9;
#[allow(dead_code)]
pub const O_YEKT: fos = O_P5;

/// tuple of arguments for function `ymdhmsn`
#[allow(dead_code)]
pub type ymdhmsn_args = (
    // fixedoffset
    fos,
    // year
    i32,
    // month
    u32,
    // day
    u32,
    // hour
    u32,
    // minute
    u32,
    // second
    u32,
    // nanosecond
    i64,
);

#[allow(dead_code)]
pub const DUMMY_ARGS: ymdhmsn_args = (0, 1972, 1, 1, 0, 0, 0, 123456789);

/*
selective copy of chrono `strftime` specifier reference table
copied from https://docs.rs/chrono/0.4.38/chrono/format/strftime/index.html

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
//       refactor to remove intermediary `DTP_*` variables
//       allow more flexible regex grouping and name declarations.

/// DateTime Format Specifier for a Year.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Year {
    /// %Y, four-digit year
    Y,
    /// %y, two-digit year
    y,
    /// none provided, must be filled.
    /// the associated `pattern` should use "%Y`
    _fill,
    /// no year
    _none,
}

/// DateTime Format Specifier for a Month.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Month {
    /// %m, month numbers 00 to 12, double-digit
    m,
    /// %m, month numbers 0 to 12, single-digit
    ///
    /// Transformed to form `%m` in function `captures_to_buffer_bytes`.
    ms,
    /// %b, month abbreviated to three characters, e.g. `"Jan"`.
    b,
    /// %B, month full name, e.g. `"January"`
    ///
    /// Transformed to form `%b` in
    /// function `month_bB_to_month_m_bytes` called by
    /// function `captures_to_buffer_bytes`
    B,
    /// no month
    _none,
}

/// DateTime Format Specifier for a Day.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Day {
    /// `%d`, day number 01 to 31
    /// `%e`, day number 1 to 31
    ///
    /// Single-digit `" 8"` or `"8"` is transformed to `"08"` in
    /// function `captures_to_buffer_bytes`.
    _e_or_d,
    /// no day
    _none,
}

/// DateTime Format Specifier for an Hour.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Hour {
    /// %H, 24 hour, 00 to 23, double-digit
    H,
    /// %k, 24 hour, 0 to 23, single-digit
    k,
    /// %I, 12 hour, 01 to 12, double-digit
    I,
    /// %l, 12 hour, 1 to 12, single-digit
    l,
    /// no hour
    _none,
}

/// DateTime Format Specifier for a Minute.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Minute {
    /// %M, 00 to 59, double-digit
    M,
    /// %M, 0 to 59, single-digit
    m,
    /// no minute
    _none,
}

/// DateTime Format Specifier for a Second.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Second {
    /// %S, 00 to 60, double-digit
    S,
    /// %s, 0 to 60, single-digit
    s,
    /// fill with value `0`
    _fill,
    /// no second
    _none,
}

/// DateTime Format Specifier for a Fractional or fractional second.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Fractional {
    /// %f, subsecond decimal digits
    f,
    /// no fractional
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
    /// The associated `pattern` should use `%:z` (variable name substring `Zc`)
    /// as that is the form displayed by
    /// `chrono::FixedOffset::east(0).as_string().to_str()`
    _fill,
    /// no timezone
    _none,
}

/// DateTime Format Specifier for a Unix Epoch.
/// Follows chrono `strftime` specifier formatting.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Epoch {
    /// Unix Epoch in milliseconds, seconds passed in `%s`
    ms,
    /// `%s` Unix Epoch in seconds
    s,
    /// none
    _none,
}

/// DateTime Format Specifier for an uptime, seconds since system boot.
/// e.g. from log message `[    0.001000] kernel: Linux starting...`.
/// This the default format for dmesg-style log files.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Uptime {
    /// system uptime in seconds
    u,
    /// none
    _none,
}

/// `DTFSSet`, "DateTime Format Specifer Set", is essentially instructions
/// to transcribe regular expression [`named capture groups`] into a
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
/// fn main() {
///     let data = r"[2020/Mar/05 12:17:59.631000 PMDT] ../source3/smbd/oplock.c:1340(init_oplocks)";
///     let pattern = r"^\[(?<year>[12][0-9]{3})[ /\-]?(?<month>(?i)01|02|03|04|05|06|07|08|09|10|11|12|jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec(?-i))[ /\-]?(?<day>01|02|03|04|05|06|07|08|09|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31)[ T]?(?<hour>00|01|02|03|04|05|06|07|08|09|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24)[:]?(?<minute>[012345][0-9])[:]?(?<second>[0123456][0-9])[\.,](?<subsecond>[0-9]{3,9})[[:blank:]](?<tz>ACDT|ACST|ACT|ADT|AEDT|AEST|AET|AFT|AKDT|AKST|ALMT|AMST|AMT|ANAT|AQTT|ART|AST|AWST|AZOT|AZT|BIOT|BIT|BNT|BOT|BRST|BRT|BST|BTT|CAT|CCT|CDT|CEST|CET|CHADT|CHOT|CHST|CHUT|CIST|CKT|CLST|CLT|COST|COT|CST|CT|CVT|CWST|CXT|DAVT|DDUT|DFT|EAST|EAT|ECT|EDT|EEST|EET|EGST|EGT|EST|ET|FET|FJT|FKST|FKT|FNT|GALT|GAMT|GET|GFT|GILT|GIT|GMT|GST|GYT|HAEC|HDT|HKT|HMT|HOVT|HST|ICT|IDLW|IDT|IOT|IRDT|IRKT|IRST|IST|JST|KALT|KGT|KOST|KRAT|KST|LHST|LINT|MAGT|MART|MAWT|MDT|MEST|MET|MHT|MIST|MIT|MMT|MSK|MST|MUT|MVT|MYT|NCT|NDT|NFT|NOVT|NPT|NST|NT|NUT|NZDT|NZST|OMST|ORAT|PDT|PETT|PET|PGT|PHOT|PHST|PHT|PKT|PMDT|PMST|PONT|PST|PWT|PYST|PYT|RET|ROTT|SAKT|SAMT|SAST|SBT|SCT|SDT|SGT|SLST|SRET|SRT|SST|SYOT|TAHT|TFT|THA|TJT|TKT|TLT|TMT|TOT|TRT|TVT|ULAT|UTC|UT|UYST|UYT|UZT|VET|VLAT|VOLT|VOST|VUT|WAKT|WAST|WAT|WEST|WET|WGST|WGT|WIB|WITA|WIT|WST|YAKT|YEKT)[^[[:upper:]]]";
/// }
/// ```
/// [(Rust Playground)],
///
/// should become:
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
/// [`named capture groups`]: https://docs.rs/regex/1.10.5/regex/bytes/struct.Captures.html
/// [`chrono::DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/struct.DateTime.html#method.parse_from_str
/// [`DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/struct.DateTime.html#method.parse_from_str
/// [`NaiveDateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/naive/struct.NaiveDateTime.html#method.parse_from_str
/// [`strftime`]: https://docs.rs/chrono/0.4.38/chrono/format/strftime/index.html
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
    pub epoch: DTFS_Epoch,
    pub uptime: DTFS_Uptime,
    /// strftime pattern passed to [`chrono::DateTime::parse_from_str`] or
    /// [`chrono::NaiveDateTime::parse_from_str`]
    /// in function [`datetime_parse_from_str`]. Directly relates to order of
    /// capture group extractions and `push_str` done in private
    /// `captures_to_buffer_bytes`.
    ///
    /// `pattern` is interdependent with other members.
    ///
    /// Tested in test `test_DATETIME_PARSE_DATAS_builtin`.
    ///
    /// [`chrono::DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/struct.DateTime.html#method.parse_from_str
    /// [`chrono::NaiveDateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/naive/struct.NaiveDate.html#method.parse_from_str
    pub pattern: &'a DateTimePattern_str,
}

impl DTFSSet<'_> {

    /// does the `DTFSSet` expect a year?
    pub const fn has_year(&self) -> bool {
        match self.year {
            DTFS_Year::Y
            | DTFS_Year::y => true,
            DTFS_Year::_fill
            | DTFS_Year::_none => false,
        }
    }

    /// does the `DTFSSet` expect a 4-digit year?
    pub const fn has_year4(&self) -> bool {
        match self.year {
            DTFS_Year::Y => true,
            DTFS_Year::y
            | DTFS_Year::_fill
            | DTFS_Year::_none
            => false,
        }
    }

    /// does the `DTFSSet` expect a month?
    pub const fn has_month(&self) -> bool {
        match self.month {
            DTFS_Month::m
            | DTFS_Month::ms
            | DTFS_Month::b
            | DTFS_Month::B => true,
            DTFS_Month::_none => false,
        }
    }

    /// does the `DTFSSet` expect a day?
    pub const fn has_day(&self) -> bool {
        match self.day {
            DTFS_Day::_e_or_d => true,
            DTFS_Day::_none => false,
        }
    }

    /// does the `DTFSSet` expect a hour?
    pub const fn has_hour(&self) -> bool {
        match self.hour {
            DTFS_Hour::H
            | DTFS_Hour::k
            | DTFS_Hour::I
            | DTFS_Hour::l => true,
            DTFS_Hour::_none => false,
        }
    }

    /// does the `DTFSSet` expect a minute?
    pub const fn has_minute(&self) -> bool {
        match self.minute {
            DTFS_Minute::M
            | DTFS_Minute::m => true,
            DTFS_Minute::_none => false,
        }
    }

    /// does the `DTFSSet` expect a timezone?
    pub const fn has_tz(&self) -> bool {
        match self.tz {
            DTFS_Tz::z
            | DTFS_Tz::zc
            | DTFS_Tz::zp
            | DTFS_Tz::Z => true,
            DTFS_Tz::_fill
            | DTFS_Tz::_none => false,
        }
    }

    /// does the `DTFSSet` expect an uptime?
    pub const fn has_uptime(&self) -> bool {
        match self.uptime {
            DTFS_Uptime::u => true,
            DTFS_Uptime::_none => false,
        }
    }

    /// does the `DTFSSet` expect to capture a sequence of two decimal digits?
    pub const fn has_d2(&self) -> bool {
        match self.year {
            DTFS_Year::Y => return true,
            DTFS_Year::y
            | DTFS_Year::_fill
            | DTFS_Year::_none => {}
        }
        match self.month {
            DTFS_Month::m => return true,
            DTFS_Month::ms
            | DTFS_Month::b
            | DTFS_Month::B
            | DTFS_Month::_none => {}
        }
        match self.hour {
            DTFS_Hour::H
            | DTFS_Hour::I => return true,
            DTFS_Hour::k
            | DTFS_Hour::l
            | DTFS_Hour::_none => {}
        }
        match self.minute {
            DTFS_Minute::M
            | DTFS_Minute::m => return true,
            DTFS_Minute::_none => {}
        }
        match self.second {
            DTFS_Second::S
            | DTFS_Second::s => return true,
            DTFS_Second::_fill
            | DTFS_Second::_none => {}
        }
        match self.tz {
            DTFS_Tz::z
            | DTFS_Tz::zc
            | DTFS_Tz::zp => return true,
            DTFS_Tz::Z
            | DTFS_Tz::_fill
            | DTFS_Tz::_none => {}
        }
        match self.epoch {
            DTFS_Epoch::ms
            | DTFS_Epoch::s => return true,
            DTFS_Epoch::_none => {}
        }
        match self.uptime {
            DTFS_Uptime::u => return true,
            DTFS_Uptime::_none => {}
        }

        false
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
            .field("year?", &self.has_year())
            .field("uptime?", &self.has_uptime())
            .field("tz?", &self.has_tz())
            .field("pattern", &self.pattern)
            ;

        f_.finish()
    }
}

// `strftime` patterns used in `DTFSSet!` declarations

// TODO: [2023/01] replace name `DTPD` with `DTPI`
// TODO: [2026/04] pass a tuple for `$sib, $sie` and `$cgn_first, $cgn_last`
//       how to do this in a macro? the point is the call site should be required
//       to pass a tuple.
// TODO: [2022/10/08] refactor for consistent naming of  `DTP_*` variables:
//       put 'Y' in front, so it matches
//       strftime specifier ordering within the value.
//       e.g one is named `DTP_YmdHMSz` and starts with `%Y`, another is named
//       `DTP_mdHMYZc` and also starts with `%Y`.
//       e.g. variable `DTP_BdHMSYz` has value `"%Y%m%dT%H%M%S%z"`, the `%Y`
//       is in front, so the variable should match the ordering, `DTP_YBdHMSz`.
//       a few less human brain cycles to grok the var.
// TODO: [2022/10/10] refactor for consistent naming of timezone in variables
//       names. Sometimes it is `DTP_YmdHMSzc` (notice `zc`) but then there
//       is `DTP_bdHMSYZc` (noticed `Zc`).

pub const DTP_YmdHMSzc: &DateTimePattern_str  = "%Y%m%dT%H%M%S%:z";
pub const DTP_YmdHMSz: &DateTimePattern_str   = "%Y%m%dT%H%M%S%z";
pub const DTP_YmdHMSzp: &DateTimePattern_str  = "%Y%m%dT%H%M%S%#z";
pub const DTP_YmdHMSfzc: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%:z";
pub const DTP_YmdHMSfz: &DateTimePattern_str  = "%Y%m%dT%H%M%S.%f%z";
pub const DTP_YmdHMSfzp: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%#z";
/// no second, chrono will set to value 0
pub const DTP_mdHMYZc: &DateTimePattern_str = "%Y%m%dT%H%M%:z";
/// `%Y` `%:z` is filled, `%B` value transformed to `%m` value by [`captures_to_buffer_bytes`]
pub const DTP_BdHMS: &DateTimePattern_str = "%m%dT%H%M%S%:z";
/// `%b` value transformed to `%m` value by [`captures_to_buffer_bytes`]
pub const DTP_bdHMSyZc: &DateTimePattern_str = "%y%m%dT%H%M%S%:z";
/// `%s` value
pub const DTP_s: &DateTimePattern_str = "%sT";
/// `%s.%f` value
pub const DTP_sf: &DateTimePattern_str = "%sT.%f";
/// `%s.%f` value alternate
pub const DTP_s3fT: &DateTimePattern_str = "%s.%3fT";

/// For testing
///
/// check `DTP_ALL` has all `DTP_` vars
///
///     grep -Fe ' DTP_' subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs | grep -Fe 'const ' | grep -Eve '^[[:space:]]*//' | grep -oEe 'DTP_[[:alnum:]]+' | grep -Fve 'DTP_ALL' | sed 's/$/,/' | sort | uniq
///
#[doc(hidden)]
#[allow(dead_code)]
pub const DTP_ALL: &[&DateTimePattern_str] = &[
    DTP_bdHMSyZc,
    DTP_mdHMYZc,
    DTP_s,
    DTP_sf,
    DTP_s3fT,
    DTP_YmdHMSfz,
    DTP_YmdHMSfzc,
    DTP_YmdHMSfzp,
    DTP_YmdHMSz,
    DTP_YmdHMSzc,
    DTP_YmdHMSzp,
];

// The variable name represents what is available. The value represents it's
// rearranged form using in function `captures_to_buffer_bytes`.

// TODO: rename these `DTFSSet` are named inconsistently.
//       some are `DTFSS__YmdHMS` and others use `DTFSS_BdHMSY`.
//       The lettering in the name is not consistent. Notice where the `Y` is
//       placed. Other parts are inconsistent, too.
//       Name these consistently.

pub const DTFSS_YmdHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
// single-digit month, single-digit hour
pub const DTFSS_YmsdkMS: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::ms,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::k,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
pub const DTFSS_YmdHMSz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::z,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSz,
};
pub const DTFSS_YmdHMSzc: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zc,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
pub const DTFSS_YmdHMSzp: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zp,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzp,
};
pub const DTFSS_YmdHMSZ: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::Z,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSz,
};

pub const DTFSS_YmdHMSf: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSfzc,
};
pub const DTFSS_YmdHMSfz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::z,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSfz,
};
pub const DTFSS_YmdHMSfzc: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::zc,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSfzc,
};
pub const DTFSS_YmdHMSfzp: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::zp,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSfzp,
};
pub const DTFSS_YmdHMSfZ: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::Z,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSfzc,
};

pub const DTFSS_Ymdkms: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::ms,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::k,
    minute: DTFS_Minute::m,
    second: DTFS_Second::s,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
pub const DTFSS_Ymdkmsf: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::ms,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::k,
    minute: DTFS_Minute::m,
    second: DTFS_Second::s,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSfzc,
};

pub const DTFSS_mdHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::_fill,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
pub const DTFSS_mdHMSf: DTFSSet = DTFSSet {
    year: DTFS_Year::_fill,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSfzc,
};

pub const DTFSS_BdHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::_fill,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
pub const DTFSS_BdHMSZ: DTFSSet = DTFSSet {
    year: DTFS_Year::_fill,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::Z,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
pub const DTFSS_BdHMSY: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
pub const DTFSS_BdHMSYZ: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::Z,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
pub const DTFSS_BdHMSYz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::z,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
pub const DTFSS_BdHMSYzc: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zc,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
pub const DTFSS_BdHMSYzp: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::B,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zp,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzp,
};

pub const DTFSS_bdHMSYf: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSfzc,
};
pub const DTFSS_bdHMSYfz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::z,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSfz,
};
pub const DTFSS_bdHMSYfzc: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::zc,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSfzc,
};
pub const DTFSS_bdHMSYfzp: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::zp,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSfzp,
};

pub const DTFSS_YbdHMSzc: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zc,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
pub const DTFSS_YbdHMSzp: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::zp,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzp,
};
pub const DTFSS_YbdHMSz: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::z,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSz,
};
pub const DTFSS_YbdHMSZ: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::Z,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};
pub const DTFSS_YbdHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_YmdHMSzc,
};

pub const DTFSS_ybdHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::y,
    month: DTFS_Month::b,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_bdHMSyZc,
};
pub const DTFSS_ymdHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_bdHMSyZc,
};

pub const DTFSS_YmdHM: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::_e_or_d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::_none,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_mdHMYZc,
};

pub const DTFSS_s: DTFSSet = DTFSSet {
    year: DTFS_Year::_none,
    month: DTFS_Month::_none,
    day: DTFS_Day::_none,
    hour: DTFS_Hour::_none,
    minute: DTFS_Minute::_none,
    second: DTFS_Second::_none,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_none,
    epoch: DTFS_Epoch::s,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_s,
};
pub const DTFSS_ms: DTFSSet = DTFSSet {
    year: DTFS_Year::_none,
    month: DTFS_Month::_none,
    day: DTFS_Day::_none,
    hour: DTFS_Hour::_none,
    minute: DTFS_Minute::_none,
    second: DTFS_Second::_none,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_none,
    epoch: DTFS_Epoch::ms,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_s3fT,
};

/// for epoch syslog lines
pub const DTFSS_sf: DTFSSet = DTFSSet {
    year: DTFS_Year::_none,
    month: DTFS_Month::_none,
    day: DTFS_Day::_none,
    hour: DTFS_Hour::_none,
    minute: DTFS_Minute::_none,
    second: DTFS_Second::_none,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::_none,
    epoch: DTFS_Epoch::s,
    uptime: DTFS_Uptime::_none,
    pattern: DTP_sf,
};

/// for `dmesg` syslog lines, e.g.
/// `[    0.4074] kernel: Linux version 5.15.0-47-generic (buildd@lcy02-amd64-060)`
pub const DTFSS_u: DTFSSet = DTFSSet {
    year: DTFS_Year::_none,
    month: DTFS_Month::_none,
    day: DTFS_Day::_none,
    hour: DTFS_Hour::_none,
    minute: DTFS_Minute::_none,
    second: DTFS_Second::_none,
    fractional: DTFS_Fractional::f,
    tz: DTFS_Tz::_none,
    epoch: DTFS_Epoch::_none,
    uptime: DTFS_Uptime::u,
    pattern: DTP_sf,
};

/// For testing
///
/// check `DTFSS_ALL` has all `DTFSS_` vars
///
///     grep -Fe ' DTFSS_' subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs | grep -Fe 'const ' | grep -Eve '^[[:space:]]*//' | grep -oEe 'DTFSS_[[:alnum:]]+' | grep -Fve 'DTFSS_ALL' | sed 's/$/,/' | sort | uniq | sed -Ee 's/(.*),/(\&\1, stringify!(\1)),/g'
///
#[doc(hidden)]
#[allow(dead_code)]
// TODO: all data that might be analsyzed by re should be called a `haystack` and typed as a `haystack`
pub const DTFSS_ALL: &[(&DTFSSet, &str)] = &[
    (&DTFSS_BdHMS, stringify!(DTFSS_BdHMS)),
    (&DTFSS_BdHMSY, stringify!(DTFSS_BdHMSY)),
    (&DTFSS_bdHMSYf, stringify!(DTFSS_bdHMSYf)),
    (&DTFSS_bdHMSYfz, stringify!(DTFSS_bdHMSYfz)),
    (&DTFSS_bdHMSYfzc, stringify!(DTFSS_bdHMSYfzc)),
    (&DTFSS_bdHMSYfzp, stringify!(DTFSS_bdHMSYfzp)),
    (&DTFSS_BdHMSYz, stringify!(DTFSS_BdHMSYz)),
    (&DTFSS_BdHMSYZ, stringify!(DTFSS_BdHMSYZ)),
    (&DTFSS_BdHMSYzc, stringify!(DTFSS_BdHMSYzc)),
    (&DTFSS_BdHMSYzp, stringify!(DTFSS_BdHMSYzp)),
    (&DTFSS_BdHMSZ, stringify!(DTFSS_BdHMSZ)),
    (&DTFSS_mdHMS, stringify!(DTFSS_mdHMS)),
    (&DTFSS_mdHMSf, stringify!(DTFSS_mdHMSf)),
    (&DTFSS_ms, stringify!(DTFSS_ms)),
    (&DTFSS_s, stringify!(DTFSS_s)),
    (&DTFSS_sf, stringify!(DTFSS_sf)),
    (&DTFSS_u, stringify!(DTFSS_u)),
    (&DTFSS_ybdHMS, stringify!(DTFSS_ybdHMS)),
    (&DTFSS_YbdHMS, stringify!(DTFSS_YbdHMS)),
    (&DTFSS_YbdHMSz, stringify!(DTFSS_YbdHMSz)),
    (&DTFSS_YbdHMSZ, stringify!(DTFSS_YbdHMSZ)),
    (&DTFSS_YbdHMSzc, stringify!(DTFSS_YbdHMSzc)),
    (&DTFSS_YbdHMSzp, stringify!(DTFSS_YbdHMSzp)),
    (&DTFSS_YmdHM, stringify!(DTFSS_YmdHM)),
    (&DTFSS_ymdHMS, stringify!(DTFSS_ymdHMS)),
    (&DTFSS_YmdHMS, stringify!(DTFSS_YmdHMS)),
    (&DTFSS_YmdHMSf, stringify!(DTFSS_YmdHMSf)),
    (&DTFSS_YmdHMSfz, stringify!(DTFSS_YmdHMSfz)),
    (&DTFSS_YmdHMSfZ, stringify!(DTFSS_YmdHMSfZ)),
    (&DTFSS_YmdHMSfzc, stringify!(DTFSS_YmdHMSfzc)),
    (&DTFSS_YmdHMSfzp, stringify!(DTFSS_YmdHMSfzp)),
    (&DTFSS_YmdHMSz, stringify!(DTFSS_YmdHMSz)),
    (&DTFSS_YmdHMSZ, stringify!(DTFSS_YmdHMSZ)),
    (&DTFSS_YmdHMSzc, stringify!(DTFSS_YmdHMSzc)),
    (&DTFSS_YmdHMSzp, stringify!(DTFSS_YmdHMSzp)),
    (&DTFSS_Ymdkms, stringify!(DTFSS_Ymdkms)),
    (&DTFSS_Ymdkmsf, stringify!(DTFSS_Ymdkmsf)),
    (&DTFSS_YmsdkMS, stringify!(DTFSS_YmsdkMS)),
];

// regular expression capture group names

/// corresponds to `strftime` specifier `%Y`
pub const CGN_YEAR: &CaptureGroupName = "year";
/// corresponds to `strftime` specifier `%m`
pub const CGN_MONTH: &CaptureGroupName = "month";
/// corresponds to `strftime` specifier `%d`
pub const CGN_DAY: &CaptureGroupName = "day";
/// corresponds to `strftime` specifier `%a`
pub const CGN_DAY_IGNORE: &CaptureGroupName = "day_ignore";
/// corresponds to `strftime` specifier `%H`
pub const CGN_HOUR: &CaptureGroupName = "hour";
/// corresponds to `strftime` specifier `%M`
pub const CGN_MINUTE: &CaptureGroupName = "minute";
/// corresponds to `strftime` specifier `%S`
pub const CGN_SECOND: &CaptureGroupName = "second";
/// corresponds to `strftime` specifier `%f`
pub const CGN_FRACTIONAL: &CaptureGroupName = "fractional";
/// corresponds to `strftime` specifier `%Z`, `%z`, `%:z`, `%#z`
pub const CGN_TZ: &CaptureGroupName = "tz";
/// special case: Unix epoch seconds
pub const CGN_EPOCH: &CaptureGroupName = "epoch";
/// special case: `dmesg` uptime
pub const CGN_UPTIME: &CaptureGroupName = "uptime";

/// all capture group names.
/// used where an index `usize` must be mapped to a name `&str`
pub const CGN_ALL: [&CaptureGroupName; 11] = [
    CGN_YEAR,
    CGN_MONTH,
    CGN_DAY,
    CGN_DAY_IGNORE,
    CGN_HOUR,
    CGN_MINUTE,
    CGN_SECOND,
    CGN_FRACTIONAL,
    CGN_TZ,
    CGN_UPTIME,
    CGN_EPOCH,
];

// capture group index into `CGN_ALL` array

pub type CaptureGroupIndex = usize;
pub const CGI_YEAR: CaptureGroupIndex = 0;
pub const CGI_MONTH: CaptureGroupIndex = 1;
pub const CGI_DAY: CaptureGroupIndex = 2;
pub const CGI_DAY_IGNORE: CaptureGroupIndex = 3;
pub const CGI_HOUR: CaptureGroupIndex = 4;
pub const CGI_MINUTE: CaptureGroupIndex = 5;
pub const CGI_SECOND: CaptureGroupIndex = 6;
pub const CGI_FRACTIONAL: CaptureGroupIndex = 7;
pub const CGI_TZ: CaptureGroupIndex = 8;
pub const CGI_UPTIME: CaptureGroupIndex = 9;
pub const CGI_EPOCH: CaptureGroupIndex = 10;

// Names used in the upcoming capture group pattern variable values (`CGP_*`)
// *MUST* match the values of previous capture group name values (`CGN_*`).

/// Regex capture group pattern for `strftime` year specifier `%Y`, as
/// four decimal number characters.
pub const CGP_YEAR: &CaptureGroupPattern = r"(?<year>1969|19[789][[:digit:]]|20[[:digit:]]{2})";
/// Regex capture group pattern for `strftime` year specifier `%y`, as
/// two decimal number characters.
pub const CGP_YEARy: &CaptureGroupPattern = r"(?<year>[[:digit:]]{2})";
/// Regex capture group pattern for `strftime` month specifier `%m`, but using
/// single digit month numbers `"1"` to `"12"`.
pub const CGP_MONTHms: &CaptureGroupPattern = r"(?<month>[[:digit:]]|1[012])";
/// Regex capture group pattern for `strftime` month specifier `%m`,
/// month numbers `"01"` to `"12"`.
pub const CGP_MONTHm: &CaptureGroupPattern = r"(?<month>0[[:digit:]]|1[012])";
/// Regex capture group pattern for `strftime` month specifier `%m`,
/// single-digit or double-digit
pub const CGP_MONTHm_sd: &CaptureGroupPattern = r"(?<month>0[[:digit:]]|1[012]|[[:digit:]])";
/// Regex capture group pattern for `strftime` month specifier `%b`,
/// month name abbreviated to three characters, e.g. `Jan`.
pub const CGP_MONTHb: &CaptureGroupPattern = r"(?<month>(jan|Jan|JAN|feb|Feb|FEB|mar|Mar|MAR|apr|Apr|APR|may|May|MAY|jun|Jun|JUN|jul|Jul|JUL|aug|Aug|AUG|sep|Sep|SEP|oct|Oct|OCT|nov|Nov|NOV|dec|Dec|DEC)[\.]?)";
/// Regex capture group pattern for `strftime` month specifier `%B`,
/// month name long, e.g. `January`.
pub const CGP_MONTHB: &CaptureGroupPattern = r"(?<month>january|January|JANUARY|february|February|FEBRUARY|march|March|MARCH|april|April|APRIL|may|May|MAY|june|June|JUNE|july|July|JULY|august|August|AUGUST|september|September|SEPTEMBER|october|October|OCTOBER|november|November|NOVEMBER|december|December|DECEMBER)";
/// Regex capture group pattern for `strftime` month specifier `%B` and `%b`,
/// e.g. `January` or `Jan`.
pub const CGP_MONTHBb: &CaptureGroupPattern = r"(?<month>january|January|JANUARY|jan[\.]?|Jan[\.]?|JAN[\.]?|february|February|FEBRUARY|feb[\.]?|Feb[\.]?|FEB[\.]?|march|March|MARCH|mar[\.]?|Mar[\.]?|MAR[\.]?|april|April|APRIL|apr[\.]?|Apr[\.]?|APR[\.]?|may|May|MAY|june|June|JUNE|jun[\.]?|Jun[\.]?|JUN[\.]?|july|July|JULY|jul[\.]?|Jul[\.]?|JUL[\.]?|august|August|AUGUST|aug[\.]?|Aug[\.]?|AUG[\.]?|september|September|SEPTEMBER|sep[\.]?|Sep[\.]?|SEP[\.]?|october|October|OCTOBER|oct[\.]?|Oct[\.]?|OCT[\.]?|november|November|NOVEMBER|nov[\.]?|Nov[\.]?|NOV[\.]?|december|December|DECEMBER|dec[\.]?|Dec[\.]?|DEC[\.]?)";
/// Regex capture group pattern for `strftime` day specifier `%d`,
/// number day of month with leading zero, e.g. `"02"` or `"31"`.
/// Regex capture group pattern for `strftime` day specifier `%e`,
/// number day of month, 1 to 31, e.g. `"2"` or `"31"`.
/// Transformed to equivalent `%d` form within function
/// `captures_to_buffer_bytes` (i.e. `'0'` is prepended if necessary).
/// single-digit or double-digit
pub const CGP_DAYde: &CaptureGroupPattern = r"(?<day>[012][[:digit:]]|3[01]| [[:digit:]]|[[:digit:]])";
/// Regex capture group pattern for `strftime` day specifier `%d`,
/// number day of month with leading zero, e.g. `"02"` or `"31"`.
/// double-digit
pub const CGP_DAYd: &CaptureGroupPattern = r"(?<day>[012][[:digit:]]|3[01])";
/// Regex capture group pattern for `strftime` day specifier `%a`,
/// named day of week, either long name or abbreviated three character name,
/// e.g. `"Mon"`.
pub const CGP_DAYa3: &CaptureGroupPattern = r"(?<day_ignore>(mon|Mon|MON|tue|Tue|TUE|wed|Wed|WED|thu|Thu|THU|fri|Fri|FRI|sat|Sat|SAT|sun|Sun|SUN)[\.]?)";
/// Regex capture group pattern for `strftime` day specifier `%a`,
/// named day of week, either long name or abbreviated three character name,
/// e.g. `"Mon"` or `"Monday"`.
pub const CGP_DAYa: &CaptureGroupPattern = r"(?<day_ignore>monday|Monday|MONDAY|mon[\.]?|Mon[\.]?|MON[\.]?|tuesday|Tuesday|TUESDAY|tue[\.]?|Tue[\.]?|TUE[\.]?|wednesday|Wednesday|WEDNESDAY|wed[\.]?|Wed[\.]?|WED[\.]?|thursday|Thursday|THURSDAY|thu[\.]?|Thu[\.]?|THU[\.]?|friday|Friday|FRIDAY|fri[\.]?|Fri[\.]?|FRI[\.]?|saturday|Saturday|SATURDAY|sat[\.]?|Sat[\.]?|SAT[\.]?|sunday|Sunday|SUNDAY|sun[\.]?|Sun[\.]?|SUN[\.]?)";
/// Regex capture group pattern for `strftime` hour specifier `%H`, 00 to 24.
/// double-digit
pub const CGP_HOUR: &CaptureGroupPattern = r"(?<hour>0[[:digit:]]|1[[:digit:]]|2[0-4])";
/// Regex capture group pattern for `strftime` hour specifier `%h`, 1 to 12.
/// single-digit
pub const CGP_HOURh: &CaptureGroupPattern = r"(?<hour>1[012]|[[:digit:]])";
/// Regex capture group pattern for `strftime` hour specifier `%H`, 0 to 24.
/// single-digit or double-digit
pub const CGP_HOUR_sd: &CaptureGroupPattern = r"(?<hour>1[[:digit:]]|2[0-4]|0[[:digit:]]|[[:digit:]])";
/// Regex capture group pattern for `strftime` minute specifier `%M`, 00 to 59.
/// double-digit
pub const CGP_MINUTE: &CaptureGroupPattern = r"(?<minute>[012345][[:digit:]])";
/// Regex capture group pattern for `strftime` minute specifier `%M`, 0 to 59.
/// single-digit or double-digit
pub const CGP_MINUTE_sd: &CaptureGroupPattern = r"(?<minute>[012345][[:digit:]]|[[:digit:]])";
/// Regex capture group pattern for `strftime` second specifier `%S`, 00 to 60.
/// Includes leap second "60".
/// double-digit
pub const CGP_SECOND: &CaptureGroupPattern = r"(?<second>[012345][[:digit:]]|60)";
/// Regex capture group pattern for `strftime` second specifier `%S`, 0 to 60.
/// Includes leap second "60".
/// single-digit or double-digit
pub const CGP_SECOND_sd: &CaptureGroupPattern = r"(?<second>[012345][[:digit:]]|60|[[:digit:]])";
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
pub const CGP_FRACTIONAL19: &CaptureGroupPattern = r"(?<fractional>[[:digit:]]{1,9})";
/// Like [`CGP_FRACTIONAL19`] but only matches 2 digits, `%2f`.
pub const CGP_FRACTIONAL23: &CaptureGroupPattern = r"(?<fractional>[[:digit:]]{2,3})";
/// Like [`CGP_FRACTIONAL19`] but only matches 3 digits, `%3f`.
pub const CGP_FRACTIONAL3: &CaptureGroupPattern = r"(?<fractional>[[:digit:]][[:digit:]][[:digit:]])";
/// Like [`CGP_FRACTIONAL19`] but only matches 6 digits, `%6f`.
pub const CGP_FRACTIONAL6: &CaptureGroupPattern = r"(?<fractional>[[:digit:]]{6})";
/// Like [`CGP_FRACTIONAL19`] but only matches 9 digits, `%9f`.
pub const CGP_FRACTIONAL9: &CaptureGroupPattern = r"(?<fractional>[[:digit:]]{9})";
/// Like [`CGP_FRACTIONAL19`] but matches 3 to 9 digits, `%f`.
pub const CGP_FRACTIONAL39: &CaptureGroupPattern = r"(?<fractional>[[:digit:]]{3,9})";
/// Like [`CGP_FRACTIONAL19`] but matches 3, 6, or 9 digits, `%f`.
pub const CGP_FRACTIONAL369: &CaptureGroupPattern = r"(?<fractional>[[:digit:]]{3}|[[:digit:]]{6}|[[:digit:]]{9})";
/// Regex capture group pattern for dmesg uptime seconds.
/// 9 digits of seconds covers uptime of 11574 days,
/// e.g. `[    1.407] kernel: Linux version 5.15.0-47-generic (buildd@lcy02-amd64-060)`
/// the `1`
pub const CGP_UPTIME: &CaptureGroupPattern = r"(?<uptime>[[:digit:]]{1,9})";
/// Regex capture group pattern for dmesg uptime seconds + fractional,
/// e.g. from `[    1.407456] kernel: Linux version 5.15.0-47-generic (buildd@lcy02-amd64-060)`
/// the `1.407456`
pub const CGP_UPTIME_F: &CaptureGroupPattern = concat!(CGP_UPTIME, r"\.", CGP_FRACTIONAL39);
/// Shorter version of [`CGP_UPTIME_F`], for dmesg uptime seconds + fractional,
pub const CGP_UPTIME_F23: &CaptureGroupPattern = concat!(CGP_UPTIME, r"\.", CGP_FRACTIONAL23);
/// Regex capture group pattern for Unix epoch in seconds
/// limited to datetimes from year 2000 and afterward.
/// Low numbers are likely to be errant matches, e.g. random string "55"
/// is most likely not meant to signify epoch seconds.
///
/// Datetime _2000-01-01 00:00:00_, value `946684800`, is a reasonable past datetime limit.
///
/// Datetime _2038-01-19 03:14:06_, value `2147483647`, is a reasonable high datetime limit.
pub const CGP_EPOCH: &CaptureGroupPattern = r"(?<epoch>9[[:digit:]]{8}|[12][[:digit:]]{9})";

/// Regex capture group pattern for Unix epoch in milliseconds
/// limited to datetimes from around 2000/01/01 and afterward.
/// Low numbers are likely to be errant matches, e.g. random string "55",
/// is most likely not meant to signify epoch milliseconds.
///
/// 2000-01-01 00:00:00 is epoch milliseconds  946684800000
/// 2033-05-18 03:33:20 is epoch milliseconds 2000000000000
/// 2061-11-23 19:33:20 is epoch milliseconds 2900000000000
/// 2065-01-23 21:19:60 is epoch milliseconds 2999999999999
pub const CGP_EPOCHms: &CaptureGroupPattern = r"(?<epoch>[12][[:digit:]]{12}|9[[:digit:]]{11})";

/// for testing
#[doc(hidden)]
#[allow(dead_code)]
pub const CGP_YEAR_ALL: &[&CaptureGroupPattern] = &[
    CGP_YEAR,
    CGP_YEARy,
];

/// for testing
#[doc(hidden)]
#[allow(dead_code)]
pub const CGP_MONTH_ALL: &[&CaptureGroupPattern] = &[
    CGP_MONTHm,
    CGP_MONTHms,
    CGP_MONTHm_sd,
    CGP_MONTHb,
    CGP_MONTHB,
    CGP_MONTHBb,
];

/// for testing
#[doc(hidden)]
#[allow(dead_code)]
pub const CGP_DAY_ALL: &[&CaptureGroupPattern] = &[
    CGP_DAYde,
    CGP_DAYd,
    CGP_DAYa3,
    CGP_DAYa,
];

/// for testing
#[doc(hidden)]
#[allow(dead_code)]
pub const CGP_HOUR_ALL: &[&CaptureGroupPattern] = &[
    CGP_HOUR,
    CGP_HOUR_sd,
    CGP_HOURh,
];

/// for testing
#[doc(hidden)]
#[allow(dead_code)]
pub const CGP_MINUTE_ALL: &[&CaptureGroupPattern] = &[
    CGP_MINUTE,
    CGP_MINUTE_sd,
];

/// for testing
#[doc(hidden)]
#[allow(dead_code)]
pub const CGP_SECOND_ALL: &[&CaptureGroupPattern] = &[
    CGP_SECOND,
    CGP_SECOND_sd,
];

/// for testing
#[doc(hidden)]
#[allow(dead_code)]
pub const CGP_FRACTIONAL_ALL: &[&CaptureGroupPattern] = &[
    CGP_FRACTIONAL19,
    CGP_FRACTIONAL23,
    CGP_FRACTIONAL3,
    CGP_FRACTIONAL6,
    CGP_FRACTIONAL9,
    CGP_FRACTIONAL39,
    CGP_FRACTIONAL369,
];

/// for testing
#[doc(hidden)]
#[allow(dead_code)]
pub const CGP_EPOCH_ALL: &[&CaptureGroupPattern] = &[
    CGP_EPOCH,
    CGP_EPOCHms,
];

// Regarding timezone formatting, ISO 8601 allows Unicode "minus sign".
// See https://en.wikipedia.org/w/index.php?title=ISO_8601&oldid=1114291504#Time_offsets_from_UTC
// Unicode "minus sign" will be replaced with ASCII "hyphen-minus" for
// processing by chrono `DateTime::parse_from_str`.
// See https://github.com/chronotope/chrono/issues/835

/// Unicode MINUS SIGN `U+2212`
/// or as bytes `[0xE2, 0x88, 0x92]` or `[226, 136, 146]` or `b"\xe2\x88\x92"`
pub const MINUS_SIGN: &[u8] = "−".as_bytes();
/// Unicode/ASCII HYPHEN-MINUS `U+002D`
pub const HYPHEN_MINUS: &[u8] = "-".as_bytes();
// TODO: modify `ere` to handle regex with non-ASCII

// The "|" operator will match in order. And pattern ordering matters.
// So patterns should attempt longer matches first, e.g.
//   Pattern "PETT" should occur before "PET"; "PETT|PET"

/// `strftime` specifier `%z` e.g. `"+0930"`
pub const CGP_TZz: &CaptureGroupPattern = r"(?<tz>[+-−][012][[:digit:]]{3})";

/// `strftime` specifier `%:z` e.g. `"+09:30"`
pub const CGP_TZzc: &CaptureGroupPattern = r"(?<tz>[+-−][012][[:digit:]]:[[:digit:]]{2})";

/// `strftime` specifier `%#z` e.g. `"+09"`
pub const CGP_TZzp: &CaptureGroupPattern = r"(?<tz>[+-−][012][[:digit:]])";

/// `strftime` specifier `%Z` e.g. `"ACST"`, lowercase also allowed
/// ordering is important for more complete matches,
/// e.g. `"PETT"` should occur before `"PET"`
pub const CGP_TZZ: &CaptureGroupPattern = "(?<tz>\
ACDT|ACST|ACT|ACWST|ADT|AEDT|AEST|AET|AFT|AKDT|AKST|ALMT|AMST|AMT|ANAT|AQTT|ART|AST|AWST|AZOST|AZOT|AZT|BIOT|BIT|BNT|BOT|BRST|BRT|BST|BTT|CAT|CCT|CDT|CEST|CET|CHADT|CHAST|CHOST|CHOT|CHST|CHUT|CIST|CKT|CLST|CLT|COST|COT|CST|CT|CVT|CWST|CXT|DAVT|DDUT|DFT|EASST|EAST|EAT|ECT|EDT|EEST|EET|EGST|EGT|EST|ET|FET|FJT|FKST|FKT|FNT|GALT|GAMT|GET|GFT|GILT|GIT|GMT|GST|GYT|HAEC|HDT|HKT|HMT|HOVST|HOVT|HST|ICT|IDLW|IDT|IOT|IRDT|IRKT|IRST|IST|JST|KALT|KGT|KOST|KRAT|KST|LHST|LINT|MAGT|MART|MAWT|MDT|MEST|MET|MHT|MIST|MIT|MMT|MSK|MST|MUT|MVT|MYT|NCT|NDT|NFT|NOVT|NPT|NST|NT|NUT|NZDT|NZST|OMST|ORAT|PDT|PETT|PET|PGT|PHOT|PHST|PHT|PKT|PMDT|PMST|PONT|PST|PWT|PYST|PYT|RET|ROTT|SAKT|SAMT|SAST|SBT|SCT|SDT|SGT|SLST|SRET|SRT|SST|SYOT|TAHT|TFT|THA|TJT|TKT|TLT|TMT|TOT|TRT|TVT|ULAST|ULAT|UTC|UT|UYST|UYT|UZT|VET|VLAT|VOLT|VOST|VUT|WAKT|WAST|WAT|WEST|WET|WGST|WGT|WIB|WITA|WIT|WST|YAKT|YEKT|ZULU|Z|\
acdt|acst|act|acwst|adt|aedt|aest|aet|aft|akdt|akst|almt|amst|amt|anat|aqtt|art|ast|awst|azost|azot|azt|biot|bit|bnt|bot|brst|brt|bst|btt|cat|cct|cdt|cest|cet|chadt|chast|chost|chot|chst|chut|cist|ckt|clst|clt|cost|cot|cst|ct|cvt|cwst|cxt|davt|ddut|dft|easst|east|eat|ect|edt|eest|eet|egst|egt|est|et|fet|fjt|fkst|fkt|fnt|galt|gamt|get|gft|gilt|git|gmt|gst|gyt|haec|hdt|hkt|hmt|hovst|hovt|hst|ict|idlw|idt|iot|irdt|irkt|irst|ist|jst|kalt|kgt|kost|krat|kst|lhst|lint|magt|mart|mawt|mdt|mest|met|mht|mist|mit|mmt|msk|mst|mut|mvt|myt|nct|ndt|nft|novt|npt|nst|nt|nut|nzdt|nzst|omst|orat|pdt|pett|pet|pgt|phot|phst|pht|pkt|pmdt|pmst|pont|pst|pwt|pyst|pyt|ret|rott|sakt|samt|sast|sbt|sct|sdt|sgt|slst|sret|srt|sst|syot|taht|tft|tha|tjt|tkt|tlt|tmt|tot|trt|tvt|ulast|ulat|utc|ut|uyst|uyt|uzt|vet|vlat|volt|vost|vut|wakt|wast|wat|west|wet|wgst|wgt|wib|wita|wit|wst|yakt|yekt|zulu|z\
)";
/// same as [`CGP_TZZ`] but only uppercase
pub const CGP_TZZ_U: &CaptureGroupPattern = "(?<tz>\
ACDT|ACST|ACT|ACWST|ADT|AEDT|AEST|AET|AFT|AKDT|AKST|ALMT|AMST|AMT|ANAT|AQTT|ART|AST|AWST|AZOST|AZOT|AZT|BIOT|BIT|BNT|BOT|BRST|BRT|BST|BTT|CAT|CCT|CDT|CEST|CET|CHADT|CHAST|CHOST|CHOT|CHST|CHUT|CIST|CKT|CLST|CLT|COST|COT|CST|CT|CVT|CWST|CXT|DAVT|DDUT|DFT|EASST|EAST|EAT|ECT|EDT|EEST|EET|EGST|EGT|EST|ET|FET|FJT|FKST|FKT|FNT|GALT|GAMT|GET|GFT|GILT|GIT|GMT|GST|GYT|HAEC|HDT|HKT|HMT|HOVST|HOVT|HST|ICT|IDLW|IDT|IOT|IRDT|IRKT|IRST|IST|JST|KALT|KGT|KOST|KRAT|KST|LHST|LINT|MAGT|MART|MAWT|MDT|MEST|MET|MHT|MIST|MIT|MMT|MSK|MST|MUT|MVT|MYT|NCT|NDT|NFT|NOVT|NPT|NST|NT|NUT|NZDT|NZST|OMST|ORAT|PDT|PETT|PET|PGT|PHOT|PHST|PHT|PKT|PMDT|PMST|PONT|PST|PWT|PYST|PYT|RET|ROTT|SAKT|SAMT|SAST|SBT|SCT|SDT|SGT|SLST|SRET|SRT|SST|SYOT|TAHT|TFT|THA|TJT|TKT|TLT|TMT|TOT|TRT|TVT|ULAST|ULAT|UTC|UT|UYST|UYT|UZT|VET|VLAT|VOLT|VOST|VUT|WAKT|WAST|WAT|WEST|WET|WGST|WGT|WIB|WITA|WIT|WST|YAKT|YEKT|ZULU|Z\
)";
/// same as [`CGP_TZZ`] but only lowercase
pub const CGP_TZZ_L: &CaptureGroupPattern = "(?<tz>\
acdt|acst|act|acwst|adt|aedt|aest|aet|aft|akdt|akst|almt|amst|amt|anat|aqtt|art|ast|awst|azost|azot|azt|biot|bit|bnt|bot|brst|brt|bst|btt|cat|cct|cdt|cest|cet|chadt|chast|chost|chot|chst|chut|cist|ckt|clst|clt|cost|cot|cst|ct|cvt|cwst|cxt|davt|ddut|dft|easst|east|eat|ect|edt|eest|eet|egst|egt|est|et|fet|fjt|fkst|fkt|fnt|galt|gamt|get|gft|gilt|git|gmt|gst|gyt|haec|hdt|hkt|hmt|hovst|hovt|hst|ict|idlw|idt|iot|irdt|irkt|irst|ist|jst|kalt|kgt|kost|krat|kst|lhst|lint|magt|mart|mawt|mdt|mest|met|mht|mist|mit|mmt|msk|mst|mut|mvt|myt|nct|ndt|nft|novt|npt|nst|nt|nut|nzdt|nzst|omst|orat|pdt|pett|pet|pgt|phot|phst|pht|pkt|pmdt|pmst|pont|pst|pwt|pyst|pyt|ret|rott|sakt|samt|sast|sbt|sct|sdt|sgt|slst|sret|srt|sst|syot|taht|tft|tha|tjt|tkt|tlt|tmt|tot|trt|tvt|ulast|ulat|utc|ut|uyst|uyt|uzt|vet|vlat|volt|vost|vut|wakt|wast|wat|west|wet|wgst|wgt|wib|wita|wit|wst|yakt|yekt|zulu|z\
)";

/// for testing
#[doc(hidden)]
#[allow(dead_code)]
pub const CGP_TZ_ALL: &[&CaptureGroupPattern] = &[
    CGP_TZz,
    CGP_TZzc,
    CGP_TZzp,
    CGP_TZZ,
    CGP_TZZ_U,
    CGP_TZZ_L,
];

/// no alphabetic or line end, helper to [`CGP_TZZ`]
pub const RP_NOALPHA: &RegexPattern = r"([[:^alpha:]]|$)";

/// no alphabetic or line begin, helper to [`CGP_TZZ`] and [`CGP_YEAR`]
pub const RP_NOALPHAb: &RegexPattern = r"([[:^alpha:]]|^)";

/// no alphanumeric or line end, helper to [`CGP_TZZ`] and [`CGP_YEAR`]
pub const RP_NOALNUM: &RegexPattern = r"([[:^alnum:]]|$)";

/// no alphanumeric or line begin, helper to [`CGP_TZZ`] and [`CGP_YEAR`]
pub const RP_NOALNUMb: &RegexPattern = r"(^|[[:^alnum:]])";

/// no alphanumeric plus minus or line end, helper to [`CGP_TZZ`] and [`CGP_YEAR`]
pub const RP_NOALNUMpm: &RegexPattern = r"([[:^alnum:]]|-|\+|$)";

/// no numeric or line end, helper to [`CGP_TZZ`] and [`CGP_YEAR`]
pub const RP_NODIGIT: &RegexPattern = r"([[:^digit:]]|$)";

/// no numeric or line begin, helper to [`CGP_TZZ`] and [`CGP_YEAR`]
pub const RP_NODIGITb: &RegexPattern = r"([[:^digit:]]|^)";

/// one or more digits
pub const RP_DIGITS: &RegexPattern = "[[:digit:]]+";

/// one to three digits
pub const RP_DIGITS3: &RegexPattern = r"[[:digit:]]{1,3}";

/// field name header for date in RFC 2822 line-oriented message,
/// not matching wacky-case variants like `dAte:`
pub const RP_RFC2822_DATE: &RegexPattern = "(date|Date|DATE):";

/// [`RegexPattern`] divider _date?_ `2020/01/01` or `2020-01-01` or
/// `2020 01 01` or `2020-01-01` or `20200101`
pub const D_Dq: &RegexPattern = r"[ /\-]?";
/// [`RegexPattern`] divider _date?_ `2020/01/01` or `2020-01-01` or
/// `2020 01 01` or `20200101` or `2020\01\01`
/// Uses `\` which is typically only seen in messier datetime formats, like
/// those without timezones.
pub const D_Deq: &RegexPattern = r"[ /\\\-]?";
/// [`RegexPattern`] divider _date_, `2020/01/01` or `2020-01-01` or
/// `2020 01 01`
pub const D_D: &RegexPattern = r"[ /\-]";
/// [`RegexPattern`] divider _time_, `20:30:00`
pub const D_T: &RegexPattern = r"[\:]?";
/// [`RegexPattern`] divider _time_ with extras, `20:30:00` or `20-00`
pub const D_Teq: &RegexPattern = r"[\:\-]?";
/// [`RegexPattern`] divider _time_ with extras, `20:30:00` or `20-00`
pub const D_Te: &RegexPattern = r"[\:\-]";
/// [`RegexPattern`] divider _time_ dot colon
pub const D_Tcd: &RegexPattern = r"[\:\.]";
/// [`RegexPattern`] divider _time_ dot colon comma
pub const D_Tcdc: &RegexPattern = r"[\:\.,]";
/// [`RegexPattern`] divider _day_ to _hour_, `2020/01/01T20:30:00`
pub const D_DHq: &RegexPattern = "[ T]?";
/// [`RegexPattern`] divider _day_ to _hour_ with dash, `2020:01:01-20:30:00`.
pub const D_DHdq: &RegexPattern = r"[ T\-]?";
/// [`RegexPattern`] divider _day_ to _hour_ with colon or dash,
/// `2020:01:01-20:30:00`.
pub const D_DHcd: &RegexPattern = r"[ T\:\-]";
/// [`RegexPattern`] divider _day_ to _hour_ with colon or dash,
/// `2020:01:01-20:30:00`.
pub const D_DHcdq: &RegexPattern = r"[ T\:\-]?";
/// [`RegexPattern`] divider _day_ to _hour_ with colon or dash or underline,
/// `2020:01:01_20:30:00`.
pub const D_DHcdqu: &RegexPattern = r"[ T_\:\-]?";
/// [`RegexPattern`] divider _day_ to _hour_ with colon or dash or underline
/// or slash, `2020:01:01\20:30:00`.
pub const D_DHcdqus: &RegexPattern = r"[ T\:/\\\-_]?";
/// [`RegexPattern`] divider _day_ to _hour_ with colon or dash or underline
/// or slash, `2020:01:01\20:30:00`.
pub const D_DHcds: &RegexPattern = r"[ T\:/\\\-]";
/// [`RegexPattern`] divider _fractional_, `2020/01/01T20:30:00,123456`
// TODO: fix `ere` to handle char grouping ending with `\:` or `\\`
pub const D_SF: &RegexPattern = r"[\.,]";

/// [`RegexPattern`] dot or comma?
pub const RP_dcq: &RegexPattern = r"[\.,]?";
/// [`RegexPattern`] comma?
pub const RP_cq: &RegexPattern = "[,]?";
pub const RP_LEVELS_FRAGMENT_UP: &RegexPattern = r"DEBUG[[:digit:]]|DEBUG|INFO[[:digit:]]|INFO|ERROR[[:digit:]]|ERROR|ERR|TRACE[[:digit:]]|TRACE|WARN[[:digit:]]|WARN|WARNING|VERBOSE[[:digit:]]|VERBOSE|EMERGENCY|EMERG|NOTICE|CRIT|CRITICAL|ALERT[[:digit:]]|ALERT|PANIC";
pub const RP_LEVELS_FRAGMENT_LO: &RegexPattern = r"debug[[:digit:]]|debug|info[[:digit:]]|info|error[[:digit:]]|error|err|trace[[:digit:]]|trace|warn[[:digit:]]|warn|warning|verbose[[:digit:]]|verbose|emergency|emerg|notice|crit|critical|alert[[:digit:]]|alert|panic";
/// [`RegexPattern`] of commonly found syslog level names
///
/// References:
/// - <https://www.rfc-editor.org/rfc/rfc5427#section-3>
/// - <https://learningnetwork.cisco.com/s/article/syslog-severity-amp-level>
/// - <https://learningnetwork.cisco.com/s/feed/0D53i00000KsKHECA3> <https://archive.ph/RC33J>
/// - <https://success.trendmicro.com/dcx/s/solution/TP000086250>
pub const RP_LEVELS: &RegexPattern = concat!("(", RP_LEVELS_FRAGMENT_UP, "|", RP_LEVELS_FRAGMENT_LO, ")");
/// [`RegexPattern`] blank
pub const RP_BLANK: &RegexPattern = "[[:blank:]]";
/// [`RegexPattern`] blank?
pub const RP_BLANKq: &RegexPattern = "[[:blank:]]?";
/// [`RegexPattern`] blank or end
pub const RP_BLANKe: &RegexPattern = "([[:blank:]]|$)";
/// [`RegexPattern`] blank, 1 or 2
pub const RP_BLANK12: &RegexPattern = r"[[:blank:]]{1,2}";
/// [`RegexPattern`] blank, 1 or 2?
pub const RP_BLANK12q: &RegexPattern = r"([[:blank:]]{1,2})?";
/// [`RegexPattern`] blanks
pub const RP_BLANKS: &RegexPattern = "[[:blank:]]+";
/// [`RegexPattern`] blanks?
pub const RP_BLANKSq: &RegexPattern = "[[:blank:]]*";
/// [`RegexPattern`] _not_ blank
pub const RP_BLANK_NO: &RegexPattern = "[^[:blank:]]";
/// [`RegexPattern`] anything plus
pub const RP_ANYp: &RegexPattern = ".+";
/// [`RegexPattern`] left-side brackets
pub const RP_LB: &RegexPattern = r"(\[|\(|<|\{)";
/// [`RegexPattern`] right-side brackets
pub const RP_RB: &RegexPattern = r"(\]|\)|>|\})";

pub const GROUP_NAMES_LEN: usize = CGN_ALL.len();

/// All capture group names.
/// Must match ordering in [`CGN_ALL`]
pub const GROUP_NAMES: [&CaptureGroupName; GROUP_NAMES_LEN] = [
    CGN_YEAR,
    CGN_MONTH,
    CGN_DAY,
    CGN_DAY_IGNORE,
    CGN_HOUR,
    CGN_MINUTE,
    CGN_SECOND,
    CGN_FRACTIONAL,
    CGN_TZ,
    CGN_UPTIME,
    CGN_EPOCH,
];

::lazy_static::lazy_static! {
    /// The `HashSet` of [`GROUP_NAMES`]
    pub static ref GROUP_NAMES_SET: HashSet<String> = {
        let mut set = HashSet::new();
        for name in GROUP_NAMES.iter() {
            set.insert(name.to_string());
        }
        debug_assert_eq!(
            set.len(), GROUP_NAMES_LEN,
            "GROUP_NAMES_SET length {} does not match GROUP_NAMES_LEN {}",
            set.len(), GROUP_NAMES_LEN
        );
        defñ!("init GROUP_NAMES_SET");

        set
    };

    /// A map from group name `&str` to its index in [`GROUP_NAMES`].
    pub static ref GROUP_NAMES_MAP: HashMap<String, usize> = {
        let mut map = HashMap::new();
        map.insert(CGN_YEAR.to_string(), CGI_YEAR);
        map.insert(CGN_MONTH.to_string(), CGI_MONTH);
        map.insert(CGN_DAY.to_string(), CGI_DAY);
        map.insert(CGN_DAY_IGNORE.to_string(), CGI_DAY_IGNORE);
        map.insert(CGN_HOUR.to_string(), CGI_HOUR);
        map.insert(CGN_MINUTE.to_string(), CGI_MINUTE);
        map.insert(CGN_SECOND.to_string(), CGI_SECOND);
        map.insert(CGN_FRACTIONAL.to_string(), CGI_FRACTIONAL);
        map.insert(CGN_TZ.to_string(), CGI_TZ);
        map.insert(CGN_UPTIME.to_string(), CGI_UPTIME);
        map.insert(CGN_EPOCH.to_string(), CGI_EPOCH);
        debug_assert_eq!(
            map.len(), GROUP_NAMES_LEN,
            "GROUP_NAMES_MAP length {} does not match GROUP_NAMES_LEN {}",
            map.len(), GROUP_NAMES_LEN
        );
        defñ!("init GROUP_NAMES_MAP");

        map
    };

    pub static ref GROUP_NAMES_MAP_STR: HashMap<&'static str, usize> = {
        let mut map = HashMap::new();
        map.insert(CGN_YEAR, CGI_YEAR);
        map.insert(CGN_MONTH, CGI_MONTH);
        map.insert(CGN_DAY, CGI_DAY);
        map.insert(CGN_DAY_IGNORE, CGI_DAY_IGNORE);
        map.insert(CGN_HOUR, CGI_HOUR);
        map.insert(CGN_MINUTE, CGI_MINUTE);
        map.insert(CGN_SECOND, CGI_SECOND);
        map.insert(CGN_FRACTIONAL, CGI_FRACTIONAL);
        map.insert(CGN_TZ, CGI_TZ);
        map.insert(CGN_UPTIME, CGI_UPTIME);
        map.insert(CGN_EPOCH, CGI_EPOCH);
        debug_assert_eq!(
            map.len(), GROUP_NAMES_LEN,
            "GROUP_NAMES_MAP_STR length {} does not match GROUP_NAMES_LEN {}",
            map.len(), GROUP_NAMES_LEN
        );
        defñ!("init GROUP_NAMES_MAP_STR");

        map
    };
}

/// `Instr`uctions for `pars`ing from some unknown [`bytes`](u8) to a
/// [`regex::Regex.captures`] instance to a `&str` value that can be passed to
/// [`chrono::DateTime::parse_from_str`] or
/// [`chrono::NaiveDateTime::parse_from_str`].
///
/// An explanation of a `DateTimeParseInstr` instance:
///
/// 1. All `DateTimeParseInstr` instances are declared within the array
///    [`pub const DATETIME_PARSE_DATAS`].
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
///    [`SyslineReader`], and later passed to a [`PrinterLogMessage`] which
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
/// [`regex::Regex.captures`]: https://docs.rs/regex/1.10.5/regex/bytes/struct.Captures.html
/// [`chrono::DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/struct.DateTime.html#method.parse_from_str
/// [`chrono::NaiveDateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/naive/struct.NaiveDateTime.html#method.parse_from_str
/// [chrono `strftime`]: https://docs.rs/chrono/0.4.38/chrono/format/strftime/index.html
/// [`SyslineReader`]: crate::readers::syslinereader::SyslineReader
/// [`PrinterLogMessage`]: crate::printer::printers::PrinterLogMessage
/// [`Line`]: crate::data::line::Line
#[derive(Hash)]
pub struct DateTimeParseInstr<'a> {
    pub regex_id: RegexId,
    /// Regex pattern for [`captures`].
    ///
    /// [`captures`]: https://docs.rs/regex/1.10.5/regex/bytes/struct.Regex.html#method.captures
    pub regex_pattern: &'a DateTimeRegex_str,
    //pub regex_fields: EreRegexFields,
    pub regex_fn: RegexFn,
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
    /// instead of the entire `Line`, may improve run-time
    /// performance.
    ///
    /// [`Line`]: crate::data::line::Line
    /// [`Regex`]: https://docs.rs/regex/1.10.5/regex/bytes/struct.Regex.html#method.captures
    pub range_regex: RangeLineIndex,
    /// Capture named group first (left-most) position in `regex_pattern`.
    pub cgn_first: &'a CaptureGroupName,
    /// Capture named group last (right-most) position in `regex_pattern`.
    pub cgn_last: &'a CaptureGroupName,
    /// Hardcoded self-test cases.
    pub _test_cases: &'a [(LineIndex, LineIndex, ymdhmsn_args, &'a HaystackType)],
    /// Source code line number of declaration.
    /// Only to aid humans reviewing failing tests.
    pub _line_num: u32,
    /// Counter of declared instances.
    pub _counter: usize,
}

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

/// Only used for tests.
impl PartialOrd for DateTimeParseInstr<'_> {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Only used for tests.
impl PartialEq for DateTimeParseInstr<'_> {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.regex_pattern == other.regex_pattern && self.dtfs == other.dtfs
    }
}

/// Only used for tests.
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

impl DateTimeParseInstr<'_> {
    /// wrapper to [`DTFSSet::has_year`]
    pub const fn has_year(&self) -> bool {
        self.dtfs.has_year()
    }

    /// wrapper to [`DTFSSet::has_year4`]
    pub const fn has_year4(&self) -> bool {
        self.dtfs.has_year4()
    }

    /// wrapper to [`DTFSSet::has_tz`]
    pub const fn has_tz(&self) -> bool {
        self.dtfs.has_tz()
    }

    /// wrapper to [`DTFSSet::has_d2`]
    pub const fn has_d2(&self) -> bool {
        self.dtfs.has_d2()
    }
}

/// Returns a `DateTimeParseInstr` instance with the provided values
/// including the compiled `ere` match function.
#[macro_export]
macro_rules! ERE_REGEX_DATETIME {
    (
        $regex_id:literal,
        $counter:expr,
        $regex_pattern:expr,
        $engine:ident,
        $dtfs:expr,
        $sib:literal,
        $sie:literal,
        $cgn_first:ident,
        $cgn_last:ident,
        $test_cases:expr,
        $line_num:expr,
    ) => {
        {
            DateTimeParseInstr {
                regex_id: $regex_id as RegexId,
                regex_pattern: $regex_pattern,
                regex_fn: new_ere_regex!(
                    $regex_id,
                    $regex_pattern.as_bytes(),
                    $engine,
                    file!(),
                    $line_num,
                    true
                ),
                dtfs: $dtfs,
                range_regex: RangeLineIndex {
                    start: $sib,
                    end: $sie,
                },
                cgn_first: $cgn_first,
                cgn_last: $cgn_last,
                //#[cfg(test)]
                _test_cases: $test_cases,
                //#[cfg(not(test))]
                //_test_cases: &[],
                _line_num: $line_num,
                _counter: $counter
            }
        }
    };
}

pub const DP_KEY: &str = "ERE_REGEXES";

/// The global list of built-in Datetime parsing "instructions".
///
/// These are all the possible regular expression patterns that will be
/// match-attempted on each [`Line`] of a processed file.
///
/// Order of declaration matters: during initial parsing of a syslog file, all
/// of these regex patterns are match-attempted in order of their declaration.
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
/// Generally, more specific regex patterns should be listed before
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
/// it may try *all* the patterns (from index 0 of
/// `DATETIME_PARSE_DATAS` to wherever it finds a match).
/// So if a file has a datetime pattern that matches the last entry in
/// `DATETIME_PARSE_DATAS` then the `SyslineReader` will try *all*
/// the `DateTimeParseInstr` within `DATETIME_PARSE_DATAS` several times.
/// Once the controlling `SyslineReader` calls
/// [`dt_patterns_analysis`] for the last time, then only one
/// `DateTimeParseInstr` is tried for each `Line`.
/// But until `dt_patterns_analysis` is called, there will be many missed
/// matches. Regular expression creation and matching uses a large amount of
/// compute and time resources.
///
/// [`SyslineReader`]: crate::readers::syslinereader::SyslineReader
/// [`dt_patterns_analysis`]: crate::readers::syslinereader::SyslineReader#method.dt_patterns_analysis
/// [`Line`]: crate::data::line::Line
// TODO: [2023/01/14] add test of shortest possible match for all DTPD!
// TODO: [2023/05/11] modify `_tests` to allow testing invalid patterns, e.g.
//       ("2000-XY-01T00:00:00Z")
//       consider a two-value enum `DateTimeParseInstrTest` with variants
//       `Valid` and `Invalid`
pub const DATETIME_PARSE_DATAS: [DateTimeParseInstr; DATETIME_PARSE_DATAS_LEN] = [
    #[cfg(any(regex = "1", regex = "ALL", regex = "TEST"))]
    ERE_REGEX_DATETIME!(
        1,
        counter!(DP_KEY),
        concat!("^", RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZz, RP_RB),
        DfaU8,
        DTFSS_YmdHMSfz,
        0, 40,
        CGN_YEAR, CGN_TZ,
        &[
          (1, 28, (O_M11, 2000, 1, 1, 0, 0, 2, 100000000), b"(2000/01/01 00:00:02.1 -1100) ../source3/smbd/oplock.c:1340(init_oplocks)"),
          (1, 29, (O_M11, 2000, 1, 1, 0, 0, 2, 120000000), b"(2000/01/01 00:00:02.12 -1100) ../source3/smbd/oplock.c:1340(init_oplocks)"),
          (1, 30, (O_M11, 2000, 1, 1, 0, 0, 2, 123000000), b"(2000/01/01 00:00:02.123 -1100)\xFF ../source3/smbd/oplock.c:1340(init_oplocks)"),
          (1, 33, (O_M11, 2000, 1, 1, 0, 0, 2, 123400000), b"(2000/01/01 00:00:02.1234 \xe2\x88\x921100)\xFF ../source3/smbd/oplock.c:1340(init_oplocks)"),
          (1, 32, (O_M11, 2000, 1, 1, 0, 0, 2, 123450000), b"(2000/01/01 00:00:02.12345 -1100) ../source3/smbd/oplock.c:1340(init_oplocks)"),
          (1, 36, (O_M11, 2000, 1, 1, 0, 0, 2, 123456789), b"(2000/01/01 00:00:02.123456789 -1100) ../source3/smbd/oplock.c:1340(init_oplocks)"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "2", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        2,
        counter!(DP_KEY),
        concat!("^", RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzc, RP_RB),
        DfaU8,
        DTFSS_YmdHMSfzc,
        0, 40,
        CGN_YEAR, CGN_TZ,
        &[
          (1, 37, (O_M1130, 2000, 1, 1, 0, 0, 3, 123456789), br"{2000/01/01 00:00:03.123456789 -11:30} ../source3/smbd/oplock.c:1340(init_oplocks)"),
          (1, 31, (O_M1130, 2000, 1, 1, 0, 0, 3, 123000000), br"{2000/01/01 00:00:03.123 -11:30} ../source3/smbd/oplock.c:1340(init_oplocks)"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "3", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        3,
        counter!(DP_KEY),
        concat!("^", RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzp, RP_RB), // reduced
        DfaU8,
        DTFSS_YmdHMSfzp,
        0, 40,
        CGN_YEAR, CGN_TZ,
        &[
            (1, 34, (O_M11, 2000, 1, 1, 0, 0, 4, 123456789), b"(2000/01/01 00:00:04.123456789 -11) ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 34, (O_M11, 2000, 1, 1, 0, 0, 4, 123456789), b"(2000/01/01 00:00:04.123456789 -11)"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "4", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        4,
        counter!(DP_KEY),
        concat!("^", RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL369, RP_BLANKq, CGP_TZZ, RP_RB),
        DfaU8,
        DTFSS_YmdHMSfZ,
        0, 40,
        CGN_YEAR, CGN_TZ,
        &[
            (1, 35, (O_VLAT, 2000, 1, 1, 0, 0, 5, 123456789), b"(2000/01/01 00:00:05.123456789 VLAT) ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 34, (O_WAT, 2000, 1, 1, 0, 0, 5, 123456789), b"<2000/01/01 00:00:05.123456789 WAT> ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 34, (O_PST, 2000, 1, 1, 0, 0, 5, 123456789), b"<2000/01/01 00:00:05.123456789 PST> ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 28, (O_PST, 2000, 1, 1, 0, 0, 5, 123000000), b"[2000/01/01 00:00:05,123 PST] ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 34, (O_PST, 2000, 1, 1, 0, 0, 5, 123456789), b"<2000/01/01 00:00:05.123456789 pst> ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 33, (O_PST, 2000, 1, 1, 0, 0, 5, 123456789), b"<2000/01/01 00:00:05.123456789pst> ../source3/smbd/oplock.c:1340(init_oplocks)"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "5", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        5,
        counter!(DP_KEY),
        concat!("^", RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, r"[,\.]", RP_BLANKSq, r"[a-zA-Z0-9]{1,20}", RP_RB),
        DfaU8,
        DTFSS_YmdHMSf,
        0, 40,
        CGN_YEAR, CGN_FRACTIONAL,
        &[
            (1, 27, (O_L, 2020, 3, 5, 12, 17, 59, 631000000), b"[2020/03/05 12:17:59.631000, FOO] ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 27, (O_L, 2020, 3, 5, 12, 17, 59, 631000000), b"[2020/03/05 12:17:59.631000, bubba] ../source3/smbd/oplock.c:1340(init_oplocks)"),
            (1, 27, (O_L, 2026, 4, 3, 21, 58, 47, 225843000), b"[2026/04/03 21:58:47.225843,  22] ../../auth/gensec/gensec_start.c:987(gensec_register)"),
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
    #[cfg(any(regex = "6", regex = "ALL", regex = "TEST"))]
    ERE_REGEX_DATETIME!(
        6,
        counter!(DP_KEY),
        concat!("^", RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEARy, D_DHq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_RB),
        DfaU8,
        DTFSS_ybdHMS,
        0, 40,
        CGN_DAY, CGN_SECOND,
        &[
            (1, 19, (O_L, 2017, 2, 22, 21, 24, 20, 0), b"[22-Feb-17 21:24:20] Section [ALLOWED-CLIENTS] Invalid entry 192.168.0.0-192.168.0.255 in ini file, ignored")
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    // syslog messages using datetime format ISO 8601 / RFC 3339 (real "syslog" messages)
    //
    //      <14>2023-01-01T15:00:36-08:00 (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received \xE2\xB8\xA8<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received\xE2\xB8\xA9
    //      <29>2023-01-01T14:21:13-08:00 (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up \xE2\xB8\xA8<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up\xE2\xB8\xA9
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
    #[cfg(any(regex = "7", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        7,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzc, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMSfzc,
        0, 50,
        CGN_YEAR, CGN_TZ,
        &[
            (4, 36, (O_M8, 2023, 1, 6, 14, 35, 0, 506282000), b"<31>2023-01-06T14:35:00.506282-08:00 (host) (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
            (6, 42, (O_M8, 2023, 1, 6, 14, 35, 0, 506282871), b"<128> 2023-01-06T14:35:00.506282871 -08:00[host] (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "8", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        8,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMSfz,
        0, 50,
        CGN_YEAR, CGN_TZ,
        &[
            (4, 35, (O_P8, 2023, 1, 6, 14, 35, 0, 506282000), b"<31>2023-01-06T14:35:00.506282+0800 (host) (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
            (6, 41, (O_P8, 2023, 1, 6, 14, 35, 0, 506282871), b"<128> 2023-01-06T14:35:00.506282871 +0800[host] (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "9", regex = "ALL", regex = "TEST"))]
    ERE_REGEX_DATETIME!(
        9,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzp, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMSfzp,
        0, 50,
        CGN_YEAR, CGN_TZ,
        &[
            (4, 33, (O_P8, 2023, 1, 6, 14, 35, 0, 506282000), b"<31>2023-01-06T14:35:00.506282+08 (host) (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
            (6, 39, (O_P8, 2023, 1, 6, 14, 35, 0, 506282871), b"<128> 2023-01-06T14:35:00.506282871 +08[host] (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "10", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        10,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZZ, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMSfZ,
        0, 50,
        CGN_YEAR, CGN_TZ,
        &[
            (4, 34, (O_PST, 2023, 1, 6, 14, 35, 0, 506282000), b"<31>2023-01-06T14:35:00.506282 PST (host) (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
            (6, 40, (O_WITA, 2023, 1, 6, 14, 35, 0, 506282871), b"<128> 2023-01-06T14:35:00.506282871 WITA[host] (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
            (6, 39, (O_WITA, 2023, 1, 6, 14, 35, 0, 506282871), b"<128> 2023-01-06T14:35:00.506282871WITA[host] (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
            (6, 39, (O_WITA, 2023, 1, 6, 14, 35, 0, 506282871), b"<128> 2023-01-06T14:35:00.506282871WITA|host|192.168.0.1|unbound[63893]|daemon|debug|cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "11", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        11,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_NOALNUM),
        DfaU8,
        DTFSS_YmdHMSf,
        0, 50,
        CGN_YEAR, CGN_FRACTIONAL,
        &[
            (4, 30, (O_L, 2023, 1, 6, 14, 35, 0, 506282000), b"<31>2023-01-06T14:35:00.506282 (host) (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
            (6, 35, (O_L, 2023, 1, 6, 14, 35, 0, 506282871), b"<128> 2023-01-06T14:35:00.506282871[host] (192.168.0.1) [unbound[63893] daemon:debug] [63893]:  [63893:0] debug: cache memory msg=76002 rrset=120560 infra=18065 val=78826"),
        ],
        line!(),
    ),
    // syslog format without fractional seconds
    #[cfg(any(regex = "12", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        12,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzc),
        DfaU8,
        DTFSS_YmdHMSzc,
        0, 46,
        CGN_YEAR, CGN_TZ,
        &[
            (4, 29, (O_M8, 2023, 2, 1, 15, 0, 36, 0), b"<14>2023-02-01T15:00:36-08:00 (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received \xE2\xB8\xA8<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received\xE2\xB8\xA9"),
            (4, 29, (O_0, 2023, 2, 1, 15, 0, 36, 0), b"<14>2023-02-01T15:00:36+00:00 (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received \xE2\xB8\xA8<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received\xE2\xB8\xA9"),
            (4, 30, (O_M8, 2023, 2, 1, 14, 21, 13, 0), b"<29>2023-02-01T14:21:13 -08:00 (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up \xE2\xB8\xA8<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up\xE2\xB8\xA9"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "13", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        13,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzp),
        DfaU8,
        DTFSS_YmdHMSzp,
        0, 50,
        CGN_YEAR, CGN_TZ,
        &[
            (4, 26, (O_M8, 2023, 2, 1, 15, 0, 36, 0), b"<14>2023-02-01T15:00:36-08 (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received \xE2\xB8\xA8<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received\xE2\xB8\xA9"),
            (4, 27, (O_M8, 2023, 2, 1, 14, 21, 13, 0), b"<29>2023-02-01T14:21:13 -08 (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up \xE2\xB8\xA8<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up\xE2\xB8\xA9"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "14", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        14,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZZ, RP_NOALNUM),
        DfaU8,
        DTFSS_YmdHMSZ,
        0, 50,
        CGN_YEAR, CGN_TZ,
        &[
            (4, 27, (O_PST, 2023, 2, 1, 15, 0, 36, 0), b"<14>2023-02-01T15:00:36 PST (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received \xE2\xB8\xA8<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received\xE2\xB8\xA9"),
            (4, 28, (O_CIST, 2023, 2, 1, 14, 21, 13, 0), b"<29>2023-02-01T14:21:13 CIST (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up \xE2\xB8\xA8<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up\xE2\xB8\xA9"),
            (4, 27, (O_CIST, 2023, 2, 1, 14, 21, 13, 0), b"<29>2023-02-01T14:21:13CIST (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up \xE2\xB8\xA8<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up\xE2\xB8\xA9"),
            (4, 24, (O_Z, 2023, 2, 1, 14, 21, 13, 0), b"<29>2023-02-01T14:21:13Z (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up \xE2\xB8\xA8<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up\xE2\xB8\xA9"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "15", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        15,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NOALNUM),
        DfaU8,
        DTFSS_YmdHMS,
        0, 45,
        CGN_YEAR, CGN_SECOND,
        &[
            (4, 23, (O_L, 2023, 2, 1, 15, 0, 36, 0), b"<14>2023-02-01T15:00:36 (HOST) (192.168.0.1) [dropbear[23732]: authpriv:info] [23732]:  Exit (root): Disconnect received \xE2\xB8\xA8<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received\xE2\xB8\xA9"),
            (4, 23, (O_L, 2023, 2, 1, 14, 21, 13, 0), b"<29>2023-02-01T14:21:13(HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up \xE2\xB8\xA8<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up\xE2\xB8\xA9"),
            (5, 24, (O_L, 2023, 2, 1, 14, 21, 13, 0), b"<191>2023-02-01T14:21:13 (HOST) (192.168.0.1) [netifd: daemon:notice] [-]:  Network device 'eth0' link is up \xE2\xB8\xA8<29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up\xE2\xB8\xA9"),
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
    #[cfg(any(regex = "16", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        16,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKSq, CGP_TZzc, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSYzc,
        0, 40,
        CGN_MONTH, CGN_TZ,
        &[
            (4, 31, (O_M2, 2023, 1, 1, 15, 0, 36, 0), b"<14>Jan  1 15:00:36 2023 -02:00 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (4, 31, (O_M2, 2023, 1, 1, 15, 0, 36, 0), b"<14>Jan 01 15:00:36 2023 -02:00 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 30, (O_M2, 2023, 1, 31, 14, 21, 13, 0), b"<1>Jan 31 14:21:13 2023 -02:00 HOST netifd: Network device 'eth0' link is up"),
            (5, 32, (O_M2, 2023, 1, 31, 14, 21, 13, 0), b"<135>Jan 31 14:21:13 2023 -02:00 HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "17", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        17,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKSq, CGP_TZzp, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSYzp,
        0, 40,
        CGN_MONTH, CGN_TZ,
        &[
            (4, 28, (O_M2, 2023, 1, 1, 15, 0, 36, 0), b"<14>Jan  1 15:00:36 2023 -02 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (4, 28, (O_M2, 2023, 1, 1, 15, 0, 36, 0), b"<14>Jan 01 15:00:36 2023 -02 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 27, (O_M2, 2023, 1, 31, 14, 21, 13, 0), b"<1>Jan 31 14:21:13 2023 -02 HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "18", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        18,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKSq, CGP_TZZ, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSYZ,
        0, 40,
        CGN_MONTH, CGN_TZ,
        &[
            (4, 29, (O_WGST, 2023, 1, 1, 15, 0, 36, 0), b"<14>Jan  1 15:00:36 2023 WGST HOST dropbear[23732]: Exit (root): Disconnect received"),
            (4, 29, (O_WGST, 2023, 1, 1, 15, 0, 36, 0), b"<14>Jan 01 15:00:36 2023 WGST HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 28, (O_WGST, 2023, 1, 31, 14, 21, 13, 0), b"<1>Jan 31 14:21:13 2023 WGST HOST netifd: Network device 'eth0' link is up"),
            (3, 27, (O_WGST, 2023, 1, 31, 14, 21, 13, 0), b"<1>Jan 31 14:21:13 2023WGST HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "19", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        19,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSY,
        0, 35,
        CGN_MONTH, CGN_YEAR,
        &[
            (4, 24, (O_L, 2023, 1, 1, 15, 0, 36, 0), b"<14>Jan  1 15:00:36 2023 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 23, (O_L, 2023, 1, 31, 14, 21, 13, 0), b"<1>Jan 31 14:21:13 2023 HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    //
    //      <14>Jan  1 15:00:36 WIST 2023 HOST dropbear[23732]: Exit (root): Disconnect received
    //
    #[cfg(any(regex = "20", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        20,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_TZzc, RP_BLANKS, CGP_YEAR, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSYzc,
        0, 40,
        CGN_MONTH, CGN_YEAR,
        &[
            (4, 31, (O_M2, 2023, 1, 1, 15, 0, 36, 0), b"<14>Jan  1 15:00:36 -02:00 2023 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 30, (O_M2, 2023, 1, 31, 14, 21, 13, 0), b"<1>Jan 31 14:21:13 -02:00 2023 HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "21", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        21,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_TZzp, RP_BLANKS, CGP_YEAR, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSYzp,
        0, 40,
        CGN_MONTH, CGN_YEAR,
        &[
            (4, 28, (O_M2, 2023, 1, 1, 15, 0, 36, 0), b"<14>Jan  1 15:00:36 -02 2023 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 27, (O_M2, 2023, 1, 31, 14, 21, 13, 0), b"<1>Jan 31 14:21:13 -02 2023 HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "22", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        22,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_TZZ, RP_BLANKS, CGP_YEAR, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSYZ,
        0, 40,
        CGN_MONTH, CGN_YEAR,
        &[
            (4, 29, (O_WGST, 2023, 1, 1, 15, 0, 36, 0), b"<14>Jan  1 15:00:36 WGST 2023 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 28, (O_WGST, 2023, 1, 31, 14, 21, 13, 0), b"<1>Jan 31 14:21:13 WGST 2023 HOST netifd: Network device 'eth0' link is up"),
        ],
        line!(),
    ),
    //
    //      <14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received
    //      <29>Jan  1 14:21:13 HOST netifd: Network device 'eth0' link is up
    //      <7>Mar 23 09:35:30 localhost DPKG [9332:<console>.<module>:2] debug info
    //
    #[cfg(any(regex = "23", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        23,
        counter!(DP_KEY),
        concat!("^<", RP_DIGITS3, ">", RP_BLANKq, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKSq),
        DfaU8,
        DTFSS_BdHMS,
        0, 30,
        CGN_MONTH, CGN_SECOND,
        &[
            (4, 19, (O_L, YD, 1, 1, 15, 0, 36, 0), b"<14>Jan  1 15:00:36 HOST dropbear[23732]: Exit (root): Disconnect received"),
            (3, 18, (O_L, YD, 1, 31, 14, 21, 13, 0), b"<1>Jan 31 14:21:13 HOST netifd: Network device 'eth0' link is up"),
            (3, 18, (O_L, YD, 3, 23, 9, 35, 30, 0), b"<7>Mar 23 09:35:30 localhost DPKG [9332:<console>.<module>:2] debug info"),
            (5, 20, (O_L, YD, 3, 23, 9, 35, 30, 0), b"<144>Mar 23 09:35:30 localhost DPKG [9332:<console>.<module>:2] debug info"),
            // example scraped from RFC 3164 (https://www.rfc-editor.org/rfc/rfc3164.html#section-5.4)
            (4, 19, (O_L, YD, 10, 11, 22, 14, 15, 0), b"<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8"),
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
    #[cfg(any(regex = "24", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        24,
        counter!(DP_KEY),
        concat!("^((log|Log|LOG) (started|Started|STARTED|ended|Ended|ENDED))", RP_BLANKSq, "[:]?", RP_BLANKSq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, "(T|[[:blank:]]+)", CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NOALNUM),
        DfaU8,
        DTFSS_YmdHMS,
        0, 50,
        CGN_YEAR, CGN_SECOND,
        &[
            (13, 33, (O_L, 2022, 7, 14, 6, 48, 58, 0), b"Log started: 2022-07-14  06:48:58\n(Reading database \xE2\x80\xA6"),
            (13, 32, (O_L, 2022, 7, 14, 6, 48, 58, 0), b"Log started: 2022-07-14 06:48:58 Reading database ..."),
            (13, 32, (O_L, 2022, 7, 14, 6, 48, 58, 0), b"Log started: 2022-07-14T06:48:58"),
            (11, 31, (O_L, 2022, 7, 14, 6, 49, 59, 0), b"Log ended: 2022-07-14  06:49:59"),
            (11, 30, (O_L, 2022, 7, 14, 6, 49, 59, 0), b"Log ended:\t2022-07-14\t06:49:59"),
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
    #[cfg(any(regex = "25", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        25,
        counter!(DP_KEY),
        concat!("(Started On|Started on|started on|STARTED|Started|started|Finished On|Finished on|finished on|FINISHED|Finished|finished)[:]?", RP_BLANK, CGP_DAYa, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        //DfaU8, // 10m39s build time
        FlatLockstepNfaU8,
        DTFSS_YbdHMS,
        0, 140,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (11, 35, (O_L, 2020, 9, 10, 10, 8, 35, 0), b"Started On Thu Sep 10 10:08:35 2020"),
            (11, 34, (O_L, 2020, 9, 1, 10, 8, 35, 0), b"Started On Thu Sep 1 10:08:35 2020"),
            (11, 35, (O_L, 2020, 9, 1, 10, 8, 35, 0), b"Started On Thu Sep  1 10:08:35 2020"),
            (11, 35, (O_L, 2020, 9, 1, 10, 8, 35, 0), b"Started On Thu Sep. 1 10:08:35 2020"),
            (11, 35, (O_L, 2020, 9, 1, 10, 8, 35, 0), b"Started On Thu. Sep 1 10:08:35 2020"),
            (11, 36, (O_L, 2020, 9, 1, 10, 8, 35, 0), b"Started On Thu. Sep. 1 10:08:35 2020"),
            (11, 36, (O_L, 2020, 9, 1, 10, 8, 35, 0), b"Started On thu. sep. 1 10:08:35 2020"),
            (11, 36, (O_L, 2020, 9, 1, 10, 8, 35, 0), b"Started On THU. SEP. 1 10:08:35 2020"),
            (62, 86, (O_L, 2020, 11, 10, 18, 54, 47, 0), b"Microsoft Windows Malicious Software Removal Tool Finished On Tue Nov 10 18:54:47 2020"),
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
    #[cfg(any(regex = "26", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        26,
        counter!(DP_KEY),
        concat!("^", RP_LB, CGP_MONTHm, D_D, CGP_DAYde, D_D, CGP_YEAR, D_DHdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_RB),
        DfaU8,
        DTFSS_YmdHMSf,
        0, 40,
        CGN_MONTH, CGN_FRACTIONAL,
        &[
            (1, 25, (O_L, 2019, 8, 10, 1, 46, 44, 4200000), br#"(08/10/2019-01:46:44.0042) Filtering object "\\HOST\ROOT\CIMV2\mdm\dmmap:MDM_Policy_Config01_Location02" during apply"#),
            (1, 25, (O_L, 2020, 5, 27, 12, 25, 43, 87700000), b"(05/27/2020-12:25:43.0877) Total number of objects successfully migrated :2346, failed objects :16"),
            (1, 24, (O_L, 2020, 5, 27, 12, 25, 43, 87000000), b"<05/27/2020-12:25:43.087> Total number of objects successfully migrated :2346, failed objects :16"),
            // XXX: brackets can be mismatched which is not preferred. However, adding a regexp
            //      for every bracket type would be too many regexp and/or too complex.
            (1, 24, (O_L, 2020, 5, 27, 12, 25, 43, 87000000), b"(05/27/2020-12:25:43.087> Total number of objects successfully migrated :2346, failed objects :16"),
            (1, 24, (O_L, 2020, 5, 27, 12, 25, 43, 87000000), b"{05/27/2020-12:25:43.087] Total number of objects successfully migrated :2346, failed objects :16"),
            (1, 24, (O_L, 2020, 5, 27, 12, 25, 43, 87000000), b"[05/27/2020-12:25:43.087> Total number of objects successfully migrated :2346, failed objects :16"),
            (1, 24, (O_L, 2020, 5, 27, 12, 25, 43, 87000000), b"{05/27/2020-12:25:43.087] Total number of objects successfully migrated :2346, failed objects :16"),
        ],
        line!(),
    ),
    //
    // ---------------------------------------------------------------------------------------------
    // from the beginning of file `C:/Windows/INF/setupapi.setup.log`,
    // the `Boot Session` is printed once at the beginning, then it prints
    // `Section [start|end]`
    //
    //     [Device Install Log]
    //          OS Version = 10.0.22621
    //          Service Pack = 0.0
    //          Suite = 0x0100
    //          ProductType = 1
    //          Architecture = amd64
    //
    //     [BeginLog]
    //
    //     [Boot Session: 2023/02/21 07:06:52.500]
    //
    //     >>>  [Sysprep Specialize - {8effc0f9-2fb5-a7ad-8a21-581222c83283}]
    //     >>>  Section start 2023/02/21 07:06:59.461
    //       set: System Information:
    //       set:      BIOS Release Date: 11/11/2023
    //       set:      BIOS Vendor: American Megatrends Inc.
    //       set:      BIOS Version: 2003
    //       set:      System Family: To be filled by O.E.M.
    //       set:      System Manufacturer: ASUS
    //         set: Initialized PnP data. Time = 47 ms. 07:06:59.524
    //         set: Cleaned up unneeded PnP data. Time = 0 ms. 07:07:05.689
    //         set: Installed primitive drivers. Time = 15 ms. 07:07:05.706
    //     <<<  Section end 2023/02/21 07:07:05.710
    //     <<<  [Exit status: SUCCESS]
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
    #[cfg(any(regex = "27", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        27,
        counter!(DP_KEY),
        concat!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZZ_U, RP_NOALPHA),
        DfaU8,
        DTFSS_BdHMSYZ,
        0, 40,
        CGN_MONTH, CGN_TZ,
        &[
            (0, 30, (O_PWT, 2000, 9, 3, 8, 10, 29, 0), b"September 03 08:10:29 2000 PWT hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 24, (O_PWT, 2000, 1, 3, 1, 2, 4, 0), b"Jan 03 01:02:04 2000 PWT \xF0\x9F\x98\x80"),
            (0, 24, (O_PWT, 2000, 1, 3, 1, 2, 4, 0), b"Jan  3 01:02:04 2000 PWT \xF0\x9F\x98\x80"),
            (0, 23, (O_PWT, 2000, 1, 3, 1, 2, 4, 0), b"Jan 3 01:02:04 2000 PWT \xF0\x9F\x98\x80"),
            (0, 24, (O_PWT, 2000, 2, 29, 1, 2, 4, 0), b"Feb 29 01:02:04 2000 PWT \xF0\x9F\x98\x80"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "28", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        28,
        counter!(DP_KEY),
        concat!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZzc, RP_NOALPHA),
        DfaU8,
        DTFSS_BdHMSYzc,
        0, 40,
        CGN_MONTH, CGN_TZ,
        &[
            (0, 33, (O_P3, 2000, 9, 3, 8, 10, 29, 0), b"September 03 08:10:29 2000 +03:00 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 27, (O_P3, 2000, 1, 3, 1, 2, 4, 0), b"Jan 03 01:02:04 2000 +03:00 \xF0\x9F\x98\x80"),
            (0, 26, (O_M730, 2000, 1, 3, 1, 2, 4, 0), b"Jan 3 01:02:04 2000 -07:30 \xF0\x9F\x98\x80"),
            (0, 27, (O_M730, 2000, 1, 3, 1, 2, 4, 0), b"Jan  3 01:02:04 2000 -07:30 \xF0\x9F\x98\x80"),
            (0, 27, (O_P3, 2000, 2, 29, 1, 2, 4, 0), b"Feb 29 01:02:04 2000 +03:00 \xF0\x9F\x98\x80"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "29", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        29,
        counter!(DP_KEY),
        concat!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZz, RP_NOALPHA),
        DfaU8,
        DTFSS_BdHMSYz,
        0, 40,
        CGN_MONTH, CGN_TZ,
        &[
            (0, 32, (O_P3, 2000, 9, 3, 8, 10, 29, 0), b"September 03 08:10:29 2000 +0300 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 26, (O_P3, 2000, 1, 3, 1, 2, 4, 0), b"Jan 03 01:02:04 2000 +0300 \xF0\x9F\x98\x80"),
            (0, 26, (O_P3, 2000, 1, 3, 1, 2, 4, 0), b"Jan  3 01:02:04 2000 +0300 \xF0\x9F\x98\x80"),
            (0, 25, (O_M730, 2000, 1, 2, 3, 4, 5, 0), b"Jan 2 03:04:05 2000 -0730 \xF0\x9F\x98\x80"),
            (0, 26, (O_P3, 2000, 2, 29, 1, 2, 4, 0), b"Feb 29 01:02:04 2000 +0300 \xF0\x9F\x98\x80"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "30", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        30,
        counter!(DP_KEY),
        concat!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZzp, RP_NOALPHA),
        DfaU8,
        DTFSS_BdHMSYzp,
        0, 40,
        CGN_MONTH, CGN_TZ,
        &[
            (0, 30, (O_P3, 2000, 9, 3, 8, 10, 29, 0), b"September 03 08:10:29 2000 +03 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 24, (O_P3, 2000, 1, 5, 1, 2, 4, 0), b"Jan 05 01:02:04 2000 +03 \xF0\x9F\x98\x80"),
            (0, 24, (O_P3, 2000, 2, 29, 1, 2, 4, 0), b"Feb 29 01:02:04 2000 +03 \xF0\x9F\x98\x80"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "31", regex = "ALL", regex = "TEST"))]
    ERE_REGEX_DATETIME!(
        31,
        counter!(DP_KEY),
        concat!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSY,
        0, 35,
        CGN_MONTH, CGN_YEAR,
        &[
            (0, 26, (O_L, 2000, 9, 3, 8, 10, 29, 0), b"September 03 08:10:29 2000:hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 20, (O_L, 2000, 1, 2, 3, 4, 5, 0), b"Jan 02 03:04:05 2000|\xF0\x9F\x98\x80"),
            (0, 19, (O_L, 2000, 1, 5, 1, 2, 4, 0), b"Jan 5 01:02:04 2000 \xF0\x9F\x98\x80"),
            (0, 20, (O_L, 2000, 2, 29, 1, 2, 4, 0), b"Feb 29 01:02:04 2000 \xF0\x9F\x98\x80"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "32", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        32,
        counter!(DP_KEY),
        concat!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_TZZ, RP_NOALPHA),
        DfaU8,
        DTFSS_BdHMSZ,
        0, 35,
        CGN_MONTH, CGN_TZ,
        &[
            (0, 25, (O_PWT, YD, 9, 3, 8, 10, 29, 0), b"September 03 08:10:29 PWT hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 19, (O_PWT, YD, 1, 2, 3, 4, 5, 0), b"Jan 02 03:04:05 pwt \xF0\x9F\x98\x80"),
            (0, 18, (O_PWT, YD, 1, 2, 3, 4, 5, 0), b"Jan 2 03:04:05 PWT \xF0\x9F\x98\x80"),
            (0, 19, (O_PWT, YD, 2, 29, 1, 2, 4, 0), b"Feb 29 01:02:04 PWT \xF0\x9F\x98\x80"),
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
    #[cfg(any(regex = "33", regex = "ALL", regex = "TEST"))]
    ERE_REGEX_DATETIME!(
        33,
        counter!(DP_KEY),
        concat!("^", CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKSq),
        DfaU8,
        DTFSS_BdHMS,
        0, 22,
        CGN_MONTH, CGN_SECOND,
        &[
            (0, 21, (O_L, YD, 9, 3, 8, 10, 29, 0), b"September 03 08:10:29 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode"),
            (0, 15, (O_L, YD, 1, 2, 3, 4, 5, 0), b"Jan 02 03:04:05 1900 a\xF0\x9F\x98\x80"),
            (0, 15, (O_L, YD, 1, 2, 3, 4, 5, 0), b"Jan 02 03:04:05 1900 \xF0\x9F\x98\x80"),
            (0, 14, (O_L, YD, 1, 2, 3, 4, 5, 0), b"Jan 2 03:04:05 1900 \xF0\x9F\x98\x80"),
            (0, 19, (O_L, YD, 1, 3, 13, 47, 7, 0), b"January 03 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119"),
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
    #[cfg(any(regex = "34", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        34,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZz, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSYz,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (0, 30, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"mon Jun 28 2022 01:51:12 +1230"),
            (0, 31, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"tue. Jun 28 2022 01:51:12 +1230"),
            (0, 31, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"Mon. Jun 28 2022 01:51:12 +1230"),
            (0, 30, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"Mon Jun 28 2022 01:51:12 +1230"),
            (0, 31, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"Mon. Jun 28 2022 01:51:12 +1230"),
            (0, 31, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"Mon Jun. 28 2022 01:51:12 +1230"),
            (0, 32, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"Mon. Jun. 28 2022 01:51:12 +1230"),
            (0, 30, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), b"Wed Jun 02 2022 01:51:12 +1230"),
            (0, 29, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), b"thu Jun 2 2022 01:51:12 +1230"),
            (0, 33, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"Friday Jun 28 2022 01:51:12 +1230"),
            (0, 34, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"monday, Jun 28 2022 01:51:12 +1230"),
            (0, 30, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"Tue Jun 28 2022 01:51:12 +1230 FOOBAR"),
            (0, 31, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"Tue, Jun 28 2022 01:51:12 +1230"),
            (0, 35, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"Tuesday. Jun 28 2022 01:51:12 +1230 FOOBAR"),
            (0, 35, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"TUESDAY. Jun 28 2022 01:51:12 +1230 FOOBAR"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "35", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        35,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZzc, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSYzc,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (0, 31, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"WED Jun 28 2022 01:51:12 +01:30"),
            (0, 32, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"Wed, Jun 28 2022 01:51:12 +01:30"),
            (0, 32, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"wed. Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (0, 32, (O_P1_30, 2022, 6, 2, 1, 51, 12, 0), b"wed. Jun 02 2022 01:51:12 +01:30 FOOBAR"),
            (0, 31, (O_P1_30, 2022, 6, 2, 1, 51, 12, 0), b"wed. Jun 2 2022 01:51:12 +01:30 FOOBAR"),
            (0, 37, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"Wednesday Jun 28 2022 01:51:12 +01:30"),
            (0, 38, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"Wednesday, Jun 28 2022 01:51:12 +01:30"),
            (0, 31, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"thu Jun 28 2022 01:51:12 +01:30 FOOBAR"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "36", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        36,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZzp, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSYzp,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (0, 34, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"THURSDAY, Jun 28 2022 01:51:12 +01"),
            (0, 34, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"thursday, Jun 28 2022 01:51:12 +01"),
            (0, 29, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"fri. Jun 28 2022 01:51:12 +01 FOOBAR"),
            (0, 29, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"fri, Jun 28 2022 01:51:12 +01"),
            (0, 28, (O_P1, 2022, 6, 2, 1, 51, 12, 0), b"fri, Jun 2 2022 01:51:12 +01"),
            (0, 31, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"FRIDAY Jun 28 2022 01:51:12 +01 FOOBAR"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "37", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        37,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZZ, RP_NOALPHA),
        DfaU8,
        DTFSS_BdHMSYZ,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (0, 34, (O_WIT, 2022, 6, 28, 1, 51, 12, 0), b"Saturday, Jun 28 2022 01:51:12 WIT"),
            (0, 30, (O_WITA, 2022, 6, 28, 1, 51, 12, 0), b"SAT, Jun 28 2022 01:51:12 WITA:FOOBAR"),
            (0, 29, (O_WST, 2022, 6, 28, 1, 51, 12, 0), b"SAT. Jun 28 2022 01:51:12 WST FOOBAR"),
            (0, 29, (O_YAKT, 2022, 6, 28, 1, 51, 12, 0), b"sun Jun 28 2022 01:51:12 YAKT"),
            (0, 28, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), b"sun Jun 2 2022 01:51:12 YAKT"),
            (0, 32, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), b"sunday Jun 28 2022 01:51:12 YEKT FOOBAR"),
            (0, 32, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), b"sunday Jun 28 2022 01:51:12 yekt FOOBAR"),
            (0, 32, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), b"SUNDAY Jun 28 2022 01:51:12 YEKT FOOBAR"),
            (0, 33, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), b"SUNDAY, Jun 28 2022 01:51:12 YEKT FOOBAR"),
        ],
        line!(),
    ),
    // TODO: break out 37 into variations for DAYa3, DAYa3, CGP_TZZ_U, CGP_TZZ_L
    // ---------------------------------------------------------------------------------------------
    //
    // RFC 2822
    //
    // https://dencode.com/date/rfc2822 uses leading zero for day of month
    // https://www.rfc-editor.org/rfc/rfc2822.html#page-41 no leading zero for day of month
    //
    #[cfg(any(regex = "38", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        38,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa3, ",", RP_BLANK, CGP_DAYde, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_YEAR, RP_cq, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZz, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSYz,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (0, 31, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"Mon, 28 Jun 2022 01:51:12 +1230"),
            (0, 31, (O_M5, 2018, 10, 7, 22, 55, 18, 0), b"Sat, 07 Oct 2018 22:55:18 -0500 hello this datetime stamp from https://dencode.com/date/rfc2822"),
            (0, 30, (O_P2, 2003, 7, 1, 10, 52, 37, 0), b"Tue, 1 Jul 2003 10:52:37 +0200 from https://www.rfc-editor.org/rfc/rfc2822.html#page-41"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "39", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        39,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa3, ",", RP_BLANK, CGP_DAYde, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_YEAR, RP_cq, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZzc, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSYzc,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (0, 32, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"Mon, 28 Jun 2022 01:51:12 +01:30"),
            (0, 32, (O_M5, 2018, 10, 7, 22, 55, 18, 0), b"Sat, 07 Oct 2018 22:55:18 -05:00 hello this datetime stamp from https://dencode.com/date/rfc2822"),
            (0, 31, (O_P2, 2003, 7, 1, 10, 52, 37, 0), b"Tue, 1 Jul 2003 10:52:37 +02:00 from https://www.rfc-editor.org/rfc/rfc2822.html#page-41"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "40", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        40,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa3, ",", RP_BLANK, CGP_DAYde, RP_BLANK, CGP_MONTHb, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZZ, RP_NOALPHA),
        DfaU8,
        DTFSS_BdHMSYZ,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (0, 29, (O_WIT, 2022, 6, 28, 1, 51, 12, 0), b"Mon, 28 Jun 2022 01:51:12 WIT"),
            (0, 29, (O_EDT, 2018, 10, 7, 22, 55, 18, 0), b"Sat, 07 Oct 2018 22:55:18 EDT hello this datetime stamp from https://dencode.com/date/rfc2822"),
            (0, 29, (O_CAT, 2003, 7, 1, 10, 52, 37, 0), b"Tue, 1 Jul 2003 10:52:37  CAT from https://www.rfc-editor.org/rfc/rfc2822.html#page-41"),
        ],
        line!(),
    ),
    // RFC 2822 with leading field title "Date:"
    // taken from https://www.rfc-editor.org/rfc/rfc2822#appendix-A.1
    #[cfg(any(regex = "41", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        41,
        counter!(DP_KEY),
        concat!("^", RP_RFC2822_DATE, RP_BLANKq, CGP_DAYa3, ",", RP_BLANK, CGP_DAYde, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_YEAR, RP_cq, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZz, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSYz,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (6, 37, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"Date:	Mon, 28 Jun 2022 01:51:12 +1230"),
            (6, 37, (O_M5, 2018, 10, 7, 22, 55, 18, 0), b"DATE: Sat, 07 Oct 2018 22:55:18 -0500 hello this datetime stamp from https://dencode.com/date/rfc2822"),
            (5, 35, (O_P2, 2003, 7, 1, 10, 52, 37, 0), b"date:tue, 1 jul 2003 10:52:37 +0200 from https://www.rfc-editor.org/rfc/rfc2822.html#page-41"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "42", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        42,
        counter!(DP_KEY),
        concat!("^", RP_RFC2822_DATE, RP_BLANKq, CGP_DAYa3, ",", RP_BLANK, CGP_DAYde, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_YEAR, RP_cq, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZzc, RP_NODIGIT),
        DfaU8,
        DTFSS_BdHMSYzc,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (6, 38, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"Date:	Mon, 28 Jun 2022 01:51:12 +01:30"),
            (6, 38, (O_M5, 2018, 10, 7, 22, 55, 18, 0), b"DATE: Sat, 07 Oct 2018 22:55:18 -05:00 hello this datetime stamp from https://dencode.com/date/rfc2822"),
            (5, 36, (O_P2, 2003, 7, 1, 10, 52, 37, 0), b"date:tue, 1 jul 2003 10:52:37 +02:00 from https://www.rfc-editor.org/rfc/rfc2822.html#page-41"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "43", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        43,
        counter!(DP_KEY),
        concat!("^", RP_RFC2822_DATE, RP_BLANKq, CGP_DAYa3, ",", RP_BLANK, CGP_DAYde, RP_BLANK, CGP_MONTHb, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZZ, RP_NOALPHA),
        DfaU8,
        DTFSS_BdHMSYZ,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (6, 35, (O_WIT, 2022, 6, 28, 1, 51, 12, 0), b"Date:	Mon, 28 Jun 2022 01:51:12 WIT"),
            (6, 35, (O_EDT, 2018, 10, 7, 22, 55, 18, 0), b"DATE: Sat, 07 Oct 2018 22:55:18 EDT hello this datetime stamp from https://dencode.com/date/rfc2822"),
            (5, 34, (O_CAT, 2003, 7, 1, 10, 52, 37, 0), b"date:tue, 1 jul 2003 10:52:37  CAT from https://www.rfc-editor.org/rfc/rfc2822.html#page-41"),
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
    #[cfg(any(regex = "44", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        44,
        counter!(DP_KEY),
        concat!("^(start|Start|START|end|End|END)[- ]?(date|Date|DATE)", D_T, RP_BLANKSq, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMS,
        0, 35,
        CGN_YEAR, CGN_SECOND,
        &[
            (12, 32, (O_L, 2022, 7, 18, 19, 35, 1, 0), b"Start-Date: 2022-07-18  19:35:01\nCommandline: apt-get install -y gnupg2\nInstall: gnupg2:amd64 (2.2.27-3ubuntu2.1)\n"),
            (10, 30, (O_L, 2022, 7, 18, 19, 35, 2, 0), b"End-Date: 2022-07-18  19:35:02\n"),
            (10, 30, (O_L, 2022, 7, 18, 19, 35, 3, 0), b"End-Date: 2022-07-18  19:35:03"),
            (9, 29, (O_L, 2022, 7, 18, 19, 35, 4, 0), b"End-Date:2022-07-18  19:35:04"),
            (9, 28, (O_L, 2022, 7, 18, 19, 35, 5, 0), b"End Date:2022-07-18 19:35:05\n"),
            (9, 28, (O_L, 2022, 7, 18, 19, 35, 6, 0), b"End-Date 2022-07-18 19:35:06\n"),
            (10, 29, (O_L, 2022, 7, 18, 19, 35, 7, 0), b"END-DATE  2022-07-18 19:35:07 Foobar"),
            (10, 29, (O_L, 2022, 7, 18, 19, 35, 7, 0), b"END DATE		2022-07-18 19:35:07	Foobar"),
            (9, 28, (O_L, 2022, 7, 18, 19, 35, 7, 0), b"END-DATE	2022-07-18 19:35:07 Foobar"),
            (10, 29, (O_L, 2022, 7, 18, 19, 35, 7, 0), b"END-DATE:	2022-07-18 19:35:07 Foobar"),
            (9, 28, (O_L, 2022, 7, 18, 19, 35, 8, 0), b"end-date 2022-07-18T19:35:08 Foobar"),
            (14, 33, (O_L, 2022, 7, 18, 19, 35, 9, 0), b"START-DATE:   2022-07-18 19:35:09\nCommandline: apt-get install -y gnupg2\n"),
            (11, 30, (O_L, 2022, 7, 18, 19, 35, 9, 0), b"STARTDATE:	2022/07/18 19:35:09\nCommandline: apt-get install -y gnupg2\n"),
        ],
        line!(),
    ),
    /*
    add bracketed datetime offset fro beginning
    /mnt/c/Users/jtmmo/AppData/Local/Temp/dd_BackgroundDownload_20260617093422.log: Unicode text, UTF-8 text, with very long lines (1604), with CRLF line terminators

    [6aa0:0001][2026-06-17T09:34:22] Creating new ExperimentationService
    [6aa0:0001][2026-06-17T09:34:22] Telemetry property VS.ABExp.Flights : lazytoolboxinit;fwlargebuffer;refactoring;spmoretempsbtn1;asloff;keybindgoldbarext;asynccsproj;vsfricheditor;completionapi;4f604693:30775293
    [6aa0:0001][2026-06-17T09:34:22] UserActivityMonitor started.

     */
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
    #[cfg(any(regex = "45", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        45,
        counter!(DP_KEY),
        concat!(CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, ":", RP_BLANKe),
        FlatLockstepNfaU8,
        DTFSS_YmdHMS,
        0, 30,
        CGN_YEAR, CGN_SECOND,
        &[
            (0, 19, (O_L, 2017, 5, 14, 4, 0, 7, 0), b"2017-05-14 04-00-07:"),
            (0, 19, (O_L, 2017, 5, 14, 4, 0, 8, 0), b"2017-05-14 04-00-08: "),
            (0, 19, (O_L, 2017, 5, 14, 4, 0, 9, 0), b"2017-05-14 04-00-09: -------------------- report start"),
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
    #[cfg(any(regex = "46", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        46,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz, RP_RB),
        DfaU8, // 3m2s compile time
        //FlatLockstepNfaU8, // 30s
        DTFSS_YbdHMSz,
        0, 300, CGN_DAY, CGN_TZ,
        &[
            (19, 45, (O_P1, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172 - - [11/Oct/2022:00:10:26 +0100] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 45, (O_P1, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172 - - {11/oct/2022 00:10:26 +0100} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 40, (O_P1, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172	<11-oct-2022 00:10:26+0100>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 43, (O_M8, 2020, 3, 7, 6, 30, 43, 0), br#"192.168.0.8 - - [07/Mar/2020:06:30:43 -0800] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "47", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        47,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZZ, RP_RB),
        //DfaU8, // 30+m compile time
        FlatLockstepNfaU8,
        DTFSS_YbdHMSZ,
        0, 300,
        CGN_DAY, CGN_TZ,
        &[
            (19, 44, (O_CHUT, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172 - - [11/Oct/2022:00:10:26 CHUT] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 44, (O_CHUT, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172 - - {11/oct/2022 00:10:26 CHUT} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 40, (O_CHUT, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172	<11-oct-2022 00:10:26 CHUT>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 32, (O_Z, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172	<11OCT2022T001026Z>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 42, (O_CHUT, 2020, 3, 7, 6, 30, 43, 0), br#"192.168.0.8 - - [07/Mar/2020:06:30:43 CHUT] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "48", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        48,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzc, RP_RB),
        DfaU8,
        DTFSS_YbdHMSzc,
        0, 300,
        CGN_DAY, CGN_TZ,
        &[
            (19, 46, (O_P1, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172 - - [11/Oct/2022:00:10:26 +01:00] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 46, (O_P1, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172 - - {11/oct/2022 00:10:26 +01:00} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 41, (O_P1, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172	<11-oct-2022 00:10:26+01:00>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 44, (O_M8, 2020, 3, 7, 6, 30, 43, 0), br#"192.168.0.8 - - [07/Mar/2020:06:30:43 -08:00] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "49", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        49,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzp, RP_RB),
        DfaU8,
        DTFSS_YbdHMSzp,
        0, 300,
        CGN_DAY, CGN_TZ,
        &[
            (19, 43, (O_P1, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172 - - [11/Oct/2022:00:10:26 +01] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 43, (O_P1, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172 - - {11/oct/2022 00:10:26 +01} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 38, (O_P1, 2022, 10, 11, 0, 10, 26, 0), br#"192.168.0.172	<11-oct-2022 00:10:26+01>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 41, (O_M8, 2020, 3, 7, 6, 30, 43, 0), br#"192.168.0.8 - - [07/Mar/2020:06:30:43 -08] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "50", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        50,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, RP_RB),
        //DfaU8, // 2m15s
        FlatLockstepNfaU8,
        DTFSS_YbdHMS,
        0, 300, CGN_DAY, CGN_SECOND,
        &[
            // Flask web server default log format
            (15, 35, (O_L, 2024, 3, 22, 15, 11, 28, 0), br#"127.0.0.1 - - [22/Mar/2024 15:11:28] "GET / HTTP/1.1" 200 -"#),
        ],
        line!(),
    ),
    // prior patterns with fractionals 1-9
    #[cfg(any(regex = "51", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        51,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZz, RP_RB),
        DfaU8,
        DTFSS_bdHMSYfz,
        0, 300,
        CGN_DAY, CGN_TZ,
        &[
            (19, 49, (O_P1, 2022, 10, 11, 0, 10, 26, 123000000), br#"192.168.0.172 - - [11/Oct/2022:00:10:26.123 +0100] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 48, (O_P1, 2022, 10, 11, 0, 10, 26, 110000000), br#"192.168.0.172 - - {11/oct/2022 00:10:26.11 +0100} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 42, (O_P1, 2022, 10, 11, 0, 10, 26, 100000000), br#"192.168.0.172	<11-oct-2022 00:10:26.1+0100>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 53, (O_M8, 2020, 3, 7, 6, 30, 43, 123456789), br#"192.168.0.8 - - [07/Mar/2020:06:30:43.123456789 -0800] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "52", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        52,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzc, RP_RB),
        DfaU8,
        DTFSS_bdHMSYfzc,
        0, 300,
        CGN_DAY, CGN_TZ,
        &[
            (19, 50, (O_P1, 2022, 10, 11, 0, 10, 26, 123000000), br#"192.168.0.172 - - [11/Oct/2022:00:10:26.123 +01:00] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 49, (O_P1, 2022, 10, 11, 0, 10, 26, 110000000), br#"192.168.0.172 - - {11/oct/2022 00:10:26.11 +01:00} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 43, (O_P1, 2022, 10, 11, 0, 10, 26, 100000000), br#"192.168.0.172	<11-oct-2022 00:10:26.1+01:00>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 54, (O_M8, 2020, 3, 7, 6, 30, 43, 123456789), br#"192.168.0.8 - - [07/Mar/2020:06:30:43.123456789 -08:00] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "53", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        53,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzp, RP_RB),
        DfaU8,
        DTFSS_bdHMSYfzp,
        0, 300,
        CGN_DAY, CGN_TZ,
        &[
            (19, 47, (O_P1, 2022, 10, 11, 0, 10, 26, 123000000), br#"192.168.0.172 - - [11/Oct/2022:00:10:26.123 +01] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 46, (O_P1, 2022, 10, 11, 0, 10, 26, 110000000), br#"192.168.0.172 - - {11/oct/2022 00:10:26.11 +01} "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 40, (O_P1, 2022, 10, 11, 0, 10, 26, 100000000), br#"192.168.0.172	<11-oct-2022 00:10:26.1+01>	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 51, (O_M8, 2020, 3, 7, 6, 30, 43, 123456789), br#"192.168.0.8 - - [07/Mar/2020:06:30:43.123456789 -08] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "54", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        54,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKSq, RP_RB),
        DfaU8,
        DTFSS_bdHMSYf,
        0, 300,
        CGN_DAY, CGN_FRACTIONAL,
        &[
            (19, 43, (O_L, 2022, 10, 11, 0, 10, 26, 123000000), br#"192.168.0.172 - - [11/Oct/2022:00:10:26.123] "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (19, 42, (O_L, 2022, 10, 11, 0, 10, 26, 110000000), br#"192.168.0.172 - - {11/oct/2022 00:10:26.11 } "GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (15, 37, (O_L, 2022, 10, 11, 0, 10, 26, 100000000), br#"192.168.0.172	<11-oct-2022 00:10:26.1        >	"GET / HTTP/1.0" 200 3343 "-" "Lynx/2.9.0dev.10 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.7.1""#),
            (17, 47, (O_L, 2020, 3, 7, 6, 30, 43, 123456789), br#"192.168.0.8 - - [07/Mar/2020:06:30:43.123456789] "GET /path2/feed.rss HTTP/1.1" 404 178 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36 OPR/66.0.3515.72""#),
        ],
        line!(),
    ),
    // prior patterns with numeric month, no timezone
    //
    // from file `./logs/Windows11Pro/setupact.log`
    //
    //      [02/21/2023 07:07.05.259] WudfCoInstaller: ReadWdfSection: Checking WdfSection [Basic_Install.Wdf]
    //
    #[cfg(any(regex = "55", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        55,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_MONTHm, D_Dq, CGP_DAYde, D_Dq, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, r"[:\.]?", CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_RB),
        DfaU8,
        DTFSS_YmdHMSf,
        0, 300,
        CGN_MONTH, CGN_FRACTIONAL,
        &[
            (1, 24, (O_L, 2023, 2, 21, 7, 7, 5, 262000000), b"[02/21/2023 07:07.05.262] WudfCoInstaller: Configuring UMDF Service WpdFs.\n\n"),
            (18, 41, (O_L, 2023, 2, 21, 7, 7, 5, 263000000), b"WudfCoInstaller: [02/21/2023 07:07.05.263] ImpersonationLevel set to 2\n\n"),
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
    #[cfg(any(regex = "56", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        56,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_DAYa, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANK, CGP_YEAR, RP_RB),
        DfaU8,
        DTFSS_bdHMSYf,
        0, 300,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (1, 32, (O_L, 2022, 10, 10, 23, 56, 29, 204202000), b"[Mon Oct 10 23:56:29.204202 2022] [mpm_event:notice] [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations"),
            (20, 51, (O_L, 2022, 10, 10, 23, 56, 29, 204202000), b"[mpm_event:notice]	<Mon Oct 10	23:56:29.204202 2022> [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations"),
            (20, 48, (O_L, 2022, 10, 30, 23, 56, 29, 204000000), b"[mpm_event:notice]	<sun Oct 30	23:56:29.204 2022> [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations"),
            (20, 54, (O_L, 2022, 10, 5, 23, 56, 29, 204948193), b"[mpm_event:notice]	<WED oct 05	23:56:29.204948193 2022> [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "57", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        57,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_DAYa, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_RB),
        DfaU8,
        DTFSS_YbdHMS,
        0, 300,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (1, 25, (O_L, 2022, 10, 10, 23, 56, 29, 0), b"[Mon Oct 10 23:56:29 2022] [mpm_event:notice] [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations"),
            (20, 44, (O_L, 2022, 10, 10, 23, 56, 29, 0), b"[mpm_event:notice]	(Mon Oct 10	23:56:29 2022) [pid 11709:tid 140582486756672] AH00489: Apache/2.4.54 (Debian) configured -- resuming normal operations"),
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
    #[cfg(any(regex = "58", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        58,
        counter!(DP_KEY),
        concat!("^", CGP_DAYde, D_Dq, CGP_MONTHb, D_Dq, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL3, RP_NOALNUMpm),
        DfaU8,
        DTFSS_bdHMSYf,
        0, 30,
        CGN_DAY, CGN_FRACTIONAL,
        &[
            (0, 24, (O_L, 2023, 2, 8, 12, 13, 9, 827000000), br#"08-Feb-2023 12:13:09.827 INFO [main] org.apache.coyote.AbstractProtocol.init Initializing ProtocolHandler ["http-nio2-0.0.0.0-8080"]"#),
            (0, 24, (O_L, 2023, 2, 8, 12, 13, 20, 63000000), b"08-Feb-2023 12:13:20.063 INFO [localhost-startStop-1] org.apache.jasper.servlet.TldScanner.scanJars At least one JAR was scanned for TLDs yet contained no TLDs. Enable debug logging for this logger for a complete list of JARs that were scanned but no TLDs were found in them. Skipping unneeded JARs during scanning can improve startup time and JSP compilation time.\nSLF4J: Class path contains multiple SLF4J bindings."),
        ],
        line!(),
    ),
    #[cfg(any(regex = "59", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        59,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, "(START|END|Start|End|start|end)", RP_BLANKSq, "[:]?", RP_BLANKSq, CGP_YEAR, D_Deq, CGP_MONTHms, D_Deq, CGP_DAYde, D_DHcdqu, CGP_HOUR_sd, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NODIGIT),
        // DfaU8, // fails to build
        FlatLockstepNfaU8,
        DTFSS_YmsdkMS,
        0, 1024,
        CGN_YEAR, CGN_SECOND,
        &[
            // from `C:/Windows/Performance/WinSAT/winsat.log`
            // a datetime format with redundant `AM` and `PM`, see Issue #64
            // with single-digit month and single-month hour
            (50, 67, (O_L, 2023, 2, 22, 4, 5, 7, 0), br"59805625 (9340) - exe\logging.cpp:0841: --- START 2023\2\22 4:05:07 AM ---1"),
            (50, 67, (O_L, 2023, 2, 22, 1, 5, 7, 0), br"59805625 (9340) - exe\logging.cpp:0841: --- START 2023\2\22 1:05:07 AM ---2"),
            (50, 68, (O_L, 2023, 2, 22, 12, 5, 7, 0), br"59805625 (9340) - exe\logging.cpp:0841: --- Start 2023\2\22 12:05:07 AM ---3"),
            (50, 69, (O_L, 2023, 11, 22, 12, 5, 7, 0), br"59805625 (9340) - exe\logging.cpp:0841: --- start 2023\11\22 12:05:07 AM ---4"),
            (48, 67, (O_L, 2023, 11, 22, 12, 5, 7, 0), br"59805625 (9340) - exe\logging.cpp:0841: --- End 2023\11\22 12:05:07 AM ---5"),
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
    #[cfg(any(regex = "60", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        60,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, "[:]?", RP_BLANKSq, CGP_DAYa, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzc, RP_NODIGIT),
        DfaU8,
        DTFSS_YbdHMSzc,
        0, 60,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (7, 38, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"TRACE:	Sat Jan 01 2000 08:45:55 +09:00 TRACE: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 39, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"trace0:	sat jan 01 2000 08:45:55 +09:00 trace0: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (9, 40, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"TRACE1:		Sat Jan 01 2000 08:45:55 +09:00 TRACE1: \xE2\x87\xA5 \xD72\xE2\x80\xBC"),
            (8, 39, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"TRACE2:	Sat Jan 01 2000 08:45:55 +09:00 TRACE2: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (7, 38, (O_P9, 2000, 1, 2, 21, 0, 0, 0), b"DEBUG: Sun Jan 02 2000 21:00:00 +09:00 DEBUG:\xE2\x80\xBC"),
            (7, 38, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"debug: sat jan 01 2000 08:45:55 +09:00 debug:\xE2\x80\xBC"),
            (7, 38, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"DEBUG0 Sat Jan 01 2000 08:45:55 +09:00 debug0\xE2\x80\xBC"),
            (8, 39, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"DEBUG9: Sat Jan 01 2000 08:45:55 +09:00 debug9:\xE2\x80\xBC"),
            (6, 37, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"INFO: Sat Jan 01 2000 08:45:55 -09:00 info:\xE2\x80\xBC"),
            (7, 38, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"INFO2: Sat Jan 01 2000 08:45:55 -09:00 info2:\xE2\x80\xBC"),
            (9, 40, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"warning: Sat Jan 01 2000 08:45:55 -09:00 warning:\xE2\x80\xBC"),
            (8, 39, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"warning sat jan 01 2000 08:45:55 -09:00 warning\xE2\x80\xBC"),
            (8, 39, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"warning	Sat Jan 01 2000 08:45:55 -09:00 warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 39, (O_M9, 2000, 1, 3, 23, 30, 59, 0), b"WARNING	MON JAN 03 2000 23:30:59 -09:00 warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 40, (O_M9, 2000, 1, 3, 23, 30, 59, 0), b"WARNING	MON. JAN 03 2000 23:30:59 -09:00 warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 40, (O_M9, 2000, 1, 3, 23, 30, 59, 0), b"WARNING	MON JAN. 03 2000 23:30:59 -09:00 warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 41, (O_M9, 2000, 1, 3, 23, 30, 59, 0), b"WARNING	MON. JAN. 03 2000 23:30:59 -09:00 warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (9, 40, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"warning		Sat Jan 01 2000 08:45:55 -09:00 warning \xE2\x87\xA5 \xD72\xE2\x80\xBC"),
            (6, 37, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"WARN: SAT JAN 01 2000 08:45:55 -09:00 warn:\xE2\x80\xBC"),
            (7, 38, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"ERROR: Sat Jan 01 2000 08:45:55 -09:00 error:\xE2\x80\xBC"),
            (5, 36, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"ERR: Sat Jan 01 2000 08:45:55 -09:00 err:\xE2\x80\xBC"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "61", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        61,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, "[:]?", RP_BLANKSq, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        DfaU8,
        DTFSS_YbdHMSz,
        0, 60,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (7, 37, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"TRACE:	Sat Jan 01 2000 08:45:55 +0900 TRACE: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 38, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"trace0:	sat jan 01 2000 08:45:55 +0900	trace0: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (9, 39, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"TRACE1:		Sat Jan 01 2000 08:45:55 +0900		TRACE1: \xE2\x87\xA5 \xD72\xE2\x80\xBC"),
            (8, 38, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"TRACE2:	Sat Jan 01 2000 08:45:55 +0900	TRACE2: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (7, 37, (O_P9, 2000, 1, 2, 21, 0, 0, 0), b"DEBUG: Sun Jan 02 2000 21:00:00 +0900 DEBUG:\xE2\x80\xBC"),
            (7, 37, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"debug: sat jan 01 2000 08:45:55 +0900 debug:\xE2\x80\xBC"),
            (7, 37, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"DEBUG0 Sat Jan 01 2000 08:45:55 +0900 debug0\xE2\x80\xBC"),
            (8, 38, (O_P9, 2000, 1, 1, 8, 45, 55, 0), b"DEBUG9: Sat Jan 01 2000 08:45:55 +0900 debug9:\xE2\x80\xBC"),
            (6, 36, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"INFO: Sat Jan 01 2000 08:45:55 -0900 info:\xE2\x80\xBC"),
            (7, 37, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"INFO2: Sat Jan 01 2000 08:45:55 -0900 info2:\xE2\x80\xBC"),
            (9, 39, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"warning: Sat Jan 01 2000 08:45:55 -0900 warning:\xE2\x80\xBC"),
            (9, 40, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"warning: Sat. Jan 01 2000 08:45:55 -0900 warning:\xE2\x80\xBC"),
            (9, 40, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"warning: Sat Jan. 01 2000 08:45:55 -0900 warning:\xE2\x80\xBC"),
            (9, 41, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"warning: Sat. Jan. 01 2000 08:45:55 -0900 warning:\xE2\x80\xBC"),
            (8, 38, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"warning sat jan 01 2000 08:45:55 -0900 warning\xE2\x80\xBC"),
            (8, 38, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"warning	Sat Jan 01 2000 08:45:55 -0900	warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 38, (O_M9, 2000, 1, 3, 23, 30, 59, 0), b"WARNING	MON JAN 03 2000 23:30:59 -0900	warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (9, 39, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"warning		Sat Jan 01 2000 08:45:55 -0900		warning \xE2\x87\xA5 \xD72\xE2\x80\xBC"),
            (6, 36, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"WARN: SAT JAN 01 2000 08:45:55 -0900 warn:\xE2\x80\xBC"),
            (7, 37, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"ERROR: Sat Jan 01 2000 08:45:55 -0900 error:\xE2\x80\xBC"),
            (5, 35, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"ERR: Sat Jan 01 2000 08:45:55 -0900 err:\xE2\x80\xBC"),
            (5, 39, (O_M9, 2000, 1, 1, 8, 45, 55, 0), b"ERR: Sat January 01 2000 08:45:55 -0900 err:\xE2\x80\xBC"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "62", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        62,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, "[:]?", RP_BLANKSq, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzp, RP_NODIGIT),
        DfaU8,
        DTFSS_YbdHMSzp,
        0, 60,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (7, 35, (O_P9, 2000, 1, 31, 8, 45, 55, 0), b"TRACE:	Sat Jan 31 2000 08:45:55 +09 TRACE: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 36, (O_P9, 2000, 1, 31, 8, 45, 55, 0), b"trace0:	sat jan 31 2000 08:45:55 +09	trace0: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (9, 37, (O_P9, 2000, 1, 31, 8, 45, 55, 0), b"TRACE1:		Sat Jan 31 2000 08:45:55 +09		TRACE1: \xE2\x87\xA5 \xD72\xE2\x80\xBC"),
            (8, 36, (O_P9, 2000, 1, 31, 8, 45, 55, 0), b"TRACE2:	Sat Jan 31 2000 08:45:55 +09	TRACE2: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (7, 35, (O_P9, 2000, 1, 2, 21, 0, 0, 0), b"DEBUG: Sun Jan 02 2000 21:00:00 +09 DEBUG:\xE2\x80\xBC"),
            (7, 35, (O_P9, 2000, 1, 31, 8, 45, 55, 0), b"debug: sat jan 31 2000 08:45:55 +09 debug:\xE2\x80\xBC"),
            (7, 35, (O_P9, 2000, 1, 31, 8, 45, 55, 0), b"DEBUG0 Sat Jan 31 2000 08:45:55 +09 debug0\xE2\x80\xBC"),
            (8, 36, (O_P9, 2000, 1, 31, 8, 45, 55, 0), b"DEBUG9: Sat Jan 31 2000 08:45:55 +09 debug9:\xE2\x80\xBC"),
            (6, 34, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"INFO: Sat Jan 31 2000 08:45:55 -09 info:\xE2\x80\xBC"),
            (7, 35, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"INFO2: Sat Jan 31 2000 08:45:55 -09 info2:\xE2\x80\xBC"),
            (9, 37, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"warning: Sat Jan 31 2000 08:45:55 -09 warning:\xE2\x80\xBC"),
            (8, 36, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"warning sat jan 31 2000 08:45:55 -09 warning\xE2\x80\xBC"),
            (8, 37, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"warning sat. jan 31 2000 08:45:55 -09 warning\xE2\x80\xBC"),
            (8, 37, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"warning sat jan. 31 2000 08:45:55 -09 warning\xE2\x80\xBC"),
            (8, 38, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"warning sat. jan. 31 2000 08:45:55 -09 warning\xE2\x80\xBC"),
            (8, 36, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"warning	Sat Jan 31 2000 08:45:55 -09	warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 36, (O_M9, 2000, 1, 3, 23, 30, 59, 0), b"WARNING	MON JAN 03 2000 23:30:59 -09	warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (9, 37, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"warning		Sat Jan 31 2000 08:45:55 -09		warning \xE2\x87\xA5 \xD72\xE2\x80\xBC"),
            (6, 34, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"WARN: SAT JAN 31 2000 08:45:55 -09 warn:\xE2\x80\xBC"),
            (7, 35, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"ERROR: Sat Jan 31 2000 08:45:55 -09 error:\xE2\x80\xBC"),
            (5, 33, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"ERR: sat jan 31 2000 08:45:55 -09 err:\xE2\x80\xBC"),
            (4, 32, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"err SAT JAN 31 2000 08:45:55 -09 err:\xE2\x80\xBC"),
            (4, 36, (O_M9, 2000, 1, 31, 8, 45, 55, 0), b"err SAT JANUARY 31 2000 08:45:55 -09 err:\xE2\x80\xBC"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "63", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        63,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, "[:]?", RP_BLANKSq, CGP_DAYa3, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZZ_U, RP_NOALPHA), // reduced
        DfaU8,
        DTFSS_YbdHMSZ,
        0, 65,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (7, 37, (O_PST, 2000, 1, 31, 8, 45, 55, 0), b"TRACE:	Sat. Jan. 31 2000 08:45:55 PST"),
            (7, 35, (O_PST, 2000, 1, 31, 8, 45, 55, 0), b"TRACE:	Sat Jan 31 2000 08:45:55 PST TRACE: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 36, (O_PET, 2000, 1, 31, 8, 45, 55, 0), b"trace0:	sat jan 31 2000 08:45:55 PET	trace0: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (9, 38, (O_PETT, 2000, 1, 31, 8, 45, 55, 0), b"TRACE1:		Sat Jan 31 2000 08:45:55 PETT	TRACE1: \xE2\x87\xA5 \xD72\xE2\x80\xBC"),
            (8, 37, (O_WITA, 2000, 1, 31, 8, 45, 55, 0), b"TRACE2:	Sat Jan 31 2000 08:45:55 WITA	TRACE2: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (7, 36, (O_WITA, 2000, 1, 2, 21, 45, 55, 0), b"DEBUG: Sun Jan 02 2000 21:45:55 WITA DEBUG:\xE2\x80\xBC"),
            (7, 36, (O_WITA, 2000, 1, 31, 8, 45, 55, 0), b"debug: sat jan 31 2000 08:45:55 WITA debug:\xE2\x80\xBC"),
            (7, 36, (O_WITA, 2000, 1, 31, 8, 45, 55, 0), b"DEBUG0 Sat Jan 31 2000 08:45:55 WITA debug0\xE2\x80\xBC"),
            (8, 37, (O_WITA, 2000, 1, 31, 8, 45, 55, 0), b"DEBUG9: Sat Jan 31 2000 08:45:55 WITA debug9:\xE2\x80\xBC"),
            (6, 35, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"INFO: Sat Jan 31 2000 08:45:55 PONT info:\xE2\x80\xBC"),
            (7, 36, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"INFO2: Sat Jan 31 2000 08:45:55 PONT info2:\xE2\x80\xBC"),
            (9, 38, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"warning: Sat Jan 31 2000 08:45:55 PONT warning:\xE2\x80\xBC"),
            (9, 39, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"warning: Sat. Jan 31 2000 08:45:55 PONT warning:\xE2\x80\xBC"),
            (9, 39, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"warning: Sat Jan. 31 2000 08:45:55 PONT warning:\xE2\x80\xBC"),
            (9, 40, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"warning: Sat. Jan. 31 2000 08:45:55 PONT warning:\xE2\x80\xBC"),
            (8, 37, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"warning sat jan 31 2000 08:45:55 PONT warning\xE2\x80\xBC"),
            (8, 37, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"warning	Sat Jan 31 2000 08:45:55 PONT	warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 37, (O_PONT, 2000, 1, 3, 23, 30, 59, 0), b"WARNING	MON JAN 03 2000 23:30:59 PONT		warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (9, 38, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"warning		Sat Jan 31 2000 08:45:55 PONT	warning \xE2\x87\xA5 \xD72\xE2\x80\xBC"),
            (6, 35, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"WARN: SAT JAN 31 2000 08:45:55 PONT:warn:\xE2\x80\xBC"),
            (7, 36, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"ERROR: SAT jan 31 2000 08:45:55 PONT|error:\xE2\x80\xBC"),
            (5, 34, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"ERR: sat Jan 31 2000 08:45:55 PONT err:\xE2\x80\xBC"),
            (5, 34, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), b"ERR: Sat jan 31 2000 08:45:55 PONT err:\xE2\x80\xBC"),
            //(5, 38, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "ERR: Sat january 31 2000 08:45:55 PONT err:\xE2\x80\xBC"),
            //(5, 37, (O_PONT, 2000, 1, 31, 8, 45, 55, 0), "ERR: Sat january 31 2000 08:45:55PONT err:\xE2\x80\xBC"),
        ],
        line!(),
    ),
    // TODO: break out into CFG_DAYa, CFG_DAYa3, CFG_MONTHb, CFG_MONTHBb, CGP_TZZ_U, CGP_TZZ_L
    #[cfg(any(regex = "64", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        64,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, "[:]?", RP_BLANKSq, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NODIGIT),
        DfaU8,
        DTFSS_YbdHMS,
        0, 60,
        CGN_DAY_IGNORE, CGN_SECOND,
        &[
            (7, 31, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"TRACE:	Sat Jan 31 2000 08:45:55 TRACE: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 32, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"trace0:	sat jan 31 2000 08:45:55	trace0: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (9, 33, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"TRACE1:		Sat Jan 31 2000 08:45:55	TRACE1: \xE2\x87\xA5 \xD72\xE2\x80\xBC"),
            (8, 32, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"TRACE2:	Sat Jan 31 2000 08:45:55	TRACE2: \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (7, 31, (O_L, 2000, 1, 2, 21, 0, 0, 0), b"DEBUG: Sun Jan 02 2000 21:00:00 DEBUG:\xE2\x80\xBC"),
            (7, 31, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"debug: sat jan 31 2000 08:45:55 debug:\xE2\x80\xBC"),
            (7, 31, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"DEBUG0 Sat Jan 31 2000 08:45:55 debug0\xE2\x80\xBC"),
            (8, 32, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"DEBUG9: Sat Jan 31 2000 08:45:55 debug9:\xE2\x80\xBC"),
            (6, 30, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"INFO: Sat Jan 31 2000 08:45:55 info:\xE2\x80\xBC"),
            (7, 31, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"INFO2: Sat Jan 31 2000 08:45:55 info2:\xE2\x80\xBC"),
            (9, 33, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"warning: Sat Jan 31 2000 08:45:55 -09:00 warning:\xE2\x80\xBC"),
            (8, 32, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"warning sat jan 31 2000 08:45:55 warning\xE2\x80\xBC"),
            (8, 32, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"warning	Sat Jan 31 2000 08:45:55	warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 32, (O_L, 2000, 1, 3, 23, 30, 59, 0), b"WARNING	MON JAN 03 2000 23:30:59	warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (8, 31, (O_L, 2000, 1, 3, 23, 30, 59, 0), b"WARNING	MON JAN 3 2000 23:30:59	warning \xE2\x87\xA5 \xD71\xE2\x80\xBC"),
            (9, 33, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"warning		Sat Jan 31 2000 08:45:55		warning \xE2\x87\xA5 \xD72\xE2\x80\xBC"),
            (6, 30, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"WARN: SAT JAN 31 2000 08:45:55 warn:\xE2\x80\xBC"),
            (7, 31, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"ERROR: Sat Jan 31 2000 08:45:55 error:\xE2\x80\xBC"),
            (5, 29, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"ERR: Sat Jan 31 2000 08:45:55 err:\xE2\x80\xBC"),
            (5, 29, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"ERR: Sat JAN 31 2000 08:45:55 err:\xE2\x80\xBC"),
            (4, 32, (O_L, 2000, 1, 31, 8, 45, 55, 0), b"ERR Sat JANUARY 31 2000 08:45:55 err:\xE2\x80\xBC"),
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
    #[cfg(any(regex = "65", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        65,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, "[:]?", RP_ANYp, RP_BLANK, CGP_DAYa, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZzc, RP_NODIGIT),
        DfaU8,
        DTFSS_YbdHMSzc,
        0, 120,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (22, 53, (O_M7, 2020, 2, 27, 0, 33, 59, 0), b"ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 -07:00: called for pid 8581, signal 24, core limit 0, dump mode 1"),
            (27, 58, (O_M330, 2022, 8, 13, 8, 48, 3, 0), br#"ERROR: apport (pid 529343) Sat Aug 13 08:48:03 2022 -03:30: executable: [s4] (command line "./target/release/s4 -s -wp /dev")"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "66", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        66,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, "[:]?", RP_ANYp, RP_BLANK, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZz, RP_NODIGIT),
        //DfaU8, // 11m16s build time
        FlatLockstepNfaU8,
        DTFSS_YbdHMSz,
        0, 120,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (22, 52, (O_M7, 2020, 2, 27, 0, 33, 59, 0), b"ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 -0700: called for pid 8581, signal 24, core limit 0, dump mode 1"),
            (27, 57, (O_M330, 2022, 8, 13, 8, 48, 3, 0), br#"ERROR: apport (pid 529343) Sat Aug 13 08:48:03 2022 -0330: executable: [s4] (command line "./target/release/s4 -s -wp /dev")"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "67", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        67,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, "[:]?", RP_ANYp, RP_BLANK, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZzp, RP_NODIGIT),
        //DfaU8, // 11m12s build time
        FlatLockstepNfaU8,
        DTFSS_YbdHMSzp,
        0, 120,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (22, 50, (O_M7, 2020, 2, 27, 0, 33, 59, 0), b"ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 -07: called for pid 8581, signal 24, core limit 0, dump mode 1"),
            (27, 55, (O_M3, 2022, 8, 13, 8, 48, 3, 0), br#"ERROR: apport (pid 529343) Sat Aug 13 08:48:03 2022 -03: executable: [s4] (command line "./target/release/s4 -s -wp /dev")"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "68", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        68,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, "[:]?", RP_ANYp, RP_BLANK, CGP_DAYa, RP_BLANK, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_BLANKS, CGP_TZZ_U, RP_NOALPHA), // reduced
        //DfaU8, // 11m12s build time
        FlatLockstepNfaU8,
        DTFSS_YbdHMSZ,
        0, 120,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (22, 51, (O_ALMT, 2020, 2, 27, 0, 33, 59, 0), b"ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 ALMT called for pid 8581, signal 24, core limit 0, dump mode 1"),
            (27, 56, (O_ALMT, 2022, 8, 13, 8, 48, 3, 0), br#"ERROR: apport (pid 529343) Sat Aug 13 08:48:03 2022 ALMT: executable: [s4] (command line "./target/release/s4 -s -wp /dev")"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "69", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        69,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, "[:]?", RP_ANYp, RP_BLANK, CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKS, CGP_YEAR, RP_NOALNUM),
        //DfaU8, // 9m8s build time
        FlatLockstepNfaU8,
        DTFSS_YbdHMS,
        0, 120,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (22, 46, (O_L, 2020, 2, 27, 0, 33, 59, 0), b"ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020 called for pid 8581, signal 24, core limit 0, dump mode 1"),
            (27, 51, (O_L, 2022, 8, 13, 8, 48, 3, 0), br#"ERROR: apport (pid 529343) Sat Aug 13 08:48:03 2022: executable: [s4] (command line "./target/release/s4 -s -wp /dev")"#),
            (25, 49, (O_L, 2020, 2, 20, 0, 59, 59, 0), br#"ERROR: apport (pid 9359) Thu Feb 20 00:59:59 2020: executable: /usr/lib/firefox/firefox (command line "/usr/lib/firefox/firefox"#),
            (27, 51, (O_L, 2023, 1, 8, 12, 53, 11, 0), b"ERROR: apport (pid 150689) Sun Jan  8 12:53:11 2023: called for pid 150672, signal 6, core limit 0, dump mode 1\n"),
        ],
        line!(),
    ),
    // TODO: add `RP_LEVELS` regex for lines like:
    //       <Notice>: 2024-03-24 19:46:10.665578 (pid/5566 [diskutil]) Service stub created for com.apple.audio.SandboxHelper
    //       [INFO]	2024-03-24 19:46:10.665578	(pid/5566 [diskutil])	Service stub created for com.apple.audio.SandboxHelper
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
    #[cfg(any(regex = "70", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        70,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMSfz,
        0, 50,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 29, (O_M11, 2000, 1, 2, 0, 0, 2, 123000000), b"2000/01/02 00:00:02.123 -1100 a"),

        ],
        line!(),
    ),
    #[cfg(any(regex = "71", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        71,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzc, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMSfzc,
        0, 50,
        CGN_YEAR, CGN_TZ,        &[(0, 33, (O_M1130, 2000, 1, 3, 0, 0, 3, 123456000), b"2000/01/03 00:00:03.123456 -11:30 ab")],
        line!(),
    ),
    #[cfg(any(regex = "72", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        72,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzp, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMSfzp,
        0, 50,
        CGN_YEAR, CGN_TZ,        &[(0, 33, (O_M11, 2000, 1, 4, 0, 0, 4, 123456789), b"2000/01/04 00:00:04,123456789 -11 abc")],
        line!(),
    ),
    #[cfg(any(regex = "73", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        73,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZZ, RP_NOALPHA),
        DfaU8,
        DTFSS_YmdHMSfZ,
        0, 50,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 34, (O_PETT, 2000, 1, 5, 0, 0, 5, 123456789), b"2000/01/05 00:00:05.123456789 PETT abcd"),
            (0, 33, (O_PETT, 2000, 1, 5, 0, 0, 5, 123456789), b"2000/01/05 00:00:05.123456789PETT abcd"),
            (0, 33, (O_PETT, 2000, 1, 5, 0, 0, 5, 123456789), b"2000/01/05 00:00:05.123456789PETT:abcd"),
            (0, 33, (O_PETT, 2000, 1, 5, 0, 0, 5, 123456789), b"2000/01/05 00:00:05.123456789PETT|abcd"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "74", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        74,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMSf,
        0, 50,
        CGN_YEAR, CGN_FRACTIONAL,        &[(0, 29, (O_L, 2020, 1, 6, 0, 0, 26, 123456789), b"2020-01-06 00:00:26.123456789 abcdefg")],
        line!(),
    ),
    //
    #[cfg(any(regex = "75", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        75,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMSz,
        0, 50,
        CGN_YEAR, CGN_TZ,        &[(0, 25, (O_M11, 2000, 1, 7, 0, 0, 2, 0), b"2000/01/07T00:00:02 -1100 abcdefgh")],
        line!(),
    ),
    #[cfg(any(regex = "76", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        76,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzc, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMSzc,
        0, 50,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 26, (O_M1130, 2000, 1, 8, 0, 0, 3, 0), b"2000-01-08-00:00:03 -11:30 abcdefghi"),
            // ISO 8601, time extended format
            (0, 25, (O_M1, 2020, 1, 2, 3, 4, 5, 0), b"2020-01-02T03:04:05-01:00 The standard uses the Gregorian calendar, which 'serves as an international standard for civil use'.[18]"),
            // ISO 8601, time basic format
            (0, 23, (O_M1, 2020, 1, 2, 3, 4, 5, 0), b"2020-01-02T030405-01:00 ISO 8601:2004 fixes a reference calendar date to the Gregorian calendar of 20 May 1875 as the date the Convention du M\xE8tre (Metre Convention) was signed in Paris (the explicit reference date was removed in ISO 8601-1:2019)."),
            // ISO 8601, time extended format
            (0, 23, (O_M1, 2020, 1, 2, 3, 4, 5, 0), b"20200102T03:04:05-01:00 However, ISO calendar dates before the convention are still compatible with the Gregorian calendar all the way back to the official introduction of the Gregorian calendar on 15 October 1582."),
            // ISO 8601, time extended format
            (0, 23, (O_M1, 2020, 1, 2, 3, 4, 5, 0), b"20200102T03:04:05-01:00 Calendar date representations are in the form shown in the adjacent box. [YYYY] indicates a four-digit year, 0000 through 9999. [MM] indicates a two-digit month of the year, 01 through 12. [DD] indicates a two-digit day of that month, 01 through 31."),
            // ISO 8601 / RFC 3339, time basic format
            (0, 21, (O_Z, 2020, 1, 2, 3, 4, 5, 0), b"20200102T030405-00:00 IETF RFC 3339[43] defines a profile of ISO 8601 for use in Internet protocols and standards."),
            // ISO 8601 / RFC 3339, time extended format
            (0, 23, (O_Z, 2020, 1, 2, 3, 4, 5, 0), b"20200102T03:04:05-00:00 RFC 3339 deviates from ISO 8601 in allowing a zero time zone offset to be specified as '-00:00;', which ISO 8601 forbids."),
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
            (0, 27, (O_M1, 2020, 1, 2, 3, 4, 5, 0), b"2020-01-02T03:04:05\xe2\x88\x9201:00 To represent a negative offset, ISO 8601 specifies using a minus sign, (\xe2\x88\x92) (U+2212)."),
        ],
        line!(),
    ),
    #[cfg(any(regex = "77", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        77,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzp, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMSzp,
        0, 50,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 23, (O_M11, 2000, 1, 9, 0, 0, 4, 0), b"2000/01/09 00:00:04 -11 abcdefghij"),
            (0, 25, (O_M11, 2000, 1, 9, 0, 0, 4, 0), b"2000/01/09 00:00:04 \xe2\x88\x9211 abcdefghij"), // U+2212
        ],
        line!(),
    ),
    #[cfg(any(regex = "78", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        78,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZZ, RP_NOALPHA),
        DfaU8,
        DTFSS_YmdHMSZ,
        0, 50,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 24, (O_VLAT, 2000, 1, 10, 0, 0, 5, 0), b"2000/01/10T00:00:05 VLAT abcdefghijk"),
            (0, 23, (O_PST, 2000, 1, 10, 0, 0, 5, 0), b"2000/01/10T00:00:05 pst abcdefghijk"),
            (0, 24, (O_VLAT, 2000, 1, 10, 0, 0, 5, 0), b"2000/01/10T00:00:05 VLAT "),
            (0, 23, (O_PST, 2000, 1, 10, 0, 0, 5, 0), b"2000/01/10T00:00:05 pst\tfoo"),
            (0, 24, (O_VLAT, 2000, 1, 10, 0, 0, 5, 0), b"2000/01/10T00:00:05 VLAT"),
            (0, 23, (O_PST, 2000, 1, 10, 0, 0, 5, 0), b"2000/01/10T00:00:05 pst123"),
            (0, 22, (O_PST, 2000, 1, 10, 0, 0, 5, 0), b"2000/01/10T00:00:05pst:foo"),
        ],
        line!(),
    ),
    //
    #[cfg(any(regex = "79", regex = "ALL", regex = "TEST"))]
    ERE_REGEX_DATETIME!(
        79,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NODIGIT),
        DfaU8,
        DTFSS_YmdHMS,
        0, 50,
        CGN_YEAR, CGN_SECOND,
        &[
            (0, 19, (O_L, 2020, 1, 11, 0, 0, 26, 0), b"2020-01-11 00:00:26 abcdefghijkl"),
            (0, 19, (O_L, 2020, 1, 11, 0, 0, 26, 0), b"2020-01-11 00:00:26 pstxxxxxxxxx"),
            (0, 19, (O_L, 2020, 1, 11, 0, 0, 26, 0), b"2020-01-11 00:00:26 \xe2\x88\x92pstxxxxxxxxx"), // U+2212
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
    // timezone then year
    #[cfg(any(regex = "80", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        80,
        counter!(DP_KEY),
        // TODO: add variation with `CGP_TZZ_L`, `CGP_MONTHB`
        concat!("^", CGP_DAYa3, RP_BLANK, CGP_MONTHb, RP_BLANK12, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZZ_U, RP_BLANK12, CGP_YEAR, RP_NOALNUM), // reduced
        DfaU8,
        DTFSS_BdHMSYZ,
        0, 45,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (0, 27, (O_PST, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 PST 2016 try umount root [1] times"),
            (0, 28, (O_PST, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC  5 21:01:12 PST 2016 try umount root [1] times"),
            (0, 28, (O_PST, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC 05 21:01:12 PST 2016 try umount root [1] times"),
            (0, 28, (O_PST, 2016, 12, 5, 21, 1, 12, 0), b"mon dec  5 21:01:12 PST 2016 try umount root [1] times"),
            (0, 28, (O_PST, 2016, 12, 5, 21, 1, 12, 0), b"mon dec. 5 21:01:12 PST 2016 try umount root [1] times"),
            //(0, 31, (O_PST, 2016, 12, 5, 21, 1, 12, 0), "MONDAY dec  5 21:01:12 PST 2016 try umount root [1] times"),
            //(0, 31, (O_PST, 2016, 12, 5, 21, 1, 12, 0), "MONDAY DEC  5 21:01:12 PST 2016 try umount root [1] times"),
            (0, 27, (O_PDT, 2017, 5, 8, 8, 33, 0, 0), b"mon May 8 08:33:00 PDT 2017 try umount root [1] times"),
            (0, 28, (O_PST, 2018, 2, 28, 14, 58, 7, 0), b"Wed Feb 28 14:58:07 PST 2018 try umount root [1] times"),
            //(0, 34, (O_PST, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY Feb 28 14:58:07 PST 2018 try umount root [1] times"),
            //(0, 40, (O_PST, 2018, 2, 28, 14, 58, 7, 0), "WEDNESDAY February 28 14:58:07\tPST\t\t2018 try umount root [1] times"),
            //(0, 35, (O_PST, 2018, 2, 28, 14, 58, 7, 0), "WED. February 28 14:58:07\tPST\t\t2018 try umount root [1] times"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "81", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        81,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZz, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSYz,
        0, 45,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (0, 29, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 -0000 2016 try umount root [1] times"),
            (0, 31, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 \xe2\x88\x920000 2016 try umount root [1] times"), // U+2212
            (0, 30, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC  5 21:01:12 +0000 2016 try umount root [1] times"),
            (0, 30, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC 05 21:01:12 +0000 2016 try umount root [1] times"),
            (0, 30, (O_M1130, 2016, 12, 5, 21, 1, 12, 0), b"mon dec  5 21:01:12 -1130 2016 try umount root [1] times"),
            (0, 29, (O_P945, 2017, 5, 8, 8, 33, 0, 0), b"mon May 8 08:33:00 +0945 2017 try umount root [1] times"),
            (0, 32, (O_P945, 2017, 5, 8, 8, 33, 0, 0), b"monday may 8 08:33:00 +0945 2017 try umount root [1] times"),
            (0, 30, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), b"Wed Feb 28 14:58:07 -1030 2018 try umount root [1] times"),
            (0, 36, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY Feb 28 14:58:07 -1030 2018 try umount root [1] times"),
            (0, 41, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY February 28 14:58:07 -1030 2018 try umount root [1] times"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "82", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        82,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZzc, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSYzc,
        0, 45,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (0, 30, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 -00:00 2016 try umount root [1] times"),
            (0, 32, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 \xe2\x88\x9200:00 2016 try umount root [1] times"), // U+2212
            (0, 31, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC  5 21:01:12 +00:00 2016 try umount root [1] times"),
            (0, 31, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC 05 21:01:12 +00:00 2016 try umount root [1] times"),
            (0, 31, (O_M1130, 2016, 12, 5, 21, 1, 12, 0), b"mon dec  5 21:01:12 -11:30 2016 try umount root [1] times"),
            (0, 30, (O_P945, 2017, 5, 8, 8, 33, 0, 0), b"mon May 8 08:33:00 +09:45 2017 try umount root [1] times"),
            (0, 33, (O_P945, 2017, 5, 8, 8, 33, 0, 0), b"monday may 8 08:33:00 +09:45 2017 try umount root [1] times"),
            (0, 31, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), b"Wed Feb 28 14:58:07 -10:30 2018 try umount root [1] times"),
            (0, 37, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY Feb 28 14:58:07 -10:30 2018 try umount root [1] times"),
            (0, 42, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY February 28 14:58:07 -10:30 2018 try umount root [1] times"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "83", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        83,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZzp, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSYzp,
        0, 45,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (0, 27, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 -00 2016 try umount root [1] times"),
            (0, 28, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC  5 21:01:12 +00 2016 try umount root [1] times"),
            (0, 28, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC 05 21:01:12 +00 2016 try umount root [1] times"),
            (0, 28, (O_M11, 2016, 12, 5, 21, 1, 12, 0), b"mon dec  5 21:01:12 -11 2016 try umount root [1] times"),
            (0, 33, (O_M11, 2016, 12, 5, 21, 1, 12, 0), b"mon december  5 21:01:12 -11 2016 try umount root [1] times"),
            (0, 27, (O_P9, 2017, 5, 8, 8, 33, 0, 0), b"mon May 8 08:33:00 +09 2017 try umount root [1] times"),
            (0, 27, (O_P9, 2017, 5, 8, 8, 33, 0, 0), b"mon MAY 8 08:33:00 +09 2017 try umount root [1] times"),
            (0, 28, (O_M10, 2018, 2, 28, 14, 58, 7, 0), b"Wed Feb 28 14:58:07 -10 2018 try umount root [1] times"),
            (0, 34, (O_M10, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY Feb 28 14:58:07 -10 2018 try umount root [1] times"),
            (0, 39, (O_M10, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY February 28 14:58:07 -10 2018 try umount root [1] times"),
            (0, 39, (O_M10, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY FEBRUARY 28 14:58:07 -10 2018 try umount root [1] times"),
        ],
        line!(),
    ),
    // year then timezone
    #[cfg(any(regex = "84", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        84,
        counter!(DP_KEY),
        // TODO: add variation with `CGP_TZZ_L`
        concat!("^", CGP_DAYa3, RP_BLANK, CGP_MONTHb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_YEAR, RP_BLANK12, CGP_TZZ_U, RP_NOALNUM), // reduced
        DfaU8,
        DTFSS_BdHMSYZ,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (0, 27, (O_PST, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 2016 PST try umount root [1] times"),
            (0, 28, (O_PST, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC  5 21:01:12 2016 PST try umount root [1] times"),
            (0, 28, (O_PST, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC. 5 21:01:12 2016 PST try umount root [1] times"),
            (0, 29, (O_PST, 2016, 12, 5, 21, 1, 12, 0), b"MON. DEC. 5 21:01:12 2016 PST try umount root [1] times"),
            (0, 28, (O_PST, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC 05 21:01:12 2016 PST try umount root [1] times"),
            (0, 28, (O_PST, 2016, 12, 5, 21, 1, 12, 0), b"mon dec  5 21:01:12 2016 PST try umount root [1] times"),
            //(0, 28, (O_PST, 2016, 12, 5, 21, 1, 12, 0), b"mon dec  5 21:01:12 2016 pst try umount root [1] times"),
            (0, 27, (O_PDT, 2017, 5, 8, 8, 33, 0, 0), b"mon May 8 08:33:00 2017 PDT try umount root [1] times"),
            (0, 28, (O_PST, 2018, 2, 28, 14, 58, 7, 0), b"Wed Feb 28 14:58:07 2018 PST try umount root [1] times"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "85", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        85,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa3, RP_BLANK, CGP_MONTHB, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_YEAR, RP_BLANK12, CGP_TZZ_U, RP_NOALNUM), // reduced
        DfaU8,
        DTFSS_BdHMSYZ,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (0, 33, (O_PST, 2018, 2, 8, 14, 58, 7, 0), b"WED February  8 14:58:07\t2018\tPST try umount root [1] times"),
            (0, 33, (O_PST, 2018, 2, 28, 14, 58, 7, 0), b"WED February 28 14:58:07\t2018\tPST try umount root [1] times"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "86", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        86,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZz, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSYz,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (0, 29, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 2016 -0000 try umount root [1] times"),
            (0, 31, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 2016 \xe2\x88\x920000 try umount root [1] times"), // U+2212
            (0, 30, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC  5 21:01:12 2016 +0000 try umount root [1] times"),
            (0, 30, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC. 5 21:01:12 2016 +0000 try umount root [1] times"),
            (0, 30, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC 05 21:01:12 2016 +0000 try umount root [1] times"),
            (0, 30, (O_M1130, 2016, 12, 5, 21, 1, 12, 0), b"mon dec  5 21:01:12 2016 -1130 try umount root [1] times"),
            (0, 29, (O_P945, 2017, 5, 8, 8, 33, 0, 0), b"mon May 8 08:33:00 2017 +0945 try umount root [1] times"),
            (0, 32, (O_P945, 2017, 5, 8, 8, 33, 0, 0), b"monday may 8 08:33:00 2017 +0945 try umount root [1] times"),
            (0, 30, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), b"Wed Feb 28 14:58:07 2018 -1030 try umount root [1] times"),
            (0, 36, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY Feb 28 14:58:07 2018 -1030 try umount root [1] times"),
            (0, 41, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY February 28 14:58:07\t2018\t-1030 try umount root [1] times"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "87", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        87,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZzc, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSYzc,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (0, 30, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 2016 -00:00 try umount root [1] times"),
            (0, 32, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 2016 \xe2\x88\x9200:00 try umount root [1] times"), // U+2212
            (0, 31, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC  5 21:01:12 2016 +00:00 try umount root [1] times"),
            (0, 31, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC. 5 21:01:12 2016 +00:00 try umount root [1] times"),
            (0, 31, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC 05 21:01:12 2016 +00:00 try umount root [1] times"),
            (0, 31, (O_M1130, 2016, 12, 5, 21, 1, 12, 0), b"mon dec  5 21:01:12 2016 -11:30 try umount root [1] times"),
            (0, 30, (O_P945, 2017, 5, 8, 8, 33, 0, 0), b"mon May 8 08:33:00 2017 +09:45 try umount root [1] times"),
            (0, 33, (O_P945, 2017, 5, 8, 8, 33, 0, 0), b"monday may 8 08:33:00 2017 +09:45 try umount root [1] times"),
            (0, 31, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), b"Wed Feb 28 14:58:07 2018 -10:30 try umount root [1] times"),
            (0, 37, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY Feb 28 14:58:07 2018 -10:30 try umount root [1] times"),
            (0, 42, (O_M1030, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY February 28 14:58:07 2018 -10:30 try umount root [1] times"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "88", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        88,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZzp, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSYzp,
        0, 45,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (0, 27, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 2016 -00 try umount root [1] times"),
            (0, 28, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC  5 21:01:12 2016 +00 try umount root [1] times"),
            (0, 28, (O_Z, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC 05 21:01:12 2016 +00 try umount root [1] times"),
            (0, 28, (O_M11, 2016, 12, 5, 21, 1, 12, 0), b"mon dec  5 21:01:12 2016 -11 try umount root [1] times"),
            (0, 28, (O_M11, 2016, 12, 5, 21, 1, 12, 0), b"mon dec. 5 21:01:12 2016 -11 try umount root [1] times"),
            (0, 33, (O_M11, 2016, 12, 5, 21, 1, 12, 0), b"mon december  5 21:01:12 2016 -11 try umount root [1] times"),
            (0, 27, (O_P9, 2017, 5, 8, 8, 33, 0, 0), b"mon May 8 08:33:00 2017 +09 try umount root [1] times"),
            (0, 27, (O_P9, 2017, 5, 8, 8, 33, 0, 0), b"mon MAY 8 08:33:00 2017 +09 try umount root [1] times"),
            (0, 28, (O_M10, 2018, 2, 28, 14, 58, 7, 0), b"Wed Feb 28 14:58:07 2018 -10 try umount root [1] times"),
            (0, 34, (O_M10, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY Feb 28 14:58:07 2018 -10 try umount root [1] times"),
            (0, 39, (O_M10, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY February 28 14:58:07 2018 -10 try umount root [1] times"),
            (0, 39, (O_M10, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY FEBRUARY 28 14:58:07 2018 -10 try umount root [1] times"),
        ],
        line!(),
    ),
    // no timezone
    #[cfg(any(regex = "89", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        89,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSY,
        0, 45,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (0, 23, (O_L, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 2016 try umount root [1] times"),
            (0, 24, (O_L, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC  5 21:01:12 2016 try umount root [1] times"),
            (0, 24, (O_L, 2016, 12, 5, 21, 1, 12, 0), b"MON DEC 05 21:01:12 2016 try umount root [1] times"),
            (0, 24, (O_L, 2016, 12, 5, 21, 1, 12, 0), b"mon dec  5 21:01:12 2016 try umount root [1] times"),
            (0, 29, (O_L, 2016, 12, 5, 21, 1, 12, 0), b"mon december  5 21:01:12 2016 try umount root [1] times"),
            (0, 23, (O_L, 2017, 5, 8, 8, 33, 0, 0), b"mon May 8 08:33:00 2017 try umount root [1] times"),
            (0, 23, (O_L, 2017, 5, 8, 8, 33, 0, 0), b"mon MAY 8 08:33:00 2017 try umount root [1] times"),
            (0, 24, (O_L, 2018, 2, 28, 14, 58, 7, 0), b"Wed Feb 28 14:58:07 2018 try umount root [1] times"),
            (0, 30, (O_L, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY Feb 28 14:58:07 2018 try umount root [1] times"),
            (0, 35, (O_L, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY February 28 14:58:07 2018 try umount root [1] times"),
            (0, 35, (O_L, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY FEBRUARY 28 14:58:07 2018 try umount root [1] times"),
        ],
        line!(),
    ),
    //
    // from file `./logs/programs/proftpd/xferlog`, Issue #42
    // example with offset:
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     Sat Oct 03 11:26:12 2020 0 192.168.0.8 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c
    //     Sat Oct 03 11:26:12 2020 0 192.168.0.8 2323 /var/log/proftpd/proftpd.log b _ o r root ftp 0 * c
    //
    #[cfg(any(regex = "90", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        90,
        counter!(DP_KEY),
        concat!("^", CGP_DAYa, RP_BLANK, CGP_MONTHBb, RP_BLANKS, CGP_DAYde, RP_BLANK, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_NOALNUMpm),
        DfaU8,
        DTFSS_BdHMSY,
        0, 40,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (0, 23, (O_L, 2016, 12, 5, 21, 1, 12, 0), b"Mon Dec 5 21:01:12 2016 try umount root [1] times"),
            (0, 24, (O_L, 2016, 12, 5, 21, 1, 13, 0), b"MON DEC  5 21:01:13 2016 try umount root [1] times"),
            (0, 24, (O_L, 2016, 12, 5, 21, 1, 14, 0), b"mon dec  5 21:01:14 2016 try umount root [1] times"),
            (0, 29, (O_L, 2016, 12, 5, 21, 1, 12, 0), b"mon december  5 21:01:12 2016 try umount root [1] times"),
            (0, 23, (O_L, 2017, 5, 8, 8, 33, 0, 0), b"mon May 8 08:33:00 2017 try umount root [1] times"),
            (0, 23, (O_L, 2017, 5, 8, 8, 33, 0, 0), b"mon MAY 8 08:33:00 2017 try umount root [1] times"),
            (0, 24, (O_L, 2018, 2, 28, 14, 58, 7, 0), b"Wed Feb 28 14:58:07 2018 try umount root [1] times"),
            (0, 30, (O_L, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY Feb 28 14:58:07 2018 try umount root [1] times"),
            (0, 35, (O_L, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY February 28 14:58:07 2018 try umount root [1] times"),
            (0, 35, (O_L, 2018, 2, 28, 14, 58, 7, 0), b"WEDNESDAY FEBRUARY 28 14:58:07 2018 try umount root [1] times"),
            (0, 24, (O_L, 2020, 10, 3, 11, 26, 12, 0), b"Sat Oct  3 11:26:12 2020 0 192.168.0.8 2323 /var/log/proftpd/proftpd.log b _ o r root ftp 0 * c")
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // file `logs/other/tests/dtf14a.log`
    //
    //                1         2         3         4
    //      01234567890123456789012345678901234567890
    //      2023 Aug 31 20:01:05 UTC [ERROR] dev-disk-a error 0x08320105
    //      2023 Aug 31 20:01:09 UTC [WARNING] dev-disk-a disconnected.
    //
    #[cfg(any(regex = "91", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        91,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, RP_BLANK12, CGP_MONTHBb, RP_BLANK12q, CGP_DAYde, RP_BLANK12q, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12q, CGP_TZzc, RP_NOALNUM),
        DfaU8,
        DTFSS_YbdHMSzc,
        0, 30,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 27, (O_0, 2023, 8, 31, 20, 1, 5, 0), b"2023 Aug 31 20:01:05 -00:00 [ERROR] dev-disk-a error 0x08320105"),
            (0, 27, (O_P1, 2023, 8, 31, 20, 1, 9, 0), b"2023 Aug 31 20:01:09 +01:00 [WARNING] dev-disk-a disconnected."),
        ],
        line!(),
    ),
    #[cfg(any(regex = "92", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        92,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, RP_BLANK12, CGP_MONTHBb, RP_BLANK12q, CGP_DAYde, RP_BLANK12q, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12q, CGP_TZz, RP_NOALNUM),
        DfaU8,
        DTFSS_YbdHMSz,
        0, 30,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 26, (O_0, 2023, 8, 31, 20, 1, 5, 0), b"2023 Aug 31 20:01:05 -0000 [ERROR] dev-disk-a error 0x08320105"),
            (0, 26, (O_P1, 2023, 8, 31, 20, 1, 9, 0), b"2023 Aug 31 20:01:09 +0100 [WARNING] dev-disk-a disconnected."),
        ],
        line!(),
    ),
    #[cfg(any(regex = "93", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        93,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, RP_BLANK12, CGP_MONTHBb, RP_BLANK12q, CGP_DAYde, RP_BLANK12q, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12q, CGP_TZzp, RP_NOALNUM),
        DfaU8,
        DTFSS_YbdHMSzp,
        0, 30,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 24, (O_0, 2023, 8, 31, 20, 1, 5, 0), b"2023 Aug 31 20:01:05 -00 [ERROR] dev-disk-a error 0x08320105"),
            (0, 24, (O_P1, 2023, 8, 31, 20, 1, 9, 0), b"2023 Aug 31 20:01:09 +01 [WARNING] dev-disk-a disconnected."),
        ],
        line!(),
    ),
    #[cfg(any(regex = "94", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        94,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, RP_BLANK12, CGP_MONTHBb, RP_BLANK12q, CGP_DAYde, RP_BLANK12q, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12q, CGP_TZZ, RP_NOALNUM),
        DfaU8,
        DTFSS_YbdHMSZ,
        0, 30,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 24, (O_0, 2023, 8, 31, 20, 1, 5, 0), b"2023 Aug 31 20:01:05 UTC [ERROR] dev-disk-a error 0x08320105"),
            (0, 25, (O_P1, 2023, 8, 31, 20, 1, 9, 0), b"2023 Aug 31 20:01:09 WEST [WARNING] dev-disk-a disconnected."),
        ],
        line!(),
    ),
    #[cfg(any(regex = "95", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        95,
        counter!(DP_KEY),
        concat!("^", CGP_YEAR, RP_BLANK12, CGP_MONTHBb, RP_BLANK12q, CGP_DAYde, RP_BLANK12q, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NOALNUM),
        DfaU8,
        DTFSS_YbdHMS,
        0, 25,
        CGN_YEAR, CGN_SECOND,
        &[
            (0, 20, (O_L, 2023, 8, 31, 20, 1, 5, 0), b"2023 Aug 31 20:01:05 [ERROR] dev-disk-a error 0x08320105"),
            (0, 20, (O_L, 2023, 8, 31, 20, 1, 9, 0), b"2023 Aug 31 20:01:09 [WARNING] dev-disk-a disconnected."),
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
    #[cfg(any(regex = "96", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        96,
        counter!(DP_KEY),
        // add more "guard" chars
        concat!(r"^\[", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYde, D_DHq, CGP_HOUR, D_Teq, CGP_MINUTE, r"\]"),
        DfaU8,
        DTFSS_YmdHM,
        0, 20,
        CGN_YEAR, CGN_MINUTE,
        &[
            (1, 17, (O_L, 2019, 3, 1, 16, 56, 0, 0), b"[2019-03-01 16:56] [PACMAN] synchronizing package lists"),
            (1, 17, (O_L, 2018, 5, 31, 12, 19, 0, 0), b"[2018-05-31 12:19] [PACMAN] Running 'pacman -Syu --root /tmp/newmsys/msys64'"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // Red Hat Audit log format, example with offset:
    //
    //               1         2         3         4         5         6
    //     0123456789012345678901234567890123456789012345678901234567890
    //     type=DAEMON_START msg=audit(1681160194.260:3932): op=start ver=3.0.7 format=enriched kernel=5.14.0-162.6.1.el9_1.x86_64 auid=4294967295 pid=718 uid=0 ses=4294967295 subj=system_u:system_r:auditd_t:s0 res=success�AUID="unset" UID="root"
    //
    #[cfg(any(regex = "97", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        97,
        counter!(DP_KEY),
        concat!(RP_BLANK, r"msg=audit\(", CGP_EPOCH, ".", CGP_FRACTIONAL3, r":[0-9]{1,5}\):", RP_BLANK),
        DfaU8,
        DTFSS_sf,
        0, 100,
        CGN_EPOCH, CGN_FRACTIONAL,
        &[
            (28, 42, (O_L, 2023, 4, 10, 20, 56, 34, 260000000), b"type=DAEMON_START msg=audit(1681160194.260:3932): op=start ver=3.0.7 format=enriched kernel=5.14.0-162.6.1.el9_1.x86_64 auid=4294967295 pid=718 uid=0 ses=4294967295 subj=system_u:system_r:auditd_t:s0 res=success\xE2\x80\xA6AUID=\"unset\" UID=\"root\""),
            (31, 45, (O_L, 2023, 5, 8, 6, 9, 26, 814000000), br#"type=CRYPTO_KEY_USER msg=audit(1683526166.814:492): pid=13862 uid=0 auid=0 ses=6 subj=system_u:system_r:sshd_t:s0-s0:c0.c1023 msg='op=destroy kind=server fp=SHA256:34:76:7b:a4:dc:bb:e0:b6:5e:5d:73:e9:a1:db:89:21:c0:0d:ca:54:f4:7d:46:9c:b2:87:c4:ed:0b:d4:3f:59 direction=? spid=13900 suid=0  exe="/usr/sbin/sshd" hostname=? addr=? terminal=? res=success'UID="root" AUID="root" SUID="root""#),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // strace formats
    //
    //                1         2
    //      012345678901234567890
    //      $ strace -ttt ls
    //      1716853121.780157 execve("/usr/bin/ls", ["ls"], 0x7ffe4c501508 /* 41 vars */) = 0
    //
    // strace `--timestamp=unix,ms'
    // TODO: move this clump of patterns down in importance as these can
    //       match too many other things
    #[cfg(any(regex = "98", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        98,
        counter!(DP_KEY),
        concat!("^", CGP_EPOCH, "[.,]", CGP_FRACTIONAL3, RP_BLANK, RP_BLANK_NO),
        DfaU8,
        DTFSS_sf,
        0, 23,
        CGN_EPOCH, CGN_FRACTIONAL,
        &[
            (0, 14, (O_L, 2024, 5, 27, 23, 38, 41, 780000000), br#"1716853121.780 execve("/usr/bin/ls", ["ls"], 0x7ffe4c501508 /* 41 vars */) = 0"#),
        ],
        line!(),
    ),
    // strace `--timestamp=unix,ns' or `-ttt`
    #[cfg(any(regex = "99", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        99,
        counter!(DP_KEY),
        concat!("^", CGP_EPOCH, "[.,]", CGP_FRACTIONAL6, RP_BLANK, RP_BLANK_NO),
        DfaU8,
        DTFSS_sf,
        0, 26,
        CGN_EPOCH, CGN_FRACTIONAL,
        &[
            (0, 17, (O_L, 2024, 5, 27, 23, 38, 41, 780157000), br#"1716853121.780157 execve("/usr/bin/ls", ["ls"], 0x7ffe4c501508 /* 41 vars */) = 0"#),
        ],
        line!(),
    ),
    // strace `--timestamp=unix,ns'
    #[cfg(any(regex = "100", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        100,
        counter!(DP_KEY),
        concat!("^", CGP_EPOCH, "[.,]", CGP_FRACTIONAL9, RP_BLANK, RP_BLANK_NO),
        DfaU8,
        DTFSS_sf,
        0, 29,
        CGN_EPOCH, CGN_FRACTIONAL,
        &[
            (0, 20, (O_L, 2024, 5, 27, 23, 38, 41, 780157012), br#"1716853121.780157012 execve("/usr/bin/ls", ["ls"], 0x7ffe4c501508 /* 41 vars */) = 0"#),
        ],
        line!(),
    ),
    // strace `--timestamp=unix' or `--timestamp=unix,s'
    #[cfg(any(regex = "101", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        101,
        counter!(DP_KEY),
        concat!("^", CGP_EPOCH, RP_BLANK, RP_BLANK_NO),
        DfaU8,
        DTFSS_s,
        0, 19,
        CGN_EPOCH, CGN_EPOCH,
        &[
            (0, 10, (O_L, 2024, 5, 27, 23, 38, 41, 0), br#"1716853121 execve("/usr/bin/ls", ["ls"], 0x7ffe4c501508 /* 41 vars */) = 0"#),
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
    #[cfg(any(regex = "102", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        102,
        counter!(DP_KEY),
        concat!(RP_NODIGITb, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, ":", CGP_FRACTIONAL3, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        //DfaU8, // fails to build
        FlatLockstepNfaU8,
        DTFSS_YmdHMSfz,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
                (40, 68, (O_M7, 2022, 10, 12, 9, 26, 44, 980000000), br"{5F45546A-691D-4519-810C-9B159EA7A24F}  2022-10-12 09:26:44:980-0700    1       181 [AGENT_INSTALLING_STARTED]  101      {ADF3720E-8453-44C7-82EF-F9F5DA2D8551}  1       0 Update;ScanForUpdates    Success Content Download        Download succeeded.     te2D3dMIjE2PeNSM.86.3.1.0.0.85.0"),
                (40, 68, (O_M7, 2022, 10, 12, 9, 26, 44, 169000000), br"{F4A3F9DB-F870-4022-A079-D5D2B596519D}  2022-10-12 09:26:44:169-0700    1       162 [AGENT_DOWNLOAD_SUCCEEDED]  101     {ADF3720E-8453-44C7-82EF-F9F5DA2D8551}  1       0       Update;ScanForUpdates   SuccessContent Download Download succeeded.     te2D3dMIjE2PeNSM.86.3.1.0.0.85.0"),
        ],
        line!(),
    ),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // matches of datetime field commonly found in JSONL files (single-line JSON entries)
    //
    // example with offset:
    //
    //               1         2         3         4         5         6         7         8         9         0         1         2         3         4         5
    //     0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890
    //     {"level":"INFO","message":"Started","timestamp":"2024-04-08T21:55:48.726Z"}
    //
    // "timestamp" with fractional
    #[cfg(any(regex = "103", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        103,
        counter!(DP_KEY),
        concat!(r#""(TIMESTAMP|Timestamp|timestamp)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZZ, "\""),
        //DfaU8, // 24m22s build time
        FlatLockstepNfaU8,
        DTFSS_YmdHMSfZ,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (51, 75, (O_Z, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"level":"INFO", "message":"Started", "timestamp":"2000-01-02T05:01:32.123Z"}"#),
            (16, 43, (O_M8, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"TIMESTAMP" : "2000/01/02 05-01-32.123 PST", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "104", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        104,
        counter!(DP_KEY),
        concat!(r#""(TIMESTAMP|Timestamp|timestamp)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzc, "\""),
        DfaU8,
        DTFSS_YmdHMSfzc,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (51, 80, (O_Z, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"level":"INFO", "message":"Started", "timestamp":"2000-01-02T05:01:32.123+00:00"}"#),
            (16, 46, (O_M8, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"TIMESTAMP" : "2000/01/02 05-01-32.123 -08:00", "data" : ""}"#),
            (17, 49, (O_M8, 2000, 1, 2, 5, 1, 32, 123000000), b"{{\"TIMESTAMP\" : \"2000/01/02 05-01-32.123 \xe2\x88\x9208:00\", \"data\" : \"\"}"), // U+2212
        ],
        line!(),
    ),
    #[cfg(any(regex = "105", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        105,
        counter!(DP_KEY),
        concat!(r#""(TIMESTAMP|Timestamp|timestamp)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZz, "\""),
        // DfaU8, // fails to build; exceeded DFA state limit of 256
        FlatLockstepNfaU8,
        DTFSS_YmdHMSfz,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (51, 79, (O_Z, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"level":"INFO", "message":"Started", "timestamp":"2000-01-02T05:01:32.123+0000"}"#),
            (16, 45, (O_M8, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"TIMESTAMP" : "2000/01/02 05-01-32.123 -0800", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "106", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        106,
        counter!(DP_KEY),
        concat!(r#""(TIMESTAMP|Timestamp|timestamp)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzp, "\""),
        DfaU8,
        DTFSS_YmdHMSfzp,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (51, 78, (O_Z, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"level":"INFO", "message":"Started", "timestamp":"2000-01-02T05:01:32.123 +00"}"#),
            (16, 42, (O_M8, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"TIMESTAMP" : "2000/01/02 05-01-32.123-08", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "107", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        107,
        counter!(DP_KEY),
        concat!(r#""(TIMESTAMP|Timestamp|timestamp)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, "\""),
        DfaU8,
        DTFSS_YmdHMSf,
        0, 2056,
        CGN_YEAR, CGN_FRACTIONAL,
        &[
            (51, 74, (O_L, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"level":"INFO", "message":"Started", "timestamp":"2000-01-02T05:01:32.123"}"#),
            (16, 39, (O_L, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"TIMESTAMP" : "2000/01/02 05-01-32.123", "data" : ""}"#),
        ],
        line!(),
    ),
    // "timestamp" without fractional
    #[cfg(any(regex = "108", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        108,
        counter!(DP_KEY),
        // TODO: declare with separate `CGP_TZZ_U` and `CGP_TZZ_L`
        concat!(r#""(TIMESTAMP|Timestamp|timestamp)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, RP_BLANKq, CGP_TZZ, "\""),
        //DfaU8, // 13m39s build time
        FlatLockstepNfaU8,
        DTFSS_YmdHMSZ,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (51, 71, (O_Z, 2000, 1, 2, 5, 1, 32, 0), br#"{"level":"INFO", "message":"Started", "timestamp":"2000-01-02T05:01:32Z"}"#),
            (16, 39, (O_M8, 2000, 1, 2, 5, 1, 32, 0), br#"{"TIMESTAMP" : "2000/01/02 05-01-32 PST", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "109", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        109,
        counter!(DP_KEY),
        concat!(r#""(TIMESTAMP|Timestamp|timestamp)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, RP_BLANKq, CGP_TZzc, "\""),
        DfaU8,
        DTFSS_YmdHMSzc,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (51, 77, (O_Z, 2000, 1, 2, 5, 1, 32, 0), br#"{"level":"INFO", "message":"Started", "timestamp":"2000-01-02T05:01:32 +00:00"}"#),
            (16, 41, (O_M8, 2000, 1, 2, 5, 1, 32, 0), br#"{"TIMESTAMP" : "2000/01/02 05-01-32-08:00", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "110", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        110,
        counter!(DP_KEY),
        concat!(r#""(TIMESTAMP|Timestamp|timestamp)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, RP_BLANKq, CGP_TZz, "\""),
        DfaU8,
        DTFSS_YmdHMSz,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (51, 75, (O_Z, 2000, 1, 2, 5, 1, 32, 0), br#"{"level":"INFO", "message":"Started", "timestamp":"2000-01-02T05:01:32+0000"}"#),
            (16, 41, (O_M8, 2000, 1, 2, 5, 1, 32, 0), br#"{"TIMESTAMP" : "2000/01/02 05-01-32 -0800", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "111", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        111,
        counter!(DP_KEY),
        concat!(r#""(TIMESTAMP|Timestamp|timestamp)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, RP_BLANKq, CGP_TZzp, "\""),
        DfaU8,
        DTFSS_YmdHMSzp,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (51, 74, (O_Z, 2000, 1, 2, 5, 1, 32, 0), br#"{"level":"INFO", "message":"Started", "timestamp":"2000-01-02T05:01:32 +00"}"#),
            (16, 39, (O_M8, 2000, 1, 2, 5, 1, 32, 0), br#"{"TIMESTAMP" : "2000/01/02 05-01-32 -08", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "112", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        112,
        counter!(DP_KEY),
        concat!(r#""(TIMESTAMP|Timestamp|timestamp)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, "\""),
        DfaU8,
        DTFSS_YmdHMS,
        0, 2056,
        CGN_YEAR, CGN_SECOND,
        &[
            (51, 70, (O_L, 2000, 1, 2, 5, 1, 32, 0), br#"{"level":"INFO", "message":"Started", "timestamp":"2000-01-02T05:01:32"}"#),
            (16, 35, (O_L, 2000, 1, 2, 5, 1, 32, 0), br#"{"TIMESTAMP" : "2000/01/02 05:01:32", "data" : ""}"#),
        ],
        line!(),
    ),
    // "datetime" with fractional
    #[cfg(any(regex = "113", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        113,
        counter!(DP_KEY),
        concat!(r#""(DATETIME|Datetime|datetime)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZZ, "\""),
        //DfaU8, // 24m2s build time
        FlatLockstepNfaU8,
        DTFSS_YmdHMSfZ,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (50, 74, (O_Z, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"level":"INFO", "message":"Started", "datetime":"2000-01-02T05:01:32.123Z"}"#),
            (15, 42, (O_M8, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"DATETIME" : "2000/01/02 05-01-32.123 PST", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "114", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        114,
        counter!(DP_KEY),
        concat!(r#""(DATETIME|Datetime|datetime)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzc, "\""),
        DfaU8,
        DTFSS_YmdHMSfzc,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (50, 79, (O_Z, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"level":"INFO", "message":"Started", "datetime":"2000-01-02T05:01:32.123+00:00"}"#),
            (15, 45, (O_M8, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"DATETIME" : "2000/01/02 05-01-32.123 -08:00", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "115", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        115,
        counter!(DP_KEY),
        concat!(r#""(DATETIME|Datetime|datetime)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZz, "\""),
        // DfaU8, // fails to build; exceeded DFA state limit of 256
        FlatLockstepNfaU8,
        DTFSS_YmdHMSfz,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (50, 78, (O_Z, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"level":"INFO", "message":"Started", "datetime":"2000-01-02T05:01:32.123+0000"}"#),
            (15, 44, (O_M8, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"DATETIME" : "2000/01/02 05-01-32.123 -0800", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "116", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        116,
        counter!(DP_KEY),
        concat!(r#""(DATETIME|Datetime|datetime)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzp, "\""),
        DfaU8,
        DTFSS_YmdHMSfzp,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (50, 77, (O_Z, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"level":"INFO", "message":"Started", "datetime":"2000-01-02T05:01:32.123 +00"}"#),
            (15, 41, (O_M8, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"DATETIME" : "2000/01/02 05-01-32.123-08", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "117", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        117,
        counter!(DP_KEY),
        concat!(r#""(DATETIME|Datetime|datetime)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL19, "\""),
        DfaU8,
        DTFSS_YmdHMSf,
        0, 2056,
        CGN_YEAR, CGN_FRACTIONAL,
        &[
            (50, 73, (O_L, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"level":"INFO", "message":"Started", "datetime":"2000-01-02T05:01:32.123"}"#),
            (15, 38, (O_L, 2000, 1, 2, 5, 1, 32, 123000000), br#"{"DATETIME" : "2000/01/02 05-01-32.123", "data" : ""}"#),
        ],
        line!(),
    ),
    // "datetime" without fractional
    #[cfg(any(regex = "118", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        118,
        counter!(DP_KEY),
        concat!(r#""(DATETIME|Datetime|datetime)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, RP_BLANKq, CGP_TZZ, "\""),
        // DfaU8, // 13m30s build time
        FlatLockstepNfaU8,
        DTFSS_YmdHMSZ,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (50, 70, (O_Z, 2000, 1, 2, 5, 1, 32, 0), br#"{"level":"INFO", "message":"Started", "datetime":"2000-01-02T05:01:32Z"}"#),
            (15, 38, (O_M8, 2000, 1, 2, 5, 1, 32, 0), br#"{"DATETIME" : "2000/01/02 05-01-32 PST", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "119", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        119,
        counter!(DP_KEY),
        concat!(r#""(DATETIME|Datetime|datetime)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, RP_BLANKq, CGP_TZzc, "\""),
        DfaU8,
        DTFSS_YmdHMSzc,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (50, 76, (O_Z, 2000, 1, 2, 5, 1, 32, 0), br#"{"level":"INFO", "message":"Started", "datetime":"2000-01-02T05:01:32 +00:00"}"#),
            (15, 40, (O_M8, 2000, 1, 2, 5, 1, 32, 0), br#"{"DATETIME" : "2000/01/02 05-01-32-08:00", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "120", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        120,
        counter!(DP_KEY),
        concat!(r#""(DATETIME|Datetime|datetime)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, RP_BLANKq, CGP_TZz, "\""),
        DfaU8,
        DTFSS_YmdHMSz,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (50, 74, (O_Z, 2000, 1, 2, 5, 1, 32, 0), br#"{"level":"INFO", "message":"Started", "datetime":"2000-01-02T05:01:32+0000"}"#),
            (15, 40, (O_M8, 2000, 1, 2, 5, 1, 32, 0), br#"{"DATETIME" : "2000/01/02 05-01-32 -0800", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "121", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        121,
        counter!(DP_KEY),
        concat!(r#""(DATETIME|Datetime|datetime)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, RP_BLANKq, CGP_TZzp, "\""),
        DfaU8,
        DTFSS_YmdHMSzp,
        0, 2056,
        CGN_YEAR, CGN_TZ,
        &[
            (50, 73, (O_Z, 2000, 1, 2, 5, 1, 32, 0), br#"{"level":"INFO", "message":"Started", "datetime":"2000-01-02T05:01:32 +00"}"#),
            (15, 38, (O_M8, 2000, 1, 2, 5, 1, 32, 0), br#"{"DATETIME" : "2000/01/02 05-01-32 -08", "data" : ""}"#),
        ],
        line!(),
    ),
    #[cfg(any(regex = "122", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        122,
        counter!(DP_KEY),
        concat!(r#""(DATETIME|Datetime|datetime)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, "\""),
        DfaU8,
        DTFSS_YmdHMS,
        0, 2056,
        CGN_YEAR, CGN_SECOND,
        &[
            (50, 69, (O_L, 2000, 1, 2, 5, 1, 32, 0), br#"{"level":"INFO", "message":"Started", "datetime":"2000-01-02T05:01:32"}"#),
            (15, 34, (O_L, 2000, 1, 2, 5, 1, 32, 0), br#"{"DATETIME" : "2000/01/02 05:01:32", "data" : ""}"#),
        ],
        line!(),
    ),
    // milliseconds since epoch
    // Found in a ~/.claude/history.jsonl file.
    // Unfortunately, the `timestamp` field is after the `display` field. The `display` field
    // is most of the user prompt (I can't tell if it's truncated)
    // but in the file sample I had the largest `display`  field was ~4000 characters.
    // So this looks in first 10,000 chars.
    // TODO: currently the two constrain ranges, `$sib, $sie`, only accept offset from line
    //       beginning. Allow them to be offset from the end by passing as a negative number.
    //       e.g. values `-256, -1` would constrain the match to be within the last 256 chars
    //       of the line.
    #[cfg(any(regex = "123", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        123,
        counter!(DP_KEY),
        concat!(r#""(TIMESTAMP|Timestamp|timestamp)""#, RP_BLANKq, ":", RP_BLANKq, CGP_EPOCHms, r"[ ,\}]"),
        DfaU8,
        DTFSS_ms,
        0, 10_000,
        CGN_EPOCH, CGN_EPOCH,
        &[
            (49, 62, (O_L, 2026, 5, 23, 3, 3, 34, 374000000), br#"{"display": "Claude are you alive?", "timestamp":1779505414374}"#),
            (15, 28, (O_L, 2026, 5, 23, 3, 3, 34, 74000000), br#"{"TIMESTAMP" : 1779505414074, "data" : "DATA!!!"}"#),
            (15, 28, (O_L, 2026, 5, 23, 3, 3, 34, 74000000), br#"{"TIMESTAMP" : 1779505414074 , "data" : "DATA!!!"}"#),
        ],
        line!(),
    ),
    // Same as prior but seconds since epoch
    #[cfg(any(regex = "124", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        124,
        counter!(DP_KEY),
        concat!(r#""(TIMESTAMP|Timestamp|timestamp)""#, RP_BLANKq, ":", RP_BLANKq, CGP_EPOCH, r"[ ,\}]"),
        DfaU8,
        DTFSS_s,
        0, 10_000,
        CGN_EPOCH, CGN_EPOCH,
        &[
            (49, 59, (O_L, 2026, 5, 23, 3, 3, 34, 0), br#"{"display": "Claude are you alive?", "timestamp":1779505414}"#),
            (15, 25, (O_L, 2026, 5, 23, 3, 3, 34, 0), br#"{"TIMESTAMP" : 1779505414, "data" : "DATA!!!"}"#),
            (15, 25, (O_L, 2026, 5, 23, 3, 3, 34, 0), br#"{"TIMESTAMP" : 1779505414 , "data" : "DATA!!!"}"#),
        ],
        line!(),
    ),
    // TODO: add equivalent with leading "time" field.

    // ---------------------------------------------------------------------------------------------
    //
    // Chrome cv_debug.log format
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     {"logTime": "0226/052726", "correlationVector":"C3BF38D097234ED3A46F33A1C497BF65","action":"FETCH_UX_CONFIG", "result":""}
    //
    #[cfg(any(regex = "125", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        125,
        counter!(DP_KEY),
        concat!(r#""(LOGTIME|LogTime|logTime|logtime)""#, RP_BLANKq, ":", RP_BLANKq, "\"", CGP_MONTHm, D_Deq, CGP_DAYde, D_DHcdqus, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, "\""),
        DfaU8,
        DTFSS_mdHMS,
        0, 512,
        CGN_MONTH, CGN_SECOND,
        &[
            (13, 24, (O_L, YD, 2, 26, 5, 27, 26, 0), br#"{"logTime": "0226/052726", "correlationVector":"A", "action":"FETCH_UX_CONFIG", "result":""}"#),
            (13, 24, (O_L, YD, 2, 26, 5, 27, 26, 0), br#"{"LOGTIME" :"0226/052726", "correlationVector":"A", "action":"FETCH_UX_CONFIG", "result":""}"#),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // general matches anywhere in the first 1024 bytes of the line
    //
    // these are most likely to match datetimes in the log *message*, i.e. a substring that happens
    // to be a datetime but is not the formal log timestamp. In other words, most likely to cause
    // errant matches. One reason they are declared last and so attempted last.
    //
    #[cfg(any(regex = "126", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        126,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZz, RP_RB),
        // DfaU8, // fails to build; exceeded DFA state limit of 226.
        FlatLockstepNfaU8,
        DTFSS_YmdHMSfz,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
            (1, 30, (O_M11, 2000, 1, 2, 5, 1, 32, 123000000), b"<2000/01/02 05:01:32.123 -1100> a"),
            (1, 30, (O_M11, 2000, 1, 2, 5, 1, 32, 123000000), b"{2000/01/02 05:01:32.123 -1100} a"),
            (1, 32, (O_M11, 2000, 1, 2, 5, 1, 32, 123000000), b"{2000/01/02 05:01:32.123 \xe2\x88\x921100} a"), // U+2212
        ],
        line!(),
    ),
    #[cfg(any(regex = "127", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        127,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL19, RP_BLANKq, CGP_TZzc, RP_RB),
        DfaU8,
        DTFSS_YmdHMSfzc,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
            (11, 43, (O_M1130, 2000, 1, 3, 5, 2, 33, 123456000), b"[LOGGER]  {2000/01/03 05:02:33.123456-11:30} ab"),
            (1, 34, (O_M1130, 2000, 1, 3, 5, 2, 33, 123456000), b"<2000-01-03T05:02:33.123456 -11:30> ab"),
            (1, 36, (O_M1130, 2000, 1, 3, 5, 2, 33, 123456000), b"<2000-01-03T05:02:33.123456 \xe2\x88\x9211:30> ab"), // U+2212
        ],
        line!(),
    ),
    #[cfg(any(regex = "128", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        128,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL369, RP_BLANKq, CGP_TZzp, RP_RB),
        DfaU8,
        DTFSS_YmdHMSfzp,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
            (11, 44, (O_M11, 2000, 1, 4, 0, 3, 34, 123456789), b"[LOGGER]  [2000/01/04 00:03:34,123456789 -11]"),
            (11, 44, (O_M11, 2000, 1, 4, 0, 3, 34, 123456789), b"[LOGGER]  [2000/01/04 00:03:34.123456789 -11] abc"),
            (11, 46, (O_M11, 2000, 1, 4, 0, 3, 34, 123456789), b"[LOGGER]  [2000/01/04 00:03:34.123456789 \xe2\x88\x9211] abc"), // U+2212
            (11, 43, (O_M11, 2000, 1, 4, 0, 3, 34, 123456789), b"[LOGGER]  [2000/01/04T00:03:34,123456789-11]abc"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "129", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        129,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL369, RP_BLANKq, CGP_TZZ_U, RP_RB), // reduced
        //DfaU8, // 30+m build time
        FlatLockstepNfaU8,
        DTFSS_YmdHMSfZ,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), b"[LOGGER]\t\t<2000/01/05 00:04:35.123456789 VLAT>:"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), b"[LOGGER]  <2000/01/05 00:04:35.123456789 VLAT>"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), b"[LOGGER]  <2000/01/05 00:04:35.123456789 VLAT> abcd"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), b"[LOGGER]  <2000/01/05 00:04:35.123456789 VLAT>abcd"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), b"[LOGGER]  <2000/01/05 00:04:35.123456789 VLAT>abcd"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), b"[LOGGER]  [2000/01/05 00:04:35.123456789 VLAT] abcd"),
            (11, 45, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), b"[LOGGER]  [2000/01/05-00:04:35.123456789 VLAT]"),
            (11, 44, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), b"[LOGGER]  [2000/01/05-00:04:35.123456789VLAT]"),
            (10, 44, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), b"[LOGGER] (2000/01/05T00:04:35.123456789 VLAT) abcd"),
            //(10, 44, (O_VLAT, 2000, 1, 5, 0, 4, 35, 123456789), "[LOGGER] (2000/01/05T00:04:35.123456789 vlat) abcd"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "130", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        130,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL369, RP_RB),
        DfaU8,
        DTFSS_YmdHMSf,
        0, 1024,
        CGN_YEAR, CGN_FRACTIONAL,
        &[
            (11, 40, (O_L, 2020, 1, 6, 0, 5, 26, 123456789), b"[LOGGER]  (2020-01-06 00:05:26.123456789) abcdefg"),
            (21, 50, (O_L, 2020, 1, 6, 0, 5, 26, 123456789), b"[FOOBAR] (PID 2005) (2020-01-06 00:05:26.123456789) foobar!"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "131", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        131,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL369, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        //DfaU8, // fails to build
        FlatLockstepNfaU8,
        DTFSS_YmdHMSfz,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 29, (O_M11, 2000, 1, 2, 7, 8, 32, 123000000), b"2000/01/02 07:08:32.123 -1100 a"),
            (0, 29, (O_M11, 2000, 1, 2, 7, 8, 32, 123000000), b"2000-01-02T07:08:32.123 -1100 ab"),
            (0, 25, (O_M11, 2000, 1, 2, 7, 8, 32, 123000000), b"20000102:070832.123 -1100 abc"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "132", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        132,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL369, RP_BLANKq, CGP_TZzc, RP_NODIGIT),
        //DfaU8, // fails to build
        FlatLockstepNfaU8,
        DTFSS_YmdHMSfzc,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 33, (O_M1130, 2000, 1, 3, 0, 2, 3, 123456000), b"2000/01/03 00:02:03.123456 -11:30 ab"),
            (1, 34, (O_M1130, 2000, 1, 3, 0, 2, 3, 123456000), b"|2000/01/03:00:02:03.123456 -11:30|ab"),
            (1, 34, (O_M1130, 2000, 1, 3, 0, 2, 3, 123456000), b"<2000/01/03T00:02:03.123456 -11:30 abc"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "133", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        133,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL369, RP_BLANKq, CGP_TZzp, RP_NODIGIT),
        //DfaU8, // fails to build
        FlatLockstepNfaU8,
        DTFSS_YmdHMSfzp,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 33, (O_M11, 2000, 1, 4, 0, 23, 24, 123456789), b"2000/01/04 00:23:24,123456789 -11"),
            (0, 33, (O_M11, 2000, 1, 4, 0, 23, 24, 123456789), b"2000/01/04 00:23:24,123456789 -11 abc"),
            (0, 33, (O_M11, 2000, 1, 4, 0, 23, 24, 123456789), b"2000/01/04 00:23:24,123456789 -11_abc"),
            (1, 34, (O_M11, 2000, 1, 4, 0, 23, 24, 123456789), b"|2000/01/04-00:23:24,123456789 -11|abc"),
            (1, 34, (O_M11, 2000, 1, 4, 0, 23, 24, 123456789), b"[2000/01/04T00:23:24,123456789 -11] abc"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "134", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        134,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL369, RP_BLANKq, CGP_TZZ_U, RP_NOALPHA), // reduced
        //DfaU8, // 30+m build time
        FlatLockstepNfaU8,
        DTFSS_YmdHMSfZ,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 34, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), b"2000/01/05 00:34:35.123456789 VLAT:"),
            (0, 34, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), b"2000/01/05 00:34:35.123456789 VLAT"),
            (0, 34, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), b"2000/01/05 00:34:35.123456789 VLAT abcd"),
            (0, 34, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), b"2000/01/05:00:34:35.123456789 VLAT:abcd"),
            (1, 35, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), b"|2000/01/05 00:34:35.123456789 VLAT|abcd"),
            (1, 35, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), b":2000/01/05 00:34:35.123456789 VLAT: abcd"),
            (1, 35, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), b"[2000/01/05T00:34:35.123456789 VLAT]"),
            (1, 34, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), b"[2000/01/05T00:34:35.123456789VLAT]"),
            //(0, 34, (O_VLAT, 2000, 1, 5, 0, 34, 35, 123456789), "2000/01/05-00:34:35.123456789 vlat abcd"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "135", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        135,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_YEAR, D_Deq, CGP_MONTHm, D_Deq, CGP_DAYde, D_DHcdq, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL369, RP_NODIGIT),
        // DfaU8, // fails to build
        FlatLockstepNfaU8,
        DTFSS_YmdHMSf,
        0, 1024,
        CGN_YEAR, CGN_FRACTIONAL,
        &[
            (0, 29, (O_L, 2020, 1, 6, 0, 5, 26, 123456789), b"2020-01-06 00:05:26.123456789 abcdefg"),
            (6, 29, (O_L, 2020, 1, 6, 0, 5, 26, 123000000), b"FIVER 2020-01-06T00:05:26.123 abcdefg"),
            (0, 29, (O_L, 2020, 1, 6, 0, 5, 26, 123456789), br"2020\01\06 00:05:26.123456789 abcdefg"),
            (20, 49, (O_L, 2020, 1, 6, 0, 5, 26, 123456789), b"[FOOBAR] (PID 2005) 2020-01-06 00:05:26.123456789 foobar!"),
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
    #[cfg(any(regex = "136", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        136,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz, RP_NODIGIT),
        // DfaU8, // fails to build
        FlatLockstepNfaU8,
        DTFSS_YmdHMSz,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 25, (O_M11, 2000, 1, 7, 0, 6, 2, 0), b"2000/01/07T00:06:02 -1100 abcdefgh"),
            (1, 26, (O_M11, 2000, 1, 7, 0, 6, 2, 0), b"[2000/01/07T00:06:02 -1100]	abcdefgh"),
            (0, 21, (O_M11, 2020, 3, 7, 20, 25, 30, 0), b"20200307_202530 -1100 /sbin/e2fsck -pvf"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "137", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        137,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzc, RP_NODIGIT),
        // DfaU8, // fails to build
        FlatLockstepNfaU8,
        DTFSS_YmdHMSzc,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 26, (O_M1130, 2000, 1, 8, 0, 7, 3, 0), b"2000-01-08-00:07:03 -11:30 abcdefghi"),
            (0, 26, (O_M1130, 2000, 1, 8, 0, 7, 3, 0), b"2000-01-08-00:07:03 -11:30	abcdefghi"),
            (1, 27, (O_M1130, 2000, 1, 8, 0, 7, 3, 0), b"[2000-01-08-00:07:03 -11:30] abcdefghi"),
            (0, 22, (O_M11, 2020, 3, 7, 20, 25, 30, 0), b"20200307_202530 -11:00 /sbin/e2fsck -pvf"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "138", regex = "ALL", regex = "TEST"))]
    ERE_REGEX_DATETIME!(
        138,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZzp, RP_NODIGIT),
        //DfaU8, // fails to build
        FlatLockstepNfaU8,
        DTFSS_YmdHMSzp,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 23, (O_M11, 2000, 1, 9, 0, 8, 4, 0), b"2000/01/09 00:08:04 -11 abcdefghij"),
            (1, 24, (O_M11, 2000, 1, 9, 0, 8, 4, 0), b"[2000/01/09 00:08:04 -11] abcdefghij"),
            (0, 19, (O_M11, 2020, 3, 7, 20, 25, 30, 0), b"20200307_202530 -11 /sbin/e2fsck -pvf"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "139", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        139,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_YEAR, D_Dq, CGP_MONTHm, D_Dq, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZZ_U, RP_NOALPHA), // reduced
        //DfaU8, // 19m build time
        FlatLockstepNfaU8,
        DTFSS_YmdHMSZ,
        0, 1024,
        CGN_YEAR, CGN_TZ,
        &[
            (0, 24, (O_VLAT, 2000, 1, 10, 0, 9, 5, 0), b"2000/01/10T00:09:05 VLAT abcdefghijk"),
            (0, 24, (O_VLAT, 2000, 1, 10, 0, 9, 5, 0), b"2000/01/10T00:09:05 VLAT_abcdefghijk"),
            (1, 25, (O_VLAT, 2000, 1, 10, 0, 9, 5, 0), b"[2000/01/10T00:09:05 VLAT] abcdefghijk"),
            (1, 25, (O_VLAT, 2000, 1, 10, 0, 9, 5, 0), b"[2000/01/10T00:09:05 VLAT] abcdefghijk"),
            (1, 25, (O_VLAT, 2000, 1, 10, 0, 9, 5, 0), b"<2000/01/10T00:09:05 VLAT> abcdefghijk"),
            (1, 24, (O_VLAT, 2000, 1, 10, 0, 9, 5, 0), b"<2000/01/10T00:09:05VLAT> abcdefghijk"),
            (0, 20, (O_VLAT, 2020, 3, 7, 20, 25, 30, 0), b"20200307_202530 VLAT /sbin/e2fsck -pvf"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "140", regex = "ALL", regex = "TEST"))]
    ERE_REGEX_DATETIME!(
        140,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_YEAR, D_Deq, CGP_MONTHm, D_Deq, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NOALNUM),
        //DfaU8, // fails to build
        FlatLockstepNfaU8,
        DTFSS_YmdHMS,
        0, 512,
        CGN_YEAR, CGN_SECOND,
        &[
            (0, 19, (O_L, 2020, 1, 11, 0, 10, 26, 0), b"2020-01-11 00:10:26 abcdefghijkl"),
            (0, 19, (O_L, 2020, 1, 11, 0, 10, 26, 0), br"2020\01\11 00:10:26 abcdefghijkl"),
            (0, 15, (O_L, 2020, 3, 7, 20, 25, 30, 0), b"20200307_202530:/sbin/e2fsck -pvf"),
            // from `C:/Windows/Performance/WinSAT/winsat.log`
            // a datetime format with redundant `AM` and `PM`, see Issue #64
            (50, 69, (O_L, 2023, 2, 22, 16, 4, 7, 0), br"59805625 (9340) - exe\logging.cpp:0841: --- START 2023\02\22 16:04:07 PM ---"),
        ],
        line!(),
    ),
    // variation of prior using single-digit months and hours; Issue #64
    #[cfg(any(regex = "141", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        141,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_YEAR, D_Deq, CGP_MONTHms, D_Deq, CGP_DAYde, D_DHcdqu, CGP_HOUR_sd, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NOALNUM),
        FlatLockstepNfaU8,
        DTFSS_YmsdkMS,
        0, 512,
        CGN_YEAR, CGN_SECOND,
        &[
            (0, 17, (O_L, 2020, 1, 11, 0, 10, 26, 0), b"2020-1-11 0:10:26 abcdefghijkl 0"),
            (0, 18, (O_L, 2020, 12, 11, 0, 10, 26, 0), b"2020-12-11 0:10:26 abcdefghijkl 1"),
            (0, 17, (O_L, 2020, 1, 11, 0, 10, 26, 0), br"2020\1\11 0:10:26 abcdefghijkl 2"),
            (0, 18, (O_L, 2020, 1, 11, 14, 10, 26, 0), br"2020\1\11 14:10:26 abcdefghijkl 3"),
            (0, 13, (O_L, 2020, 3, 7, 4, 25, 30, 0), b"2020307_42530:/sbin/e2fsck -pvf"),
            (1, 14, (O_L, 2020, 3, 7, 4, 25, 30, 0), br"[2020307_42530] /sbin/e2fsck -pvf"),
            // from `C:/Windows/Performance/WinSAT/winsat.log`
            // a datetime format with redundant `AM` and `PM`, see Issue #64
            // with single-digit month and hour
            (50, 67, (O_L, 2023, 2, 22, 4, 5, 7, 0), br"59805625 (9340) - exe\logging.cpp:0841: --- START 2023\2\22 4:05:07 AM ---"),
            (50, 67, (O_L, 2023, 2, 22, 1, 5, 7, 0), br"59805625 (9340) - exe\logging.cpp:0841: --- START 2023\2\22 1:05:07 AM ---"),
        ],
        line!(),
    ),
    //
    // another general match variation
    //
    #[cfg(any(regex = "142", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        142,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZz, RP_NODIGIT),
        //DfaU8, // 28m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYz,
        0, 1024,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (8, 42, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"RSYSLOG Tuesday Jun 28 2022 01:51:12 +1230"),
            (8, 38, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"RSYSLOG Tue Jun 28 2022 01:51:12 +1230 FOOBAR"),
            (8, 39, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"RSYSLOG Tue, Jun 28 2022 01:51:12 +1230"),
            (8, 39, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), b"RSYSLOG Tue, Jun  2 2022 01:51:12 +1230"),
            (8, 39, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), b"RSYSLOG Tue, Jun 02 2022 01:51:12 +1230"),
            (8, 38, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), b"RSYSLOG Tue, Jun 2 2022 01:51:12 +1230"),
            (8, 38, (O_M11, 2022, 6, 2, 1, 51, 12, 0), b"RSYSLOG Tue, Jun 2 2022 01:51:12 -1100"),
            (8, 40, (O_M11, 2022, 6, 2, 1, 51, 12, 0), b"RSYSLOG Tue, Jun 2 2022 01:51:12 \xe2\x88\x921100"), // U+2212
            (8, 39, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"RSYSLOG Tue. Jun 28 2022 01:51:12 +1230 FOOBAR"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "143", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        143,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZzc, RP_NODIGIT),
        // DfaU8, // 29m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYzc,
        0, 1024,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (3, 35, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"<7>Tue, Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (4, 36, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"<33>Tue, Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (28, 60, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"[SOME OTHER FIELD] BLARG<33>Tue, Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (1, 33, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"*Tue, Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (3, 35, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"***Tue, Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (11, 43, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"[RSYSLOG]: Tue, Jun 28 2022 01:51:12 +01:30"),
            (8, 40, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"[INFO]: Tue. Jun 28 2022 01:51:12 +01:30:FOOBAR"),
            (7, 38, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"[INFO]:Tue Jun 28 2022 01:51:12 +01:30<33>FOOBAR"),
            (6, 37, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"[INFO]Tue Jun 28 2022 01:51:12 +01:30FOOBAR"),
            (7, 38, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), br"{INFO} Tue Jun 28 2022 01:51:12 +01:30 FOOBAR"),
            (7, 38, (O_M1_30, 2022, 6, 28, 1, 51, 12, 0), br"{INFO} Tue Jun 28 2022 01:51:12 -01:30 FOOBAR"),
            (7, 40, (O_M1_30, 2022, 6, 28, 1, 51, 12, 0), b"{INFO} Tue Jun 28 2022 01:51:12 \xe2\x88\x9201:30 FOOBAR with MINUS SIGN"), // U+2212
        ],
        line!(),
    ),
    #[cfg(any(regex = "144", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        144,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZzp, RP_NODIGIT),
        // DfaU8, // 27m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYzp,
        0, 1024,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (8, 41, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"[DEBUG] Tuesday, Jun 28 2022 01:51:12 +01"),
            (9, 38, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"[TRACE1] Tue. Jun 28 2022 01:51:12 +01 FOOBAR"),
            (9, 38, (O_P1, 2022, 6, 2, 1, 51, 12, 0), b"[TRACE1] Tue. Jun  2 2022 01:51:12 +01 FOOBAR"),
            (9, 38, (O_P1, 2022, 6, 2, 1, 51, 12, 0), b"[TRACE1] Tue. Jun 02 2022 01:51:12 +01 FOOBAR"),
            (9, 37, (O_P1, 2022, 6, 2, 1, 51, 12, 0), b"[TRACE1] Tue. Jun 2 2022 01:51:12 +01 FOOBAR"),
            (9, 38, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"[TRACE2] Tue, Jun 28 2022 01:51:12 +01"),
            (9, 37, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"[TRACE1] Tue Jun 28 2022 01:51:12 +01 FOOBAR"),
            (9, 37, (O_M1, 2022, 6, 28, 1, 51, 12, 0), b"[TRACE1] Tue Jun 28 2022 01:51:12 -01 FOOBAR"),
            (9, 39, (O_M1, 2022, 6, 28, 1, 51, 12, 0), b"[TRACE1] Tue Jun 28 2022 01:51:12 \xe2\x88\x9201 FOOBAR MINUS SIGN"), // U+2212
        ],
        line!(),
    ),
    #[cfg(any(regex = "145", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        145,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa3, RP_dcq, RP_BLANK12, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZZ_U, RP_NOALPHA), // reduced
        //DfaU8, // 27m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYZ,
        0, 1024,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (6, 36, (O_WITA, 2022, 6, 28, 1, 51, 12, 0), b"LOGGR Tue, Jun 28 2022 01:51:12 WITA:FOOBAR"),
            (6, 35, (O_WST, 2022, 6, 28, 1, 51, 12, 0), b"LOGGR Tue. Jun 28 2022 01:51:12 WST FOOBAR"),
            (8, 37, (O_YAKT, 2022, 6, 28, 1, 51, 12, 0), b"RSYSLOG Tue Jun 28 2022 01:51:12 YAKT"),
            (8, 37, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), b"RSYSLOG Tue Jun  2 2022 01:51:12 YAKT"),
            (8, 37, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), b"RSYSLOG Tue Jun 02 2022 01:51:12 YAKT"),
            (8, 36, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), b"RSYSLOG Tue Jun 2 2022 01:51:12 YAKT"),
            (8, 37, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), b"RSYSLOG Tue Jun 28 2022 01:51:12 YEKT FOOBAR"),
            (8, 37, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), b"RSYSLOG Tue jun 28 2022 01:51:12\tYEKT\t\tfoobar"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "146", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        146,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa3, RP_dcq, RP_BLANK12, CGP_MONTHB, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_TZZ_U, RP_NOALPHA), // reduced
        //DfaU8, // 27m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYZ,
        0, 1024,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (6, 37, (O_WITA, 2022, 6, 28, 1, 51, 12, 0), b"LOGGR Tue, June 28 2022 01:51:12 WITA:FOOBAR"),
            (6, 36, (O_WST, 2022, 6, 28, 1, 51, 12, 0), b"LOGGR Tue. JUNE 28 2022 01:51:12 WST FOOBAR"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "147", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        147,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_YEAR, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NOALNUM),
        // DfaU8, // 19m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSY,
        0, 1024,
        CGN_DAY_IGNORE, CGN_SECOND,
        &[
            (6, 35, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"LOGGR Tuesday, Jun 28 2022 01:51:12 "),
            (6, 35, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"LOGGR Tuesday, Jun 28 2022 01:51:12"),
            (6, 31, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"LOGGR Tue, Jun 28 2022 01:51:12 FOOBAR"),
            (7, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"blarg: Tue. Jun 28 2022 01:51:12 WST"),
            (8, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"RSYSLOG Tue Jun 28 2022 01:51:12[abc"),
            (8, 32, (O_L, 2022, 6, 2, 1, 51, 12, 0), b"RSYSLOG Tue Jun  2 2022 01:51:12[abc"),
            (8, 32, (O_L, 2022, 6, 2, 1, 51, 12, 0), b"RSYSLOG Tue Jun 02 2022 01:51:12;abc"),
            (8, 31, (O_L, 2022, 6, 2, 1, 51, 12, 0), b"RSYSLOG Tue Jun 2 2022 01:51:12 YAKT"),
            (8, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"RSYSLOG Tue Jun 28 2022 01:51:12 YEKT FOOBAR"),
            (8, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"RSYSLOG Tue Jun 28 2022 01:51:12 foobar"),
            (8, 33, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"RSYSLOG Tue June 28 2022 01:51:12               foobar"),
            (8, 33, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"RSYSLOG Tue JUNE 28 2022 01:51:12[YEKT]"),
            (6, 31, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"LOGGR|Tue june 28 2022 01:51:12|YEKT"),
        ],
        line!(),
    ),
    //
    #[cfg(any(regex = "148", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        148,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_YEAR, RP_BLANK12q, CGP_TZz, RP_NODIGIT),
        // DfaU8, // 29m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYz,
        0, 1024,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (27, 57, (O_P1230, 2023, 1, 12, 22, 26, 47, 0), b"ERROR: apport (pid 486722) Thu Jan 12 22:26:47 2023 +1230: called for pid 486450, signal 6, core limit 0, dump mode 1"),
            (8, 42, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"MESSAGE Tuesday Jun 28 01:51:12 2022 +1230"),
            (8, 38, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"MESSAGE Tue Jun 28 01:51:12 2022 +1230 FOOBAR"),
            (8, 39, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"MESSAGE Tue, Jun 28 01:51:12 2022 +1230"),
            (8, 39, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), b"MESSAGE Tue, Jun  2 01:51:12 2022 +1230"),
            (8, 39, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), b"MESSAGE Tue, Jun 02 01:51:12 2022 +1230"),
            (8, 38, (O_P1230, 2022, 6, 2, 1, 51, 12, 0), b"MESSAGE Tue, Jun 2 01:51:12 2022 +1230"),
            (8, 39, (O_P1230, 2022, 6, 28, 1, 51, 12, 0), b"MESSAGE Tue. Jun 28 01:51:12 2022 +1230 FOOBAR"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "149", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        149,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_YEAR, RP_BLANK12q, CGP_TZzc, RP_NODIGIT),
        // DfaU8, // 30+m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYzc,
        0, 1024,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (27, 58, (O_P1230, 2023, 1, 12, 22, 26, 47, 0), b"ERROR: apport (pid 486722) Thu Jan 12 22:26:47 2023 +12:30: called for pid 486450, signal 6, core limit 0, dump mode 1"),
            (3, 35, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"<7>Tue, Jun 28 01:51:12 2022 +01:30 FOOBAR"),
            (4, 36, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"<33>Tue, Jun 28 01:51:12 2022 +01:30 FOOBAR"),
            (28, 60, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"[SOME OTHER FIELD] BLARG<33>Tue, Jun 28 01:51:12 2022 +01:30 FOOBAR"),
            (1, 33, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"*Tue, Jun 28 01:51:12 2022 +01:30 FOOBAR"),
            (3, 35, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"***Tue, Jun 28 01:51:12 2022 +01:30 FOOBAR"),
            (11, 43, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"[MESSAGE]: Tue, Jun 28 01:51:12 2022 +01:30"),
            (8, 40, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"[INFO]: Tue. Jun 28 01:51:12 2022 +01:30:FOOBAR"),
            (7, 38, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"[INFO]:Tue Jun 28 01:51:12 2022 +01:30<33>FOOBAR"),
            (6, 37, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"[INFO]Tue Jun 28 01:51:12 2022 +01:30FOOBAR"),
            (7, 38, (O_P1_30, 2022, 6, 28, 1, 51, 12, 0), b"{INFO} Tue Jun 28 01:51:12 2022 +01:30 FOOBAR"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "150", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        150,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_YEAR, RP_BLANK12q, CGP_TZzp, RP_NODIGIT),
        // DfaU8, // 29m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYzp,
        0, 1024,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (27, 55, (O_P12, 2023, 1, 12, 22, 26, 47, 0), b"ERROR: apport (pid 486722) Thu Jan 12 22:26:47 2023 +12: called for pid 486450, signal 6, core limit 0, dump mode 1"),
            (8, 41, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"[DEBUG] Tuesday, Jun 28 01:51:12 2022 +01"),
            (9, 38, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"[TRACE1] Tue. Jun 28 01:51:12 2022 +01 FOOBAR"),
            (9, 38, (O_P1, 2022, 6, 2, 1, 51, 12, 0), b"[TRACE1] Tue. Jun  2 01:51:12 2022 +01 FOOBAR"),
            (9, 38, (O_P1, 2022, 6, 2, 1, 51, 12, 0), b"[TRACE1] Tue. Jun 02 01:51:12 2022 +01 FOOBAR"),
            (9, 37, (O_P1, 2022, 6, 2, 1, 51, 12, 0), b"[TRACE1] Tue. Jun 2 01:51:12 2022 +01 FOOBAR"),
            (9, 38, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"[TRACE2] Tue, Jun 28 01:51:12 2022 +01"),
            (9, 37, (O_P1, 2022, 6, 28, 1, 51, 12, 0), b"[TRACE1] Tue Jun 28 01:51:12 2022 +01 FOOBAR"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "151", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        151,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa3, RP_dcq, RP_BLANK12, CGP_MONTHb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_YEAR, RP_BLANK12q, CGP_TZZ_U, RP_NOALPHA), // reduced
        // DfaU8, // 28m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYZ,
        0, 1024,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (27, 56, (O_WITA, 2023, 1, 12, 22, 26, 47, 0), b"ERROR: apport (pid 486722) Thu Jan 12 22:26:47 2023 WITA: called for pid 486450, signal 6, core limit 0, dump mode 1"),
            //(6, 39, (O_WIT, 2022, 6, 28, 1, 51, 12, 0), "MESSG Tuesday, Jun 28 01:51:12 2022 WIT"),
            (6, 36, (O_WITA, 2022, 6, 28, 1, 51, 12, 0), b"MESSG Tue, Jun 28 01:51:12 2022 WITA:FOOBAR"),
            (6, 35, (O_WST, 2022, 6, 28, 1, 51, 12, 0), b"MESSG Tue. Jun 28 01:51:12 2022 WST FOOBAR"),
            (8, 37, (O_YAKT, 2022, 6, 28, 1, 51, 12, 0), b"MESSAGE Tue Jun 28 01:51:12 2022 YAKT"),
            (8, 37, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), b"MESSAGE Tue Jun  2 01:51:12 2022 YAKT"),
            (8, 37, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), b"MESSAGE Tue Jun 02 01:51:12 2022 YAKT"),
            (8, 36, (O_YAKT, 2022, 6, 2, 1, 51, 12, 0), b"MESSAGE Tue Jun 2 01:51:12 2022 YAKT"),
            (8, 37, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), b"MESSAGE Tue Jun 28 01:51:12 2022 YEKT FOOBAR"),
            (8, 38, (O_YEKT, 2022, 6, 28, 1, 51, 12, 0), b"MESSAGE Tue jun 28 01:51:12 2022  YEKT\tfoobar"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "152", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        152,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa, RP_dcq, RP_BLANK12, CGP_MONTHBb, RP_BLANK, CGP_DAYde, RP_cq, RP_BLANK12, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK12, CGP_YEAR, RP_NOALNUM),
        // DfaU8, // 27m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSY,
        0, 1024,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (27, 51, (O_L, 2023, 1, 12, 22, 26, 47, 0), b"ERROR: apport (pid 486722) Thu Jan 12 22:26:47 2023: called for pid 486450, signal 6, core limit 0, dump mode 1"),
            (6, 31, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"MESSG Tue, Jun 28 01:51:12 2022 FOOBAR"),
            (7, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"messg: Tue. Jun 28 01:51:12 2022 WST"),
            (8, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"MESSAGE Tue Jun 28 01:51:12 2022[abc"),
            (8, 32, (O_L, 2022, 6, 2, 1, 51, 12, 0), b"MESSAGE Tue Jun  2 01:51:12 2022[abc"),
            (8, 32, (O_L, 2022, 6, 2, 1, 51, 12, 0), b"MESSAGE Tue Jun 02 01:51:12 2022;abc"),
            (8, 31, (O_L, 2022, 6, 2, 1, 51, 12, 0), b"MESSAGE Tue Jun 2 01:51:12 2022 YAKT"),
            (8, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"MESSAGE Tue Jun 28 01:51:12 2022 YEKT FOOBAR"),
            (8, 32, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"MESSAGE Tue Jun 28 01:51:12 2022 foobar"),
            (8, 33, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"MESSAGE Tue June 28 01:51:12 2022               foobar"),
            (8, 33, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"FOOBAR! Tue JUNE 28 01:51:12 2022[YEKT]"),
            (8, 33, (O_L, 2022, 6, 28, 1, 51, 12, 0), b"MESSAGE|Tue JUNE 28 01:51:12 2022|YEKT|foobar!"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // file `FedoraRemix29/hawkeye.log` (with many variations)
    //
    //                1         2         3         4
    //      01234567890123456789012345678901234567890
    //      INFO Jun-16 14:09:58 === Started libdnf-0.31.0 ===
    //      DEBUG Jun-16 14:09:58 fetching rpmdb
    //
    #[cfg(any(regex = "153", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        153,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, RP_BLANKSq, "[:]?", RP_BLANKSq, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZZ_U, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSYZ,
        0, 64,
        CGN_MONTH, CGN_TZ,
        &[
            (5, 29, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"INFO Jun-16 14:09:58 2000 PDT === Started libdnf-0.31.0 ==="),
            (6, 30, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"DEBUG Jun 16 14:09:58 2000 PDT fetching rpmdb"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "154", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        154,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, RP_BLANKSq, "[:]?", RP_BLANKSq, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZzc, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSYzc,
        0, 64,
        CGN_MONTH, CGN_TZ,
        &[
            (5, 32, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"INFO Jun-16 14:09:58 2000 -07:00 === Started libdnf-0.31.0 ==="),
            (6, 33, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"DEBUG Jun 16 14:09:58 2000 -07:00 fetching rpmdb"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "155", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        155,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, RP_BLANKSq, "[:]?", RP_BLANKSq, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZz, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSYz,
        0, 64,
        CGN_MONTH, CGN_TZ,
        &[
            (5, 31, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"INFO Jun-16 14:09:58 2000 -0700 === Started libdnf-0.31.0 ==="),
            (6, 32, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"DEBUG Jun 16 14:09:58 2000 -0700 fetching rpmdb"),
            (6, 34, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"DEBUG Jun 16 14:09:58 2000 \xe2\x88\x920700 fetching rpmdb MINUS SIGN"), // U+2212
        ],
        line!(),
    ),
    #[cfg(any(regex = "156", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        156,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, RP_BLANKSq, "[:]?", RP_BLANKSq, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZzp, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSYzp,
        0, 64,
        CGN_MONTH, CGN_TZ,
        &[
            (5, 29, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"INFO Jun-16 14:09:58 2000 -07 === Started libdnf-0.31.0 ==="),
            (6, 30, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"DEBUG Jun 16 14:09:58 2000 -07 fetching rpmdb"),
            (6, 32, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"DEBUG Jun 16 14:09:58 2000 \xe2\x88\x9207 fetching rpmdb MINUS SIGN"), // U+2212
        ],
        line!(),
    ),
    #[cfg(any(regex = "157", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        157,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, RP_BLANKSq, "[:]?", RP_BLANKSq, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMSY,
        0, 64,
        CGN_MONTH, CGN_YEAR,
        &[
            (5, 25, (O_L, 2000, 6, 16, 14, 9, 58, 0), b"INFO Jun-16 14:09:58 2000 === Started libdnf-0.31.0 ==="),
            (6, 26, (O_L, 2000, 6, 16, 14, 9, 58, 0), b"DEBUG Jun 16 14:09:58 2000 fetching rpmdb"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "158", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        158,
        counter!(DP_KEY),
        concat!("^", RP_LEVELS, RP_BLANKSq, "[:]?", RP_BLANKSq, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NOALNUM),
        DfaU8,
        DTFSS_BdHMS,
        0, 64,
        CGN_MONTH, CGN_SECOND,
        &[
            (5, 20, (O_L, YD, 6, 16, 14, 9, 58, 0), b"INFO Jun-16 14:09:58 === Started libdnf-0.31.0 ==="),
            (6, 21, (O_L, YD, 6, 16, 14, 9, 58, 0), b"DEBUG Jun-16 14:09:58 fetching rpmdb"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // dmesg "uptime" format, example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     [    0.000000] kernel: Linux version 5.15.0-43-generic (build@lcy02-amd64-076) (gcc (Ubuntu 11.2.0-19ubuntu1) 11.2.0, GNU ld (GNU Binutils for Ubuntu) 2.38) #46-Ubuntu SMP Tue Jul 12 10:30:17 UTC 2022 (Ubuntu 5.15.0-43.46-generic 5.15.39)
    //     [    0.000001] kernel: Command line: BOOT_IMAGE=/boot/vmlinuz-5.15.0-43-generic root=UUID=136735fa-5cc1-470f-9359-ee736e42f844 ro console=tty1 console=ttyS0 net.ifnames=0 biosdevname=0
    //     [    0.000002] kernel: KERNEL supported cpus:
    //     [    0.000002] kernel:   Intel GenuineIntel
    //
    #[cfg(any(regex = "159", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        159,
        counter!(DP_KEY),
        concat!(r"^\[", RP_BLANKSq, CGP_UPTIME_F, r"\]", RP_BLANK),
        DfaU8,
        DTFSS_u,
        0, 20,
        CGN_UPTIME, CGN_FRACTIONAL,
        &[
            (5, 13, (O_L, 1970, 1, 1, 0, 0, 1, 3000000), b"[    1.003000] kernel: Linux version 5.15.0-48-generic (buildd@lcy02-amd64-080) (gcc (Ubuntu 11.2.0-19ubuntu1) 11.2.0, GNU ld (GNU Binutils for Ubuntu) 2.38) #54-Ubuntu SMP Fri Aug 26 13:26:29 UTC 2022 (Ubuntu 5.15.0-48.54-generic 5.15.53)"),
            (4, 13, (O_L, 1970, 1, 1, 0, 0, 15, 364159000), b"[   15.364159] kernel: ISO 9660 Extensions: RRIP_1991A"),
            (1, 15, (O_L, 1970, 1, 21, 0, 40, 35, 564122000), b"[1730435.564122] wireguard: wg1: Handshake for peer 481 ((einval)) did not complete after 20 attempts, giving up"),
        ],
        line!(),
    ),
    //
    // lightdm.log "uptime" format, example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     [+0.00s] DEBUG: Logging to /var/log/lightdm/lightdm.log
    //     [+2.80s] DEBUG: XServer 0: Got signal from X server :0
    //     [+2147.35s] DEBUG: Seat seat0: Display server stopped
    //
    #[cfg(any(regex = "160", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        160,
        counter!(DP_KEY),
        concat!(r"^\[", RP_BLANKSq, r"\+", CGP_UPTIME_F23, r"s\]", RP_BLANK),
        DfaU8,
        DTFSS_u,
        0, 25,
        CGN_UPTIME, CGN_FRACTIONAL,
        &[
            (2, 6, (O_L, 1970, 1, 1, 0, 0, 0, 0), b"[+0.00s] DEBUG: Logging to /var/log/lightdm/lightdm.log"),
            (5, 12, (O_L, 1970, 1, 1, 0, 35, 47, 350000000), b"[   +2147.35s] DEBUG: Seat seat0: Display server stopped"),
            (2, 9, (O_L, 1970, 1, 1, 0, 35, 47, 350000000), b"[+2147.35s] DEBUG: Seat seat0: Display server stopped"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // same pattern as prior without specifying leading RP_LEVELS, e.g. `DEBUG`, with leading day of week
    //
    #[cfg(any(regex = "161", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        161,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa3, RP_BLANK, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZZ_U, RP_NOALNUM),
        // DfaU8, // 30+m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYZ,
        0, 400,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (5, 33, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Fri Jun-16 14:09:58 2000 PDT === Started libdnf-0.31.0 ==="),
            (6, 34, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Fri Jun 16 14:09:58 2000 PDT fetching rpmdb"),
            (6, 34, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Fri Jun 16 14:09:58 2000 PDT"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "162", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        162,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa3, RP_BLANK, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZzc, RP_NOALNUM),
        // DfaU8, // 21m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYzc,
        0, 400,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (5, 36, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Fri Jun-16 14:09:58 2000 -07:00 === Started libdnf-0.31.0 ==="),
            (6, 37, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Fri Jun 16 14:09:58 2000 -07:00 fetching rpmdb"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "163", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        163,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa3, RP_BLANK, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZz, RP_NOALNUM),
        // DfaU8, // 21m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYz,
        0, 400,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (5, 35, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Fri Jun-16 14:09:58 2000 -0700 === Started libdnf-0.31.0 ==="),
            (6, 36, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Fri Jun 16 14:09:58 2000 -0700 fetching rpmdb"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "164", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        164,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa3, RP_BLANK, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZzp, RP_NOALNUM),
        // DfaU8, // 21m buld time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYzp,
        0, 400,
        CGN_DAY_IGNORE, CGN_TZ,
        &[
            (5, 33, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Fri Jun-16 14:09:58 2000 -07 === Started libdnf-0.31.0 ==="),
            (6, 34, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Fri Jun 16 14:09:58 2000 -07 fetching rpmdb"),
        ],
        line!(),
    ),
    // same pattern as prior swapped year and timezone
    #[cfg(any(regex = "165", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        165,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa3, RP_BLANK, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZZ_U, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        // DfaU8, // 30+m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYZ,
        0, 400,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (5, 33, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Fri Jun-16 14:09:58 PDT 2000 === Started libdnf-0.31.0 ==="),
            (6, 34, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Fri Jun 16 14:09:58 PDT 2000 fetching rpmdb"),
            (6, 34, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Fri Jun 16 14:09:58 PDT 2000"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "166", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        166,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa3, RP_BLANK, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZzc, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        // DfaU8, // 21m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYzc,
        0, 400,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (5, 36, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Fri Jun-16 14:09:58 -07:00 2000 === Started libdnf-0.31.0 ==="),
            (6, 37, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Fri Jun 16 14:09:58 -07:00 2000 fetching rpmdb"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "167", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        167,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa3, RP_BLANK, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZz, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        // DfaU8, // 21m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYz,
        0, 400,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (5, 35, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Fri Jun-16 14:09:58 -0700 2000 === Started libdnf-0.31.0 ==="),
            (6, 36, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Fri Jun 16 14:09:58 -0700 2000 fetching rpmdb"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "168", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        168,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_DAYa3, RP_BLANK, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZzp, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        // DfaU8, // 21m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYzp,
        0, 400,
        CGN_DAY_IGNORE, CGN_YEAR,
        &[
            (5, 33, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Fri Jun-16 14:09:58 -07 2000 === Started libdnf-0.31.0 ==="),
            (6, 34, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Fri Jun 16 14:09:58 -07 2000 fetching rpmdb"),
        ],
        line!(),
    ),
    // same pattern as prior without leading day of week
    #[cfg(any(regex = "169", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        169,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZZ_U, RP_NOALNUM),
        // DfaU8, // 27m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYZ,
        0, 400,
        CGN_MONTH, CGN_TZ,
        &[
            (5, 29, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Jun-16 14:09:58 2000 PDT === Started libdnf-0.31.0 ==="),
            (6, 30, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Jun 16 14:09:58 2000 PDT fetching rpmdb"),
            (6, 30, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Jun 16 14:09:58 2000 PDT"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "170", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        170,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZzc, RP_NOALNUM),
        //DfaU8, // 12m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYzc,
        0, 400,
        CGN_MONTH, CGN_TZ,
        &[
            (5, 32, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Jun-16 14:09:58 2000 -07:00 === Started libdnf-0.31.0 ==="),
            (6, 33, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Jun 16 14:09:58 2000 -07:00 fetching rpmdb"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "171", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        171,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZz, RP_NOALNUM),
        //DfaU8, // 12m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYz,
        0, 400,
        CGN_MONTH, CGN_TZ,
        &[
            (5, 31, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Jun-16 14:09:58 2000 -0700 === Started libdnf-0.31.0 ==="),
            (6, 32, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Jun 16 14:09:58 2000 -0700 fetching rpmdb"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "172", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        172,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_BLANK, CGP_TZzp, RP_NOALNUM),
        // DfaU8, // 11m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYzp,
        0, 400,
        CGN_MONTH, CGN_TZ,
        &[
            (5, 29, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Jun-16 14:09:58 2000 -07 === Started libdnf-0.31.0 ==="),
            (6, 30, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Jun 16 14:09:58 2000 -07 fetching rpmdb"),
        ],
        line!(),
    ),
    // same pattern as prior but swapped year and timezone
    #[cfg(any(regex = "173", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        173,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZZ_U, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        // DfaU8, // 30+m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYZ,
        0, 400,
        CGN_MONTH, CGN_YEAR,
        &[
            (5, 29, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"____ Jun-16 14:09:58 PDT 2000 === Started libdnf-0.31.0 ==="),
            (6, 30, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Jun 16 14:09:58 PDT 2000 fetching rpmdb"),
            (6, 30, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"_____ Jun 16 14:09:58 PDT 2000\n"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "174", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        174,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZzc, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        // DfaU8, // 12m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYzc,
        0, 400,
        CGN_MONTH, CGN_YEAR,
        &[
            (5, 32, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"INFO Jun-16 14:09:58 -07:00 2000 === Started libdnf-0.31.0 ==="),
            (6, 33, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"DEBUG Jun 16 14:09:58 -07:00 2000 fetching rpmdb"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "175", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        175,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZz, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        // DfaU8, // 11m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYz,
        0, 400,
        CGN_MONTH, CGN_YEAR,
        &[
            (5, 31, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"INFO Jun-16 14:09:58 -0700 2000 === Started libdnf-0.31.0 ==="),
            (6, 32, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"DEBUG Jun 16 14:09:58 -0700 2000 fetching rpmdb"),
        ],
        line!(),
    ),
    #[cfg(any(regex = "176", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        176,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZzp, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        // DfaU8, // 11m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSYzp,
        0, 400,
        CGN_MONTH, CGN_YEAR,
        &[
            (5, 29, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"INFO Jun-16 14:09:58 -07 2000 === Started libdnf-0.31.0 ==="),
            (6, 30, (O_M7, 2000, 6, 16, 14, 9, 58, 0), b"DEBUG Jun 16 14:09:58 -07 2000 fetching rpmdb"),
        ],
        line!(),
    ),
    // same pattern as prior without timezone
    #[cfg(any(regex = "177", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        177,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_YEAR, RP_NOALNUM),
        // DfaU8, // 13m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMSY,
        0, 400,
        CGN_MONTH, CGN_YEAR,
        &[
            (5, 25, (O_L, 2000, 6, 16, 14, 9, 58, 0), b"INFO Jun-16 14:09:58 2000 === Started libdnf-0.31.0 ==="),
            (6, 26, (O_L, 2000, 6, 16, 14, 9, 58, 0), b"DEBUG Jun 16 14:09:58 2000 fetching rpmdb"),
        ],
        line!(),
    ),
    // same pattern as prior without timezone or year
    #[cfg(any(regex = "178", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        178,
        counter!(DP_KEY),
        concat!(RP_NOALPHAb, CGP_MONTHBb, D_D, CGP_DAYde, D_DHcdqu, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_NOALNUM),
        // DfaU8, 7m build time
        FlatLockstepNfaU8,
        DTFSS_BdHMS,
        0, 400,
        CGN_MONTH, CGN_SECOND,
        &[
            (5, 20, (O_L, YD, 6, 16, 14, 9, 58, 0), b"INFO Jun-16 14:09:58 === Started libdnf-0.31.0 ==="),
            (6, 21, (O_L, YD, 6, 16, 14, 9, 58, 0), b"DEBUG Jun-16 14:09:58 fetching rpmdb"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // Squirrel-Install.log
    //
    // [29-08-24 13:17:25] info: Program: Starting Squirrel Updater: --install . --rerunningWithoutUAC
    // [29-08-24 13:17:25] info: Program: Starting install, writing to C:\Users\user1\AppData\Local\SquirrelTemp
    //
    #[cfg(any(regex = "179", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        179,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_DAYd, D_D, CGP_MONTHm, D_D, CGP_YEARy, D_DHcd, CGP_HOUR, D_Tcd, CGP_MINUTE, D_Tcd, CGP_SECOND, RP_RB),
        DfaU8,
        DTFSS_ymdHMS,
        0, 40,
        CGN_DAY, CGN_SECOND,
        &[
            (1, 18, (O_L, 2024, 8, 29, 13, 17, 25, 0), b"[29-08-24 13:17:25] info: Program: Starting Squirrel Updater: --install . --rerunningWithoutUAC"),
        ],
        line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // Month/Day/Year formats
    //
    // ---------------------------------------------------------------------------------------------
    //
    // UTF-16LE
    //
    // ./logs/Windows10Pro/SysWOW64/Macromed/Flash/FlashInstall32.log
    //
    //      =X====== M/32.0.0.114 2019-01-29+02-07-27.809 ========
    //      0000 [I] 00000044
    //      0001 [I] 00000045
    //      0002 [W] 00001113 C:\Users\user\AppData\Roaming\Macromedia\Flash Player\www.macromedia.com\bin\* 3
    //      ...
    //      2021-1-3+0-4-2.265 [error] 1223 1056
    //      2021-4-26+0-4-1.791 [error] 1223 1056
    //      2021-5-1+21-39-33.489 [error] 1226 1062
    //      =O====== M/32.0.0.465 2021-05-01+21-39-33.382 ========
    //      ...
    //
    // This log is sloppy: besides having an nonstandard '+' DT separator,
    // worse is it switches formats depending upon the message type. 🙄🙄🙄
    // So the regex here is pretty loose.
    //
    #[cfg(any(regex = "180", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        180,
        counter!(DP_KEY),
        concat!("(^|[[:blank:]])", CGP_YEAR, "-", CGP_MONTHm_sd, "-", CGP_DAYde, r"[\+T]", CGP_HOUR_sd, "-", CGP_MINUTE_sd, "-", CGP_SECOND_sd, D_SF, CGP_FRACTIONAL369, RP_NOALNUMpm),
        DfaU8,
        DTFSS_Ymdkmsf,
        0, 150,
        CGN_YEAR, CGN_FRACTIONAL,
        &[
            (1, 24, (O_L, 2019, 1, 29, 2, 7, 27, 809000000), b" 2019-01-29+02-07-27.809 "),
            (22, 45, (O_L, 2019, 1, 29, 2, 7, 27, 809000000), b"=X====== M/32.0.0.114 2019-01-29+02-07-27.809 ========"),
            (24, 47, (O_L, 2019, 2, 26, 3, 21, 52, 720000000), b"\0\0=O====== M/32.0.0.142 2019-02-26+03-21-52.720 ========"),
            (0, 19, (O_L, 2021, 4, 26, 0, 4, 1, 791000000), b"2021-4-26+0-4-1.791 [error] 1 1056"),
            (0, 20, (O_L, 2021, 4, 26, 10, 4, 1, 791000000), b"2021-4-26+10-4-1.791 [error] 2 1056"),
            (0, 20, (O_L, 2021, 4, 6, 0, 14, 1, 791000000), b"2021-04-6+0-14-1.791 [error] 3 1056"),
            (0, 20, (O_L, 2021, 4, 6, 0, 4, 21, 791000000), b"2021-04-6+0-4-21.791 [error] 4 1056"),
            (0, 19, (O_L, 2021, 4, 6, 0, 4, 1, 791000000), b"2021-04-6+0-4-1.791 [error] 5 1056"),
            (0, 19, (O_L, 2021, 4, 6, 0, 4, 1, 791000000), b"2021-4-06+0-4-1.791 [error] 6 1056"),
            (0, 19, (O_L, 2021, 4, 6, 0, 4, 1, 791000000), b"2021-4-6+00-4-1.791 [error] 7 1056"),
            (0, 19, (O_L, 2021, 4, 6, 0, 4, 1, 791000000), b"2021-4-6+0-04-1.791 [error] 8 1056"),
            (0, 19, (O_L, 2021, 4, 6, 0, 4, 1, 791000000), b"2021-4-6+0-4-01.791 [error] 9 1056"),
            (0, 18, (O_L, 2021, 4, 6, 0, 4, 1, 791000000), b"2021-4-6+0-4-1.791 [error] 10 1056"),
        ],
        line!(),
    ),
    // ./logs/Windows11Pro/setupact.log
    //
    //      [02/21/2023 07:07.05.259] WudfCoInstaller: ReadWdfSection: Checking WdfSection [Basic_Install.Wdf]
    //      [02/21/2023 07:07.05.262] WudfCoInstaller: Configuring UMDF Service WpdFs.
    //
    // C:/Users/user1/AppData/Local/Cache/Microsoft/MSTeams/Logs/tma_addin_msi.log
    //
    //      INFO   : [04/10/2025 12:45:55:442] [CheckFX                                 ]: Custom Action is starting...
    //      INFO   : [04/10/2025 12:45:55:442] [CheckFX                                 ]: CoInitializeEx - COM initialization Apartment Threaded...
    //
    #[cfg(any(regex = "181", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        181,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_MONTHm, D_D, CGP_DAYd, D_D, CGP_YEAR, D_DHcd, CGP_HOUR, D_Teq, CGP_MINUTE, D_Tcd, CGP_SECOND, D_Tcdc, CGP_FRACTIONAL369, RP_RB),
        DfaU8,
        DTFSS_YmdHMSf,
        0, 90,
        CGN_MONTH, CGN_FRACTIONAL,
        &[
            (1, 24, (O_L, 2023, 2, 21, 7, 7, 5, 262000000), b"[02/21/2023 07:07.05.262] WudfCoInstaller: Configuring UMDF Service WpdFs."),
            (10, 33, (O_L, 2025, 4, 10, 12, 45, 55, 442000000), b"INFO   : (04/10/2025 12:45:55:442) [CheckFX                                 ]: CoInitializeEx - COM initialization Apartment Threaded..."),
        ],
        line!(),
    ),
    #[cfg(any(regex = "182", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        182,
        counter!(DP_KEY),
        concat!(RP_LB, CGP_MONTHm, D_D, CGP_DAYd, D_D, CGP_YEAR, D_DHcd, CGP_HOUR, D_Te, CGP_MINUTE, D_Tcd, CGP_SECOND, RP_RB),
        DfaU8,
        DTFSS_YmdHMS,
        0, 80,
        CGN_MONTH, CGN_SECOND,
        &[
            (1, 20, (O_L, 2023, 2, 21, 7, 9, 5, 0), b"[02/21/2023 07:09.05] WudfCoInstaller: Configuring UMDF Service WpdFs."),
            (6, 25, (O_L, 2025, 4, 10, 12, 45, 55, 0), b"INFO (04/10/2025 12:45:55) [CheckFX                                 ]: CoInitializeEx - COM initialization Apartment Threaded..."),
        ],
        line!(),
    ),
    // ./logs/Windows11Pro/Local/Microsoft/CLR_v4.0_32/ngen.log
    //
    //      10/11/2022 14:14:04.751 [7712]: Native image used by one or more roots, cannot be uninstalled
    //      10/11/2022 14:14:04.754 [7712]: ngen returning 0x00000000
    //
    #[cfg(any(regex = "183", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        183,
        counter!(DP_KEY),
        concat!("^", CGP_MONTHm_sd, D_D, CGP_DAYd, D_D, CGP_YEAR, D_DHcd, CGP_HOUR_sd, D_Tcd, CGP_MINUTE_sd, D_Tcd, CGP_SECOND_sd, D_Tcdc, CGP_FRACTIONAL369, RP_NOALNUMpm),
        DfaU8,
        DTFSS_YmdHMSf,
        0, 46,
        CGN_MONTH, CGN_FRACTIONAL,
        &[
            (0, 23, (O_L, 2022, 10, 11, 13, 14, 9, 751000000), b"10/11/2022 13:14:09.751 [7712]: Native image used by one or more roots, cannot be uninstalled"),
        ],
        line!(),
    ),
    // UTF-16LE
    //
    // ./logs/Windows11Pro/Local/Microsoft/Internet Explorer/ie4uinit-ClearIconCache.log
    //
    //      ��02/21/2023:06:27:45: Starting ie4uinit.exe. Command Line:-ClearIconCache
    //      02/21/2023:06:27:45: Executing Command: -ClearIconCache
    //      02/21/2023:06:27:45: In CmdClearIconCache
    //
    // ./logs/Windows10Pro/PFRO.log
    //
    //      11/28/2018 19:17:56 - PFRO Error: \??\C:\Users\user1\AppData\Local\Temp\nst4E80.tmp\, |delete operation|, 0xc0000034
    //      11/28/2018 19:17:56 - 6 Successful PFRO operations
    //      05-07-2022 05:28 : DTC Install error = 0, Action: None, o
    //
    #[cfg(any(regex = "184", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        184,
        counter!(DP_KEY),
        concat!("^", CGP_MONTHm_sd, D_D, CGP_DAYd, D_D, CGP_YEAR, D_DHcd, CGP_HOUR_sd, D_Te, CGP_MINUTE_sd, D_Te, CGP_SECOND_sd, RP_NOALNUMpm),
        DfaU8,
        DTFSS_Ymdkms,
        0, 34,
        CGN_MONTH, CGN_SECOND,
        &[
            (0, 19, (O_L, 2023, 2, 21, 6, 27, 45, 0), b"02/21/2023:06:27:45: Starting ie4uinit.exe. Command Line:-ClearIconCache"),
            (0, 19, (O_L, 2018, 11, 28, 19, 17, 56, 0), br"11/28/2018 19:17:56 - PFRO Error: \??\C:\Users\user1\AppData\Local\Temp\nst4E80.tmp\, |delete operation|, 0xc0000034"),
            (0, 17, (O_L, 2021, 6, 23, 20, 1, 41, 0), b"6/23/2021 20:1:41 - 0 Successful PFRO operations"),
        ],
        line!(),
    ),
    //
    // TODO: consider pattern `09/12/2022 @ 7:05am`
    //
    // C:/Users/user1/AppData/Local/Temp/vivaldi_installer.log
    //
    //      [0509/110534.597:ERROR:installer\mini_installer\setup\install_worker.cc:152] Failed creating a firewall rules. Continuing with install.
    //      [0509/110534.660:VERBOSE1:installer\util\vivaldi_setup_util.cc:445] Initial command line:
    //
    #[cfg(any(regex = "185", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        185,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_MONTHm, D_Deq, CGP_DAYd, D_DHcds, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, D_SF, CGP_FRACTIONAL369, RP_NOALNUM),
        // DfaU8, // exceeded DFA state limit of 162
        FlatLockstepNfaU8,
        DTFSS_mdHMSf,
        0, 60,
        CGN_MONTH, CGN_FRACTIONAL,
        &[
            (1, 16, (O_L, YD, 5, 9, 11, 5, 34, 660000000), br#"[0509/110534.660:VERBOSE1:installer\util\vivaldi_setup_util.cc:445] Initial command line:"#),
            (1, 19, (O_L, YD, 5, 9, 11, 5, 34, 660000000), br#"[05/09/11:05:34.660:VERBOSE1:installer\util\vivaldi_setup_util.cc:445] Initial command line:"#),
        ],
        line!(),
    ),
    //
    // C:/Users/user1/AppData/Local/Temp/cv_debug.log
    //
    //      {"logTime": "0425/073721", "correlationVector":"63EBBED7FB5845DDB9AF2810D983A3BD","action":"FETCH_UX_CONFIG", "result":""}
    //      {"logTime": "0425/073750", "correlationVector":"8Ffe+ZgWUZAP9cYd0PWnWm","action":"EXTENSION_UPDATE_SERVICE", "result":""}
    //
    #[cfg(any(regex = "186", regex = "ALL"))]
    ERE_REGEX_DATETIME!(
        186,
        counter!(DP_KEY),
        concat!(RP_NOALNUMb, CGP_MONTHm, D_Deq, CGP_DAYd, D_DHcds, CGP_HOUR, D_Teq, CGP_MINUTE, D_Teq, CGP_SECOND, RP_NOALNUM),
        // DfaU8, // exceeded DFA state limit of 162
        FlatLockstepNfaU8,
        DTFSS_mdHMS,
        0, 60,
        CGN_MONTH, CGN_SECOND,
        &[
            (13, 24, (O_L, YD, 4, 25, 7, 37, 50, 0), br#"{"logTime": "0425/073750", "correlationVector":"8Ffe+ZgWUZAP9cYd0PWnWm","action":"EXTENSION_UPDATE_SERVICE", "result":""}"#),
        ],
        line!(),
    ),
];
/// proc-macro generated count of compiled regex.
/// May be less than the number of possible entries in `DATETIME_PARSE_DATAS`.
/// This value depends upon build cfg of env var `S4_BUILD_REGEX`.
pub const DATETIME_PARSE_DATAS_LEN: usize = counter_last!(DP_KEY);
/// the maximum possible length of `DATETIME_PARSE_DATAS`
pub const DATETIME_PARSE_DATAS_LEN_MAX: usize = 186;

/// Check if the `regex_id` is in the compiled `DATETIME_PARSE_DATAS`.
/// `DATETIME_PARSE_DATAS` may vary depending upon build cfg of env var `REGEX`.
/// A value of `0` returns `true`.
pub fn regex_id_compiled(regex_id: RegexId) -> bool {
    if regex_id == 0 {
        return true;
    }
    DATETIME_PARSE_DATAS.iter().any(|dtpd| dtpd.regex_id == regex_id)
}
