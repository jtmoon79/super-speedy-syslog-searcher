// Readers/linereader.rs
//

pub use crate::common::{
    FPath,
    FileOffset,
    CharSz,
    NLu8,
    ResultS3,
};

use crate::Readers::blockreader::{
    BlockSz,
    BlockOffset,
    BlockIndex,
    BlockP,
    Slices,
    BlockReader,
    ResultS3_ReadBlock,
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
    snx,
    so,
    sx,
};

use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::io::{
    Error,
    Result,
};
use std::io::prelude::*;
use std::sync::Arc;

extern crate debug_print;
use debug_print::{debug_eprint, debug_eprintln};
#[allow(unused_imports)]
use debug_print::{debug_print, debug_println};

extern crate lru;
use lru::LruCache;

extern crate mime;
use mime::{
    Mime,
};

extern crate mime_guess;
use mime_guess::MimeGuess;

extern crate mime_sniffer;
use mime_sniffer::MimeTypeSniffer;  // adds extension method `sniff_mime_type` to `[u8]`

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

type ResultS3_find_byte_val = (BlockIndex, BlockP);
type ResultS3_find_byte = ResultS3<ResultS3_find_byte_val, Error>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LinePart, Line, and LineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/*
lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref MIME_PARSEABLE: [&str] = [
        "",
    ];
}
*/

/// Struct describing a part or all of a line within a `Block`
/// A "line" can span more than one `Block`. This tracks part or all of a line within
/// one `Block`. One `LinePart` to one `Block`.
/// One or more `LinePart`s are necessary to represent an entire `Line`.
pub struct LinePart {
    /// index into the `blockp`, index at beginning
    /// used as-is in slice notation
    pub blocki_beg: BlockIndex,
    /// index into the `blockp`, index at one after ending '\n' (may refer to one past end of `Block`)
    /// used as-is in slice notation
    pub blocki_end: BlockIndex,
    /// the `Block` pointer
    pub blockp: BlockP,
    /// the byte offset into the file where this `LinePart` begins
    pub fileoffset: FileOffset,
    /// blockoffset: debug helper, might be good to get rid of this?
    pub blockoffset: BlockOffset,
    /// the file-designated BlockSz, _not_ the size of this particular block (yes, somewhat confusing)
    /// blocksz: debug helper, might be good to get rid of this? seems confusing and unnecessary.
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

    /// create a new `LinePart`. Remember that `blocki_end` points to one byte past
    /// because it used directly in byte array slice notation (exclusive).
    /// i.e. `(*blockp)[blocki_beg..blocki_end]`
    pub fn new(
        blocki_beg: BlockIndex,
        blocki_end: BlockIndex,
        blockp: BlockP,
        fileoffset: FileOffset,
        blockoffset: BlockOffset,
        blocksz: BlockSz,
    ) -> LinePart {
        debug_eprintln!(
            "{}LinePart::new(blocki_beg {}, blocki_end {}, Block @{:p}, fileoffset {}, blockoffset {}, blocksz {}) (blockp.len() {})",
            so(),
            blocki_beg,
            blocki_end,
            &*blockp,
            fileoffset,
            blockoffset,
            blocksz,
            (*blockp).len(),
        );
        // some sanity checks
        assert_ne!(fileoffset, FileOffset::MAX, "Bad fileoffset MAX");
        assert_ne!(blockoffset, BlockOffset::MAX, "Bad blockoffset MAX");
        let fo1 = BlockReader::file_offset_at_block_offset(blockoffset, blocksz);
        assert_le!(fo1, fileoffset, "Bad FileOffset {}, must ≥ {} (based on file_offset_at_block_offset(BlockOffset {}, BlockSz {}))", fileoffset, fo1, blockoffset, blocksz);
        let fo2 = BlockReader::file_offset_at_block_offset(blockoffset + 1, blocksz);
        assert_le!(fileoffset, fo2, "Bad FileOffset {}, must ≤ {} (based on file_offset_at_block_offset(BlockOffset {}, BlockSz {}))", fileoffset, fo2, blockoffset + 1, blocksz);
        let bo = BlockReader::block_offset_at_file_offset(fileoffset, blocksz);
        assert_eq!(blockoffset, bo, "Bad BlockOffset {}, expected {} (based on block_offset_at_file_offset(FileOffset {}, BlockSz {}))", blockoffset, bo, fileoffset, blocksz);
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
        assert_le!(((*blockp).len() as BlockSz), blocksz, "block.len() {} should be ≤ blocksz {}", (*blockp).len(), blocksz);
        assert_ge!((*blockp).len(), blocki_end - blocki_beg, "block.len() {} should be ≥ {} (blocki_end {} - {} blocki_beg)", (*blockp).len(), blocki_end - blocki_beg, blocki_end, blocki_beg);
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
    pub(crate) lineparts: LineParts,
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

    /// insert `linepart` at back of `self.lineparts`
    pub fn append(&mut self, linepart: LinePart) {
        debug_eprintln!("{}Line.append({:?}) {:?}", so(), &linepart, linepart.to_String_noraw());
        let l_ = self.lineparts.len();
        if l_ > 0 {
            // sanity checks; each `LinePart` should be stored in same order as it appears in the file
            // only need to compare to last `LinePart`
            let li = &self.lineparts[l_ - 1];
            assert_le!(
                li.blockoffset,
                linepart.blockoffset,
                "Line.append: Prior stored LinePart at blockoffset {} is after passed LinePart at blockoffset {}\n{:?}\n{:?}\n",
                li.blockoffset,
                linepart.blockoffset,
                li,
                linepart,
            );
            assert_lt!(
                li.fileoffset,
                linepart.fileoffset,
                "Line.append: Prior stored LinePart at fileoffset {} is at or after passed LinePart at fileoffset {}\n{:?}\n{:?}\n",
                li.fileoffset,
                linepart.fileoffset,
                li,
                linepart,
            );
        }
        self.lineparts.push(linepart);
    }

    /// insert `linepart` at front of `self.lineparts`
    pub fn prepend(&mut self, linepart: LinePart) {
        debug_eprintln!("{}Line.prepend({:?}) {:?}", so(), &linepart, linepart.to_String_noraw());
        let l_ = self.lineparts.len();
        if l_ > 0 {
            // sanity checks; each `LinePart` should be stored in same order as it appears in the file
            // only need to compare to last `LinePart`
            let li: &LinePart = &self.lineparts[0];
            assert_ge!(
                li.blockoffset,
                linepart.blockoffset,
                "Line.prepend: Prior stored LinePart at blockoffset {} is before passed LinePart at blockoffset {} (should be after)",
                li.blockoffset,
                linepart.blockoffset,
            );
            assert_gt!(
                li.fileoffset,
                linepart.fileoffset,
                "Line.prepend: Prior stored LinePart at fileoffset {} is at or before passed LinePart at fileoffset {} (should be after)",
                li.fileoffset,
                linepart.fileoffset,
            );
        }
        self.lineparts.insert(0, linepart);
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

    /// length of this `Line` in bytes
    pub fn len(self: &Line) -> usize {
        (self.fileoffset_end() - self.fileoffset_begin() + 1) as usize
    }

    /// count of `LinePart` in `self.lineparts.len()`
    /// XXX: redundant, need to decide which to keep.
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

    /// does this `Line` store a `LinePart.blockoffset == bo`?
    ///
    /// O(n)
    pub fn stores_blockoffset(self: &Line, bo: BlockOffset) -> bool {
        for linepart in self.lineparts.iter() {
            if linepart.blockoffset == bo {
                return true;
            }
        }
        false
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
    /// TODO: handle char sizes > 1 byte, multi-byte encodings
    _charsz: CharSz,
    /// has self.zeroblock_analsys completed?
    _zeroblock_analysis_done: bool,
    /// enable internal LRU cache for `find_line` (default `true`)
    _find_line_lru_cache_enabled: bool,
    /// internal LRU cache for `find_line`
    _find_line_lru_cache: LinesLRUCache,
    /// internal stats
    pub(crate) _find_line_lru_cache_hit: u64,
    /// internal stats
    pub(crate) _find_line_lru_cache_miss: u64,
    // internal stats
    //pub(crate) _bytes_processed: u64,
}

impl fmt::Debug for LineReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //let f_ = match &self.file_metadata {
        //    None => format!("None"),
        //    Some(val) => format!("{:?}", val.file_type()),
        //};
        f.debug_struct("LineReader")
            //.field("@", format!("{:p}", &self))
            .field("LRU cache enabled", &self._find_line_lru_cache_enabled)
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
            _zeroblock_analysis_done: false,
            _find_line_lru_cache_enabled: true,
            _find_line_lru_cache: LinesLRUCache::new(LineReader::FIND_LINE_LRC_CACHE_SZ),
            _find_line_lru_cache_hit: 0,
            _find_line_lru_cache_miss: 0,
        })
    }

    /// smallest size character in bytes
    #[inline]
    pub fn charsz(&self) -> usize {
        self._charsz
    }

    /// `Block` size in bytes
    #[inline]
    pub fn blocksz(&self) -> BlockSz {
        self.blockreader.blocksz
    }

    /// File Size in bytes
    #[inline]
    pub fn filesz(&self) -> BlockSz {
        self.blockreader.filesz
    }

    /// File path
    #[inline]
    pub fn path(&self) -> &FPath {
        &self.blockreader.path
    }

    /// enable internal LRU cache used by `find_line`
    pub fn LRU_cache_enable(&mut self) {
        self._find_line_lru_cache_enabled = true;
        self._find_line_lru_cache.clear();
        self._find_line_lru_cache.resize(LineReader::FIND_LINE_LRC_CACHE_SZ);
    }

    /// disable internal LRU cache used by `find_line`
    /// intended for testing
    pub fn LRU_cache_disable(&mut self) {
        self._find_line_lru_cache_enabled = false;
        self._find_line_lru_cache.clear();
        self._find_line_lru_cache.resize(0);
    }

    /// Testing helper only
    /// print `Line` at `fileoffset`
    /// return `false` if `fileoffset` not found
    #[cfg(any(debug_assertions, test))]
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
    #[cfg(any(debug_assertions, test))]
    pub(crate) fn print_all(&self) {
        for fo in self.lines.keys() {
            self.print(fo);
        }
    }

    /// Testing helper only
    /// copy `Line`s at `fileoffset` to String buffer
    #[cfg(test)]
    pub(crate) fn copy_line(&self, fileoffset: &FileOffset, buffer: &mut String) -> bool {
        if !self.lines.contains_key(fileoffset) {
            return false;
        }
        let line: &Line = &self.lines[fileoffset];
        let s: String = line.to_String();
        eprintln!("{}LineReader.copy_line({:2}) {:?}", snx(), fileoffset, s);
        buffer.push_str(s.as_str());
        true
    }

    /// Testing helper only
    /// copy all `Line`s to String buffer
    #[cfg(test)]
    pub(crate) fn copy_all_lines(&self, buffer: &mut String) {
        // reserve capacity in buffer
        let mut sz: usize = 0;
        for fo in self.lines.keys() {
            sz += &(self.lines[fo]).len();
        }
        sz += 1;
        buffer.clear();
        if buffer.capacity() < sz {
            eprintln!("{}LineReader.copy_all_lines() buffer.reserve({:?})", sn(), sz);
            buffer.reserve(sz);
        }
        for fo in self.lines.keys() {
            if !self.copy_line(fo, buffer){
                panic!("copy_line({}, ...) failed", fo);
            }
        }
        eprintln!("{}LineReader.copy_all_lines()", sx());
    }


    /// count of lines processed by this LineReader
    #[inline]
    pub fn count(&self) -> u64 {
        self.lines_count
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    #[inline]
    pub fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        BlockReader::block_offset_at_file_offset(fileoffset, self.blocksz())
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    #[inline]
    pub fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz())
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    #[inline]
    pub fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        BlockReader::file_offset_at_block_offset_index(blockoffset, self.blocksz(), blockindex)
    }

    /// return block index at given `FileOffset`
    #[inline]
    pub fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        BlockReader::block_index_at_file_offset(fileoffset, self.blocksz())
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    #[inline]
    pub fn file_blocks_count(&self) -> u64 {
        BlockReader::file_blocks_count(self.filesz(), self.blocksz())
    }

    /// last valid `BlockOffset` for the file (inclusive)
    /// (expected largest `BlockOffset` value, no relation to `Block`s processed)
    pub fn blockoffset_last(&self) -> BlockOffset {
        self.blockreader.blockoffset_last()
    }

    /// should `LineReader` attempt to parse this file/MIME type?
    /// (i.e. call `find_line`)
    pub fn parseable_mimestr(mimeguess: &str) -> bool {
        /*
        // see https://docs.rs/mime/latest/mime/
        let TEXT = mime::TEXT.as_ref();
        let TEXT_PLAIN = mime::TEXT_PLAIN.as_ref();
        let TEXT_PLAIN_UTF_8 = mime::TEXT_PLAIN_UTF_8.as_ref();
        let TEXT_STAR = mime::TEXT_STAR.as_ref();
        let UTF_8 = mime::UTF_8.as_ref();
        */

        debug_eprintln!("{}LineReader::parseable_mimestr: mimeguess {:?}", snx(), mimeguess);

        // see https://docs.rs/mime/latest/src/mime/lib.rs.html#572-575
        match mimeguess {
            "plain"
            | "text"
            | "text/plain"
            | "text/*"
            | "utf-8" => {true},
            _ => {false},
        }
    }

    /// should `LineReader` attempt to parse this file/MIME type?
    /// (i.e. call `find_line`)
    pub fn parseable_mimeguess(mimeguess: &MimeGuess) -> bool {
        match mimeguess.first() {
            Some(first_) => {
                LineReader::parseable_mimestr(first_.as_ref())
            },
            None => {false},
        }
    }

    /// Given a file of an unknown MIME type (`self.blockreader.mimeguess.is_empty()`),
    /// analyze block 0 (the first block, the "zero block") and make best guesses
    /// about the file.
    ///
    /// Return `true` if enough is known about the file to proceed with byte analysis
    /// (e.g. calls from `LineReader::find_line`).
    /// Else return `false`.
    /// If block 0 has not been read from source then return `false`.
    ///
    /// Should only call to completion once per `LineReader` instance.
    pub(crate) fn zeroblock_process(&mut self) -> std::io::Result<bool> {
        assert!(!self._zeroblock_analysis_done, "zeroblock_analysis should only be completed once.");
        if !self.blockreader.mimeguess.is_empty() {
            self._zeroblock_analysis_done = true;
            debug_eprintln!("{}linereader.zeroblock_process: mimeguess is {:?}", sn(), self.blockreader.mimeguess);
            let is_parseable = LineReader::parseable_mimeguess(&self.blockreader.mimeguess);
            debug_eprintln!("{}linereader.zeroblock_process: Ok({:?})", sx(), is_parseable);
            return Ok(is_parseable);
        }
        let bo_zero: FileOffset = 0;
        debug_eprintln!("{}linereader.zeroblock_process: self.blockreader.read_block({:?})", sn(), bo_zero);
        let bptr: BlockP = match self.blockreader.read_block(bo_zero) {
            ResultS3_ReadBlock::Found(val) => val,
            ResultS3_ReadBlock::Done => {
                debug_eprintln!("{}linereader.zeroblock_process: read_block({}) returned Done for {:?}, return Error(UnexpectedEof)", sx(), bo_zero, self.path());
                return std::io::Result::Err(
                    io::Error::new(std::io::ErrorKind::UnexpectedEof, "zeroblock_process read_block(0) returned Done")
                );
            },
            ResultS3_ReadBlock::Err(err) => {
                debug_eprintln!("{}linereader.zeroblock_process: read_block({}) returned Err {:?}", sx(), bo_zero, err);
                return std::io::Result::Err(err);
            },
        };

        let sniff: String = String::from((*bptr).as_slice().sniff_mime_type().unwrap_or(""));
        debug_eprintln!("{}linereader.zeroblock_process: sniff_mime_type {:?}", so(), sniff);
        let is_parseable = LineReader::parseable_mimestr(sniff.as_ref());

        self._zeroblock_analysis_done = true;

        debug_eprintln!("{}linereader.zeroblock_process: Ok({:?})", sx(), is_parseable);

        Ok(is_parseable)
    }

    /// store information about a single line in a file
    /// returns pointer to that `Line`
    fn insert_line(&mut self, line: Line) -> LineP {
        debug_eprintln!("{}LineReader.insert_line(Line @{:p})", sn(), &line);
        let fo_beg = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let linep = LineP::new(line);
        debug_eprintln!("{}LineReader.insert_line: lines.insert({}, Line @{:p})", so(), fo_beg, &(*linep));
        debug_assert!(!self.lines.contains_key(&fo_beg), "self.lines already contains key {}", fo_beg);
        self.lines.insert(fo_beg, linep.clone());
        debug_eprintln!("{}LineReader.insert_line: foend_to_fobeg.insert({}, {})", so(), fo_end, fo_beg);
        debug_assert!(!self.foend_to_fobeg.contains_key(&fo_end), "self.foend_to_fobeg already contains key {}", fo_end);
        self.foend_to_fobeg.insert(fo_end, fo_beg);
        self.lines_count += 1;
        debug_eprintln!("{}LineReader.insert_line() returning @{:p}", sx(), linep);

        linep
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
    /// O(log(n))
    // XXX: this fails `pub(in crate::Readers::linereader_tests)`
    pub(crate) fn get_linep(&self, fileoffset: &FileOffset) -> Option<LineP> {
        // I'm somewhat sure this is O(log(n))
        let fo_beg: &FileOffset = match self.foend_to_fobeg.range(fileoffset..).next() {
            Some((_, fo_beg_)) => {
                fo_beg_
            },
            None => { return None; },
        };
        if fileoffset < fo_beg {
            return None;
        }
        #[allow(clippy::manual_map)]
        match self.lines.get(fo_beg) {
            Some(lp) => { Some(lp.clone()) }
            None => { None }
        }
    }

    fn block_find_byte_forwards(&mut self, bof: BlockOffset, begin: Option<BlockIndex>, byte_: u8) -> ResultS3_find_byte
    {
        unimplemented!("NOT YET TESTED");
        let bptr: BlockP = match self.blockreader.read_block(bof) {
            ResultS3_ReadBlock::Found(val) => {
                debug_eprintln!(
                    "{}block_find_byte_forwards: read_block({}) returned Found Block @{:p} len {} while searching for newline A",
                    so(),
                    bof,
                    &(*val),
                    (*val).len()
                );
                val
            },
            ResultS3_ReadBlock::Done => {
                debug_eprintln!("{}block_find_byte_forwards: read_block({}) returned Done {:?}", so(), bof, self.path());
                return ResultS3_find_byte::Done;
            },
            ResultS3_ReadBlock::Err(err) => {
                debug_eprintln!("{}block_find_byte_forwards: read_block({}) returned Err, return ResultS3_find_byte::Err({:?})", sx(), bof, err);
                return ResultS3_find_byte::Err(err);
            },
        };
        let mut bi_beg: BlockIndex = begin.unwrap_or(0);
        let bi_end = bptr.len() as BlockIndex;
        assert_ge!(bi_end, self._charsz as BlockIndex, "blockindex bi_end {} is less than charsz; not yet handled", bi_end);
        debug_eprintln!(
            "{}block_find_byte_forwards: scan block {} forwards, starting from blockindex {} (fileoffset {}) up to blockindex {}",
            so(),
            bof,
            bi_beg,
            self.file_offset_at_block_offset_index(bof, bi_beg),
            bi_end,
        );
        loop {
            // XXX: single-byte encoding
            if (*bptr)[bi_beg] == byte_ {
                debug_eprintln!(
                    "{}block_find_byte_forwards: found byte 0x{:02x} {:?} during byte search, blockoffset {} blockindex {}",
                    sx(),
                    byte_,
                    byte_to_char_noraw(byte_),
                    bof,
                    bi_beg,
                );
                return ResultS3_find_byte::Found((bi_beg, bptr));
            } else {
                bi_beg += self._charsz as BlockIndex;
            }
            if bi_beg >= bi_end {
                break;
            }
        }  // end loop
        debug_eprintln!("{}block_find_byte_forwards: return Done", so());
        return ResultS3_find_byte::Done;
    }

    fn block_find_byte_backwards(&mut self, bof: BlockOffset, begin: Option<BlockIndex>, byte_: u8) -> ResultS3_find_byte
    {
        unimplemented!("NOT YET TESTED");
        let bptr: BlockP = match self.blockreader.read_block(bof) {
            ResultS3_ReadBlock::Found(val) => {
                debug_eprintln!(
                    "{}block_find_byte_backwards: read_block({}) returned Found Block @{:p} len {} while searching for newline A",
                    so(),
                    bof,
                    &(*val),
                    (*val).len()
                );
                val
            },
            ResultS3_ReadBlock::Done => {
                debug_eprintln!("{}block_find_byte_backwards: read_block({}) returned Done {:?}", so(), bof, self.path());
                return ResultS3_find_byte::Done;
            },
            ResultS3_ReadBlock::Err(err) => {
                debug_eprintln!("{}block_find_byte_backwards: read_block({}) returned Err, return ResultS3_find_byte::Err({:?})", sx(), bof, err);
                return ResultS3_find_byte::Err(err);
            },
        };
        let bi_beg: BlockIndex = 0;
        let mut bi_end: BlockIndex = begin.unwrap_or(bptr.len() as BlockIndex);
        assert_ge!(bi_end, self._charsz, "blockindex bi_end {} is less than charsz; not yet handled", bi_end);
        debug_eprintln!(
            "{}block_find_byte_backwards: scan block {} backwards, starting from blockindex {} (fileoffset {}) down to blockindex {}",
            so(),
            bof,
            bi_end,
            self.file_offset_at_block_offset_index(bof, bi_end),
            bi_beg,
        );
        loop {
            // XXX: single-byte encoding
            if (*bptr)[bi_end] == byte_ {
                debug_eprintln!(
                    "{}block_find_byte_forwards: found byte 0x{:02x} {:?} during byte search, blockoffset {} blockindex {}",
                    sx(),
                    byte_,
                    byte_to_char_noraw(byte_),
                    bof,
                    bi_end,
                );
                return ResultS3_find_byte::Found((bi_end, bptr));
            }
            if bi_end == 0 {
                break;
            }
            bi_end -= self._charsz as BlockIndex;
            if bi_end < bi_beg {
                break;
            }
        }  // end loop
        debug_eprintln!("{}block_find_byte_forwards: return Done", so());
        return ResultS3_find_byte::Done;
    }

    /// find next `Line` starting from `fileoffset`.
    /// During the process of finding, creates and stores the `Line` from underlying `Block` data.
    /// Returns `Found`(`FileOffset` of beginning of the _next_ line, found `LineP`)
    /// Reaching end of file (and no new line) returns `Found_EOF`.
    /// Reaching end of file returns `FileOffset` value that is one byte past the actual end of file (and should not be used).
    /// Otherwise returns `Err`, all other `Result::Err` errors are propagated.
    ///
    /// This function has the densest number of byte↔char transitions.
    ///
    /// correllary to `find_sysline`, `read_block`
    ///
    /// Throughout this function, newline A points to the line beginning, newline B
    /// points to line ending. Both are inclusive.
    /// Here are two defining cases of this function:
    ///
    /// given a file of four newlines:
    ///
    ///     byte: 0123
    ///     char: ␊␊␊␊
    ///
    /// calls to `find_line` would result in a `Line`
    ///
    ///     A=Line.fileoffset_begin();
    ///     B=Line.fileoffset_end();
    ///     Val=Line.to_string();
    ///
    ///                     A,B  Val
    ///     find_line(0) -> 0,0 "␊"
    ///     find_line(1) -> 1,1 "␊"
    ///     find_line(2) -> 2,2 "␊"
    ///     find_line(3) -> 3,3 "␊"
    ///
    /// given a file with two alphabet chars and one newline:
    ///
    ///     012
    ///     x␊y
    ///
    ///                     A,B  Val
    ///     fine_line(0) -> 0,1 "x␊"
    ///     fine_line(1) -> 0,1 "x␊"
    ///     fine_line(2) -> 2,2 "y"
    ///
    /// XXX: presumes a single-byte can represent a '\n'; i.e. does not handle UTF-16 or UTF-32 or other.
    /// TODO: [2021/08/30] handle different encodings
    /// XXX: returning the "next fileoffset (along with `LineP`) is jenky. Just return the `LineP`.
    ///      and/or do not return "fileoffset next" for `Found_EOF` (doesn't make sense).
    ///      and/or add `iter` capabilities to `Line` that will hide tracking the "next fileoffset".
    /// XXX: this function is fragile and cumbersome, any tweaks require extensive retesting
    pub fn find_line(&mut self, fileoffset: FileOffset) -> ResultS4_LineFind {
        debug_eprintln!("{}find_line(LineReader@{:p}, {})", sn(), self, fileoffset);

        // some helpful constants
        let charsz_fo = self._charsz as FileOffset;
        let charsz_bi = self._charsz as BlockIndex;
        let filesz = self.filesz();
        let blockoffset_last = self.blockoffset_last();
        let bsz: BlockSz = self.blocksz();

        // check LRU cache first (this is very fast)
        if self._find_line_lru_cache_enabled {
            match self._find_line_lru_cache.get(&fileoffset) {
                Some(rlp) => {
                    debug_eprint!("{}find_line({}): found LRU cached for offset {}", sx(), fileoffset, fileoffset);
                    self._find_line_lru_cache_hit += 1;
                    match rlp {
                        ResultS4_LineFind::Found(val) => {
                            debug_eprintln!(" return ResultS4_LineFind::Found(({}, …)) @[{}, {}] {:?}", val.0, val.1.fileoffset_begin(), val.1.fileoffset_end(), val.1.to_String_noraw());
                            return ResultS4_LineFind::Found((val.0, val.1.clone()));
                        }
                        ResultS4_LineFind::Found_EOF(val) => {
                            debug_eprintln!(" return ResultS4_LineFind::Found_EOF(({}, …)) @[{}, {}] {:?}", val.0, val.1.fileoffset_begin(), val.1.fileoffset_end(), val.1.to_String_noraw());
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
        }

        // handle special cases
        if filesz == 0 {
            debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Done; file is empty", sx(), fileoffset);
            return ResultS4_LineFind::Done;
        } else if fileoffset > filesz {
            // TODO: [2021/10] need to decide on consistent behavior for passing fileoffset > filesz
            //       should it really Error or be Done?
            //       Make that consisetent among all LineReader and SyslineReader `find_*` functions
            debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Done; fileoffset {} was too big filesz {}!", sx(), fileoffset, fileoffset, filesz);
            return ResultS4_LineFind::Done;
        } else if fileoffset == filesz {
            debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Done(); fileoffset {} is at end of file {}!", sx(), fileoffset, fileoffset, filesz);
            return ResultS4_LineFind::Done;
        }

        // search containers of `Line`s
        {
            // first, check if there is a `Line` already known at this fileoffset
            if self.lines.contains_key(&fileoffset) {
                debug_eprintln!("{}find_line: hit self.lines for FileOffset {}", so(), fileoffset);
                debug_assert!(self.lines_contains(&fileoffset), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fileoffset);
                let lp = self.lines[&fileoffset].clone();
                let fo_next = (*lp).fileoffset_end() + charsz_fo;
                if self._find_line_lru_cache_enabled {
                    debug_eprintln!("{}find_line: LRU Cache put({}, Found({}, …))", so(), fileoffset, fo_next);
                    self._find_line_lru_cache
                        .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
                }
                debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Found({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end(), (*lp).to_String_noraw());
                return ResultS4_LineFind::Found((fo_next, lp));
            }
            // second, check if there is a `Line` at a preceding offset
            match self.get_linep(&fileoffset) {
                Some(lp) => {
                    debug_eprintln!(
                        "{}find_line: self.get_linep({}) returned @{:p}",
                        so(),
                        fileoffset,
                        lp
                    );
                    let fo_next = (*lp).fileoffset_end() + charsz_fo;
                    if self._find_line_lru_cache_enabled {
                        debug_eprintln!("{}find_line: LRU Cache put({}, Found({}, …)) {:?}", so(), fileoffset, fo_next, (*lp).to_String_noraw());
                        self._find_line_lru_cache
                            .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
                    }
                    debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Found({}, {:p}) @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end(), (*lp).to_String_noraw());
                    return ResultS4_LineFind::Found((fo_next, lp));
                }
                None => {
                    debug_eprintln!("{}find_line: fileoffset {} not found in self.lines_by_range", so(), fileoffset);
                }
            }
            debug_eprintln!("{}find_line: fileoffset {} not found in self.lines", so(), fileoffset);
        }

        //
        // could not find `fileoffset` from prior saved information so...
        // walk through blocks and bytes looking for beginning of a line (a newline character)
        // start with newline search "part B" (look for line terminating '\n' or end of file)
        // then do search "part A" (look for line terminating '\n' of previous Line or beginning
        // of file)
        //

        debug_eprintln!("{}find_line: searching for first newline B (line terminator) …", so());

        // block pointer to the current block of interest
        let mut bptr: BlockP;
        // found newline part A? Line begins after that newline
        let mut found_nl_a = false;
        // found newline part B? Line ends at this.
        let mut found_nl_b: bool = false;
        // `fo_nl_a` should eventually "point" to beginning of `Line` (one char after found newline A)
        let mut fo_nl_a: FileOffset = fileoffset;
        // `fo_nl_b` should eventually "point" to end of `Line` including the newline char.
        // if  line is terminated by end-of-file then "points" to last char of file.
        let mut fo_nl_b: FileOffset = fileoffset;
        let mut bi_nl_b: BlockIndex;
        let mut fo_nl_b_in_middle: bool = false;
        // was newline B actually the end of file?
        let mut nl_b_eof: bool = false;
        // if at first byte of file no need to search for first newline
        if fileoffset == 0 {
            found_nl_a = true;
            debug_eprintln!("{}find_line: newline A0 is {} because fileoffset {} is beginning of file!", so(), fo_nl_a, fileoffset);
        }
        // append new `LinePart`s to this `Line`
        let mut line: Line = Line::new();

        // The "middle" block is block referred to by `fileoffset` and could be the inexact "middle"
        // of the eventually found `Line`. In other words, `Line.fileoffset_begin` could be before it (or in it)
        // and `Line.fileoffset_end` could be after it (or in it).
        let bo_middle: BlockOffset = self.block_offset_at_file_offset(fileoffset);
        let bi_middle: BlockIndex = self.block_index_at_file_offset(fileoffset);
        let mut bi_middle_end: BlockIndex = bi_middle;
        let bptr_middle: BlockP;

        // search within "middle" block for newline B
        {  // arbitrary statement block
            bptr_middle = match self.blockreader.read_block(bo_middle) {
                ResultS3_ReadBlock::Found(val) => {
                    debug_eprintln!(
                        "{}find_line B1: read_block({}) returned Found Block @{:p} len {} while searching for newline A",
                        so(),
                        bo_middle,
                        &(*val),
                        (*val).len()
                    );
                    val
                },
                ResultS3_ReadBlock::Done => {
                    debug_eprintln!("{}find_line B1: read_block({}) returned Done {:?}", so(), bo_middle, self.path());
                    return ResultS4_LineFind::Done;
                },
                ResultS3_ReadBlock::Err(err) => {
                    debug_eprintln!("{}find_line({}) B1: read_block({}) returned Err, return ResultS4_LineFind::Err({:?})", sx(), fileoffset, bo_middle, err);
                    return ResultS4_LineFind::Err(err);
                }
            };
            let mut bi_at: BlockIndex = bi_middle;
            let bi_stop: BlockIndex = bptr_middle.len() as BlockIndex;
            assert_ge!(bi_stop, charsz_bi, "bi_stop is less than charsz; not yet handled");
            // XXX: multi-byte
            //bi_beg = bi_stop - charsz_bi;
            debug_eprintln!("{}find_line B1: scan middle block {} forwards, starting from blockindex {} (fileoffset {}) searching for newline B", so(), bo_middle, bi_at, self.file_offset_at_block_offset_index(bo_middle, bi_at));
            loop {
                // XXX: single-byte encoding
                if (*bptr_middle)[bi_at] == NLu8 {
                    found_nl_b = true;
                    fo_nl_b = self.file_offset_at_block_offset_index(bo_middle, bi_at);
                    bi_nl_b = bi_at;
                    bi_middle_end = bi_at;
                    debug_eprintln!("{}find_line B1: bi_middle_end {:?} bi_nl_b {:?} fo_nl_b {:?}", so(), bi_middle_end, bi_nl_b, fo_nl_b);
                    fo_nl_b_in_middle = true;
                    debug_eprintln!(
                        "{}find_line B1: found newline B in middle block during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                        so(),
                        bo_middle,
                        bi_at,
                        fo_nl_b,
                        byte_to_char_noraw((*bptr_middle)[bi_at]),
                    );
                    break;
                } else {
                    bi_at += charsz_bi;
                }
                if bi_at >= bi_stop {
                    break;
                }
            }  // end loop
            // if (newline B not found and the "middle" block was the last block) then eof is newline B
            if !found_nl_b && bo_middle == blockoffset_last {
                found_nl_b = true;
                assert_ge!(bi_at, charsz_bi, "blockindex begin {} is less than charsz {} before attempt to subtract to determine newline B1 at end of file", bi_at, charsz_bi);
                let bi_ = bi_at - charsz_bi;
                fo_nl_b = self.file_offset_at_block_offset_index(bo_middle, bi_);
                bi_nl_b = bi_;
                bi_middle_end = bi_;
                debug_eprintln!("{}find_line B1: bi_middle_end {:?} bi_nl_b {:?} fo_nl_b {:?} blockoffset_last {:?}", so(), bi_middle_end, bi_nl_b, fo_nl_b, blockoffset_last);
                fo_nl_b_in_middle = true;
                nl_b_eof = true;
                assert_eq!(
                    fo_nl_b, filesz - 1,
                    "newline B1 fileoffset {} is at end of file, yet filesz is {}; there was a bad calcuation of newline B1 from blockoffset {} blockindex {} (blockoffset last {})",
                    fo_nl_b,
                    filesz,
                    bo_middle,
                    bi_,
                    blockoffset_last,
                );
            } else if !found_nl_b {
                bi_middle_end = bi_stop - charsz_bi;
                debug_eprintln!("{}find_line B1: bi_middle_end {:?}", so(), bi_middle_end);
            }
        }

        if found_nl_b {
            debug_eprintln!("{}find_line B2: skip continued backwards search for newline B (already found)", so());
        } else {
            // search within proceeding blocks for newline B
            let mut bi_beg: BlockIndex = 99999;  // XXX: lame "uninitialized" signal
            let mut bi_end: BlockIndex;
            let mut bof = bo_middle + 1;
            while !found_nl_b && bof <= blockoffset_last {
                let bptr: BlockP = match self.blockreader.read_block(bof) {
                    ResultS3_ReadBlock::Found(val) => {
                        debug_eprintln!(
                            "{}find_line B2: read_block({}) returned Found Block @{:p} len {} while searching for newline B",
                            so(),
                            bof,
                            &(*val),
                            (*val).len()
                        );
                        val
                    },
                    ResultS3_ReadBlock::Done => {
                        debug_eprintln!("{}find_line B2: read_block({}) returned Done {:?}", so(), bof, self.path());
                        return ResultS4_LineFind::Done;
                    },
                    ResultS3_ReadBlock::Err(err) => {
                        debug_eprintln!("{}find_line({}) B2: read_block({}) returned Err, return ResultS4_LineFind::Err({:?})", sx(), fileoffset, bof, err);
                        return ResultS4_LineFind::Err(err);
                    },
                };
                bi_beg = 0;
                bi_end = bptr.len() as BlockIndex;
                assert_ge!(bi_end, charsz_bi, "blockindex bi_end {} is less than charsz; not yet handled", bi_end);
                // XXX: multi-byte
                //bi_beg = bi_end - charsz_bi;
                debug_eprintln!(
                    "{}find_line B2: scan block {} forwards, starting from blockindex {} (fileoffset {}) up to blockindex {} searching for newline B",
                    so(),
                    bof,
                    bi_beg,
                    self.file_offset_at_block_offset_index(bof, bi_beg),
                    bi_end,
                );
                loop {
                    // XXX: single-byte encoding
                    if (*bptr)[bi_beg] == NLu8 {
                        found_nl_b = true;
                        fo_nl_b = self.file_offset_at_block_offset_index(bof, bi_beg);
                        bi_nl_b = bi_beg;
                        assert!(!fo_nl_b_in_middle, "fo_nl_b_in_middle should be false");
                        debug_eprintln!(
                            "{}find_line B2: found newline B during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                            so(),
                            bof,
                            bi_beg,
                            fo_nl_b,
                            byte_to_char_noraw((*bptr)[bi_beg]),
                        );
                        let li = LinePart::new(
                            0,
                            bi_beg + 1,
                            bptr.clone(),
                            self.file_offset_at_block_offset_index(bof, 0),
                            bof,
                            self.blocksz(),
                        );
                        line.append(li);
                        break;
                    } else {
                        bi_beg += charsz_bi;
                    }
                    if bi_beg >= bi_end {
                        break;
                    }
                }  // end loop
                if found_nl_b {
                    break;
                }
                // newline B was not found in this `Block`, but must save this `Block` information as a `LinePart.
                let li = LinePart::new(
                    0,
                    bi_beg,
                    bptr.clone(),
                    self.file_offset_at_block_offset_index(bof, 0),
                    bof,
                    self.blocksz(),
                );
                line.append(li);
                bof += 1;
            }  // end while bof <= blockoffset_last
            // if newline B not found and last checked block was the last block then eof is newline B
            if !found_nl_b && bof >= blockoffset_last {
                bof = blockoffset_last;
                found_nl_b = true;
                assert_ge!(bi_beg, charsz_bi, "blockindex begin {} is less than charsz {} before attempt to subtract to determine newline B2 at end of file", bi_beg, charsz_bi);
                let bi_ = bi_beg - charsz_bi;
                fo_nl_b = self.file_offset_at_block_offset_index(bof, bi_);
                bi_nl_b = bi_;
                nl_b_eof = true;
                debug_eprintln!(
                    "{}find_line B2: newline B is end of file; blockoffset {} blockindex {} fileoffset {} (blockoffset last {})",
                    so(),
                    bof,
                    bi_,
                    fo_nl_b,
                    blockoffset_last,
                );
                assert_eq!(
                    fo_nl_b, filesz - 1,
                    "newline B2 fileoffset {} is at end of file, yet filesz is {}; there was a bad calcuation of newline B2 from blockoffset {} blockindex {} (last blockoffset {})",
                    fo_nl_b,
                    filesz,
                    bof,
                    bi_,
                    blockoffset_last,
                );
            }
        }  // end if ! found_nl_b

        //
        // walk backwards through blocks and bytes looking for newline A (line terminator of preceding Line or beginning of file)
        //

        debug_eprintln!(
            "{}find_line: found first newline B at FileOffset {}, searching for preceding newline A. Search starts at FileOffset {} …",
            so(),
            fo_nl_b,
            fileoffset,
        );

        // if found_nl_a was already found then this function can return
        if found_nl_a {
            debug_eprintln!("{}find_line A0: already found newline A and newline B, return early", so());
            assert_eq!(fo_nl_a, 0, "newline A is {}, only reason newline A should be found at this point was if passed fileoffset 0, (passed fileoffset {})", fo_nl_a, fileoffset);
            let li = LinePart::new(
                self.block_index_at_file_offset(fo_nl_a),
                bi_middle_end + 1,
                bptr_middle,
                fo_nl_a,
                self.block_offset_at_file_offset(fo_nl_a),
                self.blocksz(),
            );
            line.prepend(li);
            let linep = self.insert_line(line);
            let fo_next = fo_nl_b + charsz_fo;
            debug_assert_eq!(fo_next, (*linep).fileoffset_end() + charsz_fo, "mismatching fo_next {} != (*linep).fileoffset_end()+1", fo_next);
            if !nl_b_eof {
                if self._find_line_lru_cache_enabled {
                    debug_eprintln!("{}find_line A0: LRU cache put({}, Found(({}, @{:p})))", so(), fileoffset, fo_next, linep);
                    self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Found((fo_next, linep.clone())));
                }
                debug_eprintln!("{}find_line({}) A0: return ResultS4_LineFind::Found(({}, @{:p})) @[{}, {}] {:?}", sx(), fileoffset, fo_next, linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                return ResultS4_LineFind::Found((fo_next, linep.clone()));
            } else {
                if self._find_line_lru_cache_enabled {
                    debug_eprintln!("{}find_line A0: LRU cache put({}, Found_EOF(({}, @{:p})))", so(), fileoffset, fo_next, linep);
                    self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Found_EOF((fo_next, linep.clone())));
                }
                debug_eprintln!("{}find_line({}) A0: return ResultS4_LineFind::Found_EOF(({}, @{:p})) @[{}, {}] {:?}", sx(), fileoffset, fo_next, linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                return ResultS4_LineFind::Found_EOF((fo_next, linep.clone()));
            };
        }
        assert!(!found_nl_a, "already found newline A; was finding it once not good enough?");
        assert!(found_nl_b, "found newline A, have not found newline B; bird with one wing.");
    
        // …but before doing work of discovering a new `Line` (newline A),
        // check various maps at `fileoffset + 1` to see if the preceding
        // `Line` has already been discovered and processed.
        // This is common for sequential calls to this function.
        if fileoffset >= charsz_fo {
            let fo_ = fileoffset - charsz_fo;
            if self.lines.contains_key(&fo_) {
                debug_eprintln!("{}find_line A1a: hit in self.lines for FileOffset {} (before part A)", so(), fo_);
                fo_nl_a = fo_;
                let linep_prev = self.lines[&fo_nl_a].clone();
                assert_eq!(
                    fo_nl_a, (*linep_prev).fileoffset_end(),
                    "get_linep({}) returned Line with fileoffset_end() {}; these should match",
                    fo_nl_a,
                    (*linep_prev).fileoffset_end(),
                );
                let li = LinePart::new(
                    self.block_index_at_file_offset(fileoffset),
                    bi_middle_end + 1,
                    bptr_middle,
                    fileoffset,
                    self.block_offset_at_file_offset(fileoffset),
                    self.blocksz(),
                );
                line.prepend(li);
                let linep = self.insert_line(line);
                let fo_next = fo_nl_b + charsz_fo;
                if self._find_line_lru_cache_enabled {
                    debug_eprintln!("{}find_line A1a: LRU Cache put({}, Found({}, …)) {:?}", so(), fileoffset, fo_next, (*linep).to_String_noraw());
                    self._find_line_lru_cache
                        .put(fileoffset, ResultS4_LineFind::Found((fo_next, linep.clone())));
                }
                debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Found({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                return ResultS4_LineFind::Found((fo_next, linep));
            } else {
                debug_eprintln!("{}find_line A1a: miss in self.lines for FileOffset {} (quick check before part A)", so(), fo_);
            }
            match self.get_linep(&fo_) {
                Some(linep_prev) => {
                    debug_eprintln!(
                        "{}find_line A1b: self.get_linep({}) returned {:p}",
                        so(),
                        fo_,
                        linep_prev,
                    );
                    found_nl_a = true;
                    fo_nl_a = (*linep_prev).fileoffset_end();
                    assert_eq!(
                        fo_nl_a, fo_,
                        "get_linep({}) returned Line with fileoffset_end() {}; these should match",
                        fo_,
                        fo_nl_a,
                    );
                    let li = LinePart::new(
                        self.block_index_at_file_offset(fileoffset),
                        bi_middle_end + 1,
                        bptr_middle,
                        fileoffset,
                        self.block_offset_at_file_offset(fileoffset),
                        self.blocksz(),
                    );
                    line.prepend(li);
                    let linep = self.insert_line(line);
                    let fo_next = fo_nl_b + charsz_fo;
                    if self._find_line_lru_cache_enabled {
                        debug_eprintln!("{}find_line A1b: LRU Cache put({}, Found({}, …)) {:?}", so(), fileoffset, fo_next, (*linep).to_String_noraw());
                        self._find_line_lru_cache
                            .put(fileoffset, ResultS4_LineFind::Found((fo_next, linep.clone())));
                    }
                    debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Found({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                    return ResultS4_LineFind::Found((fo_next, linep));
                },
                None => {
                    debug_eprintln!("{}find_line A1b: self.get_linep({}) returned None (quick check before part A)", so(), fo_);
                },
            }
        }

        //
        // getting here means this function is discovering a brand new `Line` (searching for newline A)
        // walk *backwards* to find line-terminating newline of the preceding line (or beginning of file)
        //

        let fo_nl_a_search_start = std::cmp::max(fileoffset, charsz_fo) - charsz_fo;
        let mut bof: BlockOffset = self.block_offset_at_file_offset(fo_nl_a_search_start);
        let mut begof: bool = false;  // run into beginning of file (as in first byte)?
        // newline A plus one (one charsz past preceding Line terminating '\n')
        let mut fo_nl_a1: FileOffset = 0;

        if bof == bo_middle {
            // search for newline A starts within "middle" block
            let mut bi_at: BlockIndex = self.block_index_at_file_offset(fo_nl_a_search_start);
            let bi_stop: BlockIndex = 0;
            debug_eprintln!(
                "{}find_line A2a: scan middle block {} backwards, starting from blockindex {} (fileoffset {}) down to blockindex {} searching for newline A",
                so(), bo_middle, bi_at, self.file_offset_at_block_offset_index(bo_middle, bi_at), bi_stop,
            );
            loop {
                // XXX: single-byte encoding
                if (*bptr_middle)[bi_at] == NLu8 {
                    found_nl_a = true;
                    fo_nl_a = self.file_offset_at_block_offset_index(bo_middle, bi_at);
                    debug_eprintln!(
                        "{}find_line A2a: found newline A in middle block during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                        so(),
                        bo_middle,
                        bi_at,
                        fo_nl_a,
                        byte_to_char_noraw((*bptr_middle)[bi_at]),
                    );
                    // adjust offsets one forward
                    // XXX: single-byte encoding
                    fo_nl_a1 = fo_nl_a + charsz_fo;
                    bi_at += charsz_bi;
                    break;
                }
                if bi_at == 0 {
                    break;
                }
                // XXX: single-byte encoding
                bi_at -= charsz_bi;
                if bi_at < bi_stop {
                    break;
                }
            }
            let fo_: FileOffset;
            if found_nl_a {
                fo_ = fo_nl_a1;
            } else {
                debug_eprintln!("{}find_line A2a: newline A not found in middle block {} but store middle block", so(), bo_middle);
                fo_ = self.file_offset_at_block_offset_index(bo_middle, bi_at);
            }
            let li = LinePart::new(
                bi_at,
                bi_middle_end + 1,
                bptr_middle.clone(),
                fo_,
                bo_middle,
                self.blocksz(),
            );
            line.prepend(li);
            if bof != 0 {
                debug_eprintln!("{}find_line A2a: blockoffset set to {}", so(), bof);
                bof -= 1;
            } else {
                debug_eprintln!("{}find_line A2a: run into beginning of file", so());
                begof = true;
            }
        } else {
            debug_eprintln!("{}find_line A2b: search for newline A crossed block boundary {} -> {}, save LinePart", so(), bo_middle, bof);
            // the charsz shift backward to begin search for newline A crossed block boundary
            // so save linepart from "middle" block before searching further
            let li = LinePart::new(
                0,
                bi_middle_end + 1,
                bptr_middle.clone(),
                self.file_offset_at_block_offset_index(bo_middle, 0),
                bo_middle,
                self.blocksz(),
            );
            line.prepend(li);
        }

        if !found_nl_a && begof {
            found_nl_a = true;
            fo_nl_a = 0;
            fo_nl_a1 = 0;
        }

        if !found_nl_a && !begof {
            let mut bptr_prior: BlockP;
            let mut bptr: BlockP = bptr_middle.clone();
            let mut bi_start_prior: BlockIndex;
            let mut bi_start: BlockIndex = bi_middle;
            while !found_nl_a && !begof {
                // "middle" block should have been handled by now
                // remainder is to just walk backwards chedcking for first newline or beginning of file
                debug_eprintln!("{}find_line A4: searching blockoffset {} ...", so(), bof);
                bptr_prior = bptr;
                bptr = match self.blockreader.read_block(bof) {
                    ResultS3_ReadBlock::Found(val) => {
                        debug_eprintln!(
                            "{}find_line A4: read_block({}) returned Found Block @{:p} len {} while searching for newline A",
                            so(),
                            bof,
                            &(*val),
                            (*val).len()
                        );
                        val
                    },
                    ResultS3_ReadBlock::Done => {
                        debug_eprintln!("{}find_line A4: read_block({}) returned Done {:?}", so(), bof, self.path());
                        return ResultS4_LineFind::Done;
                    },
                    ResultS3_ReadBlock::Err(err) => {
                        debug_eprintln!("{}find_line({}) A4: read_block({}) returned Err, return ResultS4_LineFind::Err({:?})", sx(), fileoffset, bof, err);
                        return ResultS4_LineFind::Err(err);
                    }
                };
                let blen: BlockIndex = bptr.len() as BlockIndex;
                assert_ge!(blen, charsz_bi, "blen is less than charsz; not yet handled");
                bi_start_prior = bi_start;
                bi_start = blen - charsz_bi;
                let mut bi_at: BlockIndex = bi_start;
                let bi_stop: BlockIndex = 0;
                debug_eprintln!(
                    "{}find_line A5: scan block {} backwards, starting from blockindex {} (fileoffset {}) down to blockindex {} searching for newline A",
                    so(), bof, bi_at, self.file_offset_at_block_offset_index(bof, bi_at), bi_stop,
                );
                loop {
                    // XXX: single-byte encoding
                    if (*bptr)[bi_at] == NLu8 {
                        found_nl_a = true;
                        fo_nl_a = self.file_offset_at_block_offset_index(bof, bi_at);
                        debug_eprintln!(
                            "{}find_line A5: found newline A during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                            so(),
                            bof,
                            bi_at,
                            fo_nl_a,
                            byte_to_char_noraw((*bptr)[bi_at]),
                        );
                        // adjust offsets one forward
                        // XXX: single-byte encoding
                        fo_nl_a1 = fo_nl_a + charsz_fo;
                        bi_at += charsz_bi;
                        let bof_a1 = self.block_offset_at_file_offset(fo_nl_a1);
                        if bof_a1 == bof {
                            // newline A and first line char does not cross block boundary
                            debug_eprintln!("{}find_line A5: store current blockoffset {}", so(), bof);
                            let li = LinePart::new(
                                bi_at,
                                bi_start + 1,
                                bptr.clone(),
                                fo_nl_a1,
                                bof,
                                self.blocksz(),
                            );
                            line.prepend(li);
                        } else if !line.stores_blockoffset(bof_a1) {
                            // newline A and first line char does cross block boundary
                            debug_eprintln!("{}find_line A5: store prior blockoffset {}", so(), bof_a1);
                            // use prior block data
                            let li = LinePart::new(
                                0,
                                bi_start_prior + 1,
                                bptr_prior,
                                fo_nl_a1,
                                bof_a1,
                                self.blocksz(),
                            );
                            line.prepend(li);
                        } else {
                            // newline A and first line char does cross block boundary
                            debug_eprintln!("{}find_line A5: blockoffset {} was already stored", so(), bof_a1);
                        }
                        break;
                    }
                    if bi_at == 0 {
                        break;
                    }
                    // XXX: single-byte encoding
                    bi_at -= charsz_bi;
                    if bi_at < bi_stop {
                        break;
                    }
                }
                if found_nl_a {
                    break;
                }
                debug_eprintln!("{}find_line A5: store blockoffset {}", so(), bof);
                let li = LinePart::new(
                    bi_stop,
                    bi_start + 1,
                    bptr.clone(),
                    self.file_offset_at_block_offset_index(bof, 0),
                    bof,
                    self.blocksz(),
                );
                line.prepend(li);                
                if bof != 0 {
                    // newline A not found
                    debug_eprintln!("{}find_line A5: newline A not found in block {}", so(), bof);
                    bof -= 1;
                } else {
                    // hit beginning of file, "newline A" is the beginning of the file (not a newline char)
                    // store that first block
                    debug_eprintln!("{}find_line A5: ran into beginning of file", so());
                    found_nl_a = true;
                    begof = true;
                    debug_assert!(line.stores_blockoffset(0), "block 0 was not stored but ran into beginning of file");
                }
            }  // end while !found_nl_a !begof
        }// end if !found_nl_a !begof

        // may occur in files ending on a single newline
        debug_eprintln!("{}find_line C: line.count() is {}", so(), line.count());
        if line.count() == 0 {
            if self._find_line_lru_cache_enabled {
                debug_eprintln!("{}find_line C: LRU Cache put({}, Done)", so(), fileoffset);
                self._find_line_lru_cache
                    .put(fileoffset, ResultS4_LineFind::Done);
            }
            debug_eprintln!("{}find_line({}) C: return ResultS4_LineFind::Done;", sx(), fileoffset);
            return ResultS4_LineFind::Done;
        }

        debug_eprintln!("{}find_line D: return {:?};", so(), line);
        let fo_end = line.fileoffset_end();
        let lp = self.insert_line(line);
        if self._find_line_lru_cache_enabled {
            debug_eprintln!("{}find_line D: LRU Cache put({}, Found({}, …))", so(), fileoffset, fo_end + 1);
            self._find_line_lru_cache
                .put(fileoffset, ResultS4_LineFind::Found((fo_end + 1, lp.clone())));
        }
        debug_eprintln!(
            "{}find_line({}) D: return ResultS4_LineFind::Found(({}, @{:p})) @[{}, {}] {:?}",
            sx(),
            fileoffset,
            fo_end + 1,
            &*lp,
            (*lp).fileoffset_begin(),
            (*lp).fileoffset_end(),
            (*lp).to_String_noraw()
        );

        ResultS4_LineFind::Found((fo_end + 1, lp))
    }
}
