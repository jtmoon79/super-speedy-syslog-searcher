// src/data/datetime.rs

// ‥
// …

//! Functions to perform regular expression ("regex") searches on bytes and
//! transform matches to chrono [`DateTime`] instances.
//!
//! Parsing bytes and finding datetime strings requires:
//! 1. searching some slice of bytes from a [`Line`] for a regular expression match.
//! 2. using a [`DateTimeParseInstr`], attempting to transform the matched regular expression named
//!    capture groups into data passable to chrono [`DateTime::parse_from_str`] or
//!    [`NaiveDateTime::parse_from_str`].
//! 3. return chrono `DateTime` instances along with byte offsets of the found matches to a caller
//!    (who will presumably use it create a new [`Sysline`]).
//!
//! The most relevant documents to understand this file are:
//! - `chrono` crate [`strftime`] format.
//! - `regex` crate [Regular Expression syntax].
//!
//! The most relevant functions are:
//! 1. [`bytes_to_regex_to_datetime`] which calls private function:
//! 2. [`captures_to_buffer_bytes`]
//!
//! Uses regular expressions defined in [`DATETIME_PARSE_DATAS`].
//!
//! [`DATETIME_PARSE_DATAS`]: crate::data::datetime::DATETIME_PARSE_DATAS
//! [`Line`]: crate::data::line::Line
//! [`Sysline`]: crate::data::sysline::Sysline
//! [`DateTime`]: https://docs.rs/chrono/0.4.38/chrono/struct.DateTime.html
//! [`DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/struct.DateTime.html#method.parse_from_str
//! [`NaiveDateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/naive/struct.NaiveDateTime.html#method.parse_from_str
//! [`strftime`]: https://docs.rs/chrono/0.4.38/chrono/format/strftime/index.html
//! [Regular Expression syntax]: https://docs.rs/regex/1.10.5/regex/index.html#syntax

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::time::Duration as StdDuration;
#[doc(hidden)]
pub use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

#[doc(hidden)]
pub use ::chrono::{
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
use ::more_asserts::{
    debug_assert_ge,
    debug_assert_le,
    debug_assert_lt,
};
use ::numtoa::NumToA; // adds `numtoa` method to numbers
use ::phf::{
    phf_map,
    Map as PhfMap,
};
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

pub use ::ere_datetimes_impl::{
    CaptureGroupName,
    CaptureGroupPattern,
    CGI_YEAR,
    CGI_MONTH,
    CGI_DAY,
    CGI_DAY_IGNORE,
    CGI_HOUR,
    CGI_MINUTE,
    CGI_SECOND,
    CGI_FRACTIONAL,
    CGI_TZ,
    CGI_UPTIME,
    CGI_EPOCH,
    CGN_ALL,
    DateTimeParseInstr,
    DateTimePattern_str,
    DateTimePattern_string,
    DATETIME_PARSE_DATAS,
    DATETIME_PARSE_DATAS_LEN,
    DATETIME_PARSE_DATAS_LEN_MAX,
    GROUP_NAMES_MAP_STR,
    MatchType,
    MatchesType,
    fos,
    REGEX_ALL_COMPILED,
    RegexPattern,
    DTFSSet,
    DTFS_Year,
    DTFS_Month,
    DTFS_Day,
    DTFS_Hour,
    DTFS_Minute,
    DTFS_Second,
    DTFS_Fractional,
    DTFS_Tz,
    DTFS_Epoch,
    DTFS_Uptime,
    MINUS_SIGN,
    HYPHEN_MINUS,
    regex_id_compiled,
    O_L,
    YEAR_FALLBACKDUMMY_VAL,
    YEAR_FALLBACKDUMMY,
    Uptime,
};

/// A _Year_ in a date
pub type Year = i32;

#[cfg(any(debug_assertions, test))]
use crate::common::FPath;
#[doc(hidden)]
pub use crate::data::line::{
    LineIndex,
    RangeLineIndex,
};
#[cfg(any(debug_assertions, test))]
use crate::debug::printers::{
    buffer_to_string_noraw,
    str_to_string_noraw,
};
use crate::{
    de_err,
    de_wrn,
    debug_panic,
};

// -----------------------------------------------
// DateTime Regex Matching and strftime formatting

/// A chrono [`DateTime`] type used in _s4lib_.
///
/// [`DateTime`]: https://docs.rs/chrono/0.4.38/chrono/struct.DateTime.html
// TODO: rename to `DateTimeS4`
pub type DateTimeL = DateTime<FixedOffset>;
pub type DateTimeLOpt = Option<DateTimeL>;

pub(crate) const UPTIME_DEFAULT_OFFSET: SystemTime = UNIX_EPOCH;

/// Convert a `T` to a [`SystemTime`].
///
/// [`SystemTime`]: std::time::SystemTime
pub fn convert_to_systemtime<T>(epoch_seconds: T) -> SystemTime
where
    u64: From<T>,
{
    UNIX_EPOCH + std::time::Duration::from_secs(epoch_seconds.into())
}

/// create a `DateTime`
///
/// wrapper for chrono DateTime creation function
#[allow(unused)]
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

/// create a `DateTime` with FixedOffset `+00:00`
///
/// wrapper for chrono DateTime creation function
#[allow(unused)]
pub fn ymdhms0(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
) -> DateTimeL {
    let fixedoffset = FixedOffset::east_opt(0).unwrap();
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
    + Duration::try_milliseconds(milli).unwrap()
}

/// create a `DateTime` with microseconds
///
/// wrapper for chrono DateTime creation function
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
#[allow(unused)]
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

/// All named timezone abbreviations, maps all chrono strftime `%Z` values
/// (e.g. `"EDT"`) to equivalent `%:z` value (e.g. `"-04:00"`).
/// Crate `chrono` does not parse named timezone. This mapping bridges that gap.
///
/// _Super Speedy Syslog Searcher_ attempts to be more lenient than chrono
/// about matching named abbreviated timezones, e.g. `"EDT"`.
/// Chrono provides `%Z` strftime specifier
/// yet rejects named timezones when passed to [`DateTime::parse_from_str`].
/// `MAP_TZZ_TO_TZz` provides the necessary mapping.
///
/// However, due to duplicate timezone names, some valid timezone names
/// will result in the default timezone. For example, there are three named
/// timezones `"IST"` that refer to different timezone offsets. If `"IST"` is
/// parsed as a timezone in a sysline then the resultant value will be the
/// default timezone offset value, e.g. the value passed to `--tz-offset`.
/// See the opening paragraph in [_List of time zone abbreviations_].
///
/// In this structure, ambiguous timezone names have their values set to empty
/// string, e.g. `"SST"` maps to `""`. See [Issue #59].
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
///     | rg -r '"$1"), ' -e '^[[:blank:]]*([[:print:]−±+]*[0-9]{1,4}.*$)' -C5 \
///     | rg -r '"$1:00"' -e '"(.?[[:digit:]][[:digit:]])"' -C5 \
///     | sed -e 's/\n"/"/g' -e 'N;s/\n/ /' -e 's/−/-/g' -e 's/±/-/g' \
///     | tr -s ' '
/// ```
///
/// See also:
/// - Applicable tz offsets <https://en.wikipedia.org/wiki/List_of_UTC_offsets>
/// - Applicable tz abbreviations <https://en.wikipedia.org/wiki/List_of_time_zone_abbreviations>
///
/// [Issue #59]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/59
/// [_List of time zone abbreviations_]: https://en.wikipedia.org/w/index.php?title=List_of_time_zone_abbreviations&oldid=1106679802
/// [`DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/format/strftime/#fn7
// TODO: why not map directly to a `FixedOffset` to skip having chrono do another
//       step of translating from `str` to `FixedOffset`?
pub static MAP_TZZ_TO_TZz: PhfMap<&'static str, &'static str> = phf_map! {
        // uppercase
        "ACDT" => "+10:30",
        "ACST" => "+09:30",
        "ACT" => "",
        //"ACT" => "-05:00",
        //"ACT" => "+08:00",
        "ACWST" => "+08:45",
        "ADT" => "-03:00",
        "AEDT" => "+11:00",
        "AEST" => "+10:00",
        "AET" => "+11:00",
        "AFT" => "+04:30",
        "AKDT" => "-08:00",
        "AKST" => "-09:00",
        "ALMT" => "+06:00",
        "AMST" => "-03:00",
        "AMT" => "",
        //"AMT" => "-04:00",
        //"AMT" => "+04:00",
        "ANAT" => "+12:00",
        "AQTT" => "+05:00",
        "ART" => "-03:00",
        "AST" => "",
        //"AST" => "+03:00",
        //"AST" => "-04:00",
        "AWST" => "+08:00",
        "AZOST" => "+00:00",
        "AZOT" => "-01:00",
        "AZT" => "+04:00",
        "BNT" => "+08:00",
        "BIOT" => "+06:00",
        "BIT" => "-12:00",
        "BOT" => "-04:00",
        "BRST" => "-02:00",
        "BRT" => "-03:00",
        "BST" => "",
        //"BST" => "+06:00",
        //"BST" => "+11:00",
        //"BST" => "+01:00",
        "BTT" => "+06:00",
        "CAT" => "+02:00",
        "CCT" => "+06:30",
        "CDT" => "",
        //"CDT" => "-05:00",
        //"CDT" => "-04:00",
        "CEST" => "+02:00",
        "CET" => "+01:00",
        "CHADT" => "+13:45",
        "CHAST" => "+12:45",
        "CHOT" => "+08:00",
        "CHOST" => "+09:00",
        "CHST" => "+10:00",
        "CHUT" => "+10:00",
        "CIST" => "-08:00",
        "CKT" => "-10:00",
        "CLST" => "-03:00",
        "CLT" => "-04:00",
        "COST" => "-04:00",
        "COT" => "-05:00",
        "CST" => "",
        //"CST" => "-06:00",
        //"CST" => "+08:00",
        //"CST" => "-05:00",
        "CT" => "-05:00",
        "CVT" => "-01:00",
        "CWST" => "+08:45",
        "CXT" => "+07:00",
        "DAVT" => "+07:00",
        "DDUT" => "+10:00",
        "DFT" => "+01:00",
        "EASST" => "-05:00",
        "EAST" => "-06:00",
        "EAT" => "+03:00",
        "ECT" => "",
        //"ECT" => "-04:00",
        //"ECT" => "-05:00",
        "EDT" => "-04:00",
        "EEST" => "+03:00",
        "EET" => "+02:00",
        "EGST" => "-00:00",
        "EGT" => "-01:00",
        "EST" => "-05:00",
        "ET" => "-04:00",
        "FET" => "+03:00",
        "FJT" => "+12:00",
        "FKST" => "-03:00",
        "FKT" => "-04:00",
        "FNT" => "-02:00",
        "GALT" => "-06:00",
        "GAMT" => "-09:00",
        "GET" => "+04:00",
        "GFT" => "-03:00",
        "GILT" => "+12:00",
        "GIT" => "-09:00",
        "GMT" => "-00:00",
        "GST" => "",
        //"GST" => "-02:00",
        //"GST" => "+04:00",
        "GYT" => "-04:00",
        "HDT" => "-09:00",
        "HAEC" => "+02:00",
        "HST" => "-10:00",
        "HKT" => "+08:00",
        "HMT" => "+05:00",
        "HOVST" => "+08:00",
        "HOVT" => "+07:00",
        "ICT" => "+07:00",
        "IDLW" => "-12:00",
        "IDT" => "+03:00",
        "IOT" => "+03:00",
        "IRDT" => "+04:30",
        "IRKT" => "+08:00",
        "IRST" => "+03:30",
        "IST" => "",
        //"IST" => "+05:30",
        //"IST" => "+01:00",
        //"IST" => "+02:00",
        "JST" => "+09:00",
        "KALT" => "+02:00",
        "KGT" => "+06:00",
        "KOST" => "+11:00",
        "KRAT" => "+07:00",
        "KST" => "+09:00",
        "LHST" => "",
        //"LHST" => "+10:30",
        //"LHST" => "+11:00",
        "LINT" => "+14:00",
        "MAGT" => "+12:00",
        "MART" => "-09:30",
        "MAWT" => "+05:00",
        "MDT" => "-06:00",
        "MET" => "+01:00",
        "MEST" => "+02:00",
        "MHT" => "+12:00",
        "MIST" => "+11:00",
        "MIT" => "-09:30",
        "MMT" => "+06:30",
        "MSK" => "+03:00",
        "MST" => "",
        //"MST" => "+08:00",
        //"MST" => "-07:00",
        "MUT" => "+04:00",
        "MVT" => "+05:00",
        "MYT" => "+08:00",
        "NCT" => "+11:00",
        "NDT" => "-02:30",
        "NFT" => "+11:00",
        "NOVT" => "+07:00",
        "NPT" => "+05:45",
        "NST" => "-03:30",
        "NT" => "-03:30",
        "NUT" => "-11:00",
        "NZDT" => "+13:00",
        "NZST" => "+12:00",
        "OMST" => "+06:00",
        "ORAT" => "+05:00",
        "PDT" => "-07:00",
        "PET" => "-05:00",
        "PETT" => "+12:00",
        "PGT" => "+10:00",
        "PHOT" => "+13:00",
        "PHT" => "+08:00",
        "PHST" => "+08:00",
        "PKT" => "+05:00",
        "PMDT" => "-02:00",
        "PMST" => "-03:00",
        "PONT" => "+11:00",
        "PST" => "-08:00",
        "PWT" => "+09:00",
        "PYST" => "-03:00",
        "PYT" => "-04:00",
        "RET" => "+04:00",
        "ROTT" => "-03:00",
        "SAKT" => "+11:00",
        "SAMT" => "+04:00",
        "SAST" => "+02:00",
        "SBT" => "+11:00",
        "SCT" => "+04:00",
        "SDT" => "-10:00",
        "SGT" => "+08:00",
        "SLST" => "+05:30",
        "SRET" => "+11:00",
        "SRT" => "-03:00",
        "SST" => "",
        //"SST" => "-11:00",
        //"SST" => "+08:00",
        "SYOT" => "+03:00",
        "TAHT" => "-10:00",
        "THA" => "+07:00",
        "TFT" => "+05:00",
        "TJT" => "+05:00",
        "TKT" => "+13:00",
        "TLT" => "+09:00",
        "TMT" => "+05:00",
        "TRT" => "+03:00",
        "TOT" => "+13:00",
        "TVT" => "+12:00",
        "ULAST" => "+09:00",
        "ULAT" => "+08:00",
        "UT" => "-00:00",
        "UTC" => "-00:00",
        "UYST" => "-02:00",
        "UYT" => "-03:00",
        "UZT" => "+05:00",
        "VET" => "-04:00",
        "VLAT" => "+10:00",
        "VOLT" => "+03:00",
        "VOST" => "+06:00",
        "VUT" => "+11:00",
        "WAKT" => "+12:00",
        "WAST" => "+02:00",
        "WAT" => "+01:00",
        "WEST" => "+01:00",
        "WET" => "-00:00",
        "WIB" => "+07:00",
        "WIT" => "+09:00",
        "WITA" => "+08:00",
        "WGST" => "-02:00",
        "WGT" => "-03:00",
        "WST" => "+08:00",
        "YAKT" => "+09:00",
        "YEKT" => "+05:00",
        "ZULU" => "+00:00",
        "Z" => "+00:00",
        // lowercase
        "acdt" => "+10:30",
        "acst" => "+09:30",
        "act" => "",
        //"act" => "-05:00",
        //"act" => "+08:00",
        "acwst" => "+08:45",
        "adt" => "-03:00",
        "aedt" => "+11:00",
        "aest" => "+10:00",
        "aet" => "+11:00",
        "aft" => "+04:30",
        "akdt" => "-08:00",
        "akst" => "-09:00",
        "almt" => "+06:00",
        "amst" => "-03:00",
        "amt" => "",
        //"amt" => "-04:00",
        //"amt" => "+04:00",
        "anat" => "+12:00",
        "aqtt" => "+05:00",
        "art" => "-03:00",
        "ast" => "",
        //"ast" => "+03:00",
        //"ast" => "-04:00",
        "awst" => "+08:00",
        "azost" => "-00:00",
        "azot" => "-01:00",
        "azt" => "+04:00",
        "bnt" => "+08:00",
        "biot" => "+06:00",
        "bit" => "-12:00",
        "bot" => "-04:00",
        "brst" => "-02:00",
        "brt" => "-03:00",
        "bst" => "",
        //"bst" => "+06:00",
        //"bst" => "+11:00",
        //"bst" => "+01:00",
        "btt" => "+06:00",
        "cat" => "+02:00",
        "cct" => "+06:30",
        "cdt" => "",
        //"cdt" => "-05:00",
        //"cdt" => "-04:00",
        "cest" => "+02:00",
        "cet" => "+01:00",
        "chadt" => "+13:45",
        "chast" => "+12:45",
        "chot" => "+08:00",
        "chost" => "+09:00",
        "chst" => "+10:00",
        "chut" => "+10:00",
        "cist" => "-08:00",
        "ckt" => "-10:00",
        "clst" => "-03:00",
        "clt" => "-04:00",
        "cost" => "-04:00",
        "cot" => "-05:00",
        "cst" => "",
        //"cst" => "-06:00",
        //"cst" => "+08:00",
        //"cst" => "-05:00",
        "ct" => "-05:00",
        "cvt" => "-01:00",
        "cwst" => "+08:45",
        "cxt" => "+07:00",
        "davt" => "+07:00",
        "ddut" => "+10:00",
        "dft" => "+01:00",
        "easst" => "-05:00",
        "east" => "-06:00",
        "eat" => "+03:00",
        "ect" => "",
        //"ect" => "-04:00",
        //"ect" => "-05:00",
        "edt" => "-04:00",
        "eest" => "+03:00",
        "eet" => "+02:00",
        "egst" => "-00:00",
        "egt" => "-01:00",
        "est" => "-05:00",
        "et" => "-04:00",
        "fet" => "+03:00",
        "fjt" => "+12:00",
        "fkst" => "-03:00",
        "fkt" => "-04:00",
        "fnt" => "-02:00",
        "galt" => "-06:00",
        "gamt" => "-09:00",
        "get" => "+04:00",
        "gft" => "-03:00",
        "gilt" => "+12:00",
        "git" => "-09:00",
        "gmt" => "-00:00",
        "gst" => "",
        //"gst" => "-02:00",
        //"gst" => "+04:00",
        "gyt" => "-04:00",
        "hdt" => "-09:00",
        "haec" => "+02:00",
        "hst" => "-10:00",
        "hkt" => "+08:00",
        "hmt" => "+05:00",
        "hovst" => "+08:00",
        "hovt" => "+07:00",
        "ict" => "+07:00",
        "idlw" => "-12:00",
        "idt" => "+03:00",
        "iot" => "+03:00",
        "irdt" => "+04:30",
        "irkt" => "+08:00",
        "irst" => "+03:30",
        "ist" => "",
        //"ist" => "+05:30",
        //"ist" => "+01:00",
        //"ist" => "+02:00",
        "jst" => "+09:00",
        "kalt" => "+02:00",
        "kgt" => "+06:00",
        "kost" => "+11:00",
        "krat" => "+07:00",
        "kst" => "+09:00",
        "lhst" => "",
        //"lhst" => "+10:30",
        //"lhst" => "+11:00",
        "lint" => "+14:00",
        "magt" => "+12:00",
        "mart" => "-09:30",
        "mawt" => "+05:00",
        "mdt" => "-06:00",
        "met" => "+01:00",
        "mest" => "+02:00",
        "mht" => "+12:00",
        "mist" => "+11:00",
        "mit" => "-09:30",
        "mmt" => "+06:30",
        "msk" => "+03:00",
        "mst" => "",
        //"mst" => "+08:00",
        //"mst" => "-07:00",
        "mut" => "+04:00",
        "mvt" => "+05:00",
        "myt" => "+08:00",
        "nct" => "+11:00",
        "ndt" => "-02:30",
        "nft" => "+11:00",
        "novt" => "+07:00",
        "npt" => "+05:45",
        "nst" => "-03:30",
        "nt" => "-03:30",
        "nut" => "-11:00",
        "nzdt" => "+13:00",
        "nzst" => "+12:00",
        "omst" => "+06:00",
        "orat" => "+05:00",
        "pdt" => "-07:00",
        "pet" => "-05:00",
        "pett" => "+12:00",
        "pgt" => "+10:00",
        "phot" => "+13:00",
        "pht" => "+08:00",
        "phst" => "+08:00",
        "pkt" => "+05:00",
        "pmdt" => "-02:00",
        "pmst" => "-03:00",
        "pont" => "+11:00",
        "pst" => "-08:00",
        "pwt" => "+09:00",
        "pyst" => "-03:00",
        "pyt" => "-04:00",
        "ret" => "+04:00",
        "rott" => "-03:00",
        "sakt" => "+11:00",
        "samt" => "+04:00",
        "sast" => "+02:00",
        "sbt" => "+11:00",
        "sct" => "+04:00",
        "sdt" => "-10:00",
        "sgt" => "+08:00",
        "slst" => "+05:30",
        "sret" => "+11:00",
        "srt" => "-03:00",
        "sst" => "",
        //"sst" => "-11:00",
        //"sst" => "+08:00",
        "syot" => "+03:00",
        "taht" => "-10:00",
        "tha" => "+07:00",
        "tft" => "+05:00",
        "tjt" => "+05:00",
        "tkt" => "+13:00",
        "tlt" => "+09:00",
        "tmt" => "+05:00",
        "trt" => "+03:00",
        "tot" => "+13:00",
        "tvt" => "+12:00",
        "ulast" => "+09:00",
        "ulat" => "+08:00",
        "ut" => "-00:00",
        "utc" => "-00:00",
        "uyst" => "-02:00",
        "uyt" => "-03:00",
        "uzt" => "+05:00",
        "vet" => "-04:00",
        "vlat" => "+10:00",
        "volt" => "+03:00",
        "vost" => "+06:00",
        "vut" => "+11:00",
        "wakt" => "+12:00",
        "wast" => "+02:00",
        "wat" => "+01:00",
        "west" => "+01:00",
        "wet" => "-00:00",
        "wib" => "+07:00",
        "wit" => "+09:00",
        "wita" => "+08:00",
        "wgst" => "-02:00",
        "wgt" => "-03:00",
        "wst" => "+08:00",
        "yakt" => "+09:00",
        "yekt" => "+05:00",
        "zulu" => "+00:00",
        "z" => "+00:00",
};

/// Index into the global [`DATETIME_PARSE_DATAS`]
pub type DateTimeParseInstrsIndex = usize;

pub const DateTimeParseDatasCompiledCount: usize = 0;

// TODO: Issue #6 handle all Unicode whitespace.
//       This fn is essentially counteracting an errant call to
//       `std::string:trim` within `Local.datetime_from_str`.
//       `trim` removes "Unicode Derived Core Property White_Space".
//       This implementation handles three whitespace chars. There are
//       twenty-five whitespace chars according to
//       <https://en.wikipedia.org/wiki/Unicode_character_property#Whitespace>.
//
/// Match spaces at beginning and ending of `value`.
/// Return `true` if mismatch of whitespace was found between `value` and
/// `pattern`, e.g. `value` is `"2022-01-01T02:03:04"`
/// but pattern is `"    %Y-%d-%mT%H:%M:%S"`.
/// Else return `false`.
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
            de_wrn!("u8_to_str return None; va {} != {} data.len()", va, data.len());
            return None; // invalid UTF8
        }
    }
    unsafe {
        dts = std::str::from_utf8_unchecked(data);
    };

    Some(dts)
}

/// Convert `data` to a chrono [`Option<DateTime<FixedOffset>>`] instance.
///
/// Compensate for a missing timezone.
///
/// - `data` to parse that has a datetime string
/// - strftime `pattern` to use for parsing, must complement `data`
/// - `has_tz`, the `pattern` has a timezone (`%Z`, `%z`, etc.)?
/// - `tz_offset` fallback timezone offset when `!has_tz`
///
/// [`Option<DateTime<FixedOffset>>`]: https://docs.rs/chrono/0.4.38/chrono/struct.DateTime.html#impl-DateTime%3CFixedOffset%3E
pub fn datetime_parse_from_str(
    data: &str,
    pattern: &DateTimePattern_str,
    has_tz: bool,
    tz_offset: &FixedOffset,
) -> DateTimeLOpt {
    defn!("(pattern {:?}, has_tz {}, tz_offset {:?}, data {:?})", pattern, has_tz, tz_offset, str_to_string_noraw(data));

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
                    str_to_string_noraw(data),
                    pattern,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `data`
                //       and `dt_pattern`
                //       See Issue #6
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
        // !has_tz (no timezone in `data`)
        // first convert to a `NaiveDateTime` instance
        let dt_naive = match NaiveDateTime::parse_from_str(data, pattern) {
            Ok(val) => {
                defo!(
                    "NaiveDateTime.parse_from_str({:?}, {:?}) extrapolated NaiveDateTime {:?}",
                    str_to_string_noraw(data),
                    pattern,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `data`
                //       and `pattern`
                if !datetime_from_str_workaround_Issue660(data, pattern) {
                    defx!("skip match due to chrono Issue #660");
                    return None;
                }
                defo!("dt_naive={:?}", val);

                val
            }
            Err(_err) => {
                defx!("NaiveDateTime.parse_from_str({:?}, {:?}) failed ParseError: {}", data, pattern, _err);
                return None;
            }
        };
        // second convert the `NaiveDateTime` instance to a `DateTime<FixedOffset>` instance
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
/// [`regex::Captures`]: https://docs.rs/regex/1.10.5/regex/bytes/struct.Captures.html
// TODO: change to a typed `struct CapturedDtData(...)`
pub type CapturedDtData = (LineIndex, LineIndex, DateTimeL);

/// Macro helper to [`captures_to_buffer_bytes`].
/// - `$cgi_index` is the `GroupsIndex` index into `GROUP_NAMES` for the capture group name.
/// - `$data` is the `&[u8]` data to copy from.
/// - `$captures` is the `Vec<MatchType>` = `Vec<(SpanS4, GroupsIndex)>` = `Vec<({usize, usize}, usize)>` of capture groups from the regex match.
/// - `$buffer` is the `&mut [u8]` buffer to copy into.
/// - `$at` is the `usize` index into `buffer` to copy at and is updated.
macro_rules! copy_capturegroup_to_buffer {
    (
        // `GroupsIndex`
        $cgi_index:ident,
        // `&[u8]`
        $data:ident,
         //  `Vec<MatchType>` = `Vec<(SpanS4, GroupsIndex)>` = `Vec<({usize, usize}, usize)>`
        $captures:ident,
        // `&mut [u8]`
        $buffer:ident,
        // `usize`
        $at:ident
    ) => {
        {
            $captures.iter().find(|cgn| cgn.group_index() == $cgi_index).map(|cgn| {
                let a = cgn.start();
                let b = cgn.end();
                debug_assert_le!(a, b);
                let len_: usize = b - a;
                defo!("copy_capturegroup_to_buffer! buffer[{:?}‥{:?}] = {:?}", $at, $at + len_, &$data[a..b]);
                $buffer[$at..$at + len_].copy_from_slice(&$data[a..b]);
                $at += len_;
            });
        }
    };
}

/// Macro helper to [`captures_to_buffer_bytes`].
macro_rules! copy_slice_to_buffer {
    (
        $u8_slice:expr,
        $buffer:ident,
        $at:ident
    ) => {
        {
            let len_: usize = $u8_slice.len();
            defo!("copy_slice_to_buffer! buffer[{:?}‥{:?}]", $at, $at + len_);
            $buffer[$at..$at + len_].copy_from_slice($u8_slice);
            $at += len_;
        }
    };
}

/// Macro helper to [`captures_to_buffer_bytes`].
macro_rules! copy_u8_to_buffer {
    (
        $u8_:expr,
        $buffer:ident,
        $at:ident
    ) => {
        {
            defo!("copy_slice_to_buffer! buffer[{:?}] = {:?}", $at, $u8_);
            $buffer[$at] = $u8_;
            $at += 1;
        }
    };
}

// Variables `const MONTH_` are helpers to [`month_bB_to_month_m_bytes`].
//
// MONTH_XY_B_l, month XY as `%B` form, lowercase
// MONTH_XY_b_l, month XY as `%b` form, lowercase
// MONTH_XY_B_u, month XY as `%B` form, uppercase
// MONTH_XY_b_u, month XY as `%b` form, uppercase
// MONTH_XY_b_U, month XY as `%b` form, uppercase all

const MONTH_01_b_l: &[u8] = b"jan";
const MONTH_01_b_u: &[u8] = b"Jan";
const MONTH_01_b_U: &[u8] = b"JAN";
const MONTH_01_b_ld: &[u8] = b"jan.";
const MONTH_01_b_ud: &[u8] = b"Jan.";
const MONTH_01_b_Ud: &[u8] = b"JAN.";
const MONTH_01_B_l: &[u8] = b"january";
const MONTH_01_B_u: &[u8] = b"January";
const MONTH_01_B_U: &[u8] = b"JANUARY";
const MONTH_01_m: &[u8] = b"01";
const MONTH_02_b_l: &[u8] = b"feb";
const MONTH_02_b_u: &[u8] = b"Feb";
const MONTH_02_b_U: &[u8] = b"FEB";
const MONTH_02_b_ld: &[u8] = b"feb.";
const MONTH_02_b_ud: &[u8] = b"Feb.";
const MONTH_02_b_Ud: &[u8] = b"FEB.";
const MONTH_02_B_l: &[u8] = b"february";
const MONTH_02_B_u: &[u8] = b"February";
const MONTH_02_B_U: &[u8] = b"FEBRUARY";
const MONTH_02_m: &[u8] = b"02";
const MONTH_03_b_l: &[u8] = b"mar";
const MONTH_03_b_u: &[u8] = b"Mar";
const MONTH_03_b_U: &[u8] = b"MAR";
const MONTH_03_b_ld: &[u8] = b"mar.";
const MONTH_03_b_ud: &[u8] = b"Mar.";
const MONTH_03_b_Ud: &[u8] = b"MAR.";
const MONTH_03_B_l: &[u8] = b"march";
const MONTH_03_B_u: &[u8] = b"March";
const MONTH_03_B_U: &[u8] = b"MARCH";
const MONTH_03_m: &[u8] = b"03";
const MONTH_04_b_l: &[u8] = b"apr";
const MONTH_04_b_u: &[u8] = b"Apr";
const MONTH_04_b_U: &[u8] = b"APR";
const MONTH_04_b_ld: &[u8] = b"apr.";
const MONTH_04_b_ud: &[u8] = b"Apr.";
const MONTH_04_b_Ud: &[u8] = b"APR.";
const MONTH_04_B_l: &[u8] = b"april";
const MONTH_04_B_u: &[u8] = b"April";
const MONTH_04_B_U: &[u8] = b"APRIL";
const MONTH_04_m: &[u8] = b"04";
const MONTH_05_b_l: &[u8] = b"may";
const MONTH_05_b_u: &[u8] = b"May";
const MONTH_05_b_U: &[u8] = b"MAY";
#[allow(dead_code)]
const MONTH_05_B_l: &[u8] = b"may"; // not used, defined for completeness
#[allow(dead_code)]
const MONTH_05_B_u: &[u8] = b"May"; // not used, defined for completeness
#[allow(dead_code)]
const MONTH_05_B_U: &[u8] = b"MAY"; // not used, defined for completeness
const MONTH_05_m: &[u8] = b"05";
const MONTH_06_b_l: &[u8] = b"jun";
const MONTH_06_b_u: &[u8] = b"Jun";
const MONTH_06_b_U: &[u8] = b"JUN";
const MONTH_06_b_ld: &[u8] = b"jun.";
const MONTH_06_b_ud: &[u8] = b"Jun.";
const MONTH_06_b_Ud: &[u8] = b"JUN.";
const MONTH_06_B_l: &[u8] = b"june";
const MONTH_06_B_u: &[u8] = b"June";
const MONTH_06_B_U: &[u8] = b"JUNE";
const MONTH_06_m: &[u8] = b"06";
const MONTH_07_b_l: &[u8] = b"jul";
const MONTH_07_b_u: &[u8] = b"Jul";
const MONTH_07_b_U: &[u8] = b"JUL";
const MONTH_07_b_ld: &[u8] = b"jul.";
const MONTH_07_b_ud: &[u8] = b"Jul.";
const MONTH_07_b_Ud: &[u8] = b"JUL.";
const MONTH_07_B_l: &[u8] = b"july";
const MONTH_07_B_u: &[u8] = b"July";
const MONTH_07_B_U: &[u8] = b"JULY";
const MONTH_07_m: &[u8] = b"07";
const MONTH_08_b_l: &[u8] = b"aug";
const MONTH_08_b_u: &[u8] = b"Aug";
const MONTH_08_b_U: &[u8] = b"AUG";
const MONTH_08_b_ld: &[u8] = b"aug.";
const MONTH_08_b_ud: &[u8] = b"Aug.";
const MONTH_08_b_Ud: &[u8] = b"AUG.";
const MONTH_08_B_l: &[u8] = b"august";
const MONTH_08_B_u: &[u8] = b"August";
const MONTH_08_B_U: &[u8] = b"AUGUST";
const MONTH_08_m: &[u8] = b"08";
const MONTH_09_b_l: &[u8] = b"sep";
const MONTH_09_b_u: &[u8] = b"Sep";
const MONTH_09_b_U: &[u8] = b"SEP";
const MONTH_09_b_ld: &[u8] = b"sep.";
const MONTH_09_b_ud: &[u8] = b"Sep.";
const MONTH_09_b_Ud: &[u8] = b"SEP.";
const MONTH_09_B_l: &[u8] = b"september";
const MONTH_09_B_u: &[u8] = b"September";
const MONTH_09_B_U: &[u8] = b"SEPTEMBER";
const MONTH_09_m: &[u8] = b"09";
const MONTH_10_b_l: &[u8] = b"oct";
const MONTH_10_b_u: &[u8] = b"Oct";
const MONTH_10_b_U: &[u8] = b"OCT";
const MONTH_10_b_ld: &[u8] = b"oct.";
const MONTH_10_b_ud: &[u8] = b"Oct.";
const MONTH_10_b_Ud: &[u8] = b"OCT.";
const MONTH_10_B_l: &[u8] = b"october";
const MONTH_10_B_u: &[u8] = b"October";
const MONTH_10_B_U: &[u8] = b"OCTOBER";
const MONTH_10_m: &[u8] = b"10";
const MONTH_11_b_l: &[u8] = b"nov";
const MONTH_11_b_u: &[u8] = b"Nov";
const MONTH_11_b_U: &[u8] = b"NOV";
const MONTH_11_b_ld: &[u8] = b"nov.";
const MONTH_11_b_ud: &[u8] = b"Nov.";
const MONTH_11_b_Ud: &[u8] = b"NOV.";
const MONTH_11_B_l: &[u8] = b"november";
const MONTH_11_B_u: &[u8] = b"November";
const MONTH_11_B_U: &[u8] = b"NOVEMBER";
const MONTH_11_m: &[u8] = b"11";
const MONTH_12_b_l: &[u8] = b"dec";
const MONTH_12_b_u: &[u8] = b"Dec";
const MONTH_12_b_U: &[u8] = b"DEC";
const MONTH_12_b_ld: &[u8] = b"dec.";
const MONTH_12_b_ud: &[u8] = b"Dec.";
const MONTH_12_b_Ud: &[u8] = b"DEC.";
const MONTH_12_B_l: &[u8] = b"december";
const MONTH_12_B_u: &[u8] = b"December";
const MONTH_12_B_U: &[u8] = b"DECEMBER";
const MONTH_12_m: &[u8] = b"12";

/// Transform strftime `%B`, `%b` (i.e. `"January"`, `"Jan"`) to
/// strftime `%m` (i.e. `"01"`).
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
        // then try *b*dot matches
        MONTH_01_b_ld | MONTH_01_b_ud | MONTH_01_b_Ud => buffer.copy_from_slice(MONTH_01_m),
        MONTH_02_b_ld | MONTH_02_b_ud | MONTH_02_b_Ud => buffer.copy_from_slice(MONTH_02_m),
        MONTH_03_b_ld | MONTH_03_b_ud | MONTH_03_b_Ud => buffer.copy_from_slice(MONTH_03_m),
        MONTH_04_b_ld | MONTH_04_b_ud | MONTH_04_b_Ud => buffer.copy_from_slice(MONTH_04_m),
        // MONTH_05_b_ld not needed
        MONTH_06_b_ld | MONTH_06_b_ud | MONTH_06_b_Ud => buffer.copy_from_slice(MONTH_06_m),
        MONTH_07_b_ld | MONTH_07_b_ud | MONTH_07_b_Ud => buffer.copy_from_slice(MONTH_07_m),
        MONTH_08_b_ld | MONTH_08_b_ud | MONTH_08_b_Ud => buffer.copy_from_slice(MONTH_08_m),
        MONTH_09_b_ld | MONTH_09_b_ud | MONTH_09_b_Ud => buffer.copy_from_slice(MONTH_09_m),
        MONTH_10_b_ld | MONTH_10_b_ud | MONTH_10_b_Ud => buffer.copy_from_slice(MONTH_10_m),
        MONTH_11_b_ld | MONTH_11_b_ud | MONTH_11_b_Ud => buffer.copy_from_slice(MONTH_11_m),
        MONTH_12_b_ld | MONTH_12_b_ud | MONTH_12_b_Ud => buffer.copy_from_slice(MONTH_12_m),
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

/// Put [`Captures`] into `buffer` in a particular order and formatting.
/// This is to prepare the regex matched data for passing to a later call to
/// [`DateTime::parse_from_str`] (called outside of this function).
///
/// This bridges the crate `regex` regular expression pattern strings,
/// [`DateTimeParseInstr::regex_pattern`], to crate `chrono` strftime strings,
/// [`DateTimeParseInstr::dtfs`].
///
/// Directly relates to datetime format `dtfs` values in
/// [`test_DATETIME_PARSE_DATAS_test_cases`] which use `DTFSS_YmdHMS`, etc.
///
/// Transforms `%B` acceptable value to `%m` acceptable value.
///
/// Transforms `%e` acceptable value to `%d` acceptable value.
///
/// Transforms an uptime (seconds since system boot) to current seconds offset
/// since UNIX_EPOCH. This allows later conversion via chrono strftime `%s`.
///
/// Transforms timezone offset inidicator MINUS SIGN `−` (U+2212) into
/// HYPHEN-MINUS `-` (U+2D), e.g `−0700` becomes `-0700`.
///
/// [`Captures`]: https://docs.rs/regex/1.10.5/regex/bytes/struct.Captures.html
/// [`DateTime::parse_from_str`]: https://docs.rs/chrono/0.4.38/chrono/struct.DateTime.html#method.parse_from_str
// TODO: allow returning an `Error` instead of `panic!`
#[inline(always)]
pub(crate) fn captures_to_buffer_bytes(
    buffer: &mut [u8],
    data: &[u8],
    captures: &MatchesType,
    year_opt: &Option<Year>,
    systemtime_at_uptime_zero: &Option<SystemTime>,
    tz_offset_string: &String,
    dtfs: &DTFSSet,
) -> usize {
    defn!("(…, …, year_opt {:?}, systemtime_at_uptime_zero {:?}, tz_offset {:?}, …)",
          year_opt, systemtime_at_uptime_zero, tz_offset_string);

    let mut at: usize = 0;

    defo!("process <epoch> {:?}…", dtfs.epoch);
    match dtfs.epoch {
        DTFS_Epoch::s => {
            copy_capturegroup_to_buffer!(CGI_EPOCH, data, captures, buffer, at);
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        }
        DTFS_Epoch::ms => {
            // special case
            // copy the epoch milliseconds but only the seconds part
            // then copy the milliseconds, the last 3 chars, as a fractional
            captures.iter().find(|cgn| cgn.group_index() == CGI_EPOCH).map(|cgn| {
                // copy the seconds part of the epoch milliseconds
                let a = cgn.start();
                let mut b = cgn.end();
                debug_assert_le!(a, b);
                if (b - a) > 3 {
                    b -= 3;
                } else {
                    debug_panic!("epoch ms capture group too short to have milliseconds part: {:?}", &data[a..b]);
                }
                let len_: usize = b - a;
                defo!("copy buffer[{:?}‥{:?}] = {:?}", at, at + len_, &data[a..b]);
                buffer[at..at + len_].copy_from_slice(&data[a..b]);
                at += len_;
                // separate seconds and milliseconds with `.`
                copy_u8_to_buffer!(b'.', buffer, at);
                // copy the milliseconds part of the epoch milliseconds, last 3 chars
                let a = b;
                let b = cgn.end();
                debug_assert_le!(a, b);
                let len_: usize = b - a;
                debug_assert_eq!(len_, 3, "last milliseconds part should be 3 chars");
                defo!("copy buffer[{:?}‥{:?}] = {:?}", at, at + len_, &data[a..b]);
                buffer[at..at + len_].copy_from_slice(&data[a..b]);
                at += len_;
            });
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        }
        DTFS_Epoch::_none => {}
    }

    defo!("process <uptime> {:?}…", dtfs.uptime);
    match dtfs.uptime {
        DTFS_Uptime::u => {
            // Here is where an important conversion happens:
            // get the log uptime string, e.g. `"1.340"`, and convert it to a
            // Duration. Then add that to the `systemtime_at_uptime_zero`.
            // So value `1.340` is added to the wrapped `SystemTime` value.
            // Then that is written into a buffer as seconds since UNIX_EPOCH.
            // Later, in `datetime_parse_from_str`, this buffer is converted to
            // a `DateTime` value.

            // copy the `uptime` capture group to a temporary local buffer
            let mut at_uptime = 0;
            const BUFLEN: usize = 30;
            let mut buf_uptime: [u8; BUFLEN] = [0; BUFLEN];
            copy_capturegroup_to_buffer!(CGI_UPTIME, data, captures, buf_uptime, at_uptime);
            defo!("buf_uptime {:?}", buffer_to_string_noraw(&buf_uptime));
            // TODO: use `u8_to_str`
            let buf_uptime_s: &str = match std::str::from_utf8(&buf_uptime[..at_uptime]) {
                Ok(s) => s,
                Err(_err) => {
                    de_err!("uptime str::from_utf8 error: {}", _err);
                    // fallback to zero
                    "0"
                }
            };
            defo!("buf_uptime_s {:?}", buf_uptime_s);
            // extract the uptime string to an `Uptime` value
            let uptime_val: Uptime = match buf_uptime_s.parse::<Uptime>() {
                Ok(uptime_) => uptime_,
                Err(_err) => {
                    de_err!("uptime parse error: {}", _err);
                    // fallback to zero
                    0
                }
            };
            defo!("uptime_val {:?}", uptime_val);

            let uptime_zero: SystemTime = match systemtime_at_uptime_zero {
                Some(val) => *val,
                None => UPTIME_DEFAULT_OFFSET,
            };
            defo!("uptime_zero {:?}", uptime_zero);

            // convert the uptime value to a `std::time::Duration`
            let uptime_dur: StdDuration = StdDuration::new(uptime_val as u64, 0);
            defo!("uptime_dur {:?}", uptime_dur);
            // add the uptime value to the uptime_zero value
            let uptime_zero_plus_uptime: SystemTime = match uptime_zero.checked_add(uptime_dur) {
                Some(st) => st,
                None => {
                    debug_panic!("failed checked_add({:?})", uptime_dur);
                    // I'm not sure what else to do here in a release build

                    UPTIME_DEFAULT_OFFSET
                },
            };
            defo!("uptime_zero_plus_uptime {:?}", uptime_zero_plus_uptime);
            // convert the `uptime_zero_plus_uptime` value to a string to a [u8]
            buf_uptime.fill(0);
            let uptime_zero_plus_uptime_dur = match uptime_zero_plus_uptime.duration_since(SystemTime::UNIX_EPOCH) {
                Ok(dur) => dur,
                Err(_err) => {
                    debug_panic!("uptime_zero_plus_uptime.duration_since(UPTIME_DEFAULT_OFFSET) failed: {}", _err);
                    // fallback to zero
                    StdDuration::from_secs(0)
                }
            };
            let uptime_zero_plus_uptime_n = uptime_zero_plus_uptime_dur.as_secs();
            defo!("uptime_zero_plus_uptime_n {:?}", uptime_zero_plus_uptime_n);
            let buf_uptime_plus: &[u8] = uptime_zero_plus_uptime_n.numtoa(10, &mut buf_uptime);
            defo!("buf_uptime_plus {:?}", buffer_to_string_noraw(buf_uptime_plus));
            // copy the local temporary buffer to the main buffer
            copy_slice_to_buffer!(&buf_uptime_plus, buffer, at);
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        }
        DTFS_Uptime::_none => {}
    }

    // year
    defo!("process <year> {:?}…", dtfs.year);
    match dtfs.year {
        DTFS_Year::Y
        | DTFS_Year::y => {
            copy_capturegroup_to_buffer!(CGI_YEAR, data, captures, buffer, at);
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        }
        | DTFS_Year::_fill => {
            let mut found_year: bool = false;
            captures.iter().find(|cgn| cgn.group_index() == CGI_YEAR).map(|match_type| {
                let a = match_type.start();
                let b = match_type.end();
                let year: &[u8] = &data[a..b];
                copy_slice_to_buffer!(year, buffer, at);
                defo!("buffer {:?}", buffer_to_string_noraw(buffer));
                found_year = true;
            });
            if !found_year{
                match year_opt {
                    Some(year) => {
                        // TODO: 2022/07/11 cost-savings: pass in `Option<&[u8]>`, avoid creating `String`
                        let year_s: String = year.to_string();
                        debug_assert_eq!(year_s.len(), 4, "Bad year string {:?}", year_s);
                        defo!("using fallback year {:?}", year_s);
                        copy_slice_to_buffer!(year_s.as_bytes(), buffer, at);
                        defo!("buffer {:?}", buffer_to_string_noraw(buffer));
                    }
                    None => {
                        defo!("using hardcoded dummy year {:?}", YEAR_FALLBACKDUMMY);
                        copy_slice_to_buffer!(YEAR_FALLBACKDUMMY.as_bytes(), buffer, at);
                        defo!("buffer {:?}", buffer_to_string_noraw(buffer));
                    }
                }
            }
        }
        DTFS_Year::_none => {}
    }
    // month
    defo!("process <month> {:?}…", dtfs.month);
    match dtfs.month {
        DTFS_Month::m => {
            copy_capturegroup_to_buffer!(CGI_MONTH, data, captures, buffer, at);
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        }
        DTFS_Month::ms => {
            let month: &[u8] = captures.iter().find(|cgn| cgn.group_index() == CGI_MONTH).map(|match_type| {
                let a = match_type.start();
                let b = match_type.end();
                &data[a..b]
            }).expect("missing month capture group");
            // chrono strftime expects numeric months to be two-digit
            // so prepend `0` if necessary
            match month.len() {
                1 => {
                    copy_slice_to_buffer!(b"0", buffer, at);
                    copy_slice_to_buffer!(month, buffer, at);
                }
                _val => {
                    debug_assert_eq!(_val, 2, "unexpected Month length {}", _val);
                    copy_slice_to_buffer!(month, buffer, at);
                }
            }
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        }
        DTFS_Month::b | DTFS_Month::B => {
            let month: &[u8] = captures.iter().find(|cgn| cgn.group_index() == CGI_MONTH).map(|match_type| {
                let a = match_type.start();
                let b = match_type.end();
                &data[a..b]
            }).expect("missing month capture group");
            month_bB_to_month_m_bytes(
                month,
                &mut buffer[at..at + 2],
            );
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
            at += 2;
        }
        DTFS_Month::_none => {}
    }
    // day
    defo!("process <day> {:?}…", dtfs.day);
    match dtfs.day {
        DTFS_Day::_e_or_d => {
            let day: &[u8] = captures.iter().find(|cgn| cgn.group_index() == CGI_DAY).map(|match_type| {
                let a = match_type.start();
                let b = match_type.end();

                &data[a..b]
            }).expect("missing day capture group");
            debug_assert_ge!(day.len(), 1, "bad named group 'day' data {:?}, expected data ge 1", day);
            debug_assert_le!(day.len(), 2, "bad named group 'day' data {:?}, expected data le 2", day);
            match day.len() {
                1 => {
                    // change day "8" (%e) to "08" (%d)
                    copy_u8_to_buffer!(b'0', buffer, at);
                    copy_u8_to_buffer!(day[0], buffer, at);
                    defo!("buffer {:?}", buffer_to_string_noraw(buffer));
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
                    defo!("buffer {:?}", buffer_to_string_noraw(buffer));
                }
                _ => {
                    panic!("bad day.len() {}", day.len());
                }
            }
        }
        DTFS_Day::_none => {}
    }
    // Day pattern `%a` (`Monday`, 'Tue`, etc.) (capture group `CGN_DAY_IGNORE`) is captured but not
    // passed along to chrono functions.

    // day-time divider
    defo!("process date-time divider…");
    copy_u8_to_buffer!(b'T', buffer, at);
    defo!("buffer {:?}", buffer_to_string_noraw(buffer));
    // hour
    defo!("process <hour> {:?}…", dtfs.hour);
    match dtfs.hour {
        DTFS_Hour::I
        | DTFS_Hour::l
        | DTFS_Hour::H => {
            copy_capturegroup_to_buffer!(CGI_HOUR, data, captures, buffer, at);
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        }
        DTFS_Hour::k => {
            let hour: &[u8] = captures.iter().find(|cgn| cgn.group_index() == CGI_HOUR).map(|match_type| {
                let a = match_type.start();
                let b = match_type.end();
                &data[a..b]
            }).expect("missing hour capture group");
            // chrono strftime expects numeric hour to be two-digit
            // so prepend `0` if necessary
            match hour.len() {
                1 => {
                    copy_slice_to_buffer!(b"0", buffer, at);
                    copy_slice_to_buffer!(hour, buffer, at);
                }
                _val => {
                    debug_assert_eq!(_val, 2, "unexpected Month length {}", _val);
                    copy_slice_to_buffer!(hour, buffer, at);
                }
            }
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        }
        DTFS_Hour::_none => {}
    }
    // minute
    defo!("process <minute> {:?}…", dtfs.minute);
    match dtfs.minute {
        DTFS_Minute::M => {
            copy_capturegroup_to_buffer!(CGI_MINUTE, data, captures, buffer, at);
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        },
        DTFS_Minute::m => {
            let minute: &[u8] = captures.iter().find(|cgn| cgn.group_index() == CGI_MINUTE).map(|match_type| {
                let a = match_type.start();
                let b = match_type.end();
                &data[a..b]
            }).expect("missing minute capture group");
            // chrono strftime expects numeric minute to be two-digit
            // so prepend `0` if necessary
            match minute.len() {
                1 => {
                    copy_slice_to_buffer!(b"0", buffer, at);
                    copy_slice_to_buffer!(minute, buffer, at);
                }
                _val => {
                    debug_assert_eq!(_val, 2, "unexpected Minute length {}", _val);
                    copy_slice_to_buffer!(minute, buffer, at);
                }
            }
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        },
        DTFS_Minute::_none => {}
    }
    // second
    defo!("process <second> {:?}…", dtfs.second);
    match dtfs.second {
        DTFS_Second::S => {
            copy_capturegroup_to_buffer!(CGI_SECOND, data, captures, buffer, at);
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        }
        DTFS_Second::s => {
            let second: &[u8] = captures.iter().find(|cgn| cgn.group_index() == CGI_SECOND).map(|match_type| {
                let a = match_type.start();
                let b = match_type.end();
                &data[a..b]
            }).expect("missing second capture group");
            // chrono strftime expects numeric second to be two-digit
            // so prepend `0` if necessary
            match second.len() {
                1 => {
                    copy_slice_to_buffer!(b"0", buffer, at);
                    copy_slice_to_buffer!(second, buffer, at);
                }
                _val => {
                    debug_assert_eq!(_val, 2, "unexpected Second length {}", _val);
                    copy_slice_to_buffer!(second, buffer, at);
                }
            }
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        }
        DTFS_Second::_fill => {
            copy_slice_to_buffer!(b"00", buffer, at);
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        }
        DTFS_Second::_none => {}
    }
    // fractional
    defo!("process <fractional> {:?}…", dtfs.fractional);
    match dtfs.fractional {
        DTFS_Fractional::f => {
            defo!("matched DTFS_Fractional::f");
            copy_u8_to_buffer!(b'.', buffer, at);
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
            let fractional: &[u8] = captures.iter().find(|cgn| cgn.group_index() == CGI_FRACTIONAL).map(|match_type| {
                let a = match_type.start();
                let b = match_type.end();
                &data[a..b]
            }).expect("missing fractional capture group");
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
                    // fractional is too large; copy only left-most 9 chars
                    copy_slice_to_buffer!(&fractional[..9], buffer, at);
                    de_wrn!("fractional string {:?} is length {} bytes, only copying 9 bytes", fractional, len)
                }
                _ => {
                    // something is wrong with this matched string; ignore it
                    de_err!("unexpected fractional string match {:?} length {} bytes", fractional, len)
                }
            }
            defo!("buffer {:?}", buffer_to_string_noraw(buffer));
        }
        DTFS_Fractional::_none => {}
    }

    // tz
    defo!("process <tz> {:?}…", dtfs.tz);
    match dtfs.tz {
        DTFS_Tz::_fill => {
            copy_slice_to_buffer!(tz_offset_string.as_bytes(), buffer, at);
        }
        DTFS_Tz::z | DTFS_Tz::zc | DTFS_Tz::zp => {
            // for data passed to chrono `DateTime::parse_from_str`,
            // replace Unicode "minus sign" to ASCII "hyphen-minus"
            // see Issue https://github.com/chronotope/chrono/issues/835
            // XXX: chrono 0.4.27 handles MINUS SIGN (U+2212)
            //      see PR https://github.com/chronotope/chrono/pull/1087
            //      however, keep this code here as it works fine
            let captureb: &[u8] = captures.iter().find(|cgn| cgn.group_index() == CGI_TZ).map(|match_type| {
                let a = match_type.start();
                let b = match_type.end();
                &data[a..b]
            }).expect("missing tz capture group");
            match captureb.starts_with(MINUS_SIGN) {
                true => {
                    defo!("found Unicode 'minus sign', transform to ASCII 'hyphen-minus'");
                    // found Unicode "minus sign", replace with ASCII
                    // "hyphen-minus"
                    copy_slice_to_buffer!(HYPHEN_MINUS, buffer, at);
                    defo!("buffer {:?}", buffer_to_string_noraw(buffer));
                    // copy data remaining after Unicode "minus sign"
                    // TODO: use u8_to_str
                    match std::str::from_utf8(captureb) {
                        Ok(val) => {
                            match val.char_indices().nth(1) {
                                Some((offset, _)) => {
                                    copy_slice_to_buffer!(val[offset..].as_bytes(), buffer, at);
                                    defo!("buffer {:?}", buffer_to_string_noraw(buffer));
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
                    defo!("buffer {:?}", buffer_to_string_noraw(buffer));
                }
            }
        }
        DTFS_Tz::Z => {
            #[allow(non_snake_case)]
            let tzZ: &str = u8_to_str(
                captures.iter().find(|cgn| cgn.group_index() == CGI_TZ).map(|match_type| {
                    let a = match_type.start();
                    let b = match_type.end();
                    &data[a..b]
                }).expect("missing tz capture group")
            ).unwrap_or_default();
            if tzZ.is_empty() {
                debug_panic!("tzZ.is_empty()");
                // `u8_to_str` failed, fallback to using passed TZ offset
                copy_slice_to_buffer!(tz_offset_string.as_bytes(), buffer, at);
            } else {
                match MAP_TZZ_TO_TZz.get_entry(tzZ) {
                    Some((_tz_abbr, tz_offset_val)) => {
                        match tz_offset_val.is_empty() {
                            true => {
                                // given an ambiguous timezone name, fallback to
                                // passed TZ offset
                                copy_slice_to_buffer!(tz_offset_string.as_bytes(), buffer, at);
                            }
                            false => {
                                // given an unambiguous timezone name, use associated offset
                                copy_slice_to_buffer!(tz_offset_val.as_bytes(), buffer, at);
                            }
                        }
                    }
                    None => {
                        // cannot find entry in MAP_TZZ_TO_TZz, use passed TZ offset
                        debug_panic!("captured named timezone {:?} not found in MAP_TZZ_TO_TZz", tzZ);
                        copy_slice_to_buffer!(tz_offset_string.as_bytes(), buffer, at);
                    }
                }
            }
        }
        DTFS_Tz::_none => {}
    }
    defo!("buffer {:?}", buffer_to_string_noraw(buffer));

    defx!("return {:?}", at);

    at
}

/// Run [`regex::Captures`] on the `data` then convert to a chrono
/// [`Option<DateTime<FixedOffset>>`] instance. Uses matching and pattern
/// information hardcoded in `DATETIME_PARSE_DATAS`.
///
/// [`regex::Captures`]: https://docs.rs/regex/1.10.5/regex/bytes/struct.Regex.html#method.captures
/// [`Option<DateTime<FixedOffset>>`]: https://docs.rs/chrono/0.4.38/chrono/struct.DateTime.html#impl-DateTime%3CFixedOffset%3E
pub fn bytes_to_regex_to_datetime(
    data: &[u8],
    index: &DateTimeParseInstrsIndex,
    year_opt: &Option<Year>,
    systemtime_at_uptime_zero: &Option<SystemTime>,
    tz_offset: &FixedOffset,
    tz_offset_string: &String,
    #[cfg(any(debug_assertions, test))]
    _path: &FPath,
) -> Option<CapturedDtData> {
    defn!("(data {} bytes, index={:?}, year_opt={:?}, tz_offset={:?}, tz_offset_string={:?})",
        data.len(), index, year_opt, tz_offset, tz_offset_string);

    let dtpd: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*index];
    // here is the regular expression function call!
    defo!("regex_fn({:?})…", buffer_to_string_noraw(data));
    let captures: MatchesType = match (dtpd.regex_fn)(data) {
        None => {
            defx!(
                "regex: no captures (returned None) for regex #{} at index {}, line {}",
                dtpd.regex_id, index, dtpd._line_num,
            );
            return None;
        }
        Some(captures) => captures,
    };
    defo!("regex: captured {} matches for regex #{} at index {}, line {}",
        captures.len(), dtpd.regex_id, index, dtpd._line_num);
    #[cfg(any(debug_assertions, test))]
    {
        for (i, mi) in captures.iter().enumerate() {
            let mi_name: &str = CGN_ALL[mi.group_index()];
            let m_data = &data[mi.start()..mi.end()];
            let m_data_s = u8_to_str(m_data).unwrap_or("ERROR DECODING");
            defo!("regex: match[{}] = [{}‥{}] = [{:?}] = {:?}", i, mi.start(), mi.end(), mi_name, m_data_s);
        }
    }

    // copy regex matches into a buffer with predictable ordering
    // this ordering relates to datetime format strings in `test_DATETIME_PARSE_DATAS_test_cases`
    const BUFLEN: usize = 35;
    let mut buffer: [u8; BUFLEN] = [0; BUFLEN];
    let copiedn = captures_to_buffer_bytes(
        &mut buffer,
        data,
        &captures,
        year_opt,
        systemtime_at_uptime_zero,
        tz_offset_string,
        &dtpd.dtfs
    );

    // use the `dt_format` to parse the buffer of regex matches
    let buffer_s: &str = match u8_to_str(&buffer[0..copiedn]) {
        Some(s) => s,
        None => {
            defx!("u8_to_str failed to convert slice of {} bytes", copiedn);
            return None;
        }
    };
    let dt = match datetime_parse_from_str(
        buffer_s,
        dtpd.dtfs.pattern,
        dtpd.dtfs.has_tz(),
        tz_offset,
    ) {
        Some(dt_) => dt_,
        None => {
            defx!("return None; datetime_parse_from_str returned None");
            return None;
        }
    };

    // derive the `LineIndex` bounds of the datetime substring within `data`
    let cgi_first = match GROUP_NAMES_MAP_STR
        .get(dtpd.cgn_first) {
        Some(cgi) => cgi,
        None => panic!("missing cgn_first {:?} in GROUP_NAMES_MAP_STR", dtpd.cgn_first),
    };
    let dt_beg: LineIndex = match captures.iter().find(|cgn| cgn.group_index() == *cgi_first) {
        Some(match_) => match_.start() as LineIndex,
        None => 0,
    };
    let cgi_last = match GROUP_NAMES_MAP_STR
        .get(dtpd.cgn_last) {
        Some(cgi) => cgi,
        None => panic!("missing cgn_last {:?} in GROUP_NAMES_MAP_STR", dtpd.cgn_last),
    };
    let dt_end: LineIndex = match captures.iter().find(|cgn| cgn.group_index() == *cgi_last) {
        Some(match_) => match_.end() as LineIndex,
        None => 0,
    };
    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}, index {}", dt_beg, dt_end, index);

    defx!("return Some({:?}, {:?}, {:?})", dt_beg, dt_end, dt);
    Some((dt_beg, dt_end, dt))
}

// --------------------
// DateTime comparisons

/// Describe the result of comparing one [`DateTimeL`] to one DateTime Filter.
#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq)]
pub enum Result_Filter_DateTime1 {
    /// like Skip
    Pass,
    /// the `DateTimeL` instance occurs at or after the datetime filter
    OccursAtOrAfter,
    /// the `DateTimeL` instance occurs before the datetime filter
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
/// If `dt` is at or after `dt_filter` then return [`Result_Filter_DateTime1::OccursAtOrAfter`]<br/>
/// If `dt` is before `dt_filter` then return [`Result_Filter_DateTime1::OccursBefore`]<br/>
/// Else return [`Result_Filter_DateTime1::Pass`] (including if `dt_filter` is `None`)
pub fn dt_after_or_before(
    dt: &DateTimeL,
    dt_filter: &DateTimeLOpt,
) -> Result_Filter_DateTime1 {
    let dt_a: &DateTimeL = match dt_filter {
        Some(dt_) => dt_,
        None => {
            defñ!("return Pass; (no dt filters)");
            return Result_Filter_DateTime1::Pass;
        }
    };
    if dt < dt_a {
        defñ!("return OccursBefore; (dt {:?} is before dt_filter {:?})", dt, dt_a);
        return Result_Filter_DateTime1::OccursBefore;
    }
    defñ!("return OccursAtOrAfter; (dt {:?} is at or after dt_filter {:?})", dt, dt_a);

    Result_Filter_DateTime1::OccursAtOrAfter
}

/// How does the passed `dt` pass the optional `DateTimeLOpt`
/// filter instances, `dt_filter_after` and `dt_filter_before`?
/// Is `dt` before ([`BeforeRange`]), after ([`AfterRange`]),
/// or in between ([`InRange`])?
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
    match (dt_filter_after, dt_filter_before) {
        (None, None) => {
            defx!("return InRange; (no dt filters)");

            Result_Filter_DateTime2::InRange
        }
        (Some(da), Some(db)) => {
            debug_assert_le!(da, db, "Bad datetime range values filter_after {:?} {:?} filter_before", da, db);
            if dt < da {
                defx!("return BeforeRange");
                return Result_Filter_DateTime2::BeforeRange;
            }
            if db < dt {
                defx!("return AfterRange");
                return Result_Filter_DateTime2::AfterRange;
            }
            debug_assert_le!(da, dt, "Unexpected range values da dt");
            debug_assert_le!(dt, db, "Unexpected range values dt db");
            defx!("return InRange");

            Result_Filter_DateTime2::InRange
        }
        (Some(da), None) => {
            if dt < da {
                defx!("return BeforeRange");
                return Result_Filter_DateTime2::BeforeRange;
            }
            defx!("return InRange");

            Result_Filter_DateTime2::InRange
        }
        (None, Some(db)) => {
            if db < dt {
                defx!("return AfterRange");
                return Result_Filter_DateTime2::AfterRange;
            }
            defx!("return InRange");

            Result_Filter_DateTime2::InRange
        }
    }
}

// ---------------------------------------------
// other miscellaneous DateTime function helpers

/// Create a new [`DateTimeL`] instance that uses the passed `DateTimeL`
/// month, day, and time, combined with the passed `Year`.
///
/// In case of error, return a copy of the passed `DateTimeL`.
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
/// [`FixedOffset`]: https://docs.rs/chrono/0.4.38/chrono/offset/struct.FixedOffset.html
/// [`SystemTime`]: std::time::SystemTime
pub fn systemtime_to_datetime(
    fixedoffset: &FixedOffset,
    systemtime: &SystemTime,
) -> DateTimeL {
    // https://users.rust-lang.org/t/convert-std-time-systemtime-to-chrono-datetime-datetime/7684/6
    let dtu: DateTime<Utc> = (*systemtime).into();

    dtu.with_timezone(fixedoffset)
}

/// Subtract a [`SystemTime`] from a [`DateTimeL`].
pub fn datetime_minus_systemtime(
    datetime: &DateTimeL,
    systemtime: &SystemTime,
) -> Duration {
    *datetime - systemtime_to_datetime(
        datetime.offset(),
        systemtime,
    )
}

/// Convert passed seconds since Unix Epoch to a `SystemTime`.
pub fn seconds_to_systemtime(
    seconds: &u64,
) -> SystemTime {
    let duration = std::time::Duration::from_secs(*seconds);

    // TODO: [2024/06] handle `None`
    // TODO: if `checked_add` becomes `const` then multiple other functions
    //       could become `const`
    SystemTime::UNIX_EPOCH.checked_add(duration).unwrap()
}

/// Return the year of the `systemtime`
pub fn systemtime_year(
    systemtime: &SystemTime,
) -> Year {
    let dtu: DateTime<Utc> = (*systemtime).into();

    dtu.year()
}
