// src/Data/sysline.rs
//

pub use crate::common::{
    Bytes,
    Count,
    FPath,
    FileOffset,
    NLu8,
    CharSz,
};

use crate::readers::blockreader::{
    BlockOffset,
    Slices,
};

use crate::data::datetime::{
    DateTimeLOpt,
    Duration,
};

use crate::data::line::{
    LineIndex,
    Line,
    LineP,
    Lines,
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
    dp_err,
    dp_wrn,
    p_err,
    p_wrn,
};

use std::fmt;
use std::sync::Arc;

extern crate more_asserts;
use more_asserts::{
    assert_ge,
    debug_assert_ge,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sysline
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A `Sysline` has information about a "syslog line" that spans one or more `Line`s.
/// A "syslog line" or `Sysline` is one or more `Line`s, where the first line contains a
/// datetime stamp. That datetime stamp is consistent format of other nearby syslog lines.
pub struct Sysline {
    /// the one or more `Line` that make up a Sysline
    pub(crate) lines: Lines,
    /// index into `Line` where datetime string starts
    /// (inclusive)
    ///
    /// byte-based count
    ///
    /// datetime is presumed to be on first Line
    ///
    /// TODO: use `Range_LineIndex`
    pub(crate) dt_beg: LineIndex,
    /// index into `Line` where datetime string ends, one char past last character of datetime string
    /// (exclusive)
    ///
    /// byte-based count
    ///
    /// datetime is presumed to be on first Line
    pub(crate) dt_end: LineIndex,
    /// parsed DateTime instance
    pub(crate) dt: DateTimeLOpt,
}

/// a signifier value for "not set" or "null" - because sometimes Option is a PitA
const LI_NULL: LineIndex = LineIndex::MAX;

impl std::fmt::Debug for Sysline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut li_s = String::new();
        for lp in self.lines.iter() {
            li_s.push_str(&format!(
                "Line @{:p} (fileoffset_beg {}, fileoffset_end {}, len() {}, count_lineparts() {}",
                &*lp,
                (*lp).fileoffset_begin(),
                (*lp).fileoffset_end(),
                (*lp).len(),
                (*lp).count_lineparts()
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

    pub fn new_from_parts(lines: Lines, dt_beg: LineIndex, dt_end: LineIndex, dt: DateTimeLOpt) -> Sysline {
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

    pub fn dt(self: &Sysline) -> &DateTimeLOpt {
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

    /// return duration of difference between the two `DateTimeL` of each `Sysline`
    pub fn dt_difference(&self, otherp: &SyslineP) -> Duration {
        // XXX: would prefer not to make copies, but using refs is not supported
        (*self.dt.as_ref().unwrap()) - (*(*otherp).dt().as_ref().unwrap())
    }

    pub fn push(&mut self, linep: LineP) {
        dpo!("SyslineReader.push(), self.lines.len() is now {}", self.lines.len() + 1);
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

    /// the fileoffset into the immediately next sysline.
    ///
    /// the `self` Sysline does not know if the "next" Sysline has been processed or if it even exists.
    /// this Sysline does not know if that fileoffset points to the end of file (one past last actual byte)
    pub fn fileoffset_next(self: &Sysline) -> FileOffset {
        self.fileoffset_end() + (self.charsz() as FileOffset)
    }

    /// return the first `BlockOffset`s on which data for this Sysline resides.
    /// Presumes underlying `Line` and `LinePart` hold data else panic!
    pub fn blockoffset_first(self: &Sysline) -> BlockOffset {
        debug_assert_ge!(self.lines.len(), 1, "Sysline contains no lines");
        self.lines[0].blockoffset_first()
    }

    /// Return the last `BlockOffset`s on which data for this Sysline resides.
    /// Presumes underlying `Line` and `LinePart` hold data else panic!
    pub fn blockoffset_last(self: &Sysline) -> BlockOffset {
        let line: &Line = &self.lines[self.lines.len() - 1];

        line.blockoffset_last()
    }

    /// length in bytes of this Sysline
    pub fn len(self: &Sysline) -> usize {
        ((self.fileoffset_end() - self.fileoffset_begin()) + 1) as usize
    }

    // TODO: return `usize`, let callers change as needed
    /// count of `Line` in `self.lines`
    ///
    pub fn count_lines(self: &Sysline) -> Count {
        self.lines.len() as Count
    }

    /// sum of all `Line.count_bytes`
    pub fn count_bytes(self: &Sysline) -> Count {
        let mut cb: Count = 0;
        for line in self.lines.iter() {
            cb += line.count_bytes();
        }

        cb
    }

    /// do the bytes of this `Sysline` reside on one `Block`?
    pub fn occupies_one_block(self: &Sysline) -> bool {
        self.blockoffset_first() == self.blockoffset_last()
    }

    // TODO: [2022/06/13] for case of one Slice then return `&[u8]` directly, bypass creation of `Vec`
    //       requires enum to carry both types `Vec<&[u8]>` or `&[u8]`
    //       or instead of that, just return `&[&[u8]]`
    /// return all the slices that make up this `Sysline`.
    ///
    /// Similar to `get_slices_line` but for all lines.
    ///
    /// Only for testing
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

    /// create `String` from `self.lines`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_char_noraw`)
    /// TODO: this would be more efficient returning `&str`
    ///       https://bes.github.io/blog/rust-strings
    /// TODO: remove this
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    fn impl_to_String_raw(self: &Sysline, raw: bool) -> String {
        let mut sz: usize = 0;
        for lp in &self.lines {
            sz += (*lp).len();
        }
        // XXX: intermixing byte lengths and character lengths
        // XXX: does not handle multi-byte
        let mut s_ = String::with_capacity(sz + 1);
        for lp in &self.lines {
            s_ += (*lp).impl_to_String_raw(raw).as_str();
        }
        s_
    }

    /// `Sysline` to `String`
    ///
    /// inefficient; only for debugging
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
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

    /// `Sysline` to `String` but using printable chars for non-printable and/or formatting characters
    ///
    /// inefficient; only for debugging
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &Sysline) -> String {
        self.impl_to_String_raw(false)
    }
}

/// thread-safe Atomic Reference Counting pointer to a `Sysline`
pub type SyslineP = Arc<Sysline>;
#[allow(non_camel_case_types)]
pub type SyslineP_Opt = Option<Arc<Sysline>>;
