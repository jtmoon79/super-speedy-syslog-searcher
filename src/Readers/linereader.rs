// Readers/linereader.rs
//

pub use crate::common::{
    FPath,
    FileOffset,
    CharSz,
    NLu8,
};

use crate::Readers::blockreader::{
    BlockSz,
    BlockOffset,
    BlockIndex,
    BlockP,
    Slices,
    EndOfFile,
    BlockReader,
};

use crate::common::{
    Bytes,
    ResultS4,
};

#[cfg(any(debug_assertions,test))]
use crate::dbgpr::printers::{
    byte_to_char_noraw,
    buffer_to_String_noraw,
    char_to_char_noraw,
};

use crate::dbgpr::stack::{
    sn,
    so,
    sx,
};

use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::io::{
    Error,
    Result,
    //Seek,
    //SeekFrom,
};
use std::io::prelude::*;
use std::sync::Arc;

extern crate debug_print;
use debug_print::{debug_eprint, debug_eprintln};
#[allow(unused_imports)]
use debug_print::{debug_print, debug_println};

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
    //debug_assert_ge,
    debug_assert_gt,
};

/// A sequence to track a `Line`.
/// A "line" may span multiple `Block`s. One `LinePart` is needed for each `Block`.
pub type LineParts = Vec<LinePart>;
/// A sequence to track one or more `LineP` that make up a `Sysline` 
pub type Lines = Vec<LineP>;
/// An offset into a `Line`
pub type LineIndex = usize;
/// thread-safe Atomic Reference Counting pointer to a `Line`
pub type LineP = Arc<Line>;
/// storage for Lines found from the underlying `BlockReader`
/// FileOffset key is the first byte/offset that begins the `Line`
pub type FoToLine = BTreeMap<FileOffset, LineP>;
pub type FoToFo = BTreeMap<FileOffset, FileOffset>;
/// `LineReader.find_line` searching results
#[allow(non_camel_case_types)]
pub type ResultS4_LineFind = ResultS4<(FileOffset, LineP), Error>;
pub type LinesLRUCache = LruCache<FileOffset, ResultS4_LineFind>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LinePart, Line, and LineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Struct describing a part or all of a line within a `Block`
/// A "line" can span more than one `Block`. This tracks part or all of a line within
/// one `Block`. One `LinePart` to one `Block`.
/// But one or more `LinePart` are necessary to represent an entire "line".
pub struct LinePart {
    /// index into the `blockp`, index at beginning
    pub blocki_beg: BlockIndex,
    /// index into the `blockp`, index at one after ending '\n' (may refer to one past end of `Block`)
    pub blocki_end: BlockIndex,
    /// the `Block` pointer
    pub blockp: BlockP,
    /// the byte offset into the file where this `LinePart` begins
    pub fileoffset: FileOffset,
    /// debug helper, might be good to get rid of this?
    pub blockoffset: BlockOffset,
    /// debug helper, might be good to get rid of this?
    pub blocksz: BlockSz,
    // TODO: add size of *this* block
}

impl fmt::Debug for LinePart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LinePart")
            .field("LinePart @", &format_args!("{:p}", &self))
            .field("blocki_beg", &self.blocki_beg)
            .field("blocki_end", &self.blocki_end)
            .field("len", &self.len())
            .field("blockp @", &format_args!("{:p}", &(*self.blockp)))
            .field("fileoffset", &self.fileoffset)
            .field("blockoffset", &self.blockoffset)
            .finish()
    }
}

impl LinePart {
    // XXX: does not handle multi-byte encodings
    const CHARSZ: usize = 1;

    pub fn new(
        blocki_beg: BlockIndex, blocki_end: BlockIndex, blockp: BlockP, fileoffset: FileOffset,
        blockoffset: BlockOffset, blocksz: BlockSz,
    ) -> LinePart {
        debug_eprintln!(
            "{}LinePart::new(blocki_beg {}, blocki_end {}, Block @{:p}, fileoffset {}, blockoffset {}, blocksz {})",
            so(),
            blocki_beg,
            blocki_end,
            &*blockp,
            fileoffset,
            blockoffset,
            blocksz
        );
        // some sanity checks
        assert_ne!(fileoffset, FileOffset::MAX, "Bad fileoffset MAX");
        assert_ne!(blockoffset, BlockOffset::MAX, "Bad blockoffset MAX");
        let fo1 = BlockReader::file_offset_at_block_offset(blockoffset, blocksz);
        assert_le!(fo1, fileoffset, "Bad FileOffset {}, must ≥ {}", fileoffset, fo1);
        let fo2 = BlockReader::file_offset_at_block_offset(blockoffset + 1, blocksz);
        assert_le!(fileoffset, fo2, "Bad FileOffset {}, must ≤ {}", fileoffset, fo2);
        let bo = BlockReader::block_offset_at_file_offset(fileoffset, blocksz);
        assert_eq!(blockoffset, bo, "Bad BlockOffset {}, expected {}", blockoffset, bo);
        let bi = BlockReader::block_index_at_file_offset(fileoffset, blocksz);
        assert_eq!(
            blocki_beg, bi,
            "blocki_beg {} ≠ {} block_index_at_file_offset({}, {})",
            blocki_beg, bi, fileoffset, blocksz
        );
        assert_ne!(blocki_end, 0, "Bad blocki_end 0, expected > 0");
        assert_lt!(blocki_beg, blocki_end, "blocki_beg {} should be < blocki_end {}", blocki_beg, blocki_end);
        assert_lt!((blocki_beg as BlockSz), blocksz, "blocki_beg {} should be < blocksz {}", blocki_beg, blocksz);
        assert_le!((blocki_end as BlockSz), blocksz, "blocki_end {} should be ≤ blocksz {}", blocki_end, blocksz);
        LinePart {
            blocki_beg,
            blocki_end,
            blockp,
            fileoffset,
            blockoffset,
            blocksz,
        }
    }

    /// length of line starting at index `blocki_beg`
    pub fn len(&self) -> usize {
        (self.blocki_end - self.blocki_beg) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// count of bytes of this `LinePart`
    /// XXX: `count_bytes` and `len` is overlapping and confusing.
    pub fn count_bytes(&self) -> u64 {
        (self.len() * LinePart::CHARSZ) as u64
    }

    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub(self) fn _to_String_raw(self: &LinePart, raw: bool) -> String {
        // XXX: intermixing byte lengths and character lengths
        // XXX: does not handle multi-byte
        let s1: String;
        let slice_ = &(*self.blockp)[self.blocki_beg..self.blocki_end];
        if raw {
            unsafe {
                s1 = String::from_utf8_unchecked(Vec::<u8>::from(slice_));
            }
            return s1;
        }
        s1 = buffer_to_String_noraw(slice_);
        s1
    }

    pub fn contains(self: &LinePart, byte_: &u8) -> bool {
        (*self.blockp).contains(byte_)
    }

    /// `Line` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &LinePart) -> String {
        self._to_String_raw(false)
    }

    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &LinePart) -> String {
        self._to_String_raw(true)
    }

    /// return Box pointer to slice of bytes that make up this `LinePart`
    pub fn block_boxptr(&self) -> Box<&[u8]> {
        let slice_ = &(*self.blockp).as_slice()[self.blocki_beg..self.blocki_end];
        //let slice_ptr: *const &[u8] = **slice_;
        Box::new(slice_)
    }

    /// return Box pointer to slice of bytes in this `LinePart` from `a` to end
    pub fn block_boxptr_a(&self, a: &LineIndex) -> Box<&[u8]> {
        debug_assert_lt!(self.blocki_beg+a, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed a {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, a, self.blocki_beg + a, self.blocki_end);
        let slice1 = &(*self.blockp).as_slice()[(self.blocki_beg+a)..self.blocki_end];
        //let slice2 = &slice1[*a..];
        Box::new(slice1)
    }

    /// return Box pointer to slice of bytes in this `LinePart` from beginning to `b`
    pub fn block_boxptr_b(&self, b: &LineIndex) -> Box<&[u8]> {
        debug_assert_lt!(self.blocki_beg+b, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed b {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, b, self.blocki_beg + b, self.blocki_end);
        let slice1 = &(*self.blockp).as_slice()[..self.blocki_beg+b];
        //let slice2 = &slice1[..*b];
        Box::new(slice1)
    }
    

    /// return Box pointer to slice of bytes in this `LinePart` from `a` to `b`
    pub fn block_boxptr_ab(&self, a: &LineIndex, b: &LineIndex) -> Box<&[u8]> {
        debug_assert_lt!(a, b, "bad LineIndex");
        debug_assert_lt!(self.blocki_beg+a, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed a {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, a, self.blocki_beg + a, self.blocki_end);
        debug_assert_lt!(self.blocki_beg+b, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed b {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, b, self.blocki_beg + b, self.blocki_end);
        debug_assert_lt!(b - a, self.len(), "Passed LineIndex {}..{} (diff {}) are larger than this LinePart 'slice' {}", a, b, b - a, self.len());
        let slice1 = &(*self.blockp).as_slice()[(self.blocki_beg+a)..(self.blocki_beg+b)];
        //let slice2 = &slice1[*a..*b];
        Box::new(slice1)
    }
}

/// A `Line` has information about a "line" that may or may not span more than one `Block`
pub struct Line {
    lineparts: LineParts,
}

impl fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut li_s = String::new();
        for li in self.lineparts.iter() {
            li_s.push_str(&format!(
                " @{:p} (blocki_beg {}, blocki_end {}, len() {}, BlockP.len() {}, fileoffset {}, blockoffset {})",
                &li,
                &li.blocki_beg,
                &li.blocki_end,
                &li.len(),
                &li.blockp.len(),
                &li.fileoffset,
                &li.blockoffset
            ));
        }
        let mut fo_b = 0;
        if !self.lineparts.is_empty() {
            fo_b = self.lineparts[0].fileoffset;
        }
        let mut fo_e = 0;
        if !self.lineparts.is_empty() {
            let last_li = self.lineparts.len() - 1;
            fo_e = self.lineparts[last_li].fileoffset + (self.lineparts[last_li].len() as FileOffset) - 1;
        }
        f.debug_struct("Line")
            .field("line.fileoffset_begin()", &fo_b)
            .field("line.fileoffset_end()", &fo_e)
            .field("lineparts @", &format_args!("{:p}", &self))
            .field("lineparts.len", &self.lineparts.len())
            .field("lineparts", &li_s)
            .finish()
    }
}

/// return value for `Line::get_boxptrs`
pub enum enum_BoxPtrs <'a> {
    SinglePtr(Box<&'a [u8]>),
    MultiPtr(Vec<Box<&'a [u8]>>),
}

impl Default for Line {
    fn default() -> Self {
        Self {
            lineparts: LineParts::with_capacity(Line::LINE_PARTS_WITH_CAPACITY),
        }
    }
}

impl Line {
    /// default `with_capacity` for a `LineParts`, most often will only need 1 capacity
    /// as the found "line" will likely reside within one `Block`
    const LINE_PARTS_WITH_CAPACITY: usize = 1;

    pub fn new() -> Line {
        Line::default()
    }

    pub fn new_from_linepart(linepart: LinePart) -> Line {
        let mut v = LineParts::with_capacity(Line::LINE_PARTS_WITH_CAPACITY);
        v.push(linepart);
        Line {
            lineparts: v,
        }
    }

    //pub fn charsz(self: &Line) {
    //    self.lineparts.first().unwrap().
    //}

    pub fn push(&mut self, linepart: LinePart) {
        let l_ = self.lineparts.len();
        if l_ > 0 {
            // sanity checks; each `LinePart` should be stored in same order as it appears in the file
            // only need to compare to last `LinePart`
            let li = &self.lineparts[l_ - 1];
            assert_le!(
                li.blockoffset,
                linepart.blockoffset,
                "Prior stored LinePart at blockoffset {} is after passed LinePart at blockoffset {}",
                li.blockoffset,
                linepart.blockoffset,
            );
            assert_lt!(
                li.fileoffset,
                linepart.fileoffset,
                "Prior stored LinePart at fileoffset {} is at or after passed LinePart at fileoffset {}",
                li.fileoffset,
                linepart.fileoffset,
            );
        }
        // TODO: add sanity checks of all prior `linepart` that all `blocki_end` match `*blockp.len()`
        self.lineparts.push(linepart);
    }

    /// the byte offset into the file where this `Line` begins
    /// "points" to first character of `Line`
    pub fn fileoffset_begin(self: &Line) -> FileOffset {
        debug_assert_ne!(self.lineparts.len(), 0, "This Line has no `LinePart`");
        self.lineparts[0].fileoffset
    }

    /// the byte offset into the file where this `Line` ends, inclusive (not one past ending)
    pub fn fileoffset_end(self: &Line) -> FileOffset {
        debug_assert_ne!(self.lineparts.len(), 0, "This Line has no `LinePart`");
        let last_li = self.lineparts.len() - 1;
        self.lineparts[last_li].fileoffset + (self.lineparts[last_li].len() as FileOffset) - 1
    }

    /// XXX: is this correct?
    pub fn len(self: &Line) -> usize {
        (self.fileoffset_end() - self.fileoffset_begin() + 1) as usize
    }

    /// count of `LinePart` in `self.lineparts.len()`
    pub fn count(self: &Line) -> usize {
        self.lineparts.len()
    }

    /// sum of `LinePart.count_bytes`
    pub fn count_bytes(self: &Line) -> u64 {
        let mut cb: u64 = 0;
        for lp in self.lineparts.iter() {
            cb += lp.count_bytes();
        }
        cb
    }

    pub fn get_linepart(self: &Line, mut a: LineIndex) -> &LinePart {
        for linepart in self.lineparts.iter() {
            let len_ = linepart.len();
            if a < len_ {
                return linepart;
            }
            a -= len_;
        }
        // XXX: not sure if this is the best choice
        self.lineparts.last().unwrap()
    }

    /// does the `Line` contain the byte value?
    pub fn contains(self: &Line, byte_: &u8) -> bool {
        for linepart in self.lineparts.iter() {
            if linepart.contains(byte_) {
                return true;
            }
        }
        false
    }

    /// does the `Line` contain the byte value?
    pub fn contains_at(self: &Line, byte_: &u8, a: &LineIndex, b: &LineIndex) -> bool {
        debug_assert_le!(a, b, "passed bad LineIndex pair");
        for linepart in self.lineparts.iter() {
            if linepart.contains(byte_) {
                return true;
            }
        }
        false
    }

    /// return all slices that make up this `Line`
    /// CANDIDATE FOR REMOVAL?
    pub fn get_slices(self: &Line) -> Slices {
        // short-circuit this case
        let sz = self.lineparts.len();
        let mut slices = Slices::with_capacity(sz);
        for linepart in self.lineparts.iter() {
            let slice = &linepart.blockp[linepart.blocki_beg..linepart.blocki_end];
            slices.push(slice);
        }

        slices
    }

    /// return a count of slices that would be returned by `get_slices`
    /// CANDIDATE FOR REMOVAL?
    pub fn get_slices_count(self: &Line) -> usize {
        self.lineparts.len()
    }


    /// get Box pointers to the underlying `&[u8]` slice that makes up this `Line`.
    /// There may be more than one slice as the `Line` may cross block boundaries. So
    /// return the sequence of Box pointers in a `Vec`.
    /// TODO: the `Vec<Box<&[u8]>>` creation is expensive
    ///       consider allowing a mut &Vec to be passed in. However, this will require declaring lifetimes!
    ///       LAST WORKING HERE 2022/04/03 23:54:00
    // TODO: due to unstable feature `Sized` in `Box`, cannot do
    //           fn get_boxptrs(...) -> either::Either<Box<&[u8]>, Vec<Box<&[u8]>>>
    //       causes error `experimental Sized`
    pub fn get_boxptrs(self: &Line, mut a: LineIndex, mut b: LineIndex) -> enum_BoxPtrs<'_> {
        debug_assert_le!(a, b, "passed bad LineIndex pair");
        // do the simple case first (single `Box` pointer required)
        // doing this here, as opposed to intermixing with multiple case, avoids compiler complaint of "use of possibly-uninitialized `ptrs`"
        let mut a1: LineIndex = a;
        let mut b1: LineIndex = b;
        for linepart_ in &self.lineparts {
            let len_ = linepart_.len();
            if a1 < len_ && b1 < len_ {
                return enum_BoxPtrs::SinglePtr(linepart_.block_boxptr_ab(&a1, &b1));
            } else if a1 < len_ && len_ <= b1 {
                break;
            }
            a1 -= len_;
            b1 -= len_;
        }
        // do the harder case (multiple `Box` pointers required)
        let mut a_found = false;
        let mut b_search = false;
        let mut ptrs: Vec<Box<&[u8]>> = Vec::<Box::<&[u8]>>::new();
        for linepart_ in &self.lineparts {
            debug_eprintln!("{}get_boxptrs: linepart {:?}", so(), linepart_.to_String_noraw());
            let len_ = linepart_.len();
            if !a_found && a < len_ {
                a_found = true;
                b_search = true;
                if b < len_ {
                    debug_eprintln!("{}get_boxptrs: ptrs.push(linepart_.block_boxptr_ab({}, {}))", so(), a, b);
                    ptrs.push(linepart_.block_boxptr_ab(&a, &b));  // store [a..b]  (entire slice, entire `Line`)
                    debug_assert_gt!(ptrs.len(), 1, "ptrs is {} elements, expected >= 1; this should have been handled earlier", ptrs.len());
                    return enum_BoxPtrs::MultiPtr(ptrs);
                }
                debug_eprintln!("{}get_boxptrs: ptrs.push(linepart_.block_boxptr_a({}))", so(), a);
                ptrs.push(linepart_.block_boxptr_a(&a));  // store [a..]  (first slice of `Line`)
                b -= len_;
                continue;
            } else if !a_found {
                a -= len_;
                continue;
            }
            if b_search && b < len_ {
                debug_eprintln!("{}get_boxptrs: ptrs.push(linepart_.block_boxptr_b({}))", so(), b);
                ptrs.push(linepart_.block_boxptr_b(&b));  // store [..b] (last slice of `Line`)
                break;
            } else  {
                debug_eprintln!("{}get_boxptrs: ptrs.push(linepart_.block_boxptr())", so());
                ptrs.push(linepart_.block_boxptr());  // store [..] (entire slice, middle part of `Line`)
                b -= len_;
            }
        }
        enum_BoxPtrs::MultiPtr(ptrs)
    }

    /// `raw` true will write directly to stdout from the stored `Block`
    /// `raw` false will write transcode each byte to a character and use pictoral representations
    /// XXX: `raw==false` does not handle multi-byte encodings
    #[cfg(any(debug_assertions,test))]
    pub fn print(self: &Line, raw: bool) {
        // is this an expensive command? should `stdout` be cached?
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();
        for linepart in &self.lineparts {
            // TODO: I'm somewhat sure this is not creating anything new but I should verify with `gdb-rust`.
            let slice = &linepart.blockp[linepart.blocki_beg..linepart.blocki_end];
            if raw {
                match stdout_lock.write(slice) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!(
                            "ERROR: StdoutLock.write(@{:p}[{}‥{}]) error {}",
                            &*linepart.blockp, linepart.blocki_beg, linepart.blocki_end, err
                        );
                    }
                }
            } else {
                // XXX: only handle single-byte encodings
                // XXX: this is not efficient
                //let s = match str::from_utf8_lossy(slice) {
                let s = match std::str::from_utf8(slice) {
                    Ok(val) => val,
                    Err(err) => {
                        eprintln!("ERROR: Invalid UTF-8 sequence during from_utf8: {:?}", err);
                        continue;
                    }
                };
                let mut dst: [u8; 4] = [0, 0, 0, 0];
                for c in s.chars() {
                    let c_ = char_to_char_noraw(c);
                    let _cs = c_.encode_utf8(&mut dst);
                    match stdout_lock.write(&dst) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("ERROR: StdoutLock.write({:?}) error {}", &dst, err);
                        }
                    }
                }
            }
        }
        match stdout_lock.flush() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: stdout flushing error {}", err);
            }
        }
    }

    /// create `String` from known bytes referenced by `self.lineparts`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_char_noraw`)
    /// XXX: not efficient!
    /// TODO: this would be more efficient returning `&str`
    ///       https://bes.github.io/blog/rust-strings
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub(crate) fn _to_String_raw(self: &Line, raw: bool) -> String {
        let mut sz: usize = 0;
        for linepart in &self.lineparts {
            sz += linepart.len();
        }
        let mut s1 = String::with_capacity(sz);

        for linepart in &self.lineparts {
            if raw {
                // transform slices to `str`, can this be done more efficiently?
                // XXX: here is a good place to use `bstr`
                let s2 = &(&*linepart.blockp)[linepart.blocki_beg..linepart.blocki_end];
                let s3 = match std::str::from_utf8(s2) {
                    Ok(val) => val,
                    Err(err) => {
                        let fo1 = self.fileoffset_begin() + (linepart.blocki_beg as FileOffset);
                        let fo2 = self.fileoffset_begin() + (linepart.blocki_end as FileOffset);
                        eprintln!("ERROR: failed to convert [u8] at FileOffset[{}‥{}] to utf8 str; {}", fo1, fo2, err);
                        continue;
                    }
                };
                s1.push_str(s3);
            } else {
                // copy u8 as char to `s1`
                let stop = linepart.len();
                let block_iter = (&*linepart.blockp).iter();
                for (bi, b) in block_iter.skip(linepart.blocki_beg).enumerate() {
                    if bi >= stop {
                        break;
                    }
                    let c = byte_to_char_noraw(*b);
                    s1.push(c);
                }
            }
        }

        s1
    }

    // XXX: rust does not support function overloading which is really surprising and disappointing
    /// `Line` to `String`
    #[allow(dead_code, non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &Line) -> String {
        self._to_String_raw(true)
    }

    /// `Line` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &Line) -> String {
        self._to_String_raw(false)
    }

    /// slice that represents the entire `Line`
    /// if `Line` does not cross a Block then this returns slice into the `Block`,
    /// otherwise it requires a copy of `Block`s data
    /// XXX: naive implementation
    /// XXX: cannot return slice because
    ///      1. size not known at compile time so cannot place on stack
    ///      2. slice is an array which is not an "owned type"
    /// TODO: add tests
    /// CANDIDATE FOR REMOVAL
    pub(crate) fn as_bytes(self: &Line) -> Bytes {
        assert_gt!(self.lineparts.len(), 0, "This Line has no LineParts");
        // efficient case, Line does not cross any Blocks
        if self.lineparts.len() == 1 {
            let bi_beg = self.lineparts[0].blocki_beg;
            let bi_end = self.lineparts[0].blocki_end;
            assert_eq!(bi_end - bi_beg, self.len(), "bi_end-bi_beg != line.len()");
            return Bytes::from(&(*(self.lineparts[0].blockp))[bi_beg..bi_end]);
        }
        // not efficient case, Line crosses stored Blocks so have to create a new vec
        let sz = self.len();
        assert_ne!(sz, 0, "self.len() is zero!?");
        let mut data = Bytes::with_capacity(sz);
        for lp in self.lineparts.iter() {
            let bi_beg = lp.blocki_beg;
            let bi_end = lp.blocki_end;
            data.extend_from_slice(&(*(lp.blockp))[bi_beg..bi_end]);
        }
        assert_eq!(data.len(), self.len(), "Line.as_bytes: data.len() != self.len()");

        data
    }

    /// do be do
    /// CANDIDATE FOR REMOVAL
    //pub fn as_vec(self: &Line, beg: LineIndex, end: LineIndex) -> Bytes {
    #[allow(unreachable_code)]
    pub(crate) fn as_vec(self: &Line, beg: LineIndex, end: LineIndex) -> Bytes {
        assert_gt!(self.lineparts.len(), 0, "This Line has no LineParts");
        // efficient case, Line does not cross any Blocks
        if self.lineparts.len() == 1 {
            //let bi_beg = self.lineparts[0].blocki_beg;
            //let bi_end = self.lineparts[0].blocki_end;
            assert_le!(end - beg, self.len(), "end-beg > line.len()");

            return Bytes::from(&(*(self.lineparts[0].blockp))[beg as usize..end as usize]);
        }
        unreachable!("as_vec does not handle multiple lineparts yet");
        // XXX: incredibly inefficient case, Line crosses stored Blocks so have to create a new vec
        let sz = self.len();
        assert_ne!(sz, 0, "self.len() is zero!?");
        let mut data: Bytes = Bytes::with_capacity(sz);
        for lp in self.lineparts.iter() {
            let bi_beg = lp.blocki_beg;
            let bi_end = lp.blocki_end;
            data.extend_from_slice(&(*(lp.blockp))[bi_beg..bi_end]);
        }
        assert_eq!(data.len(), self.len(), "Line.as_vec: data.len() != self.len()");
        data
    }
}

/// Specialized Reader that uses BlockReader to find FoToLine
pub struct LineReader<'linereader> {
    pub(crate) blockreader: BlockReader<'linereader>,
    /// track `Line` found among blocks in `blockreader`, tracked by line beginning `FileOffset`
    /// key value `FileOffset` should agree with `(*LineP).fileoffset_begin()`
    pub lines: FoToLine,
    /// for all `Lines`, map `Line.fileoffset_end` to `Line.fileoffset_beg`
    foend_to_fobeg: FoToFo,
    /// count of `Line`s. Tracked outside of `self.lines.len()` as that may
    /// have contents removed when --streaming
    lines_count: u64,
    /// char size in bytes
    /// TODO: handle char sizes > 1 byte
    /// TODO: handle multi-byte encodings
    _charsz: CharSz,
    /// internal LRU cache for `find_line`
    _find_line_lru_cache: LinesLRUCache,
    /// internal stats
    pub(crate) _find_line_lru_cache_hit: u64,
    /// internal stats
    pub(crate) _find_line_lru_cache_miss: u64,
}

impl fmt::Debug for LineReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //let f_ = match &self.file_metadata {
        //    None => format!("None"),
        //    Some(val) => format!("{:?}", val.file_type()),
        //};
        f.debug_struct("LineReader")
            //.field("@", format!("{:p}", &self))
            .field("blockreader", &self.blockreader)
            .field("_charsz", &self._charsz)
            .field("lines", &self.lines)
            .finish()
    }
}

// XXX: cannot place these within `impl LineReader`?
/// minimum char storage size in bytes
static CHARSZ_MIN: CharSz = 1;
/// maximum char storage size in bytes
static CHARSZ_MAX: CharSz = 4;
/// default char storage size in bytes
/// XXX: does not handle multi-byte encodings (e.g. UTF-8) or multi-byte character storage (e.g. UTF-32)
static CHARSZ: CharSz = CHARSZ_MIN;

/// implement the LineReader things
impl<'linereader> LineReader<'linereader> {
    const FIND_LINE_LRC_CACHE_SZ: usize = 8;

    pub fn new(path: &'linereader FPath, blocksz: BlockSz) -> Result<LineReader<'linereader>> {
        // XXX: multi-byte
        assert_ge!(
            blocksz,
            (CHARSZ_MIN as BlockSz),
            "BlockSz {} is too small, must be greater than or equal {}",
            blocksz,
            CHARSZ_MAX
        );
        assert_ne!(blocksz, 0, "BlockSz is zero");
        let mut br = BlockReader::new(path, blocksz);
        if let Err(err) = br.open() {
            return Err(err);
        };
        Ok(LineReader {
            blockreader: br,
            lines: FoToLine::new(),
            foend_to_fobeg: FoToFo::new(),
            lines_count: 0,
            _charsz: CHARSZ,
            _find_line_lru_cache: LinesLRUCache::new(LineReader::FIND_LINE_LRC_CACHE_SZ),
            _find_line_lru_cache_hit: 0,
            _find_line_lru_cache_miss: 0,
        })
    }

    /// smallest size character in bytes
    pub fn charsz(&self) -> usize {
        self._charsz
    }

    pub fn blocksz(&self) -> BlockSz {
        self.blockreader.blocksz
    }

    pub fn filesz(&self) -> BlockSz {
        self.blockreader.filesz
    }

    pub fn path(&self) -> &FPath {
        &self.blockreader.path
    }

    /// print `Line` at `fileoffset`
    /// return `false` if `fileoffset` not found
    #[cfg(any(debug_assertions,test))]
    pub fn print(&self, fileoffset: &FileOffset) -> bool {
        if !self.lines.contains_key(fileoffset) {
            return false;
        }
        let lp = &self.lines[fileoffset];
        lp.print(true);
        true
    }

    /// Testing helper only
    /// print all known `Line`s
    #[cfg(any(debug_assertions,test))]
    pub fn print_all(&self) {
        for fo in self.lines.keys() {
            self.print(fo);
        }
    }

    /// count of lines processed by this LineReader
    pub fn count(&self) -> u64 {
        self.lines_count
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    pub fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        BlockReader::block_offset_at_file_offset(fileoffset, self.blocksz())
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    pub fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz())
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        BlockReader::file_offset_at_block_offset_index(blockoffset, self.blocksz(), blockindex)
    }

    /// return block index at given `FileOffset`
    pub fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        BlockReader::block_index_at_file_offset(fileoffset, self.blocksz())
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    pub fn file_blocks_count(&self) -> u64 {
        BlockReader::file_blocks_count(self.filesz(), self.blocksz())
    }

    pub fn blockoffset_last(&self) -> BlockOffset {
        self.blockreader.blockoffset_last()
    }
    
    fn insert_line(&mut self, line: Line) -> LineP {
        debug_eprintln!("{}LineReader.insert_line(Line @{:p})", sn(), &line);
        let fo_beg = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let rl = LineP::new(line);
        debug_eprintln!("{}LineReader.insert_line: lines.insert({}, Line @{:p})", so(), fo_beg, &(*rl));
        debug_assert!(!self.lines.contains_key(&fo_beg), "self.lines already contains key {}", fo_beg);
        self.lines.insert(fo_beg, rl.clone());
        debug_eprintln!("{}LineReader.insert_line: foend_to_fobeg.insert({}, {})", so(), fo_end, fo_beg);
        debug_assert!(!self.foend_to_fobeg.contains_key(&fo_end), "self.foend_to_fobeg already contains key {}", fo_end);
        self.foend_to_fobeg.insert(fo_end, fo_beg);
        self.lines_count += 1;
        debug_eprintln!("{}LineReader.insert_line() returning @{:p}", sx(), rl);
        rl
    }

    /// does `self` "contain" this `fileoffset`? That is, already know about it?
    /// the `fileoffset` can be any value (does not have to be begining or ending of
    /// a `Line`).
    fn lines_contains(&self, fileoffset: &FileOffset) -> bool {
        let fo_beg = match self.foend_to_fobeg.range(fileoffset..).next() {
            Some((_, fo_beg_)) => {
                fo_beg_
            },
            None => { return false; },
        };
        if fileoffset < fo_beg {
            return false;
        }
        self.lines.contains_key(fo_beg)
    }

    /// for any `FileOffset`, get the `Line` (if available)
    /// The passed `FileOffset` can be any value (does not have to be begining or ending of
    /// a `Line`).
    // XXX: this fails `pub(in crate::Readers::linereader_tests)`
    pub(crate) fn get_linep(&self, fileoffset: &FileOffset) -> Option<LineP> {
        let fo_beg = match self.foend_to_fobeg.range(fileoffset..).next() {
            Some((_, fo_beg_)) => {
                fo_beg_
            },
            None => { return None; },
        };
        if fileoffset < fo_beg {
            return None;
        }
        match self.lines.get(&fo_beg) {
            Some(slp) => { Some(slp.clone()) }
            None => { None }
        }
    }

    /// find next `Line` starting from `fileoffset`
    /// in the process of finding, creates and stores the `Line` from underlying `Block` data
    /// returns `Found`(`FileOffset` of beginning of the _next_ line, found `LineP`)
    /// reaching end of file (and no new line) returns `Found_EOF`
    /// reaching end of file returns `FileOffset` value that is one byte past the actual end of file (and should not be used)
    /// otherwise `Err`, all other `Result::Err` errors are propagated
    /// 
    /// similar to `find_sysline`, `read_block`
    ///
    /// XXX: presumes single-byte to one '\n', does not handle UTF-16 or UTF-32 or other (`charsz` hardcoded to 1)
    /// TODO: [2021/08/30] handle different encodings
    /// XXX: this function is fragile and cumbersome, any tweaks require extensive retesting
    pub fn find_line(&mut self, fileoffset: FileOffset) -> ResultS4_LineFind {
        debug_eprintln!("{}find_line(LineReader@{:p}, {})", sn(), self, fileoffset);

        // some helpful constants
        let charsz_fo = self._charsz as FileOffset;
        let charsz_bi = self._charsz as BlockIndex;
        let filesz = self.filesz();
        let blockoffset_last = self.blockoffset_last();

        // check LRU cache first (this is very fast)
        match self._find_line_lru_cache.get(&fileoffset) {
            Some(rlp) => {
                debug_eprint!("{}find_line: found LRU cached for offset {}", sx(), fileoffset);
                self._find_line_lru_cache_hit += 1;
                match rlp {
                    ResultS4_LineFind::Found(val) => {
                        debug_eprintln!(" return ResultS4_LineFind::Found(({}, …)) @[{}, {}]", val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                        return ResultS4_LineFind::Found((val.0, val.1.clone()));
                    }
                    ResultS4_LineFind::Found_EOF(val) => {
                        debug_eprintln!(" return ResultS4_LineFind::Found_EOF(({}, …)) @[{}, {}]", val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                        return ResultS4_LineFind::Found_EOF((val.0, val.1.clone()));
                    }
                    ResultS4_LineFind::Done => {
                        debug_eprintln!(" return ResultS4_LineFind::Done");
                        return ResultS4_LineFind::Done;
                    }
                    _ => {
                        debug_eprintln!(" Err");
                        eprintln!("ERROR: unexpected value store in _find_line_lru_cache, fileoffset {}", fileoffset);
                    }
                }
            }
            None => {
                self._find_line_lru_cache_miss += 1;
                debug_eprintln!("{}find_line: fileoffset {} not found in LRU cache", so(), fileoffset);
            }
        }

        // handle special cases
        if filesz == 0 {
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Done; file is empty", sx());
            return ResultS4_LineFind::Done;
        } else if fileoffset > filesz {
            // TODO: [2021/10] need to decide on consistent behavior for passing fileoffset > filesz
            //       should it really Error or be Done?
            //       Make that consisetent among all LineReader and SyslineReader `find_*` functions
            /*
            let err = Error::new(
                ErrorKind::AddrNotAvailable,
                format!("Passed fileoffset {} past file size {}", fileoffset, filesz),
            );
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Err({}); fileoffset {} was too big filesz {}!", sx(), err, fileoffset, filesz);
            return ResultS4_LineFind::Err(err);
            */
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Done; fileoffset {} was too big filesz {}!", sx(), fileoffset, filesz);
            return ResultS4_LineFind::Done;
        } else if fileoffset == filesz {
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Done(); fileoffset {} is at end of file {}!", sx(), fileoffset, filesz);
            return ResultS4_LineFind::Done;
        }

        {
            // first check if there is a `Line` already known at this fileoffset
            if self.lines.contains_key(&fileoffset) {
                debug_eprintln!("{}find_line: hit cache for FileOffset {}", so(), fileoffset);

                //debug_assert!(self.lines_by_range.contains_key(&fileoffset), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fileoffset);
                //debug_assert!(self.lines_by_range.contains(&fileoffset), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fileoffset);
                //debug_assert!(hashset_contains(&self.lines_by_range, &fileoffset), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fileoffset);
                debug_assert!(self.lines_contains(&fileoffset), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fileoffset);

                let lp = self.lines[&fileoffset].clone();
                let fo_next = (*lp).fileoffset_end() + charsz_fo;
                // TODO: add stats like BlockReader._stats*
                debug_eprintln!("{}find_line: LRU Cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_next);
                self._find_line_lru_cache
                    .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
                debug_eprintln!("{}find_line: return ResultS4_LineFind::Found({}, {:p})  @[{}, {}]", sx(), fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end());
                return ResultS4_LineFind::Found((fo_next, lp));
            }
            //match self.lines_by_range.get(&fileoffset) {
            //match hashset_get(&self.lines_by_range, &fileoffset) {
            match self.get_linep(&fileoffset) {
                //Some(fo_range) => {
                Some(lp) => {
                    debug_eprintln!(
                        "{}find_line: self.get_linep({}) returned @{:p}",
                        so(),
                        fileoffset,
                        lp
                    );
                    //debug_eprintln!(
                    //    "{}find_line: fileoffset {} refers to self.lines_by_range {:?}",
                    //    so(),
                    //    fileoffset,
                    //    fo_range
                    //);
                    //let lp = self.lines[fo_range].clone();
                    //let lp = self.lines[&fo_range.start].clone();
                    let fo_next = (*lp).fileoffset_end() + charsz_fo;
                    // TODO: add stats like BlockReader._stats*
                    debug_eprintln!("{}find_line: LRU Cache put({}, Found({}, …)) {:?}", so(), fileoffset, fo_next, (*lp).to_String_noraw());
                    self._find_line_lru_cache
                        .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
                    debug_eprintln!("{}find_line: return ResultS4_LineFind::Found({}, {:p}) @[{}, {}]", sx(), fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end());
                    return ResultS4_LineFind::Found((fo_next, lp));
                }
                None => {
                    //self._read_block_cache_lru_miss += 1;
                    debug_eprintln!("{}find_line: fileoffset {} not found in self.lines_by_range", so(), fileoffset);
                }
            }
            debug_eprintln!("{}find_line: fileoffset {} not found in self.lines", so(), fileoffset);
            debug_eprintln!("{}find_line: searching for first newline newline A …", so());
        }

        //
        // walk through blocks and bytes looking for beginning of a line (a newline character; part A)
        //

        // block pointer to the current block of interest
        let mut bp: BlockP;
        // found newline part A? Line begins after that newline
        let mut found_nl_a = false;
        // should point to beginning of `Line` (one char after found newline A)
        let mut fo_nl_a: FileOffset = 0;
        // if at first byte of file no need to search for first newline
        if fileoffset == 0 {
            found_nl_a = true;
            debug_eprintln!("{}find_line: newline A is {} because at beginning of file!", so(), fo_nl_a);
        }
        // if prior char at fileoffset-1 has newline then use that
        // caller's commonly call this function `find_line` in a sequence so it's an easy check
        // with likely success
        if !found_nl_a {
            // XXX: single-byte encoding, does not handle multi-byte
            let fo1 = fileoffset - charsz_fo;
            if self.foend_to_fobeg.contains_key(&fo1) {
                found_nl_a = true;
                debug_eprintln!(
                    "{}find_line: found newline A {} from lookup of passed fileoffset-1 {}",
                    so(),
                    fo1,
                    fileoffset - 1
                );
                // `fo_nl_a` should refer to first char past newline A
                // XXX: single-byte encoding
                fo_nl_a = fo1 + charsz_fo;
            }
        }

        let mut eof = false;
        let mut bo = self.block_offset_at_file_offset(fileoffset);
        let mut bin_beg_init_a = self.block_index_at_file_offset(fileoffset);
        while !found_nl_a && bo <= blockoffset_last {
            debug_eprintln!("{}find_line: self.blockreader.read_block({})", so(), bo);
            match self.blockreader.read_block(bo) {
                Ok(val) => {
                    debug_eprintln!(
                        "{}find_line: read_block returned Block @{:p} len {} while searching for newline A",
                        so(),
                        &(*val),
                        (*val).len()
                    );
                    bp = val;
                }
                Err(err) => {
                    if err.kind() == EndOfFile {
                        debug_eprintln!("{}find_line: read_block returned EndOfFile {:?} searching for found_nl_a failed (IS THIS AN ERROR???????)", so(), self.path());
                        // reached end of file, no beginning newlines found
                        // TODO: Is this an error state? should this be handled differently?
                        debug_eprintln!("{}find_line: return ResultS4_LineFind::Done; EOF from read_block; NOT SURE IF THIS IS CORRECT", sx());
                        return ResultS4_LineFind::Done;
                    }
                    debug_eprintln!("{}find_line: LRU cache put({}, Done)", so(), fileoffset);
                    self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Done);
                    debug_eprintln!("{}find_line: return ResultS4_LineFind::Done; NOT SURE IF THIS IS CORRECT!!!!", sx());
                    return ResultS4_LineFind::Done;
                }
            }
            let blen = (*bp).len() as BlockIndex;
            let mut bin_beg = bin_beg_init_a;
            while bin_beg < blen {
                // XXX: single-byte encoding
                if (*bp)[bin_beg] == NLu8 {
                    found_nl_a = true;
                    fo_nl_a = self.file_offset_at_block_offset_index(bo, bin_beg);
                    debug_eprintln!(
                        "{}find_line: found newline A from byte search at fileoffset {} ≟ blockoffset {} blockindex {}",
                        so(),
                        fo_nl_a,
                        bo,
                        bin_beg
                    );
                    // `fo_nl_a` should refer to first char past newline A
                    // XXX: single-byte encoding
                    fo_nl_a += charsz_fo;
                    break;
                }
                // XXX: single-byte encoding
                bin_beg += charsz_bi;
            }
            if found_nl_a {
                break;
            }
            bin_beg_init_a = 0;
            bo += 1;
            if bo > blockoffset_last {
                debug_eprintln!("{}find_line: EOF blockoffset {} > {} blockoffset_last", so(), bo, blockoffset_last);
                eof = true;
                break;
            }
            if fo_nl_a >= filesz {
                debug_eprintln!("{}find_line: EOF newline A fileoffset {} > {} file size", so(), fo_nl_a, filesz);
                eof = true;
                break;
            }
        } // ! found_nl_a

        assert_lt!(fo_nl_a, filesz + 1, "ERROR: newline A {} is past end of file {}", fo_nl_a, filesz + 1);
        if eof {
            debug_eprintln!("{}find_line: LRU Cache put({}, Done)", so(), fileoffset);
            self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Done);
            // the last character in the file is a newline
            // XXX: is this correct?
            debug_eprintln!(
                "{}find_line: return ResultS4_LineFind::Done; newline A is at last char in file {}, not a line IS THIS CORRECT?",
                sx(),
                filesz - 1
            );
            return ResultS4_LineFind::Done;
        }

        //
        // walk through blocks and bytes looking for ending of line (a newline character; part B)
        //
        debug_eprintln!(
            "{}find_line: found first newline A, searching for second B newline starting at {} …",
            so(),
            fo_nl_a
        );

        {
            // …but before doing work of discovering a new `Line` (part B), first checks various
            // maps in `self` to see if this `Line` has already been discovered and processed
            if self.lines.contains_key(&fo_nl_a) {
                debug_eprintln!("{}find_line: hit for self.lines for FileOffset {} (before part B)", so(), fo_nl_a);

                //debug_assert!(self.lines_by_range.contains_key(&fo_nl_a), "self.lines and self.lines_by_range are out of synch on key {}", fo_nl_a);
                //debug_assert!(self.lines_by_range.contains(&fo_nl_a), "self.lines and self.lines_by_range are out of synch on key {}", fo_nl_a);
                //debug_assert!(hashset_contains(&self.lines_by_range, &fo_nl_a), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fo_nl_a);
                debug_assert!(self.lines_contains(&fo_nl_a), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fo_nl_a);

                let lp = self.lines[&fo_nl_a].clone();
                let fo_next = (*lp).fileoffset_end() + charsz_fo;
                // TODO: add stats like BlockReader._stats*
                debug_eprintln!("{}find_line: LRU Cache put({}, Found_EOF({}, …))", so(), fo_nl_a, fo_next);
                self._find_line_lru_cache
                    .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
                debug_eprintln!("{}find_line: return ResultS4_LineFind::Found({}, {:p})  @[{}, {}]", sx(), fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end());
                return ResultS4_LineFind::Found((fo_next, lp));
            }
            match self.get_linep(&fo_nl_a) {
            //match hashset_get(&self.lines_by_range, &fileoffset) {
                //Some(fo_range) => {
                Some(lp) => {
                    debug_eprintln!(
                        "{}find_line: self.get_linep({}) returned {:p}",
                        so(),
                        fo_nl_a,
                        lp
                    );
                    //debug_eprintln!(
                    //    "{}find_line: fo_nl_a {} refers to self.lines_by_range {:?}",
                    //    so(),
                    //    fo_nl_a,
                    //    fo_range
                    //);
                    //let lp = self.lines[fo_range].clone();
                    //let lp = self.lines[&fo_range.start].clone();
                    let fo_next = (*lp).fileoffset_end() + charsz_fo;
                    // TODO: add stats like BlockReader._stats*
                    debug_eprintln!("{}find_line: LRU Cache put({}, Found({}, …)) {:?}", so(), fo_nl_a, fo_next, (*lp).to_String_noraw());
                    self._find_line_lru_cache
                        .put(fo_nl_a, ResultS4_LineFind::Found((fo_next, lp.clone())));
                    debug_eprintln!("{}find_line: return ResultS4_LineFind::Found({}, {:p}) @[{}, {}]", sx(), fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end());
                    return ResultS4_LineFind::Found((fo_next, lp));
                }
                None => {
                    //self._read_block_cache_lru_miss += 1;
                    debug_eprintln!("{}find_line: fileoffset {} not found in self.lines_by_range", so(), fo_nl_a);
                }
            }
        }

        // getting here means this function is discovering a brand new `Line` (part B)

        // found newline part B? Line ends at this
        let mut found_nl_b: bool = false;
        // this is effectively the cursor that is being analyzed
        let mut fo_nl_b: FileOffset = fo_nl_a;
        // set for the first loop (first block), then is zero
        let mut bin_beg_init_b: BlockIndex = self.block_index_at_file_offset(fo_nl_b);
        // append LinePart to this `Line`
        let mut line: Line = Line::new();
        bo = self.block_offset_at_file_offset(fo_nl_b);
        while !found_nl_b && bo <= blockoffset_last {
            debug_eprintln!("{}find_line: self.blockreader.read_block({})", so(), bo);
            match self.blockreader.read_block(bo) {
                Ok(val) => {
                    debug_eprintln!(
                        "{}find_line: read_block returned Block @{:p} len {} while searching for newline B",
                        so(),
                        &(*val),
                        (*val).len()
                    );
                    bp = val;
                }
                Err(err) => {
                    if err.kind() == EndOfFile {
                        debug_eprintln!(
                            "{}find_line: read_block returned EndOfFile {:?} while searching for newline B",
                            so(),
                            self.path()
                        );
                        let rl = self.insert_line(line);
                        let fo_ = (*rl).fileoffset_end() + charsz_fo;
                        debug_eprintln!("{}find_line: LRU Cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_);
                        self._find_line_lru_cache
                            .put(fileoffset, ResultS4_LineFind::Found_EOF((fo_, rl.clone())));
                        debug_eprintln!(
                            "{}find_line: return ResultS4_LineFind::Found_EOF(({}, {:p})) @[{} , {}]; {:?}",
                            sx(),
                            fo_,
                            &*rl,
                            (*rl).fileoffset_begin(),
                            (*rl).fileoffset_end(),
                            (*rl).to_String_noraw()
                        );
                        return ResultS4_LineFind::Found_EOF((fo_, rl));
                    }
                    debug_eprintln!("{}find_line: return ResultS4_LineFind::Err({:?});", sx(), err);
                    return ResultS4_LineFind::Err(err);
                }
            }
            let blen = (*bp).len() as BlockIndex;
            let bin_beg = bin_beg_init_b;
            let mut bin_end = bin_beg;
            while bin_end < blen {
                // XXX: single-byte encoding
                if (*bp)[bin_end] == NLu8 {
                    found_nl_b = true;
                    fo_nl_b = self.file_offset_at_block_offset_index(bo, bin_end);
                    bin_end += charsz_bi; // refer to one past end
                    debug_eprintln!(
                        "{}find_line: newline B found by byte search fileoffset {} ≟ blockoffset {} blockindex {}",
                        so(),
                        fo_nl_b,
                        bo,
                        bin_end
                    );
                    break;
                }
                // XXX: single-byte encoding
                bin_end += charsz_bi;
            }
            let fo_beg = self.file_offset_at_block_offset_index(bo, bin_beg);
            // sanity check
            if fo_beg == filesz {
                assert_eq!(bin_end - bin_beg, 0, "fileoffset of beginning of line {} is at end of file, yet found a linepart of length {} (expected zero)", fo_beg, bin_end - bin_beg);
            }
            // sanity check
            if bin_end - bin_beg == 0 {
                assert_eq!(fo_beg, filesz, "fileoffset of beginning of line {} is at end of file, yet found a linepart of length {} (expected zero)", fo_beg, bin_end - bin_beg);
            }
            // at end of file, "zero length" LinePart, skip creating a `LinePart`
            if bin_end - bin_beg == 0 && fo_beg == filesz {
                debug_eprintln!("{}find_line: no newline B, at end of file", so());
                break;
            }
            let li = LinePart::new(bin_beg, bin_end, bp.clone(), fo_beg, bo, self.blocksz());
            debug_eprintln!("{}find_line: Line.push({:?})", so(), &li);
            line.push(li);
            if found_nl_b {
                break;
            }
            bin_beg_init_b = 0;
            bo += 1;
            if bo > blockoffset_last {
                break;
            }
        } // ! found_nl_b

        // may occur in files ending on a single newline
        if line.count() == 0 {
            debug_eprintln!("{}find_line: LRU Cache put({}, Done)", so(), fileoffset);
            self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Done);
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Done;", sx());
            return ResultS4_LineFind::Done;
        }

        // sanity check
        debug_eprintln!("{}find_line: return {:?};", so(), line);
        let fo_beg = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        //assert_eq!(fo_beg, fo_nl_a, "line.fileoffset_begin() {} ≠ {} searched fo_nl_a", fo_beg, fo_nl_a);
        //assert_eq!(fo_end, fo_nl_b, "line.fileoffset_end() {} ≠ {} searched fo_nl_b", fo_end, fo_nl_b);
        if fo_beg != fo_nl_a {
            debug_eprintln!("WARNING: line.fileoffset_begin() {} ≠ {} searched fo_nl_a", fo_beg, fo_nl_a);
        }
        if fo_end != fo_nl_b {
            debug_eprintln!("WARNING: line.fileoffset_end() {} ≠ {} searched fo_nl_b", fo_end, fo_nl_b);
        }
        assert_lt!(fo_end, filesz, "line.fileoffset_end() {} is past file size {}", fo_end, filesz);

        let rl = self.insert_line(line);
        debug_eprintln!("{}find_line: LRU Cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_end + 1);
        self._find_line_lru_cache
            .put(fileoffset, ResultS4_LineFind::Found((fo_end + 1, rl.clone())));
        debug_eprintln!(
            "{}find_line: return ResultS4_LineFind::Found(({}, @{:p})) @[{}, {}]; {:?}",
            sx(),
            fo_end + 1,
            &*rl,
            (*rl).fileoffset_begin(),
            (*rl).fileoffset_end(),
            (*rl).to_String_noraw()
        );

        ResultS4_LineFind::Found((fo_end + 1, rl))
    }
}