// Data/datetime.rs


#[cfg(any(debug_assertions,test))]
use crate::printer_debug::printers::{
    str_to_String_noraw,
};

use crate::printer_debug::stack::{
    sn,
    snx,
    so,
    sx,
};

pub use crate::Data::line::{
    LineIndex,
    Range_LineIndex,
};

use std::collections::BTreeMap;
use std::fmt;

extern crate arrayref;
use arrayref::array_ref;

extern crate chrono;
pub use chrono::{
    DateTime,
    Datelike, // adds method `.year()` onto `DateTime`
    FixedOffset,
    Local,
    NaiveDateTime,
    Offset,
    TimeZone,
    Utc,
};

extern crate const_format;
use const_format::concatcp;

extern crate debug_print;
use debug_print::debug_eprintln;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    debug_assert_lt,
};

extern crate regex;
use regex::Regex;

extern crate unroll;
use unroll::unroll_for_loops;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DateTime Regex Matching and strftime formatting
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// chrono strftime formatting pattern, passed to `chrono::datetime_from_str`
pub type DateTimePattern_str = str;
//pub type DateTimePattern_string = String;
/// regular expression formatting pattern, passed to `Regex::new`
pub type DateTimeRegex_str = str;
//pub type DateTimeRegex_string = String;
pub type CaptureGroupName = str;
pub type CaptureGroupPattern = str;
pub type RegexPattern = str;
/// the regular expression "class" used here, specifically for matching datetime substrings within
/// a `&str`
pub type DateTimeRegex = Regex;
/// the chrono DateTime type used here
pub type DateTimeL = DateTime<FixedOffset>;
pub type DateTimeL_Opt = Option<DateTimeL>;

/// DateTime Format Specifier for Year
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
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Month {
    /// %m
    m,
    /// %b
    b,
    /// %B
    B,
}

/// DateTime Format Specifier for Day
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Day {
    /// %d
    d,
    /// %e
    e,
    /// %e or %d (" 8" or "08") captured but must be changed to %d ("08")
    _de_to_d,
    // TODO: does this need %a %A... ?
}

/// DateTime Format Specifier for Hour
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
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Minute {
    /// %M
    M,
}

/// DateTime Format Specifier for Minute
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Second {
    /// %S
    S,
}

/// DateTime Format Specifier for Fractional or fractional second
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Fractional {
    /// %f
    f,
    /// none, will not be filled
    _none,
}

/// DateTime Format Specifier for Timezone
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum DTFS_Tz {
    /// %z +0930
    z,
    /// %:z +09:30
    cz,
    /// %#z +09
    pz,
    /// %Z
    Z,
    /// none, must be filled
    /// the associated `pattern` should use "%:z` as that is the form displayed
    /// by `chrono::FixedOffset::east(0).as_string().to_str()`
    _fill,
}

/// `DTFSSet` is essentially instructions to transcribe regex captures to a strftime-ready
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
    /// done in `captures_to_buffer`.
    ///
    /// `pattern` is interdependent with other members.
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


/// Settings for processing from an unknown `str` to `regex::Regex().captures()` to
/// to `chrono::DateTime::parse_from_str`.
///
/// The settings is entirely interdependent. Tested in `test_DATETIME_PARSE_DATAS_builtin`.
#[derive(Hash)]
pub struct DateTime_Parse_Data<'a> {
    // regex pattern for `captures`
    pub regex_pattern: &'a DateTimeRegex_str,
    /// in what strftime form are the regex `regex_pattern` capture groups?
    pub dtfs: DTFSSet<'a>,
    /// slice range of widest regex pattern match unioned of all possible matches
    pub range_regex: Range_LineIndex,
    /// capture named group first (left-most) position in regex
    pub cgn_first: &'a CaptureGroupName,
    /// capture named group last (right-most) position in regex
    pub cgn_last: &'a CaptureGroupName,
    /// hardcoded test case
    #[cfg(any(debug_assertions,test))]
    pub _test_case: &'a str,
    /// line number of declaration, to aid debugging
    pub _line_num: u32,
}

/// declare a `DateTime_Parse_Data` tuple a little more easily
#[macro_export]
macro_rules! DTPD {
    (
        $dtr:expr,
        $dtfs:expr,
        $sib:literal,
        $sie:literal,
        $cgn_first:ident,
        $cgn_last:ident,
        $test_case:literal,
        $line_num:expr,
    ) => {
        DateTime_Parse_Data {
            regex_pattern: $dtr,
            dtfs: $dtfs,
            range_regex: Range_LineIndex { start: $sib, end: $sie },
            cgn_first: $cgn_first,
            cgn_last: $cgn_last,
            #[cfg(any(debug_assertions,test))]
            _test_case: $test_case,
            _line_num: $line_num,
        }
    }
}
// allow easy macro import via `use s4lib::Data::datetime::DTPD;`
pub use DTPD;

// implement traits to allow sorting collections of `DateTime_Parse_Data`
// only used for tests

impl Ord for DateTime_Parse_Data<'_> {
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

impl PartialOrd for DateTime_Parse_Data<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DateTime_Parse_Data<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.regex_pattern == other.regex_pattern
        && self.dtfs == other.dtfs
    }
}

impl Eq for DateTime_Parse_Data<'_> {}

impl fmt::Debug for DateTime_Parse_Data<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // regexp strings can be very long, truncate it
        const maxlen: usize = 20;
        let mut rp: String = String::with_capacity(maxlen + 5);
        rp.extend(self.regex_pattern.chars().take(maxlen));
        if self.regex_pattern.len() > maxlen {
            rp.push('…');
        }
        let mut f_ = f.debug_struct("DateTime_Parse_Data:");
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


// chrono strftime formatting strings used in `datetime_parse_from_str`.
// `DTF` is "DateTime Format"
//
// These are effectively mappings to receive extracting datetime substrings in a `&str`
// then to rearrange those into order suitable for `captures_to_buffer`.
//
// The variable name represents what is available. The value represents it's rearranged form
// using in `captures_to_buffer`.

pub(crate) const DTFSS_YmdHMS: DTFSSet = DTFSSet {
    year: DTFS_Year::Y,
    month: DTFS_Month::m,
    day: DTFS_Day::d,
    hour: DTFS_Hour::H,
    minute: DTFS_Minute::M,
    second: DTFS_Second::S,
    fractional: DTFS_Fractional::_none,
    tz: DTFS_Tz::_fill,
    pattern: "%Y%m%dT%H%M%S%:z",
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
    pattern: "%Y%m%dT%H%M%S%z",
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
    pattern: "%Y%m%dT%H%M%S%:z",
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
    pattern: "%Y%m%dT%H%M%S%#z",
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
    pattern: "%Y%m%dT%H%M%S%z",
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
    pattern: "%Y%m%dT%H%M%S.%f%:z",
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
    pattern: "%Y%m%dT%H%M%S.%f%z",
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
    pattern: "%Y%m%dT%H%M%S.%f%:z",
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
    pattern: "%Y%m%dT%H%M%S.%f%#z",
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
    pattern: "%Y%m%dT%H%M%S.%f%:z",
};

/// %Z is mapped to %z by `captures_to_buffer`
const DTF_YmdHMSfZ: &DateTimePattern_str = "%Y%m%dT%H%M%S.%f%z";

const DTF_YbdHMSz: &DateTimePattern_str = "%Y%b%dT%H%M%S%z";
const DTF_YbdHMScz: &DateTimePattern_str = "%Y%b%dT%H%M%S%:z";
const DTF_YBdHMSz: &DateTimePattern_str = "%Y%B%dT%H%M%S%z";
/// %z is filled by `captures_to_buffer`
const DTF_YbdHMS: &DateTimePattern_str = "%Y%b%dT%H%M%S%z";
/// %z is filled by `captures_to_buffer`
const DTF_YBdHMS: &DateTimePattern_str = "%Y%B%dT%H%M%S%z";
/// %z is filled by `captures_to_buffer`
const DTF_YbeHMS: &DateTimePattern_str = "%Y%b%eT%H%M%S%z";
/// %z is filled by `captures_to_buffer`
const DTF_YBeHMS: &DateTimePattern_str = "%Y%B%eT%H%M%S%z";

/// %Y %z is filled by `captures_to_buffer`
const DTF_BdHMS: &DateTimePattern_str = "%Y%B%dT%H%M%S%z";
/// %Y %z is filled by `captures_to_buffer`
const DTF_BeHMS: &DateTimePattern_str = "%Y%B%eT%H%M%S%z";
/// %Y %z is filled by `captures_to_buffer`
const DTF_bdHMS: &DateTimePattern_str = "%Y%B%dT%H%M%S%z";
/// %Y %z is filled by `captures_to_buffer`
const DTF_beHMS: &DateTimePattern_str = "%Y%B%eT%H%M%S%z";

/// to aid testing
#[cfg(any(debug_assertions,test))]
pub(crate) const _DTF_ALL: &[&DateTimePattern_str] = &[
    //DTF_YmdHMS,
    //DTF_YmdHMSz,
    //DTF_YmdHMScz,
    //DTF_YmdHMSpz,
    //DTF_YmdHMSf,
    //DTF_YmdHMSfz,
    //DTF_YmdHMSfcz,
    //DTF_YmdHMSfpz,
    DTF_YmdHMSfZ,
    DTF_YbdHMSz,
    DTF_YbdHMScz,
    DTF_YBdHMSz,
    DTF_YbdHMS,
    DTF_YBdHMS,
    DTF_YbeHMS,
    DTF_YBeHMS,
    DTF_BdHMS,
    DTF_BeHMS,
    DTF_bdHMS,
    DTF_beHMS,
];


// `regex::Captures` capture group names

/// corresponds to strftime %Y
const CGN_YEAR: &CaptureGroupName = "year";
/// corresponds to strftime %m
const CGN_MONTH: &CaptureGroupName = "month";
/// corresponds to strftime %d
const CGN_DAY: &CaptureGroupName = "day";
/// corresponds to strftime %H
const CGN_HOUR: &CaptureGroupName = "hour";
/// corresponds to strftime %M
const CGN_MINUTE: &CaptureGroupName = "minute";
/// corresponds to strftime %S
const CGN_SECOND: &CaptureGroupName = "second";
/// corresponds to strftime %f
const CGN_FRACTIONAL: &CaptureGroupName = "fractional";
/// corresponds to strftime "%Z" "%z" "%:z" "%#z"
const CGN_TZ: &CaptureGroupName = "tz";
/// all capture group names, for testing
#[cfg(any(debug_assertions,test))]
pub(crate) const CGN_ALL_: [&CaptureGroupName; 8] = [
    CGN_YEAR,
    CGN_MONTH,
    CGN_DAY,
    CGN_HOUR,
    CGN_MINUTE,
    CGN_SECOND,
    CGN_FRACTIONAL,
    CGN_TZ,
];

/// regex capture group pattern for strftime year
pub const CGP_YEAR: &CaptureGroupPattern = r"(?P<year>[12]\d{3})";
/// regex capture group pattern for strftime month %m
pub const CGP_MONTHm: &CaptureGroupPattern = r"(?P<month>01|02|03|04|05|06|07|08|09|10|11|12)";
/// regex capture group pattern for strftime month %b
pub const CGP_MONTHb: &CaptureGroupPattern = r"(?P<month>Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)";
/// regex capture group pattern for strftime month %B
pub const CGP_MONTHB: &CaptureGroupPattern = r"(?P<month>January|February|March|April|May|June|July|August|September|October|November|December)";
/// regex capture group pattern for strftime day %d
pub const CGP_DAYd: &CaptureGroupPattern = r"(?P<day>01|02|03|04|05|06|07|08|09|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31)";
/// regex capture group pattern for strftime day %e
pub const CGP_DAYe: &CaptureGroupPattern = r"(?P<day> 1| 2| 3| 4| 5| 6| 7| 8| 9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31)";
/// regex capture group pattern for strftime hour %H
pub const CGP_HOUR: &CaptureGroupPattern = r"(?P<hour>00|01|02|03|04|05|06|07|08|09|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24)";
/// regex capture group pattern for strftime minute %M
pub const CGP_MINUTE: &CaptureGroupPattern = r"(?P<minute>[012345]\d)";
/// regex capture group pattern for strftime second %S, includes leap second "60"
pub const CGP_SECOND: &CaptureGroupPattern = r"(?P<second>[012345]\d|60)";
/// regex capture group pattern for strftime fractional %f,
/// all strftime patterns %f %3f %6f %9f
pub const CGP_FRACTIONAL: &CaptureGroupPattern = r"(?P<fractional>\d{3,9})";
/// for help in testing only
#[cfg(any(debug_assertions,test))]
pub(crate) const CGP_MONTH_ALL_: &[&CaptureGroupPattern] = &[
    CGP_MONTHm,
    CGP_MONTHb,
    CGP_MONTHB,
];
/// for help in testing only
#[cfg(any(debug_assertions,test))]
pub(crate) const CGP_DAY_ALL_: &[&CaptureGroupPattern] = &[
    CGP_DAYd,
    CGP_DAYe,
];
// Applicable tz offsets https://en.wikipedia.org/wiki/List_of_UTC_offsets
// Applicable tz abbreviations https://en.wikipedia.org/wiki/List_of_time_zone_abbreviations
// chrono strftime https://docs.rs/chrono/latest/chrono/format/strftime/index.html
//
/// %z +0930
pub(crate) const CGP_TZz: &CaptureGroupPattern = r"(?P<tz>[\+\-][01]\d{3})";
/// %:z +09:30
pub(crate) const CGP_TZcz: &CaptureGroupPattern = r"(?P<tz>[\+\-][01]\d:\d\d)";
/// %#z +09
pub(crate) const CGP_TZpz: &CaptureGroupPattern = r"(?P<tz>[\+\-][01]\d)";
/// %Z ACST
pub(crate) const CGP_TZZ: &CaptureGroupPattern = r"(?P<tz>ACDT|ACST|ACT|ADT|AEDT|AEST|AET|AFT|AKDT|AKST|ALMT|AMST|AMT|ANAT|AQTT|ART|AST|AWST|AZOT|AZT|BIOT|BIT|BNT|BOT|BRST|BRT|BST|BTT|CAT|CCT|CDT|CEST|CET|CHOT|CHST|CHUT|CIST|CKT|CLST|CLT|COST|COT|CST|CT|CVT|CWST|CXT|DAVT|DDUT|DFT|EAST|EAT|ECT|EDT|EEST|EET|EGST|EGT|EST|ET|FET|FJT|FKST|FKT|FNT|GALT|GAMT|GET|GFT|GILT|GIT|GMT|GST|GYT|HAEC|HDT|HKT|HMT|HOVT|HST|ICT|IDLW|IDT|IOT|IRDT|IRKT|IRST|IST|JST|KALT|KGT|KOST|KRAT|KST|LHST|LINT|MAGT|MART|MAWT|MDT|MEST|MET|MHT|MIST|MIT|MMT|MSK|MST|MUT|MVT|MYT|NCT|NDT|NFT|NOVT|NPT|NST|NT|NUT|NZDT|NZST|OMST|ORAT|PDT|PET|PETT|PGT|PHOT|PHST|PHT|PKT|PMDT|PMST|PONT|PST|PWT|PYST|PYT|RET|ROTT|SAKT|SAMT|SAST|SBT|SCT|SDT|SGT|SLST|SRET|SRT|SST|SYOT|TAHT|TFT|THA|TJT|TKT|TLT|TMT|TOT|TRT|TVT|ULAT|UTC|UYST|UYT|UZT|VET|VLAT|VOLT|VOST|VUT|WAKT|WAST|WAT|WEST|WET|WGST|WGT|WIB|WIT|WITA|WST|YAKT|YEKT)";
/// for help in testing only
pub(crate) const CGP_TZ_ALL_: &[&CaptureGroupPattern] = &[
    CGP_TZz,
    CGP_TZcz,
    CGP_TZpz,
    CGP_TZZ,
];

/// no uppercase, helper to `CGP_TZZ`
pub(crate) const RP_NOUPPER: &RegexPattern = r"([^[[:upper:]]]|$)";
/// timezone abbreviation, not followed by uppercase
pub(crate) const CGP_TZZn: &CaptureGroupPattern = concatcp!(CGP_TZZ, RP_NOUPPER);

pub(crate) const NOENTRY: &str = "";

/// all timezone abbreviations, maps all strftime "%Z" to strftime "%:z".
///
/// attempts to be more lenient than chrono
/// https://docs.rs/chrono/latest/chrono/format/strftime/#fn7
///
///
/// latest listing of timezone abbreviations can be retrieved by:
/*
    $ curl "https://en.wikipedia.org/wiki/List_of_time_zone_abbreviations" \
        | grep -Ee '^<td>[[:upper:]]{2,4}</td>' \
        | grep -oEe '[[:upper:]]{2,4}' \
        | sort \
        | uniq \
        | sed -Ee ':a;N;$!ba;s/\n/|/g'

    $ curl "https://en.wikipedia.org/wiki/List_of_time_zone_abbreviations" \
        | rg -or '$1 $2' -e '^<td>([[:upper:]]{2,5})</td>' -e '^<td data-sort-value.*>UTC(.*)</a>' \
        | sed -e '/^$/d' \
        | rg -r '("$1", ' -e '^([[:upper:]]{2,5})' -C5 \
        | rg -r '"$1"), ' -e '^[[:blank:]]*([[:print:]−±+]*[[:digit:]]{1,4}.*$)' -C5 \
        | rg -r '"$1:00"' -e '"(.?[[:digit:]][[:digit:]])"' -C5 \
        | sed -e 's/\n"/"/g' -e 'N;s/\n/ /' -e 's/−/-/g' -e 's/±/-/g' \
        | tr -s ' '
*/
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
const D_DHc: &RegexPattern = r"[ T:]?";
/// regexp divider day hour with dash, 2020:01:01-20:30:00
const D_DHd: &RegexPattern = r"[ T\-]?";
/// regexp divider day hour with colon or dash, 2020:01:01-20:30:00
const D_DHcd: &RegexPattern = r"[ T\-:]?";
/// regexp divider fractional, 2020/01/01T20:30:00,123456
const D_SF: &RegexPattern = r"[\.,]";

/// commonly found syslog level names
const RP_LEVELS: &RegexPattern = r"((?i)DEBUG|INFO|ERR|ERROR|TRACE|WARN|WARNING|VERBOSE(?-i))";
/// regex blank
const RP_BLANK: &RegexPattern = r"[[:blank:]]";
/// regex blank?
const RP_BLANKq: &RegexPattern = r"[[:blank:]]?";
/// regex blanks
const RP_BLANKS: &RegexPattern = r"[[:blank:]]+";
/// regex blanks?
const RP_BLANKSq: &RegexPattern = r"[[:blank:]]*";
/// left-side brackets
pub(crate) const RP_LB: &RegexPattern = r"[\[\(<{]";
/// right-side brackets
pub(crate) const RP_RB: &RegexPattern = r"[\]\)>}]";

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// the global list of built-in Datetime parsing "instructions"
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// index into the global `DATETIME_PARSE_DATAS`.
pub type DateTime_Parse_Datas_Index = usize;
/// a run-time created vector of `Regex` instances that is a counterpart to `DATETIME_PARSE_DATAS`.
pub type DateTime_Parse_Datas_Regex_vec = Vec<DateTimeRegex>;

pub const DATETIME_PARSE_DATAS_LEN: usize = 26;

/// built-in `const DateTime_Parse_Data` datetime parsing patterns.
///
/// These are all regexp patterns that will be attempted on processed files.
///
/// `DateTime_Parse_Data` should be listed from specific regexp to generic regexp. A more specific
/// regexp pattern is always preferred.
/// 
/// Notice the "with timezone" versions of `DateTime_Parse_Data` are often listed before the same
/// `DateTime_Parse_Data` "without". 
///
/// A drawback to specific-to-general approach: during `SyslineReader` initial reading stage, it
/// will try *all* the patterns (from index 0 to whereever it finds a match). So if a file has a
/// very general pattern (like it only matches the last listed `DateTime_Parse_Data` here) then
/// the `SyslineReader` will try *all* the `DateTime_Parse_Data` within `DATETIME_PARSE_DATAS`
/// several times (until `SyslineReader` is satisfied it has found the definitive pattern).
/// The many missed matches use a lot of resources and time.
///
pub const DATETIME_PARSE_DATAS: [DateTime_Parse_Data; DATETIME_PARSE_DATAS_LEN] = [
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
    //
    // LAST WORKING HERE 2022/06/29 15:41:00 implementing `DTFSSet` to replace old smattering of data, incomplete
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_RB),
        DTFSS_YmdHMSf, 0, 40, CGN_YEAR, CGN_FRACTIONAL,
        "[2000/01/01 00:00:01.123] ../source3/smbd/oplock.c:1340(init_oplocks)", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZz, RP_RB),
        DTFSS_YmdHMSfz, 0, 40, CGN_YEAR, CGN_TZ,
        "(2000/01/01 00:00:02.123456 -1100) ../source3/smbd/oplock.c:1340(init_oplocks)", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZcz, RP_RB),
        DTFSS_YmdHMSfcz, 0, 40, CGN_YEAR, CGN_TZ,
        r"{2000/01/01 00:00:03.123456789 -11:30} ../source3/smbd/oplock.c:1340(init_oplocks)", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZpz, RP_RB),
        DTFSS_YmdHMSfpz, 0, 40, CGN_YEAR, CGN_TZ,
        r"(2000/01/01 00:00:04.123456789 -11) ../source3/smbd/oplock.c:1340(init_oplocks)", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANK, CGP_TZZ, RP_RB),
        DTFSS_YmdHMSfZ, 0, 40, CGN_YEAR, CGN_TZ,
        r"(2000/01/01 00:00:05.123456789 VLAT) ../source3/smbd/oplock.c:1340(init_oplocks)", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, r"[,\.\| \t]", RP_BLANKSq, r"[[:word:]]{1,20}", RP_RB),
        DTFSS_YmdHMSf, 0, 40, CGN_YEAR, CGN_FRACTIONAL,
        "[2020/03/05 12:17:59.631000, FOO] ../source3/smbd/oplock.c:1340(init_oplocks)", line!(),
    ),
    /*
    //
    // from file `./logs/Ubuntu18/vmware-installer.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2019-05-06 11:24:34,074] Successfully loaded GTK libraries.
    //
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_RB),
        DTF_YmdHMS, true, false, false, false, 0, 30, CGN_YEAR, CGN_SECOND,
        "[20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, "[ ]?", D_SF, CGP_FRACTIONAL, RP_RB),
        DTF_YmdHMSf, true, false, false, false, 0, 40, CGN_YEAR, CGN_FRACTIONAL,
        r"[2019-05-06 11:24:34,074] Successfully loaded GTK libraries.", line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     [ERROR] 2000-01-02 12:33:01 -1200 1
    //     [WARNING] 2000-01-02 12:33:02 -1130 22
    //     [INFO] 2000-01-02 12:33:03 +1100 333
    //     [VERBOSE] 2000-01-02T12:33:04 -1030 4444
    //     [TRACE] 2000-01-02T12:33:05 -1000 55555
    //
    DTPD!(
        concatcp!("^", RP_LB, RP_LEVELS, RP_RB, RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz),
        DTF_YmdHMSz, true, false, true, false, 0, 40, CGN_YEAR, CGN_TZ,
        "[VERBOSE] 2000-01-02T12:33:04 -1030 blah", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, RP_LEVELS, RP_RB, RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZcz),
        DTF_YmdHMScz, true, false, true, false, 0, 40, CGN_YEAR, CGN_TZ,
        "[VERBOSE] 2000-01-02T12:33:04 -10:00 blah", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, RP_LEVELS, RP_RB, RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZpz),
        DTF_YmdHMSpz, true, false, true, false, 0, 40, CGN_YEAR, CGN_TZ,
        "[VERBOSE] 2000-01-02T12:33:04 -10 blah", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, RP_LEVELS, RP_RB, RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZZ),
        DTF_YmdHMSZ, true, false, true, true, 0, 40, CGN_YEAR, CGN_TZ,
        "[VERBOSE] 2000-01-02T12:33:04 PST blah", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LB, RP_LEVELS, RP_RB, RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND),
        DTF_YmdHMS, true, false, false, false, 0, 40, CGN_YEAR, CGN_SECOND,
        "[VERBOSE] 2000-01-02T12:33:04blah", line!(),
    ),
    //
    DTPD!(
        concatcp!("^", RP_LEVELS, r"[:]?", RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz),
        DTF_YmdHMSz, true, false, true, false, 0, 40, CGN_YEAR, CGN_TZ,
        "VERBOSE: 2000-01-02T12:33:04 -1030 blah", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LEVELS, r"[:]?", RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZcz),
        DTF_YmdHMScz, true, false, true, false, 0, 40, CGN_YEAR, CGN_TZ,
        "VERBOSE: 2000-01-02T12:33:04 -10:00 blah", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LEVELS, r"[:]?", RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZpz),
        DTF_YmdHMSpz, true, false, true, false, 0, 40, CGN_YEAR, CGN_TZ,
        "VERBOSE: 2000-01-02T12:33:04 -10 blah", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LEVELS, r"[:]?", RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZZ),
        DTF_YmdHMSZ, true, false, true, true, 0, 40, CGN_YEAR, CGN_TZ,
        "VERBOSE: 2000-01-02T12:33:04 PST blah", line!(),
    ),
    DTPD!(
        concatcp!("^", RP_LEVELS, r"[:]?", RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " "),
        DTF_YmdHMS, true, false, false, false, 0, 40, CGN_YEAR, CGN_SECOND,
        "VERBOSE: 2000-01-02T12:33:04 blah", line!(),
    ),
    //
    DTPD!(
        concatcp!(RP_LEVELS, r"[:]?", RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz),
        DTF_YmdHMSz, true, false, true, false, 0, 40, CGN_YEAR, CGN_TZ,
        "[kernel] VERBOSE: 2000-01-02T12:33:04 -1030 blah", line!(),
    ),
    DTPD!(
        concatcp!(RP_LEVELS, r"[:]?", RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZcz),
        DTF_YmdHMScz, true, false, true, false, 0, 40, CGN_YEAR, CGN_TZ,
        "[kernel] VERBOSE: 2000-01-02T12:33:04 -10:00 blah", line!(),
    ),
    DTPD!(
        concatcp!(RP_LEVELS, r"[:]?", RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZpz),
        DTF_YmdHMSpz, true, false, true, false, 0, 40, CGN_YEAR, CGN_TZ,
        "[kernel] VERBOSE: 2000-01-02T12:33:04 -10 blah", line!(),
    ),
    DTPD!(
        concatcp!(RP_LEVELS, r"[:]?", RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZZ),
        DTF_YmdHMSZ, true, false, true, true, 0, 40, CGN_YEAR, CGN_TZ,
        "[kernel] VERBOSE: 2000-01-02T12:33:04 PST blah", line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/synology/usbcopyd.log`
    //
    // example with offset:
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     2017-05-24T19:14:38-07:00 hostname1 usb-copy-starter
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
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, "[ ]?", CGP_TZz),
        DTF_YmdHMSz, true, true, false, 0, 30, CGN_YEAR, CGN_TZ,
        "2000-01-02 12:33:01 -0400 foo", line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, "[ ]?", CGP_TZZ),
        DTF_YmdHMSZ, true, true, true, 0, 30, CGN_YEAR, CGN_TZ,
        "2000-01-02 12:33:01 EAST foo", line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, "[ ]?", CGP_TZcz),
        DTF_YmdHMSZ, true, true, true, 0, 30, CGN_YEAR, CGN_TZ,
        "2000-01-02 12:33:01 -04:00 foo", line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, "[ ]?", CGP_TZpz),
        DTF_YmdHMSpz, true, true, true, 0, 30, CGN_YEAR, CGN_TZ,
        "2000-01-02 12:33:01 -04 foo", line!(),
    ),
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-02 12:33:05,123 -0400 foo
    //     2000-01-02 12:33:05,123 -04:00 foo
    //     2000-01-02T12:33:05,123 -0400 foo
    //     2000-01-02T12:33:05,123 -04:00 foo
    //
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, " ", CGP_TZz),
        DTF_YmdHMSfz, true, true, false, 0, 45, CGN_YEAR, CGN_TZ,
        "2000-01-02 12:33:01,123 -0400 foo", line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, " ", CGP_TZZ),
        DTF_YmdHMSfZ, true, true, true, 0, 45, CGN_YEAR, CGN_TZ,
        "2000-01-02 12:33:01,123 EAST foo", line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, " ", CGP_TZcz),
        DTF_YmdHMSfcZ, true, true, true, 0, 45, CGN_YEAR, CGN_TZ,
        "2000-01-02 12:33:01,123 -04:00 foo", line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, " ", CGP_TZpz),
        DTF_YmdHMSfpz, true, true, false, 0, 45, CGN_YEAR, CGN_TZ,
        "2000-01-02 12:33:01,123 -04 foo", line!(),
    ),
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-02 12:33:05.123456 foo
    //
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL),
        DTF_YmdHMSf, true, false, false, 0, 45, CGN_YEAR, CGN_FRACTIONAL,
        "2000-01-02 12:33:01.123456 foo", line!(),
    ),
    //               1         2         3
    //     0123456789012345678901234567890
    //     2000-01-02 12:33:05 foo
    //     2000-01-02 12:33:05 foo
    //     2000-01-02T12:33:05 foo
    //     2000-01-02T12:33:05 foo
    //
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, "[ ]?"),
        DTF_YmdHMS, true, false, false, 0, 20, CGN_YEAR, CGN_SECOND,
        "2000-01-02 12:33:01 foo", line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //               1         2         3
    //     0123456789012345678901234567890
    //     [ERROR] 2000-01-02 12:33:01 1
    //     [WARNING] 2000-01-02T12:33:02 22
    //     [INFO] 2000-01-02T12:33:03 333
    //     [VERBOSE] 2000-01-02 12:33:04 4444
    //     [TRACE] 2000-01-02 12:33:05 55555
    //
    DTPD!(
        concatcp!("^", RP_LB, RP_LEVELS, r"\][ ]?", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " "),
        DTF_YmdHMS, true, false, false, 0, 30, CGN_YEAR, CGN_SECOND,
        "[ERROR] 2000-01-02 12:33:01 foobar", line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/Ubuntu18/vmware/hostd-62.log`
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     2019-07-26T10:40:29.682-07:00 info hostd[03210] [Originator@6876 sub=Default] Current working directory: /usr/bin
    //
    DTPD!(
        // LAST WORKING HERE 2022/06/28 17:00:34 fails test though it looks fine
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, "[ ]?", CGP_TZz, RP_BLANKS, RP_LEVELS),
        DTF_YmdHMSfz, true, true, false, 0, 45, CGN_YEAR, CGN_TZ,
        "2019-07-26T10:40:29.123-0700 info hostd[03210] [Originator@6876 sub=Default]", line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, "[ ]?", CGP_TZcz, RP_BLANKS, RP_LEVELS),
        DTF_YmdHMSfcz, true, true, false, 0, 45, CGN_YEAR, CGN_TZ,
        "2019-07-26T10:40:29.123456-07:00 info hostd[03210] [Originator@6876 sub=Default]", line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, "[ ]?", CGP_TZpz, RP_BLANKS, RP_LEVELS),
        DTF_YmdHMSfpz, true, true, false, 0, 45, CGN_YEAR, CGN_TZ,
        "2019-07-26T10:40:29.123456789-07 info hostd[03210] [Originator@6876 sub=Default]", line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, " ", CGP_TZZ, RP_BLANKS, RP_LEVELS),
        DTF_YmdHMSfZ, true, true, true, 0, 45, CGN_YEAR, CGN_TZ,
        "2019-07-26T10:40:29.123456789 PST info hostd[03210] [Originator@6876 sub=Default]", line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/Ubuntu18/kernel.log`
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
    //     Jan 03 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119
    //     January  3 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119
    //
    DTPD!(
        concatcp!(r"^", CGP_MONTHb, " ", CGP_DAYd, " ", CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " "),
        DTF_bdHMS, false, false, false, 0, 20, CGN_MONTH, CGN_SECOND,
        "Mar 09 08:10:29 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode", line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_MONTHb, " ", CGP_DAYe, " ", CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " "),
        DTF_beHMS, false, false, false, 0, 20, CGN_MONTH, CGN_SECOND,
        "Mar  9 08:10:29 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode", line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_MONTHB, " ", CGP_DAYd, " ", CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " "),
        DTF_BdHMS, false, false, false, 0, 30, CGN_MONTH, CGN_SECOND,
        "January 03 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119", line!(),
    ),
    DTPD!(
        concatcp!(r"^", CGP_MONTHB, " ", CGP_DAYd, " ", CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " "),
        DTF_BeHMS, false, false, false, 0, 30, CGN_MONTH, CGN_SECOND,
        "January  3 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119", line!(),
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
    DTPD!(
        concatcp!(RP_LB, CGP_DAYd, D_D, CGP_MONTHb, D_D, CGP_YEAR, D_DHc, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, "[ ]?", CGP_TZz, RP_RB),
        DTF_YmdHMSz, true, true, false, 0, 100, CGN_DAY, CGN_TZ,
        r"E [30/Aug/2019:12:59:01 -0700] Unable to open listen socket for address [v1.::1]:631 - Cannot assign requested address.", line!(),
    ),
    DTPD!(
        concatcp!(RP_LB, CGP_DAYd, D_D, CGP_MONTHb, D_D, CGP_YEAR, D_DHc, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, "[ ]?", CGP_TZcz, RP_RB),
        DTF_YmdHMScz, true, true, false, 0, 100, CGN_DAY, CGN_TZ,
        r"E [30/Aug/2019:12:59:01 -07:00] Unable to open listen socket for address [v1.::1]:631 - Cannot assign requested address.", line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/other/archives/proftpd/xferlog`
    // example with offset:
    //
    //               1         2
    //     0123456789012345678901234
    //     Sat Oct 03 11:26:12 2020 0 192.168.1.1 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c
    //
    // XXX: ignore the leading Day Of Week substring
    DTPD!(
        concatcp!(CGP_MONTHb, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " ", CGP_YEAR, " ", CGP_TZz),
        DTF_YbdHMSz, true, true, false, 0, 100, CGN_MONTH, CGN_TZ,
        r"Sat Oct 03 11:26:59 2020 +0930 0 192.168.1.1 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c", line!(),
    ),
    DTPD!(
        concatcp!(CGP_MONTHb, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " ", CGP_YEAR, " ", CGP_TZcz),
        DTF_YbdHMScz, true, true, false, 0, 100, CGN_MONTH, CGN_TZ,
        r"Sat Oct 03 11:26:59 2020 +09:30 0 192.168.1.1 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c", line!(),
    ),
    // TODO: need to add the other timezone variations for each of these
    DTPD!(
        concatcp!(CGP_MONTHb, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " ", CGP_YEAR),
        DTF_YbdHMS, true, false, false, 0, 100, CGN_MONTH, CGN_YEAR,
        r"Sat Oct 03 11:26:59 2020 0 192.168.1.1 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c", line!(),
    ),
    DTPD!(
        concatcp!(CGP_MONTHB, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " ", CGP_YEAR),
        DTF_YBdHMS, true, false, false, 0, 100, CGN_MONTH, CGN_YEAR,
        r"Sat October 03 11:26:59 2020 0 192.168.1.1 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c", line!(),
    ),
    // TODO: the CGP_DAYe matches could be reduced by swallowing blanks RP_BLANKS, but how to replace leading zero?
    //       might need to add bool flag for that too
    DTPD!(
        concatcp!(CGP_MONTHb, D_D, CGP_DAYe, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " ", CGP_YEAR),
        DTF_YbeHMS, true, false, false, 0, 100, CGN_MONTH, CGN_YEAR,
        r"Sat Oct  3 11:26:59 2020 0 192.168.1.1 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c", line!(),
    ),
    DTPD!(
        concatcp!(CGP_MONTHb, D_D, CGP_DAYe, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " ", CGP_YEAR),
        DTF_YBeHMS, true, false, false, 0, 100, CGN_MONTH, CGN_YEAR,
        r"Sat October  3 11:26:59 2020 0 192.168.1.1 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c", line!(),
    ),
    //
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/OpenSUSE15/zypper.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2019-05-23 16:53:43 <1> trenker(24689) [zypper] main.cc(main):74 ===== Hi, me zypper 1.14.27
    //
    ////("%Y-%m-%d %H:%M:%S ", 0, 20, 0, 19),
    //
    // ---------------------------------------------------------------------------------------------
    // from file `./logs/synology/synoupdate.log`
    // example with offset:
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     2016/12/05 21:34:43	Start of the update…
    //
    //("%Y/%m/%d %H:%M:%S	", true, false, false, 0, 20, 0, 19),
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
    DTPD!(
        concatcp!(r"^", RP_LEVELS, RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZz, RP_BLANK, r"[[:alpha:]]"),
        DTF_YmdHMSz, true, true, false, 0, 50, CGN_YEAR, CGN_TZ,
        "info	2017/02/21 23:01:59 -0700	admin: Setting of backup task [Local Storage 1] was created", line!(),
    ),
    DTPD!(
        concatcp!(r"^", RP_LEVELS, RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZcz, RP_BLANK, r"[[:alpha:]]"),
        DTF_YmdHMScz, true, true, false, 0, 50, CGN_YEAR, CGN_TZ,
        "info	2017/02/21 23:01:59 -07:00	admin: Setting of backup task [Local Storage 1] was created", line!(),
    ),
    DTPD!(
        concatcp!(r"^", RP_LEVELS, RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZpz, RP_BLANK, r"[[:alpha:]]"),
        DTF_YmdHMSpz, true, true, false, 0, 50, CGN_YEAR, CGN_TZ,
        "info	2017/02/21 23:01:59 -07	admin: Setting of backup task [Local Storage 1] was created", line!(),
    ),
    DTPD!(
        concatcp!(r"^", RP_LEVELS, RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZZ, RP_BLANK, r"[[:alpha:]]"),
        DTF_YmdHMSZ, true, true, true, 0, 50, CGN_YEAR, CGN_TZ,
        "info	2017/02/21 23:01:59 PST	admin: Setting of backup task [Local Storage 1] was created", line!(),
    ),
    DTPD!(
        concatcp!(r"^", RP_LEVELS, RP_BLANKS, CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, r"[[:alpha:]]"),
        DTF_YmdHMS, true, false, false, 0, 50, CGN_YEAR, CGN_SECOND,
        "info	2017/02/21 23:01:59	admin: Setting of backup task [Local Storage 1] was created", line!(),
    ),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     2020-01-02 12:33:59.001 xyz
    //
    DTPD!(
        concatcp!(r"^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL),
        DTF_YmdHMSf, true, false, false, 0, 20, CGN_YEAR, CGN_FRACTIONAL,
        "2020-01-02 12:33:59.001 xyz", line!(),
    ),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/debian9/apport.log.1`
    // example with offset:
    //
    //               1         2         3         4         5
    //     012345678901234567890123456789012345678901234567890
    //     ERROR: apport (pid 9) Thu Feb 27 00:33:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 9359) Thu Feb 27 00:33:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    DTPD!(
        concatcp!(RP_BLANK, CGP_MONTHb, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " ", CGP_YEAR, " ", CGP_TZz, r"[: ]"),
        DTF_YmdHMSz, true, true, false, 0, 200, CGN_MONTH, CGN_TZ,
        "ERROR: apport (pid 9359) Thu Feb 27 00:33:59 2020 -0700: called for pid 8581, signal 24, core limit 0, dump mode 1", line!(),
    ),
    DTPD!(
        concatcp!(RP_BLANK, CGP_MONTHb, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " ", CGP_YEAR, " ", CGP_TZcz, r"[: ]"),
        DTF_YmdHMScz, true, true, false, 0, 200, CGN_MONTH, CGN_TZ,
        "ERROR: apport (pid 9359) Thu Feb 27 00:33:59 2020 -07:00: called for pid 8581, signal 24, core limit 0, dump mode 1", line!(),
    ),
    DTPD!(
        concatcp!(RP_BLANK, CGP_MONTHb, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " ", CGP_YEAR, " ", CGP_TZpz, r"[: ]"),
        DTF_YmdHMSpz, true, true, false, 0, 200, CGN_MONTH, CGN_TZ,
        "ERROR: apport (pid 9359) Thu Feb 27 00:33:59 2020 -07: called for pid 8581, signal 24, core limit 0, dump mode 1", line!(),
    ),
    DTPD!(
        concatcp!(RP_BLANK, CGP_MONTHb, D_D, CGP_DAYd, D_DH, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, " ", CGP_YEAR, r":"),
        DTF_YmdHMSz, true, false, false, 0, 200, CGN_MONTH, CGN_YEAR,
        "ERROR: apport (pid 9359) Thu Feb 27 00:33:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1", line!(),
    ),
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
    //(" %a %b %d %H:%M:%S %Y ", true, false, false, 5, 31, 6, 30),
    //(" %a %b %d %H:%M:%S %Y ", true, false, false, 6, 32, 7, 31),
    //(" %a %b %d %H:%M:%S %Y ", true, false, false, 7, 33, 8, 32),
    //(" %a %b %d %H:%M:%S %Y ", true, false, false, 8, 34, 9, 33),
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
    //(" %a %b %d %Y %H:%M:%S ", true, false, false, 5, 31, 6, 30),
    //(" %a %b %d %Y %H:%M:%S ", true, false, false, 6, 32, 7, 31),
    //(" %a %b %d %Y %H:%M:%S ", true, false, false, 7, 33, 8, 32),
    //(" %a %b %d %Y %H:%M:%S ", true, false, false, 8, 34, 9, 33),
    // ---------------------------------------------------------------------------------------------
    // TODO: [2022/03/24] add timestamp formats seen at https://www.unixtimestamp.com/index.php
    */
    //
    // general matches from start
    //
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZz),
        DTFSS_YmdHMSfz, 0, 40, CGN_YEAR, CGN_TZ,
        "2000/01/02 00:00:02.123 -1100 a", line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZcz),
        DTFSS_YmdHMSfcz, 0, 40, CGN_YEAR, CGN_TZ,
        r"2000/01/03 00:00:03.123456 -11:30 ab", line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZpz),
        DTFSS_YmdHMSfpz, 0, 40, CGN_YEAR, CGN_TZ,
        r"2000/01/04 00:00:04,123456789 -11 abc", line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANK, CGP_TZZ),
        DTFSS_YmdHMSfZ, 0, 40, CGN_YEAR, CGN_TZ,
        r"2000/01/05 00:00:05.123456789 VLAT abcd", line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL),
        DTFSS_YmdHMSf, 0, 40, CGN_YEAR, CGN_FRACTIONAL,
        "2020-01-06 00:00:26.123456789 abcdefg", line!(),
    ),
    //
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz),
        DTFSS_YmdHMSz, 0, 40, CGN_YEAR, CGN_TZ,
        "2000/01/07T00:00:02 -1100 abcdefgh", line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZcz),
        DTFSS_YmdHMScz, 0, 40, CGN_YEAR, CGN_TZ,
        r"2000-01-08-00:00:03 -11:30 aabcdefghi", line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZpz),
        DTFSS_YmdHMSpz, 0, 40, CGN_YEAR, CGN_TZ,
        r"2000/01/09 00:00:04 -11 abcdefghij", line!(),
    ),
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZZ),
        DTFSS_YmdHMSZ, 0, 40, CGN_YEAR, CGN_TZ,
        r"2000/01/10T00:00:05 VLAT abcdefghijk", line!(),
    ),
    //
    DTPD!(
        concatcp!("^", CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND),
        DTFSS_YmdHMS, 0, 40, CGN_YEAR, CGN_SECOND,
        "2020-01-11 00:00:26 abcdefghijkl", line!(),
    ),
    //
    // general matches anywhere in the first 1024 bytes of the line
    //
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZz),
        DTFSS_YmdHMSfz, 0, 1024, CGN_YEAR, CGN_TZ,
        "2000/01/02 00:01:02.123 -1100 a", line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZcz),
        DTFSS_YmdHMSfcz, 0, 1024, CGN_YEAR, CGN_TZ,
        r"2000/01/03 00:02:03.123456 -11:30 ab", line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANKq, CGP_TZpz),
        DTFSS_YmdHMSfpz, 0, 1024, CGN_YEAR, CGN_TZ,
        r"2000/01/04 00:03:04,123456789 -11 abc", line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL, RP_BLANK, CGP_TZZ),
        DTFSS_YmdHMSfZ, 0, 1024, CGN_YEAR, CGN_TZ,
        r"2000/01/05 00:04:05.123456789 VLAT abcd", line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, D_SF, CGP_FRACTIONAL),
        DTFSS_YmdHMSf, 0, 1024, CGN_YEAR, CGN_FRACTIONAL,
        "2020-01-06 00:05:26.123456789 abcdefg", line!(),
    ),
    //
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZz),
        DTFSS_YmdHMSz, 0, 1024, CGN_YEAR, CGN_TZ,
        "2000/01/07T00:06:02 -1100 abcdefgh", line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZcz),
        DTFSS_YmdHMScz, 0, 1024, CGN_YEAR, CGN_TZ,
        r"2000-01-08-00:07:03 -11:30 aabcdefghi", line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANKq, CGP_TZpz),
        DTFSS_YmdHMSpz, 0, 1024, CGN_YEAR, CGN_TZ,
        r"2000/01/09 00:08:04 -11 abcdefghij", line!(),
    ),
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND, RP_BLANK, CGP_TZZ),
        DTFSS_YmdHMSZ, 0, 1024, CGN_YEAR, CGN_TZ,
        r"2000/01/10T00:09:05 VLAT abcdefghijk", line!(),
    ),
    //
    DTPD!(
        concatcp!(CGP_YEAR, D_D, CGP_MONTHm, D_D, CGP_DAYd, D_DHcd, CGP_HOUR, D_T, CGP_MINUTE, D_T, CGP_SECOND),
        DTFSS_YmdHMS, 0, 1024, CGN_YEAR, CGN_SECOND,
        "2020-01-11 00:10:26 abcdefghijkl", line!(),
    ),
];

lazy_static! {
    // `Regex::new` runs at run-time, create this vector on-demand
    pub(crate) static ref DATETIME_PARSE_DATAS_REGEX_VEC: DateTime_Parse_Datas_Regex_vec =
        DATETIME_PARSE_DATAS.iter().map(
            |x| Regex::new(x.regex_pattern).unwrap()
        ).collect();
}

/// workaround for chrono Issue #660 https://github.com/chronotope/chrono/issues/660
/// match spaces at beginning and ending of inputs
/// TODO: handle all Unicode whitespace.
///       This fn is essentially counteracting an errant call to `std::string:trim`
///       within `Local.datetime_from_str`.
///       `trim` removes "Unicode Derived Core Property White_Space".
///       This implementation handles three whitespace chars. There are twenty-five whitespace
///       chars according to
///       https://en.wikipedia.org/wiki/Unicode_character_property#Whitespace
pub fn datetime_from_str_workaround_Issue660(value: &str, pattern: &DateTimePattern_str) -> bool {
    let spaces = " ";
    let tabs = "\t";
    let lineends = "\n\r";

    // match whitespace forwards from beginning
    let mut v_sc: u32 = 0;  // `value` spaces count
    let mut v_tc: u32 = 0;  // `value` tabs count
    let mut v_ec: u32 = 0;  // `value` line ends count
    let mut v_brk: bool = false;
    for v_ in value.chars() {
        if spaces.contains(v_) {
            v_sc += 1;
        } else if tabs.contains(v_) {
            v_tc += 1;
        } else if lineends.contains(v_) {
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
        if spaces.contains(p_) {
            p_sc += 1;
        } else if tabs.contains(p_) {
            p_tc += 1;
        } else if lineends.contains(p_) {
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
            if spaces.contains(v_) {
                v_sc += 1;
            } else if tabs.contains(v_) {
                v_tc += 1;
            } else if lineends.contains(v_) {
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
            if spaces.contains(p_) {
                p_sc += 1;
            } else if tabs.contains(p_) {
                p_tc += 1;
            } else if lineends.contains(p_) {
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
pub fn u8_to_str(slice_: &[u8]) -> Option<&str> {
    let dts: &str;
    let mut fallback = false;
    // custom check for UTF8; fast but imperfect
    if ! slice_.is_ascii() {
        fallback = true;
    }
    if fallback {
        // found non-ASCII, fallback to checking with `utf8_latin1_up_to` which is a thorough check
        let va = encoding_rs::mem::utf8_latin1_up_to(slice_);
        if va != slice_.len() {
            return None;  // invalid UTF8
        }
    }
    unsafe {
        dts = std::str::from_utf8_unchecked(slice_);
    };
    Some(dts)
}

/// convert a `&str` to a chrono `Option<DateTime<FixedOffset>>` instance.
/// compensate for missing year or missing timezone
pub fn datetime_parse_from_str(
    data: &str,
    pattern: &DateTimePattern_str,
    has_year: bool,
    has_tz: bool,
    tz_offset: &FixedOffset,
) -> DateTimeL_Opt {
    debug_eprintln!("{}datetime_parse_from_str({:?}, {:?}, {:?})", sn(), str_to_String_noraw(data), pattern, tz_offset);
    // TODO: 2022/04/07
    //       if dt_pattern has TZ then create a `DateTime`
    //       if dt_pattern does not have TZ then create a `NaiveDateTime`
    //       then convert that to `DateTime` with aid of crate `chrono_tz`
    //       TZ::from_local_datetime();
    //       How to determine TZ to use? Should it just use Local?
    //       Defaulting to local TZ would be an adequate start.
    //       But pass around as `chrono::DateTime`, not `chrono::Local`.
    //       Replace use of `Local` with `DateTime. Change typecast `DateTimeL`
    //       type. Can leave the name in place for now.
    if has_year && has_tz {
        match DateTime::parse_from_str(data, pattern) {
            Ok(val) => {
                debug_eprintln!(
                    "{}datetime_parse_from_str: DateTime::parse_from_str({:?}, {:?}) extrapolated DateTime {:?}",
                    so(),
                    str_to_String_noraw(data),
                    pattern,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `data`
                //       and `dt_pattern`
                if !datetime_from_str_workaround_Issue660(data, pattern) {
                    debug_eprintln!("{}datetime_parse_from_str: skip match due to chrono Issue #660", sx());
                    return None;
                }
                debug_eprintln!("{}datetime_parse_from_str return {:?}", sx(), Some(val));

                Some(val)
            }
            Err(_err) => {
                debug_eprintln!("{}datetime_parse_from_str: DateTime::parse_from_str({:?}, {:?}) failed ParseError: {}", sx(), data, pattern, _err);

                None
            }
        }
    } else if !has_tz {
        // no timezone in `pattern` so first convert to a `NaiveDateTime` instance
        let dt_naive = match NaiveDateTime::parse_from_str(data, pattern) {
            Ok(val) => {
                debug_eprintln!(
                    "{}datetime_parse_from_str: NaiveDateTime.parse_from_str({:?}, {:?}) extrapolated NaiveDateTime {:?}",
                    so(),
                    str_to_String_noraw(data),
                    pattern,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `data`
                //       and `pattern`
                if !datetime_from_str_workaround_Issue660(data, pattern) {
                    debug_eprintln!("{}datetime_parse_from_str: skip match due to chrono Issue #660", sx());
                    return None;
                }
                val
            }
            Err(_err) => {
                debug_eprintln!("{}datetime_parse_from_str: NaiveDateTime.parse_from_str({:?}, {:?}) failed ParseError: {}", sx(), data, pattern, _err);
                return None;
            }
        };
        // second convert the `NaiveDateTime` instance to `DateTime<FixedOffset>` instance
        match tz_offset.from_local_datetime(&dt_naive).earliest() {
            Some(val) => {
                debug_eprintln!(
                    "{}datetime_parse_from_str: tz_offset.from_local_datetime({:?}).earliest() extrapolated NaiveDateTime {:?}",
                    so(),
                    dt_naive,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `data`
                //       and `pattern`
                if !datetime_from_str_workaround_Issue660(data, pattern) {
                    debug_eprintln!("{}datetime_parse_from_str: skip match due to chrono Issue #660, return None", sx());
                    return None;
                }
                debug_eprintln!("{}datetime_parse_from_str return {:?}", sx(), Some(val));

                Some(val)
            }
            None => {
                debug_eprintln!("{}datetime_parse_from_str: tz_offset.from_local_datetime({:?}, {:?}) returned None, return None", sx(), data, pattern);
                None
            }
        }
    } else {
        panic!("Not implemented !has_year, TODO: implement it")
    }
}

/// data of interest from a set of `regex::Captures` for a datetime substring found in a `Line`
///
/// - datetime substring begin index
/// - datetime substring end index
/// - datetime
pub type CapturedDtData = (LineIndex, LineIndex, DateTimeL);

/// Put `Captures` into a `String` buffer in a particular order and formatting. This bridges the
/// `DateTime_Parse_Data::regex_pattern` to `DateTime_Parse_Data::dt_pattern`.
///
/// Directly relates to datetime format `dt_pattern` values in `DATETIME_PARSE_DATAS`
/// which use `DTFSS_YmdHMS`, etc.
#[inline(always)]
fn captures_to_buffer(
    buffer: &mut String,
    captures: &regex::Captures,
    tz_offset: &FixedOffset,
    dtfs: &DTFSSet,
) {
    // year
    match captures.name(CGN_YEAR).as_ref() {
        Some(match_) => {
            buffer.push_str(match_.as_str());
        }
        None => {
            // TODO: 2022/06/27 do something smarter than setting current year
            buffer.push_str(Local::today().year().to_string().as_str());
        }
    }
    // month
    buffer.push_str(captures.name(CGN_MONTH).as_ref().unwrap().as_str());
    // day
    match dtfs.day {
        DTFS_Day::d | DTFS_Day::e => {
            buffer.push_str(captures.name(CGN_DAY).as_ref().unwrap().as_str());
        },
        DTFS_Day::_de_to_d => {
            let day: &str = captures.name(CGN_DAY).as_ref().unwrap().as_str();
            debug_assert_eq!(day.len(), 2, "bad named group 'day' data {:?}, expected data of len 2", day);
            match day.chars().next() {
                // change day " 8" to "08"
                Some(' ') => {
                    buffer.push('0');
                    buffer.push(day.chars().nth(1).unwrap());
                }
                Some(_) => {
                    buffer.push_str(day);
                }
                None => {
                    panic!("day.chars().next() returned None, {:?}", day);
                }
            }
        }
    }
    buffer.push('T');
    // hour
    buffer.push_str(captures.name(CGN_HOUR).as_ref().unwrap().as_str());
    // minute
    buffer.push_str(captures.name(CGN_MINUTE).as_ref().unwrap().as_str());
    // second
    buffer.push_str(captures.name(CGN_SECOND).as_ref().unwrap().as_str());
    // fractional
    match dtfs.fractional {
        DTFS_Fractional::f => {
            buffer.push('.');
            buffer.push_str(captures.name(CGN_FRACTIONAL).as_ref().unwrap().as_str());
        }
        DTFS_Fractional::_none => {}
    }
    // tz
    match dtfs.tz {
        DTFS_Tz::_fill => {
            // TODO: cost-savings: pass pre-created TZ `&str`
            buffer.push_str(tz_offset.to_string().as_str());
        }
        DTFS_Tz::z | DTFS_Tz::cz | DTFS_Tz::pz => {
            buffer.push_str(captures.name(CGN_TZ).as_ref().unwrap().as_str());
        }
        DTFS_Tz::Z => {
            let tzZ: &str = captures.name(CGN_TZ).as_ref().unwrap().as_str();
            match MAP_TZZ_TO_TZz.get_key_value(tzZ) {
                Some((_tz_abbr, tz_offset_)) => {
                    buffer.push_str(tz_offset_);
                }
                None => {
                    // cannot find entry in MAP_TZZ_TO_TZz, fill with passed TZ
                    buffer.push_str(tz_offset.to_string().as_str());
                }
            }

        }
    }
    debug_eprintln!("{}captures_to_buffer buffer {:?}", snx(), buffer);
}

/// run `regex::Captures` on the `data` then convert to a chrono
/// `Option<DateTime<FixedOffset>>` instance. Uses matching and pattern information
/// hardcoded in `DATETIME_PARSE_DATAS_REGEX` and `DATETIME_PARSE_DATAS`.
pub fn str_to_regex_to_datetime(
    data: &str,
    index: &DateTime_Parse_Datas_Index,
    tz_offset: &FixedOffset,
) -> Option<CapturedDtData> {
    debug_eprintln!("{}str_to_regex_to_datetime({:?}, {:?}, {:?})", sn(), data, index, tz_offset);

    let regex_: &Regex = match DATETIME_PARSE_DATAS_REGEX_VEC.get(*index) {
        Some(val) => val,
        None => {
            panic!("requested DATETIME_PARSE_DATAS_REGEX_VEC.get({}), returned None. DATETIME_PARSE_DATAS_REGEX_VEC.len() {}", index, DATETIME_PARSE_DATAS_REGEX_VEC.len());
        }
    };

    let captures: regex::Captures = match regex_.captures(data) {
        None => {
            debug_eprintln!("{}str_to_regex_to_datetime: regex: no captures (returned None)", sx());
            return None;
        }
        Some(captures) => {
            debug_eprintln!("{}str_to_regex_to_datetime: regex: captures.len() {}", so(), captures.len());

            captures
        }
    };
    if cfg!(debug_assertions) {
        for (i, name_opt) in regex_.capture_names().enumerate() {
            let match_ = match captures.get(i) {
                Some(m_) => m_,
                None => {
                    match name_opt {
                        Some(name) => {
                            eprintln!("{}str_to_regex_to_datetime: regex captures: {:2} {:<20} None", so(), i, name);
                        },
                        None => {
                            eprintln!("{}str_to_regex_to_datetime: regex captures: {:2} {:<20} None", so(), i, "None");
                        }
                    }
                    continue;
                }
            };
            match name_opt {
                Some(name) => {
                    eprintln!("{}str_to_regex_to_datetime: regex captures: {:2} {:<20} {:?}", so(), i, name, match_.as_str());
                },
                None => {
                    eprintln!("{}str_to_regex_to_datetime: regex captures: {:2} {:<20} {:?}", so(), i, "NO NAME", match_.as_str());
                }
            }
            
        }
    }
    // sanity check
    debug_assert!(!captures.iter().any(|x| x.is_none()), "a match in the regex::Captures was None");

    
    let dtpd: &DateTime_Parse_Data = &DATETIME_PARSE_DATAS[*index];
    // copy regex matches into a buffer with predictable ordering
    // this ordering relates to datetime format strings in `DATETIME_PARSE_DATAS`
    // TODO: [2022/06/26] cost-savings: avoid a `String` alloc by passing precreated buffer
    let mut buffer = String::with_capacity(35);
    captures_to_buffer(&mut buffer, &captures, tz_offset, &dtpd.dtfs);

    // use the `dt_format` to parse the buffer of regex matches
    let dt = match datetime_parse_from_str(
        buffer.as_str(),
        dtpd.dtfs.pattern,
        dtpd.dtfs.has_year(),
        dtpd.dtfs.has_tz(),
        tz_offset,
    ) {
        Some(dt_) => dt_,
        None => {
            return None;
        }
    };

    // derive the `LineIndex` bounds of the datetime substring within `data`
    let dt_beg: LineIndex = match captures.name(dtpd.cgn_first) {
        Some(match_) => match_.start() as LineIndex,
        None => 0,
    };
    let dt_end: LineIndex = match captures.name(dtpd.cgn_last) {
        Some(match_) => match_.end() as LineIndex,
        None => 0,
    };
    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}, index {}", dt_beg, dt_end, index);

    debug_eprintln!("{}str_to_regex_to_datetime: return Some({:?}, {:?}, {:?})", sx(), dt_beg, dt_end, dt);
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
pub fn dt_after_or_before(dt: &DateTimeL, dt_filter: &DateTimeL_Opt) -> Result_Filter_DateTime1 {
    if dt_filter.is_none() {
        debug_eprintln!("{}dt_after_or_before(…) return Result_Filter_DateTime1::Pass; (no dt filters)", snx(),);
        return Result_Filter_DateTime1::Pass;
    }

    let dt_a = &dt_filter.unwrap();
    debug_eprintln!("{}dt_after_or_before comparing dt datetime {:?} to filter datetime {:?}", sn(), dt, dt_a);
    if dt < dt_a {
        debug_eprintln!("{}dt_after_or_before(…) return Result_Filter_DateTime1::OccursBefore; (dt {:?} is before dt_filter {:?})", sx(), dt, dt_a);
        return Result_Filter_DateTime1::OccursBefore;
    }
    debug_eprintln!("{}dt_after_or_before(…) return Result_Filter_DateTime1::OccursAtOrAfter; (dt {:?} is at or after dt_filter {:?})", sx(), dt, dt_a);

    Result_Filter_DateTime1::OccursAtOrAfter
}

/// If both filters are `Some` and `syslinep.dt` is "between" the filters then return `Pass`
/// comparison is "inclusive" i.e. `dt` == `dt_filter_after` will return `Pass`
/// If both filters are `None` then return `Pass`
/// TODO: finish this docstring
pub fn dt_pass_filters(
    dt: &DateTimeL, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
) -> Result_Filter_DateTime2 {
    debug_eprintln!("{}dt_pass_filters({:?}, {:?}, {:?})", sn(), dt, dt_filter_after, dt_filter_before,);
    if dt_filter_after.is_none() && dt_filter_before.is_none() {
        debug_eprintln!(
            "{}dt_pass_filters(…) return Result_Filter_DateTime2::InRange; (no dt filters)",
            sx(),
        );
        return Result_Filter_DateTime2::InRange;
    }
    if dt_filter_after.is_some() && dt_filter_before.is_some() {
        debug_eprintln!(
            "{}dt_pass_filters comparing datetime dt_filter_after {:?} < {:?} dt < {:?} dt_fiter_before ???",
            so(),
            &dt_filter_after.unwrap(),
            dt,
            &dt_filter_before.unwrap()
        );
        let da = &dt_filter_after.unwrap();
        let db = &dt_filter_before.unwrap();
        assert_le!(da, db, "Bad datetime range values filter_after {:?} {:?} filter_before", da, db);
        if dt < da {
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::BeforeRange;", sx());
            return Result_Filter_DateTime2::BeforeRange;
        }
        if db < dt {
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::AfterRange;", sx());
            return Result_Filter_DateTime2::AfterRange;
        }
        // assert da < dt && dt < db
        assert_le!(da, dt, "Unexpected range values da dt");
        assert_le!(dt, db, "Unexpected range values dt db");
        debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::InRange;", sx());

        Result_Filter_DateTime2::InRange
    } else if dt_filter_after.is_some() {
        debug_eprintln!(
            "{}dt_pass_filters comparing datetime dt_filter_after {:?} < {:?} dt ???",
            so(),
            &dt_filter_after.unwrap(),
            dt
        );
        let da = &dt_filter_after.unwrap();
        if dt < da {
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::BeforeRange;", sx());
            return Result_Filter_DateTime2::BeforeRange;
        }
        debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::InRange;", sx());

        Result_Filter_DateTime2::InRange
    } else {
        debug_eprintln!(
            "{}dt_pass_filters comparing datetime dt {:?} < {:?} dt_filter_before ???",
            so(),
            dt,
            &dt_filter_before.unwrap()
        );
        let db = &dt_filter_before.unwrap();
        if db < dt {
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::AfterRange;", sx());
            return Result_Filter_DateTime2::AfterRange;
        }
        debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::InRange;", sx());
        return Result_Filter_DateTime2::InRange;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions - search a slice quickly (loop unroll version)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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

/// loop unrolled implementation of `slice.contains` for a byte slice and a hardcorded array
/// benchmark `benches/bench_slice_contains.rs` demonstrates this is faster
#[inline(always)]
pub fn slice_contains_X_2(slice_: &[u8], search: &[u8; 2]) -> bool {
    match slice_.len() {
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
        _ => {
            for c in slice_.iter() {
                if c == &search[0] || c == &search[1] {
                    return true;
                }
            }
            false
        }
    }
}
