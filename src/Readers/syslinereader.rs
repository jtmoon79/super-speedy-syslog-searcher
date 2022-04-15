// Readers/syslinereader.rs
//
// TODO: move datetime stuff to it's own `datetime.rs`

pub use crate::common::{
    FPath,
    FileOffset,
    NLu8,
    CharSz,
};

use crate::Readers::blockreader::{
    BlockSz,
    BlockOffset,
    BlockIndex,
    Slices,
};

use crate::Readers::linereader::{
    LineIndex,
    Line,
    LineP,
    Lines,
    LineReader,
    ResultS4_LineFind,
    enum_BoxPtrs,
};

use crate::Readers::summary::{
    Summary,
};

use crate::common::{
    Bytes,
    ResultS4,
};

#[cfg(any(debug_assertions,test))]
use crate::dbgpr::printers::{
    str_to_String_noraw,
};
use crate::dbgpr::printers::{
    Color,
    ColorSpec,
    WriteColor,
};

use crate::dbgpr::stack::{
    sn,
    snx,
    so,
    sx,
};

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::io;
//use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};
use std::io::{
    Error,
    Result,
    ErrorKind,
    //Seek,
    //SeekFrom,
};
use std::io::prelude::*;
use std::str;
use std::sync::Arc;

extern crate arrayref;
use arrayref::array_ref;

extern crate chain_cmp;
use chain_cmp::chmp;

extern crate chrono;
use chrono::{
    DateTime,
    //Local,
    //Offset,
    NaiveDateTime,
    TimeZone,
    //Utc
};
pub use chrono::{
    FixedOffset,
    Local,
    Utc,
};

extern crate debug_print;
use debug_print::{debug_eprint, debug_eprintln};

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate lru;
use lru::LruCache;

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    assert_ge,
    assert_gt,
    debug_assert_le,
    debug_assert_lt,
    debug_assert_ge,
    //debug_assert_gt,
};

extern crate rangemap;
use rangemap::RangeMap;

extern crate unroll;
use unroll::unroll_for_loops;


/// testing helper
/// TODO: move this elsewhere
#[cfg(test)]
pub fn randomize(v_: &mut Vec<FileOffset>) {
    // XXX: can also use `rand::shuffle` https://docs.rs/rand/0.8.4/rand/seq/trait.SliceRandom.html#tymethod.shuffle
    let sz = v_.len();
    let mut i = 0;
    while i < sz {
        let r = rand::random::<usize>() % sz;
        v_.swap(r, i);
        i += 1;
    }
}

/// testing helper
/// TODO: move this elsewhere
#[cfg(test)]
pub fn fill(v_: &mut Vec<FileOffset>) {
    let sz = v_.capacity();
    let mut i = 0;
    while i < sz {
        v_.push(i as FileOffset);
        i += 1;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sysline
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// DateTime typing

/// typical DateTime with TZ type
pub type DateTimeL = DateTime<FixedOffset>;
#[allow(non_camel_case_types)]
pub type DateTimeL_Opt = Option<DateTimeL>;
/// Sysline Searching error
/// TODO: does SyslineFind need an `Found_EOF` state? Is it an unnecessary overlap of `Ok` and `Done`?
#[allow(non_camel_case_types)]
pub type ResultS4_SyslineFind = ResultS4<(FileOffset, SyslineP), Error>;

/// A `Sysline` has information about a "syslog line" that spans one or more `Line`s
/// a "syslog line" is one or more lines, where the first line starts with a
/// datetime stamp. That datetime stamp is consistent format of other nearby syslines.
pub struct Sysline {
    /// the one or more `Line` that make up a Sysline
    pub(crate) lines: Lines,
    /// index into `Line` where datetime string starts
    /// byte-based count
    /// datetime is presumed to be on first Line
    pub(crate) dt_beg: LineIndex,
    /// index into `Line` where datetime string ends, one char past last character of datetime string
    /// byte-based count
    /// datetime is presumed to be on first Line
    pub(crate) dt_end: LineIndex,
    /// parsed DateTime instance
    pub(crate) dt: DateTimeL_Opt,
}

/// a signifier value for "not set" or "null" - because sometimes Option is a PitA
const LI_NULL: LineIndex = LineIndex::MAX;

impl fmt::Debug for Sysline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut li_s = String::new();
        for lp in self.lines.iter() {
            li_s.push_str(&format!(
                "Line @{:p} (fileoffset_beg {}, fileoffset_end {}, len() {}, count() {}",
                &*lp,
                (*lp).fileoffset_begin(),
                (*lp).fileoffset_end(),
                (*lp).len(),
                (*lp).count()
            ));
        }
        f.debug_struct("Sysline")
            .field("fileoffset_begin()", &self.fileoffset_begin())
            .field("fileoffset_end()", &self.fileoffset_end())
            .field("lines @", &format_args!("{:p}", &self.lines))
            .field("lines.len", &self.lines.len())
            .field("dt_beg", &self.dt_beg)
            .field("dt_end", &self.dt_end)
            .field("dt", &self.dt)
            .field("lines", &li_s)
            .finish()
    }
}

impl Default for Sysline {
    fn default() -> Self {
        Self {
            lines: Lines::with_capacity(Sysline::SYSLINE_PARTS_WITH_CAPACITY),
            dt_beg: LI_NULL,
            dt_end: LI_NULL,
            dt: None,
        }
    }
}

impl Sysline {
    /// default `with_capacity` for a `Lines`, most often will only need 1 capacity
    /// as the found "sysline" will likely be one `Line`
    const SYSLINE_PARTS_WITH_CAPACITY: usize = 1;
    // XXX: does not handle multi-byte encodings
    const CHARSZ: usize = 1;

    pub fn new() -> Sysline {
        Sysline::default()
    }

    pub fn new_from_line(linep: LineP) -> Sysline {
        let mut v = Lines::with_capacity(Sysline::SYSLINE_PARTS_WITH_CAPACITY);
        v.push(linep);
        Sysline {
            lines: v,
            dt_beg: LI_NULL,
            dt_end: LI_NULL,
            dt: None,
        }
    }

    pub fn charsz(self: &Sysline) -> usize {
        Sysline::CHARSZ
    }

    pub fn push(&mut self, linep: LineP) {
        if !self.lines.is_empty() {
            // TODO: sanity check lines are in sequence
        }
        debug_eprintln!(
            "{}SyslineReader.push(@{:p}), self.lines.len() is now {}",
            so(),
            &*linep,
            self.lines.len() + 1
        );
        self.lines.push(linep);
    }

    /// the byte offset into the file where this `Sysline` begins
    /// "points" to first character of `Sysline` (and underlying `Line`)
    pub fn fileoffset_begin(self: &Sysline) -> FileOffset {
        assert_ne!(self.lines.len(), 0, "This Sysline has no Line");
        (*self.lines[0]).fileoffset_begin()
    }

    /// the byte offset into the file where this `Sysline` ends, inclusive (not one past ending)
    pub fn fileoffset_end(self: &Sysline) -> FileOffset {
        assert_ne!(self.lines.len(), 0, "This Sysline has no Line");
        let last_ = self.lines.len() - 1;
        (*self.lines[last_]).fileoffset_end()
    }

    /// the fileoffset into the next sysline
    /// this Sysline does not know if that fileoffset points to the end of file (one past last actual byte)
    pub fn fileoffset_next(self: &Sysline) -> FileOffset {
        self.fileoffset_end() + (self.charsz() as FileOffset)
    }

    /// length in bytes of this Sysline
    pub fn len(self: &Sysline) -> usize {
        (self.fileoffset_end() - self.fileoffset_begin() + 1) as usize
    }

    /// count of `Line` in `self.lines`
    pub fn count(self: &Sysline) -> u64 {
        self.lines.len() as u64
    }

    /// sum of `Line.count_bytes`
    pub fn count_bytes(self: &Sysline) -> u64 {
        let mut cb = 0;
        for ln in self.lines.iter() {
            cb += ln.count_bytes();
        }
        cb
    }

    /// a `String` copy of the demarcating datetime string found in the Sysline
    #[allow(non_snake_case)]
    pub fn datetime_String(self: &Sysline) -> String {
        assert_ne!(self.dt_beg, LI_NULL, "dt_beg has not been set");
        assert_ne!(self.dt_end, LI_NULL, "dt_end has not been set");
        assert_lt!(self.dt_beg, self.dt_end, "bad values dt_end {} dt_beg {}", self.dt_end, self.dt_beg);
        let slice_ = self.lines[0].as_bytes();
        assert_lt!(
            self.dt_beg,
            slice_.len(),
            "dt_beg {} past end of slice[{}‥{}]?",
            self.dt_beg,
            self.dt_beg,
            self.dt_end
        );
        assert_le!(
            self.dt_end,
            slice_.len(),
            "dt_end {} past end of slice[{}‥{}]?",
            self.dt_end,
            self.dt_beg,
            self.dt_end
        );
        // TODO: here is a place to use `bstr`
        let buf: &[u8] = &slice_[self.dt_beg..self.dt_end];
        let s_ = match str::from_utf8(buf) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Error in datetime_String() during str::from_utf8 {} buffer {:?}", err, buf);
                ""
            }
        };
        String::from(s_)
    }

    /// return all the slices that make up this `Sysline`
    /// TODO: make an iterable trait of this struct
    pub fn get_slices(self: &Sysline) -> Slices {
        let mut sz: usize = 0;
        for lp in &self.lines {
            sz += lp.get_slices_count();
        }
        let mut slices = Slices::with_capacity(sz);
        for lp in &self.lines {
            slices.extend(lp.get_slices().iter());
        }
        slices
    }

    /// print approach #1, use underlying `Line` to `print`
    /// `raw` true will write directly to stdout from the stored `Block`
    /// `raw` false will write transcode each byte to a character and use pictoral representations
    /// XXX: `raw==false` does not handle multi-byte encodings
    /// TODO: move this into a `Printer` class
    #[cfg(any(debug_assertions,test))]
    pub fn print1(self: &Sysline, raw: bool) {
        for lp in &self.lines {
            (*lp).print(raw);
        }
    }

    // TODO: [2022/03/23] implement an `iter_slices` that does not require creating a new `vec`, just
    //       passes `&bytes` back. Call `iter_slices` from `print`

    /// print approach #2, print by slices
    /// prints raw data from underlying `Block`
    /// testing helper
    /// TODO: move this into a `Printer` class
    #[cfg(any(debug_assertions,test))]
    #[allow(dead_code)]
    fn print2(&self) {
        let slices = self.get_slices();
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();
        for slice in slices.iter() {
            match stdout_lock.write(slice) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: write: StdoutLock.write(slice@{:p} (len {})) error {}", slice, slice.len(), err);
                }
            }
        }
        match stdout_lock.flush() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: write: stdout flushing error {}", err);
            }
        }
    }

    /// helper to `print_color`
    /// caller must acquire stdout.Lock, and call `stdout.flush()`
    /// TODO: move this into a `Printer` class
    fn print_color_slices(stdclr: &mut termcolor::StandardStream, colors: &[Color], values:&[&[u8]]) -> Result<()> {
        assert_eq!(colors.len(), values.len());
        for (color, value) in colors.iter().zip(values.iter())
        {
            if let Err(err) = stdclr.set_color(ColorSpec::new().set_fg(Some(color.clone()))) {
                eprintln!("ERROR: print_color_slices: stdout.set_color({:?}) returned error {}", color, err);
                //continue;
                return Err(err);
            };
            if let Err(err) = stdclr.write(value) {
                eprintln!("ERROR: print_color_slices: stdout.write(…) returned error {}", err);
                //continue;
                return Err(err);
            };
        }
        Ok(())
    }

    /// print with color
    /// prints raw data from underlying `Block` bytes
    /// XXX: does not handle multi-byte strings
    /// TODO: needs a global mutex
    /// TODO: move this into a `Printer` class
    pub fn print_color(&self, color_text: Color, color_datetime: Color) -> Result<()> {
        let slices = self.get_slices();
        //let mut stdout = io::stdout();
        //let mut stdout_lock = stdout.lock();
        let mut choice: termcolor::ColorChoice = termcolor::ColorChoice::Never;
        if atty::is(atty::Stream::Stdout) || cfg!(debug_assertions) {
            choice = termcolor::ColorChoice::Always;
        }
        let mut clrout = termcolor::StandardStream::stdout(choice);
        let mut at: LineIndex = 0;
        let dtb = self.dt_beg;
        let dte = self.dt_end;
        for slice in slices.iter() {
            let len_ = slice.len();
            // datetime entirely in this `slice`
            if chmp!(at <= dtb < dte < (at + len_)) {
                let a = &slice[..(dtb-at)];
                let b = &slice[(dtb-at)..(dte-at)];
                let c = &slice[(dte-at)..];
                match Sysline::print_color_slices(&mut clrout, &[color_text, color_datetime, color_text], &[a, b, c]) {
                    Ok(_) => {},
                    Err(err) => {
                        return Err(err);
                    }
                };
            } // XXX: incomplete datetime crosses into next slice
            else {
                match Sysline::print_color_slices(&mut clrout, &[color_text], &[slice]) {
                    Ok(_) => {},
                    Err(err) => {
                        return Err(err);
                    }
                };
            }
            at += len_;
        }
        let mut ret = Ok(());
        if let Err(err) = clrout.flush() {
            eprintln!("ERROR: print_color: stdout.flush() {}", err);
            ret = Err(err);
        }
        if let Err(err) = clrout.reset() {
            eprintln!("ERROR: print_color: stdout.reset() {}", err);
            return Err(err);
        }
        ret
    }

    /// create `String` from `self.lines`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_char_noraw`)
    /// TODO: this would be more efficient returning `&str`
    ///       https://bes.github.io/blog/rust-strings
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    fn _to_String_raw(self: &Sysline, raw: bool) -> String {
        let mut sz: usize = 0;
        for lp in &self.lines {
            sz += (*lp).len();
        }
        // XXX: intermixing byte lengths and character lengths
        // XXX: does not handle multi-byte
        let mut s_ = String::with_capacity(sz + 1);
        for lp in &self.lines {
            s_ += (*lp)._to_String_raw(raw).as_str();
        }
        s_
    }

    /*
    /// create `str` from `self.lines`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_char_noraw`)
    /// TODO: can this be more efficient? specialized for `str`?
    #[allow(non_snake_case)]
    fn _to_str_raw(self: &Sysline, raw: bool) -> &str {
        return (&self._to_String_raw(raw)).as_str();
    }
     */

    // XXX: rust does not support function overloading which is really surprising and disappointing
    /// `Line` to `String`
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &Sysline) -> String {
        self._to_String_raw(true)
    }

    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_from(self: &Sysline, _from: usize) -> String {
        unimplemented!("yep");
    }

    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_from_to(self: &Sysline, _from: usize, _to: usize) -> String {
        unimplemented!("yep");
    }

    /// `Sysline` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &Sysline) -> String {
        self._to_String_raw(false)
    }

    #[allow(non_snake_case)]
    #[cfg(not(any(debug_assertions,test)))]
    pub fn to_String_noraw(self: &Sysline) -> String {
        panic!("should not call function 'Sysline::to_String_noraw' in release build");
        String::from("")
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DateTime typing, strings, and formatting
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// DateTime typing

/// DateTime formatting pattern, passed to `chrono::datetime_from_str`
pub type DateTimePattern = String;
pub type DateTimePattern_str = str;
/// DateTimePattern for searching a line (not the results)
/// slice index begin, slice index end of entire datetime pattern
/// slice index begin just the datetime, slice index end just the datetime
/// TODO: why not define as a `struct` instead of a tuple?
/// TODO: why not use `String` type for the datetime pattern? I don't recall why I chose `str`.
/// TODO: instead of `LineIndex, LineIndex`, use `(RangeInclusive, Offset)` for the two pairs of LineIndex ranges
///       processing functions would attempt all values within `RangeInclusive` (plus the `Offset`).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub struct DateTime_Parse_Data {
    pub(crate) pattern: DateTimePattern,
    /// does the `pattern` have a year? ("%Y", "%y")
    pub(crate) year: bool,
    /// does the `pattern` have a timezone? ("%z", "%Z", etc.)
    pub(crate) tz: bool,
    /// slice index begin of entire pattern
    pub(crate) sib: LineIndex,
    /// slice index end of entire pattern
    pub(crate) sie: LineIndex,
    /// slice index begin of only datetime portion of pattern
    pub(crate) siba: LineIndex,
    /// slice index end of only datetime portion of pattern
    pub(crate) siea: LineIndex,
}

//type DateTime_Parse_Data = (DateTimePattern, bool, LineIndex, LineIndex, LineIndex, LineIndex);
pub(crate) type DateTime_Parse_Data_str<'a> = (&'a DateTimePattern_str, bool, bool, LineIndex, LineIndex, LineIndex, LineIndex);
//type DateTime_Parse_Datas_ar<'a> = [DateTime_Parse_Data<'a>];
pub type DateTime_Parse_Datas_vec = Vec<DateTime_Parse_Data>;
//type DateTime_Parse_Data_BoxP<'syslinereader> = Box<&'syslinereader DateTime_Parse_Data>;
/// count of datetime format strings used
// TODO: how to do this a bit more efficiently, and not store an entire copy?
type DateTime_Pattern_Counts = HashMap<DateTime_Parse_Data, u64>;
/// return type for `SyslineReader::find_datetime_in_line`
pub type Result_FindDateTime = Result<(DateTime_Parse_Data, DateTimeL)>;
/// return type for `SyslineReader::parse_datetime_in_line`
pub type Result_ParseDateTime = Result<(LineIndex, LineIndex, DateTimeL)>;
pub type Result_ParseDateTimeP = Arc<Result_ParseDateTime>;

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
    OccursInRange,
    /// FAIL
    OccursBeforeRange,
    /// FAIL
    OccursAfterRange,
}

impl Result_Filter_DateTime2 {
    #[inline(always)]
    pub const fn is_pass(&self) -> bool {
        matches!(*self, Result_Filter_DateTime2::OccursInRange)
    }

    #[inline(always)]
    pub const fn is_fail(&self) -> bool {
        matches!(*self, Result_Filter_DateTime2::OccursAfterRange | Result_Filter_DateTime2::OccursBeforeRange)
    }
}


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// built-in Datetime formats
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const DATETIME_PARSE_DATAS_LEN: usize = 104;

/// built-in datetime parsing patterns, these are all known patterns attempted on processed files
/// first string is a chrono strftime pattern
/// https://docs.rs/chrono/latest/chrono/format/strftime/
/// first two numbers are total string slice offsets
/// last two numbers are string slice offsets constrained to *only* the datetime portion
/// offset values are [X, Y) (beginning offset is inclusive, ending offset is exclusive or "one past")
/// i.e. string `"[2000-01-01 00:00:00]"`, the pattern may begin at `"["`, the datetime begins at `"2"`
///      same rule for the endings.
/// TODO: use std::ops::RangeInclusive
pub(crate) const DATETIME_PARSE_DATAS: [DateTime_Parse_Data_str; DATETIME_PARSE_DATAS_LEN] = [
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/samba/log.10.7.190.134` (multi-line)
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2020/03/05 12:17:59.631000,  3] ../source3/smbd/oplock.c:1340(init_oplocks)
    //        init_oplocks: initializing messages.
    //
    ("[%Y/%m/%d %H:%M:%S%.6f,", true, false, 0, 28, 1, 27),
    //
    // similar:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2000/01/01 00:00:04.123456] foo
    //
    ("[%Y/%m/%d %H:%M:%S%.6f]", true, false, 0, 28, 1, 27),
    // ---------------------------------------------------------------------------------------------
    // prescripted datetime+tz
    //
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-01 00:00:05 -0400 foo
    //     2000-01-01 00:00:05-0400 foo
    //
    ("%Y-%m-%d %H:%M:%S %z ", true, true, 0, 26, 0, 25),
    ("%Y-%m-%d %H:%M:%S%z ", true, true, 0, 25, 0, 24),
    ("%Y-%m-%dT%H:%M:%S %z ", true, true, 0, 26, 0, 25),
    ("%Y-%m-%dT%H:%M:%S%z ", true, true, 0, 25, 0, 24),
    //
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-01 00:00:05 ACST foo
    //     2000-01-01 00:00:05ACST foo
    //
    ("%Y-%m-%d %H:%M:%S %Z ", true, true, 0, 25, 0, 24),
    ("%Y-%m-%d %H:%M:%S%Z ", true, true, 0, 24, 0, 23),
    ("%Y-%m-%dT%H:%M:%S %Z ", true, true, 0, 25, 0, 24),
    ("%Y-%m-%dT%H:%M:%S%Z ", true, true, 0, 24, 0, 23),
    //
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-01 00:00:05 -04:00 foo
    //     2000-01-01 00:00:05-04:00 foo
    //
    ("%Y-%m-%d %H:%M:%S %:z ", true, true, 0, 27, 0, 26),
    ("%Y-%m-%d %H:%M:%S%:z ", true, true, 0, 26, 0, 25),
    ("%Y-%m-%dT%H:%M:%S %:z ", true, true, 0, 27, 0, 26),
    ("%Y-%m-%dT%H:%M:%S%:z ", true, true, 0, 26, 0, 25),
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     2000-01-01 00:00:01.234-0500 foo
    //     2000-01-01 00:00:01.234-05:00 foo
    //     2000-01-01 00:00:01.234 ACST foo
    //     2000-00-01T00:00:05.123-00:00 Five
    //
    ("%Y-%m-%d %H:%M:%S%.3f%z ", true, true, 0, 29, 0, 28),
    ("%Y-%m-%d %H:%M:%S%.3f%:z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%d %H:%M:%S%.3f %z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%d %H:%M:%S%.3f %:z ", true, true, 0, 31, 0, 30),
    ("%Y-%m-%d %H:%M:%S%.3f %Z ", true, true, 0, 29, 0, 28),
    ("%Y-%m-%dT%H:%M:%S%.3f%z ", true, true, 0, 29, 0, 28),
    ("%Y-%m-%dT%H:%M:%S%.3f%:z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%dT%H:%M:%S%.3f %z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%dT%H:%M:%S%.3f %:z ", true, true, 0, 31, 0, 30),
    ("%Y-%m-%dT%H:%M:%S%.3f %Z ", true, true, 0, 29, 0, 28),
    //
    //               1         2         3
    //     0123456789012345678901234567890123456789
    //     2000-01-01 00:00:01.234567-0800 foo
    //     2000-01-01 00:00:01.234567-08:00 foo
    //     2000-01-01 00:00:01.234567 ACST foo
    //
    ("%Y-%m-%d %H:%M:%S%.6f%z ", true, true, 0, 32, 0, 31),
    ("%Y-%m-%d %H:%M:%S%.6f %z ", true, true, 0, 33, 0, 32),
    ("%Y-%m-%d %H:%M:%S%.6f%:z ", true, true, 0, 33, 0, 32),
    ("%Y-%m-%d %H:%M:%S%.6f %:z ", true, true, 0, 34, 0, 33),
    ("%Y-%m-%d %H:%M:%S%.6f %Z ", true, true, 0, 32, 0, 31),
    ("%Y-%m-%dT%H:%M:%S%.6f%z ", true, true, 0, 32, 0, 31),
    ("%Y-%m-%dT%H:%M:%S%.6f %z ", true, true, 0, 33, 0, 32),
    ("%Y-%m-%dT%H:%M:%S%.6f%:z ", true, true, 0, 33, 0, 32),
    ("%Y-%m-%dT%H:%M:%S%.6f %:z ", true, true, 0, 34, 0, 33),
    ("%Y-%m-%dT%H:%M:%S%.6f %Z ", true, true, 0, 32, 0, 31),
    //
    //               1         2         3
    //     0123456789012345678901234567890123456789
    //     20000101T000001 -0800 foo
    //     20000101T000001 -08:00 foo
    //     20000101T000001 ACST foo
    //
    ("%Y%m%dT%H%M%S %z ", true, true, 0, 22, 0, 21),
    ("%Y%m%dT%H%M%S %:z ", true, true, 0, 23, 0, 22),
    ("%Y%m%dT%H%M%S %Z ", true, true, 0, 22, 0, 21),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/vmware/hostd-62.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     2019-07-26T10:40:29.682-07:00 info hostd[03210] [Originator@6876 sub=Default] Current working directory: /usr/bin
    //
    ("%Y-%m-%dT%H:%M:%S%.3f%z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%dT%H:%M:%S%.3f%Z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%dT%H:%M:%S%.3f-", true, false, 0, 24, 0, 23),  // XXX: temporary stand-in
    ("%Y-%m-%d %H:%M:%S%.6f-", true, false, 0, 27, 0, 26),  // XXX: temporary stand-in
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/kernel.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     Mar  9 08:10:29 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode
    //
    // TODO: [2021/10/03] no support of inferring the year
    //("%b %e %H:%M:%S ", 0, 25, 0, 25),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/synology/synobackup.log` (has horizontal alignment tabs)
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     info	2017/02/21 21:50:48	SYSTEM:	[Local][Backup Task LocalBackup1] Backup task started.
    //     err	2017/02/23 02:55:58	SYSTEM:	[Local][Backup Task LocalBackup1] Exception occured while backing up data. (Capacity at destination is insufficient.) [Path: /volume1/LocalBackup1.hbk]
    // example escaped:
    //     info␉2017/02/21 21:50:48␉SYSTEM:␉[Local][Backup Task LocalBackup1] Backup task started.
    //     err␉2017/02/23 02:55:58␉SYSTEM:␉[Local][Backup Task LocalBackup1] Exception occured while backing up data. (Capacity at destination is insufficient.) [Path: /volume1/LocalBackup1.hbk]
    //
    // TODO: [2021/10/03] no support of variable offset datetime
    //       this could be done by trying random offsets into something
    //       better is to search for a preceding regexp pattern
    //("\t%Y/%m/%d %H:%M:%S\t", 5, 24, 0, 24),
    // ---------------------------------------------------------------------------------------------
    //
    // iptables warning from kernel, from file `/var/log/messages` on OpenWRT
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     Mar 30 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/xrdp.log`
    // example with offset:
    //
    //               1
    //     01234567890123456789
    //     [20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
    //
    ("[%Y%m%d-%H:%M:%S]", true, false, 0, 19, 1, 18),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/vmware-installer.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2019-05-06 11:24:34,074] Successfully loaded GTK libraries.
    //
    ("[%Y-%m-%d %H:%M:%S,%3f] ", true, false, 0, 26, 1, 24),
    // repeat prior but no trailing space
    ("[%Y-%m-%d %H:%M:%S,%3f]", true, false, 0, 25, 1, 24),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/other/archives/proftpd/xferlog`
    // example with offset:
    //
    //               1         2
    //     0123456789012345678901234
    //     Sat Oct 03 11:26:12 2020 0 192.168.1.12 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c
    //
    ("%a %b %d %H:%M:%S %Y ", true, false, 0, 25, 0, 24),
    // repeat prior but no trailing space
    ("%a %b %d %H:%M:%S %Y", true, false, 0, 24, 0, 24),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/OpenSUSE15/zypper.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2019-05-23 16:53:43 <1> trenker(24689) [zypper] main.cc(main):74 ===== Hi, me zypper 1.14.27
    //
    //("%Y-%m-%d %H:%M:%S ", 0, 20, 0, 19),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     2020-01-01 00:00:01.001 xyz
    //      2020-01-01 00:00:01.001 xyz
    //       2020-01-01 00:00:01.001 xyz
    //        2020-01-01 00:00:01.001 xyz
    //         2020-01-01 00:00:01.001 xyz
    //          2020-01-01 00:00:01.001 xyz
    //           2020-01-01 00:00:01.001 xyz
    //            2020-01-01 00:00:01.001 xyz
    //             2020-01-01 00:00:01.001 xyz
    //              2020-01-01 00:00:01.001 xyz
    //     2020-01-01 00:00:01 xyz
    //      2020-01-01 00:00:01 xyz
    //       2020-01-01 00:00:01 xyz
    //        2020-01-01 00:00:01 xyz
    //         2020-01-01 00:00:01 xyz
    //          2020-01-01 00:00:01 xyz
    //           2020-01-01 00:00:01 xyz
    //            2020-01-01 00:00:01 xyz
    //             2020-01-01 00:00:01 xyz
    //              2020-01-01 00:00:01 xyz
    //     2020-01-01 00:00:01xyz
    //      2020-01-01 00:00:01xyz
    //       2020-01-01 00:00:01xyz
    //        2020-01-01 00:00:01xyz
    //         2020-01-01 00:00:01xyz
    //          2020-01-01 00:00:01xyz
    //           2020-01-01 00:00:01xyz
    //            2020-01-01 00:00:01xyz
    //             2020-01-01 00:00:01xyz
    //              2020-01-01 00:00:01xyz
    //
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 0, 24, 0, 23),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 1, 25, 1, 24),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 2, 26, 2, 25),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 3, 27, 3, 26),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 4, 28, 4, 27),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 5, 29, 5, 28),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 6, 30, 6, 29),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 7, 31, 7, 30),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 8, 32, 8, 31),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 9, 33, 9, 32),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 10, 34, 10, 33),
    ("%Y-%m-%d %H:%M:%S ", true, false, 0, 20, 0, 19),
    ("%Y-%m-%d %H:%M:%S ", true, false, 1, 21, 1, 20),
    ("%Y-%m-%d %H:%M:%S ", true, false, 2, 22, 2, 21),
    ("%Y-%m-%d %H:%M:%S ", true, false, 3, 23, 3, 22),
    ("%Y-%m-%d %H:%M:%S ", true, false, 4, 24, 4, 23),
    ("%Y-%m-%d %H:%M:%S ", true, false, 5, 25, 5, 24),
    ("%Y-%m-%d %H:%M:%S ", true, false, 6, 26, 6, 25),
    ("%Y-%m-%d %H:%M:%S ", true, false, 7, 27, 7, 26),
    ("%Y-%m-%d %H:%M:%S ", true, false, 8, 28, 8, 27),
    ("%Y-%m-%d %H:%M:%S ", true, false, 9, 29, 9, 28),
    ("%Y-%m-%d %H:%M:%S ", true, false, 10, 30, 10, 29),
    ("%Y-%m-%d %H:%M:%S", true, false, 0, 19, 0, 19),
    ("%Y-%m-%d %H:%M:%S", true, false, 1, 20, 1, 20),
    ("%Y-%m-%d %H:%M:%S", true, false, 2, 21, 2, 21),
    ("%Y-%m-%d %H:%M:%S", true, false, 3, 22, 3, 22),
    ("%Y-%m-%d %H:%M:%S", true, false, 4, 23, 4, 23),
    ("%Y-%m-%d %H:%M:%S", true, false, 5, 24, 5, 24),
    ("%Y-%m-%d %H:%M:%S", true, false, 6, 25, 6, 25),
    ("%Y-%m-%d %H:%M:%S", true, false, 7, 26, 7, 26),
    ("%Y-%m-%d %H:%M:%S", true, false, 8, 27, 8, 27),
    ("%Y-%m-%d %H:%M:%S", true, false, 9, 28, 9, 28),
    ("%Y-%m-%d %H:%M:%S", true, false, 10, 29, 10, 29),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2020-01-01T00:00:01 xyz
    //
    ("%Y-%m-%dT%H:%M:%S ", true, false, 0, 20, 0, 19),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2020-01-01T00:00:01xyz
    //
    ("%Y-%m-%dT%H:%M:%S", true, false, 0, 19, 0, 19),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1
    //     012345678901234567
    //     20200101 000001 xyz
    //
    ("%Y%m%d %H%M%S ", true, false, 0, 16, 0, 15),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1
    //     012345678901234567
    //     20200101T000001 xyz
    //
    ("%Y%m%dT%H%M%S ", true, false, 0, 16, 0, 15),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1
    //     012345678901234567
    //     20200101T000001xyz
    //
    ("%Y%m%dT%H%M%S", true, false, 0, 15, 0, 15),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/debian9/apport.log.1`
    // example with offset:
    //
    //               1         2         3         4         5
    //     012345678901234567890123456789012345678901234567890
    //     ERROR: apport (pid 9) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 93) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 935) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 9359) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //
    (" %a %b %d %H:%M:%S %Y:", true, false, 22, 47, 22, 46),
    (" %a %b %d %H:%M:%S %Y:", true, false, 23, 48, 23, 47),
    (" %a %b %d %H:%M:%S %Y:", true, false, 24, 49, 24, 48),
    (" %a %b %d %H:%M:%S %Y:", true, false, 25, 50, 25, 49),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     INFO: Thu Feb 20 00:59:59 2020 info
    //     ERROR: Thu Feb 20 00:59:59 2020 error
    //     DEBUG: Thu Feb 20 00:59:59 2020 debug
    //     VERBOSE: Thu Feb 20 00:59:59 2020 verbose
    //
    (" %a %b %d %H:%M:%S %Y ", true, false, 5, 31, 6, 30),
    (" %a %b %d %H:%M:%S %Y ", true, false, 6, 32, 7, 31),
    (" %a %b %d %H:%M:%S %Y ", true, false, 7, 33, 8, 32),
    (" %a %b %d %H:%M:%S %Y ", true, false, 8, 34, 9, 33),
    // ---------------------------------------------------------------------------------------------
    //
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
    (" %a %b %d %Y %H:%M:%S ", true, false, 5, 31, 6, 30),
    (" %a %b %d %Y %H:%M:%S ", true, false, 6, 32, 7, 31),
    (" %a %b %d %Y %H:%M:%S ", true, false, 7, 33, 8, 32),
    (" %a %b %d %Y %H:%M:%S ", true, false, 8, 34, 9, 33),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     [ERROR] 2000-01-01T00:00:03 foo
    //     [WARN] 2000-01-01T00:00:03 foo
    //     [DEBUG] 2000-01-01T00:00:03 foo
    //     [INFO] 2000-01-01T00:00:03 foo
    //     [VERBOSE] 2000-01-01T00:00:03 foo
    //
    ("] %Y-%m-%dT%H:%M:%S ", true, false, 5, 27, 7, 26),
    ("] %Y-%m-%dT%H:%M:%S ", true, false, 6, 28, 8, 27),
    ("] %Y-%m-%dT%H:%M:%S ", true, false, 7, 29, 9, 28),
    ("] %Y-%m-%dT%H:%M:%S ", true, false, 8, 30, 10, 29),
    ("] %Y-%m-%d %H:%M:%S ", true, false, 5, 27, 7, 26),
    ("] %Y-%m-%d %H:%M:%S ", true, false, 6, 28, 8, 27),
    ("] %Y-%m-%d %H:%M:%S ", true, false, 7, 29, 9, 28),
    ("] %Y-%m-%d %H:%M:%S ", true, false, 8, 30, 10, 29),
    // ---------------------------------------------------------------------------------------------
    // TODO: [2022/03/24] add timestamp formats seen at https://www.unixtimestamp.com/index.php
];

pub(crate)
fn DateTime_Parse_Data_str_to_DateTime_Parse_Data(dtpds: &DateTime_Parse_Data_str) -> DateTime_Parse_Data {
    DateTime_Parse_Data {
        pattern: dtpds.0.to_string(),
        year: dtpds.1,
        tz: dtpds.2,
        sib: dtpds.3,
        sie: dtpds.4,
        siba: dtpds.5,
        siea: dtpds.6,
    }
}

lazy_static! {
    pub(crate) static ref DATETIME_PARSE_DATAS_VEC: DateTime_Parse_Datas_vec =
        DATETIME_PARSE_DATAS.iter().map(
            |&x| DateTime_Parse_Data_str_to_DateTime_Parse_Data(&x)
        ).collect();
}

lazy_static! {
    static ref DATETIME_PARSE_DATAS_VEC_LONGEST: usize =
        DATETIME_PARSE_DATAS.iter().max_by(|x, y| x.0.len().cmp(&y.0.len())).unwrap().0.len();
}

/// does chrono datetime pattern have a timezone
/// see https://docs.rs/chrono/latest/chrono/format/strftime/
#[inline(always)]
#[cfg(test)]
pub fn dt_pattern_has_tz(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%Z") ||
    pattern.contains("%z") ||
    pattern.contains("%:z") ||
    pattern.contains("%#z")
}

/// does chrono datetime pattern have a year
/// see https://docs.rs/chrono/latest/chrono/format/strftime/
#[inline(always)]
#[cfg(test)]
pub fn dt_pattern_has_year(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%Y") ||
    pattern.contains("%y")
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// thread-safe Atomic Reference Counting Pointer to a `Sysline`
pub type SyslineP = Arc<Sysline>;
pub type SyslineP_Opt = Option<Arc<Sysline>>;
/// storage for `Sysline`
pub type Syslines = BTreeMap<FileOffset, SyslineP>;
/// range map where key is sysline begin to end `[ Sysline.fileoffset_begin(), Sysline.fileoffset_end()]`
/// and where value is sysline begin (`Sysline.fileoffset_begin()`). Use the value to lookup associated `Syslines` map
type SyslinesRangeMap = RangeMap<FileOffset, FileOffset>;
/// used internally by `SyslineReader`
type SyslinesLRUCache = LruCache<FileOffset, ResultS4_SyslineFind>;
/// used internally by `SyslineReader`
type LineParsedCache = BTreeMap<FileOffset, Result_ParseDateTimeP>;

/// Specialized Reader that uses `LineReader` to find syslog lines
pub struct SyslineReader<'syslinereader> {
    pub(crate) linereader: LineReader<'syslinereader>,
    /// Syslines by fileoffset_begin
    pub(crate) syslines: Syslines,
    /// count of Syslines processed
    syslines_count: u64,
    // TODO: has `syslines_by_range` ever found a sysline?
    //       would be good to add a test for it.
    /// Syslines fileoffset by sysline fileoffset range, i.e. `[Sysline.fileoffset_begin(), Sysline.fileoffset_end()+1)`
    /// the stored value can be used as a key for `self.syslines`
    syslines_by_range: SyslinesRangeMap,
    /// datetime formatting data, for extracting datetime strings from Lines
    /// TODO: change to Set
    dt_patterns: DateTime_Parse_Datas_vec,
    /// internal use; counts found patterns stored in `dt_patterns`
    /// not used after `analyzed == true`
    dt_patterns_counts: DateTime_Pattern_Counts,
    /// default FixedOffset for found `DateTime` without timezone
    tz_offset: FixedOffset,
    // TODO: is the LRU cache really helping?
    /// internal LRU cache for `find_sysline`. maintained in `SyslineReader::find_sysline`
    _find_sysline_lru_cache: SyslinesLRUCache,
    // internal cache of calls to `SyslineReader::parse_datetime_in_line`. maintained in `SyslineReader::find_sysline`
    _parse_datetime_in_line_lru_cache: LineParsedCache,
    // internal stats for `self._parse_datetime_in_line_lru_cache`
    pub(self) _parse_datetime_in_line_lru_cache_hit: u64,
    // internal stats for `self._parse_datetime_in_line_lru_cache`
    pub(self) _parse_datetime_in_line_lru_cache_miss: u64,
    /// has `self.file_analysis` completed?
    analyzed: bool,
}

// TODO: [2021/09/19]
//       put all filter data into one struct `SyslineFilter`, simpler to pass around

impl fmt::Debug for SyslineReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyslineReader")
            .field("linereader", &self.linereader)
            .field("syslines", &self.syslines)
            .finish()
    }
}

/// quick debug helper
#[allow(non_snake_case, dead_code)]
#[cfg(debug_assertions)]
fn debug_eprint_LRU_cache<K, V>(cache: &LruCache<K, V>)
where
    K: std::fmt::Debug,
    K: std::hash::Hash,
    K: Eq,
    V: std::fmt::Debug,
{
    if !cfg!(debug_assertions) {
        return;
    }
    debug_eprint!("[");
    for (key, val) in cache.iter() {
        debug_eprint!(" Key: {:?}, Value: {:?};", key, val);
    }
    debug_eprint!("]");
}

/// implement SyslineReader things
impl<'syslinereader> SyslineReader<'syslinereader> {
    const DT_PATTERN_MAX_PRE_ANALYSIS: usize = 4;
    const DT_PATTERN_MAX: usize = 1;
    const ANALYSIS_THRESHOLD: u64 = 5;
    const _FIND_SYSLINE_LRU_CACHE_SZ: usize = 4;

    pub fn new(path: &'syslinereader FPath, blocksz: BlockSz, tz_offset: FixedOffset) -> Result<SyslineReader<'syslinereader>> {
        let lr = match LineReader::new(path, blocksz) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("ERROR: LineReader::new({}, {}) failed {}", path, blocksz, err);
                return Err(err);
            }
        };
        Ok(SyslineReader {
            linereader: lr,
            syslines: Syslines::new(),
            syslines_count: 0,
            syslines_by_range: SyslinesRangeMap::new(),
            dt_patterns: DateTime_Parse_Datas_vec::with_capacity(SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS),
            dt_patterns_counts: DateTime_Pattern_Counts::with_capacity(SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS),
            tz_offset,
            _find_sysline_lru_cache: SyslinesLRUCache::new(SyslineReader::_FIND_SYSLINE_LRU_CACHE_SZ),
            _parse_datetime_in_line_lru_cache: LineParsedCache::new(),
            _parse_datetime_in_line_lru_cache_hit: 0,
            _parse_datetime_in_line_lru_cache_miss: 0,
            analyzed: false,
        })
    }

    pub fn blocksz(&self) -> BlockSz {
        self.linereader.blocksz()
    }

    pub fn filesz(&self) -> BlockSz {
        self.linereader.filesz()
    }

    pub fn path(&self) -> &FPath {
        self.linereader.path()
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    pub fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        self.linereader.block_offset_at_file_offset(fileoffset)
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    pub fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        self.linereader.file_offset_at_block_offset(blockoffset)
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        self.linereader
            .file_offset_at_block_offset_index(blockoffset, blockindex)
    }

    /// return block index at given `FileOffset`
    pub fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        self.linereader.block_index_at_file_offset(fileoffset)
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    pub fn file_blocks_count(&self) -> u64 {
        self.linereader.file_blocks_count()
    }

    pub fn blockoffset_last(&self) -> BlockOffset {
        self.linereader.blockoffset_last()
    }

    /// smallest size character
    pub fn charsz(&self) -> usize {
        self.linereader.charsz()
    }

    /// count of `Sysline`s processed
    pub fn count(&self) -> u64 {
        self.syslines_count
    }

    /// Testing helper only
    #[cfg(any(debug_assertions,test))]
    pub fn print(&self, fileoffset: FileOffset, raw: bool) {
        let syslinep: &SyslineP = match self.syslines.get(&fileoffset) {
            Some(val) => val,
            None => {
                eprintln!("ERROR: in print, self.syslines.get({}) returned None", fileoffset);
                return;
            }
        };
        for linep in &(*syslinep).lines {
            (*linep).print(raw);
        }
    }

    /// Testing helper only
    /// print all known `Sysline`s
    #[cfg(any(debug_assertions,test))]
    pub fn print_all(&self, raw: bool) {
        debug_eprintln!("{}print_all(true)", sn());
        for fo in self.syslines.keys() {
            self.print(*fo, raw);
        }
        debug_eprintln!("\n{}print_all(true)", sx());
    }

    /// is given `SyslineP` last in the file?
    pub(crate) fn is_sysline_last(&self, syslinep: &SyslineP) -> bool {
        let filesz = self.filesz();
        let fo_end = (*syslinep).fileoffset_end();
        if (fo_end + 1) == filesz {
            return true;
        }
        assert_lt!(fo_end + 1, filesz, "fileoffset_end() {} is at or after filesz() fileoffset {}", fo_end, filesz);
        false
    }

    /// store passed `Sysline` in `self.syslines`, update other fields
    fn insert_sysline(&mut self, line: Sysline) -> SyslineP {
        let fo_beg: FileOffset = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let slp = SyslineP::new(line);
        debug_eprintln!("{}SyslineReader.insert_sysline: syslines.insert({}, Sysline @{:p})", so(), fo_beg, &*slp);
        self.syslines.insert(fo_beg, slp.clone());
        self.syslines_count += 1;
        // XXX: multi-byte character
        let fo_end1 = fo_end + (self.charsz() as FileOffset);
        debug_eprintln!(
            "{}SyslineReader.insert_sysline: syslines_by_range.insert(({}‥{}], {})",
            so(),
            fo_beg,
            fo_end1,
            fo_beg
        );
        self.syslines_by_range.insert(fo_beg..fo_end1, fo_beg);
        slp
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
    fn u8_to_str(slice_: &[u8]) -> Option<&str> {
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

    pub fn str_datetime(dts: &str, dtpd: &DateTime_Parse_Data, tz_offset: &FixedOffset) -> DateTimeL_Opt {
        str_datetime(dts, dtpd.pattern.as_str(), dtpd.tz, tz_offset)
    }

    /// if datetime found in `Line` returns `Ok` around
    /// indexes into `line` of found datetime string `(start of string, end of string)`
    /// else returns `Err`
    /// TODO: assumes Local TZ
    /// TODO: 2022/03/11 14:30:00
    ///      The concept of static pattern lengths (beg_i, end_i, actual_beg_i, actual_end_i) won't work for
    ///      variable length datetime patterns, i.e. full month names 'July 1, 2020' and 'December 1, 2020'.
    ///      Instead of fixing the current problem of unexpected datetime matches,
    ///      fix the concept problem of passing around fixed-length datetime strings. Then redo this.
    pub fn find_datetime_in_line(
        line: &Line, parse_data: &'syslinereader DateTime_Parse_Datas_vec, fpath: &FPath, charsz: &CharSz, tz_offset: &FixedOffset,
    ) -> Result_FindDateTime {
        debug_eprintln!("{}find_datetime_in_line:(Line@{:p}, {:?}) {:?}", sn(), &line, line.to_String_noraw(), fpath);
        // skip easy case; no possible datetime
        if line.len() < 4 {
            debug_eprintln!("{}find_datetime_in_line: return Err(ErrorKind::InvalidInput);", sx());
            return Result_FindDateTime::Err(Error::new(ErrorKind::InvalidInput, "Line is too short"));
        }

        //let longest: usize = *DATETIME_PARSE_DATAS_VEC_LONGEST;
        //let mut dtsS: String = String::with_capacity(longest * (2 as usize));

        let hack12 = &b"12";
        let mut i = 0;
        // `sie` and `siea` is one past last char; exclusive.
        // `actual` are more confined slice offsets of the datetime,
        // XXX: it might be faster to skip the special formatting and look directly for the datetime stamp.
        //      calls to chrono are long according to the flamegraph.
        //      however, using the demarcating characters ("[", "]") does give better assurance.
        for dtpd in parse_data.iter() {
            i += 1;
            debug_eprintln!("{}find_datetime_in_line: pattern tuple {} ({:?}, {}, {}, {}, {})", so(), i, dtpd.pattern, dtpd.sib, dtpd.sie, dtpd.siba, dtpd.siea);
            debug_assert_lt!(dtpd.sib, dtpd.sie, "Bad values dtpd.sib dtpd.sie");
            debug_assert_ge!(dtpd.siba, dtpd.sib, "Bad values dtpd.siba dtpd.sib");
            debug_assert_le!(dtpd.siea, dtpd.sie, "Bad values dtpd.siea dtpd.sie");
            //debug_eprintln!("{}find_datetime_in_line searching for pattern {} {:?}", so(), i, dtpd.pattern);
            let len_ = (dtpd.sie - dtpd.sib) as LineIndex;
            // XXX: does not support multi-byte string; assumes single-byte
            if line.len() < dtpd.sie {
                debug_eprintln!(
                    "{}find_datetime_in_line: line len {} is too short for pattern {} len {} @({}, {}] {:?}",
                    so(),
                    line.len(),
                    i,
                    len_,
                    dtpd.sib,
                    dtpd.sie,
                    dtpd.pattern,
                );
                continue;
            }
            // take a slice of the `line_as_slice` then convert to `str`
            // this is to force the parsing function `Local.datetime_from_str` to constrain where it
            // searches within the `Line`
            // TODO: to make this a bit more efficient, would be good to do a lookahead. Add a funciton like
            //       `Line.crosses_block(a: LineIndex, b: LineIndex) -> bool`. Then could set capacity of things
            //       ahead of time.
            let slice_: &[u8];
            let mut hack_slice: Bytes;
            match line.get_boxptrs(dtpd.sib, dtpd.sie) {
                enum_BoxPtrs::SinglePtr(box_slice) => {
                    slice_ = *box_slice;
                },
                enum_BoxPtrs::MultiPtr(vec_box_slice) => {
                    // XXX: really inefficient!
                    hack_slice = Bytes::new();
                    for box_ in vec_box_slice {
                        hack_slice.extend_from_slice(*box_);
                    }
                    slice_ = hack_slice.as_slice();
                },
            };
            // hack efficiency improvement, presumes all found years will have a '1' or a '2' in them
            if charsz == &1 && dtpd.year && !slice_contains_X_2(slice_, &hack12) {
            //if charsz == &1 && dtpd.year && !(slice_.contains(&hack12[0]) || slice_.contains(&hack12[1])) {
                debug_eprintln!("{}find_datetime_in_line: skip slice, does not have '1' or '2'", so());
                continue;
            }
            let dts: &str = match SyslineReader::u8_to_str(slice_) {
                Some(val) => val,
                None => { continue; }
            };
            debug_eprintln!(
                "{}find_datetime_in_line: searching for pattern {} {:?} in {:?} (slice [{}‥{}] from Line {:?})",
                so(),
                i,
                dtpd.pattern,
                str_to_String_noraw(dts),
                dtpd.sib,
                dtpd.sie,
                line.to_String_noraw(),
            );
            // TODO: [2021/10/03]
            //       according to flamegraph, this function `Local::datetime_from_str` takes a very large amount of
            //       runtime, around 20% to 25% of entire process runtime. How to improve that?
            //       Could I create my own hardcoded parsing for a few common patterns?
            let dt = match SyslineReader::str_datetime(dts, dtpd, &tz_offset) {
                Some(val) => {
                    debug_eprintln!("{}find_datetime_in_line: str_datetime returned {:?}", so(), val);
                    val
                }
                None => {
                    debug_eprintln!("{}find_datetime_in_line: str_datetime returned None", so());
                    continue;
                }
            }; // end for(pattern, ...)
            debug_eprintln!("{}find_datetime_in_line: return Ok({}, {}, {});", sx(), dtpd.sib, dtpd.sie, &dt);
            return Result_FindDateTime::Ok((dtpd.clone(), dt));
        }

        debug_eprintln!("{}find_datetime_in_line: return Err(ErrorKind::NotFound);", sx());
        Result_FindDateTime::Err(Error::new(ErrorKind::NotFound, "No datetime found!"))
    }

    /// private helper function to update `self.dt_patterns`
    fn dt_patterns_update(&mut self, datetime_parse_data: DateTime_Parse_Data) {
        if self.analyzed {
            return;
        }
        debug_eprintln!("{}dt_patterns_update(SyslineReader@{:p}, {:?})", sn(), self, datetime_parse_data);
        //
        // update `self.dt_patterns_counts`
        //
        if self.dt_patterns_counts.contains_key(&datetime_parse_data) {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns_counts.get_mut({:?}) += 1",
                so(),
                self,
                datetime_parse_data
            );
            let counter = self.dt_patterns_counts.get_mut(&datetime_parse_data).unwrap();
            *counter += 1;
        } else if self.dt_patterns_counts.len() < SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns_counts.insert({:?}, 0)",
                so(),
                self,
                datetime_parse_data
            );
            self.dt_patterns_counts.insert(datetime_parse_data.clone(), 1);
        }
        //
        // update `self.dt_patterns`
        //
        if self.dt_patterns.len() >= SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns already DT_PATTERN_MAX_PRE_ANALYSIS length {:?}",
                sx(),
                self,
                &self.dt_patterns.len()
            );
            return;
        }
        if self.dt_patterns.contains(&datetime_parse_data) {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) found DateTime_Parse_Data; skip self.dt_patterns.push",
                sx(),
                self
            );
            return;
        }
        debug_eprintln!(
            "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns.push({:?})",
            sx(),
            self,
            datetime_parse_data
        );
        self.dt_patterns.push(datetime_parse_data);
    }

    /// analyze syslines gathered
    /// when a threshold of syslines or bytes has been processed, then
    /// 1. narrow down datetime formats used. this greatly reduces resources
    /// used by `SyslineReader::find_datetime_in_line`
    /// 2. TODO: for any prior analyzed syslines using a datetime format that wasn't accepted,
    ///          retry parsing the lines with narrowed set of datetime formats. however, if those reparse attempts fail, keep the prior parse results using the
    ///          "odd man out" format
    /// TODO: will break if DT_PATTERN_MAX > 1
    fn dt_patterns_analysis(&mut self) {
        if self.analyzed || self.count() < SyslineReader::ANALYSIS_THRESHOLD {
            return;
        }
        debug_eprintln!("{}dt_patterns_analysis()", sn());
        if SyslineReader::DT_PATTERN_MAX != 1 {
            unimplemented!("function dt_patterns_analysis unimplemented for DT_PATTERN_MAX > 1; it is {}", SyslineReader::DT_PATTERN_MAX);
        }
        debug_assert_eq!(self.dt_patterns.len(), self.dt_patterns_counts.len(),
            "dt_patterns.len() != dt_patterns_count.len()\nself.dt_patterns ({})     : {:?}\nself.dt_patterns_counts ({}): {:?}", self.dt_patterns.len(), self.dt_patterns, self.dt_patterns_counts.len(), self.dt_patterns_counts);
        // TODO: change pattern tuple `DateTime_Parse_Data` to use Ranges, currently this is
        //       removing valid patterns in different (beg,end) positions
        // ripped from https://stackoverflow.com/a/60134450/471376
        // test https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b8eb53f40fd89461c9dad9c976746cc3
        let max_ = (&self.dt_patterns_counts).iter().fold(
            std::u64::MIN, |a,b| a.max(*(b.1))
        );
        self.dt_patterns_counts.retain(|_, v| *v >= max_);
        self.dt_patterns_counts.shrink_to(SyslineReader::DT_PATTERN_MAX);
        if cfg!(debug_assertions) {
            for (k, v) in self.dt_patterns_counts.iter() {
                debug_eprintln!("{}dt_patterns_analysis: self.dt_patterns_counts[{:?}]={:?}", so(), k, v);
            }
        }
        // XXX: is there a simpler way to get the first element?
        let datetime_parse_data = match self.dt_patterns_counts.iter().next() {
            Some((k, _)) => { k },
            None => {
                eprintln!("ERROR: self.dt_patterns_counts.values().next() returned None, it is len {}", self.dt_patterns_counts.len());
                self.analyzed = true;
                return;
            }
        };
        debug_eprintln!("{}dt_patterns_analysis: chose dt_pattern", so());
        // effectively remove all elements by index, except for `datetime_parse_data`
        // XXX: what is the rust idiomatic way to remove all but a few elements by index?
        let mut patts = DateTime_Parse_Datas_vec::with_capacity(SyslineReader::DT_PATTERN_MAX);
        let mut index_: usize = 0;
        for datetime_parse_data_ in &self.dt_patterns {
            if datetime_parse_data_.eq(datetime_parse_data) {
                break;
            }
            index_ += 1;
        }
        patts.push(self.dt_patterns.swap_remove(index_));
        self.dt_patterns = patts;
        //self.dt_patterns.retain(|v| v == **patt);
        self.dt_patterns.shrink_to(SyslineReader::DT_PATTERN_MAX);
        if cfg!(debug_assertions) {
            for dtpd in self.dt_patterns.iter() {
                debug_eprintln!("{}dt_patterns_analysis: self.dt_pattern {:?}", so(), dtpd);
            }
        }
        self.dt_patterns_counts.clear();
        self.dt_patterns_counts.shrink_to(0);
        self.analyzed = true;
        debug_eprintln!("{}dt_patterns_analysis()", sx());
    }

    /// attempt to parse a DateTime substring in the passed `Line`
    /// wraps call to `self.find_datetime_in_line` according to status of `self.dt_patterns`
    /// if `self.dt_patterns` is `None`, will set `self.dt_patterns`
    fn parse_datetime_in_line(&mut self, line: &Line, charsz: &CharSz) -> Result_ParseDateTime {
        // XXX: would prefer this at the end of this function but borrow error occurs
        if !self.analyzed {
            self.dt_patterns_analysis();
        }
        debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}); {:?}", sn(), self, line.to_String_noraw());
        // if no `dt_patterns` have been found then try the default datetime patterns immediately
        if self.dt_patterns.is_empty() {
            debug_eprintln!("{}parse_datetime_in_line self.dt_patterns is empty", sn());
            // this `SyslineReader` has not determined it's own DateTime formatting data `self.dt_patterns`
            // so pass the built-in `DATETIME_PARSE_DATAS`.
            // Use the extra data returned by `find_datetime_in_line` to set `self.dt_patterns` once.
            // This will only happen once per `SyslineReader` (assuming a valid Syslog file)
            let result = SyslineReader::find_datetime_in_line(line, &DATETIME_PARSE_DATAS_VEC, self.path(), charsz, &self.tz_offset);
            let (datetime_parse_data, dt) = match result {
                Ok(val) => val,
                Err(err) => {
                    debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Err {};", sx(), self, err);
                    return Err(err);
                }
            };
            self.dt_patterns_update(datetime_parse_data.clone());
            debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return OK;", sx(), self);
            return Result_ParseDateTime::Ok((datetime_parse_data.siba, datetime_parse_data.siea, dt));
        }
        debug_eprintln!("{}parse_datetime_in_line self.dt_patterns has {} entries", so(), &self.dt_patterns.len());
        // have already determined DateTime formatting for this file, so
        // no need to try *all* built-in DateTime formats, just try the known good formats `self.dt_patterns`
        let result = SyslineReader::find_datetime_in_line(line, &self.dt_patterns, self.path(), charsz, &self.tz_offset);
        let (datetime_parse_data, dt) = match result {
            Ok(val) => val,
            Err(err) => {
                if self.analyzed {
                    debug_eprintln!(
                        "{}parse_datetime_in_line(SyslineReader@{:p}) return Err {};",
                        sx(),
                        self,
                        err
                    );
                    return Err(err);
                }
                // The known good format failed and this SyslineReader has not yet run `dt_format_analysis`
                // so now try other default formats. This is a resource expensive operation.
                debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Err {}; try again using default DATETIME_PARSE_DATAS_VEC", so(), self, err);
                match SyslineReader::find_datetime_in_line(line, &DATETIME_PARSE_DATAS_VEC, self.path(), charsz, &self.tz_offset) {
                    Ok((datetime_parse_data_, dt_)) => {
                        self.dt_patterns_update(datetime_parse_data_.clone());
                        (datetime_parse_data_, dt_)
                    }
                    Err(err_) => {
                        debug_eprintln!(
                            "{}parse_datetime_in_line(SyslineReader@{:p}) return Err {};",
                            sx(),
                            self,
                            err_
                        );
                        return Err(err_);
                    }
                }
            }
        };
        debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Ok;", sx(), self);

        Result_ParseDateTime::Ok((datetime_parse_data.sib, datetime_parse_data.sie, dt))
    }

    /// helper to `find_sysline`
    /// call `self.parse_datetime_in_line` with help of `self._parse_datetime_in_line_cache`
    fn parse_datetime_in_line_cached(&mut self, lp: &LineP, charsz: &usize) -> Result_ParseDateTimeP {
        match self._parse_datetime_in_line_lru_cache.get(&lp.fileoffset_begin()) {
            Some(val) => {
                self._parse_datetime_in_line_lru_cache_hit +=1;
                return val.clone();
            },
            _ => {
                self._parse_datetime_in_line_lru_cache_miss += 1;
            },
        }
        // TODO: returned cached value
        //       also, what about change of `self.analyzed` state?
        //       could cache that state, *or*, when analyzing do `_parse_datetime_in_line_lru_cache.clear()`
        let result: Result_ParseDateTime = self.parse_datetime_in_line(&*lp, charsz);
        let resultp: Result_ParseDateTimeP = Result_ParseDateTimeP::new(result);
        let resultp2 = resultp.clone();
        match self._parse_datetime_in_line_lru_cache.insert(lp.fileoffset_begin(), resultp) {
            Some(val_prev) => {
                assert!(false, "self._parse_datetime_in_line_lru_cache already had key {:?}, value {:?}", lp.fileoffset_begin(), val_prev);
            },
            _ => {},
        };

        resultp2
    }

    /// Find first sysline at or after `fileoffset`.
    /// return (fileoffset of start of _next_ sysline, found Sysline at or after `fileoffset`)
    /// Similar to `find_line`, `read_block`.
    /// This is the heart of the algorithm to find a sysline within a syslog file quickly.
    /// It's simply a binary search.
    /// It could definitely use some improvements, but for now it gets the job done.
    /// XXX: this function is large and cumbersome. you've been warned.
    /// TODO: separate caching to wrapper `find_sysline_cached`
    pub fn find_sysline(&mut self, fileoffset: FileOffset) -> ResultS4_SyslineFind {
        debug_eprintln!("{}find_sysline(SyslineReader@{:p}, {})", sn(), self, fileoffset);

        // TODO: make these comparison values into consts
        if self.linereader.blockreader.count_bytes() > 0x4000 && self.count() < 3 {
            debug_eprintln!("{}find_sysline(SyslineReader@{:p}); too many bytes analyzed {}, yet too few syslines {}", sn(), self, self.linereader.blockreader.count_bytes(), self.count());
            // TODO: [2022/04/06] need to implement a way to abandon processing a file.
            //return Result_ParseDateTime::Error("");
        }

        { // check if `fileoffset` is already known about

            // check LRU cache
            match self._find_sysline_lru_cache.get(&fileoffset) {
                Some(rlp) => {
                    //self._read_block_cache_lru_hit += 1;
                    debug_eprintln!("{}find_sysline: found LRU cached for fileoffset {}", so(), fileoffset);
                    match rlp {
                        ResultS4_SyslineFind::Found(val) => {
                            debug_eprintln!("{}return ResultS4_SyslineFind::Found(({}, …)) @[{}, {}] from LRU cache", sx(), val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                            return ResultS4_SyslineFind::Found((val.0, val.1.clone()));
                        }
                        ResultS4_SyslineFind::Found_EOF(val) => {
                            debug_eprintln!("{}return ResultS4_SyslineFind::Found_EOF(({}, …)) @[{}, {}] from LRU cache", sx(), val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                            return ResultS4_SyslineFind::Found_EOF((val.0, val.1.clone()));
                        }
                        ResultS4_SyslineFind::Done => {
                            debug_eprintln!("{}return ResultS4_SyslineFind::Done from LRU cache", sx());
                            return ResultS4_SyslineFind::Done;
                        }
                        ResultS4_SyslineFind::Err(err) => {
                            debug_eprintln!("{}Error {}", so(), err);
                            eprintln!("ERROR: unexpected value store in _find_line_lru_cache, fileoffset {} error {}", fileoffset, err);
                        }
                    }
                }
                None => {
                    //self._read_block_cache_lru_miss += 1;
                    debug_eprintln!("{}find_sysline: fileoffset {} not found in LRU cache", so(), fileoffset);
                }
            }

            // TODO: test that retrieving by cache always returns the same ResultS4 enum value as without a cache

            // check if the offset is already in a known range
            match self.syslines_by_range.get_key_value(&fileoffset) {
                Some(range_fo) => {
                    let range = range_fo.0;
                    debug_eprintln!(
                    "{}find_sysline: hit syslines_by_range cache for FileOffset {} (found in range {:?})",
                    so(),
                    fileoffset,
                    range
                );
                    let fo = range_fo.1;
                    let slp = self.syslines[fo].clone();
                    // XXX: multi-byte character encoding
                    let fo_next = (*slp).fileoffset_next() + (self.charsz() as FileOffset);
                    if self.is_sysline_last(&slp) {
                        debug_eprintln!(
                        "{}find_sysline: return ResultS4_SyslineFind::Found_EOF(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                        sx(),
                        fo_next,
                        &*slp,
                        (*slp).fileoffset_begin(),
                        (*slp).fileoffset_end(),
                        (*slp).to_String_noraw()
                    );
                        self._find_sysline_lru_cache
                            .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_next, slp.clone())));
                        return ResultS4_SyslineFind::Found_EOF((fo_next, slp));
                    }
                    self._find_sysline_lru_cache
                        .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
                    debug_eprintln!(
                    "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                    sx(),
                    fo_next,
                    &*slp,
                    (*slp).fileoffset_begin(),
                    (*slp).fileoffset_end(),
                    (*slp).to_String_noraw()
                );
                    return ResultS4_SyslineFind::Found((fo_next, slp));
                }
                None => {
                    debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines_by_range", so(), fileoffset);
                }
            }
            debug_eprintln!("{}find_sysline: searching for first sysline datetime A …", so());

            // check if there is a Sysline already known at this fileoffset
            // XXX: not necessary to check `self.syslines` since `self.syslines_by_range` is checked.
            if self.syslines.contains_key(&fileoffset) {
                debug_assert!(self.syslines_by_range.contains_key(&fileoffset), "self.syslines.contains_key({}) however, self.syslines_by_range.contains_key({}) returned None (syslines_by_range out of synch)", fileoffset, fileoffset);
                debug_eprintln!("{}find_sysline: hit self.syslines for FileOffset {}", so(), fileoffset);
                let slp = self.syslines[&fileoffset].clone();
                // XXX: multi-byte character encoding
                let fo_next = (*slp).fileoffset_end() + (self.charsz() as FileOffset);
                // TODO: determine if `fileoffset` is the last sysline of the file
                //       should add a private helper function for this task `is_sysline_last(FileOffset)` ... something like that
                debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines {:?}",
                sx(),
                fo_next,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).fileoffset_end(),
                (*slp).to_String_noraw()
            );
                self._find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
                return ResultS4_SyslineFind::Found((fo_next, slp));
            } else {
                debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines", so(), fileoffset);
            }
        }

        //
        // find line with datetime A
        //

        let mut fo_a: FileOffset = 0;
        let mut fo1: FileOffset = fileoffset;
        let mut sl = Sysline::new();
        loop {
            debug_eprintln!("{}find_sysline: self.linereader.find_line({})", so(), fo1);
            let result: ResultS4_LineFind = self.linereader.find_line(fo1);
            let eof = result.is_eof();
            let (fo2, lp) = match result {
                ResultS4_LineFind::Found((fo_, lp_)) | ResultS4_LineFind::Found_EOF((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: A FileOffset {} Line @{:p} len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    (fo_, lp_)
                }
                ResultS4_LineFind::Done => {
                    debug_eprintln!("{}find_sysline: LRU cache put({}, Done)", so(), fileoffset);
                    self._find_sysline_lru_cache.put(fileoffset, ResultS4_SyslineFind::Done);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Done; A from LineReader.find_line({})", sx(), fo1);
                    return ResultS4_SyslineFind::Done;
                }
                ResultS4_LineFind::Err(err) => {
                    eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Err({}); A from LineReader.find_line({})", sx(), err, fo1);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            let resultp = self.parse_datetime_in_line_cached(&lp, &self.charsz());
            debug_eprintln!("{}find_sysline: A find_datetime_in_line returned {:?}", so(), resultp);
            match *resultp {
                Err(_) => {}
                Ok((dt_beg, dt_end, dt)) => {
                    // a datetime was found! beginning of a sysline
                    fo_a = fo1;
                    sl.dt_beg = dt_beg;
                    sl.dt_end = dt_end;
                    sl.dt = Some(dt);
                    debug_eprintln!("{}find_sysline: A sl.push({:?})", so(), (*lp).to_String_noraw());
                    sl.push(lp);
                    fo1 = sl.fileoffset_end() + (self.charsz() as FileOffset);
                    // sanity check
                    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}", dt_beg, dt_end);
                    debug_assert_lt!(dt_end, fo1 as usize, "bad dt_end {} fileoffset+charsz {}", dt_end, fo1 as usize);
                    if eof {
                        let slp = SyslineP::new(sl);
                        debug_eprintln!("{}find_sysline: LRU cache put({}, Found_EOF({}, …))", so(), fileoffset, fo1);
                        self._find_sysline_lru_cache
                            .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo1, slp.clone())));
                        debug_eprintln!(
                            "{}find_sysline: return ResultS4_SyslineFind::Found_EOF({}, {:p}) @[{}, {}]; A found here and LineReader.find_line({})",
                            sx(),
                            fo1,
                            &(*slp),
                            (*slp).fileoffset_begin(),
                            (*slp).fileoffset_end(),
                            fo1,
                        );
                        return ResultS4_SyslineFind::Found_EOF((fo1, slp));
                    }
                    break;
                }
            }
            debug_eprintln!("{}find_sysline: A skip push Line {:?}", so(), (*lp).to_String_noraw());
            fo1 = fo2;
        }

        debug_eprintln!(
            "{}find_sysline: found line with datetime A at FileOffset {}, searching for datetime B starting at fileoffset {} …",
            so(),
            fo_a,
            fo1
        );

        //
        // find line with datetime B
        //

        { // check if sysline at `fo1` is already known about
        /*
            // XXX: not necessary to check `self.syslines` since `self.syslines_by_range` is checked.
            // check if there is a Sysline already known at this fileoffset
            if self.syslines.contains_key(&fo1) {
                debug_assert!(self.syslines_by_range.contains_key(&fo1), "self.syslines.contains_key({}) however, self.syslines_by_range.contains_key({}); syslines_by_range out of synch", fo1, fo1);
                debug_eprintln!("{}find_sysline: hit self.syslines for FileOffset {}", so(), fo1);
                let slp = self.syslines[&fo1].clone();
                // XXX: multi-byte character encoding
                let fo_next = (*slp).fileoffset_end() + (self.charsz() as FileOffset);
                // TODO: determine if `fileoffset` is the last sysline of the file
                //       should add a private helper function for this task `is_sysline_last(FileOffset)` ... something like that
                debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines {:?}",
                sx(),
                fo_next,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).fileoffset_end(),
                (*slp).to_String_noraw()
            );
                self._find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
                return ResultS4_SyslineFind::Found((fo_next, slp));
            } else {
                debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines", so(), fileoffset);
            }
            // check if the offset is already in a known range
            match self.syslines_by_range.get_key_value(&fo1) {
                Some(range_fo) => {
                    let range = range_fo.0;
                    debug_eprintln!(
                    "{}find_sysline: hit syslines_by_range cache for FileOffset {} (found in range {:?})",
                    so(),
                    fo1,
                    range
                );
                    let fo = range_fo.1;
                    let slp = self.syslines[fo].clone();
                    // XXX: multi-byte character encoding
                    let fo_next = (*slp).fileoffset_next() + (self.charsz() as FileOffset);
                    if self.is_sysline_last(&slp) {
                        debug_eprintln!(
                            "{}find_sysline: return ResultS4_SyslineFind::Found_EOF(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                            sx(),
                            fo_next,
                            &*slp,
                            (*slp).fileoffset_begin(),
                            (*slp).fileoffset_end(),
                            (*slp).to_String_noraw()
                        );
                        self._find_sysline_lru_cache
                            .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_next, slp.clone())));
                        return ResultS4_SyslineFind::Found_EOF((fo_next, slp));
                    }
                    self._find_sysline_lru_cache
                        .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
                    debug_eprintln!(
                        "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                        sx(),
                        fo_next,
                        &*slp,
                        (*slp).fileoffset_begin(),
                        (*slp).fileoffset_end(),
                        (*slp).to_String_noraw()
                    );
                    return ResultS4_SyslineFind::Found((fo_next, slp));
                }
                None => {
                    debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines_by_range", so(), fileoffset);
                }
            }
            debug_eprintln!("{}find_sysline: searching for first sysline datetime B …", so());
        */
        }

        let mut fo_b: FileOffset = fo1;
        let mut eof = false;
        loop {
            debug_eprintln!("{}find_sysline: self.linereader.find_line({})", so(), fo1);
            let result = self.linereader.find_line(fo1);
            let (fo2, lp) = match result {
                ResultS4_LineFind::Found((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: B got Found(FileOffset {}, Line @{:p}) len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    //assert!(!eof, "ERROR: find_line returned EOF as true yet returned Found()");
                    (fo_, lp_)
                }
                ResultS4_LineFind::Found_EOF((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: B got Found_EOF(FileOffset {} Line @{:p}) len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    eof = true;
                    //assert!(!eof, "ERROR: find_line returned EOF as true yet returned Found_EOF()");
                    (fo_, lp_)
                }
                ResultS4_LineFind::Done => {
                    //debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Done; B", sx());
                    debug_eprintln!("{}find_sysline: break; B", so());
                    eof = true;
                    break;
                }
                ResultS4_LineFind::Err(err) => {
                    eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Err({}); B from LineReader.find_line({})", sx(), err, fo1);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            let resultp = self.parse_datetime_in_line_cached(&lp, &self.charsz());
            debug_eprintln!("{}find_sysline: B find_datetime_in_line returned {:?}", so(), resultp);
            match *resultp {
                Err(_) => {
                    debug_eprintln!(
                        "{}find_sysline: B append found Line to this Sysline sl.push({:?})",
                        so(),
                        (*lp).to_String_noraw()
                    );
                    sl.push(lp);
                }
                Ok(_) => {
                    // a datetime was found! end of this sysline, beginning of a new sysline
                    debug_eprintln!(
                        "{}find_sysline: B found datetime; end of this Sysline. Do not append found Line {:?}",
                        so(),
                        (*lp).to_String_noraw()
                    );
                    fo_b = fo1;
                    break;
                }
            }
            fo1 = fo2;
        }

        debug_eprintln!("{}find_sysline: found line with datetime B at FileOffset {}", so(), fo_b);

        let slp = self.insert_sysline(sl);
        if eof {
            debug_eprintln!("{}find_sysline: LRU cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_b);
            self._find_sysline_lru_cache
                .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_b, slp.clone())));
            debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Found_EOF(({}, SyslineP@{:p}) @[{}, {}] E {:?}",
                sx(),
                fo_b,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).fileoffset_end(),
                (*slp).to_String_noraw()
            );
            return ResultS4_SyslineFind::Found_EOF((fo_b, slp));
        }
        debug_eprintln!("{}find_sysline: LRU cache put({}, Found({}, …))", so(), fileoffset, fo_b);
        self._find_sysline_lru_cache
            .put(fileoffset, ResultS4_SyslineFind::Found((fo_b, slp.clone())));
        debug_eprintln!(
            "{}find_sysline: return ResultS4_SyslineFind::Found(({}, SyslineP@{:p}) @[{}, {}] E {:?}",
            sx(),
            fo_b,
            &*slp,
            (*slp).fileoffset_begin(),
            (*slp).fileoffset_end(),
            (*slp).to_String_noraw()
        );

        ResultS4_SyslineFind::Found((fo_b, slp))
    }

    /// wrapper to call each implementation of `find_sysline_at_datetime_filter`
    pub fn find_sysline_at_datetime_filter(
        &mut self, fileoffset: FileOffset, dt_filter: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        self.find_sysline_at_datetime_filter1(fileoffset, dt_filter)
    }

    /// find first sysline at or after `fileoffset` that is at or after `dt_filter`
    ///
    /// for example, given syslog file with datetimes:
    ///     20010101
    ///     20010102
    ///     20010103
    /// where the newline ending the first line is the ninth byte (fileoffset 9)
    ///
    /// calling
    ///     syslinereader.find_sysline_at_datetime_filter(0, Some(20010102 00:00:00-0000))
    /// will return
    ///     ResultS4::Found(19, SyslineP(data='20010102␊'))
    ///
    /// TODO: add more of these examples
    ///
    /// XXX: this function is large, cumbersome, and messy
    fn find_sysline_at_datetime_filter1(
        &mut self, fileoffset: FileOffset, dt_filter: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        let _fname = "find_sysline_at_datetime_filter1";
        debug_eprintln!("{}{}(SyslineReader@{:p}, {}, {:?})", sn(), _fname, self, fileoffset, dt_filter);
        let filesz = self.filesz();
        let _fo_end: FileOffset = filesz as FileOffset;
        let mut try_fo: FileOffset = fileoffset;
        let mut try_fo_last: FileOffset = try_fo;
        let mut fo_last: FileOffset = fileoffset;
        let mut slp_opt: Option<SyslineP> = None;
        let mut slp_opt_last: Option<SyslineP> = None;
        let mut fo_a: FileOffset = fileoffset; // begin "range cursor" marker
        let mut fo_b: FileOffset = _fo_end; // end "range cursor" marker
        loop {
            // TODO: [2021/09/26]
            //       this could be faster.
            //       currently it narrowing down to a byte offset
            //       but it only needs to narrow down to offsets within range of one sysline
            //       so if `fo_a` and `fo_b` are in same sysline range, then this can return that sysline.
            //       Also, add stats for this function and debug print those stats before exiting.
            //       i.e. count of loops, count of calls to sysline_dt_before_after, etc.
            //       do this before tweaking function so can be compared
            debug_eprintln!("{}{}: loop(…)!", so(), _fname);
            let result = self.find_sysline(try_fo);
            let eof = result.is_eof();
            let done = result.is_done();
            match result {
                ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                    if !eof {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(try_fo: {}) returned ResultS4_SyslineFind::Found({}, …) A",
                            so(),
                            _fname,
                            try_fo,
                            fo
                        );
                    } else {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(try_fo: {}) returned ResultS4_SyslineFind::Found_EOF({}, …) B",
                            so(),
                            _fname,
                            try_fo,
                            fo
                        );
                    }
                    debug_eprintln!(
                        "{}{}: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?} C",
                        so(),
                        _fname,
                        fo,
                        &(*slp),
                        slp.lines.len(),
                        (*slp).len(),
                        (*slp).to_String_noraw(),
                    );
                    // here is the binary search algorithm in action
                    debug_eprintln!(
                        "{}{}: sysline_dt_after_or_before(@{:p} ({:?}), {:?})",
                        so(),
                        _fname,
                        &*slp,
                        (*slp).dt,
                        dt_filter
                    );
                    match SyslineReader::sysline_dt_after_or_before(&slp, dt_filter) {
                        Result_Filter_DateTime1::Pass => {
                            debug_eprintln!(
                                "{}{}: Pass => fo {} fo_last {} try_fo {} try_fo_last {} (fo_end {})",
                                so(),
                                _fname,
                                fo,
                                fo_last,
                                try_fo,
                                try_fo_last,
                                _fo_end
                            );
                            debug_eprintln!(
                                "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); A",
                                sx(),
                                _fname,
                                fo,
                                &*slp
                            );
                            return ResultS4_SyslineFind::Found((fo, slp));
                        } // end Pass
                        Result_Filter_DateTime1::OccursAtOrAfter => {
                            // the Sysline found by `find_sysline(try_fo)` occurs at or after filter `dt_filter`, so search backward
                            // i.e. move end marker `fo_b` backward
                            debug_eprintln!("{}{}: OccursAtOrAfter => fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                            // short-circuit a common case, passed fileoffset is past the `dt_filter`, can immediately return
                            // XXX: does this mean my algorithm sucks?
                            if try_fo == fileoffset {
                                // first loop iteration
                                debug_eprintln!(
                                    "{}{}:                    try_fo {} == {} try_fo_last; early return",
                                    so(),
                                    _fname,
                                    try_fo,
                                    try_fo_last
                                );
                                debug_eprintln!(
                                    "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); B fileoffset {} {:?}",
                                    sx(),
                                    _fname,
                                    fo,
                                    &*slp,
                                    (*slp).fileoffset_begin(),
                                    (*slp).to_String_noraw()
                                );
                                return ResultS4_SyslineFind::Found((fo, slp));
                            }
                            try_fo_last = try_fo;
                            fo_b = std::cmp::min((*slp).fileoffset_begin(), try_fo_last);
                            debug_eprintln!(
                                "{}{}:                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                so(),
                                _fname,
                                fo_a,
                                fo_b,
                                fo_a
                            );
                            assert_le!(fo_a, fo_b, "Unexpected values for fo_a {} fo_b {}, FPath {:?}", fo_a, fo_b, self.path());
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursAtOrAfter
                        Result_Filter_DateTime1::OccursBefore => {
                            // the Sysline found by `find_sysline(try_fo)` occurs before filter `dt_filter`, so search forthward
                            // i.e. move begin marker `fo_a` forthward
                            debug_eprintln!("{}{}: OccursBefore =>    fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                            let slp_foe = (*slp).fileoffset_end();
                            // XXX: [2022/03/25] why was this `assert_le` here? It seems wrong.
                            //assert_le!(slp_foe, fo, "unexpected values (SyslineP@{:p}).fileoffset_end() {}, fileoffset returned by self.find_sysline({}) was {} FPath {:?}", slp, slp_foe, try_fo, fo, self.path());
                            try_fo_last = try_fo;
                            assert_le!(try_fo_last, slp_foe, "Unexpected values try_fo_last {} slp_foe {}, last tried offset (passed to self.find_sysline({})) is beyond returned Sysline@{:p}.fileoffset_end() {}!? FPath {:?}", try_fo_last, slp_foe, try_fo, slp, slp_foe, self.path());
                            debug_eprintln!(
                                "{}{}:                    ∴ fo_a = min(slp_foe {}, fo_b {});",
                                so(),
                                _fname,
                                slp_foe,
                                fo_b
                            );
                            // LAST WORKING HERE [2021/10/06 00:05:00]
                            // LAST WORKING HERE [2022/03/16 01:15:00]
                            // this code passes all tests, but runs strangely. I think the problem is the first found sysline
                            // (that may or may not satisfy the passed filter) is placed into a queue and then printed by the waiting main thread.
                            fo_a = std::cmp::min(slp_foe, fo_b);
                            //fo_a = std::cmp::max(slp_foe, fo_b);
                            //fo_a = slp_foe;
                            //assert_le!(fo_a, fo_b, "Unexpected values for fo_a {} fo_b {}", fo_a, fo_b);
                            debug_eprintln!(
                                "{}{}:                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                so(),
                                _fname,
                                fo_a,
                                fo_b,
                                fo_a
                            );
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursBefore
                    } // end SyslineReader::sysline_dt_after_or_before()
                    debug_eprintln!("{}{}:                    fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                    fo_last = fo;
                    slp_opt_last = slp_opt;
                    slp_opt = Some(slp);
                    // TODO: [2021/09/26]
                    //       I think could do an early check and skip a few loops:
                    //       if `fo_a` and `fo_b` are offsets into the same Sysline
                    //       then that Sysline is the candidate, so return Ok(...)
                    //       unless `fo_a` and `fo_b` are past last Sysline.fileoffset_begin of the file then return Done
                } // end Found | Found_EOF
                ResultS4_SyslineFind::Done => {
                    debug_eprintln!("{}{}: SyslineReader.find_sysline(try_fo: {}) returned Done", so(), _fname, try_fo);
                    debug_eprintln!(
                        "{}{}:                 try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})",
                        so(),
                        _fname,
                        try_fo,
                        try_fo_last,
                        fo_a,
                        fo_b,
                        _fo_end
                    );
                    try_fo_last = try_fo;
                    debug_eprintln!(
                        "{}{}:                 ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                        so(),
                        _fname,
                        fo_a,
                        fo_b,
                        fo_a
                    );
                    try_fo = fo_a + ((fo_b - fo_a) / 2);
                    debug_eprintln!(
                        "{}{}:                 try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})",
                        so(),
                        _fname,
                        try_fo,
                        try_fo_last,
                        fo_a,
                        fo_b,
                        _fo_end
                    );
                } // end Done
                ResultS4_SyslineFind::Err(err) => {
                    debug_eprintln!(
                        "{}{}: SyslineReader.find_sysline(try_fo: {}) returned Err({})",
                        so(),
                        _fname,
                        try_fo,
                        err
                    );
                    eprintln!("ERROR: {}", err);
                    break;
                } // end Err
            } // match result
            debug_eprintln!("{}{}: next loop will try offset {} (fo_end {})", so(), _fname, try_fo, _fo_end);

            // TODO: 2022/03/18 this latter part hints at a check that could be done sooner,
            //       before `try_fo==try_fo_last`, that would result in a bit less loops.
            //       A simpler and faster check is to do
            //           fo_next, slp = find_sysline(fileoffset)
            //           _, slp_next = find_sysline(fo_next)
            //       do this at the top of the loop. Then call `dt_after_or_before` for each
            //       `.dt` among `slp`, `slp_next`.

            // `try_fo == try_fo_last` means binary search loop is deciding on the same fileoffset upon each loop.
            // the searching is exhausted.
            if done && try_fo == try_fo_last {
                // reached a dead-end of searching the same fileoffset `find_sysline(try_fo)` and receiving Done
                // so this function is exhausted too.
                debug_eprintln!("{}{}: Done && try_fo {} == {} try_fo_last; break!", so(), _fname, try_fo, try_fo_last);
                break;
            } else if try_fo != try_fo_last {
                continue;
            }
            debug_eprintln!("{}{}: try_fo {} == {} try_fo_last;", so(), _fname, try_fo, try_fo_last);
            let mut slp = slp_opt.unwrap();
            let fo_beg = slp.fileoffset_begin();
            if self.is_sysline_last(&slp) && fo_beg < try_fo {
                // binary search stopped at fileoffset past start of last Sysline in file
                // so entirely past all acceptable syslines
                debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done; C binary searched ended after beginning of last sysline in the file", sx(), _fname,);
                return ResultS4_SyslineFind::Done;
            }
            // binary search loop is deciding on the same fileoffset upon each loop. That fileoffset must refer to
            // an acceptable sysline. However, if that fileoffset is past `slp.fileoffset_begin` than the threshold
            // change of datetime for the `dt_filter` is the *next* Sysline.
            let fo_next = slp.fileoffset_next();
            // XXX: sanity check
            //debug_assert_eq!(fo_last, fo_next, "fo {} != {} slp.fileoffset_next()", fo_last, fo_next);
            if fo_beg < try_fo {
                debug_eprintln!("{}{}: slp.fileoffset_begin() {} < {} try_fo;", so(), _fname, fo_beg, try_fo);
                let slp_next = match self.find_sysline(fo_next) {
                    ResultS4_SyslineFind::Found_EOF((_, slp_)) => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Found_EOF(…, {:?})",
                            so(),
                            _fname,
                            fo_next,
                            slp_
                        );
                        slp_
                    }
                    ResultS4_SyslineFind::Found((_, slp_)) => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Found(…, {:?})",
                            so(),
                            _fname,
                            fo_next,
                            slp_
                        );
                        slp_
                    }
                    ResultS4_SyslineFind::Done => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) unexpectedly returned Done",
                            so(),
                            _fname,
                            fo_next
                        );
                        break;
                    }
                    ResultS4_SyslineFind::Err(err) => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Err({})",
                            so(),
                            _fname,
                            fo_next,
                            err
                        );
                        eprintln!("ERROR: {}", err);
                        break;
                    }
                };
                debug_eprintln!("{}{}: dt_filter:                   {:?}", so(), _fname, dt_filter);
                debug_eprintln!(
                    "{}{}: slp      : fo_beg {:3}, fo_end {:3} {:?} {:?}",
                    so(),
                    _fname,
                    fo_beg,
                    (*slp).fileoffset_end(),
                    (*slp).dt.unwrap(),
                    (*slp).to_String_noraw()
                );
                debug_eprintln!(
                    "{}{}: slp_next : fo_beg {:3}, fo_end {:3} {:?} {:?}",
                    so(),
                    _fname,
                    (*slp_next).fileoffset_begin(),
                    (*slp_next).fileoffset_end(),
                    (*slp_next).dt.unwrap(),
                    (*slp_next).to_String_noraw()
                );
                let slp_compare = Self::dt_after_or_before(&(*slp).dt.unwrap(), dt_filter);
                let slp_next_compare = Self::dt_after_or_before(&(*slp_next).dt.unwrap(), dt_filter);
                debug_eprintln!("{}{}: match({:?}, {:?})", so(), _fname, slp_compare, slp_next_compare);
                slp = match (slp_compare, slp_next_compare) {
                    (_, Result_Filter_DateTime1::Pass) | (Result_Filter_DateTime1::Pass, _) => {
                        debug_eprintln!("{}{}: unexpected Result_Filter_DateTime1::Pass", so(), _fname);
                        eprintln!("ERROR: unexpected Result_Filter_DateTime1::Pass result");
                        break;
                    }
                    (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursBefore) => {
                        debug_eprintln!("{}{}: choosing slp_next", so(), _fname);
                        slp_next
                    }
                    (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursAtOrAfter) => {
                        debug_eprintln!("{}{}: choosing slp_next", so(), _fname);
                        slp_next
                    }
                    (Result_Filter_DateTime1::OccursAtOrAfter, Result_Filter_DateTime1::OccursAtOrAfter) => {
                        debug_eprintln!("{}{}: choosing slp", so(), _fname);
                        slp
                    }
                    _ => {
                        debug_eprintln!(
                            "{}{}: unhandled (Result_Filter_DateTime1, Result_Filter_DateTime1) tuple",
                            so(),
                            _fname
                        );
                        eprintln!("ERROR: unhandled (Result_Filter_DateTime1, Result_Filter_DateTime1) tuple");
                        break;
                    }
                };
            } else {
                debug_eprintln!(
                    "{}{}: slp.fileoffset_begin() {} >= {} try_fo; use slp",
                    so(),
                    _fname,
                    fo_beg,
                    try_fo
                );
            }
            let fo_ = slp.fileoffset_next();
            debug_eprintln!(
                "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); D fileoffset {} {:?}",
                sx(),
                _fname,
                fo_,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).to_String_noraw()
            );
            return ResultS4_SyslineFind::Found((fo_, slp));
        } // end loop

        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done; E", sx(), _fname);

        ResultS4_SyslineFind::Done
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

    /// convenience wrapper for `dt_after_or_before`
    pub fn sysline_dt_after_or_before(syslinep: &SyslineP, dt_filter: &DateTimeL_Opt) -> Result_Filter_DateTime1 {
        debug_eprintln!("{}sysline_dt_after_or_before(SyslineP@{:p}, {:?})", snx(), &*syslinep, dt_filter,);
        assert!((*syslinep).dt.is_some(), "Sysline@{:p} does not have a datetime set.", &*syslinep);

        let dt = (*syslinep).dt.unwrap();

        Self::dt_after_or_before(&dt, dt_filter)
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
                "{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange; (no dt filters)",
                sx(),
            );
            return Result_Filter_DateTime2::OccursInRange;
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
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursBeforeRange;", sx());
                return Result_Filter_DateTime2::OccursBeforeRange;
            }
            if db < dt {
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursAfterRange;", sx());
                return Result_Filter_DateTime2::OccursAfterRange;
            }
            // assert da < dt && dt < db
            assert_le!(da, dt, "Unexpected range values da dt");
            assert_le!(dt, db, "Unexpected range values dt db");
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange;", sx());
            return Result_Filter_DateTime2::OccursInRange;
        } else if dt_filter_after.is_some() {
            debug_eprintln!(
                "{}dt_pass_filters comparing datetime dt_filter_after {:?} < {:?} dt ???",
                so(),
                &dt_filter_after.unwrap(),
                dt
            );
            let da = &dt_filter_after.unwrap();
            if dt < da {
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursBeforeRange;", sx());
                return Result_Filter_DateTime2::OccursBeforeRange;
            }
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange;", sx());
            return Result_Filter_DateTime2::OccursInRange;
        } else {
            debug_eprintln!(
                "{}dt_pass_filters comparing datetime dt {:?} < {:?} dt_filter_before ???",
                so(),
                dt,
                &dt_filter_before.unwrap()
            );
            let db = &dt_filter_before.unwrap();
            if db < dt {
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursAfterRange;", sx());
                return Result_Filter_DateTime2::OccursAfterRange;
            }
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange;", sx());
            return Result_Filter_DateTime2::OccursInRange;
        }
    }

    /// wrapper for call to `dt_pass_filters`
    pub fn sysline_pass_filters(
        syslinep: &SyslineP, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
    ) -> Result_Filter_DateTime2 {
        debug_eprintln!(
            "{}sysline_pass_filters(SyslineP@{:p}, {:?}, {:?})",
            sn(),
            &*syslinep,
            dt_filter_after,
            dt_filter_before,
        );
        assert!((*syslinep).dt.is_some(), "Sysline @{:p} does not have a datetime set.", &*syslinep);
        let dt = (*syslinep).dt.unwrap();
        let result = SyslineReader::dt_pass_filters(&dt, dt_filter_after, dt_filter_before);
        debug_eprintln!("{}sysline_pass_filters(…) return {:?};", sx(), result);

        result
    }

    /// find the first `Sysline`, starting at `fileoffset`, that is at or after datetime filter
    /// `dt_filter_after` and before datetime filter `dt_filter_before`
    pub fn find_sysline_between_datetime_filters(
        &mut self, fileoffset: FileOffset, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        let _fname = "find_sysline_between_datetime_filters";
        debug_eprintln!("{}{}({}, {:?}, {:?})", sn(), _fname, fileoffset, dt_filter_after, dt_filter_before);

        match self.find_sysline_at_datetime_filter(fileoffset, dt_filter_after) {
            ResultS4_SyslineFind::Found((fo, slp)) => {
                debug_eprintln!(
                "{}{}: find_sysline_at_datetime_filter returned ResultS4_SyslineFind::Found(({}, {:?})); call sysline_pass_filters",
                    so(),
                    _fname,
                    fo,
                    slp,
                );
                match Self::sysline_pass_filters(&slp, dt_filter_after, dt_filter_before) {
                    Result_Filter_DateTime2::OccursInRange => {
                        debug_eprintln!("{}{}: sysline_pass_filters(…) returned OccursInRange;", so(), _fname);
                        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Found(({}, {:?}))", sx(), _fname, fo, slp);
                        return ResultS4_SyslineFind::Found((fo, slp));
                    },
                    Result_Filter_DateTime2::OccursBeforeRange => {
                        debug_eprintln!("{}{}: sysline_pass_filters(…) returned OccursBeforeRange;", so(), _fname);
                        eprintln!("ERROR: sysline_pass_filters(Sysline@{:p}, {:?}, {:?}) returned OccursBeforeRange, however the prior call to find_sysline_at_datetime_filter({}, {:?}) returned Found; this is unexpected.",
                                  slp, dt_filter_after, dt_filter_before,
                                  fileoffset, dt_filter_after
                        );
                        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done (not sure what to do here)", sx(), _fname);
                        return ResultS4_SyslineFind::Done; 
                    },
                    Result_Filter_DateTime2::OccursAfterRange => {
                        debug_eprintln!("{}{}: sysline_pass_filters(…) returned OccursAfterRange;", so(), _fname);
                        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done", sx(), _fname);
                        return ResultS4_SyslineFind::Done;
                    },
                };
            },
            ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                debug_eprintln!("{}{}: return ResultS4_SyslineFind::Found_EOF(({}, {:?}))", sx(), _fname, fo, slp);
                return ResultS4_SyslineFind::Found_EOF((fo, slp));
            },
            ResultS4_SyslineFind::Done => {},
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!(
                    "{}{}: find_sysline_at_datetime_filter({}, dt_after: {:?}) returned Err({})",
                    so(),
                    _fname,
                    fileoffset,
                    dt_filter_after,
                    err,
                );
                eprintln!("ERROR: {}", err);
                debug_eprintln!("{}{}: return ResultS4_SyslineFind::Err({})", sx(), _fname, err);
                return ResultS4_SyslineFind::Err(err);
            },
        };

        debug_eprintln!("{}{} return ResultS4_SyslineFind::Done", sx(), _fname);

        ResultS4_SyslineFind::Done
    }
    
    /// return an up-to-date `Summary` instance for this `SyslineReader`
    pub fn summary(&self) -> Summary {
        let bytes = self.linereader.blockreader.count_bytes();
        let bytes_total = self.linereader.blockreader.filesz as u64;
        let blocks = self.linereader.blockreader.count();
        let blocks_total = self.linereader.blockreader.blockn;
        let blocksz = self.blocksz();
        let lines = self.linereader.count();
        let syslines = self.count();
        let parse_datetime_in_line_lru_cache_hit = self._parse_datetime_in_line_lru_cache_hit;
        let parse_datetime_in_line_lru_cache_miss = self._parse_datetime_in_line_lru_cache_miss;
        let find_line_lru_cache_hit = self.linereader._find_line_lru_cache_hit;
        let find_line_lru_cache_miss = self.linereader._find_line_lru_cache_miss;
        let _read_block_cache_lru_hit = self.linereader.blockreader._read_block_cache_lru_hit;
        let _read_block_cache_lru_miss = self.linereader.blockreader._read_block_cache_lru_miss;
        let _read_blocks_hit = self.linereader.blockreader._read_blocks_hit;
        let _read_blocks_miss = self.linereader.blockreader._read_blocks_miss;

        Summary::new(
            bytes,
            bytes_total,
            blocks,
            blocks_total,
            blocksz,
            lines,
            syslines,
            parse_datetime_in_line_lru_cache_hit,
            parse_datetime_in_line_lru_cache_miss,
            find_line_lru_cache_hit,
            find_line_lru_cache_miss,
            _read_block_cache_lru_hit,
            _read_block_cache_lru_miss,
            _read_blocks_hit,
            _read_blocks_miss,
        )
    }
}

/// convert `&str` to a chrono `Option<DateTime<FixedOffset>>` instance
#[inline(always)]
pub fn str_datetime(dts: &str, pattern: &DateTimePattern_str, patt_has_tz: bool, tz_offset: &FixedOffset) -> DateTimeL_Opt {
    debug_eprintln!("{}str_datetime({:?}, {:?}, {:?}, {:?})", sn(), str_to_String_noraw(dts), pattern, patt_has_tz, tz_offset);
    // BUG: [2022/03/21] chrono Issue #660 https://github.com/chronotope/chrono/issues/660
    //      ignoring surrounding whitespace in the passed `fmt`
    // LAST WORKING HERE 2022/04/07 22:07:34 see scrap experiments in `Projects/rust-tests/test8-tz/`
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
    if patt_has_tz {
        match DateTime::parse_from_str(dts, pattern) {
            Ok(val) => {
                debug_eprintln!(
                    "{}str_datetime: DateTime::parse_from_str({:?}, {:?}) extrapolated DateTime {:?}",
                    so(),
                    str_to_String_noraw(dts),
                    pattern,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `dts`
                //       and `pattern`
                if !SyslineReader::datetime_from_str_workaround_Issue660(dts, pattern) {
                    debug_eprintln!("{}str_datetime: skip match due to chrono Issue #660", sx());
                    return None;
                }
                debug_eprintln!("{}str_datetime return {:?}", sx(), Some(val));
                return Some(val);
            }
            Err(err) => {
                debug_eprintln!("{}str_datetime: DateTime::parse_from_str({:?}, {:?}) failed ParseError {}", sx(), dts, pattern, err);
                return None;
            }
        };
    }

    // no timezone in pattern, first convert to NaiveDateTime
    //let tz_offset = FixedOffset::west(3600 * 8);
    let dt_naive = match NaiveDateTime::parse_from_str(dts, pattern) {
        Ok(val) => {
            debug_eprintln!(
                "{}str_datetime: NaiveDateTime.parse_from_str({:?}, {:?}) extrapolated NaiveDateTime {:?}",
                so(),
                str_to_String_noraw(dts),
                pattern,
                val,
            );
            // HACK: workaround chrono Issue #660 by checking for matching begin, end of `dts`
            //       and `dtpd.pattern`
            if !SyslineReader::datetime_from_str_workaround_Issue660(dts, pattern) {
                debug_eprintln!("{}str_datetime: skip match due to chrono Issue #660", sx());
                return None;
            }
            val
        }
        Err(err) => {
            debug_eprintln!("{}str_datetime: NaiveDateTime.parse_from_str({:?}, {:?}) failed ParseError {}", sx(), dts, pattern, err);
            return None;
        }
    };
    // second convert the NaiveDateTime to FixedOffset
    match tz_offset.from_local_datetime(&dt_naive).earliest() {
        Some(val) => {
            debug_eprintln!(
                "{}str_datetime: tz_offset.from_local_datetime({:?}).earliest() extrapolated NaiveDateTime {:?}",
                so(),
                dt_naive,
                val,
            );
            // HACK: workaround chrono Issue #660 by checking for matching begin, end of `dts`
            //       and `dtpd.pattern`
            if !SyslineReader::datetime_from_str_workaround_Issue660(dts, pattern) {
                debug_eprintln!("{}str_datetime: skip match due to chrono Issue #660, return None", sx());
                return None;
            }
            debug_eprintln!("{}str_datetime return {:?}", sx(), Some(val));
            return Some(val);
        }
        None => {
            debug_eprintln!("{}str_datetime: NaiveDateTime.parse_from_str({:?}, {:?}) returned None, return None", sx(), dts, pattern);
            return None;
        }
    };
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslogWriter
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// XXX: unfinished attempt at `Printer` or `Writer` "class"

type SyslineReaders<'syslogwriter> = Vec<SyslineReader<'syslogwriter>>;

/// Specialized Writer that coordinates writing multiple SyslineReaders
pub struct SyslogWriter<'syslogwriter> {
    syslinereaders: SyslineReaders<'syslogwriter>,
}

impl<'syslogwriter> SyslogWriter<'syslogwriter> {
    pub fn new(syslinereaders: SyslineReaders<'syslogwriter>) -> SyslogWriter<'syslogwriter> {
        assert_gt!(syslinereaders.len(), 0, "Passed zero SyslineReaders");
        SyslogWriter { syslinereaders }
    }

    pub fn push(&mut self, syslinereader: SyslineReader<'syslogwriter>) {
        self.syslinereaders.push(syslinereader);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions - search a slice quickly (loop unroll version)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_6_2(slice_: &[u8; 6], search: &[u8; 2]) -> bool {
    for i in 0..5 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_7_2(slice_: &[u8; 7], search: &[u8; 2]) -> bool {
    for i in 0..6 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_8_2(slice_: &[u8; 8], search: &[u8; 2]) -> bool {
    for i in 0..7 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_9_2(slice_: &[u8; 9], search: &[u8; 2]) -> bool {
    for i in 0..8 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_10_2(slice_: &[u8; 10], search: &[u8; 2]) -> bool {
    for i in 0..9 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_11_2(slice_: &[u8; 11], search: &[u8; 2]) -> bool {
    for i in 0..10 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_12_2(slice_: &[u8; 12], search: &[u8; 2]) -> bool {
    for i in 0..11 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_13_2(slice_: &[u8; 13], search: &[u8; 2]) -> bool {
    for i in 0..12 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_14_2(slice_: &[u8; 14], search: &[u8; 2]) -> bool {
    for i in 0..13 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_15_2(slice_: &[u8; 15], search: &[u8; 2]) -> bool {
    for i in 0..14 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_16_2(slice_: &[u8; 16], search: &[u8; 2]) -> bool {
    for i in 0..15 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_17_2(slice_: &[u8; 17], search: &[u8; 2]) -> bool {
    for i in 0..16 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_18_2(slice_: &[u8; 18], search: &[u8; 2]) -> bool {
    for i in 0..17 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_19_2(slice_: &[u8; 19], search: &[u8; 2]) -> bool {
    for i in 0..18 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_20_2(slice_: &[u8; 20], search: &[u8; 2]) -> bool {
    for i in 0..19 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_21_2(slice_: &[u8; 21], search: &[u8; 2]) -> bool {
    for i in 0..20 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_22_2(slice_: &[u8; 22], search: &[u8; 2]) -> bool {
    for i in 0..21 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_23_2(slice_: &[u8; 23], search: &[u8; 2]) -> bool {
    for i in 0..22 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_24_2(slice_: &[u8; 24], search: &[u8; 2]) -> bool {
    for i in 0..23 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_25_2(slice_: &[u8; 25], search: &[u8; 2]) -> bool {
    for i in 0..24 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_26_2(slice_: &[u8; 26], search: &[u8; 2]) -> bool {
    for i in 0..25 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_27_2(slice_: &[u8; 27], search: &[u8; 2]) -> bool {
    for i in 0..26 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_28_2(slice_: &[u8; 28], search: &[u8; 2]) -> bool {
    for i in 0..27 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_29_2(slice_: &[u8; 29], search: &[u8; 2]) -> bool {
    for i in 0..28 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_30_2(slice_: &[u8; 30], search: &[u8; 2]) -> bool {
    for i in 0..29 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_31_2(slice_: &[u8; 31], search: &[u8; 2]) -> bool {
    for i in 0..30 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_32_2(slice_: &[u8; 32], search: &[u8; 2]) -> bool {
    for i in 0..31 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_33_2(slice_: &[u8; 33], search: &[u8; 2]) -> bool {
    for i in 0..32 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_34_2(slice_: &[u8; 34], search: &[u8; 2]) -> bool {
    for i in 0..33 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_35_2(slice_: &[u8; 35], search: &[u8; 2]) -> bool {
    for i in 0..34 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_36_2(slice_: &[u8; 36], search: &[u8; 2]) -> bool {
    for i in 0..35 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_37_2(slice_: &[u8; 37], search: &[u8; 2]) -> bool {
    for i in 0..36 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_38_2(slice_: &[u8; 38], search: &[u8; 2]) -> bool {
    for i in 0..37 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_39_2(slice_: &[u8; 39], search: &[u8; 2]) -> bool {
    for i in 0..38 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_40_2(slice_: &[u8; 40], search: &[u8; 2]) -> bool {
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
