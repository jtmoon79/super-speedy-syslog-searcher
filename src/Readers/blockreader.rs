// Readers/blockreader.rs
//
// Blocks and BlockReader implemenations
//

pub use crate::common::{
    FPath,
    FileOffset,
};

use crate::common::{
    File,
    FileMetadata,
    FileOpenOptions,
    ResultS3,
    Bytes,
};

#[cfg(any(debug_assertions,test))]
use crate::dbgpr::printers::{
    byte_to_char_noraw,
};

use crate::dbgpr::stack::{
    sn,
    so,
    sx,
};

use std::collections::BTreeMap;
use std::fmt;
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom};
use std::io::prelude::Read;
use std::sync::Arc;

extern crate debug_print;
use debug_print::{debug_eprintln};

extern crate lru;
use lru::LruCache;

extern crate mime_guess;
use mime_guess::{
    MimeGuess,
};

extern crate more_asserts;
use more_asserts::{assert_le, assert_lt, assert_ge};

/// Block Size in bytes
pub type BlockSz = u64;
/// Byte offset (Index) _into_ a `Block` from beginning of `Block`. zero first.
pub type BlockIndex = usize;
/// Offset into a file in `Block`s, depends on `BlockSz` runtime value. zero first.
pub type BlockOffset = u64;
/// Block of bytes data read from some file storage
pub type Block = Vec<u8>;
/// thread-safe Atomic Reference Counting Pointer to a `Block`
pub type BlockP = Arc<Block>;

pub type Slices<'a> = Vec<&'a [u8]>;
/// tracker of `BlockP`
pub type Blocks = BTreeMap<BlockOffset, BlockP>;
pub type BlocksLRUCache = LruCache<BlockOffset, BlockP>;

// for case where reading blocks, lines, or syslines reaches end of file, the value `WriteZero` will
// be used here to mean "_end of file reached, nothing new_"
// XXX: this is a hack
//#[allow(non_upper_case_globals)]
//pub const EndOfFile: ErrorKind = ErrorKind::WriteZero;
#[allow(non_upper_case_globals)]
pub type ResultS3_ReadBlock = ResultS3<BlockP, Error>;
/// minimum Block Size (inclusive)
pub const BLOCKSZ_MIN: BlockSz = 1;
/// maximum Block Size (inclusive)
pub const BLOCKSZ_MAX: BlockSz = 0xFFFFFF;
/// default Block Size
//const BLOCKSZ_DEF: BlockSz = 0xFFFF;
#[allow(non_upper_case_globals)]
pub const BLOCKSZ_DEFs: &str = "0xFFFF";

/// Cached file reader that stores data in `BlockSz` byte-sized blocks.
/// A `BlockReader` corresponds to one file.
/// TODO: make a copy of `path`, no need to hold a reference, it just complicates things by introducing explicit lifetimes
pub struct BlockReader<'blockreader> {
    /// Path to file
    pub path: &'blockreader FPath,
    /// File handle, set in `open`
    file: Option<File>,
    /// File.metadata(), set in `open`
    file_metadata: Option<FileMetadata>,
    /// the `MimeGuess::from_path` result
    pub(crate) mimeguess: MimeGuess,
    /// File size in bytes, set in `open`
    pub(crate) filesz: u64,
    /// File size in blocks, set in `open`
    pub(crate) blockn: u64,
    /// BlockSz used for read operations
    pub blocksz: BlockSz,
    /// count of bytes stored by the `BlockReader`
    _count_bytes: u64,
    /// cached storage of blocks, looksups generally O(log2n)
    blocks: Blocks,
    /// internal LRU cache for `fn read_block`, lookups always O(1)
    /// XXX: but still... is `_read_block_lru_cache` accomplishing anything?
    _read_block_lru_cache: BlocksLRUCache,
    /// internal stats tracking
    pub(crate) _read_block_cache_lru_hit: u32,
    /// internal stats tracking
    pub(crate) _read_block_cache_lru_miss: u32,
    /// internal stats tracking
    pub(crate) _read_blocks_hit: u32,
    /// internal stats tracking
    pub(crate) _read_blocks_miss: u32,
}

impl fmt::Debug for BlockReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //let f_ = match &self.file_metadata {
        //    None => format!("None"),
        //    Some(val) => format!("{:?}", val.file_type()),
        //};
        f.debug_struct("BlockReader")
            .field("path", &self.path)
            .field("file", &self.file)
            //.field("file_metadata", &self.file_metadata)
            .field("mimeguess", &self.mimeguess)
            .field("filesz", &self.filesz)
            .field("blockn", &self.blockn)
            .field("blocksz", &self.blocksz)
            .field("count_bytes", &self._count_bytes)
            .field("blocks cached", &self.blocks.len())
            .field("cache LRU hit", &self._read_block_cache_lru_hit)
            .field("cache LRU miss", &self._read_block_cache_lru_miss)
            .field("cache hit", &self._read_blocks_hit)
            .field("cache miss", &self._read_blocks_miss)
            .finish()
    }
}

/// helper for humans debugging Blocks, very inefficient
#[allow(dead_code)]
#[cfg(any(debug_assertions,test))]
pub fn printblock(buffer: &Block, blockoffset: BlockOffset, fileoffset: FileOffset, blocksz: BlockSz, _mesg: String) {
    const LN: usize = 64;
    println!("╔════════════════════════════════════════════════════════════════════════════╕");
    println!(
        "║File block offset {:4}, byte offset {:4}, block length {:4} (0x{:04X}) (max {:4})",
        blockoffset,
        fileoffset,
        buffer.len(),
        buffer.len(),
        blocksz
    );
    println!("║          ┌────────────────────────────────────────────────────────────────┐");
    let mut done = false;
    let mut i = 0;
    let mut buf = Vec::<char>::with_capacity(LN);
    while i < buffer.len() && !done {
        buf.clear();
        for j in 0..LN {
            if i + j >= buffer.len() {
                done = true;
                break;
            };
            // print line number at beginning of line
            if j == 0 {
                let at: usize = i + j + ((blockoffset * blocksz) as usize);
                print!("║@0x{:06x} ", at);
            };
            let v = buffer[i + j];
            let cp = byte_to_char_noraw(v);
            buf.push(cp);
        }
        // done reading line, print buf
        i += LN;
        {
            //let s_: String = buf.into_iter().collect();
            let s_ = buf.iter().cloned().collect::<String>();
            println!("│{}│", s_);
        }
    }
    println!("╚══════════╧════════════════════════════════════════════════════════════════╛");
}

/// implement the BlockReader things
impl<'blockreader> BlockReader<'blockreader> {
    const READ_BLOCK_LRU_CACHE_SZ: usize = 4;
    /// create a new `BlockReader`
    pub fn new(path_: &'blockreader FPath, blocksz: BlockSz) -> BlockReader<'blockreader> {
        // TODO: why not open the file here? change `open` to a "static class wide" (or equivalent)
        //       that does not take a `self`. This would simplify some things about `BlockReader`
        // TODO: how to make some fields `blockn` `blocksz` `filesz` immutable?
        //       https://stackoverflow.com/questions/23743566/how-can-i-force-a-structs-field-to-always-be-immutable-in-rust
        assert_ne!(0, blocksz, "Block Size cannot be 0");
        assert_ge!(blocksz, BLOCKSZ_MIN, "Block Size too small");
        assert_le!(blocksz, BLOCKSZ_MAX, "Block Size too big");
        let p_ = std::path::Path::new(path_);
        let mg: MimeGuess = MimeGuess::from_path(p_);
        BlockReader {
            path: path_,
            file: None,
            file_metadata: None,
            mimeguess: mg,
            filesz: 0,
            blockn: 0,
            blocksz,
            _count_bytes: 0,
            blocks: Blocks::new(),
            _read_block_lru_cache: BlocksLRUCache::new(BlockReader::READ_BLOCK_LRU_CACHE_SZ),
            _read_block_cache_lru_hit: 0,
            _read_block_cache_lru_miss: 0,
            _read_blocks_hit: 0,
            _read_blocks_miss: 0,
        }
    }

    // TODO: make a `self` version of the following helpers that does not require
    //       passing `BlockSz`. Save the user some trouble.
    //       Can also `assert` that passed `FileOffset` is not larger than filesz, greater than zero.
    //       But keep the public static version available for testing.
    //       Change the LineReader calls to call `self.blockreader....`

    /// return preceding block offset at given file byte offset
    #[inline]
    pub fn block_offset_at_file_offset(file_offset: FileOffset, blocksz: BlockSz) -> BlockOffset {
        (file_offset / blocksz) as BlockOffset
    }

    /// return file_offset (byte offset) at given `BlockOffset`
    #[inline]
    pub fn file_offset_at_block_offset(block_offset: BlockOffset, blocksz: BlockSz) -> FileOffset {
        (block_offset * blocksz) as BlockOffset
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub fn file_offset_at_block_offset_index(
        blockoffset: BlockOffset, blocksz: BlockSz, blockindex: BlockIndex,
    ) -> FileOffset {
        assert_lt!(
            (blockindex as BlockSz),
            blocksz,
            "BlockIndex {} should not be greater or equal to BlockSz {}",
            blockindex,
            blocksz
        );
        BlockReader::file_offset_at_block_offset(blockoffset, blocksz) + (blockindex as FileOffset)
    }

    /// return block_index (byte offset into a `Block`) for `Block` that corresponds to `FileOffset`
    pub fn block_index_at_file_offset(file_offset: FileOffset, blocksz: BlockSz) -> BlockIndex {
        (file_offset
            - BlockReader::file_offset_at_block_offset(
                BlockReader::block_offset_at_file_offset(file_offset, blocksz),
                blocksz,
            )
        ) as BlockIndex
    }

    /// return count of blocks in a file
    #[inline]
    pub fn file_blocks_count(filesz: FileOffset, blocksz: BlockSz) -> u64 {
        filesz / blocksz + (if filesz % blocksz > 0 { 1 } else { 0 })
    }

    /// return block.len() for given block at `blockoffset`
    #[inline]
    pub fn blocklen_at_blockoffset(&self, blockoffset: &BlockOffset) -> usize {
        match self.blocks.get(blockoffset) {
            Some(blockp) => {
                blockp.len()
            }
            None => { panic!("bad blockoffset {}", blockoffset) }
        }
    }

    /// return last valid BlockIndex for block at `blockoffset
    #[inline]
    pub fn last_blockindex_at_blockoffset(&self, blockoffset: &BlockOffset) -> BlockIndex {
        (self.blocklen_at_blockoffset(blockoffset) - 1) as BlockIndex
    }

    /// last valid `BlockOffset` for the file (inclusive)
    #[inline]
    pub fn blockoffset_last(&self) -> BlockOffset {
        if self.filesz == 0 {
            return 0;
        }
        (BlockReader::file_blocks_count(self.filesz, self.blocksz) as BlockOffset) - 1
    }

    /// count of blocks stored by this `BlockReader` (during calls to `BlockReader::read_block`)
    #[inline]
    pub fn count(&self) -> u64 {
        self.blocks.len() as u64
    }

    /// count of bytes stored by this `BlockReader` (during calls to `BlockReader::read_block`)
    #[inline]
    pub fn count_bytes(&self) -> u64 {
        self._count_bytes
    }

    /// open the `self.path` file, set other field values after opening.
    /// propagates any `Err`, success returns `Ok(())`
    /// TODO: `open` should return a new `BlockReader`, this way field values of new BlockReader
    ///       are only set once, and are always accurate.
    pub fn open(&mut self) -> Result<()> {
        assert!(self.file.is_none(), "ERROR: the file is already open {:?}", &self.path);
        let mut open_options = FileOpenOptions::new();
        match open_options.read(true).open(&self.path) {
            Ok(val) => self.file = Some(val),
            Err(err) => {
                //eprintln!("ERROR: File::open({:?}) error {}", self.path, err);
                return Err(err);
            }
        };
        let file_ = self.file.as_ref().unwrap();
        match file_.metadata() {
            Ok(val) => {
                self.filesz = val.len();
                self.file_metadata = Some(val);
            }
            Err(err) => {
                eprintln!("ERROR: File::metadata() error {}", err);
                return Err(err);
            }
        };
        if self.file_metadata.as_ref().unwrap().is_dir() {
            return std::result::Result::Err(
                Error::new(
                    //ErrorKind::IsADirectory,  // XXX: error[E0658]: use of unstable library feature 'io_error_more'
                    ErrorKind::Unsupported,
                    format!("Path is a directory {:?}", self.path)
                )
            );
        }
        self.blockn = BlockReader::file_blocks_count(self.filesz, self.blocksz);
        self.blocks = Blocks::new();
        Ok(())
    }

    /// has block at `blockoffset` been read/processed?
    pub fn has_block(&self, blockoffset: &BlockOffset) -> bool {
        self.blocks.contains_key(blockoffset)
    }

    /// read a `Block` of data of max size `self.blocksz` from a prior `open`ed data source
    /// when successfully read returns `Found(BlockP)`
    /// when reached the end of the file, and no data was read returns `Done`
    /// all other `File` and `std::io` errors are propagated to the caller
    pub fn read_block(&mut self, blockoffset: BlockOffset) -> ResultS3_ReadBlock {
        debug_eprintln!("{}read_block: @{:p}.read_block({})", sn(), self, blockoffset);
        assert!(self.file.is_some(), "File has not been opened {:?}", self.path);
        { // check caches
            // check LRU cache
            match self._read_block_lru_cache.get(&blockoffset) {
                Some(bp) => {
                    self._read_block_cache_lru_hit += 1;
                    debug_eprintln!(
                        "{}read_block: return Found(BlockP@{:p}); hit LRU cache Block[{}] @[{}, {}) len {}",
                        sx(),
                        &*bp,
                        &blockoffset,
                        BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz),
                        BlockReader::file_offset_at_block_offset(blockoffset+1, self.blocksz),
                        (*bp).len(),
                    );
                    return ResultS3_ReadBlock::Found(bp.clone());
                }
                None => {
                    debug_eprintln!("{}read_block: blockoffset {} not found LRU cache", so(), blockoffset);
                    self._read_block_cache_lru_miss += 1;
                }
            }
            // check hash map cache
            if self.blocks.contains_key(&blockoffset) {
                debug_eprintln!("{}read_block: blocks.contains_key({})", so(), blockoffset);
                self._read_blocks_hit += 1;
                let bp: &BlockP = &self.blocks[&blockoffset];
                debug_eprintln!("{}read_block: LRU cache put({}, BlockP@{:p})", so(), blockoffset, bp);
                self._read_block_lru_cache.put(blockoffset, bp.clone());
                debug_eprintln!(
                    "{}read_block: return Found(BlockP@{:p}); cached Block[{}] @[{}, {}) len {}",
                    sx(),
                    &*self.blocks[&blockoffset],
                    &blockoffset,
                    BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz),
                    BlockReader::file_offset_at_block_offset(blockoffset+1, self.blocksz),
                    self.blocks[&blockoffset].len(),
                );
                return ResultS3_ReadBlock::Found(bp.clone());
            } else {
                self._read_blocks_miss += 1;
            }
        }
        let seek = (self.blocksz * blockoffset) as u64;
        let mut file_ = self.file.as_ref().unwrap();
        match file_.seek(SeekFrom::Start(seek)) {
            Ok(_) => {}
            Err(err) => {
                debug_eprintln!("{}read_block: return Err({})", sx(), err);
                eprintln!("ERROR: file.SeekFrom({}) Error {}", seek, err);
                return ResultS3_ReadBlock::Err(err);
            }
        };
        let mut reader = file_.take(self.blocksz as u64);
        // here is where the `Block` is created then set with data.
        // It should never change after this. Is there a way to mark it as "frozen"?
        // XXX: currently does not handle a partial read. From the docs (https://doc.rust-lang.org/std/io/trait.Read.html#method.read_to_end)
        //      > If any other read error is encountered then this function immediately returns. Any
        //      > bytes which have already been read will be appended to buf.
        let mut buffer = Block::with_capacity(self.blocksz as usize);
        debug_eprintln!("{}read_block: reader.read_to_end(@{:p})", so(), &buffer);
        match reader.read_to_end(&mut buffer) {
            Ok(val) => {
                if val == 0 {
                    debug_eprintln!(
                        "{}read_block: return Done blockoffset {} {:?}",
                        sx(),
                        blockoffset,
                        self.path
                    );
                    return ResultS3_ReadBlock::Done;
                }
            }
            Err(err) => {
                eprintln!("ERROR: reader.read_to_end(buffer) error {} for {:?}", err, self.path);
                debug_eprintln!("{}read_block: return Err({})", sx(), err);
                return ResultS3_ReadBlock::Err(err);
            }
        };
        let blen64 = buffer.len() as u64;
        let bp = BlockP::new(buffer);
        // store block
        debug_eprintln!("{}read_block: blocks.insert({}, BlockP@{:p})", so(), blockoffset, bp);
        #[allow(clippy::single_match)]
        match self.blocks.insert(blockoffset, bp.clone()) {
            Some(bp_) => {
                eprintln!("WARNING: blocks.insert({}, BlockP@{:p}) already had a entry BlockP@{:p}", blockoffset, bp, bp_);
            },
            _ => {},
        }
        self._count_bytes += blen64;
        // store in LRU cache
        debug_eprintln!("{}read_block: LRU cache put({}, BlockP@{:p})", so(), blockoffset, bp);
        self._read_block_lru_cache.put(blockoffset, bp.clone());
        debug_eprintln!(
            "{}read_block: return Found(BlockP@{:p}); new Block[{}] @[{}, {}) len {}",
            sx(),
            &*self.blocks[&blockoffset],
            &blockoffset,
            BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz),
            BlockReader::file_offset_at_block_offset(blockoffset+1, self.blocksz),
            (*self.blocks[&blockoffset]).len()
        );
        ResultS3_ReadBlock::Found(bp)
    }

    /// get byte at FileOffset
    /// `None` means the data at `FileOffset` was not available
    /// Does not request any `read_block`! Only copies from what is currently available from prior
    /// calls to `read_block`.
    /// debug helper only
    #[cfg(any(debug_assertions,bench,test))]
    fn _get_byte(&self, fo: FileOffset) -> Option<u8> {
        let bo = BlockReader::block_offset_at_file_offset(fo, self.blocksz);
        let bi = BlockReader::block_index_at_file_offset(fo, self.blocksz);
        if self.blocks.contains_key(&bo) {
            return Some((*self.blocks[&bo])[bi]);
        }

        None
    }

    /// return `Bytes` at `[fo_a, fo_b)`.
    /// uses `self._get_byte` which does not request any reads!
    /// debug helper only
    #[cfg(any(debug_assertions,bench,test))]
    pub(crate) fn _vec_from(&self, fo_a: FileOffset, fo_b: FileOffset) -> Bytes {
        assert_le!(fo_a, fo_b, "bad fo_a {} fo_b {} FPath {:?}", fo_a, fo_b, self.path);
        assert_le!(fo_b, self.filesz, "bad fo_b {} but filesz {} FPath {:?}", fo_b, self.filesz, self.path);
        if fo_a == fo_b {
            return Bytes::with_capacity(0);
        }
        let bo_a = BlockReader::block_offset_at_file_offset(fo_a, self.blocksz);
        let bo_b = BlockReader::block_offset_at_file_offset(fo_b, self.blocksz);
        let bo_a_i = BlockReader::block_index_at_file_offset(fo_a, self.blocksz);
        let bo_b_i = BlockReader::block_index_at_file_offset(fo_b, self.blocksz);
        if bo_a == bo_b {
            return Bytes::from(&(*self.blocks[&bo_a])[bo_a_i..bo_b_i]);
        }
        let mut fo_at = fo_a;
        let sz = (fo_b - fo_a) as usize;
        // XXX: inefficient!
        let mut vec_ = Bytes::with_capacity(sz);
        while fo_at < fo_b {
            let b = match self._get_byte(fo_at) {
                Some(val) => val,
                None => {
                    break;
                }
            };
            vec_.push(b);
            fo_at += 1;
        }

        vec_
    }
}
