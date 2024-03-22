// src/data/sysline.rs

//! Implements a [`Sysline`] struct.
//!
//! [`Sysline`]: crate::data::sysline::Sysline

#[doc(hidden)]
pub use crate::common::{
    Bytes,
    CharSz,
    Count,
    FPath,
    FileOffset,
    NLc,
    NLu8,
};
use crate::readers::blockreader::BlockOffset;
#[cfg(test)]
use crate::readers::blockreader::Slices;
use crate::data::datetime::{
    DateTimeL,
    Duration,
};
use crate::data::line::{Line, LineIndex, LineP, LinePart, Lines};
#[allow(unused_imports)]
use crate::debug::printers::{de_err, de_wrn, e_wrn};

use std::fmt;
use std::sync::Arc;

use ::more_asserts::debug_assert_ge;
#[allow(unused_imports)]
use ::si_trace_print::{defn, defo, defx, defñ, den, deo, dex, deñ};


// -------
// Sysline

/// A `Sysline` has information about a "syslog line" that spans one or more
/// [`Line`]s.
///
/// A "syslog line" or `Sysline` is one or more `Line`s, where the first `Line`
/// contains a datetime string. That datetime string is consistent format of
/// other nearby syslog lines. The datetime string is parsed from some bytes
/// and stored as a formal [`DateTimeL`], field `dt`.
///
/// [`DateTimeL`]: crate::data::datetime::DateTimeL
/// [`Line`]: crate::data::line::Line
pub struct Sysline {
    /// The one or more `Line` that make up a Sysline.
    pub(crate) lines: Lines,
    /// Index into `Line` where datetime string starts (inclusive).
    ///
    /// Byte-based count.
    ///
    /// Datetime is presumed to be on first `Line`.
    // TODO: use `RangeLineIndex`
    pub(crate) dt_beg: LineIndex,
    /// Index into `Line` where datetime string ends, one char past last
    /// character of datetime string (exclusive).
    ///
    /// Byte-based count.
    ///
    /// Datetime is presumed to be on first `Line`.
    pub(crate) dt_end: LineIndex,
    /// Parsed DateTime instance.
    dt: DateTimeL,
}
// TODO: [2023/04] replace `dt_beg` and `dt_end` with
//        `common::DtBegEndPairOpt`

impl std::fmt::Debug for Sysline {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        let mut li_s = String::new();
        for lp in self.lines.iter() {
            li_s.push_str(&format!(
                "Line (fileoffset_beg {}, fileoffset_end {}, len() {}, count_lineparts() {}",
                (*lp).fileoffset_begin(),
                (*lp).fileoffset_end(),
                (*lp).len(),
                (*lp).count_lineparts()
            ));
        }
        f.debug_struct("Sysline")
            .field("fileoffset_begin()", &self.fileoffset_begin())
            .field("fileoffset_end()", &self.fileoffset_end())
            .field("lines.len", &self.lines.len())
            .field("dt_beg", &self.dt_beg)
            .field("dt_end", &self.dt_end)
            .field("dt", &self.dt)
            .field("lines", &li_s)
            .finish()
    }
}

impl Sysline {
    /// Default [`with_capacity`] for a [`Lines`], most often will only need 1
    /// capacity as the found "sysline" will likely be one `Line`.
    ///
    /// [`with_capacity`]: std::vec::Vec#method.with_capacity
    /// [`Lines`]: crate::data::line::Lines
    const SYSLINE_PARTS_WITH_CAPACITY: usize = 1;

    // XXX: Issue #16 only handles UTF-8/ASCII encoding
    const CHARSZ: usize = 1;

    /// Create a `Sysline` from passed arguments.
    pub fn new_no_lines(
        dt_beg: LineIndex,
        dt_end: LineIndex,
        dt: DateTimeL,
    ) -> Sysline {
        Sysline {
            lines: Lines::with_capacity(Sysline::SYSLINE_PARTS_WITH_CAPACITY),
            dt_beg,
            dt_end,
            dt,
        }
    }

    /// Create a `Sysline` from passed arguments.
    pub fn from_parts(
        lines: Lines,
        dt_beg: LineIndex,
        dt_end: LineIndex,
        dt: DateTimeL,
    ) -> Sysline {
        Sysline {
            lines,
            dt_beg,
            dt_end,
            dt,
        }
    }

    pub fn charsz(self: &Sysline) -> usize {
        Sysline::CHARSZ
    }

    /// Return a reference to `self.dt`
    pub fn dt(self: &Sysline) -> &DateTimeL {
        &self.dt
    }

    /*
    /// Some syslog datetime formats do not include the year. Some `Sysline` instances are created
    /// with a dummy year. Later, after special processing, some `Sysline` may have the `dt` updated
    /// to a determined year.
    ///
    /// Not ideal.
    pub fn update_year(&mut self, year: &Year) {
        let dt_new: DateTimeL = datetime_with_year(self.dt.as_ref().unwrap(), year);
        self.dt = Some(dt_new);
    }
    */

    /// Return duration of difference between the two [`DateTimeL`] of each
    /// `Sysline`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    pub fn dt_difference(
        &self,
        otherp: &SyslineP,
    ) -> Duration {
        // XXX: would prefer not to make copies, but using refs is not supported
        self.dt - (*(*otherp).dt())
    }

    /// Append the passed [`LineP`] to `self.lines`.
    ///
    /// [LineP]: crate::data::line::LineP
    pub fn push(
        &mut self,
        linep: LineP,
    ) {
        self.lines.push(linep);
        deñ!("Sysline.push(), self.lines.len() is now {}", self.lines.len());
    }

    /// The byte offset into the file where this `Sysline` begins.
    /// "points" to first character of `Sysline` (and underlying `Line`).
    pub fn fileoffset_begin(self: &Sysline) -> FileOffset {
        assert_ne!(self.lines.len(), 0, "This Sysline has no Line");
        (*self.lines[0]).fileoffset_begin()
    }

    /// The byte offset into the file where this `Sysline` ends, inclusive
    /// (not one past ending).
    pub fn fileoffset_end(self: &Sysline) -> FileOffset {
        assert_ne!(self.lines.len(), 0, "This Sysline has no Line");
        let last_ = self.lines.len() - 1;
        (*self.lines[last_]).fileoffset_end()
    }

    /// The `FileOffset` into the immediately next sysline.
    ///
    /// The `self` `Sysline` does not know if the "next" `Sysline` has been
    /// processed or if it even exists.
    /// This `Sysline` does not know if that `FileOffset` points to
    /// the end of file (one past last actual byte).
    pub fn fileoffset_next(self: &Sysline) -> FileOffset {
        self.fileoffset_end() + (self.charsz() as FileOffset)
    }

    /// Return the first `BlockOffset` on which data for this Sysline resides.
    ///
    /// Presumes underlying `Line` and `LinePart` hold data else panic!
    pub fn blockoffset_first(self: &Sysline) -> BlockOffset {
        debug_assert_ge!(self.lines.len(), 1, "Sysline contains no lines");
        self.lines[0].blockoffset_first()
    }

    /// Return the last `BlockOffset` on which data for this `Sysline` resides.
    ///
    /// Presumes underlying `Line` and `LinePart` hold data else panic!
    pub fn blockoffset_last(self: &Sysline) -> BlockOffset {
        let line: &Line = &self.lines[self.lines.len() - 1];

        line.blockoffset_last()
    }

    /// Length in bytes of this `Sysline`.
    pub fn len(self: &Sysline) -> usize {
        ((self.fileoffset_end() - self.fileoffset_begin()) + 1) as usize
    }

    // TODO: return `usize`, let callers change as needed
    /// Count of [`Line`] in `self.lines`.
    ///
    /// [`Line`]: crate::data::line::Line
    pub fn count_lines(self: &Sysline) -> Count {
        self.lines.len() as Count
    }

    /// Sum of all `Line.count_bytes` in `self.lines`.
    pub fn count_bytes(self: &Sysline) -> Count {
        let mut cb: Count = 0;
        for line in self.lines.iter() {
            cb += line.count_bytes();
        }

        cb
    }

    /// Do the bytes of this `Sysline` reside on one [`Block`]?
    ///
    /// [`Block`]: crate::readers::blockreader::Block
    pub fn occupies_one_block(self: &Sysline) -> bool {
        self.blockoffset_first() == self.blockoffset_last()
    }

    /// Return all the slices that make up this `Sysline`.
    ///
    /// Similar to `get_slices_line` but for all lines.
    ///
    /// Inefficient. Only for testing.
    #[doc(hidden)]
    #[cfg(test)]
    pub fn get_slices(self: &Sysline) -> Slices {
        let mut count: usize = 0;
        for linep in &self.lines {
            count += linep.count_slices() as usize;
        }
        // TODO: [2022/03] prehandle common case where `count==1`
        let mut slices = Slices::with_capacity(count);
        for linep in &self.lines {
            slices.extend(linep.get_slices().iter());
        }

        slices
    }

    /// Get the last byte of this `Sysline`.
    pub(crate) fn last_byte(self: &Sysline) -> Option<u8> {
        // XXX: Issue #16 only handles UTF-8/ASCII encoding
        assert_eq!(self.charsz(), 1, "charsz {} not implemented", self.charsz());
        let len_ = self.lines.len();
        if len_ == 0 {
            return None;
        }
        let linep_last = match self.lines.get(len_ - 1) {
            Some(linep) => linep,
            None => {
                return None;
            }
        };
        let len_ = linep_last.lineparts.len();
        if len_ == 0 {
            return None;
        }
        let linepart_last: &LinePart = match (*linep_last)
            .lineparts
            .get(len_ - 1)
        {
            Some(linepart_) => linepart_,
            None => {
                return None;
            }
        };
        let slice = linepart_last.as_slice();
        let byte_: u8 = slice[slice.len() - 1];

        Some(byte_)
    }

    /// Does this `Sysline` end in a newline character?
    ///
    /// XXX: Calling this on a partially constructed `Sysline` is most
    ///      likely pointless.
    pub fn ends_with_newline(self: &Sysline) -> bool {
        let byte_last = match self.last_byte() {
            Some(byte_) => byte_,
            None => {
                return false;
            }
        };
        match char::try_from(byte_last) {
            Ok(char_) => NLc == char_,
            Err(_err) => false,
        }
    }

    /// Create a `String` from `lines`:
    ///
    /// - `raw` is `true` means use byte characters as-is
    /// - `raw` is `false` means replace formatting characters or non-printable
    ///    characters with pictoral representation (i.e. `byte_to_char_noraw`)
    ///
    // TODO: this would be more efficient returning `&str`
    //       https://bes.github.io/blog/rust-strings
    //
    // TODO: remove this
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    // TODO fix this non_snake_case (use correct snake_case)
    fn impl_to_String_raw(
        self: &Sysline,
        raw: bool,
    ) -> String {
        let mut sz: usize = 0;
        for lp in &self.lines {
            sz += (*lp).len();
        }
        // XXX: intermixing byte lengths and character lengths
        // XXX: Issue #16 only handles UTF-8/ASCII encoding
        let mut s_ = String::with_capacity(sz + 1);
        for lp in &self.lines {
            s_ += (*lp)
                .impl_to_String_raw(raw)
                .as_str();
        }
        s_
    }

    /// `Sysline` to `String`.
    ///
    /// inefficient; only for debugging or testing
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    // TODO fix this non_snake_case (use correct snake_case)
    pub fn to_String(self: &Sysline) -> String {
        // get capacity needed
        let mut sz: usize = 0;
        for linep in &self.lines {
            for linepart in (*linep).lineparts.iter() {
                sz += linepart.len()
            }
        }
        // copy byte by byte
        let mut buf: Bytes = Bytes::with_capacity(sz);
        for linep in &self.lines {
            for linepart in (*linep).lineparts.iter() {
                let ptr: Box<&[u8]> = linepart.block_boxptr();
                assert_ne!((*ptr).len(), 0, "block_boxptr points to zero-sized slice");
                for byte_ in (*ptr).iter() {
                    buf.push(*byte_);
                }
            }
        }

        String::from_utf8(buf).unwrap()
    }

    /// `Sysline` to `String` but using printable chars for non-printable
    /// and/or formatting characters.
    ///
    /// inefficient; only for debugging or testing
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    // TODO fix this non_snake_case (use correct snake_case)
    pub fn to_String_noraw(self: &Sysline) -> String {
        self.impl_to_String_raw(false)
    }
}

/// Thread-safe [Atomic Reference Counting pointer] to a [`Sysline`].
///
/// [Atomic Reference Counting pointer]: std::sync::Arc
pub type SyslineP = Arc<Sysline>;

/// Optional [`Arc`] pointer to a [`Sysline`].
///
/// [`Arc`]: std::sync::Arc
#[allow(non_camel_case_types)]
pub type SyslineP_Opt = Option<Arc<Sysline>>;
