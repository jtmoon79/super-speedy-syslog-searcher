// Data/sysline.rs
//

pub use crate::common::{
    Count,
    FPath,
    FileOffset,
    NLu8,
    CharSz,
};

use crate::Readers::blockreader::{
    BlockOffset,
    Slices,
};

use crate::Data::datetime::{
    DateTimeL_Opt,
};

use crate::Readers::linereader::{
    LineIndex,
    Line,
    LineP,
    Lines,
};

#[cfg(any(debug_assertions,test))]
use crate::printer_debug::stack::{
    so,
};

use std::fmt;
use std::sync::Arc;

extern crate debug_print;
use debug_print::debug_eprintln;

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
    ///
    /// byte-based count
    ///
    /// datetime is presumed to be on first Line
    pub(crate) dt_beg: LineIndex,
    /// index into `Line` where datetime string ends, one char past last character of datetime string
    ///
    /// byte-based count
    ///
    /// datetime is presumed to be on first Line
    pub(crate) dt_end: LineIndex,
    /// parsed DateTime instance
    pub(crate) dt: DateTimeL_Opt,
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

    pub fn charsz(self: &Sysline) -> usize {
        Sysline::CHARSZ
    }

    pub fn dt(self: &Sysline) -> &DateTimeL_Opt {
        &self.dt
    }

    pub fn push(&mut self, linep: LineP) {
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

    /// the fileoffset into the immediately next sysline.
    ///
    /// the `self` Sysline does not know if the "next" Sysline has been processed or if it even exists.
    /// this Sysline does not know if that fileoffset points to the end of file (one past last actual byte)
    pub fn fileoffset_next(self: &Sysline) -> FileOffset {
        self.fileoffset_end() + (self.charsz() as FileOffset)
    }

    /*
    /// the fileoffset into the immediately previous sysline.
    /// 
    /// the `self` Sysline does not know if the "previous" Sysline has been processed or if it even exists.
    /// if the passed `Sysline` has `fileoffset_begin()` of `0` then `0` will be returned
    pub fn fileoffset_prev(self: &Sysline) -> FileOffset {
        let charsz_ = self.charsz() as FileOffset ;
        match self.fileoffset_begin() {
            0 => 0,
            val if val < charsz_ => 0,
            val => val - charsz_,
        }
    }
    */

    /// return the first `BlockOffset`s on which data for this Sysline resides.
    /// Presumes underlying `Line` and `LinePart` hold data else panic!
    pub fn blockoffset_first(self: &Sysline) -> BlockOffset {
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
        (self.fileoffset_end() - self.fileoffset_begin() + 1) as usize
    }

    /// count of `Line` in `self.lines`
    ///
    /// TODO: return `usize`, let callers change as needed
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

    /// return all the slices that make up this `Sysline`.
    ///
    /// Similar to `get_slices_line` but for all lines.
    ///
    /// TODO: [2022/06/13] for case of one Slice then return `&[u8]` directly, bypass creation of `Vec`
    ///       requires enum to carry both types `Vec<&[u8]>` or `&[u8]`
    ///       or instead of that, just return `&[&[u8]]`
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

    /// `Sysline` to `String`
    ///
    /// inefficient; only for debugging
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &Sysline) -> String {
        self._to_String_raw(true)
    }

    /// `Sysline` to `String` but using printable chars for non-printable and/or formatting characters
    ///
    /// inefficient; only for debugging
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &Sysline) -> String {
        self._to_String_raw(false)
    }
}

/// thread-safe Atomic Reference Counting pointer to a `Sysline`
pub type SyslineP = Arc<Sysline>;
pub type SyslineP_Opt = Option<Arc<Sysline>>;
