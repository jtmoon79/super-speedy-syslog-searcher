// Readers/blockreader.rs
//
// Blocks and BlockReader implementations

pub use crate::common::{
    FPath,
    FileOffset,
    FileType,
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
    snx,
};

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::fmt;
use std::io::{
    BufReader,
    Error,
    ErrorKind,
    Result,
    Seek,
    SeekFrom,
};
use std::io::prelude::*;
use std::io::prelude::Read;
use std::sync::Arc;

extern crate debug_print;
use debug_print::{debug_eprintln};

extern crate lru;
use lru::LruCache;

extern crate mime_guess;
pub use mime_guess::{
    MimeGuess,
};

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    assert_ge,
    assert_gt,
};

extern crate flate2;
use flate2::read::{
    GzDecoder,
};
use flate2::GzHeader;

// -------------------------------------------------------------------------------------------------

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
/// tracker of `BlockOffset` that have been rad
pub type BlocksTracked = BTreeSet<BlockOffset>;
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

/// data and readers for a gzipped file
#[derive(Debug)]
pub struct GzData {
    /// size of file uncompressed, taken from trailing gzip file data
    pub filesz: u64,
    /// calls to `read` use this
    pub decoder: GzDecoder<File>,
    /// filename taken from gzip header
    pub filename: String,
    /// file mtime taken from gzip header
    pub mtime: u32,
    /// CRC32 taken from trailing gzip file data
    pub crc32: u32,
}

/// A `BlockReader` reads a file in `BlockSz` byte-sized `Block`s. It interfaces 
/// with the filesystem (or any other data retreival method). It handles the
/// lookup and storage of `Block`s of data.
///
/// A `BlockReader` uses it's `FileType` to determine how to handle files.
/// This includes reading bytes from files (e.g. `.log`),
/// compressed files (e.g. `.gz`), and archive files (e.g. `.tar`).
///
/// One `BlockReader` corresponds to one file. For archive files, one `BlockReader`
/// handles only one file *within* the archive file.
///
/// A `BlockReader` does not know about `char`s (a `LineReader` does).
///
/// XXX: not a rust "Reader"; does not implement trait `Read`
///
pub struct BlockReader {
    /// Path to file
    pub path: FPath,
    /// File handle
    file: File,
    /// File.metadata()
    /// For compressed or archived files, the metadata of the `path`
    /// compress or archive file.
    file_metadata: FileMetadata,
    /// The `MimeGuess::from_path` result
    mimeguess_: MimeGuess,
    /// enum that guides file-handling behavior in `read`, `new`
    filetype: FileType,
    /// For gzipped files (FileType::FILE_GZ), otherwise `None`
    gz: Option<GzData>,
    /// The filesz of uncompressed data, set during `new`.
    /// Should always be `== gz.unwrap().filesz`.
    ///
    /// Users should always call `filesz()`.
    pub(crate) filesz_actual: u64,
    /// File size in bytes of file at `path`, actual size.
    /// For compressed files, this is the size of the file compressed.
    /// For the uncompressed size of a compressed file, see `filesz_actual`.
    /// Set in `open`.
    ///
    /// For regular files (not compressed or archived),
    /// `filesz` and `filesz_actual` will be the same.
    ///
    /// Users should always call `filesz()`.
    pub(crate) filesz: u64,
    /// File size in blocks, set in `open`
    pub(crate) blockn: u64,
    /// BlockSz used for read operations.
    pub(crate) blocksz: BlockSz,
    /// Count of bytes stored by the `BlockReader`.
    /// May not match `self.blocks.iter().map(|x| sum += x.len()); sum` as
    /// `self.blocks` may have some elements `drop`ped during streaming.
    count_bytes_: u64,
    /// Storage of blocks `read` from storage. Lookups O(log(n)).
    ///
    /// During file processing, some elements that are not needed may be `drop`ped.
    blocks: Blocks,
    /// track blocks read in `read_block`. Never drops data.
    ///
    /// useful for when streaming kicks-in and some key+vale of `self.blocks` have
    /// been dropped.
    blocks_read: BlocksTracked,
    /// internal LRU cache for `fn read_block`. Lookups O(1).
    _read_block_lru_cache: BlocksLRUCache,
    /// enable/disable use of `_read_block_lru_cache`
    _read_block_lru_cache_enabled: bool,
    /// internal stats tracking
    pub(crate) _read_block_cache_lru_hit: u64,
    /// internal stats tracking
    pub(crate) _read_block_cache_lru_miss: u64,
    /// internal stats tracking
    pub(crate) _read_block_cache_lru_put: u64,
    /// internal stats tracking
    pub(crate) _read_blocks_hit: u64,
    /// internal stats tracking
    pub(crate) _read_blocks_miss: u64,
    /// internal stats tracking
    pub(crate) _read_blocks_insert: u64,
}

impl fmt::Debug for BlockReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //let f_ = match &self.file_metadata {
        //    None => format!("None"),
        //    Some(val) => format!("{:?}", val.file_type()),
        //};
        f.debug_struct("BlockReader")
            .field("path", &self.path)
            .field("file", &self.file)
            //.field("file_metadata", &self.file_metadata)
            .field("mimeguess", &self.mimeguess_)
            .field("filesz", &self.filesz())
            .field("blockn", &self.blockn)
            .field("blocksz", &self.blocksz)
            .field("blocks currently stored", &self.blocks.len())
            .field("blocks read", &self.blocks_read.len())
            .field("bytes read", &self.count_bytes_)
            .field("cache LRU hit", &self._read_block_cache_lru_hit)
            .field("miss", &self._read_block_cache_lru_miss)
            .field("put", &self._read_block_cache_lru_put)
            .field("cache hit", &self._read_blocks_hit)
            .field("miss", &self._read_blocks_miss)
            .field("insert", &self._read_blocks_insert)
            .finish()
    }
}

/// helper to unpack DWORD unsigned integers in a gzip header
fn dword_to_u32(buf: &[u8; 4]) -> u32 {
    let mut buf_: [u8; 4] = [0; 4];
    buf_[0] = buf[3];
    buf_[1] = buf[2];
    buf_[2] = buf[1];
    buf_[3] = buf[0];

    u32::from_be_bytes(buf_)
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
impl BlockReader {
    /// maximum size of a gzip compressed file that will be processed.
    ///
    /// XXX: The gzip standard stores uncompressed "media stream" bytes size in within
    ///      32 bits, 4 bytes. A larger uncompressed size 0xFFFFFFFF will store the modulo.
    ///      So there is no certain way to determine the size of the "media stream".
    ///      This terrible hack just aborts processing .gz files that might be over that
    ///      size.
    const GZ_MAX_SZ: u64 = 0x20000000;
    /// cache slots for `read_block` LRU cache
    const READ_BLOCK_LRU_CACHE_SZ: usize = 4;

    /// Create a new `BlockReader`.
    ///
    /// Opens the `path` file, configures settings based on determined `filetype`.
    pub fn new(path: FPath, filetype: FileType, blocksz_: BlockSz) -> Result<BlockReader> {
        // TODO: why not open the file here? change `open` to a "static class wide" (or equivalent)
        //       that does not take a `self`. This would simplify some things about `BlockReader`
        // TODO: how to make some fields `blockn` `blocksz` `filesz` immutable?
        //       https://stackoverflow.com/questions/23743566/how-can-i-force-a-structs-field-to-always-be-immutable-in-rust
        debug_eprintln!("{}BlockReader::new({:?}, {:?}, {:?})", sn(), path, filetype, blocksz_);

        assert_ne!(0, blocksz_, "Block Size cannot be 0");
        assert_ge!(blocksz_, BLOCKSZ_MIN, "Block Size {} is too small", blocksz_);
        assert_le!(blocksz_, BLOCKSZ_MAX, "Block Size {} is too big", blocksz_);
        let path_std = std::path::Path::new(&path);
        let mimeguess_: MimeGuess = MimeGuess::from_path(path_std);

        let mut open_options = FileOpenOptions::new();
        debug_eprintln!("{}BlockReader::new: open_options.read(true).open({:?})", so(), path);
        let file: File = match open_options.read(true).open(path_std) {
            Ok(val) => val,
            Err(err) => {
                //eprintln!("ERROR: File::open({:?}) error {}", path, err);
                debug_eprintln!("{}BlockReader::new: return {:?}", sx(), err);
                return Err(err);
            }
        };
        let filesz: u64;
        let filesz_actual: u64;
        let blocksz: BlockSz;
        let blockn: u64;
        let file_metadata: FileMetadata;
        let gz_opt: Option<GzData>;
        match file.metadata() {
            Ok(val) => {
                filesz = val.len();
                file_metadata = val;
            }
            Err(err) => {
                debug_eprintln!("{}BlockReader::new: return {:?}", sx(), err);
                eprintln!("ERROR: File::metadata() error {}", err);
                return Err(err);
            }
        };
        if file_metadata.is_dir() {
            debug_eprintln!("{}BlockReader::new: return Err(Unsupported)", sx());
            return std::result::Result::Err(
                Error::new(
                    //ErrorKind::IsADirectory,  // XXX: error[E0658]: use of unstable library feature 'io_error_more'
                    ErrorKind::Unsupported,
                    format!("Path is a directory {:?}", path)
                )
            );
        }
        match filetype {
            FileType::FILE => {
                filesz_actual = filesz;
                blocksz = blocksz_;
                blockn = BlockReader::file_blocks_count(filesz, blocksz);
                gz_opt = None;
            },
            FileType::FILE_GZ => {
                blocksz = blocksz_;
                debug_eprintln!("{0}BlockReader::new: blocksz set to {1} (0x{1:08X}) (passed {2} (0x{2:08X})", so(), blocksz, blocksz_);

                // GZIP last 8 bytes:
                //    4 bytes (DWORD) is CRC32
                //    4 bytes (DWORD) is gzip file uncompressed size
                // GZIP binary format https://datatracker.ietf.org/doc/html/rfc1952#page-5
                //
                // +---+---+---+---+---+---+---+---+
                // |     CRC32     |      SIZE     |
                // +---+---+---+---+---+---+---+---+
                //

                // sanity check file size
                if filesz < 8 {
                    debug_eprintln!("{}BlockReader::new: return Err(InvalidData)", sx());
                    return Result::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!("gzip file size {:?} is too small for {:?}", filesz, path)
                        )
                    );
                }
                // TODO: [2022/06] it's known that for a file larger than 4GB uncompressed,
                //       gzip cannot store it's filesz accurately, since filesz is stored within 32 bits.
                //       gzip will only store the rollover (uncompressed filesz % 4GB).
                //       How to handle large gzipped files correctly?
                //       First, how to detect that the stored filesz is a rollover value?
                //       Second, the file could be streamed and the filesz calculated from that
                //       activity. However, streaming, for example, a 3GB log.gz that uncompresses to
                //       10GB is very inefficient.
                //       Third, similar to "Second" but for very large files, i.e. a 32GB log.gz file, what then?
                if filesz > BlockReader::GZ_MAX_SZ {
                    debug_eprintln!("{}BlockReader::new: return Err(InvalidData)", sx());
                    return Result::Err(
                        Error::new(
                            // TODO: [2022/06] use `ErrorKind::FileTooLarge` when it is stable
                            //       `ErrorKind::FileTooLarge` causes error:
                            //       use of unstable library feature 'io_error_more'
                            //       see issue #86442 <https://github.com/rust-lang/rust/issues/86442> for more informationrustc(E0658)
                            ErrorKind::InvalidData,
                            format!("Cannot handle gzip files larger than semi-arbitrary {0} (0x{0:08X}) uncompressed bytes, file is {1} (0x{1:08X}) uncompressed bytes according to gzip header {2:?}", BlockReader::GZ_MAX_SZ, filesz, path),
                        )
                    );
                }

                // create "take handler" that will read 8 bytes as-is (no decompression)
                match (&file).seek(SeekFrom::End(-8)) {
                    Ok(_) => {},
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: return Err({})", sx(), err);
                        eprintln!("ERROR: file.SeekFrom(-8) Error {}", err);
                        return Err(err);
                    },
                };
                let mut reader = (&file).take(8);

                // extract DWORD for CRC32
                let mut buffer_crc32: [u8; 4] = [0; 4];
                debug_eprintln!("{}BlockReader::new: reader.read_exact(@{:p}) (buffer len {})", so(), &buffer_crc32, buffer_crc32.len());
                match reader.read_exact(&mut buffer_crc32) {
                    Ok(_) => {},
                    //Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {},
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: return {:?}", sx(), err);
                        eprintln!("reader.read_to_end(&buffer_crc32) Error {:?}", err);
                        return Err(err);
                    },
                }
                debug_eprintln!("{}BlockReader::new: buffer_crc32 {:?}", so(), buffer_crc32);
                let crc32 = dword_to_u32(&buffer_crc32);
                debug_eprintln!("{}BlockReader::new: crc32 {} (0x{:08X})", so(), crc32, crc32);

                // extract DWORD for SIZE
                let mut buffer_size: [u8; 4] = [0; 4];
                debug_eprintln!("{}BlockReader::new: reader.read_exact(@{:p}) (buffer len {})", so(), &buffer_size, buffer_size.len());
                match reader.read_exact(&mut buffer_size) {
                    Ok(_) => {},
                    Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {},
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: return {:?}", sx(), err);
                        eprintln!("reader.read_to_end(&buffer_size) Error {:?}", err);
                        return Err(err);
                    },
                }
                debug_eprintln!("{}BlockReader::new: buffer_size {:?}", so(), buffer_size);
                let size: u32 = dword_to_u32(&buffer_size);
                debug_eprintln!("{}BlockReader::new: file size uncompressed {:?} (0x{:08X})", so(), size, size);
                let filesz_uncompressed: u64 = size as u64;
                if filesz_uncompressed == 0 {
                    debug_eprintln!("{}BlockReader::new: return Err(InvalidData)", sx());
                    return Result::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!("extracted uncompressed file size value 0, nothing to read {:?}", path),
                        )
                    );
                }
                filesz_actual = filesz_uncompressed;
                blockn = BlockReader::file_blocks_count(filesz, blocksz);

                //let mut open_options = FileOpenOptions::new();
                debug_eprintln!("{}BlockReader::new: open_options.read(true).open({:?})", so(), path_std);
                let file_gz: File = match open_options.read(true).open(path_std) {
                    Ok(val) => val,
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: open_options.read({:?}) Error, return {:?}", sx(), path, err);
                        return Err(err);
                    }
                };
                let decoder: GzDecoder<File> = GzDecoder::new(file_gz);
                debug_eprintln!("{}BlockReader::new: {:?}", so(), decoder);
                let header_opt: Option<&GzHeader> = decoder.header();
                let mut filename: String = String::with_capacity(0);
                let mut mtime: u32 = 0;
                match header_opt {
                    Some(header) => {
                        let filename_: &[u8] = header.filename().unwrap_or(&[]);
                        filename = match String::from_utf8(filename_.to_vec()) {
                            Ok(val) => val,
                            Err(_err) => String::with_capacity(0),
                        };
                        mtime = header.mtime();
                    },
                    None => {
                        debug_eprintln!("{}BlockReader::new: GzDecoder::header() is None for {:?}", so(), path);
                    },
                };

                gz_opt = Some(
                    GzData {
                        filesz: filesz_uncompressed,
                        decoder,
                        filename,
                        mtime,
                        crc32,
                    }
                );
                debug_eprintln!("{}BlockReader::new: created {:?}", so(), gz_opt);
            },
            _ => {
                return Result::Err(
                    Error::new(
                        ErrorKind::Unsupported,
                        format!("Unsupported FileType {:?}", filetype)
                    )
                );
            }
        }

        // XXX: don't assert on `filesz` vs `filesz_actual`; they can be either gt, lt, or eq.

        debug_eprintln!("{}BlockReader::new: return Ok(BlockReader)", sx());

        Ok(
            BlockReader {
                path,
                file,
                file_metadata,
                mimeguess_,
                filetype,
                gz: gz_opt,
                filesz,
                filesz_actual,
                blockn,
                blocksz,
                count_bytes_: 0,
                blocks: Blocks::new(),
                blocks_read: BlocksTracked::new(),
                _read_block_lru_cache: BlocksLRUCache::new(BlockReader::READ_BLOCK_LRU_CACHE_SZ),
                _read_block_lru_cache_enabled: true,
                _read_block_cache_lru_hit: 0,
                _read_block_cache_lru_miss: 0,
                _read_block_cache_lru_put: 0,
                _read_blocks_hit: 0,
                _read_blocks_miss: 0,
                _read_blocks_insert: 0,
            }
        )
    }

    #[inline(always)]
    pub const fn mimeguess(&self) -> MimeGuess {
        self.mimeguess_
    }

    pub const fn filesz(&self) -> u64 {
        match self.filetype {
            FileType::FILE_GZ => {
                self.filesz_actual
            },
            FileType::FILE => {
                self.filesz
            },
            _ => {
                0
            },
        }
    }

    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        self.filetype
    }

    // TODO: make a `self` version of the following helpers that does not require
    //       passing `BlockSz`. Save the user some trouble.
    //       Can also `assert` that passed `FileOffset` is not larger than filesz, greater than zero.
    //       But keep the public static version available for testing.
    //       Change the LineReader calls to call `self.blockreader....`

    /// return preceding block offset at given file byte offset
    #[inline(always)]
    pub const fn block_offset_at_file_offset(file_offset: FileOffset, blocksz: BlockSz) -> BlockOffset {
        (file_offset / blocksz) as BlockOffset
    }

    /// return file_offset (byte offset) at given `BlockOffset`
    #[inline(always)]
    pub const fn file_offset_at_block_offset(block_offset: BlockOffset, blocksz: BlockSz) -> FileOffset {
        (block_offset * blocksz) as BlockOffset
    }

    /// return file_offset (byte offset) at given `BlockOffset`
    #[inline(always)]
    pub const fn file_offset_at_block_offset_self(&self, block_offset: BlockOffset) -> FileOffset {
        (block_offset * self.blocksz) as BlockOffset
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    #[inline(always)]
    pub const fn file_offset_at_block_offset_index(
        blockoffset: BlockOffset, blocksz: BlockSz, blockindex: BlockIndex,
    ) -> FileOffset {
        /*
        assert_lt!(
            (blockindex as BlockSz),
            blocksz,
            "BlockIndex {} should not be greater or equal to BlockSz {}",
            blockindex,
            blocksz
        );
        */
        BlockReader::file_offset_at_block_offset(blockoffset, blocksz) + (blockindex as FileOffset)
    }

    /// return block_index (byte offset into a `Block`) for `Block` that corresponds to `FileOffset`
    #[inline(always)]
    pub const fn block_index_at_file_offset(file_offset: FileOffset, blocksz: BlockSz) -> BlockIndex {
        (file_offset
            - BlockReader::file_offset_at_block_offset(
                BlockReader::block_offset_at_file_offset(file_offset, blocksz),
                blocksz,
            )
        ) as BlockIndex
    }

    /// return count of blocks in a file
    #[inline(always)]
    pub const fn file_blocks_count(filesz: FileOffset, blocksz: BlockSz) -> u64 {
        filesz / blocksz + (if filesz % blocksz > 0 { 1 } else { 0 })
    }

    /// specific `BlockSz` (length) of block at `blockoffset`
    pub fn blocksz_at_blockoffset(&self, blockoffset: &BlockOffset) -> Option<BlockSz> {
        debug_eprintln!("{}blockreader.blocksz_at_blockoffset({})", snx(), blockoffset);

        match self.blocks.get(blockoffset) {
            Some(blockp) => Some((*blockp).len() as BlockSz),
            None => None,
        }
    }

    /// return block.len() for given block at `blockoffset`
    /// TODO: replace all uses of this `blocklen_at_blockoffset` with `blocksz_at_blockoffset`
    #[inline(always)]
    pub fn blocklen_at_blockoffset(&self, blockoffset: &BlockOffset) -> usize {
        match self.blocks.get(blockoffset) {
            Some(blockp) => {
                blockp.len()
            }
            None => { panic!("bad blockoffset {}", blockoffset) }
        }
    }

    /// return last valid BlockIndex for block at `blockoffset
    #[inline(always)]
    pub fn last_blockindex_at_blockoffset(&self, blockoffset: &BlockOffset) -> BlockIndex {
        (self.blocklen_at_blockoffset(blockoffset) - 1) as BlockIndex
    }

    /// last valid `BlockOffset` for the file (inclusive)
    #[inline(always)]
    pub const fn blockoffset_last(&self) -> BlockOffset {
        if self.filesz() == 0 {
            return 0;
        }
        (BlockReader::file_blocks_count(self.filesz(), self.blocksz) as BlockOffset) - 1
    }

    /// count of blocks stored by this `BlockReader` (adjusted during calls to `BlockReader::read_block`)
    /// not the same as `self.blocks_read` or `self.count_bytes_`
    #[inline(always)]
    pub fn count_blocks(&self) -> u64 {
        self.blocks.len() as u64
    }

    /// count of bytes stored by this `BlockReader` (adjusted during calls to `BlockReader::read_block`)
    #[inline(always)]
    pub fn count_bytes(&self) -> u64 {
        self.count_bytes_
    }

    /// has block at `blockoffset` been read/processed?
    ///
    /// during streaming, some key+value may be dropped from `self.blocks`
    pub fn has_block(&self, blockoffset: &BlockOffset) -> bool {
        self.blocks.contains_key(blockoffset)
    }

    /// enable internal LRU cache used by `read_block`.
    ///
    /// intended to aid testing and debugging
    pub fn LRU_cache_enable(&mut self) {
        if self._read_block_lru_cache_enabled {
            return;
        }
        self._read_block_lru_cache_enabled = true;
        self._read_block_lru_cache.clear();
        self._read_block_lru_cache.resize(BlockReader::READ_BLOCK_LRU_CACHE_SZ);
    }

    /// disable internal LRU cache used by `read_block`.
    ///
    /// intended to aid testing and debugging
    pub fn LRU_cache_disable(&mut self) {
        self._read_block_lru_cache_enabled = false;
        self._read_block_lru_cache.resize(0);
    }

    /// Drop data associated with `Block` at `blockoffset`.
    ///
    /// Presumes the caller knows what they are doing!
    pub fn drop_block(&mut self, blockoffset: BlockOffset, bo_dropped: &mut HashSet<BlockOffset>) {
        if bo_dropped.contains(&blockoffset) {
            return;
        }
        match self.blocks.remove(&blockoffset) {
            Some(blockp) => {
                let sc = Arc::strong_count(&blockp);
                debug_eprintln!("{}blockreader.drop_block({}): dropped block {} @0x{:p}, len {}, strong_count {}", so(), blockoffset, blockoffset, blockp, (*blockp).len(), sc);
                bo_dropped.insert(blockoffset);
            },
            None => {
                debug_eprintln!("{}blockreader.drop_block({}): no block to drop at {}", so(), blockoffset, blockoffset);
            },
        }
        match self._read_block_lru_cache.pop(&blockoffset) {
            Some(blockp) => {
                let sc = Arc::strong_count(&blockp);
                debug_eprintln!("{}blockreader.drop_block({}): dropped block in LRU cache {} @0x{:p}, len {}, strong_count {}", so(), blockoffset, blockoffset, blockp, (*blockp).len(), sc);
                bo_dropped.insert(blockoffset);
            },
            None => {
                debug_eprintln!("{}blockreader.drop_block({}): no block in LRU cache to drop at {}", so(), blockoffset, blockoffset);
            }
        }
    }

    /// store copy of `BlockP` in LRU cache
    fn store_block_in_LRU_cache(&mut self, blockoffset: BlockOffset, blockp: &BlockP) {
        debug_eprintln!("{}store_block_in_LRU_cache: LRU cache put({}, BlockP@{:p})", so(), blockoffset, blockp);
        if ! self._read_block_lru_cache_enabled {
            return;
        }
        self._read_block_lru_cache.put(blockoffset, blockp.clone());
        self._read_block_cache_lru_put += 1;
    }

    fn store_block_in_storage(&mut self, blockoffset: BlockOffset, blockp: &BlockP) {
        // store block
        debug_eprintln!("{}store_block_in_storage: blocks.insert({}, BlockP@{:p} (len {}, capacity {}))", snx(), blockoffset, blockp, (*blockp).len(), (*blockp).capacity());
        #[allow(clippy::single_match)]
        match self.blocks.insert(blockoffset, blockp.clone()) {
            Some(bp_) => {
                eprintln!("WARNING: blockreader.blocks.insert({}, BlockP@{:p}) already had a entry BlockP@{:p}", blockoffset, blockp, bp_);
            },
            _ => {},
        }
        self._read_blocks_insert += 1;
        self.count_bytes_ += (*blockp).len() as u64;
        if let false = self.blocks_read.insert(blockoffset) {
            eprintln!("WARNING: blockreader.blocks_read({}) already had a entry", blockoffset);
        }
    }

    /// read a block of data from storage for a normal file.
    ///
    /// called from `read_block`
    fn read_block_FILE(&mut self, blockoffset: BlockOffset) -> ResultS3_ReadBlock {
        debug_eprintln!("{}read_block_FILE({})", sn(), blockoffset);
        assert_eq!(self.filetype, FileType::FILE, "wrong FileType {:?} for calling read_block_FILE", self.filetype);

        let mut buffer = Block::with_capacity(self.blocksz as usize);
        let seek = (self.blocksz * blockoffset) as u64;
        match self.file.seek(SeekFrom::Start(seek)) {
            Ok(_) => {},
            Err(err) => {
                eprintln!("ERROR: file.SeekFrom(Start({})) Error {}", seek, err);
                debug_eprintln!("{}read_block_FILE({}): return Err({})", sx(), blockoffset, err);
                return ResultS3_ReadBlock::Err(err);
            },
        };
        let mut reader = (&self.file).take(self.blocksz as u64);
        // here is where the `Block` is created then set with data.
        // It should never change after this. Is there a way to mark it as "frozen"?
        // XXX: currently does not handle a partial read. From the docs (https://doc.rust-lang.org/std/io/trait.Read.html#method.read_to_end)
        //      > If any other read error is encountered then this function immediately returns. Any
        //      > bytes which have already been read will be appended to buf.
        // TODO: change to `read_exact` which recently stabilized?
        debug_eprintln!("{}read_block_FILE({}): reader.read_to_end(@{:p})", so(), blockoffset, &buffer);
        match reader.read_to_end(&mut buffer) {
            Ok(val) => {
                if val == 0 {
                    debug_eprintln!("{}read_block_FILE({}): return Done for {:?}", sx(), blockoffset, self.path);
                    return ResultS3_ReadBlock::Done;
                }
            },
            Err(err) => {
                eprintln!("ERROR: reader.read_to_end(buffer) error {} for {:?}", err, self.path);
                debug_eprintln!("{}read_block_FILE({}): return Err({})", sx(), blockoffset, err);
                return ResultS3_ReadBlock::Err(err);
            },
        };
        let blockp: BlockP = BlockP::new(buffer);
        self.store_block_in_storage(blockoffset, &blockp);
        debug_eprintln!("{}read_block_FILE({}): return Found", sx(), blockoffset);

        ResultS3_ReadBlock::Found(blockp)
    }

    /// read a block of data from storage for a compressed gzip file.
    /// `blockoffset` refers to the uncompressed version of the file.
    ///
    /// called from `read_block`
    fn read_block_FILE_GZ(&mut self, blockoffset: BlockOffset) -> ResultS3_ReadBlock {
        debug_eprintln!("{}read_block_FILE_GZ({})", sn(), blockoffset);
        assert_eq!(self.filetype, FileType::FILE_GZ, "wrong FileType {:?} for calling read_block_FILE_GZ", self.filetype);

        let blockoffset_last: BlockOffset = self.blockoffset_last();
        let mut bo_at: BlockOffset = match self.blocks_read.iter().max() {
            Some(bo_) => *bo_,
            None => 0,
        };
        // read entire file up to `blockoffset`, storing each decompressed block
        while bo_at <= blockoffset {
            // check `self.blocks_read` (not `self.blocks`) if the Block at `blockoffset`
            // was *ever* read.
            // TODO: [2022/06/18] add another stat tracker for lookups in `self.blocks_read`
            if self.blocks_read.contains(&bo_at) {
                self._read_blocks_hit += 1;
                //debug_eprintln!("{}read_block_FILE_GZ({}): blocks_read.contains({})", so(), blockoffset, bo_at);
                if bo_at == blockoffset {
                    debug_eprintln!("{}read_block_FILE_GZ({}): return Found", sx(), blockoffset);
                    // XXX: this will panic if the key+value in `self.blocks` was dropped
                    //      which could happen during streaming stage
                    let blockp: BlockP = self.blocks.get_mut(&bo_at).unwrap().clone();
                    self.store_block_in_LRU_cache(blockoffset, &blockp);
                    return ResultS3_ReadBlock::Found(blockp);
                }
                bo_at += 1;
                continue;
            } else {
                debug_eprintln!("{}read_block_FILE_GZ({}): blocks_read.contains({}) missed (does not contain key)", so(), blockoffset, bo_at);
                debug_assert!(!self.blocks.contains_key(&bo_at), "blocks has element {} not in blocks_read", bo_at);
                self._read_blocks_miss += 1;
            }

            let blocksz_u: usize = self.blocksz as usize;
            let mut block = Block::with_capacity(blocksz_u);
            // XXX: `with_capacity, clear, resize` is a verbose way to create a new vector with a run-time determined `capacity`
            //      and `len`. `len == capacity` is necessary for calls to `decoder.read`.
            //      Using `decoder.read_exact` and `decoder.read_to_end` was more difficult.
            //      See https://github.com/rust-lang/flate2-rs/issues/308
            block.clear();
            block.resize(blocksz_u, 0);
            debug_eprintln!("{}read_block_FILE_GZ({}): blocks_read count {:?}", so(), blockoffset, self.blocks_read.len());
            debug_eprintln!("{}read_block_FILE_GZ({}): GzDecoder.read(@{:p} (len {}, capacity {}))", so(), blockoffset, &block, block.len(), block.capacity());
            match (self.gz.as_mut().unwrap().decoder).read(block.as_mut()) {
                Ok(size_) if size_ == 0 => {
                    debug_eprintln!("{}read_block_FILE_GZ({}): GzDecoder.read() returned Ok({:?})", so(), blockoffset, size_);
                    let byte_at: FileOffset = self.file_offset_at_block_offset_self(blockoffset);
                    return ResultS3_ReadBlock::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!("GzDecoder.read() returned zero bytes for vec<u8> buffer of length {}, capacity {}; stuck at inflated byte {}, file {:?} size {}, size uncompressed {} (calculated from gzip header)", block.len(), block.capacity(), byte_at, self.path, self.filesz, self.filesz_actual)
                        )
                    );
                },
                Ok(size_) if size_ == blocksz_u => {
                    debug_eprintln!("{}read_block_FILE_GZ({}): GzDecoder.read() returned Ok({:?})", so(), blockoffset, size_);
                    block.resize(size_, 0);
                }
                Ok(size_) => {
                    debug_eprintln!("{}read_block_FILE_GZ({}): GzDecoder.read() returned Ok({:?}), blocksz {}", so(), blockoffset, size_, blocksz_u);
                    block.resize(size_, 0);
                },
                //Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                //    // XXX: docs say the value of `bytes` is not guaranteed, but in practice
                //    //      the `bytes` are set to remaining data. Also, further calls to any `read`
                //    //      fail to return anything.
                //    debug_eprintln!("{}read_block_FILE_GZ({}): read() returned Err({})", sx(), blockoffset, err);
                //},
                Err(err) => {
                    debug_eprintln!("ERROR: GzDecoder.read(&block (capacity {})) error {} for {:?}", self.blocksz, err, self.path);
                    debug_eprintln!("{}read_block_FILE_GZ({}): return Err({})", sx(), blockoffset, err);
                    return ResultS3_ReadBlock::Err(err);
                }
            }

            // check returned Block is expected number of bytes
            let blocklen_sz: BlockSz = block.len() as BlockSz;
            if block.is_empty() {
                let byte_at = self.file_offset_at_block_offset_self(blockoffset);
                return ResultS3_ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                            "GzDecoder.read() zero bytes from block {} (byte {}), filesz {}, filesz uncompressed {} (according to gzip header), {:?}",
                            bo_at, byte_at, self.filesz, self.filesz_actual, self.path,
                        )
                    )
                );
            } else if bo_at == blockoffset_last {
                // last block, is blocksz correct?
                if blocklen_sz > self.blocksz {
                    let byte_at = self.file_offset_at_block_offset_self(blockoffset);
                    return ResultS3_ReadBlock::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "GzDecoder.read read {} bytes for block {} (byte {}) which is larger than blocksz {} for {:?}",
                                blocklen_sz, bo_at, byte_at, self.blocksz, self.path,
                            )
                        )
                    );            
                }
            } else if blocklen_sz != self.blocksz {
                // not last block, is blocksz correct?
                let byte_at = self.file_offset_at_block_offset_self(blockoffset);
                return ResultS3_ReadBlock::Err(
                    Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "GzDecoder.read read {} bytes for block {} (byte {}) expected to read blocksz {} bytes, blockoffset last {}, filesz {}, filesz uncompressed {} (according to gzip header), for {:?}",
                            blocklen_sz, bo_at, byte_at, self.blocksz, blockoffset_last, self.filesz, self.filesz_actual, self.path,
                        )
                    )
                );
            }

            // store decompressed block
            let blockp = BlockP::new(block);
            self.store_block_in_storage(blockoffset, &blockp);
            if bo_at == blockoffset {
                debug_eprintln!("{}read_block_FILE_GZ({}): return Found", sx(), blockoffset);
                return ResultS3_ReadBlock::Found(blockp);
            }
            bo_at += 1;
        }
        debug_eprintln!("{}read_block_FILE_GZ({}): return Done", sx(), blockoffset);

        ResultS3_ReadBlock::Done
    }

    /// read a `Block` of data of max size `self.blocksz` from a prior `open`ed data source
    /// when successfully read returns `Found(BlockP)`
    /// when reached the end of the file, and no data was read returns `Done`
    /// all other `File` and `std::io` errors are propagated to the caller
    pub fn read_block(&mut self, blockoffset: BlockOffset) -> ResultS3_ReadBlock {
        debug_eprintln!(
            "{0}read_block: @{1:p}.read_block({2}) (fileoffset {3} (0x{3:08X})), blocksz {4} (0x{4:08X}), filesz {5} (0x{5:08X})",
            sn(), self, blockoffset, self.file_offset_at_block_offset_self(blockoffset), self.blocksz, self.filesz(),
        );
        { // check storages
            // check fast LRU cache
            if self._read_block_lru_cache_enabled {
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
                        self._read_block_cache_lru_miss += 1;
                        debug_eprintln!("{}read_block: blockoffset {} not found LRU cache", so(), blockoffset);
                    }
                }
            }
            // check hash map storage
            if self.blocks_read.contains(&blockoffset) {
                self._read_blocks_hit += 1;
                debug_eprintln!("{}read_block: blocks_read.contains({})", so(), blockoffset);
                assert!(self.blocks.contains_key(&blockoffset), "requested block {} is in self.blocks_read but not in self.blocks", blockoffset);
                // XXX: during streaming, this might panic!
                let blockp: BlockP = self.blocks.get_mut(&blockoffset).unwrap().clone();
                self.store_block_in_LRU_cache(blockoffset, &blockp);
                debug_eprintln!(
                    "{}read_block: return Found(BlockP@{:p}); use stored Block[{}] @[{}, {}) len {}",
                    sx(),
                    &*self.blocks[&blockoffset],
                    &blockoffset,
                    BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz),
                    BlockReader::file_offset_at_block_offset(blockoffset+1, self.blocksz),
                    self.blocks[&blockoffset].len(),
                );
                return ResultS3_ReadBlock::Found(blockp);
            } else {
                self._read_blocks_miss += 1;
                debug_eprintln!("{}read_block: blockoffset {} not found in blocks_read", so(), blockoffset);
                debug_assert!(!self.blocks.contains_key(&blockoffset), "blocks has element {} not in blocks_read", blockoffset);
            }
        }

        match self.filetype {
            FileType::FILE => {
                self.read_block_FILE(blockoffset)
            },
            FileType::FILE_GZ => {
                self.read_block_FILE_GZ(blockoffset)
            },
            _ => {
                panic!("Unsupported filetype {:?}", self.filetype);
            },
        }
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
        assert_le!(fo_b, self.filesz(), "bad fo_b {} but filesz {} FPath {:?}", fo_b, self.filesz(), self.path);
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
