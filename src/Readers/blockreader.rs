// src/Readers/blockreader.rs
//
// Blocks and BlockReader implementations

pub use crate::common::{
    Count,
    FPath,
    FileOffset,
    FileType,
    FileSz,
};

use crate::common::{
    File,
    FileMetadata,
    FileOpenOptions,
    ResultS3,
    Bytes,
};

use crate::Data::datetime::{
    SystemTime,
};

#[cfg(any(debug_assertions,test))]
use crate::printer_debug::printers::{
    byte_to_char_noraw,
};

#[cfg(any(debug_assertions,test))]
use crate::printer_debug::stack::{
    sn,
    so,
    sx,
    snx,
};

use std::borrow::Cow;
use std::collections::{
    BTreeMap,
    BTreeSet,
    HashSet,
};
use std::fmt;
use std::fs::Metadata;
use std::io::{
    BufReader,
    Error,
    ErrorKind,
    Result,
    Seek,
    SeekFrom,
    Take,
};
use std::io::prelude::Read;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

extern crate debug_print;
use debug_print::{
    debug_eprintln
};

extern crate lru;
use lru::LruCache;

extern crate mime_guess;
use mime_guess::{
    MimeGuess,
};

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_ge,
    debug_assert_le,
};

extern crate flate2;
use flate2::read::GzDecoder;
use flate2::GzHeader;

// crate `lzma-rs` is the only pure rust crate.
// Other crates interface to liblzma which not ideal.
extern crate lzma_rs;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
pub const BLOCKSZ_DEF: usize = 0xFFFF;

/// data and readers for a gzip `.gz` file
#[derive(Debug)]
pub struct GzData {
    /// size of file uncompressed, taken from trailing gzip file data
    pub filesz: FileSz,
    /// calls to `read` use this
    pub decoder: GzDecoder<File>,
    /// filename taken from gzip header
    pub filename: String,
    /// file modified time taken from gzip header
    ///
    /// From https://datatracker.ietf.org/doc/html/rfc1952#page-7
    ///
    /// > MTIME (Modification TIME)
    /// > This gives the most recent modification time of the original
    /// > file being compressed.  The time is in Unix format, i.e.,
    /// > seconds since 00:00:00 GMT, Jan.  1, 1970.  (Note that this
    /// > may cause problems for MS-DOS and other systems that use
    /// > local rather than Universal time.)  If the compressed data
    /// > did not come from a file, MTIME is set to the time at which
    /// > compression started.  MTIME = 0 means no time stamp is
    /// > available.
    ///
    pub mtime: u32,
    /// CRC32 taken from trailing gzip file data
    pub crc32: u32,
}

type BufReader_Xz = BufReader<File>;

/// data and readers for a LZMA `.xz` file
#[derive(Debug)]
pub struct XzData {
    /// size of file uncompressed
    pub filesz: FileSz,
    pub bufreader: BufReader_Xz,
}

// TODO: 2022/07 it is not impossible for paths to have ':', use '\0' instead
//       which should never be in a path. But use ':' when printing paths.
/// separator substring for a filesystem path and subpath within an archive
/// e.g. `path/logs.tar:logs/syslog`
pub const SUBPATH_SEP: char = ':';

type TarHandle = tar::Archive::<File>;
/// taken from `tar::Archive::<File>::headers()`
type TarChecksum = u32;
/// taken from `tar::Archive::<File>::headers()`
type TarMTime = u64;

/// data and readers for a file within a `.tar` file
pub struct TarData {
    /// size of file unarchived
    pub filesz: FileSz,
    //pub handle: TarHandle,
    /// iteration count of `tar::Archive::entries_with_seek`
    pub entry_index: usize,
    /// checksum retreived from tar header
    pub checksum: TarChecksum,
    /// modified time retreived from tar header
    ///
    /// from https://www.gnu.org/software/tar/manual/html_node/Standard.html
    /// > The mtime field represents the data modification time of the file at
    /// > the time it was archived. It represents the integer number of seconds
    /// > since January 1, 1970, 00:00 Coordinated Universal Time. 
    pub mtime: TarMTime,
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
    /// subpath to file, only for `filetype.is_archived()` files
    pub subpath: Option<FPath>,
    /// File handle
    file: File,
    /// File.metadata()
    ///
    /// For compressed or archived files, the metadata of the `path`
    /// compress or archive file.
    file_metadata: FileMetadata,
    /// copy of `self.file_metadata.modified()`, copied during `new()`
    ///
    /// to simplify later retrievals
    pub(crate) file_metadata_modified: SystemTime,
    /// The `MimeGuess::from_path` result
    mimeguess_: MimeGuess,
    /// enum that guides file-handling behavior in `read`, `new`
    filetype: FileType,
    /// For gzipped files (FileType::FileGz), otherwise `None`
    gz: Option<GzData>,
    /// For LZMA xz files (FileType::FileXz), otherwise `None`
    xz: Option<XzData>,
    /// for files within a `.tar` file (FileType::FileTar), otherwise `None`
    tar: Option<TarData>,
    /// The filesz of uncompressed data, set during `new`.
    /// Should always be `== gz.unwrap().filesz`.
    ///
    /// Users should always call `filesz()`.
    pub(crate) filesz_actual: FileSz,
    /// File size in bytes of file at `path`, actual size.
    /// For compressed files, this is the size of the file compressed.
    /// For the uncompressed size of a compressed file, see `filesz_actual`.
    /// Set in `open`.
    ///
    /// For regular files (not compressed or archived),
    /// `filesz` and `filesz_actual` will be the same.
    ///
    /// Users should always call `filesz()`.
    pub(crate) filesz: FileSz,
    /// File size in blocks, set in `open`.
    pub(crate) blockn: u64,
    /// standard `Block` size in bytes; all `Block`s are this size except the
    /// last `Block` which may this size or smaller (and not zero).
    pub(crate) blocksz: BlockSz,
    /// Count of bytes stored by the `BlockReader`.
    /// May not match `self.blocks.iter().map(|x| sum += x.len()); sum` as
    /// `self.blocks` may have some elements `drop`ped during streaming.
    count_bytes_: Count,
    /// Storage of blocks `read` from storage. Lookups O(log(n)).
    ///
    /// During file processing, some elements that are not needed may be `drop`ped.
    blocks: Blocks,
    /// track blocks read in `read_block`. Never drops data.
    ///
    /// useful for when streaming kicks-in and some key+vale of `self.blocks` have
    /// been dropped.
    blocks_read: BlocksTracked,
    /// internal LRU cache for `fn read_block()`. Lookups O(1).
    read_block_lru_cache: BlocksLRUCache,
    /// enable/disable use of `read_block_lru_cache`
    read_block_lru_cache_enabled: bool,
    /// internal LRU cache count of lookup hits
    pub(crate) read_block_cache_lru_hit: Count,
    /// internal LRU cache count of lookup misses
    pub(crate) read_block_cache_lru_miss: Count,
    /// internal LRU cache count of lookup `.put`
    pub(crate) read_block_cache_lru_put: Count,
    /// internal storage count of lookup hit
    pub(crate) read_blocks_hit: Count,
    /// internal storage count of lookup miss
    pub(crate) read_blocks_miss: Count,
    /// internal storage count of `self.blocks.insert`
    pub(crate) read_blocks_put: Count,
}

impl fmt::Debug for BlockReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BlockReader")
            .field("path", &self.path)
            .field("file", &self.file)
            //.field("file_metadata", &self._file_metadata)
            .field("mimeguess", &self.mimeguess_)
            .field("filesz", &self.filesz())
            .field("blockn", &self.blockn)
            .field("blocksz", &self.blocksz)
            .field("blocks currently stored", &self.blocks.len())
            .field("blocks read", &self.blocks_read.len())
            .field("bytes read", &self.count_bytes_)
            .field("cache LRU hit", &self.read_block_cache_lru_hit)
            .field("miss", &self.read_block_cache_lru_miss)
            .field("put", &self.read_block_cache_lru_put)
            .field("cache hit", &self.read_blocks_hit)
            .field("miss", &self.read_blocks_miss)
            .field("insert", &self.read_blocks_put)
            .finish()
    }
}

/// helper to unpack DWORD unsigned integers in a gzip header
///
/// XXX: u32::from_*_bytes failed for test file compressed with GNU gzip 1.10
///
/// TODO: validate XXX, did I do that correctly?
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
    const GZ_MAX_SZ: FileSz = 0x20000000;
    /// cache slots for `read_block` LRU cache
    const READ_BLOCK_LRU_CACHE_SZ: usize = 4;

    /// Create a new `BlockReader`.
    ///
    /// Opens the `path` file, configures settings based on determined `filetype`.
    pub fn new(path: FPath, filetype: FileType, blocksz_: BlockSz) -> Result<BlockReader> {
        // TODO: how to make some fields `blockn` `blocksz` `filesz` immutable?
        //       https://stackoverflow.com/questions/23743566/how-can-i-force-a-structs-field-to-always-be-immutable-in-rust
        debug_eprintln!("{}BlockReader::new({:?}, {:?}, {:?})", sn(), path, filetype, blocksz_);

        assert_ne!(0, blocksz_, "Block Size cannot be 0");
        assert_ge!(blocksz_, BLOCKSZ_MIN, "Block Size {} is too small", blocksz_);
        assert_le!(blocksz_, BLOCKSZ_MAX, "Block Size {} is too big", blocksz_);

        // shadow passed immutable with local mutable
        let mut path: FPath = path;
        let mut subpath_opt: Option<FPath> = None;
        if filetype.is_archived() {
            debug_eprintln!("{}BlockReader::new: filetype.is_archived()", so());
            let mut path_tmp: Option<FPath> = None;
            {
                let (path_, subpath_) = match path.rsplit_once(SUBPATH_SEP) {
                    Some(val) => val,
                    None => {
                        debug_eprintln!("{}BlockReader::new: filetype {:?}, failed to find delimiter {:?} in {:?}", sx(), filetype, SUBPATH_SEP, path);
                        return Result::Err(
                            Error::new(
                                // TODO: use `ErrorKind::InvalidFilename` when it is stable
                                ErrorKind::NotFound,
                                format!("Given Filetype {:?} but failed to find delimiter {:?} in {:?}", filetype, SUBPATH_SEP, path)
                            )
                        );

                    }
                };
                path_tmp = Some(path_.to_string());
                subpath_opt = Some(subpath_.to_string());
            }
            if path_tmp.is_some() {
                path = path_tmp.unwrap();
            }
        }
        let path_std: &Path = Path::new(&path);

        // TODO: pass in `mimeguess`; avoid repeats of the tedious operation
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
        let mut blocks = Blocks::new();
        let mut blocks_read = BlocksTracked::new();
        let mut count_bytes_: Count = 0;
        let filesz: FileSz;
        let mut filesz_actual: FileSz;
        let blocksz: BlockSz;
        let file_metadata: FileMetadata;
        let file_metadata_modified: SystemTime;
        let mut gz_opt: Option<GzData> = None;
        let mut xz_opt: Option<XzData> = None;
        let mut tar_opt: Option<TarData> = None;
        let mut read_blocks_put: Count = 0;
        match file.metadata() {
            Ok(val) => {
                filesz = val.len() as FileSz;
                file_metadata = val;
                file_metadata_modified = match file_metadata.modified() {
                    Ok(systemtime_) => {
                        systemtime_
                    }
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: file_metadata.modified() failed Err {:?}", sx(), err);
                        return Result::Err(err);
                    }
                }
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
            FileType::File => {
                filesz_actual = filesz;
                blocksz = blocksz_;
            },
            FileType::FileGz => {
                blocksz = blocksz_;
                debug_eprintln!("{0}BlockReader::new: FileGz: blocksz set to {1} (0x{1:08X}) (passed {2} (0x{2:08X})", so(), blocksz, blocksz_);

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
                    debug_eprintln!("{}BlockReader::new: FileGz: return Err(InvalidData)", sx());
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
                    debug_eprintln!("{}BlockReader::new: FileGz: return Err(InvalidData)", sx());
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
                        debug_eprintln!("{}BlockReader::new: FileGz: return Err({})", sx(), err);
                        eprintln!("ERROR: file.SeekFrom(-8) Error {}", err);
                        return Err(err);
                    },
                };
                let mut reader = (&file).take(8);

                // extract DWORD for CRC32
                let mut buffer_crc32: [u8; 4] = [0; 4];
                debug_eprintln!("{}BlockReader::new: FileGz: reader.read_exact(@{:p}) (buffer len {})", so(), &buffer_crc32, buffer_crc32.len());
                match reader.read_exact(&mut buffer_crc32) {
                    Ok(_) => {},
                    //Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {},
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: FileGz: return {:?}", sx(), err);
                        eprintln!("reader.read_to_end(&buffer_crc32) Error {:?}", err);
                        return Err(err);
                    },
                }
                debug_eprintln!("{}BlockReader::new: FileGz: buffer_crc32 {:?}", so(), buffer_crc32);
                let crc32 = dword_to_u32(&buffer_crc32);
                debug_eprintln!("{}BlockReader::new: FileGz: crc32 {1} (0x{1:08X})", so(), crc32);

                // extract DWORD for SIZE
                let mut buffer_size: [u8; 4] = [0; 4];
                debug_eprintln!("{}BlockReader::new:FileGz:  reader.read_exact(@{:p}) (buffer len {})", so(), &buffer_size, buffer_size.len());
                match reader.read_exact(&mut buffer_size) {
                    Ok(_) => {},
                    Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {},
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: FileGz: return {:?}", sx(), err);
                        eprintln!("reader.read_to_end(&buffer_size) Error {:?}", err);
                        return Err(err);
                    },
                }
                debug_eprintln!("{}BlockReader::new: FileGz: buffer_size {:?}", so(), buffer_size);
                let size: u32 = dword_to_u32(&buffer_size);
                debug_eprintln!("{}BlockReader::new: FileGz: file size uncompressed {1:?} (0x{1:08X})", so(), size);
                let filesz_uncompressed: FileSz = size as FileSz;
                if filesz_uncompressed == 0 {
                    debug_eprintln!("{}BlockReader::new: FileGz: return Err(InvalidData)", sx());
                    return Result::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!("extracted uncompressed file size value 0, nothing to read {:?}", path),
                        )
                    );
                }
                filesz_actual = filesz_uncompressed;

                // reset Seek pointer
                // XXX: not sure if this is necessary
                (&file).seek(SeekFrom::Start(0));

                //let mut open_options = FileOpenOptions::new();
                debug_eprintln!("{}BlockReader::new:FileGz:  open_options.read(true).open({:?})", so(), path_std);
                let file_gz: File = match open_options.read(true).open(path_std) {
                    Ok(val) => val,
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: FileGz: open_options.read({:?}) Error, return {:?}", sx(), path, err);
                        return Err(err);
                    }
                };
                let decoder: GzDecoder<File> = GzDecoder::new(file_gz);
                debug_eprintln!("{}BlockReader::new: FileGz: {:?}", so(), decoder);
                let header_opt: Option<&GzHeader> = decoder.header();
                let mut filename: String = String::with_capacity(0);

                //
                // GZIP binary format https://datatracker.ietf.org/doc/html/rfc1952#page-5
                //
                // Each member has the following structure:
                //
                // +---+---+---+---+---+---+---+---+---+---+
                // |ID1|ID2|CM |FLG|     MTIME     |XFL|OS | (more-->)
                // +---+---+---+---+---+---+---+---+---+---+
                //
                // MTIME (Modification TIME)
                // This gives the most recent modification time of the original
                // file being compressed.  The time is in Unix format, i.e.,
                // seconds since 00:00:00 GMT, Jan.  1, 1970.  (Note that this
                // may cause problems for MS-DOS and other systems that use
                // local rather than Universal time.)  If the compressed data
                // did not come from a file, MTIME is set to the time at which
                // compression started.  MTIME = 0 means no time stamp is
                // available.
                //
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
                        debug_eprintln!("{}BlockReader::new: FileGz: GzDecoder::header() is None for {:?}", so(), path);
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
                debug_eprintln!("{}BlockReader::new: FileGz: created {:?}", so(), gz_opt);
            },
            FileType::FileXz => {
                blocksz = blocksz_;
                debug_eprintln!("{0}BlockReader::new: FileXz: blocksz set to {1} (0x{1:08X}) (passed {2} (0x{2:08X})", so(), blocksz, blocksz_);
                
                debug_eprintln!("{}BlockReader::new: FileXz: open_options.read(true).open({:?})", so(), path_std);
                let mut file_xz: File = match open_options.read(true).open(path_std) {
                    Ok(val) => val,
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: FileXz: open_options.read({:?}) Error, return {:?}", sx(), path, err);
                        return Err(err);
                    }
                };

                //
                // Get the .xz file size from XZ header
                //
                // "bare-bones" implentation of reading xz compressed file
                // other availale crates for reading `.xz` files did not meet
                // the needs of this program.
                //

                /*
                    https://tukaani.org/xz/xz-file-format.txt

                    1. Byte and Its Representation

                            In this document, byte is always 8 bits.

                            A "null byte" has all bits unset. That is, the value of a null
                            byte is 0x00.

                            To represent byte blocks, this document uses notation that
                            is similar to the notation used in [RFC-1952]:

                                +-------+
                                |  Foo  |   One byte.
                                +-------+

                                +---+---+
                                |  Foo  |   Two bytes; that is, some of the vertical bars
                                +---+---+   can be missing.

                                +=======+
                                |  Foo  |   Zero or more bytes.
                                +=======+

                    2. Overall Structure of .xz File

                            A standalone .xz files consist of one or more Streams which may
                            have Stream Padding between or after them:

                                +========+================+========+================+
                                | Stream | Stream Padding | Stream | Stream Padding | ...
                                +========+================+========+================+

                    2.1. Stream

                            +-+-+-+-+-+-+-+-+-+-+-+-+=======+=======+     +=======+
                            |     Stream Header     | Block | Block | ... | Block |
                            +-+-+-+-+-+-+-+-+-+-+-+-+=======+=======+     +=======+

                    2.1.1. Stream Header

                            +---+---+---+---+---+---+-------+------+--+--+--+--+
                            |  Header Magic Bytes   | Stream Flags |   CRC32   |
                            +---+---+---+---+---+---+-------+------+--+--+--+--+

                    3. Block

                            +==============+=================+===============+=======+
                            | Block Header | Compressed Data | Block Padding | Check |
                            +==============+=================+===============+=======+

                    3.1. Block Header

                            +-------------------+-------------+=================+
                            | Block Header Size | Block Flags | Compressed Size |
                            +-------------------+-------------+=================+

                                +===================+======================+
                            ---> | Uncompressed Size | List of Filter Flags |
                                +===================+======================+

                                +================+--+--+--+--+
                            ---> | Header Padding |   CRC32   |
                                +================+--+--+--+--+

                    3.1.1. Block Header Size

                            This field overlaps with the Index Indicator field (see
                            Section 4.1).

                            This field contains the size of the Block Header field,
                            including the Block Header Size field itself. Valid values are
                            in the range [0x01, 0xFF], which indicate the size of the Block
                            Header as multiples of four bytes, minimum size being eight
                            bytes:

                                real_header_size = (encoded_header_size + 1) * 4;

                            If a Block Header bigger than 1024 bytes is needed in the
                            future, a new field can be added between the Block Header and
                            Compressed Data fields. The presence of this new field would
                            be indicated in the Block Header field.

                    3.1.2. Block Flags

                            The Block Flags field is a bit field:

                                Bit(s)  Mask  Description
                                0-1    0x03  Number of filters (1-4)
                                2-5    0x3C  Reserved for future use; MUST be zero for now.
                                6     0x40  The Compressed Size field is present.
                                7     0x80  The Uncompressed Size field is present.

                            If any reserved bit is set, the decoder MUST indicate an error.
                            It is possible that there is a new field present which the
                            decoder is not aware of, and can thus parse the Block Header
                            incorrectly.

                    3.1.3. Compressed Size

                            This field is present only if the appropriate bit is set in
                            the Block Flags field (see Section 3.1.2).

                            The Compressed Size field contains the size of the Compressed
                            Data field, which MUST be non-zero. Compressed Size is stored
                            using the encoding described in Section 1.2. If the Compressed
                            Size doesn't match the size of the Compressed Data field, the
                            decoder MUST indicate an error.

                    3.1.4. Uncompressed Size

                            This field is present only if the appropriate bit is set in
                            the Block Flags field (see Section 3.1.2).

                            The Uncompressed Size field contains the size of the Block
                            after uncompressing. Uncompressed Size is stored using the
                            encoding described in Section 1.2. If the Uncompressed Size
                            does not match the real uncompressed size, the decoder MUST
                            indicate an error.

                            Storing the Compressed Size and Uncompressed Size fields serves
                            several purposes:
                            - The decoder knows how much memory it needs to allocate
                                for a temporary buffer in multithreaded mode.
                            - Simple error detection: wrong size indicates a broken file.
                            - Seeking forwards to a specific location in streamed mode.

                            It should be noted that the only reliable way to determine
                            the real uncompressed size is to uncompress the Block,
                            because the Block Header and Index fields may contain
                            (intentionally or unintentionally) invalid information.

                */

                // create "take handler" that will read bytes as-is (no decompression)
                match (&file_xz).seek(SeekFrom::Start(0)) {
                    Ok(_) => {},
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: FileXz: return Err({})", sx(), err);
                        eprintln!("ERROR: file.SeekFrom(0) Error {}", err);
                        return Err(err);
                    },
                };
                let mut reader = (&file_xz).take(6 + 2 + 4 + 1 + 1);

                // stream header magic bytes
                let mut buffer_: [u8; 6] = [0; 6];
                match reader.read_exact(&mut buffer_) {
                    Ok(_) => {},
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: FileXz: return {:?}", sx(), err);
                        eprintln!("reader.read_exact() (stream header magic bytes) Error {:?}", err);
                        return Err(err);
                    },
                }
                debug_eprintln!("{}BlockReader::new: FileXz: stream header magic bytes {:?}", so(), buffer_);
                if cfg!(debug_assertions) {
                    for b_ in buffer_.iter() {
                        let c_: char = (*b_) as char;
                        debug_eprintln!("{}  {1:3} (0x{1:02X}) {2:?}", so(), b_, c_);
                    }
                }
                // magic bytes expected "ý7zXZ\0"
                if buffer_ != [0xFD, 0x37, 0x7A, 0x58, 0x5A,0x00] {
                    return Result::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!("Failed to find XZ stream header magic bytes for {:?}", path_std)
                        )
                    );
                }

                // stream header flags
                let mut buffer_: [u8; 2] = [0; 2];
                match reader.read_exact(&mut buffer_) {
                    Ok(_) => {},
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: FileXz: return {:?}", sx(), err);
                        eprintln!("reader.read_exact() (stream header flags) Error {:?}", err);
                        return Err(err);
                    },
                }
                debug_eprintln!("{}BlockReader::new: FileXz: buffer {:?}", so(), buffer_);
                let _flags: u16 = u16::from_le_bytes(buffer_);
                debug_eprintln!("{}BlockReader::new: FileXz: stream header flags 0b{1:016b}", so(), _flags);

                // stream header CRC32
                let mut buffer_: [u8; 4] = [0; 4];
                match reader.read_exact(&mut buffer_) {
                    Ok(_) => {},
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: FileXz: return {:?}", sx(), err);
                        eprintln!("reader.read_exact() (stream header CRC32) Error {:?}", err);
                        return Err(err);
                    },
                }
                debug_eprintln!("{}BlockReader::new: FileXz: buffer {:?}", so(), buffer_);
                let _crc32: u32 = u32::from_le_bytes(buffer_);
                debug_eprintln!("{}BlockReader::new: FileXz: stream header CRC32 {1:} (0x{1:08X}) (0b{1:032b})", so(), _crc32);

                // block #0 block header size
                let mut buffer_: [u8; 1] = [0; 1];
                match reader.read_exact(&mut buffer_) {
                    Ok(_) => {},
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: FileXz: return {:?}", sx(), err);
                        eprintln!("reader.read_exact() (block #0 block header size) Error {:?}", err);
                        return Err(err);
                    },
                }
                debug_eprintln!("{}BlockReader::new: FileXz: buffer {:?}", so(), buffer_);
                let _bhsz: u8 = buffer_[0];
                debug_eprintln!("{}BlockReader::new: FileXz: block #0 block header size {1:} (0x{1:02X})", so(), _bhsz);

                // block #0 block header flags
                let mut buffer_: [u8; 1] = [0; 1];
                match reader.read_exact(&mut buffer_) {
                    Ok(_) => {},
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: FileXz: return {:?}", sx(), err);
                        eprintln!("reader.read_exact() (block #0 block header flags) Error {:?}", err);
                        return Err(err);
                    },
                }
                debug_eprintln!("{}BlockReader::new: FileXz: buffer {:?}", so(), buffer_);
                let _bhflags: u8 = buffer_[0];
                debug_eprintln!("{}BlockReader::new: FileXz: block #0 block header flags {1:} (0x{1:02X}) (0b{1:08b})", so(), _bhflags);

                // reset Seek pointer
                match file_xz.seek(SeekFrom::Start(0)) {
                    Ok(_) => {},
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: FileXz: return {:?}", sx(), err);
                        eprintln!("file_xz.seek() (block #0 block header flags) Error {:?}", err);
                        return Err(err);
                    }
                }

                let mut bufreader: BufReader_Xz = BufReader_Xz::new(file_xz);

                // XXX: THIS IS A TERRIBLE HACK!
                //      read the entire file into blocks in one go!
                //      putting this here until the implementation of reading the header/blocks
                //      of the underlying .xz file
                #[allow(clippy::never_loop)]
                loop {
                    let mut buffer = Block::new();
                    debug_eprintln!("{}BlockReader::new: FileXz: xz_decompress({:?}, buffer (len {}, capacity {}))", so(), bufreader, buffer.len(), buffer.capacity());
                    // XXX: xz_decompress may resize the passed `buffer`
                    match lzma_rs::xz_decompress(&mut bufreader, &mut buffer) {
                        Ok(_) => {
                            debug_eprintln!("{}BlockReader::new: FileXz: xz_decompress returned buffer len {}, capacity {}", so(), buffer.len(), buffer.capacity());
                        },
                        Err(err) => {
                            match err {
                                lzma_rs::error::Error::IoError(ref ioerr) => {
                                    if ioerr.kind() == ErrorKind::UnexpectedEof {
                                        break;
                                    }
                                }
                                _ => {},
                            }
                            debug_eprintln!("{}BlockReader::new: FileXz: xz_decompress Error, return Err({:?})", sx(), err);
                            return Err(
                                Error::new(
                                    ErrorKind::Other,
                                    format!("{:?}", err),
                                )
                            );
                        }
                    }
                    if buffer.is_empty() {
                        break;
                    }
                    let blocksz_u: usize = blocksz as usize;
                    let mut blockoffset: BlockOffset = 0;
                    // the `block`
                    while blockoffset <= ((buffer.len() / blocksz_u) as BlockOffset) {
                        let mut block: Block = Block::with_capacity(blocksz_u);
                        let a: usize = (blockoffset * blocksz) as usize;
                        let b: usize = a + (std::cmp::min(blocksz_u, buffer.len() - a));
                        debug_eprintln!("{}BlockReader::new: FileXz: block.extend_from_slice(&buffer[{}‥{}])", so(), a, b);
                        block.extend_from_slice(&buffer[a..b]);
                        let blockp: BlockP = BlockP::new(block);
                        if let Some(bp_) = blocks.insert(blockoffset, blockp.clone()) {
                            eprintln!("WARNING: blockreader.blocks.insert({}, BlockP@{:p}) already had a entry BlockP@{:p}", blockoffset, blockp, bp_);
                        }
                        read_blocks_put += 1;
                        count_bytes_ += (*blockp).len() as Count;
                        blocks_read.insert(blockoffset);
                        blockoffset += 1;
                    }
                    break;
                }

                let filesz_uncompressed: FileSz = count_bytes_ as FileSz;

                filesz_actual = filesz_uncompressed;
                xz_opt = Some(
                    XzData {
                        filesz: filesz_uncompressed,
                        bufreader,
                    }
                );
                debug_eprintln!("{}BlockReader::new: created {:?}", so(), xz_opt.as_ref().unwrap());
            },
            FileType::FileTar => {
                blocksz = blocksz_;
                debug_eprintln!("{0}BlockReader::new: FileTar: blocksz set to {1} (0x{1:08X}) (passed {2} (0x{2:08X})", so(), blocksz, blocksz_);
                filesz_actual = 0;
                let mut checksum: TarChecksum = 0;
                let mut mtime: TarMTime = 0;
                let subpath: &String = subpath_opt.as_ref().unwrap();

                let mut archive: TarHandle = BlockReader::open_tar(&path_std)?;
                let entry_iter: tar::Entries<File> = match archive.entries_with_seek() {
                    Ok(val) => {
                        val
                    },
                    Err(err) => {
                        debug_eprintln!("{}BlockReader::new: FileTar: Err {:?}", sx(), err);
                        return Result::Err(err);
                    }
                };

                let mut entry_index: usize = 0;
                for (index, entry_res) in entry_iter.enumerate() {
                    entry_index = index;
                    let entry: tar::Entry<File> = match entry_res {
                        Ok(val) => val,
                        Err(err) => {
                            debug_eprintln!("{}BlockReader::new: FileTar: entry Err {:?}", so(), err);
                            continue;
                        }
                    };
                    let subpath_cow: Cow<Path> = match entry.path() {
                        Ok(val) => val,
                        Err(err) => {
                            debug_eprintln!("{}BlockReader::new: FileTar: entry.path() Err {:?}", so(), err);
                            continue;
                        }
                    };
                    let subfpath: FPath = subpath_cow.to_string_lossy().to_string();
                    if subpath != &subfpath {
                        debug_eprintln!("{}BlockReader::new: FileTar: skip {:?}", so(), subfpath);
                        continue;
                    }
                    // found the matching subpath
                    debug_eprintln!("{}BlockReader::new: FileTar: found {:?}", so(), subpath);
                    filesz_actual = match entry.header().size() {
                        Ok(val) => val,
                        Err(err) => {
                            debug_eprintln!("{}BlockReader::new: FileTar: entry.header().size() Err {:?}", sx(), err);
                            return Result::Err(err);
                        }
                    };
                    checksum = match entry.header().cksum() {
                        Ok(val) => val,
                        Err(err) => {
                            debug_eprintln!("{}BlockReader::new: FileTar: entry.header().cksum() Err {:?}", so(), err);

                            0
                        }
                    };
                    mtime = match entry.header().mtime() {
                        Ok(val) => val,
                        Err(err) => {
                            debug_eprintln!("{}BlockReader::new: FileTar: entry.header().mtime() Err {:?}", so(), err);

                            0
                        }
                    };
                    break;
                }

                tar_opt = Some(
                    TarData {
                        filesz: filesz_actual,
                        //handle: archive,
                        entry_index,
                        checksum,
                        mtime,
                    }
                );
            }
            _ => {
                return Result::Err(
                    Error::new(
                        ErrorKind::Unsupported,
                        format!("Unsupported FileType {:?}", filetype)
                    )
                );
            }
        }

        // XXX: don't assert on `filesz` vs `filesz_actual`; for some `.gz` files they can be
        //      either gt, lt, or eq.

        let blockn: Count = BlockReader::count_blocks(filesz_actual, blocksz);

        debug_eprintln!("{}BlockReader::new: return Ok(BlockReader)", sx());

        Ok(
            BlockReader {
                path,
                subpath: subpath_opt,
                file,
                file_metadata,
                file_metadata_modified,
                mimeguess_,
                filetype,
                gz: gz_opt,
                xz: xz_opt,
                tar: tar_opt,
                filesz,
                filesz_actual,
                blockn,
                blocksz,
                //count_bytes_: 0,
                count_bytes_,
                //blocks: Blocks::new(),
                blocks,
                //blocks_read: BlocksTracked::new(),
                blocks_read,
                read_block_lru_cache: BlocksLRUCache::new(BlockReader::READ_BLOCK_LRU_CACHE_SZ),
                read_block_lru_cache_enabled: true,
                read_block_cache_lru_hit: 0,
                read_block_cache_lru_miss: 0,
                read_block_cache_lru_put: 0,
                read_blocks_hit: 0,
                read_blocks_miss: 0,
                //read_blocks_put: 0,
                read_blocks_put,
            }
        )
    }

    #[inline(always)]
    pub const fn mimeguess(&self) -> MimeGuess {
        self.mimeguess_
    }

    pub const fn filesz(&self) -> FileSz {
        match self.filetype {
            FileType::FileGz
            | FileType::FileXz
            | FileType::FileTar => {
                self.filesz_actual
            },
            FileType::File => {
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

    /// return a copy of `self.file_metadata`
    pub fn metadata(&self) -> Metadata {
        self.file_metadata.clone()
    }

    /// get the best available "Modified Datetime" file attribute avaiable.
    ///
    /// For compressed or archived files, if the embedded "MTIME" is not available then
    /// return the encompassing file's "MTIME".
    ///
    /// For example, if archived file `syslog` within archive `logs.tar` does not have a valid
    /// "MTIME", then return the file system attribute "modified time" for `logs.tar`.
    //
    // TODO: also handle when `self.file_metadata_modified` is zero (or a non-meaningful placeholder
    //       value).
    pub fn mtime(&self) -> SystemTime {
        match self.filetype {
            FileType::File
            | FileType::FileXz => {
                self.file_metadata_modified
            }
            FileType::FileGz => {
                let mtime = self.gz.as_ref().unwrap().mtime;
                if mtime != 0 {
                    SystemTime::UNIX_EPOCH + Duration::from_secs(mtime as u64)
                }
                else {
                    self.file_metadata_modified
                }
            }
            FileType::FileTar => {
                let mtime = self.tar.as_ref().unwrap().mtime;
                if mtime != 0 {
                    SystemTime::UNIX_EPOCH + Duration::from_secs(mtime)
                }
                else {
                    self.file_metadata_modified
                }
            }
            _ => {
                panic!("Unsupported filetype {:?}", self.filetype);
            }
        }
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

    /// get the last byte index of the file
    pub const fn fileoffset_last(&self) -> FileOffset {
        (self.filesz() - 1) as FileOffset
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
    pub const fn count_blocks(filesz: FileSz, blocksz: BlockSz) -> Count {
        filesz / blocksz + (if filesz % blocksz > 0 { 1 } else { 0 })
    }

    /// specific `BlockSz` (size in bytes) of block at `blockoffset`
    pub fn blocksz_at_blockoffset_(blockoffset: &BlockOffset, blockoffset_last: &BlockOffset, blocksz: &BlockSz, filesz: &FileSz) -> BlockSz {
        debug_eprintln!("{}blockreader.blocksz_at_blockoffset_(blockoffset {}, blockoffset_last {}, blocksz {}, filesz {})", snx(), blockoffset, blockoffset_last, blocksz, filesz);
        debug_assert_le!(blockoffset, blockoffset_last, "Passed blockoffset {} but blockoffset_last {}", blockoffset, blockoffset_last);
        if blockoffset == blockoffset_last {
            let remainder = filesz % blocksz;
            if remainder != 0 {
                remainder
            } else {
                *blocksz
            }
        } else {
            *blocksz
        }
    }

    /// specific `BlockSz` (size in bytes) of block at `blockoffset`
    ///
    /// This should be `self.blocksz` for all `Block`s except the last, which
    /// should be `>0` and `<=self.blocksz`
    pub fn blocksz_at_blockoffset(&self, blockoffset: &BlockOffset) -> BlockSz {
        debug_eprintln!("{}blockreader.blocksz_at_blockoffset({})", snx(), blockoffset);
        BlockReader::blocksz_at_blockoffset_(blockoffset, &self.blockoffset_last(), &self.blocksz, &self.filesz())
    }

    /// return block.len() for *stored* `Block` at `blockoffset`
    #[inline(always)]
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn last_blockindex_at_blockoffset(&self, blockoffset: &BlockOffset) -> BlockIndex {
        (self.blocklen_at_blockoffset(blockoffset) - 1) as BlockIndex
    }

    /// last valid `BlockOffset` for the file (inclusive)
    #[inline(always)]
    pub const fn blockoffset_last(&self) -> BlockOffset {
        if self.filesz() == 0 {
            return 0;
        }
        (BlockReader::count_blocks(self.filesz(), self.blocksz) as BlockOffset) - 1
    }

    /// count of blocks read by this `BlockReader` (adjusted during calls to `BlockReader::read_block`)
    ///
    /// not the same as blocks currently stored (they may be removed during streaming stage)
    #[inline(always)]
    pub fn count_blocks_processed(&self) -> Count {
        self.blocks_read.len() as Count
    }

    /// count of bytes stored by this `BlockReader` (adjusted during calls to `BlockReader::read_block`)
    #[inline(always)]
    pub fn count_bytes(&self) -> Count {
        self.count_bytes_
    }

    /// enable internal LRU cache used by `read_block`.
    ///
    /// intended to aid testing and debugging
    #[allow(dead_code)]
    pub fn LRU_cache_enable(&mut self) {
        if self.read_block_lru_cache_enabled {
            return;
        }
        self.read_block_lru_cache_enabled = true;
        self.read_block_lru_cache.clear();
        self.read_block_lru_cache.resize(BlockReader::READ_BLOCK_LRU_CACHE_SZ);
    }

    /// disable internal LRU cache used by `read_block`.
    ///
    /// intended to aid testing and debugging
    pub fn LRU_cache_disable(&mut self) {
        self.read_block_lru_cache_enabled = false;
        self.read_block_lru_cache.resize(0);
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
                debug_eprintln!("{}blockreader.drop_block({}): dropped block {} @0x{:p}, len {}, strong_count {}", so(), blockoffset, blockoffset, blockp, (*blockp).len(), Arc::strong_count(&blockp));
                bo_dropped.insert(blockoffset);
            },
            None => {
                debug_eprintln!("{}blockreader.drop_block({}): no block to drop at {}", so(), blockoffset, blockoffset);
            },
        }
        match self.read_block_lru_cache.pop(&blockoffset) {
            Some(blockp) => {
                debug_eprintln!("{}blockreader.drop_block({}): dropped block in LRU cache {} @0x{:p}, len {}, strong_count {}", so(), blockoffset, blockoffset, blockp, (*blockp).len(), Arc::strong_count(&blockp));
                bo_dropped.insert(blockoffset);
            },
            None => {
                debug_eprintln!("{}blockreader.drop_block({}): no block in LRU cache to drop at {}", so(), blockoffset, blockoffset);
            }
        }
    }

    /// store clone of `BlockP` in LRU cache
    fn store_block_in_LRU_cache(&mut self, blockoffset: BlockOffset, blockp: &BlockP) {
        debug_eprintln!("{}store_block_in_LRU_cache: LRU cache put({}, BlockP@{:p})", so(), blockoffset, blockp);
        if ! self.read_block_lru_cache_enabled {
            return;
        }
        self.read_block_lru_cache.put(blockoffset, blockp.clone());
        self.read_block_cache_lru_put += 1;
    }

    /// store clone of `BlockP` in `self.blocks` storage.
    fn store_block_in_storage(&mut self, blockoffset: BlockOffset, blockp: &BlockP) {
        debug_eprintln!("{}store_block_in_storage: blocks.insert({}, BlockP@{:p} (len {}, capacity {}))", snx(), blockoffset, blockp, (*blockp).len(), (*blockp).capacity());
        #[allow(clippy::single_match)]
        match self.blocks.insert(blockoffset, blockp.clone()) {
            Some(bp_) => {
                eprintln!("WARNING: blockreader.blocks.insert({}, BlockP@{:p}) already had a entry BlockP@{:p}", blockoffset, blockp, bp_);
            },
            _ => {},
        }
        self.read_blocks_put += 1;
        self.count_bytes_ += (*blockp).len() as Count;
        if let false = self.blocks_read.insert(blockoffset) {
            eprintln!("WARNING: blockreader.blocks_read({}) already had a entry", blockoffset);
        }
    }

    /// read up to `blocksz` bytes of data (one `Block`) from a regular filesystem file.
    ///
    /// Called from `read_block`.
    fn read_block_File(&mut self, blockoffset: BlockOffset) -> ResultS3_ReadBlock {
        debug_eprintln!("{}read_block_File({})", sn(), blockoffset);
        debug_assert_eq!(self.filetype, FileType::File, "wrong FileType {:?} for calling read_block_FILE", self.filetype);

        let seek = (self.blocksz * blockoffset) as u64;
        debug_eprintln!("{}read_block_File: self.file.seek({})", so(), seek);
        match self.file.seek(SeekFrom::Start(seek)) {
            Ok(_) => {},
            Err(err) => {
                eprintln!("ERROR: file.SeekFrom(Start({})) Error {}", seek, err);
                debug_eprintln!("{}read_block_File({}): return Err({})", sx(), blockoffset, err);
                return ResultS3_ReadBlock::Err(err);
            },
        };
        // here is where the `Block` is created then set with data.
        // It should never change after this. Is there a way to mark it as "frozen"?
        // XXX: currently does not handle a partial read. From the docs (https://doc.rust-lang.org/std/io/trait.Read.html#method.read_to_end)
        //      > If any other read error is encountered then this function immediately returns. Any
        //      > bytes which have already been read will be appended to buf.
        let cap: usize = self.blocksz_at_blockoffset(&blockoffset) as usize;
        let mut buffer = Block::with_capacity(cap);
        let mut reader: Take<&File> = (&self.file).take(cap as u64);
        debug_eprintln!("{}read_block_File: reader.read_to_end(buffer (capacity {}))", so(), cap);
        // TODO: change to `read_exact` which recently stabilized
        match reader.read_to_end(&mut buffer) {
            Ok(val) => {
                if val == 0 {
                    debug_eprintln!("{}read_block_File({}): return Done for {:?}", sx(), blockoffset, self.path);
                    return ResultS3_ReadBlock::Done;
                }
            },
            Err(err) => {
                eprintln!("ERROR: reader.read_to_end(buffer) error {} for {:?}", err, self.path);
                debug_eprintln!("{}read_block_File({}): return Err({})", sx(), blockoffset, err);
                return ResultS3_ReadBlock::Err(err);
            },
        };
        let blockp: BlockP = BlockP::new(buffer);
        self.store_block_in_storage(blockoffset, &blockp);
        self.store_block_in_LRU_cache(blockoffset, &blockp);
        debug_eprintln!("{}read_block_File({}): return Found", sx(), blockoffset);

        ResultS3_ReadBlock::Found(blockp)
    }

    /// read a block of data from storage for a compressed gzip file.
    /// `blockoffset` refers to the uncompressed version of the file.
    ///
    /// Called from `read_block`.
    ///
    /// A gzip file must be read from beginning to end in sequence (cannot jump forward and read).
    /// So `read_block_FileGz` reads entire file up to passed `blockoffset`, storing each
    /// decompressed block.
    fn read_block_FileGz(&mut self, blockoffset: BlockOffset) -> ResultS3_ReadBlock {
        debug_eprintln!("{}read_block_FileGz({})", sn(), blockoffset);
        debug_assert_eq!(self.filetype, FileType::FileGz, "wrong FileType {:?} for calling read_block_FileGz", self.filetype);

        let blockoffset_last: BlockOffset = self.blockoffset_last();
        let mut bo_at: BlockOffset = match self.blocks_read.iter().max() {
            Some(bo_) => *bo_,
            None => 0,
        };

        while bo_at <= blockoffset {
            // check `self.blocks_read` (not `self.blocks`) if the Block at `blockoffset`
            // was *ever* read.
            // TODO: [2022/06/18] add another stat tracker for lookups in `self.blocks_read`
            if self.blocks_read.contains(&bo_at) {
                self.read_blocks_hit += 1;
                //debug_eprintln!("{}read_block_FileGz({}): blocks_read.contains({})", so(), blockoffset, bo_at);
                if bo_at == blockoffset {
                    debug_eprintln!("{}read_block_FileGz({}): return Found", sx(), blockoffset);
                    // XXX: this will panic if the key+value in `self.blocks` was dropped
                    //      which could happen during streaming stage
                    let blockp: BlockP = self.blocks.get_mut(&bo_at).unwrap().clone();
                    self.store_block_in_LRU_cache(bo_at, &blockp);
                    return ResultS3_ReadBlock::Found(blockp);
                }
                bo_at += 1;
                continue;
            } else {
                debug_eprintln!("{}read_block_FileGz({}): blocks_read.contains({}) missed (does not contain key)", so(), blockoffset, bo_at);
                debug_assert!(!self.blocks.contains_key(&bo_at), "blocks has element {} not in blocks_read", bo_at);
                self.read_blocks_miss += 1;
            }

            // XXX: for some reason large block sizes are more likely to fail `.read`
            //      so do many smaller reads of a size that succeeds more often
            let blocksz_u: usize = self.blocksz_at_blockoffset(&bo_at) as usize;
            // bytes to read in all `.read()` except the last
            // in ad-hoc experiments, this size was found to succeed pretty often
            const READSZ: usize = 1024;
            //const READSZ: usize = 0xFFFF;
            // bytes to read in last `.read()`
            let readsz_last: usize = blocksz_u % READSZ;
            // number of `.read()` of size `READSZ` plus one
            let mut reads: usize = blocksz_u / READSZ + 1;
            let bytes_to_read: usize = (reads - 1) * READSZ + readsz_last;
            debug_assert_eq!(bytes_to_read, blocksz_u, "bad calculation");
            // final block of storage
            let mut block = Block::with_capacity(blocksz_u);
            // intermediate buffer of size `READSZ` for smaller reads
            let mut block_buf = Block::with_capacity(READSZ);
            // XXX: `with_capacity, clear, resize` is a verbose way to create a new vector with a run-time determined `capacity`
            //      and `len`. `len == capacity` is necessary for calls to `decoder.read`.
            //      Using `decoder.read_exact` and `decoder.read_to_end` was more difficult.
            //      See https://github.com/rust-lang/flate2-rs/issues/308
            block.clear();
            block.resize(blocksz_u, 0);
            debug_eprintln!("{}read_block_FileGz({}): blocks_read count {:?}; for blockoffset {}: must do {} reads of {} bytes, and one read of {} bytes (total {} bytes to read) (uncompressed filesz {})", so(), blockoffset, self.blocks_read.len(), bo_at, reads - 1, READSZ, readsz_last, bytes_to_read, self.filesz());
            let mut read_total: usize = 0;
            while reads > 0 {
                reads -= 1;
                let mut readsz: usize = READSZ;
                if reads == 0 {
                    readsz = readsz_last;
                }
                if readsz_last == 0 {
                    break;
                }
                // TODO: [2022/07] cost-savings, use pre-allocated buffer
                block_buf.clear();
                block_buf.resize(readsz, 0);
                debug_eprintln!("{}read_block_FileGz({}): GzDecoder.read(…); read {}, readsz {}, block len {}, block capacity {}, blockoffset {}", so(), blockoffset, reads, readsz, block_buf.len(), block_buf.capacity(), bo_at);
                match (self.gz.as_mut().unwrap().decoder).read(block_buf.as_mut()) {
                    Ok(size_) if size_ == 0 => {
                        debug_eprintln!("{}read_block_FileGz({}): GzDecoder.read() returned Ok({:?}); read_total {}", so(), blockoffset, size_, read_total);
                        
                        let byte_at: FileOffset = self.file_offset_at_block_offset_self(bo_at) + (read_total as FileOffset);
                        // in ad-hoc testing, it was found the decoder never recovers from a zero-byte read
                        return ResultS3_ReadBlock::Err(
                            Error::new(
                                ErrorKind::InvalidData,
                                format!("GzDecoder.read() read zero bytes for vec<u8> buffer of length {}, capacity {}; stuck at inflated byte {}, size {}, size uncompressed {} (calculated from gzip header); {:?}", block_buf.len(), block_buf.capacity(), byte_at, self.filesz, self.filesz_actual, self.path)
                            )
                        );
                        
                    }
                    // read was too large
                    Ok(size_) if size_ > readsz => {
                        debug_eprintln!("{}read_block_FileGz({}): GzDecoder.read() returned Ok({:?}); size too big", so(), blockoffset, size_);
                        return ResultS3_ReadBlock::Err(
                            Error::new(
                                ErrorKind::InvalidData,
                                format!("GzDecoder.read() read too many bytes {} for vec<u8> buffer of length {}, capacity {}; file size {}, file size uncompressed {} (calculated from gzip header); {:?}", size_, block_buf.len(), block_buf.capacity(), self.filesz, self.filesz_actual, self.path)
                            )
                        );
                    }
                    // first or subsequent read is le expected size
                    Ok(size_) => {
                        debug_eprintln!("{}read_block_FileGz({}): GzDecoder.read() returned Ok({:?}), readsz {}, blocksz {}", so(), blockoffset, size_, readsz, blocksz_u);
                        // TODO: cost-savings: use faster `copy_slice`
                        for byte_ in block_buf.iter().take(size_) {
                            block[read_total] = *byte_;
                            read_total += 1;
                        }
                        //debug_assert_eq!(copiedn, size_, "copied {} but read returned size {} (readsz {}) B", copiedn, size_, readsz);
                    }
                    Err(err) => {
                        debug_eprintln!("ERROR: GzDecoder.read(&block (capacity {})) error {} for {:?}", self.blocksz, err, self.path);
                        debug_eprintln!("{}read_block_FileGz({}): return Err({})", sx(), blockoffset, err);
                        return ResultS3_ReadBlock::Err(err);
                    }
                }
                debug_assert_le!(block.len(), blocksz_u, "block.len() {} was expcted to be <= blocksz {}", block.len(), blocksz_u);
            }

            // sanity check: check returned Block is expected number of bytes
            let blocklen_sz: BlockSz = block.len() as BlockSz;
            debug_eprintln!("{}read_block_FileGz({}): block.len() {}, blocksz {}, blockoffset at {}", so(), blockoffset, blocklen_sz, self.blocksz, bo_at);
            if block.is_empty() {
                let byte_at = self.file_offset_at_block_offset_self(bo_at);
                return ResultS3_ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                            "GzDecoder.read() read zero bytes from block {} (at byte {}), requested {} bytes. filesz {}, filesz uncompressed {} (according to gzip header), last block {}; {:?}",
                            bo_at, byte_at, blocksz_u, self.filesz, self.filesz_actual, blockoffset_last, self.path,
                        )
                    )
                );
            } else if bo_at == blockoffset_last {
                // last block, is blocksz correct?
                if blocklen_sz > self.blocksz {
                    let byte_at = self.file_offset_at_block_offset_self(bo_at);
                    return ResultS3_ReadBlock::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "GzDecoder.read() read {} bytes for last block {} (at byte {}) which is larger than block size {} bytes; {:?}",
                                blocklen_sz, bo_at, byte_at, self.blocksz, self.path,
                            )
                        )
                    );            
                }
            } else if blocklen_sz != self.blocksz {
                // not last block, is blocksz correct?
                let byte_at = self.file_offset_at_block_offset_self(bo_at) + blocklen_sz;
                return ResultS3_ReadBlock::Err(
                    Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "GzDecoder.read() read {} bytes for block {} expected to read {} bytes (block size), inflate stopped at byte {}. block last {}, filesz {}, filesz uncompressed {} (according to gzip header); {:?}",
                            blocklen_sz, bo_at, self.blocksz, byte_at, blockoffset_last, self.filesz, self.filesz_actual, self.path,
                        )
                    )
                );
            }

            // store decompressed block
            let blockp = BlockP::new(block);
            self.store_block_in_storage(bo_at, &blockp);
            self.store_block_in_LRU_cache(bo_at, &blockp);
            if bo_at == blockoffset {
                debug_eprintln!("{}read_block_FileGz({}): return Found", sx(), blockoffset);
                return ResultS3_ReadBlock::Found(blockp);
            }
            bo_at += 1;
        } // while bo_at <= blockoffset
        debug_eprintln!("{}read_block_FileGz({}): return Done", sx(), blockoffset);

        ResultS3_ReadBlock::Done
    }

    /// read a block of data from storage for a compressed xz file.
    /// `blockoffset` refers to the uncompressed version of the file.
    ///
    /// Called from `read_block`..
    ///
    /// An `.xz` file must read from beginning to end (cannot jump forward and read).
    /// So read entire file up to passed `blockoffset`, storing each decompressed block
    fn read_block_FileXz(&mut self, blockoffset: BlockOffset) -> ResultS3_ReadBlock {
        debug_eprintln!("{}read_block_FileXz({})", sn(), blockoffset);
        debug_assert_eq!(self.filetype, FileType::FileXz, "wrong FileType {:?} for calling read_block_FileXz", self.filetype);

        let blockoffset_last: BlockOffset = self.blockoffset_last();
        let mut bo_at: BlockOffset = match self.blocks_read.iter().max() {
            Some(bo_) => *bo_,
            None => 0,
        };
        while bo_at <= blockoffset {
            // check `self.blocks_read` (not `self.blocks`) if the Block at `blockoffset`
            // was *ever* read.
            // TODO: [2022/06/18] add another stat tracker for lookups in `self.blocks_read`
            if self.blocks_read.contains(&bo_at) {
                self.read_blocks_hit += 1;
                //debug_eprintln!("{}read_block_FileXz({}): blocks_read.contains({})", so(), blockoffset, bo_at);
                if bo_at == blockoffset {
                    debug_eprintln!("{}read_block_FileXz({}): return Found", sx(), blockoffset);
                    // XXX: this will panic if the key+value in `self.blocks` was dropped
                    //      which could happen during streaming stage
                    let blockp: BlockP = self.blocks.get_mut(&bo_at).unwrap().clone();
                    self.store_block_in_LRU_cache(bo_at, &blockp);
                    return ResultS3_ReadBlock::Found(blockp);
                }
                bo_at += 1;
                continue;
            } else {
                debug_eprintln!("{}read_block_FileXz({}): blocks_read.contains({}) missed (does not contain key)", so(), blockoffset, bo_at);
                debug_assert!(!self.blocks.contains_key(&bo_at), "blocks has element {} not in blocks_read", bo_at);
                self.read_blocks_miss += 1;
            }

            let blocksz_u: usize = self.blocksz_at_blockoffset(&bo_at) as usize;
            let mut block = Block::with_capacity(blocksz_u);
            let mut bufreader: &mut BufReader_Xz = &mut self.xz.as_mut().unwrap().bufreader;
            debug_eprintln!("{}read_block_FileXz: xz_decompress({:?}, block (len {}, capacity {}))", so(), bufreader, block.len(), block.capacity());
            // XXX: xz_decompress may resize the passed `buffer`
            match lzma_rs::xz_decompress(&mut bufreader, &mut block) {
                Ok(_) => {},
                Err(err) => {
                    // XXX: would typically `return Err(err)` but the `err` is of type
                    //      `lzma_rs::error::Error`
                    //      https://docs.rs/lzma-rs/0.2.0/lzma_rs/error/enum.Error.html
                    debug_eprintln!("{}read_block_FileXz: xz_decompress Error, return ResultS3_ReadBlock::Err({:?}) for {:?}", sx(), err, self.path);
                    return ResultS3_ReadBlock::Err(
                        Error::new(
                            ErrorKind::Other,
                            format!("{:?}", err),
                        )
                    );
                }
            }
            debug_eprintln!("{}read_block_FileXz: xz_decompress returned block len {}, capacity {}", so(), block.len(), block.capacity());

            // check returned Block is expected number of bytes
            let blocklen_sz: BlockSz = block.len() as BlockSz;
            if block.is_empty() {
                let byte_at = self.file_offset_at_block_offset_self(bo_at);
                return ResultS3_ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                            "xz_decompress read zero bytes from block {} (at byte {}), requested {} bytes. filesz {}, filesz uncompressed {} (according to gzip header), last block {}; {:?}",
                            bo_at, byte_at, blocksz_u, self.filesz, self.filesz_actual, blockoffset_last, self.path,
                        )
                    )
                );
            } else if bo_at == blockoffset_last {
                // last block, is blocksz correct?
                if blocklen_sz > self.blocksz {
                    let byte_at = self.file_offset_at_block_offset_self(bo_at);
                    return ResultS3_ReadBlock::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "xz_decompress read {} bytes for last block {} (at byte {}) which is larger than block size {} bytes; {:?}",
                                blocklen_sz, bo_at, byte_at, self.blocksz, self.path,
                            )
                        )
                    );
                }
            } else if blocklen_sz != self.blocksz {
                // not last block, is blocksz correct?
                let byte_at = self.file_offset_at_block_offset_self(bo_at) + blocklen_sz;
                return ResultS3_ReadBlock::Err(
                    Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "xz_decompress read only {} bytes for block {} expected to read {} bytes (block size), inflate stopped at byte {}. block last {}, filesz {}, filesz uncompressed {} (according to gzip header); {:?}",
                            blocklen_sz, bo_at, self.blocksz, byte_at, blockoffset_last, self.filesz, self.filesz_actual, self.path,
                        )
                    )
                );
            }

            // store decompressed block
            let blockp = BlockP::new(block);
            self.store_block_in_storage(bo_at, &blockp);
            if bo_at == blockoffset {
                debug_eprintln!("{}read_block_FileXz({}): return Found", sx(), blockoffset);
                return ResultS3_ReadBlock::Found(blockp);
            }
            bo_at += 1;
        }
        debug_eprintln!("{}read_block_FileXz({}): return Done", sx(), blockoffset);

        ResultS3_ReadBlock::Done
    }

    /// read a block of data from a file within a .tar archive file
    /// `blockoffset` refers to the uncompressed/untarred version of the file.
    ///
    /// Called from `read_block`.
    ///
    /// This reads the entire file within the `.tar` file during the first call.
    ///
    /// The big read is due to crate `tar` not providing a method to store `tar::Handle` or
    /// `tar::Entry` due to inter-instance references and explicit lifetimes.
    /// A `tar::Entry` requires explicit lifetime to store. But it also holds a reference
    /// to data within the `tar::Handle`. At least I am unable to figure out with some
    /// method to store the data.
    ///
    fn read_block_FileTar(&mut self, blockoffset: BlockOffset) -> ResultS3_ReadBlock {
        debug_eprintln!("{}read_block_FileTar({})", sn(), blockoffset);
        debug_assert_eq!(self.filetype, FileType::FileTar, "wrong FileType {:?} for calling read_block_FileTar", self.filetype);
        debug_assert_le!(self.count_blocks_processed(), blockoffset, "count_blocks_processed() {}, blockoffset {}; has read_block_FileTar been errantly called?", self.count_blocks_processed(), blockoffset);

        let path_ = self.path.clone();
        let path_std: &Path = Path::new(&path_);
        let mut archive: TarHandle = match BlockReader::open_tar(path_std) {
            Ok(val) => val,
            Err(err) => {
                debug_eprintln!("{}read_block_FileTar Err {:?}", sx(), err);
                return ResultS3_ReadBlock::Err(err);
            }
        };

        // get the file entry from the `.tar` file
        let mut entry = {
            let index_ = self.tar.as_ref().unwrap().entry_index;
            let entry_iter: tar::Entries<File> = match archive.entries_with_seek() {
                Ok(val) => {
                    val
                },
                Err(err) => {
                    debug_eprintln!("{}read_block_FileTar Err {:?}", sx(), err);
                    return ResultS3_ReadBlock::Err(err);
                }
            };
            match entry_iter.skip(index_).next() {
                Some(entry_res) => {
                    match entry_res {
                        Ok(entry) => {
                            entry
                        }
                        Err(err) => {
                            debug_eprintln!("{}read_block_FileTar Err {:?}", sx(), err);
                            return ResultS3_ReadBlock::Err(err);
                        }
                    }
                }
                None => {
                    debug_eprintln!("{}read_block_FileTar None", so());
                    return ResultS3_ReadBlock::Err(
                        Error::new(
                            ErrorKind::UnexpectedEof,
                            format!("tar.handle.entries_with_seek().entry_iter.skip({}).next() returned None", index_)
                        )
                    );
                }
            }
        };

        // read all blocks from file `entry`
        let mut bo_at: BlockOffset = 0;
        let blockoffset_last = self.blockoffset_last();
        while bo_at <= blockoffset_last {
            let cap = self.blocksz_at_blockoffset(&bo_at) as usize;
            let mut block: Block = Block::with_capacity(cap);
            block.resize(cap, 0);
            debug_eprintln!("{}read_block_FileTar: read_exact(&block (capacity {})); bo_at {}", so(), cap, bo_at);
            match entry.read_exact(block.as_mut_slice()) {
                Ok(_) => {}
                Err(err) => {
                        debug_eprintln!("{}read_block_FileTar: read_exact(&block (capacity {})) error, return {:?}", sx(), cap, err);
                        eprintln!("entry.read_exact(&block (capacity {})) path {:?} Error {:?}", cap, path_std, err);
                        return ResultS3_ReadBlock::Err(err);
                }
            }

            // check returned Block is expected number of bytes
            //let blocklen_sz: BlockSz = block.len() as BlockSz;
            if block.is_empty() {
                let byte_at = self.file_offset_at_block_offset_self(bo_at);
                return ResultS3_ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                            "read_exact read zero bytes from block {} (at byte {}), requested {} bytes. filesz {}, last block {}; {:?}",
                            bo_at, byte_at, self.blocksz, self.filesz(), blockoffset_last, self.path,
                        )
                    )
                );
            } else if cap != block.len() {
                let byte_at = self.file_offset_at_block_offset_self(bo_at);
                return ResultS3_ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                            "read_exact read {} bytes from block {} (at byte {}), requested {} bytes. filesz {}, last block {}; {:?}",
                            block.len(), bo_at, byte_at, self.blocksz, self.filesz(), blockoffset_last, self.path,
                        )
                    )
                );
            }

            let blockp: BlockP = BlockP::new(block);
            self.store_block_in_storage(bo_at, &blockp);
            bo_at += 1;
        }

        let blockp: BlockP = match self.blocks.get(&blockoffset) {
            Some(blockp_) => blockp_.clone(),
            None => {
                debug_eprintln!("{}read_block_FileTar: self.blocks.get({}), returned None, return Err(UnexpectedEof)", sx(), blockoffset);
                return ResultS3_ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!("read_block_FileTar: self.blocks.get({}) returned None", blockoffset)
                    )
                );
            }
        };

        debug_eprintln!("{}read_block_FileTar({}): return Found", sx(), blockoffset);

        ResultS3_ReadBlock::Found(blockp)
    }

    /// read a `Block` of data of max size `self.blocksz` from the file.
    /// Successful read returns `Found(BlockP)`.
    ///
    /// When reached the end of the file, and no data was read returns `Done`.
    ///
    /// All other `File` and `std::io` errors are propagated to the caller in `Err`
    pub fn read_block(&mut self, blockoffset: BlockOffset) -> ResultS3_ReadBlock {
        debug_eprintln!(
            "{0}read_block: blockreader.read_block({1}) (fileoffset {2} (0x{2:08X})), blocksz {3} (0x{3:08X}), filesz {4} (0x{4:08X})",
            sn(), blockoffset, self.file_offset_at_block_offset_self(blockoffset), self.blocksz, self.filesz(),
        );
        if blockoffset > self.blockoffset_last() {
            debug_eprintln!("{}read_block({}) is past blockoffset_last {}; return Done", sx(), blockoffset, self.blockoffset_last());
            return ResultS3_ReadBlock::Done;
        }
        { // check storages
            // check fast LRU cache
            if self.read_block_lru_cache_enabled {
                match self.read_block_lru_cache.get(&blockoffset) {
                    Some(bp) => {
                        self.read_block_cache_lru_hit += 1;
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
                        self.read_block_cache_lru_miss += 1;
                        debug_eprintln!("{}read_block: blockoffset {} not found LRU cache", so(), blockoffset);
                    }
                }
            }
            // check hash map storage
            if self.blocks_read.contains(&blockoffset) {
                self.read_blocks_hit += 1;
                debug_eprintln!("{}read_block: blocks_read.contains({})", so(), blockoffset);
                assert!(self.blocks.contains_key(&blockoffset), "requested block {} is in self.blocks_read but not in self.blocks", blockoffset);
                // BUG: during streaming, this might panic!
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
                self.read_blocks_miss += 1;
                debug_eprintln!("{}read_block: blockoffset {} not found in blocks_read", so(), blockoffset);
                debug_assert!(!self.blocks.contains_key(&blockoffset), "blocks has element {} not in blocks_read", blockoffset);
            }
        }

        match self.filetype {
            FileType::File => {
                self.read_block_File(blockoffset)
            },
            FileType::FileGz => {
                self.read_block_FileGz(blockoffset)
            },
            FileType::FileXz => {
                self.read_block_FileXz(blockoffset)
            },
            FileType::FileTar => {
                self.read_block_FileTar(blockoffset)
            },
            _ => {
                panic!("Unsupported filetype {:?}", self.filetype);
            },
        }
    }

    /// wrapper to open a `.tar` file
    fn open_tar(path_tar: &Path) -> Result<TarHandle> {
        let mut open_options = FileOpenOptions::new();
        debug_eprintln!("{}open_tar: open_options.read(true).open({:?})", so(), path_tar);
        let file_tar: File = match open_options.read(true).open(path_tar) {
            Ok(val) => val,
            Err(err) => {
                debug_eprintln!("{}open_tar: open_options.read({:?}) Error, return {:?}", sx(), path_tar, err);
                return Err(err);
            }
        };

        Ok(TarHandle::new(file_tar))
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
