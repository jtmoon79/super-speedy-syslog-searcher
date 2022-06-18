// Data/sysline.rs
//

pub use crate::common::{
    FPath,
    FileOffset,
    NLu8,
    CharSz,
};

use crate::common::{
    Bytes,
    ResultS4,
};

use crate::Readers::blockreader::{
    BlockSz,
    BlockOffset,
    BlockIndex,
    Slices,
};

use crate::Data::datetime::{
    FixedOffset,
    DateTimeL,
    DateTimeL_Opt,
    DateTime_Parse_Data,
    DateTime_Parse_Datas_vec,
    //DateTimePattern,
    //DateTimePattern_str,
    DATETIME_PARSE_DATAS_VEC,
    str_datetime,
    dt_pass_filters,
    dt_after_or_before,
    Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    slice_contains_X_2,
    u8_to_str,
};

use crate::Readers::linereader::{
    LineIndex,
    LinePart,
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

#[cfg(any(debug_assertions,test))]
use crate::dbgpr::printers::{
    str_to_String_noraw,
};

use crate::printer::printers::{
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

extern crate chain_cmp;
use chain_cmp::chmp;

extern crate debug_print;
use debug_print::{debug_eprint, debug_eprintln};

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

extern crate static_assertions;
use static_assertions::{
    const_assert,
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

    /// the fileoffset into the immediately next sysline.
    ///
    /// the `self` Sysline does not know if the "next" Sysline has been processed or if it even exists.
    /// this Sysline does not know if that fileoffset points to the end of file (one past last actual byte)
    pub fn fileoffset_next(self: &Sysline) -> FileOffset {
        self.fileoffset_end() + (self.charsz() as FileOffset)
    }

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
    pub fn count_lines(self: &Sysline) -> u64 {
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

    /// return all the slices that make up `Line` at `line_num` within this `Sysline`.
    ///
    /// Similar to `get_slices` but for one line.
    ///
    /// `line_num` counting starts at `0`
    ///
    /// TODO: [2022/06/13] for case of one Slice then return `&[u8]` directly, bypass creation of `Vec`
    ///       requires enum to carry both types `Vec<&[u8]>` or `&[u8]`
    ///       or instead of that, just return `&[&[u8]]`
    fn get_slices_at_line(self: &Sysline, line_num: usize) -> Slices {
        assert_lt!(line_num, self.lines.len(), "Requested line_num {:?} (count from zero) but there are only {:?} Lines within this Sysline (counts from one)", line_num, self.lines.len());

        let linep: &LineP = &self.lines[line_num];
        // TODO: prehandle common case `self.lines.len() == 0`
        let count: usize = linep.get_slices_count();
        let mut slices: Slices = Slices::with_capacity(count);
        slices.extend(linep.get_slices().iter());

        slices
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
        for lp in &self.lines {
            count += lp.get_slices_count();
        }
        // TODO: prehandle common case where `count==1`
        let mut slices = Slices::with_capacity(count);
        for lp in &self.lines {
            slices.extend(lp.get_slices().iter());
        }

        slices
    }

    /// print approach #1, use underlying `Line` to `print`
    /// `raw` true will write directly to stdout from the stored `Block`
    /// `raw` false will write transcode each byte to a character and use pictoral representations
    ///
    /// XXX: `raw==false` does not handle multi-byte encodings
    ///
    /// TODO: move this into a `Printer` class
    #[cfg(any(debug_assertions,test))]
    pub fn print_using_lines(self: &Sysline, raw: bool) {
        for linep in &self.lines {
            (*linep).print(raw);
        }
    }

    // TODO: [2022/03/23] implement an `iter_slices` that does not require creating a new `vec`, just
    //       passes `&bytes` back. Call `iter_slices` from `print`

    /// helper to `print_color`
    /// caller must acquire stdout.Lock, and call `stdout.flush()`
    ///
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

    /// Print `Sysline` with color.
    /// Writes raw data from underlying `Block` bytes.
    ///
    /// Passing `Some(x)` for `line_num` will print only that `Line` in the `Sysline`.
    ///
    /// XXX: does not handle multi-byte strings
    ///
    /// TODO: needs a global mutex... I think...
    ///
    /// TODO: move this into a `Printer` class or `Sysline_Printer` class
    ///
    /// TODO: [2022/04] would be interesting to benchmark difference of printing using one
    ///       instance of `termcolor::StandStream` versus many (like done here).
    ///       would one instance improve performance?
    pub fn print_color(
        &self,
        line_num: Option<usize>,
        color_choice_opt: Option<termcolor::ColorChoice>,
        color_text: Color,
        color_datetime: Color
    ) -> Result<()> {
        // print the datetime portion in color?
        let print_date_color: bool;
        let slices: Slices = match line_num {
            Some(line_num_) => {
                if line_num_ == 0 {
                    // if a single `Line` was requested
                    // then colorizing date is only significant for the first
                    // `Line` of the `Syline`
                    print_date_color = true;
                } else {
                    print_date_color = false;
                }

                self.get_slices_at_line(line_num_)
            }
            None => {
                print_date_color = true;

                self.get_slices()
            }
        };
        let color_choice: termcolor::ColorChoice = match color_choice_opt {
            Some(choice_) => choice_,
            None => termcolor::ColorChoice::Auto,
        };
        //let mut stdout = io::stdout();
        //let mut stdout_lock = stdout.lock();
        let mut clrout = termcolor::StandardStream::stdout(color_choice);
        let mut at: LineIndex = 0;
        let dtb = self.dt_beg;
        let dte = self.dt_end;
        //
        for slice in slices.iter() {
            let len_ = slice.len();
            match color_choice {
                // bypass `print_color_slices` for a small speed improvement
                termcolor::ColorChoice::Never => {
                    match clrout.write(slice) {
                        Ok(_) => {},
                        Err(err) => {
                            return Err(err);
                        }
                    }
                },
                // use `print_color_slices` to print the datetime substring and test in differing colors
                _ => {
                    // datetime entirely in this `slice`
                    if print_date_color && chmp!(at <= dtb < dte < (at + len_)) {
                        let a = &slice[..(dtb-at)];
                        let b = &slice[(dtb-at)..(dte-at)];
                        let c = &slice[(dte-at)..];
                        match Sysline::print_color_slices(&mut clrout, &[color_text, color_datetime, color_text], &[a, b, c]) {
                            Ok(_) => {},
                            Err(err) => {
                                return Err(err);
                            }
                        };
                    } else {
                        // TODO: [2022/03] datetime crosses into next slice
                        match Sysline::print_color_slices(&mut clrout, &[color_text], &[slice]) {
                            Ok(_) => {},
                            Err(err) => {
                                return Err(err);
                            }
                        };
                    }
                    at += len_;
                }
            }   
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
    //#[cfg(any(debug_assertions,test))]
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
    //#[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &Sysline) -> String {
        self._to_String_raw(true)
    }

    /// `Sysline` to `String` but using printable chars for non-printable and/or formatting characters
    ///
    /// inefficient; only for debugging
    #[allow(non_snake_case)]
    //#[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &Sysline) -> String {
        self._to_String_raw(false)
    }

    /*
    #[allow(non_snake_case)]
    #[cfg(not(any(debug_assertions,test)))]
    /// XXX: here to prevent compiler error
    ///
    /// TODO: implement this for `--release` build, then put other functions back to debug only
    pub fn to_String_noraw(self: &Sysline) -> String {
        panic!("should not call function 'Sysline::to_String_noraw' in release build");
        String::new()
    }
    */
}

/// thread-safe Atomic Reference Counting Pointer to a `Sysline`
pub type SyslineP = Arc<Sysline>;
pub type SyslineP_Opt = Option<Arc<Sysline>>;
