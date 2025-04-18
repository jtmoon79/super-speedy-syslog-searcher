// src/readers/blockreader.rs
// … ‥

//! Implements [`Block`s] and [`BlockReader`], the driver of reading bytes
//! from a file.
//! The `BlockReader` handles reading bytes from various sources including
//! decompressing files (e.g. `.gz`, `.xz`) and reading files within
//! archives (e.g. `.tar`).
//!
//! [`Block`s]: crate::readers::blockreader::Block
//! [`BlockReader`]: crate::readers::blockreader::BlockReader

#[doc(hidden)]
use crate::common::{
    err_from_err_path,
    err_from_err_path_result,
    Count,
    FileTypeArchive,
    FileOffset,
    FileSz,
    FileType,
    FPath,
};
use crate::common::{File, FileMetadata, FileOpenOptions, ResultS3};
#[cfg(test)]
use crate::common::Bytes;
use crate::data::datetime::{
    seconds_to_systemtime,
    SystemTime,
};
#[allow(unused_imports)]
use crate::debug::printers::{de_err, de_wrn, e_err, e_wrn};
use crate::debug_panic;

use std::borrow::Cow;
#[cfg(test)]
use std::collections::HashSet;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::Metadata;
use std::io::prelude::Read;
use std::io::{BufReader, Error, ErrorKind, Result, Seek, SeekFrom, Take};
use std::path::Path;
use std::sync::Arc;

use ::bzip2_rs::DecoderReader as Bz2DecoderReader;
use ::lru::LruCache;
#[allow(unused_imports)]
use ::more_asserts::{
    assert_ge,
    assert_gt,
    assert_le,
    debug_assert_ge,
    debug_assert_gt,
    debug_assert_le,
    debug_assert_lt,
};
// `flate2` is for gzip files.
use ::flate2::read::GzDecoder;
use ::flate2::GzHeader;
// `lz4_flex` is for lz4 files.
use ::lz4_flex;
// `lzma_rs` is for xz files.
use ::lzma_rs;
#[allow(unused_imports)]
use ::si_trace_print::{
    def1n,
    def1o,
    def1x,
    def1ñ,
    def2n,
    def2o,
    def2x,
    defn,
    defo,
    defx,
    defñ,
    den,
    deo,
    dex,
    deñ,
    pfn,
    pfo,
    pfx,
};
// `tar` is for tar files.
use ::tar;


/// [`Block`] Size in bytes.
pub type BlockSz = u64;

/// Byte offset (Index) _into_ a [`Block`] from the beginning of that `Block`.
/// Zero based.
///
pub type BlockIndex = usize;

/// Offset into a file in [`Block`s], depends on [`BlockSz`] runtime value.
/// Zero based.
///
/// [`Block`s]: self::Block
/// [`BlockSz`]: BlockSz
pub type BlockOffset = u64;

/// A _block_ of bytes read from some file.
pub type Block = Vec<u8>;

/// Thread-safe [Atomic Reference Counting Pointer] to a [`Block`].
///
/// [Atomic Reference Counting Pointer]: std::sync::Arc
/// [`Block`]: self::Block
pub type BlockP = Arc<Block>;

/// A sequence of byte slices.
pub type Slices<'a> = Vec<&'a [u8]>;

/// Helper to `BlockReader::read_block`; create a `ResultS3ReadBlock::Err`
/// with an error string that includes the file path
fn err_from_err_path_results3(error: &Error, path: &FPath, mesg: Option<&str>) -> ResultS3ReadBlock
{
    ResultS3ReadBlock::Err(err_from_err_path(error, path, mesg))
}

/// Map of `BlockOffset` to `BlockP` pointers.
pub type Blocks = BTreeMap<BlockOffset, BlockP>;

/// Set of successfully dropped `Block`s
#[cfg(test)]
pub type SetDroppedBlocks = HashSet<BlockOffset>;

/// History of `BlockOffset` that have been read by a [`BlockReader].
pub type BlocksTracked = BTreeSet<BlockOffset>;

/// Internal fast [LRU cache] used by [`BlockReader] in the `read_block` function.
///
/// [LRU cache]: https://docs.rs/lru/0.7.8/lru/index.html
pub type BlocksLRUCache = LruCache<BlockOffset, BlockP>;

/// A typed [`ResultS3`] for function [`BlockReader::read_block`].
///
/// [`ResultS3`]: crate::common::ResultS3
/// [`BlockReader::read_block`]: BlockReader::read_block
// TODO: rename `ResultS3ReadBlock` with `ResultReadBlock`
#[allow(non_upper_case_globals)]
pub type ResultS3ReadBlock = ResultS3<BlockP, Error>;

/// Return type for private function `BlockReader::read_data`.
///
/// Avoids allocating a new `Vec` for common cases where one or two `BlockP`
/// are returned.
pub enum ReadDataParts {
    /// One `BlockP` returned.
    One(BlockP),
    /// Two `BlockP` returned.
    Two(BlockP, BlockP),
    /// Three or more `BlockP` returned (allocates a `Vec`).
    Many(Vec<BlockP>),
}

/// Returned by private function `BlockReader::read_data`.
///
/// A tuple of:
/// - `ReadDataParts` enum
/// - `BlockIndex` of the first `BlockP` in the `ReadDataParts` enum
/// - `BlockIndex` of the last `BlockP` in the `ReadDataParts` enum
// TODO: change to a typed `struct ReadData(...)`
pub type ReadData = (ReadDataParts, BlockIndex, BlockIndex);

/// A typed [`ResultS3`] for private function `BlockReader::read_data`.
///
/// [`ResultS3`]: crate::common::ResultS3
#[allow(non_upper_case_globals)]
pub type ResultReadData = ResultS3<ReadData, Error>;

/// A typed [`ResultS3`] for function [`read_data_to_buffer`].
///
/// [`ResultS3`]: crate::common::ResultS3
/// [`read_data_to_buffer`]: BlockReader#method.read_data_to_buffer
#[allow(non_upper_case_globals)]
pub type ResultReadDataToBuffer = ResultS3<usize, Error>;

/// Helper to `BlockReader::read_data_to_buffer` to check buffer length.
/// In case of error, return from caller function with
/// `ResultReadDataToBuffer::Err`.
macro_rules! read_data_to_buffer_len_check {
    ($arg1:expr, $arg2:expr, $path:expr) => (
        if $arg1 < $arg2 {
            let err = Error::new(
                ErrorKind::Other,
                format!(
                    "buffer too small, len {}, need {} for file {:?}",
                    $arg1, $arg2, $path
                ),
            );
            defx!("return {:?}", err);
            return ResultReadDataToBuffer::Err(err);
        }
    )
}

/// Absolute minimum Block Size in bytes (inclusive).
pub const BLOCKSZ_MIN: BlockSz = 1;

/// Absolute maximum Block Size in bytes (inclusive).
pub const BLOCKSZ_MAX: BlockSz = 0xFFFFFF;

/// Default [`Block`] Size in bytes.
pub const BLOCKSZ_DEF: usize = 0x10000;

/// Data and readers for a bzip2 `.bz2` file, used by [`Bz2DecoderReader`].
///
/// <https://github.com/dsnet/compress/blob/39efe44ab707ffd2c1ef32cc7dbebfe584718686/doc/bzip2-format.pdf>
pub struct Bz2Data {
    /// size of file uncompressed
    pub filesz: FileSz,
    /// calls to `read` use this
    pub decoder:  Bz2DecoderReader<File>,
}

/// Data and readers for a gzip `.gz` file, used by [`BlockReader`].
#[derive(Debug)]
pub struct GzData {
    /// size of file uncompressed, taken from trailing gzip file data.
    /// Users should call `blockreader.filesz()`.
    pub filesz: FileSz,
    /// calls to `read` use this
    pub decoder: GzDecoder<File>,
    /// filename taken from gzip header
    pub filename: String,
    /// file modified time taken from gzip header
    ///
    /// From <https://datatracker.ietf.org/doc/html/rfc1952#page-7>
    ///
    /// > MTIME (Modification TIME)
    /// >
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

type BufReaderLz4 = BufReader<File>;
type Lz4FrameReader = lz4_flex::frame::FrameDecoder<BufReaderLz4>;

#[derive(Debug)]
pub struct Lz4Data {
    pub filesz: FileSz,
    pub reader: Lz4FrameReader,
}

type BufReaderXz = BufReader<File>;

/// Data and readers for a LZMA `.xz` file, used by [`BlockReader`].
#[derive(Debug)]
pub struct XzData {
    /// Size of file uncompressed.
    /// Users should call `blockreader.filesz()`.
    pub filesz: FileSz,
    /// [`BufReader`] to the [`File`] used by [`lzma_rs`] crate.
    ///
    /// [`BufReader`]: std::io::BufReader
    /// [`File`]: std::fs::File
    /// [`lzma_rs`]: https://docs.rs/lzma-rs/0.2.0/lzma_rs/index.html
    pub bufreader: BufReaderXz,
}

/// Separator `char` symbol for a filesystem path and subpath within a
/// compressed file or an archive file. Used by an [`FPath`].
///
/// e.g. `path/logs.tar|logs/syslog`<br/>
/// e.g. `log.xz|syslog`
///
/// [`FPath`]: crate::common::FPath
// TODO: move this to common.rs
// TODO: Issue #7
//       it is not impossible for paths to have '|', use '\0' instead
//       is even less likely to be in a path. Use '|' when printing paths.
pub const SUBPATH_SEP: char = '|';

/// crate `tar` handle for a plain `File`.
pub type TarHandle = tar::Archive<File>;

/// _Checksum_ copied from [`tar::Archive::<File>::headers()`].
///
/// Described at <https://www.gnu.org/software/tar/manual/html_node/Standard.html>
/// under `char chksum[8];`.
///
/// [`tar::Archive::<File>::headers()`]: https://docs.rs/tar/0.4.38/tar/struct.Header.html
pub type TarChecksum = u32;

/// _Modified Systemtime_ copied [`tar::Archive::<File>::headers()`].
///
/// Described at <https://www.gnu.org/software/tar/manual/html_node/Standard.html>
/// under `char mtime[12];`.
///
/// [`tar::Archive::<File>::headers()`]: https://docs.rs/tar/0.4.38/tar/struct.Header.html
pub type TarMTime = u64;

/// Data and readers for a file within a `.tar` file, used by [`BlockReader`].
pub struct TarData {
    /// Size of the file unarchived.
    pub filesz: FileSz,
    /// Iteration count of [`tar::Archive::entries_with_seek`].
    ///
    /// [`tar::Archive::entries_with_seek`]: https://docs.rs/tar/0.4.38/tar/struct.Archive.html#method.entries_with_seek
    pub entry_index: usize,
    /// Checksum retrieved from tar header.
    pub checksum: TarChecksum,
    /// Modified systemtime retrieved from tar header.
    ///
    /// From <https://www.gnu.org/software/tar/manual/html_node/Standard.html>
    ///
    /// > The mtime field represents the data modification time of the file at
    /// > the time it was archived. It represents the integer number of seconds
    /// > since January 1, 1970, 00:00 Coordinated Universal Time.
    ///
    pub mtime: TarMTime,
}

/// A `BlockReader` reads a file in [`BlockSz`] byte-sized [`Block`s]. It
/// interfaces with the filesystem. It handles reading from specialized
/// storage files like
/// compressed files (e.g. `.gz`, `.xz`) and archive files (e.g `.tar`).
/// `BlockReader` handles the run-time in-memory storage the `Block`s of data
/// it reads.
///
/// A `BlockReader` uses the passed [`FileType`] to determine how to handle
/// files.
/// This includes reading bytes from files (e.g. `.log`),
/// compressed files (e.g. `.gz`, `.xz`), and archive files (e.g. `.tar`).
///
/// One `BlockReader` corresponds to one file. For archive files and
/// compressed files, one `BlockReader` handles only one file *within*
/// the archive or compressed file.
///
/// A `BlockReader` does not know about `char`s, only bytes `u8`.
///
/// _XXX: not a rust "Reader"; does not implement trait [`Read`]._
///
/// [`Block`s]: self::Block
/// [`FileType`]: crate::common::FileType
/// [`LineReader`]: crate::readers::linereader::LineReader
/// [`Read`]: std::io::Read
/// [`BlockSz`]: BlockSz
pub struct BlockReader {
    /// Path to the file.
    path: FPath,
    /// Subpath to file. Only for `filetype.is_archived()` files.
    // XXX: Relates to Issue #7
    // TODO: rename `_subpath` to `subpath`
    _subpath: Option<FPath>,
    /// Combined `path`, Separator, and `_subpath`. When there is no `_subpath`
    /// it is only `path`.
    path_subpath: FPath,
    /// The file handle.
    file_handle: File,
    /// A copy of [`File.metadata()`].
    /// For compressed or archived files, the metadata of the `path`
    /// compress or archive file.
    ///
    /// [`File.metadata()`]: std::fs::Metadata
    file_metadata: FileMetadata,
    /// A copy of [`self.file_metadata.modified()`].
    /// Copied during function `new()`.
    ///
    /// To simplify later retrievals.
    ///
    /// [`self.file_metadata.modified()`]: std::fs::Metadata
    pub(crate) file_metadata_modified: SystemTime,
    /// Enum that guides file-handling behavior in functions `read`, and `new`.
    filetype: FileType,
    /// Do or do not drop `Block`s or other data after they are processed and
    /// printed.
    ///
    /// Only intended to be `false` for a special case of compressed syslog
    /// files without a year in the datetimestamp.
    ///
    /// For syslog files without a year in the datetimestamp, the strategy to
    /// determine the year requires reading the end of the file, then
    /// reading the syslines backwards. However, when `is_streamed_file` is
    /// `true` the `Block`s are dropped after they are read and possibly
    /// printed (`is_streamed_file` will be `true` for compressed files).
    /// But in this special case of processing the file backwards,
    /// the `Block`s should be retained as they will later be printed later.
    /// Otherwise, the reprocessing time would _O(n²)_.
    /// The tradeoff is memory usage is _O(n)_.
    /// See [`process_missing_year`]
    ///
    /// For non-compressed files, it is okay to drop `Block`s after they are
    /// read and then later read them again. This avoids memory usage of _O(n)_.
    ///
    /// Use `fn disable_drop_data` to set `drop_data` to `false`.
    ///
    /// [`process_missing_year`]: crate::readers::syslogprocessor::SyslogProcessor#method.process_missing_year
    drop_data: bool,
    /// For bzipped `.bz2` files ([FileTypeArchive::Bz2]), otherwise `None`.
    bz2: Option<Bz2Data>,
    /// For gzipped `.gz` files ([FileTypeArchive::Gz]), otherwise `None`.
    gz: Option<GzData>,
    /// For LZ4 compressed files ([FileTypeArchive::Lz4]), otherwise `None`.
    lz: Option<Lz4Data>,
    /// For LZMA xz `.xz` files ([FileTypeArchive::Xz]), otherwise `None`.
    xz: Option<XzData>,
    /// For files within a `.tar` file ([FileTypeArchive::Tar]), otherwise `None`.
    tar: Option<TarData>,
    /// The filesz of uncompressed data, set during `new`.
    /// Users should always call `filesz()`.
    pub(crate) filesz_actual: FileSz,
    /// File size in bytes of file at `self.path`, actual size.
    /// Users should always call `filesz()`.
    ///
    /// For compressed files, this is the size of the file compressed.
    /// For the uncompressed size of a compressed file, see `filesz_actual`.
    /// Set in `open`.
    ///
    /// For regular files (not compressed or archived),
    /// `filesz` and `filesz_actual` will be the same.
    pub(crate) filesz: FileSz,
    /// File size in `Block`s, set in `open`.
    pub(crate) blockn: u64,
    /// Standard `Block` size in bytes. All `Block`s are this size except the
    /// last `Block` which may this size or smaller (and not zero).
    pub(crate) blocksz: BlockSz,
    /// last requested `blockoffset` in `fn read_block()`.
    read_block_last: BlockOffset,
    /// `Count` of bytes read by the `BlockReader`.
    count_bytes_read: Count,
    /// Storage of blocks `read` from storage. Lookups O(log(n)). May `drop`
    /// data.
    ///
    /// During file processing, some elements that are not needed may be
    /// `drop`ped.
    blocks: Blocks,
    /// Track blocks read in `read_block`. Never drops data.
    ///
    /// Useful for when [streaming] kicks-in and some key+value of `self.blocks`
    /// have been dropped.
    ///
    /// _XXX:_ memory usage: O(n); one entry per Block read
    ///
    /// [streaming]: crate::readers::syslogprocessor::ProcessingStage#variant.Stage3StreamSyslines
    blocks_read: BlocksTracked,
    /// Internal [LRU cache] for `fn read_block()`. Lookups _O(1)_.
    ///
    /// [LRU cache]: https://docs.rs/lru/0.7.8/lru/index.html
    read_block_lru_cache: BlocksLRUCache,
    /// Enable or disable use of `read_block_lru_cache`.
    ///
    /// Users should call functions `LRU_cache_enable` or `LRU_cache_disable`.
    read_block_lru_cache_enabled: bool,
    /// Internal LRU cache `Count` of lookup hits.
    pub(crate) read_block_cache_lru_hit: Count,
    /// Internal LRU cache `Count` of lookup misses.
    pub(crate) read_block_cache_lru_miss: Count,
    /// Internal LRU cache `Count` of lookup `.put`.
    pub(crate) read_block_cache_lru_put: Count,
    /// Internal storage `Count` of lookup hit during `fn read_block()`.
    pub(crate) read_blocks_hit: Count,
    /// Internal storage `Count` of lookup miss during `fn read_block()`.
    pub(crate) read_blocks_miss: Count,
    /// Internal storage `Count` of calls to `self.blocks.insert`.
    pub(crate) read_blocks_put: Count,
    /// Internal storage `Count` of previously read block being read again.
    pub(crate) read_blocks_reread_error: Count,
    /// Internal tracking of "high watermark" of `self.blocks` size
    pub(crate) blocks_highest: usize,
    /// Internal count of `Block`s dropped
    pub(crate) dropped_blocks_ok: Count,
    /// Internal count of `Block`s dropped failed
    pub(crate) dropped_blocks_err: Count,
    /// Internal memory of blocks dropped.
    #[cfg(test)]
    pub(crate) dropped_blocks: SetDroppedBlocks,
}

impl fmt::Debug for BlockReader {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("BlockReader")
            .field("path", self.path())
            .field("file_handle", &self.file_handle)
            .field("filesz", &self.filesz())
            .field("blockn", &self.blockn)
            .field("blocksz", &self.blocksz)
            .field("drop_data", &self.drop_data)
            .field("blocks currently stored", &self.blocks.len())
            .field("blocks read", &self.blocks_read.len())
            .field("bytes read", &self.count_bytes_read)
            .field("cache LRU hit", &self.read_block_cache_lru_hit)
            .field("miss", &self.read_block_cache_lru_miss)
            .field("put", &self.read_block_cache_lru_put)
            .field("cache hit", &self.read_blocks_hit)
            .field("miss", &self.read_blocks_miss)
            .field("insert", &self.read_blocks_put)
            .finish()
    }
}

// TODO: [2023/04] remove redundant variable prefix name `blockreader_`
// TODO: [2023/05] instead of having 1:1 manual copying of `BlockReader`
//       fields to `SummaryBlockReader` fields, just store a
//       `SummaryBlockReader` in `BlockReader` and update directly.
#[allow(non_snake_case)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SummaryBlockReader {
    pub blockreader_bytes: Count,
    pub blockreader_bytes_total: FileSz,
    pub blockreader_blocks: Count,
    pub blockreader_blocks_total: Count,
    pub blockreader_blocksz: BlockSz,
    pub blockreader_filesz: FileSz,
    pub blockreader_filesz_actual: FileSz,
    pub blockreader_read_block_lru_cache_hit: Count,
    pub blockreader_read_block_lru_cache_miss: Count,
    pub blockreader_read_block_lru_cache_put: Count,
    pub blockreader_read_blocks_hit: Count,
    pub blockreader_read_blocks_miss: Count,
    pub blockreader_read_blocks_put: Count,
    pub blockreader_read_blocks_reread_error: Count,
    pub blockreader_blocks_highest: usize,
    pub blockreader_blocks_dropped_ok: Count,
    pub blockreader_blocks_dropped_err: Count,
}

/// Helper to unpack DWORD unsigned integers in a gzip header.
///
/// The rust built-in [`u32::from_be_bytes`] and [`u32::from_le_bytes`] failed
/// for test file compressed with GNU gzip 1.10.
///
/// [`u32::from_be_bytes`]: https://doc.rust-lang.org/std/primitive.u32.html#method.from_be_bytes
/// [`u32::from_le_bytes`]: https://doc.rust-lang.org/std/primitive.u32.html#method.from_le_bytes
const fn dword_to_u32(buf: &[u8; 4]) -> u32 {
    let mut buf_: [u8; 4] = [0; 4];
    buf_[0] = buf[3];
    buf_[1] = buf[2];
    buf_[2] = buf[1];
    buf_[3] = buf[0];

    u32::from_be_bytes(buf_)
}

/// Implements the `BlockReader`.
impl BlockReader {
    /// Maximum size of a gzip compressed file that will be processed.
    /// 0x20000000 is 512MB.
    ///
    /// XXX: The gzip standard stores uncompressed "media stream" bytes size in
    ///      within 32 bits, 4 bytes. An uncompressed size greater than
    ///      0xFFFFFFFF will store the modulo.
    ///      So there is no certain way to determine the size of the
    ///      "media stream".
    ///      This terrible hack just aborts processing .gz files that might be
    ///      over size 0xFFFFFFFF (undercuts the size).
    ///      Issue #8
    const GZ_MAX_SZ: FileSz = 0x20000000;

    /// Cache slots for `read_block` LRU cache.
    const READ_BLOCK_LRU_CACHE_SZ: usize = 4;

    /// Default state of LRU cache.
    const CACHE_ENABLE_DEFAULT: bool = true;

    /// "streamed" files will drop "old" Blocks during `read_block()`
    const READ_BLOCK_LOOKBACK_DROP: bool = true;

    /// Create a new `BlockReader`.
    ///
    /// Opens the file at `path`. Configures settings based on passed
    /// `filetype`.
    ///
    /// **NOTE:** Dissimilar to other `*Readers::new`, this `new` function may
    /// read some bytes from the file, e.g. gzip header, e.g. tar header, to
    /// determine the file type and set other initial values.
    pub fn new(
        path: FPath,
        filetype: FileType,
        blocksz_: BlockSz,
    ) -> Result<BlockReader> {
        def1n!("({:?}, {:?}, {:?})", path, filetype, blocksz_);

        assert_ne!(0, blocksz_, "Block Size cannot be 0");
        assert_ge!(blocksz_, BLOCKSZ_MIN, "Block Size {} is too small", blocksz_);
        assert_le!(blocksz_, BLOCKSZ_MAX, "Block Size {} is too big", blocksz_);

        // shadow passed immutable with local mutable
        let mut path: FPath = path;
        let mut subpath_opt: Option<FPath> = None;
        let path_subpath: FPath = path.clone();
        if filetype.is_archived() {
            def1o!("filetype.is_archived()");
            // split the passed path into the path and subpath
            let (path_, subpath_) = match path.rsplit_once(SUBPATH_SEP) {
                Some(val) => val,
                None => {
                    def1x!(
                        "filetype {:?} but failed to find delimiter {:?} in {:?}",
                        filetype,
                        SUBPATH_SEP,
                        path
                    );
                    return Result::Err(Error::new(
                        // TODO: TRACKING: use `ErrorKind::InvalidFilename` when it is stable
                        //       <https://github.com/rust-lang/rust/issues/86442>
                        ErrorKind::InvalidInput,
                        format!(
                            "Given Filetype {:?} but failed to find delimiter {:?} in {:?}",
                            filetype, SUBPATH_SEP, path
                        ),
                    ));
                }
            };
            subpath_opt = Some(subpath_.to_string());
            path = FPath::from(path_);
            def1o!("self.path is now {:?}", path);
            def1o!("self.path_subpath is now {:?}", path_subpath);
        }
        // shadow mutable variable with immutable
        // XXX: does the compiler optimize this extra `clone` alloc?
        let path = path.clone();
        let path_std: &Path = Path::new(&path);

        let mut open_options = FileOpenOptions::new();
        def1o!("open_options.read(true).open({:?})", path);
        let file_handle: File = match open_options
            .read(true)
            .open(path_std)
        {
            Ok(val) => val,
            Err(err) => {
                def1x!("return {:?}", err);
                return err_from_err_path_result::<BlockReader>(&err, &path, None);
            }
        };
        let mut blocks = Blocks::new();
        let mut blocks_read = BlocksTracked::new();
        let mut count_bytes_read: Count = 0;
        def1o!("count_bytes_read={}", count_bytes_read);
        let filesz: FileSz;
        let mut filesz_actual: FileSz;
        let blocksz: BlockSz;
        let file_metadata: FileMetadata;
        let file_metadata_modified: SystemTime = match file_handle.metadata() {
            Ok(val) => {
                filesz = val.len() as FileSz;
                file_metadata = val;
                match file_metadata.modified() {
                    Ok(systemtime_) => systemtime_,
                    Err(err) => {
                        def1x!("file_metadata.modified() failed Err {:?}", err);
                        return Result::Err(err);
                    }
                }
            }
            Err(err) => {
                def1x!("return {:?}", err);
                return Err(err_from_err_path(&err, &path, None));
            }
        };
        if file_metadata.is_dir() {
            def1x!("return Err(Unsupported)");
            return std::result::Result::Err(Error::new(
                //ErrorKind::IsADirectory,  // XXX: error[E0658]: use of unstable library feature 'io_error_more'
                ErrorKind::Unsupported,
                format!("Path is a directory {:?}", path),
            ));
        }

        let mut bz2_opt: Option<Bz2Data> = None;
        let mut gz_opt: Option<GzData> = None;
        let mut lz_opt: Option<Lz4Data> = None;
        let mut xz_opt: Option<XzData> = None;
        let mut tar_opt: Option<TarData> = None;
        let mut read_blocks_put: Count = 0;

        match filetype {
            FileType::Evtx{ .. } => {
                panic!("BlockerReader::new FileType::Evtx does not use a BlockReader")
            },
            FileType::Journal { .. } => {
                panic!("BlockerReader::new FileType::Journal does not use a BlockReader")
            }
            FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, .. }
            | FileType::Text{ archival_type: FileTypeArchive::Normal, .. }
            => {
                filesz_actual = filesz;
                blocksz = blocksz_;
            }
            FileType::FixedStruct{ archival_type: FileTypeArchive::Bz2, .. }
            | FileType::Text{ archival_type: FileTypeArchive::Bz2, .. }
            => {
                blocksz = blocksz_;
                def1o!("Bz2");

                // bzip2 file structure:
                //      https://github.com/dsnet/compress/blob/39efe44ab707ffd2c1ef32cc7dbebfe584718686/doc/bzip2-format.pdf
                //
                // StreamHeader := HeaderMagic Version Level
                //
                // The stream header is guaranteed to be byte-aligned, and its values can be read as regular bytes from the
                // underlying stream. The HeaderMagic is the 2 byte string: []byte{'B', 'Z'}, or also the value 0x425a
                // when read as a 16-bit integer. The Version is always the byte 'h', or also the value 0x68 when read as
                // an 8-bit integer. The BZip1 format used a version byte of '0'; that format is deprecated and is not
                // discussed in this document.
                //
                // The Level is a single byte between the values of '1' and '9', inclusively. This ASCII character, when
                // parsed as an integer and multiplied by , determines the size of the buffer used in the BWT stage.
                //
                // Thus, level '1' uses a byte buffer, while level '9' uses a byte buffer. When encoding,
                // the RLE1 stage must be careful that it is still able to encode the count byte if the trailing 4 bytes in the
                // buffer are duplicates. Since this determines the buffer size of the BWT stage, it is entirely possible for a
                // stream block to decompress to more bytes than the buffer size due to the RLE1 stage

                // sanity check file size
                // size of `empty.bz2` is 14 bytes
                if filesz < 12 {
                    def1x!("Bz2: return Err(InvalidData)");
                    return Result::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!("bzip2 file size {:?} is too small for {:?}", filesz, path),
                        )
                    );
                }

                let file_handle2: File = match open_options
                    .read(true)
                    .open(path_std)
                {
                    Ok(val) => val,
                    Err(err) => {
                        def1x!("return {:?}", err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, None);
                    }
                };
                def1o!("Bz2: file_handle2 {:?}", file_handle2);
                let mut bz2_decoder = Bz2DecoderReader::new(file_handle2);

                // XXX: the size of the BZ2 data is only known after decompression
                //      See Issue #XXX
                let mut filesz_uncompressed: FileSz = 0;
                const BUF_SZ: usize = 32786;
                let mut buf: [u8; BUF_SZ] = [0; BUF_SZ];
                let mut _loop_count: usize = 0;
                loop {
                    def1o!("Bz2: {:3} bz2_decoder.read(buf size {})", _loop_count, BUF_SZ);
                    match bz2_decoder.read(&mut buf) {
                        Ok(sz) => {
                            if sz == 0 {
                                break;
                            }
                            count_bytes_read += sz as Count;
                            def1o!("Bz2: count_bytes_read={}", count_bytes_read);
                            filesz_uncompressed += sz as FileSz;
                        }
                        Err(err) => {
                            def1x!("Bz2: bz2_decoder.read() Error, return {:?}", err);
                            return Err(
                                Error::new(
                                    ErrorKind::InvalidData,
                                    format!("bz2_decoder.read({}) {}", BUF_SZ, err),
                                )
                            );
                        }
                    }
                    _loop_count += 1;
                }
                def1o!("Bz2: filesz_uncompressed {}", filesz_uncompressed);
                filesz_actual = filesz_uncompressed;

                let file_handle3: File = match open_options
                    .read(true)
                    .open(path_std)
                {
                    Ok(val) => val,
                    Err(err) => {
                        def1x!("return {:?}", err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, None);
                    }
                };
                def1o!("Bz2: file_handle3 {:?}", file_handle3);

                bz2_opt = Some(Bz2Data {
                    filesz: filesz_uncompressed,
                    decoder: Bz2DecoderReader::new(file_handle3),
                });
                def1o!("Bz2: created bz2_opt");
            }
            FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, .. }
            | FileType::Text{ archival_type: FileTypeArchive::Gz, .. }
            => {
                // TODO: [2023/04] move this large chunk of code into private function
                // TODO: [2024/05] most code in this block repeats code in
                //       filedecompressor.rs. Both should be refactored to reduce
                //       duplicate code.
                blocksz = blocksz_;
                def1o!("FileGz: blocksz set to {0} (0x{0:08X}) (passed {1} (0x{1:08X})", blocksz, blocksz_);

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
                    def1x!("FileGz: return Err(InvalidData)");
                    return Result::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!("gzip file size {:?} is too small for {:?}", filesz, path),
                        )
                    );
                }
                // TODO: Issue #9 [2022/06] it's known that for a file larger than 4GB uncompressed,
                //       gzip cannot store it's filesz accurately, since filesz is stored within 32 bits.
                //       gzip will only store the rollover (uncompressed filesz % 4GB).
                //       How to handle large gzipped files correctly?
                //       First, how to detect that the stored filesz is a rollover value?
                //       Second, the file could be streamed and the filesz calculated from that
                //       activity. However, streaming, for example, a 3GB log.gz that decompresses to
                //       10GB is very inefficient.
                //       Third, similar to "Second" but for very large files that are too large
                //       to hold in memory, i.e. a 64GB log.gz file, what then?
                if filesz > BlockReader::GZ_MAX_SZ {
                    def1x!("FileGz: return Err(InvalidData)");
                    return Result::Err(
                        Error::new(
                            // TODO: Issue #10 [2022/06] use `ErrorKind::FileTooLarge` when it is stable
                            //       `ErrorKind::FileTooLarge` causes error:
                            //       use of unstable library feature 'io_error_more'
                            // TRACKING: see issue #86442 <https://github.com/rust-lang/rust/issues/86442>
                            ErrorKind::InvalidData,
                            format!("Cannot handle gzip files larger than semi-arbitrary {0} (0x{0:08X}) uncompressed bytes, file is {1} (0x{1:08X}) uncompressed bytes according to gzip header {2:?}", BlockReader::GZ_MAX_SZ, filesz, path),
                        )
                    );
                }

                // create "take handler" that will read 8 bytes as-is (no decompression)
                match (&file_handle).seek(SeekFrom::End(-8)) {
                    Ok(_) => {}
                    Err(err) => {
                        def1x!("FileGz: return Err({})", err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("first take handler"));
                    }
                };
                let mut reader: Take<&_> = (&file_handle).take(8);

                // extract DWORD for CRC32
                let mut buffer_crc32: [u8; 4] = [0; 4];
                def1o!("FileGz: reader.read_exact(@{:p}) (buffer len {})", &buffer_crc32, buffer_crc32.len());
                match reader.read_exact(&mut buffer_crc32) {
                    Ok(_) => {}
                    Err(err) => {
                        def1x!("FileGz: return {:?}", err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("DWORD for CRC32"));
                    }
                }
                def1o!("FileGz: buffer_crc32 {:?}", buffer_crc32);
                let crc32 = dword_to_u32(&buffer_crc32);
                def1o!("FileGz: crc32 {0} (0x{0:08X})", crc32);

                // extract DWORD for SIZE
                let mut buffer_size: [u8; 4] = [0; 4];
                def1o!("FileGz:  reader.read_exact(@{:p}) (buffer len {})", &buffer_size, buffer_size.len());
                match reader.read_exact(&mut buffer_size) {
                    Ok(_) => {}
                    Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {}
                    Err(err) => {
                        def1x!("FileGz: return {:?}", err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("DWORD for SIZE"));
                    }
                }
                def1o!("FileGz: buffer_size {:?}", buffer_size);
                let size: u32 = dword_to_u32(&buffer_size);
                def1o!("FileGz: file size uncompressed {0:?} (0x{0:08X})", size);
                let filesz_uncompressed: FileSz = size as FileSz;
                filesz_actual = filesz_uncompressed;

                // reset Seek pointer
                match (&file_handle).seek(SeekFrom::Start(0)) {
                    Ok(_) => {}
                    Err(err) => {
                        defx!("FileGz: return Err({:?})", err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("reset Seek pointer"));
                    }
                }

                def1o!("FileGz: open_options.read(true).open({:?})", path_std);
                let file_gz: File = match open_options
                    .read(true)
                    .open(path_std)
                {
                    Ok(val) => val,
                    Err(err) => {
                        def1x!("FileGz: open_options.read({:?}) Error, return {:?}", path, err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("second open failed"));
                    }
                };
                let decoder: GzDecoder<File> = GzDecoder::new(file_gz);
                def1o!("FileGz: {:?}", decoder);
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
                        let filename_: &[u8] = header
                            .filename()
                            .unwrap_or(&[]);
                        filename = match String::from_utf8(filename_.to_vec()) {
                            Ok(val) => val,
                            Err(_err) => String::with_capacity(0),
                        };
                        mtime = header.mtime();
                    }
                    None => {
                        def1o!("FileGz: GzDecoder::header() is None for {:?}", path);
                    }
                };
                def1o!("FileGz: filename {:?}", filename);
                def1o!("FileGz: mtime {:?}", mtime);

                gz_opt = Some(GzData {
                    filesz: filesz_uncompressed,
                    decoder,
                    filename,
                    mtime,
                    crc32,
                });
                def1o!("FileGz: created {:?}", gz_opt);
            }
            FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, .. }
            | FileType::Text{ archival_type: FileTypeArchive::Lz4, .. }
            => {
                // TODO: [2023/04] move this large chunk of code into private function
                blocksz = blocksz_;
                def1o!("FileLz4: blocksz set to {0} (0x{0:08X}) (passed {1} (0x{1:08X})", blocksz, blocksz_);
                def1o!("FileLz4: open_options.read(true).open({:?})", path_std);
                let file_lz: File = match open_options
                    .read(true)
                    .open(path_std)
                {
                    Ok(val) => val,
                    Err(err) => {
                        def1x!("FileLz4 open_options.read({:?}) Error, return {:?}", path, err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("FileXz open failed"));
                    }
                };
                def1o!("FileLz4: created {:?}", file_lz);
                let bufreader: BufReaderLz4 = BufReaderLz4::new(file_lz);
                def1o!("FileLz4: created {:?}", bufreader);

                // XXX: crate `lz4_flex` does not provide an API to retrieve the
                //      decompressed file size.
                //      So read the entire file to get the size.
                //      This is very inefficient and but it avoids OOM errors.
                //      See <https://github.com/PSeitz/lz4_flex/issues/159>
                //      See Issue #293
                let mut lz4_decoder: Lz4FrameReader = Lz4FrameReader::new(bufreader);
                let mut filesz_uncompressed: FileSz = 0;
                const BUF_SZ: usize = 32786;
                let mut buf: [u8; BUF_SZ] = [0; BUF_SZ];
                let mut _loop_count: usize = 0;
                loop {
                    def1o!("FileLz4: {:3} lz4_decoder.read(buf size {})", _loop_count, BUF_SZ);
                    match lz4_decoder.read(&mut buf) {
                        Ok(sz) => {
                            if sz == 0 {
                                break;
                            }
                            count_bytes_read += sz as Count;
                            def1o!("FileLz4: count_bytes_read={}", count_bytes_read);
                            filesz_uncompressed += sz as FileSz;
                        }
                        Err(err) => {
                            def1x!("FileLz4: lz4_decoder.read() Error, return {:?}", err);
                            return Err(
                                Error::new(
                                    ErrorKind::InvalidData,
                                    format!("lz4_decoder.read({}) {}", BUF_SZ, err),
                                )
                            );
                        }
                    }
                    _loop_count += 1;
                }
                def1o!("FileLz4: filesz_uncompressed {}", filesz_uncompressed);
                filesz_actual = filesz_uncompressed;

                // recreate the `lz4_decoder` so it is at the start of the file

                let file_lz: File = match open_options
                    .read(true)
                    .open(path_std)
                {
                    Ok(val) => val,
                    Err(err) => {
                        def1x!("FileLz4: open_options.read({:?}) Error, return {:?}", path, err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("FileXz open failed"));
                    }
                };
                def1o!("FileLz4: {:?}", file_lz);
                let bufreader: BufReaderLz4 = BufReaderLz4::new(file_lz);
                def1o!("FileLz4: {:?}", bufreader);
                let lz4_decoder: Lz4FrameReader = Lz4FrameReader::new(bufreader);
                def1o!("FileLz4: {:?}", lz4_decoder);

                lz_opt = Some(Lz4Data {
                    filesz: filesz_uncompressed,
                    reader: lz4_decoder,
                });
            }
            FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, .. }
            | FileType::Text{ archival_type: FileTypeArchive::Tar, .. }
            => {
                // TODO: [2024/05] most code in this block repeats code in
                //       filedecompressor.rs. Both should be refactored to reduce
                //       duplicate code.
                blocksz = blocksz_;
                def1o!("FileTar: blocksz set to {0} (0x{0:08X}) (passed {1} (0x{1:08X})", blocksz, blocksz_);
                filesz_actual = 0;
                let mut checksum: TarChecksum = 0;
                let mut mtime: TarMTime = 0;
                let subpath: &String = subpath_opt.as_ref().unwrap();

                // open the .tar file

                let mut archive: TarHandle = BlockReader::open_tar(path_std)?;
                let entry_iter: tar::Entries<File> = match archive.entries_with_seek() {
                    Ok(val) => val,
                    Err(err) => {
                        def1x!("FileTar: Err {:?}", err);
                        return Result::Err(err);
                    }
                };

                // in the .tar file, find the entry with the matching subpath

                let mut entry_index: usize = 0;
                for (index, entry_res) in entry_iter.enumerate() {
                    entry_index = index;
                    def1o!("FileTar: entry_index {}", entry_index);
                    let entry: tar::Entry<File> = match entry_res {
                        Ok(val) => val,
                        Err(_err) => {
                            def1o!("FileTar: entry Err {:?}", _err);
                            continue;
                        }
                    };
                    let subpath_cow: Cow<Path> = match entry.path() {
                        Ok(val) => val,
                        Err(_err) => {
                            def1o!("FileTar: entry.path() Err {:?}", _err);
                            continue;
                        }
                    };
                    let subfpath: FPath = subpath_cow
                        .to_string_lossy()
                        .to_string();
                    if subpath != &subfpath {
                        def1o!("FileTar: skip {:?}", subfpath);
                        continue;
                    }
                    // found the matching subpath
                    def1o!("FileTar: found {:?}", subpath);
                    filesz_actual = match entry.header().size() {
                        Ok(val) => val,
                        Err(err) => {
                            def1x!("FileTar: entry.header().size() Err {:?}", err);
                            return err_from_err_path_result::<BlockReader>(&err, &path, Some("(FileTar header().size)"));
                        }
                    };
                    def1o!("FileTar: filesz_actual {:?}", filesz_actual);
                    checksum = match entry.header().cksum() {
                        Ok(val) => val,
                        Err(_err) => {
                            def1o!("FileTar: entry.header().cksum() Err {:?}", _err);

                            0
                        }
                    };
                    def1o!("FileTar: checksum 0x{:08X}", checksum);
                    mtime = match entry.header().mtime() {
                        Ok(val) => val,
                        Err(_err) => {
                            def1o!("FileTar: entry.header().mtime() Err {:?}", _err);

                            0
                        }
                    };
                    def1o!("FileTar: mtime {:?}", mtime);
                    break;
                }

                tar_opt = Some(TarData {
                    filesz: filesz_actual,
                    entry_index,
                    checksum,
                    mtime,
                });
            }
            FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, .. }
            | FileType::Text{ archival_type: FileTypeArchive::Xz, .. }
            => {
                // TODO: [2023/04] move this large chunk of code into private function
                blocksz = blocksz_;
                def1o!("FileXz: blocksz set to {0} (0x{0:08X}) (passed {1} (0x{1:08X})", blocksz, blocksz_);
                def1o!("FileXz: open_options.read(true).open({:?})", path_std);
                let mut file_xz: File = match open_options
                    .read(true)
                    .open(path_std)
                {
                    Ok(val) => val,
                    Err(err) => {
                        def1x!("FileXz: open_options.read({:?}) Error, return {:?}", path, err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("FileXz open failed"));
                    }
                };

                //
                // Get the .xz file size from XZ header
                //
                // "bare-bones" implementation of reading xz compressed file
                // other available crates for reading `.xz` files did not meet
                // the needs of this program.
                //

                // TODO: Issue #11 handle multi-stream xz files
                // TODO: Issue #12 read xz by requested block offset

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
                    Ok(_) => {}
                    Err(err) => {
                        def1x!("FileXz: return A Err({})", err);
                        return err_from_err_path_result::<BlockReader>(
                            &err,
                            &path,
                            Some("FileXz second SeekFrom::Start(0) failed"),
                        );
                    }
                };
                // XXX: not sure if a Take handler is really necessary
                // XXX: not sure if 1024 is correct amount but it seems like enough plus a large
                //      margin
                let mut reader: Take<&File> = (&file_xz).take(1024);

                /*
                    2.1.1. Stream Header

                        +---+---+---+---+---+---+-------+------+--+--+--+--+
                        |  Header Magic Bytes   | Stream Flags |   CRC32   |
                        +---+---+---+---+---+---+-------+------+--+--+--+--+
                 */
                // Stream Header is 12 bytes

                // stream header 6 magic bytes
                let mut buffer_: [u8; 6] = [0; 6];
                def1o!("FileXz: reader.read_exact({})", buffer_.len());
                match reader.read_exact(&mut buffer_) {
                    Ok(_) => {}
                    Err(err) => {
                        de_err!(
                            "FileXz: reader.read_exact() (stream header magic bytes) path {:?} {}",
                            path_std, err
                        );
                        def1x!("FileXz: return B {:?}", err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("(missing XZ stream header magic bytes)"));
                    }
                }
                // magic bytes expected "ý7zXZ\0"
                const XZ_MAGIC_BYTES: [u8; 6] = [
                    0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00,
                ];
                def1o!("FileXz: stream header magic bytes {:?}", buffer_);
                #[cfg(debug_assertions)]
                {
                    for (i, b_) in buffer_.iter().enumerate() {
                        let _b_ex = XZ_MAGIC_BYTES[i];
                        let _c_ex: char = _b_ex as char;
                        let _c: char = (*b_) as char;
                        def1o!("actual {0:3} (0x{0:02X}) {1:?}", b_, _c);
                        def1o!("expect {0:3} (0x{0:02X}) {1:?}\n", _b_ex, _c_ex);
                    }
                }
                if buffer_ != XZ_MAGIC_BYTES {
                    def1x!("Return ErrorKind::InvalidData");
                    return Result::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "failed to find XZ stream header magic bytes for {:?}",
                                path_std
                            ),
                        )
                    );
                }

                /*

                    2.1.1.2. Stream Flags

                    The first byte of Stream Flags is always a null byte. In the
                    future, this byte may be used to indicate a new Stream version
                    or other Stream properties.

                    The second byte of Stream Flags is a bit field
                 */

                // stream header flags 2 bytes
                let mut buffer_: [u8; 2] = [0; 2];
                def1o!("FileXz: reader.read_exact({})", buffer_.len());
                match reader.read_exact(&mut buffer_) {
                    Ok(_) => {}
                    Err(err) => {
                        def1x!("FileXz: return C {:?}", err);
                        e_err!(
                            "FileXz: reader.read_exact() (stream header flags) path {:?} {}",
                            path_std, err
                        );
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("(FileXz header flags)"));
                    }
                }
                def1o!("FileXz: buffer {:?}", buffer_);
                let _stream_header_flags: u16 = u16::from_le_bytes(buffer_);
                def1o!("FileXz: stream header flags 0b{0:016b}", _stream_header_flags);
                let stream_header_flag_null: u8 = buffer_[0];
                let stream_header_flag_check_type: u8 = buffer_[1];
                def1o!("FileXz: stream_header_flag_null 0x{0:02X} 0b{0:08b}", stream_header_flag_null);
                if stream_header_flag_null != 0 {
                    de_err!(
                        "FileXz: stream_header_flag_null 0x{0:02X} 0b{0:08b} is not null",
                        stream_header_flag_null,
                    );
                }
                def1o!("FileXz: stream_header_flag_check_type 0x{0:02X} 0b{0:08b}", stream_header_flag_check_type);

                /*
                    The second byte of Stream Flags is a bit field:

                    Bit(s)  Mask  Description
                    0-3    0x0F  Type of Check (see Section 3.4):
                                ID    Size      Check name
                                0x00   0 bytes  None
                                0x01   4 bytes  CRC32
                                0x02   4 bytes  (Reserved)
                                0x03   4 bytes  (Reserved)
                                0x04   8 bytes  CRC64
                                0x05   8 bytes  (Reserved)
                                0x06   8 bytes  (Reserved)
                                0x07  16 bytes  (Reserved)
                                0x08  16 bytes  (Reserved)
                                0x09  16 bytes  (Reserved)
                                0x0A  32 bytes  SHA-256
                                0x0B  32 bytes  (Reserved)
                                0x0C  32 bytes  (Reserved)
                                0x0D  64 bytes  (Reserved)
                                0x0E  64 bytes  (Reserved)
                                0x0F  64 bytes  (Reserved)
                    4-7    0xF0  Reserved for future use; MUST be zero for now.
                 */
                 if stream_header_flag_check_type & 0xF0 != 0 {
                    def1x!("FileXz: return Err(InvalidData)");
                    return Result::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "xz stream header flag check type 0x{:02X} has reserved bits set for {:?}",
                                stream_header_flag_check_type, path
                            ),
                        )
                    );
                }
                let _check_size: usize = match stream_header_flag_check_type {
                    0x00 => {
                        def1o!("FileXz: stream_header_flag_check_type 0x00 (None)");

                        0
                    }
                    0x01 => {
                        def1o!("FileXz: stream_header_flag_check_type 0x01 (CRC32)");

                        4
                    }
                    0x04 => {
                        def1o!("FileXz: stream_header_flag_check_type 0x04 (CRC64)");

                        8
                    }
                    0x0A => {
                        def1o!("FileXz: stream_header_flag_check_type 0x0A (SHA-256)");

                        32
                    }
                    _ => {
                        def1o!("FileXz: stream_header_flag_check_type 0x{:02X} (unknown)", stream_header_flag_check_type);

                        0
                    }
                };

                let mut buffer_: [u8; 4] = [0; 4];
                def1o!("FileXz: reader.read_exact({})", buffer_.len());
                match reader.read_exact(&mut buffer_) {
                    Ok(_) => {}
                    Err(err) => {
                        def1x!("FileXz: return G {:?}", err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("(XZ CRC32 Check)"));
                    }
                }
                let _stream_header_crc32: u32 = u32::from_le_bytes(buffer_);
                def1o!(
                    "FileXz: stream header CRC32 {0:} (0x{0:08X}) (0b{0:032b})",
                    _stream_header_crc32,
                );

                /*
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
                 */

                // block #0 block header size 1 byte
                // block #0 block header flags 1 byte
                let mut buffer_: [u8; 2] = [0; 2];
                def1o!("FileXz: reader.read_exact({})", buffer_.len());
                match reader.read_exact(&mut buffer_) {
                    Ok(_) => {}
                    Err(err) => {
                        def1x!("FileXz: return E {:?}", err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("(XZ Block #0 Header Size)"));
                    }
                }
                def1o!("FileXz: buffer {:?}", buffer_);
                let block0_encoded_header_size: u8 = buffer_[0];
                def1o!("FileXz: block #0 block header encoded_header_size {0:} (0x{0:02X})", block0_encoded_header_size);
                let block0_real_header_size: usize = (block0_encoded_header_size as usize + 1) * 4;
                def1o!("FileXz: block #0 block header real_header_size {0:} (0x{0:04X})", block0_real_header_size);
                const REAL_HEADER_SIZE_MAX: usize = 1024;
                if block0_real_header_size > REAL_HEADER_SIZE_MAX {
                    def1o!(
                        "block0_real_header_size {} > REAL_HEADER_SIZE_MAX {}",
                        block0_real_header_size, REAL_HEADER_SIZE_MAX
                    );
                    de_wrn!(
                        "block0_real_header_size {} > REAL_HEADER_SIZE_MAX {}",
                        block0_real_header_size, REAL_HEADER_SIZE_MAX
                    );
                }

                /*
                    The Block Flags field is a bit field:

                        Bit(s)  Mask  Description
                         0-1    0x03  Number of filters (1-4)
                         2-5    0x3C  Reserved for future use; MUST be zero for now.
                          6     0x40  The Compressed Size field is present.
                          7     0x80  The Uncompressed Size field is present.
                 */

                let block0_flags: u8 = buffer_[1];
                def1o!("FileXz: block #0 block header flags {0:} (0x{0:02X}) (0b{0:08b})", block0_flags);

                let _block0_flag_number_filters = (block0_flags & 0b00000011) as usize;
                let _block0_flag_reserved = (block0_flags & 0b00111100) >> 2;
                let block0_flag_compressed_size = (block0_flags & 0b01000000) != 0;
                let block0_flag_uncompressed_size = (block0_flags & 0b10000000) != 0;

                def1o!("FileXz: _block0_flag_number_filters {}", _block0_flag_number_filters);
                def1o!("FileXz: _block0_flag_reserved {}", _block0_flag_reserved);
                def1o!("FileXz: block0_flag_compressed_size {}", block0_flag_compressed_size);
                def1o!("FileXz: block0_flag_uncompressed_size {}", block0_flag_uncompressed_size);

                if !block0_flag_uncompressed_size {
                    let _msg = format!(
                        "xz file does not have Uncompressed Size field for {:?}",
                        path
                    );
                    def1o!("xz file does not have Uncompressed Size field for {:?}", path);
                    de_wrn!("xz file does not have Uncompressed Size field for {:?}", path);
                }

                #[allow(non_camel_case_types)]
                type uint64_t = u64;

                fn read_multibyte_integer(reader: &mut Take<&File>) -> Option<uint64_t> {
                    /*
                    Multibyte integers of static length, such as CRC values,
                    are stored in little endian byte order (least significant
                    byte first).

                    When smaller values are more likely than bigger values (for
                    example file sizes), multibyte integers are encoded in a
                    variable-length representation:
                    - Numbers in the range [0, 127] are copied as is, and take
                        one byte of space.
                    - Bigger numbers will occupy two or more bytes. All but the
                        last byte of the multibyte representation have the highest
                        (eighth) bit set.

                    For now, the value of the variable-length integers is limited
                    to 63 bits, which limits the encoded size of the integer to
                    nine bytes. These limits may be increased in the future if
                    needed.
                    */
                    def2n!();

                    const SIZE_MAX: uint64_t = 9;
                    let mut buffer_: [u8; 1] = [0b10000000; 1];
                    let mut num: uint64_t = 0;
                    let mut i: uint64_t = 0;
                    while buffer_[0] & 0b10000000 != 0 {
                        // read one byte
                        def1o!("FileXz: reader.read_exact({}) {}", buffer_.len(), i);
                        match reader.read_exact(&mut buffer_) {
                            Ok(_) => {}
                            Err(_err) => {
                                def2x!("return None {:?}", _err);
                                return None;
                            }
                        }
                        let b = buffer_[0];
                        def2o!("buffer {0:?} (0x{0:02X}) (0b{0:08b})", b);

                        if i >= SIZE_MAX || b == 0 || b & 0b10000000 != 0 {
                            break;
                        }

                        num |= ((b & 0b01111111) << (i * 7)) as uint64_t;
                        def2o!("num {0} (0x{0:08X})", num);
                        i += 1;
                    }

                    def2x!("read_multibyte_integer {0:?} (0x{0:08X}) among {1} bytes", num, i);
                    return Some(num);
                }

                if block0_flag_compressed_size {
                    let _block0_compressed_size: uint64_t = match read_multibyte_integer(&mut reader) {
                        Some(val) => val,
                        None => {
                            de_wrn!("FileXz: read_multibyte_integer returned None for Compressed Size");

                            0
                        }
                    };
                    def1o!("FileXz: _block0_compressed_size {}", _block0_compressed_size);
                }

                let _block0_uncompressed_size: uint64_t = match read_multibyte_integer(&mut reader) {
                    Some(val) => val,
                    None => {
                        de_wrn!("FileXz: read_multibyte_integer returned None for Uncompressed Size");

                        0
                    }
                };
                def1o!("FileXz: _block0_uncompressed_size {}", _block0_uncompressed_size);

                // reset Seek pointer
                def1o!("FileXz: file_xz.seek(SeekFrom::Start(0))");
                match file_xz.seek(SeekFrom::Start(0)) {
                    Ok(_) => {}
                    Err(err) => {
                        def1x!("FileXz: return G {:?}", err);
                        return err_from_err_path_result::<BlockReader>(&err, &path, Some("(XZ Block #0 Header Flags; reset Seek)"));
                    }
                }

                // TODO: Issue #12
                // HACK: Read the entire xz file into memory.
                //       Extracting the size from the header is difficult
                //       (I haven't implemented it). So the file size is only known from
                //       decompressing the entire file here (and counting the bytes returned).
                //       The `self.filesz_actual` must be set before exiting this function `new`,
                //       else various byte offset and block offset calcutions will fail, and the
                //       processing of this file will fail.
                //       The `lzma_rs` crate does not provide file size for xz files.
                //       Putting this hack here until the implementation of reading the
                //       header/blocks of the underlying .xz file
                // TODO: cannot read the file in chunks, `xz_decompress` does not accept
                //       `Options` with `memset`.
                //       See <https://github.com/gendx/lzma-rs/issues/110>
                let mut bufreader: BufReaderXz = BufReaderXz::new(file_xz);
                let mut buffer = Vec::<u8>::with_capacity(blocksz as usize);
                let mut _count_xz_decompress: usize = 0;
                #[allow(clippy::never_loop)]
                loop {
                    def1o!(
                        "FileXz: xz_decompress({:?}, buffer (len {}, capacity {}))",
                        bufreader,
                        buffer.len(),
                        buffer.capacity()
                    );
                    _count_xz_decompress += 1;
                    // XXX: xz_decompress may resize the passed `buffer`
                    match lzma_rs::xz_decompress(&mut bufreader, &mut buffer) {
                        Ok(_) => {
                            def1o!(
                                "FileXz: xz_decompress returned buffer len {}, capacity {}",
                                buffer.len(),
                                buffer.capacity(),
                            );
                        }
                        Err(err) => {
                            match &err {
                                lzma_rs::error::Error::IoError(ref ioerr) => {
                                    def1o!("FileXz: ioerr.kind() {:?}", ioerr.kind());
                                    if ioerr.kind() == ErrorKind::UnexpectedEof {
                                        def1o!("FileXz: xz_decompress Error UnexpectedEof, break!");
                                        break;
                                    }
                                    return err_from_err_path_result::<BlockReader>(ioerr, &path, Some("(xz_decompress failed)"));
                                }
                                _err => {
                                    def1o!("FileXz: err {:?}", _err);
                                }
                            }
                            def1x!("FileXz: xz_decompress Error, return Err({:?})", err);
                            return Err(
                                Error::new(
                                    ErrorKind::Other, format!("{} for file {:?}", err, path)
                                )
                            );
                        }
                    }
                    if buffer.is_empty() {
                        def1o!("buffer.is_empty()");
                        break;
                    }
                    let blocksz_u: usize = blocksz as usize;
                    let mut blockoffset: BlockOffset = 0;
                    // the `block`
                    while blockoffset <= ((buffer.len() / blocksz_u) as BlockOffset) {
                        let mut block: Block = Block::with_capacity(blocksz_u);
                        let a: usize = (blockoffset * blocksz) as usize;
                        let b: usize = a + (std::cmp::min(blocksz_u, buffer.len() - a));
                        def1o!("FileXz: block.extend_from_slice(&buffer[{}‥{}])", a, b);
                        block.extend_from_slice(&buffer[a..b]);
                        let blockp: BlockP = BlockP::new(block);
                        if let Some(bp_) = blocks.insert(blockoffset, blockp.clone()) {
                            debug_panic!("blockreader.blocks.insert({}, BlockP@{:p}) already had a entry BlockP@{:p}, path {:?}", blockoffset, blockp, bp_, path_std);
                        }
                        read_blocks_put += 1;
                        count_bytes_read += (*blockp).len() as Count;
                        def1o!("FileXz: count_bytes_read={}", count_bytes_read);
                        blocks_read.insert(blockoffset);
                        blockoffset += 1;
                    }
                }
                def1o!("FileXz: count_bytes_read {}", count_bytes_read);
                def1o!("FileXz: _count_xz_decompress {}", _count_xz_decompress);

                let filesz_uncompressed: FileSz = count_bytes_read as FileSz;
                filesz_actual = filesz_uncompressed;
                xz_opt = Some(XzData {
                    filesz: filesz_uncompressed,
                    bufreader,
                });
                def1o!("FileXz: created {:?}", xz_opt.as_ref().unwrap());
            }
            FileType::Unparsable
            => {
                debug_panic!("BlockReader::new bad filetype {:?} for file {:?}", filetype, path);

                return Result::Err(
                    Error::new(
                        ErrorKind::InvalidData,
                        format!("BlockReader::new bad filetype {:?} for file {:?}", filetype, path),
                    )
                );
            }
        }

        // XXX: don't assert on `filesz` vs `filesz_actual` to sanity check them.
        //      for some `.gz` files they can be either gt, lt, or eq.

        let blockn: Count = BlockReader::count_blocks(filesz_actual, blocksz);
        let blocks_highest = blocks.len();

        def1x!("return Ok(BlockReader)");

        Ok(BlockReader {
            path,
            _subpath: subpath_opt,
            path_subpath,
            file_handle,
            file_metadata,
            file_metadata_modified,
            filetype,
            drop_data: true,
            bz2: bz2_opt,
            gz: gz_opt,
            lz: lz_opt,
            xz: xz_opt,
            tar: tar_opt,
            filesz,
            filesz_actual,
            blockn,
            blocksz,
            read_block_last: 0,
            count_bytes_read,
            blocks,
            blocks_read,
            read_block_lru_cache: BlocksLRUCache::new(
                std::num::NonZeroUsize::new(BlockReader::READ_BLOCK_LRU_CACHE_SZ).unwrap(),
            ),
            read_block_lru_cache_enabled: BlockReader::CACHE_ENABLE_DEFAULT,
            read_block_cache_lru_hit: 0,
            read_block_cache_lru_miss: 0,
            read_block_cache_lru_put: 0,
            read_blocks_hit: 0,
            read_blocks_miss: 0,
            read_blocks_put,
            read_blocks_reread_error: 0,
            blocks_highest,
            dropped_blocks_ok: 0,
            dropped_blocks_err: 0,
            #[cfg(test)]
            dropped_blocks: HashSet::<BlockOffset>::with_capacity(blockn as usize),
        })
    }

    /// Return a reference to `self.path`.
    #[inline(always)]
    pub const fn path(&self) -> &FPath {
        &self.path_subpath
    }

    /// Return the file size in bytes.
    ///
    /// For compressed or archived files, returns the original file size
    /// (uncompressed or unarchived) as found in the header.
    ///
    /// An exception is `.xz` files, which are entirely read during function
    /// `new`, and the file size determined from the uncompressed bytes.
    /// See Issue #12.
    ///
    /// For plain files, returns the file size reported by the filesystem.
    ///
    /// Users of this struct should always use this instead of accessing
    /// `self.filesz` or `self.filesz_actual` directly.
    pub const fn filesz(&self) -> FileSz {
        match self.filetype {
            FileType::Evtx{ archival_type: FileTypeArchive::Normal } => self.filesz,
            FileType::Evtx{ archival_type: FileTypeArchive::Bz2 } => self.filesz_actual,
            FileType::Evtx{ archival_type: FileTypeArchive::Gz } => self.filesz_actual,
            FileType::Evtx{ archival_type: FileTypeArchive::Lz4 } => self.filesz_actual,
            FileType::Evtx{ archival_type: FileTypeArchive::Tar } => self.filesz_actual,
            FileType::Evtx{ archival_type: FileTypeArchive::Xz } => self.filesz_actual,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, .. } => self.filesz,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Bz2, .. } => self.filesz_actual,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, .. } => self.filesz_actual,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, .. } => self.filesz_actual,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, .. } => self.filesz_actual,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, .. } => self.filesz_actual,
            FileType::Journal{ archival_type: FileTypeArchive::Normal } => self.filesz,
            FileType::Journal{ archival_type: FileTypeArchive::Bz2 } => self.filesz,
            FileType::Journal{ archival_type: FileTypeArchive::Gz } => self.filesz_actual,
            FileType::Journal{ archival_type: FileTypeArchive::Lz4 } => self.filesz_actual,
            FileType::Journal{ archival_type: FileTypeArchive::Tar } => self.filesz_actual,
            FileType::Journal{ archival_type: FileTypeArchive::Xz } => self.filesz_actual,
            FileType::Text{ archival_type: FileTypeArchive::Normal, .. } => self.filesz,
            FileType::Text{ archival_type: FileTypeArchive::Bz2, .. } => self.filesz_actual,
            FileType::Text{ archival_type: FileTypeArchive::Gz, .. } => self.filesz_actual,
            FileType::Text{ archival_type: FileTypeArchive::Lz4, .. } => self.filesz_actual,
            FileType::Text{ archival_type: FileTypeArchive::Tar, .. } => self.filesz_actual,
            FileType::Text{ archival_type: FileTypeArchive::Xz, .. } => self.filesz_actual,
            FileType::Unparsable => panic!("BlockerReader::filesz Unexpected Unparsable"),
        }
    }

    /// Return a copy of `self.blocksz`
    #[inline(always)]
    pub const fn blocksz(&self) -> BlockSz {
        self.blocksz
    }

    /// Return a copy of `self.filetype`. The `FileType` enum determines
    /// various behaviors around opening and reading from the file.
    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        self.filetype
    }

    /// Return a copy of `self.file_metadata`.
    pub fn metadata(&self) -> Metadata {
        self.file_metadata.clone()
    }

    /// Get the best available "Modified Datetime" file attribute available.
    ///
    /// For compressed or archived files, use the copied header-embedded
    /// "MTIME".
    /// If the embedded "MTIME" is not available then return the encompassing
    /// file's "MTIME".
    ///
    /// For example, if archived file `syslog` within archive `logs.tar` does
    /// not have a valid "MTIME", then return the file system attribute
    /// "modified time" for `logs.tar`.
    //
    // TODO: also handle when `self.file_metadata_modified` is zero
    //       (or a non-meaningful placeholder value).
    pub fn mtime(&self) -> SystemTime {
        match self.filetype {
            FileType::Evtx{ .. } => {
                panic!("BlockerReader::mtime FileType::Evtx does not use a BlockReader")
            },
            FileType::Journal { .. } => {
                panic!("BlockerReader::mtime FileType::Journal does not use a BlockReader")
            }
            FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, .. }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Bz2, .. }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, .. }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, .. }
            | FileType::Text{ archival_type: FileTypeArchive::Normal, .. }
            | FileType::Text{ archival_type: FileTypeArchive::Bz2, .. }
            | FileType::Text{ archival_type: FileTypeArchive::Lz4, .. }
            | FileType::Text{ archival_type: FileTypeArchive::Xz, .. }
            => {
                defñ!(
                    "{:?}: file_metadata_modified {:?}",
                    self.filetype, self.file_metadata_modified
                );

                self.file_metadata_modified
            }
            FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, .. }
            | FileType::Text{ archival_type: FileTypeArchive::Gz, .. }
            => {
                let mtime = self
                    .gz
                    .as_ref()
                    .unwrap()
                    .mtime;
                if mtime != 0 {
                    let seconds = mtime as u64;
                    let st = seconds_to_systemtime(&seconds);
                    defñ!("{:?}: mtime {} -> {:?}", self.filetype, mtime, st);

                    st
                } else {
                    defñ!(
                        "{:?}: file_metadata_modified {:?}",
                        self.filetype, self.file_metadata_modified
                    );

                    self.file_metadata_modified
                }
            }
            FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, .. }
            | FileType::Text{ archival_type: FileTypeArchive::Tar, .. }
            => {
                let mtime = self
                    .tar
                    .as_ref()
                    .unwrap()
                    .mtime;
                if mtime != 0 {
                    let seconds = mtime as u64;
                    let st = seconds_to_systemtime(&seconds);
                    defñ!("{:?}: mtime {} -> {:?}", self.filetype, mtime, st);

                    st
                } else {
                    defñ!(
                        "{:?}: file_metadata_modified {:?}",
                        self.filetype, self.file_metadata_modified
                    );

                    self.file_metadata_modified
                }
            }
            FileType::Unparsable => panic!("BlockerReader::mtime unexpected {:?}", FileType::Unparsable),
        }
    }

    // TODO: [2022/03] make a `self` version of the following fn helpers that do not require
    //       passing `BlockSz`. Save the caller some trouble.
    //       Can also `assert` that passed `FileOffset` is not larger than filesz,
    //       greater than zero.
    //       But keep these public static version available for testing.
    //       Change the LineReader calls to call
    //       `self.blockreader.block_offset_at_file_offset_sansbsz()` or whatever it's named.

    // TODO: [2023/02] rename function sub-names `file_offset` to `fileoffset`,
    //       `block_offset` to `blockoffset`, `block_index` to `blockindex`

    /// Return nearest preceding `BlockOffset` for given `FileOffset`.
    #[inline(always)]
    pub const fn block_offset_at_file_offset(
        fileoffset: FileOffset,
        blocksz: BlockSz,
    ) -> BlockOffset {
        (fileoffset / blocksz) as BlockOffset
    }

    /// Return nearest preceding `BlockOffset` for given `FileOffset`.
    #[inline(always)]
    const fn block_offset_at_file_offset_self(
        &self,
        fileoffset: FileOffset,
    ) -> BlockOffset {
        BlockReader::block_offset_at_file_offset(fileoffset, self.blocksz)
    }

    /// See [BlockReader::file_offset_at_block_offset].
    ///
    /// [BlockReader::file_offset_at_block_offset]: crate::readers::blockreader::BlockReader#method.file_offset_at_block_offset
    #[inline(always)]
    pub const fn file_offset_at_block_offset(
        blockoffset: BlockOffset,
        blocksz: BlockSz,
    ) -> FileOffset {
        (blockoffset * blocksz) as BlockOffset
    }

    /// Return `FileOffset` (byte offset) at given `BlockOffset`.
    #[inline(always)]
    pub const fn file_offset_at_block_offset_self(
        &self,
        blockoffset: BlockOffset,
    ) -> FileOffset {
        (blockoffset * self.blocksz) as BlockOffset
    }

    /// Return `FileOffset` (byte offset) at `BlockOffset` + `BlockIndex`.
    #[inline(always)]
    pub const fn file_offset_at_block_offset_index(
        blockoffset: BlockOffset,
        blocksz: BlockSz,
        blockindex: BlockIndex,
    ) -> FileOffset {
        BlockReader::file_offset_at_block_offset(blockoffset, blocksz) + (blockindex as FileOffset)
    }

    /// Get the last byte `FileOffset` (index) of the file (inclusive).
    pub const fn fileoffset_last(&self) -> FileOffset {
        (self.filesz() - 1) as FileOffset
    }

    /// Return `BlockIndex` (byte offset into a `Block`) for the `Block` that
    /// corresponds to the passed `FileOffset`.
    #[inline(always)]
    pub const fn block_index_at_file_offset(
        fileoffset: FileOffset,
        blocksz: BlockSz,
    ) -> BlockIndex {
        (fileoffset
            - BlockReader::file_offset_at_block_offset(
                BlockReader::block_offset_at_file_offset(fileoffset, blocksz),
                blocksz,
            )) as BlockIndex
    }

    /// Return `BlockIndex` (byte offset into a `Block`) for the `Block` that
    /// corresponds to the passed `FileOffset`.
    #[inline(always)]
    pub const fn block_index_at_file_offset_self(
        &self,
        fileoffset: FileOffset,
    ) -> BlockIndex {
        BlockReader::block_index_at_file_offset(fileoffset, self.blocksz)
    }

    /// Return `Count` of [`Block`s] in a file.
    ///
    /// Equivalent to the _last [`BlockOffset`] + 1_.
    ///
    /// Not a count of `Block`s that have been read; the calculated
    /// count of `Block`s based on the `FileSz`.
    ///
    /// [`Block`s]: crate::readers::blockreader::Block
    /// [`BlockOffset`]: crate::readers::blockreader::BlockOffset
    #[inline(always)]
    pub const fn count_blocks(
        filesz: FileSz,
        blocksz: BlockSz,
    ) -> Count {
        filesz / blocksz + (if filesz % blocksz > 0 { 1 } else { 0 })
    }

    /// Specific `BlockSz` (size in bytes) of block at `BlockOffset`.
    // TODO: assert this is true when storing a `Block`
    fn blocksz_at_blockoffset_impl(
        blockoffset: &BlockOffset,
        blockoffset_last: &BlockOffset,
        blocksz: &BlockSz,
        filesz: &FileSz,
    ) -> BlockSz {
        defñ!(
            "blockreader.blocksz_at_blockoffset_(blockoffset {}, blockoffset_last {}, blocksz {}, filesz {})",
            blockoffset,
            blockoffset_last,
            blocksz,
            filesz
        );
        assert_le!(
            blockoffset,
            blockoffset_last,
            "Passed blockoffset {} but blockoffset_last {}",
            blockoffset,
            blockoffset_last
        );
        if filesz == &0 {
            return 0;
        }
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

    /// Specific `BlockSz` (size in bytes) of block at `BlockOffset`.
    ///
    /// This should be `self.blocksz` for all `Block`s except the last, which
    /// should be `>0` and `<=self.blocksz`
    pub fn blocksz_at_blockoffset(
        &self,
        blockoffset: &BlockOffset,
    ) -> BlockSz {
        defñ!("({})", blockoffset);
        BlockReader::blocksz_at_blockoffset_impl(
            blockoffset,
            &self.blockoffset_last(),
            &self.blocksz,
            &self.filesz(),
        )
    }

    /// Return `block.len()` for *stored* `Block` at `BlockOffset`.
    #[doc(hidden)]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn blocklen_at_blockoffset(
        &self,
        blockoffset: &BlockOffset,
    ) -> usize {
        match self.blocks.get(blockoffset) {
            Some(blockp) => blockp.len(),
            None => {
                panic!("BlockerReader::blocklen_at_blockoffset bad blockoffset {}; path {:?}", blockoffset, self.path)
            }
        }
    }

    /// Return last valid `BlockIndex` for block at the `BlockOffset`.
    #[inline(always)]
    #[allow(dead_code)]
    pub fn last_blockindex_at_blockoffset(
        &self,
        blockoffset: &BlockOffset,
    ) -> BlockIndex {
        (self.blocklen_at_blockoffset(blockoffset) - 1) as BlockIndex
    }

    /// The last valid `BlockOffset` for the file (inclusive).
    ///
    /// In other words, expected largest `BlockOffset` value for the file.
    /// Independent of `Block`s that have been processed.
    #[inline(always)]
    pub const fn blockoffset_last(&self) -> BlockOffset {
        if self.filesz() == 0 {
            return 0;
        }
        (BlockReader::count_blocks(self.filesz(), self.blocksz) as BlockOffset) - 1
    }

    /// `Count` of blocks read by this `BlockReader`.
    ///
    /// Not always the same as blocks currently stored (those may be `drop`ped
    /// during [streaming stage]).
    ///
    /// [streaming stage]: crate::readers::syslogprocessor::ProcessingStage#variant.Stage3StreamSyslines
    #[inline(always)]
    pub fn count_blocks_processed(&self) -> Count {
        self.blocks_read.len() as Count
    }

    /// `Count` of bytes stored by this `BlockReader`.
    #[inline(always)]
    pub fn count_bytes(&self) -> Count {
        self.count_bytes_read
    }

    /// Enable internal LRU cache used by `read_block`.
    #[allow(non_snake_case)]
    pub fn LRU_cache_enable(&mut self) {
        if self.read_block_lru_cache_enabled {
            return;
        }
        self.read_block_lru_cache_enabled = true;
        self.read_block_lru_cache
            .clear();
        self.read_block_lru_cache
            .resize(std::num::NonZeroUsize::new(BlockReader::READ_BLOCK_LRU_CACHE_SZ).unwrap());
    }

    /// Disable internal LRU cache used by `read_block`.
    #[allow(non_snake_case)]
    pub fn LRU_cache_disable(&mut self) {
        self.read_block_lru_cache_enabled = false;
        self.read_block_lru_cache
            .clear();
    }

    /// Is the current file a "streaming" file? i.e. must the file be read in
    /// a linear manner starting from the first byte? Compressed files
    /// must be read in this manner.
    ///
    /// If `true` then it is presumed calls to `read_block_File` will
    /// always be in sequential (linear) manner; i.e. only increasing values of
    /// the passed `blockoffset`.
    ///
    /// If `false` then no presumptions are made about the value of
    /// `blockoffset` passed to `read_block_File`.
    pub const fn is_streamed_file(&self) -> bool {
        match self.filetype {
            FileType::Evtx{ archival_type: FileTypeArchive::Normal } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Bz2 } => true,
            FileType::Evtx{ archival_type: FileTypeArchive::Gz } => true,
            FileType::Evtx{ archival_type: FileTypeArchive::Lz4 } => true,
            FileType::Evtx{ archival_type: FileTypeArchive::Tar } => true,
            FileType::Evtx{ archival_type: FileTypeArchive::Xz } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, .. } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Bz2, .. } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, .. } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, .. } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, .. } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, .. } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Normal } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Bz2 } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Gz } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Lz4 } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Tar } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Xz } => true,
            FileType::Text{ archival_type: FileTypeArchive::Normal, .. } => false,
            FileType::Text{ archival_type: FileTypeArchive::Bz2, .. } => true,
            FileType::Text{ archival_type: FileTypeArchive::Gz, .. } => true,
            FileType::Text{ archival_type: FileTypeArchive::Lz4, .. } => true,
            FileType::Text{ archival_type: FileTypeArchive::Tar, .. } => true,
            FileType::Text{ archival_type: FileTypeArchive::Xz, .. } => true,
            FileType::Unparsable => false,
        }
    }

    /// Return `drop_data` value.
    pub const fn is_drop_data(&self) -> bool {
        self.drop_data
    }

    /// Disable `drop_data`.
    /// Calls to `drop_block` will immediately return.
    /// User classes [`LineReader`], [`SyslineReader`], and [`SyslogProcessor`]
    /// will also check this value.
    ///
    /// [`LineReader`]: crate::readers::linereader::LineReader
    /// [`SyslineReader`]: crate::readers::syslinereader::SyslineReader
    /// [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor
    pub fn disable_drop_data(&mut self) {
        if ! self.drop_data {
            panic!("BlockReader::disable_drop_data drop_data already disabled");
        }
        self.drop_data = false;
    }

    /// Proactively `drop` the [`Block`] at [`BlockOffset`].
    /// For "[streaming stage]".
    ///
    /// _The caller must know what they are doing!_
    ///
    /// For some special files, like compressed files that must be read
    /// from back to front (e.g. no year in the datetimestamp),
    /// `drop_data` may be `false`. Setting `drop_data` is up to the user.
    ///
    /// [`Block`]: crate::readers::blockreader::Block
    /// [`BlockOffset`]: crate::readers::blockreader::BlockOffset
    /// [streaming stage]: crate::readers::syslogprocessor::ProcessingStage#variant.Stage3StreamSyslines
    pub fn drop_block(
        &mut self,
        blockoffset: BlockOffset,
    ) -> bool {
        defn!("({:?})", blockoffset);
        if ! self.drop_data {
            defx!("drop_data is false");
            return false;
        }
        let mut ret = true;
        let mut blockp_opt: Option<BlockP> = None;
        match self
            .blocks
            .remove(&blockoffset)
        {
            Some(blockp) => {
                deo!("removed block {} from blocks", blockoffset);
                blockp_opt = Some(blockp)
            }
            None => {
                deo!("no block {} in blocks", blockoffset);
            }
        }
        if self.read_block_lru_cache_enabled {
            match self
                .read_block_lru_cache
                .pop(&blockoffset)
            {
                Some(blockp) => {
                    deo!("removed block {} from LRU cache", blockoffset);
                    match blockp_opt {
                        Some(ref blockp_) => {
                            debug_assert_eq!(&blockp, blockp_, "For blockoffset {}, blockp in blocks != block in LRU cache", blockoffset);
                        }
                        None => {
                            deo!("WARNING: block {} only in LRU cache, not in blocks", blockoffset);
                            blockp_opt = Some(blockp);
                        }
                    }
                }
                None => {
                    deo!("no block {} in LRU cache", blockoffset);
                }
            }
        }
        match blockp_opt {
            Some(blockp) => match Arc::try_unwrap(blockp) {
                Ok(_block) => {
                    deo!(
                        "dropped block {} @0x{:p}, len {}, total span [{}‥{})",
                        blockoffset, &_block, _block.len(),
                        blockoffset * self.blocksz,
                        blockoffset * self.blocksz + (_block.len() as BlockOffset),
                    );
                    self.dropped_blocks_ok += 1;
                    #[cfg(test)]
                    {
                        self.dropped_blocks
                            .insert(blockoffset);
                    }
                }
                Err(_blockp) => {
                    self.dropped_blocks_err += 1;
                    deo!(
                        "failed to drop block {} @0x{:p}, len {}, strong_count {}",
                        blockoffset,
                        _blockp,
                        (*_blockp).len(),
                        Arc::strong_count(&_blockp),
                    );
                    ret = false;
                }
            },
            None => {
                deo!("block {} not found in blocks or LRU cache", blockoffset);
            }
        }
        defx!("({:?}) return {}", blockoffset, ret);

        ret
    }

    /// Store clone of `BlockP` in internal LRU cache.
    #[allow(non_snake_case)]
    fn store_block_in_LRU_cache(
        &mut self,
        blockoffset: BlockOffset,
        blockp: &BlockP,
    ) {
        deo!("LRU cache put({})", blockoffset);
        if !self.read_block_lru_cache_enabled {
            return;
        }
        self.read_block_lru_cache
            .put(blockoffset, blockp.clone());
        self.read_block_cache_lru_put += 1;
    }

    /// Store clone of `BlockP` in `self.blocks` storage.
    fn store_block_in_storage(
        &mut self,
        blockoffset: BlockOffset,
        blockp: &BlockP,
    ) {
        defñ!(
            "blocks.insert({}, Block (len {}, capacity {})), fileoffset [{}‥{})",
            blockoffset,
            (*blockp).len(),
            (*blockp).capacity(),
            self.file_offset_at_block_offset_self(blockoffset),
            self.file_offset_at_block_offset_self(blockoffset) + (*blockp).len() as FileOffset,
        );
        #[allow(clippy::single_match)]
        match self
            .blocks
            .insert(blockoffset, blockp.clone())
        {
            Some(_bp) => {
                de_wrn!(
                    "blockreader.blocks.insert({}, BlockP@{:p}) already had a entry BlockP@{:p} for file {:?}",
                    blockoffset, blockp, _bp, self.path,
                );
            }
            _ => {}
        }
        self.read_blocks_put += 1;
        self.blocks_highest = std::cmp::max(self.blocks_highest, self.blocks.len());
        if let false = self
            .blocks_read
            .insert(blockoffset)
        {
            debug_panic!(
                "BlockReader::store_blocks_in_storage blockreader.blocks_read({}) already had a entry, for file {:?}",
                blockoffset, self.path
            );
        }
    }

    /// Read up to [`BlockSz`] bytes of data (one `Block`) from a regular
    /// filesystem file ([`FileType::File`]).
    ///
    /// Called from `read_block`.
    ///
    /// [`FileType::File`]: crate::common::FileType
    /// [`BlockSz`]: BlockSz
    #[allow(non_snake_case)]
    fn read_block_File(
        &mut self,
        blockoffset: BlockOffset,
    ) -> ResultS3ReadBlock {
        defn!("({})", blockoffset);
        debug_assert!(
            matches!(
                self.filetype,
                FileType::Text { archival_type: FileTypeArchive::Normal, .. }
                | FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, .. }
            ),
            "wrong FileType {:?} for calling read_block_File",
            self.filetype
        );

        let seek: u64 = self.blocksz * blockoffset;
        defo!("self.file.seek({})", seek);
        match self
            .file_handle
            .seek(SeekFrom::Start(seek))
        {
            Ok(_) => {}
            Err(err) => {
                e_err!("file.SeekFrom(Start({})) {:?} {}", seek, self.path, err);
                defx!("({}): return Err({})", blockoffset, err);
                return ResultS3ReadBlock::Err(err);
            }
        };
        // here is where the `Block` is created then set with data.
        // It should never change after this. Is there a way to mark it as "frozen"?
        let cap: usize = self.blocksz_at_blockoffset(&blockoffset) as usize;
        let mut buffer = Block::with_capacity(cap);
        // forcibly resize `buffer` so `read_exact` will write to it
        buffer.resize(cap, 0);
        defo!("reader.read_exact(buffer (capacity {}))", cap);
        match self.file_handle.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(err) => {
                e_err!("reader.read_to_end(buffer) {} for file {:?}", err, self.path);
                defx!("read_block_File({}): return Err({})", blockoffset, err);
                return ResultS3ReadBlock::Err(err);
            }
        };
        self.count_bytes_read += buffer.len() as Count;
        defo!("count_bytes_read={}", self.count_bytes_read);
        if buffer.len() == 0 {
            defx!("read_block_File({}): return Done for file {:?}", blockoffset, self.path);
            return ResultS3ReadBlock::Done;
        }
        let blockp: BlockP = BlockP::new(buffer);
        self.store_block_in_storage(blockoffset, &blockp);
        self.store_block_in_LRU_cache(blockoffset, &blockp);
        defx!("({}): return Found", blockoffset);

        ResultS3ReadBlock::Found(blockp)
    }

    /// Read a block of data from storage for a compressed gzip file
    /// ([`FileTypeArchive::Bz2`]).
    /// `blockoffset` refers to the uncompressed version of the file.
    ///
    /// Called from `read_block`.
    #[allow(non_snake_case)]
    fn read_block_FileBz2(
        &mut self,
        blockoffset: BlockOffset,
    ) -> ResultS3ReadBlock {
        defn!("({})", blockoffset);
        debug_assert!(
            matches!(
                self.filetype,
                FileType::Evtx{ archival_type: FileTypeArchive::Bz2 }
                | FileType::FixedStruct{ archival_type: FileTypeArchive::Bz2, .. }
                | FileType::Journal{ archival_type: FileTypeArchive::Bz2 }
                | FileType::Text{ archival_type: FileTypeArchive::Bz2, .. }
            ),
            "wrong FileType {:?} for calling read_block_FileBz2",
            self.filetype
        );

        // handle special case immediately
        if self.filesz_actual == 0 {
            defx!("filesz 0; return Done");
            return ResultS3ReadBlock::Done;
        }

        let blockoffset_last: BlockOffset = self.blockoffset_last();
        let mut bo_at: BlockOffset = match self.blocks_read.iter().max() {
            Some(bo_) => *bo_,
            None => 0,
        };
        let mut bo_at_old: BlockOffset = bo_at;

        while bo_at <= blockoffset {
            // check `self.blocks_read` (not `self.blocks`) if the Block at `blockoffset`
            // was *ever* read.
            // TODO: [2024/06/02] add another stat tracker for lookups in `self.blocks_read`
            if self
                .blocks_read
                .contains(&bo_at)
            {
                self.read_blocks_hit += 1;
                if bo_at == blockoffset {
                    defx!("({}): return Found", blockoffset);
                    // XXX: this will panic if the key+value in `self.blocks` was dropped
                    //      which could happen if streaming stage occurs too soon
                    let blockp: BlockP = self
                        .blocks
                        .get_mut(&bo_at)
                        .unwrap()
                        .clone();
                    self.store_block_in_LRU_cache(bo_at, &blockp);
                    return ResultS3ReadBlock::Found(blockp);
                }
                bo_at += 1;
                continue;
            } else {
                defo!(
                    "({}): blocks_read.contains({}) missed (does not contain key)",
                    blockoffset, bo_at,
                );
                debug_assert!(
                    !self
                        .blocks
                        .contains_key(&bo_at),
                    "blocks has element {} not in blocks_read",
                    bo_at
                );
                self.read_blocks_miss += 1;
            }

            let blocksz_u: usize = self.blocksz_at_blockoffset(&bo_at) as usize;
            let reader = match self.bz2 {
                Some(ref mut bz2) => &mut bz2.decoder,
                None => {
                    return ResultS3ReadBlock::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "Bz2DecoderReader not initialized for file {:?}",
                                self.path
                            )
                        )
                    );
                }
            };
            let mut block = Block::with_capacity(blocksz_u);
            // `resize` will force `block` to have a length of `blocksz_u`
            // otherwise `as_mut_slice` is a zero-length slice
            block.resize(blocksz_u, 0);
            let mut bytes_read: usize = 0;
            // Rarely `read` will return less than the length of passed slice.
            // So call `read` until `block` is filled.
            while bytes_read < blocksz_u {
                defo!("Bz2DecoderReader.read((capacity {}, len {}))", block.capacity(), block.len());
                match reader.read(&mut block[bytes_read..]) {
                    Ok(size) => {
                        defo!(
                            "({}): Bz2DecoderReader.read((capacity {}, len {})) returned Ok({:?}), blocksz {}",
                            blockoffset,
                            block.capacity(),
                            block.len(),
                            size,
                            blocksz_u,
                        );
                        self.count_bytes_read += size as Count;
                        bytes_read += size;
                        defo!(
                            "({}): count_bytes_read={}, bytes_read={}",
                            blockoffset, self.count_bytes_read, bytes_read,
                        );
                        if size == 0 {
                            // the `Bz2Decoder` stalled, bail out
                            let byte_at = self.file_offset_at_block_offset_self(bo_at);
                            return ResultS3ReadBlock::Err(
                                Error::new(
                                    ErrorKind::UnexpectedEof,
                                    format!(
                                        "Bz2DecoderReader.read() read zero bytes from block {} (at byte {}), requested {} bytes. filesz {}, filesz uncompressed {} (according to bz2 header), last block {}; file {:?}",
                                        bo_at, byte_at, blocksz_u, self.filesz, self.filesz_actual, blockoffset_last, self.path,
                                    )
                                )
                            );
                        }
                    }
                    Err(err) => {
                        de_err!(
                            "Bz2DecoderReader.read(&block (capacity {})) error {} for file {:?}",
                            self.blocksz,
                            err,
                            self.path,
                        );
                        defx!("({}): return Err({})", blockoffset, err);
                        return err_from_err_path_results3(
                            &err,
                            self.path(),
                            Some("(Bz2DecoderReader.read(…) failed)"),
                        );
                    }
                }
            }

            // sanity check: check returned Block is expected number of bytes
            let blocklen_sz: BlockSz = block.len() as BlockSz;
            defo!(
                "({}): block.len() {}, blocksz {}, blockoffset at {}",
                blockoffset,
                blocklen_sz,
                self.blocksz,
                bo_at
            );
            if block.is_empty() {
                let byte_at = self.file_offset_at_block_offset_self(bo_at);
                return ResultS3ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                            "Bz2DecoderReader.read() read zero bytes from block {} (at byte {}), requested {} bytes. filesz {}, filesz uncompressed {} (according to bz2 header), last block {}; file {:?}",
                            bo_at, byte_at, blocksz_u, self.filesz, self.filesz_actual, blockoffset_last, self.path,
                        )
                    )
                );
            } else if bo_at == blockoffset_last {
                // last block, is blocksz correct?
                if blocklen_sz > self.blocksz {
                    let byte_at = self.file_offset_at_block_offset_self(bo_at);
                    return ResultS3ReadBlock::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "Bz2DecoderReader.read() read {} bytes for last block {} (at byte {}) which is larger than block size {} bytes; file {:?}",
                                blocklen_sz, bo_at, byte_at, self.blocksz, self.path,
                            )
                        )
                    );
                }
            } else if bytes_read != (self.blocksz as usize) {
                // not last block, is blocksz correct?
                let byte_at = self.file_offset_at_block_offset_self(bo_at) + (bytes_read as FileOffset);
                return ResultS3ReadBlock::Err(
                    Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "Bz2DecoderReader.read() read {} bytes for block {} expected to read {} bytes (block size), inflate stopped at byte {}. block last {}, filesz {}, filesz uncompressed {} (according to bz2 header); file {:?}",
                            blocklen_sz, bo_at, self.blocksz, byte_at, blockoffset_last, self.filesz, self.filesz_actual, self.path,
                        )
                    )
                );
            }

            // store decompressed block
            let blockp = BlockP::new(block);
            self.store_block_in_storage(bo_at, &blockp);
            self.store_block_in_LRU_cache(bo_at, &blockp);

            // This file must be streamed i.e. Blocks are read in sequential
            // order. So drop old blocks so RAM usage is not O(n).
            // Since the Blocks dropped here were first read then dropped
            // within this function then no other entity has stored
            // a cloned `BlockP` pointer 🤞; the Block is certain to be
            // destroyed here.
            // In other words, it is presumed the caller will never call
            // `read_block_FileBz2` with a `blockoffset` value less than the
            // value passed here (if that happened then runtime would be
            // O(n²).
            // See Issue #182
            if BlockReader::READ_BLOCK_LOOKBACK_DROP && bo_at_old < bo_at {
                self.drop_block(bo_at_old);
            }
            if bo_at == blockoffset {
                defx!("({}): return Found", blockoffset);
                return ResultS3ReadBlock::Found(blockp);
            }
            bo_at_old = bo_at;
            bo_at += 1;
        } // while bo_at <= blockoffset

        defx!("({}): return Done", blockoffset);

        ResultS3ReadBlock::Done
    }

    /// Read a block of data from storage for a compressed gzip file
    /// ([`FileTypeArchive::Gz`]).
    /// `blockoffset` refers to the uncompressed version of the file.
    ///
    /// Called from `read_block`.
    ///
    /// A `.gz` file must read as a stream, from beginning to end
    /// (cannot jump forward).
    /// So read the entire file up to passed `blockoffset`, storing each
    /// decompressed `Block`.
    #[allow(non_snake_case)]
    fn read_block_FileGz(
        &mut self,
        blockoffset: BlockOffset,
    ) -> ResultS3ReadBlock {
        defn!("({})", blockoffset);
        debug_assert!(
            matches!(
                self.filetype,
                FileType::Evtx{ archival_type: FileTypeArchive::Gz }
                | FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, .. }
                | FileType::Journal{ archival_type: FileTypeArchive::Gz }
                | FileType::Text{ archival_type: FileTypeArchive::Gz, .. }
            ),
            "wrong FileType {:?} for calling read_block_FileGz",
            self.filetype
        );

        // handle special case immediately
        if self.filesz_actual == 0 {
            defx!("filesz 0; return Done");
            return ResultS3ReadBlock::Done;
        }

        let blockoffset_last: BlockOffset = self.blockoffset_last();
        // get the maximum blockoffset that has been read
        // read the file up to the passed `blockoffset`
        let mut bo_at: BlockOffset = match self.blocks_read.iter().max() {
            Some(bo_) => *bo_,
            None => 0,
        };
        let mut bo_at_old: BlockOffset = bo_at;

        while bo_at <= blockoffset {
            // check `self.blocks_read` (not `self.blocks`) if the Block at `blockoffset`
            // was *ever* read.
            // TODO: [2022/06/18] add another stat tracker for lookups in `self.blocks_read`
            if self
                .blocks_read
                .contains(&bo_at)
            {
                self.read_blocks_hit += 1;
                if bo_at == blockoffset {
                    defx!("({}): return Found", blockoffset);
                    // XXX: this will panic if the key+value in `self.blocks` was dropped
                    //      which could happen if streaming stage occurs too soon
                    let blockp: BlockP = self
                        .blocks
                        .get_mut(&bo_at)
                        .unwrap()
                        .clone();
                    self.store_block_in_LRU_cache(bo_at, &blockp);
                    return ResultS3ReadBlock::Found(blockp);
                }
                bo_at += 1;
                continue;
            } else {
                defo!("({}): blocks_read.contains({}) missed (does not contain key)", blockoffset, bo_at);
                debug_assert!(
                    !self
                        .blocks
                        .contains_key(&bo_at),
                    "blocks has element {} not in blocks_read",
                    bo_at
                );
                self.read_blocks_miss += 1;
            }

            // decompress the gzip data. This is tedious because the `GzDecoder` can stall
            // and return zero bytes read when called for "large" blocks. Through trial + error,
            // it was found reading a block size of 2056 works well without being too small.
            // So the upcoming while loop reads chunks of <=2056 bytes until the entire block is read into
            // the `block`.

            // blocksz of this block
            let blocksz_u: usize = self.blocksz_at_blockoffset(&bo_at) as usize;
            // bytes to read in all `.read()` except the last
            // BUG: large block sizes are more likely to fail `.read`
            //      so do many smaller reads of a size that succeeds more often
            //      in ad-hoc experiments, this size was found to succeed most often
            // TODO: [2024/06] good experiment for coz profiler; adjusting `BUF_SZ`
            const BUF_SZ: usize = 2056;
            // intermediate buffer of size `BUF_SZ` for smaller reads
            let mut buf: [u8; BUF_SZ] = [0; BUF_SZ];
            // bytes to read in last `.read()` for this block
            let _readsz_last: usize = blocksz_u % BUF_SZ;
            // number of expected `.read()` of size `BUF_SZ` plus one for this block
            let _reads_expect: usize = match bo_at == blockoffset_last {
                true => blocksz_u % BUF_SZ,
                false => blocksz_u / BUF_SZ + 1,
            };
            // number of actual `.read()` for this block
            let mut reads_actual: usize = 0;
            // number of expected bytes to read for this block
            let bytes_read_expect: usize = blocksz_u;
            // number of bytes actually read for this block
            let mut bytes_read_actual: usize = 0;
            debug_assert_eq!(
                bytes_read_expect, blocksz_u,
                "bad calculation; bytes_read_expect {} != blocksz_u {}; blockoffset {}", bytes_read_expect, blocksz_u, bo_at,
            );
            // final block of storage
            let mut block = Block::with_capacity(blocksz_u);
            // XXX: `with_capacity, clear, resize` is a verbose way to create a new vector with a
            //      run-time determined `capacity` and `len`. `len == capacity` is necessary for
            //      calls to `decoder.read`.
            //      Using `decoder.read_exact` and `decoder.read_to_end` was more difficult.
            //      See https://github.com/rust-lang/flate2-rs/issues/308
            block.clear();
            block.resize(blocksz_u, 0);
            defo!(
                "({}): blocks_read count {:?}; for blockoffset {}: must do {} reads of {} bytes, and one read of {} bytes (total {} bytes to read) (uncompressed filesz {})",
                blockoffset, self.blocks_read.len(), bo_at, _reads_expect - 1, BUF_SZ, _readsz_last, bytes_read_expect, self.filesz(),
            );

            while bytes_read_actual < bytes_read_expect {
                let readsz: usize = match bytes_read_expect - bytes_read_actual < BUF_SZ {
                    true => bytes_read_expect - bytes_read_actual,
                    false => BUF_SZ,
                };
                debug_assert_ne!(
                    readsz, 0,
                    "readsz 0; BUF_SZ {}, bytes_read_actual {}, bytes_read_expect {}, diff {}",
                    BUF_SZ, bytes_read_actual, bytes_read_expect, bytes_read_expect - bytes_read_actual,
                );

                defo!(
                    "({}): GzDecoder.read(…); bytes_read_actual {}, readsz {}, BUF_SZ {}, buf len {}, blockoffset {}, blocksz {}",
                    blockoffset, bytes_read_actual, readsz, BUF_SZ, buf.len(), bo_at, blocksz_u,
                );
                match (self
                    .gz
                    .as_mut()
                    .unwrap()
                    .decoder)
                    .read(buf[..readsz].as_mut())
                {
                    Ok(size_) if size_ == 0 => {
                        reads_actual += 1;
                        defo!(
                            "({}): GzDecoder.read() {} returned Ok({:?}); bytes_read_actual {}",
                            blockoffset,
                            reads_actual,
                            size_,
                            bytes_read_actual
                        );
                        let byte_at: FileOffset =
                            self.file_offset_at_block_offset_self(bo_at) + (bytes_read_actual as FileOffset);
                        // in ad-hoc testing, it was found the decoder never recovers from a
                        // zero-byte read
                        return ResultS3ReadBlock::Err(
                            Error::new(
                                ErrorKind::InvalidData,
                                format!(
                                    "GzDecoder.read() {} read zero bytes for vec<u8> buffer of length {}; stuck at inflated byte {}, size {}, size uncompressed {} (calculated from gzip header), BUF_SZ {}; file {:?}",
                                    reads_actual,
                                    buf.len(),
                                    byte_at,
                                    self.filesz,
                                    self.filesz_actual,
                                    BUF_SZ,
                                    self.path,
                                )
                            )
                        );
                    }
                    // read was too large
                    Ok(size_) if size_ > readsz => {
                        reads_actual += 1;
                        defo!("({}): GzDecoder.read() returned Ok({:?}); size too big", blockoffset, size_);
                        self.count_bytes_read += size_ as Count;
                        defo!("({}): count_bytes_read={}", blockoffset, self.count_bytes_read);
                        return ResultS3ReadBlock::Err(
                            Error::new(
                                ErrorKind::InvalidData,
                                format!(
                                    "GzDecoder.read() {} read too many bytes {} for vec<u8> buffer of length {}; file size {}, file size uncompressed {} (calculated from gzip header), BUF_SZ {}; file {:?}",
                                    reads_actual,
                                    size_,
                                    buf.len(),
                                    self.filesz,
                                    self.filesz_actual,
                                    BUF_SZ,
                                    self.path,
                                )
                            )
                        );
                    }
                    // first or subsequent read is le expected size
                    Ok(size_) => {
                        reads_actual += 1;
                        defo!(
                            "({}): GzDecoder.read() {} returned Ok({:?}), BUF_SZ {}, blocksz {}, bytes_read_actual {}",
                            blockoffset,
                            reads_actual,
                            size_,
                            readsz,
                            blocksz_u,
                            bytes_read_actual + size_,
                        );
                        self.count_bytes_read += size_ as Count;
                        block[bytes_read_actual..bytes_read_actual + size_].copy_from_slice(&buf[..size_]);
                        defo!(
                            "({}): copied {} bytes into block {}, from [{}‥{})",
                            blockoffset, size_, blockoffset, bytes_read_actual, bytes_read_actual + size_,
                        );
                        bytes_read_actual += size_;
                    }
                    Err(err) => {
                        de_err!(
                            "GzDecoder.read(&block (capacity {})) {} error {} for file {:?}",
                            self.blocksz,
                            reads_actual,
                            err,
                            self.path
                        );
                        defx!("({}): return Err({})", blockoffset, err);
                        return err_from_err_path_results3(
                            &err,
                            &self.path,
                            Some("(GzDecoder.read(…) failed)"),
                        );
                    }
                }
                debug_assert_le!(
                    block.len(),
                    blocksz_u,
                    "block.len() {} was expected to be <= blocksz {}",
                    block.len(),
                    blocksz_u
                );
                buf.fill(0);
            } // while reads > 0

            // sanity check: check returned Block is expected number of bytes
            let blocklen_sz: BlockSz = block.len() as BlockSz;
            defo!(
                "({}): block.len() {}, blocksz {}, blockoffset at {}, copied {} bytes",
                blockoffset,
                blocklen_sz,
                self.blocksz,
                bo_at,
                bytes_read_actual,
            );
            if block.is_empty() {
                let byte_at = self.file_offset_at_block_offset_self(bo_at);
                return ResultS3ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                            "GzDecoder.read() read zero bytes from block {} (at byte {}), requested {} bytes. filesz {}, filesz uncompressed {} (according to gzip header), last block {}, BUF_SZ {}; file {:?}",
                            bo_at,
                            byte_at,
                            blocksz_u,
                            self.filesz,
                            self.filesz_actual,
                            blockoffset_last,
                            BUF_SZ,
                            self.path,
                        )
                    )
                );
            } else if bo_at == blockoffset_last {
                // last block, is blocksz correct?
                if blocklen_sz > self.blocksz {
                    let byte_at = self.file_offset_at_block_offset_self(bo_at);
                    return ResultS3ReadBlock::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "GzDecoder.read() read {} bytes for last block {} (at byte {}) which is larger than block size {} bytes, BUF_SZ {}; file {:?}",
                                blocklen_sz, bo_at, byte_at, self.blocksz, BUF_SZ, self.path,
                            )
                        )
                    );
                }
            } else if blocklen_sz != self.blocksz {
                // not last block, is blocksz correct?
                let byte_at = self.file_offset_at_block_offset_self(bo_at) + blocklen_sz;
                return ResultS3ReadBlock::Err(
                    Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "GzDecoder.read() read {} bytes for block {} expected to read {} bytes (block size), inflate stopped at byte {}. block last {}, filesz {}, filesz uncompressed {} (according to gzip header), BUF_SZ {}; file {:?}",
                            blocklen_sz,
                            bo_at,
                            self.blocksz,
                            byte_at,
                            blockoffset_last,
                            self.filesz,
                            self.filesz_actual,
                            BUF_SZ,
                            self.path,
                        )
                    )
                );
            } else if bytes_read_expect != bytes_read_actual {
                return ResultS3ReadBlock::Err(
                    Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "GzDecoder.read() read {} bytes for block {} expected to read {} bytes (block size), inflate stopped at byte {}. block last {}, filesz {}, filesz uncompressed {} (according to gzip header), BUF_SZ {}; file {:?}",
                            bytes_read_actual,
                            bo_at,
                            bytes_read_expect,
                            self.file_offset_at_block_offset_self(bo_at) + (bytes_read_actual as FileOffset),
                            blockoffset_last,
                            self.filesz,
                            self.filesz_actual,
                            BUF_SZ,
                            self.path,
                        )
                    )
                );
            }

            // store decompressed block
            let blockp = BlockP::new(block);
            self.store_block_in_storage(bo_at, &blockp);
            self.store_block_in_LRU_cache(bo_at, &blockp);

            // This file must be streamed i.e. Blocks are read in sequential
            // order. So drop old blocks so RAM usage is not O(n).
            // Since the Blocks dropped here were first read then dropped
            // within this function then no other entity has stored
            // a cloned `BlockP` pointer 🤞; the Block is certain to be
            // destroyed here.
            // In other words, it is presumed the caller will never call
            // `read_block_FileGz` with a `blockoffset` value less than the
            // value passed here (if that happened then runtime would be
            // O(n²).
            // See Issue #182
            if BlockReader::READ_BLOCK_LOOKBACK_DROP && bo_at_old < bo_at {
                self.drop_block(bo_at_old);
            }
            if bo_at == blockoffset {
                defx!("({}): return Found", blockoffset);
                return ResultS3ReadBlock::Found(blockp);
            }
            bo_at_old = bo_at;
            bo_at += 1;
        } // while bo_at <= blockoffset

        defx!("({}): return Done", blockoffset);

        ResultS3ReadBlock::Done
    }

    /// Read a block of data from storage for a compressed lzma file
    /// ([`FileTypeArchive::Lz4`]).
    /// `blockoffset` refers to the uncompressed version of the file.
    ///
    /// Called from `read_block`.
    ///
    /// A `.lz4` file must read as a stream, from beginning to end
    /// (cannot jump forward).
    /// So read the entire file up to passed `blockoffset`, storing each
    /// decompressed `Block`.
    #[allow(non_snake_case)]
    fn read_block_FileLz4(
        &mut self,
        blockoffset: BlockOffset,
    ) -> ResultS3ReadBlock {
        defn!("({})", blockoffset);
        debug_assert!(
            matches!(
                self.filetype,
                FileType::Evtx{ archival_type: FileTypeArchive::Lz4 }
                | FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, .. }
                | FileType::Journal{ archival_type: FileTypeArchive::Lz4 }
                | FileType::Text{ archival_type: FileTypeArchive::Lz4, .. }
            ),
            "wrong FileType {:?} for calling read_block_FileLz4",
            self.filetype
        );

        // handle special case immediately
        if self.filesz_actual == 0 {
            defx!("filesz 0; return Done");
            return ResultS3ReadBlock::Done;
        }

        let blockoffset_last: BlockOffset = self.blockoffset_last();
        let mut bo_at: BlockOffset = match self.blocks_read.iter().max() {
            Some(bo_) => *bo_,
            None => 0,
        };
        let mut bo_at_old: BlockOffset = bo_at;

        while bo_at <= blockoffset {
            // check `self.blocks_read` (not `self.blocks`) if the Block at `blockoffset`
            // was *ever* read.
            // TODO: [2022/06/18] add another stat tracker for lookups in `self.blocks_read`
            if self
                .blocks_read
                .contains(&bo_at)
            {
                self.read_blocks_hit += 1;
                if bo_at == blockoffset {
                    defx!("({}): return Found", blockoffset);
                    // XXX: this will panic if the key+value in `self.blocks` was dropped
                    //      which could happen if streaming stage occurs too soon
                    let blockp: BlockP = self
                        .blocks
                        .get_mut(&bo_at)
                        .unwrap()
                        .clone();
                    self.store_block_in_LRU_cache(bo_at, &blockp);
                    return ResultS3ReadBlock::Found(blockp);
                }
                bo_at += 1;
                continue;
            } else {
                defo!("({}): blocks_read.contains({}) missed (does not contain key)", blockoffset, bo_at);
                debug_assert!(
                    !self
                        .blocks
                        .contains_key(&bo_at),
                    "blocks has element {} not in blocks_read",
                    bo_at
                );
                self.read_blocks_miss += 1;
            }

            let blocksz_u: usize = self.blocksz_at_blockoffset(&bo_at) as usize;
            let reader = match self.lz {
                Some(ref mut lz) => &mut lz.reader,
                None => {
                    return ResultS3ReadBlock::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "FrameDecoder not initialized for file {:?}",
                                self.path
                            )
                        )
                    );
                }
            };
            let mut block = Block::with_capacity(blocksz_u);
            defo!("FrameDecoder.read((capacity {}))", block.capacity());
            // `resize` will force `block` to have a length of `blocksz_u`
            // otherwise `as_mut_slice` is a zero-length slice
            block.resize(blocksz_u, 0);
            match reader.read(&mut block.as_mut_slice()) {
                Ok(size) => {
                    defo!(
                        "({}): FrameDecoder.read((capacity {})) returned Ok({:?}), blocksz {}",
                        blockoffset,
                        block.capacity(),
                        size,
                        blocksz_u,
                    );
                    self.count_bytes_read += size as Count;
                    defo!("({}): count_bytes_read={}", blockoffset, self.count_bytes_read);
                }
                Err(err) => {
                    de_err!(
                        "FrameDecoder.read(&block (capacity {})) error {} for file {:?}",
                        self.blocksz,
                        err,
                        self.path,
                    );
                    defx!("({}): return Err({})", blockoffset, err);
                    return err_from_err_path_results3(
                        &err,
                        self.path(),
                        Some("(FrameDecoder.read(…) failed)"),
                    );
                }
            }

            // sanity check: check returned Block is expected number of bytes
            let blocklen_sz: BlockSz = block.len() as BlockSz;
            defo!(
                "({}): block.len() {}, blocksz {}, blockoffset at {}",
                blockoffset,
                blocklen_sz,
                self.blocksz,
                bo_at
            );
            if block.is_empty() {
                let byte_at = self.file_offset_at_block_offset_self(bo_at);
                return ResultS3ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                            "FrameDecoder.read() read zero bytes from block {} (at byte {}), requested {} bytes. filesz {}, filesz uncompressed {} (according to gzip header), last block {}; file {:?}",
                            bo_at, byte_at, blocksz_u, self.filesz, self.filesz_actual, blockoffset_last, self.path,
                        )
                    )
                );
            } else if bo_at == blockoffset_last {
                // last block, is blocksz correct?
                if blocklen_sz > self.blocksz {
                    let byte_at = self.file_offset_at_block_offset_self(bo_at);
                    return ResultS3ReadBlock::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "FrameDecoder.read() read {} bytes for last block {} (at byte {}) which is larger than block size {} bytes; file {:?}",
                                blocklen_sz, bo_at, byte_at, self.blocksz, self.path,
                            )
                        )
                    );
                }
            } else if blocklen_sz != self.blocksz {
                // not last block, is blocksz correct?
                let byte_at = self.file_offset_at_block_offset_self(bo_at) + blocklen_sz;
                return ResultS3ReadBlock::Err(
                    Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "FrameDecoder.read() read {} bytes for block {} expected to read {} bytes (block size), inflate stopped at byte {}. block last {}, filesz {}, filesz uncompressed {} (according to gzip header); file {:?}",
                            blocklen_sz, bo_at, self.blocksz, byte_at, blockoffset_last, self.filesz, self.filesz_actual, self.path,
                        )
                    )
                );
            }

            // store decompressed block
            let blockp = BlockP::new(block);
            self.store_block_in_storage(bo_at, &blockp);
            self.store_block_in_LRU_cache(bo_at, &blockp);

            // This file must be streamed i.e. Blocks are read in sequential
            // order. So drop old blocks so RAM usage is not O(n).
            // Since the Blocks dropped here were first read then dropped
            // within this function then no other entity has stored
            // a cloned `BlockP` pointer 🤞; the Block is certain to be
            // destroyed here.
            // In other words, it is presumed the caller will never call
            // `read_block_FileLz4` with a `blockoffset` value less than the
            // value passed here (if that happened then runtime would be
            // O(n²).
            // See Issue #182
            if BlockReader::READ_BLOCK_LOOKBACK_DROP && bo_at_old < bo_at {
                self.drop_block(bo_at_old);
            }
            if bo_at == blockoffset {
                defx!("({}): return Found", blockoffset);
                return ResultS3ReadBlock::Found(blockp);
            }
            bo_at_old = bo_at;
            bo_at += 1;
        } // while bo_at <= blockoffset

        defx!("({}): return Done", blockoffset);

        ResultS3ReadBlock::Done
    }

    /// Read a block of data from storage for a compressed `xz` file
    /// ([`FileTypeArchive::Xz`]).
    /// `blockoffset` refers to the uncompressed version of the file.
    ///
    /// Called from `read_block`.
    ///
    /// An `.xz` file must read as a stream, from beginning to end
    /// (cannot jump forward).
    /// So read the entire file up to passed `blockoffset`, storing each
    /// decompressed `Block`.
    #[allow(non_snake_case)]
    fn read_block_FileXz(
        &mut self,
        blockoffset: BlockOffset,
    ) -> ResultS3ReadBlock {
        defn!("({})", blockoffset);
        debug_assert!(
            matches!(
                self.filetype,
                FileType::Evtx{ archival_type: FileTypeArchive::Xz }
                | FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, .. }
                | FileType::Journal{ archival_type: FileTypeArchive::Xz }
                | FileType::Text{ archival_type: FileTypeArchive::Xz, .. }
            ),
            "wrong FileType {:?} for calling read_block_FileXz",
            self.filetype
        );

        // handle special case immediately
        if self.filesz_actual == 0 {
            defx!("filesz 0; return Done");
            return ResultS3ReadBlock::Done;
        }

        let blockoffset_last: BlockOffset = self.blockoffset_last();
        let mut bo_at: BlockOffset = match self.blocks_read.iter().max() {
            Some(bo_) => *bo_,
            None => 0,
        };

        while bo_at <= blockoffset {
            // check `self.blocks_read` (not `self.blocks`) if the Block at `blockoffset`
            // was *ever* read.
            // TODO: [2022/06/18] add another stat tracker for lookups in `self.blocks_read`
            if self
                .blocks_read
                .contains(&bo_at)
            {
                self.read_blocks_hit += 1;
                if bo_at == blockoffset {
                    // XXX: this will panic if the key+value in `self.blocks` was dropped
                    //      which could happen during streaming stage
                    let blockp: BlockP = self
                        .blocks
                        .get_mut(&bo_at)
                        .unwrap()
                        .clone();
                    // drop "old" blocks
                    let mut bo_drop = bo_at;
                    while bo_drop > 0 && bo_drop <= blockoffset {
                        defo!("drop old block {}", bo_drop - 1);
                        self.drop_block(bo_drop - 1);
                        bo_drop += 1;
                    }
                    defx!("({}): return Found", blockoffset);
                    return ResultS3ReadBlock::Found(blockp);
                }
                bo_at += 1;
                continue;
            } else {
                defo!("blocks_read.contains({}) missed (does not contain key)", bo_at);
                debug_assert!(
                    !self
                        .blocks
                        .contains_key(&bo_at),
                    "blocks has element {} not in blocks_read",
                    bo_at
                );
                self.read_blocks_miss += 1;
            }

            let blocksz_u: usize = self.blocksz_at_blockoffset(&bo_at) as usize;
            let mut block = Block::with_capacity(blocksz_u);
            let mut bufreader: &mut BufReaderXz = &mut self
                .xz
                .as_mut()
                .unwrap()
                .bufreader;
            deo!(
                "xz_decompress({:?}, block (len {}, capacity {}))",
                bufreader,
                block.len(),
                block.capacity()
            );
            // XXX: xz_decompress may resize the passed `buffer`
            match lzma_rs::xz_decompress(&mut bufreader, &mut block) {
                Ok(_) => {
                    deo!("xz_decompress returned block len {}, capacity {}", block.len(), block.capacity());
                }
                Err(err) => {
                    // XXX: would typically `return Err(err)` but the `err` is of type
                    //      `lzma_rs::error::Error`
                    //      https://docs.rs/lzma-rs/0.2.0/lzma_rs/error/enum.Error.html
                    match &err {
                        lzma_rs::error::Error::IoError(ref ioerr) => {
                            defo!("ioerr.kind() {:?}", ioerr.kind());
                            if ioerr.kind() == ErrorKind::UnexpectedEof {
                                defo!("xz_decompress Error UnexpectedEof, break!");
                                break;
                            }
                        }
                        _err => {
                            defo!("err {:?}", _err);
                        }
                    }
                    defx!("xz_decompress Error, return Err({:?})", err);
                    return ResultS3::Err(Error::new(ErrorKind::Other, format!("{:?}", err)));
                }
            }
            deo!("xz_decompress returned block len {}, capacity {}", block.len(), block.capacity());
            self.count_bytes_read += block.len() as Count;
            deo!("xz_decompress count_bytes_read={}", self.count_bytes_read);
            // check returned Block is expected number of bytes
            let blocklen_sz: BlockSz = block.len() as BlockSz;
            if block.is_empty() {
                let byte_at = self.file_offset_at_block_offset_self(bo_at);
                defx!("({}): return Err", blockoffset);
                return ResultS3ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                            "xz_decompress read zero bytes from block {} (at byte {}), requested {} bytes. filesz {}, filesz uncompressed {} (according to gzip header), last block {}; file {:?}",
                            bo_at, byte_at, blocksz_u, self.filesz, self.filesz_actual, blockoffset_last, self.path,
                        )
                    )
                );
            } else if bo_at == blockoffset_last {
                // last block, is blocksz correct?
                if blocklen_sz > self.blocksz {
                    let byte_at = self.file_offset_at_block_offset_self(bo_at);
                    defx!("({}): return Err", blockoffset);
                    return ResultS3ReadBlock::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "xz_decompress read {} bytes for last block {} (at byte {}) which is larger than block size {} bytes; file {:?}",
                                blocklen_sz, bo_at, byte_at, self.blocksz, self.path,
                            )
                        )
                    );
                }
            } else if blocklen_sz != self.blocksz {
                // not last block, is blocksz correct?
                let byte_at = self.file_offset_at_block_offset_self(bo_at) + blocklen_sz;
                defx!("({}): return Err", blockoffset);
                return ResultS3ReadBlock::Err(
                    Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "xz_decompress read only {} bytes for block {} expected to read {} bytes (block size), inflate stopped at byte {}. block last {}, filesz {}, filesz uncompressed {} (according to gzip header); file {:?}",
                            blocklen_sz, bo_at, self.blocksz, byte_at, blockoffset_last, self.filesz, self.filesz_actual, self.path,
                        )
                    )
                );
            }

            // store decompressed block
            let blockp = BlockP::new(block);
            self.store_block_in_storage(bo_at, &blockp);
            if bo_at == blockoffset {
                defx!("({}): return Found", blockoffset);
                return ResultS3ReadBlock::Found(blockp);
            }
            bo_at += 1;
        }
        defx!("({}): return Done", blockoffset);

        ResultS3ReadBlock::Done
    }

    /// Read a block of data from a file within a .tar archive file
    /// ([`FileType::Tar`]).
    /// `BlockOffset` refers to the untarred version of the file.
    ///
    /// Called from `read_block`.
    ///
    /// XXX: This reads the entire file within the `.tar` file during the first call.
    ///      See Issue #13
    ///
    /// This one big read is due to crate `tar` not providing a method to store
    /// [`tar::Archive`] or [`tar::Entry`] due to inter-instance references and
    /// explicit lifetimes.
    /// A `tar::Entry` holds a reference to data within the `tar::Archive`.
    /// I found it impossible to store both related instances and then
    /// later utilize the `tar::Entry`.
    ///
    /// [`tar::Archive`]: https://docs.rs/tar/0.4.38/tar/struct.Archive.html
    /// [`tar::Entry`]: https://docs.rs/tar/0.4.38/tar/struct.Entry.html
    /// [`FileType::Tar`]: crate::common::FileType
    #[allow(non_snake_case)]
    fn read_block_FileTar(
        &mut self,
        blockoffset: BlockOffset,
    ) -> ResultS3ReadBlock {
        defn!("({})", blockoffset);
        debug_assert!(
            matches!(
                self.filetype,
                FileType::Evtx{ archival_type: FileTypeArchive::Tar }
                | FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, .. }
                | FileType::Journal{ archival_type: FileTypeArchive::Tar }
                | FileType::Text{ archival_type: FileTypeArchive::Tar, .. }
            ),
            "wrong FileType {:?} for calling read_block_FileTar",
            self.filetype
        );
        debug_assert_le!(
            self.count_blocks_processed(),
            blockoffset,
            "count_blocks_processed() {}, blockoffset {}; has read_block_FileTar been errantly called?",
            self.count_blocks_processed(),
            blockoffset
        );

        let path_ = self.path.clone();
        let path_std: &Path = Path::new(&path_);
        let mut archive: TarHandle = match BlockReader::open_tar(path_std) {
            Ok(val) => val,
            Err(err) => {
                defx!("Err {:?}", err);
                return err_from_err_path_results3(&err, &self.path, Some("(open_tar(…) failed)"));
            }
        };

        // get the file entry from the `.tar` file
        let mut entry = {
            let index = self
                .tar
                .as_ref()
                .unwrap()
                .entry_index;
            defx!("index {:?}", index);
            let mut entry_iter: tar::Entries<File> = match archive.entries_with_seek() {
                Ok(val) => val,
                Err(err) => {
                    defx!("Err {:?}", err);
                    return err_from_err_path_results3(&err, &self.path, Some("(entries_with_seek() failed)"));
                }
            };
            match entry_iter.nth(index) {
                Some(entry_res) => match entry_res {
                    Ok(entry) => entry,
                    Err(err) => {
                        defx!("Err {:?}", err);
                        return err_from_err_path_results3(&err, &self.path, Some(
                            format!("(entry_iter.nth({}) failed)", index).as_str()
                        ));
                    }
                },
                None => {
                    defx!("None");
                    return ResultS3ReadBlock::Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                            "tar.handle.entries_with_seek().entry_iter.nth({}) returned None for file {:?}",
                            index, self.path
                        ),
                    ));
                }
            }
        };

        // handle special case immediately
        // also file size zero cause `entry.read_exact` to return an error
        if self.filesz_actual == 0 {
            defx!("({}): filesz_actual 0; return Done", blockoffset);
            return ResultS3ReadBlock::Done;
        }

        // read all blocks from file `entry`
        let mut bo_at: BlockOffset = 0;
        let blockoffset_last: BlockOffset = self.blockoffset_last();
        while bo_at <= blockoffset_last {
            defo!("loop bo_at={}", bo_at);
            let cap: usize = self.blocksz_at_blockoffset(&bo_at) as usize;
            defo!("cap={}", cap);
            let mut block: Block = vec![0; cap];
            defo!("read_exact(&block (capacity {})); bo_at {}", cap, bo_at);
            match entry.read_exact(block.as_mut_slice()) {
                Ok(_) => {}
                Err(err) => {
                    defx!(
                        "read_block_FileTar: read_exact(&block (capacity {})) error, return {:?}",
                        cap,
                        err
                    );
                    e_err!("entry.read_exact(&block (capacity {})) file {:?} {}", cap, path_std, err);
                    return err_from_err_path_results3(&err, &self.path, Some("(entry.read_exact(…) failed)"));
                }
            }
            self.count_bytes_read += block.len() as Count;
            defo!("count_bytes_read={}", self.count_bytes_read);

            // check returned Block is expected number of bytes
            if block.is_empty() {
                defo!("block.is_empty()");
                let byte_at: FileOffset = self.file_offset_at_block_offset_self(bo_at);
                return ResultS3ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                            "read_exact read zero bytes from block {} (at byte {}), requested {} bytes. filesz {}, last block {}; file {:?}",
                            bo_at, byte_at, self.blocksz, self.filesz(), blockoffset_last, self.path,
                        )
                    )
                );
            } else if cap != block.len() {
                defo!("cap {} != {} block.len()", cap, block.len());
                let byte_at: FileOffset = self.file_offset_at_block_offset_self(bo_at);
                return ResultS3ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                            "read_exact read {} bytes from block {} (at byte {}), requested {} bytes. filesz {}, last block {}; file {:?}",
                            block.len(), bo_at, byte_at, self.blocksz, self.filesz(), blockoffset_last, self.path,
                        )
                    )
                );
            }

            let blockp: BlockP = BlockP::new(block);
            self.store_block_in_storage(bo_at, &blockp);
            bo_at += 1;
        }
        // all blocks have been read...

        // return only the block requested
        let blockp: BlockP = match self.blocks.get(&blockoffset) {
            Some(blockp_) => blockp_.clone(),
            None => {
                defx!("self.blocks.get({}), returned None, return Err(UnexpectedEof)", blockoffset);
                return ResultS3ReadBlock::Err(
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        format!(
                                "read_block_FileTar: self.blocks.get({}) returned None for file {:?}",
                                blockoffset, self.path
                        ),
                    )
                );
            }
        };

        defx!("({}): return Found", blockoffset);

        ResultS3ReadBlock::Found(blockp)
    }

    /// Read a `Block` of data of max size `self.blocksz` from the file.
    ///
    /// A successful read returns [`Found(BlockP)`].
    ///
    /// When at or past the end of the file and no data was read, returns
    /// [`Done`].
    ///
    /// All other `File` and `std::io` errors are propagated to the caller
    /// in [`Err`].
    ///
    /// [`Found(BlockP)`]: crate::common::ResultS3
    /// [`Done`]: crate::common::ResultS3
    /// [`Err`]: crate::common::ResultS3
    pub fn read_block(
        &mut self,
        blockoffset: BlockOffset,
    ) -> ResultS3ReadBlock {
        defn!(
            "({0}): blockreader.read_block({0}) (fileoffset {1} (0x{1:08X})), blocksz {2} (0x{2:08X}), filesz {3} (0x{3:08X})",
            blockoffset, self.file_offset_at_block_offset_self(blockoffset), self.blocksz, self.filesz(),
        );
        if cfg!(debug_assertions)
            && self.is_streamed_file()
            && self.is_drop_data()
            && self.read_block_last > blockoffset
        {
            debug_panic!(
                "read_block_last {} is greater than passed blockoffset {} for a is_streamed_file {:?}",
                self.read_block_last,
                blockoffset,
                self.path,
            );
        }
        self.read_block_last = blockoffset;

        if blockoffset > self.blockoffset_last() {
            defx!("({}) is past blockoffset_last {}; return Done", blockoffset, self.blockoffset_last());
            return ResultS3ReadBlock::Done;
        }
        {
            // check storages
            // check fast LRU cache storage
            if self.read_block_lru_cache_enabled {
                match self
                    .read_block_lru_cache
                    .get(&blockoffset)
                {
                    Some(bp) => {
                        self.read_block_cache_lru_hit += 1;
                        defx!(
                            "return Found(Block); hit LRU cache Block[{}] @[{}, {}) len {}",
                            &blockoffset,
                            BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz),
                            BlockReader::file_offset_at_block_offset(blockoffset + 1, self.blocksz),
                            (*bp).len(),
                        );
                        return ResultS3ReadBlock::Found(bp.clone());
                    }
                    None => {
                        self.read_block_cache_lru_miss += 1;
                        defo!("blockoffset {} not found LRU cache", blockoffset);
                    }
                }
            }
            // check hash map storage
            if self
                .blocks_read
                .contains(&blockoffset)
            {
                self.read_blocks_hit += 1;
                defo!("blocks_read.contains({})", blockoffset);
                // XXX: `loop` is only here to allow a rare call to `break`
                #[allow(clippy::never_loop)]
                loop {
                    let blockp: BlockP = match self
                        .blocks
                        .get_mut(&blockoffset)
                        {
                            Some(blockp) => blockp.clone(),
                            None => {
                                // BUG: getting here means something is wrong
                                //      with the internal state of `BlockReader` and likely
                                //      related to the caller's use of `drop_block`.
                                //      But we can go ahead and just read the block again though
                                //      that is inefficient and some day should be fixed.
                                debug_panic!(
                                    "requested block {} is in self.blocks_read but not in self.blocks for file {:?}",
                                    blockoffset, self.path,
                                );
                                self.read_blocks_reread_error += 1;
                                self.read_blocks_miss += 1;
                                self.blocks_read.remove(&blockoffset);
                                break;
                            }
                        };
                    self.store_block_in_LRU_cache(blockoffset, &blockp);
                    defx!(
                        "return Found(Block); use stored Block[{}] @[{}, {}) len {}",
                        &blockoffset,
                        BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz),
                        BlockReader::file_offset_at_block_offset(blockoffset + 1, self.blocksz),
                        self.blocks[&blockoffset].len(),
                    );
                    return ResultS3ReadBlock::Found(blockp);
                }
            } else {
                self.read_blocks_miss += 1;
                defo!("blockoffset {} not found in blocks_read", blockoffset);
                debug_assert!(
                    !self
                        .blocks
                        .contains_key(&blockoffset),
                    "blocks has element {} not in blocks_read",
                    blockoffset
                );
            }
        }

        match self.filetype {
            FileType::Evtx{ archival_type: _ } => panic!(
                "BlockReader::read_block unsupported filetype {:?}; path {:?}",
                self.filetype, self.path,
            ),
            FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, .. } => self.read_block_File(blockoffset),
            FileType::FixedStruct{ archival_type: FileTypeArchive::Bz2, .. } => self.read_block_FileBz2(blockoffset),
            FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, .. } => self.read_block_FileGz(blockoffset),
            FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, .. } => self.read_block_FileLz4(blockoffset),
            FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, .. } => self.read_block_FileTar(blockoffset),
            FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, .. } => self.read_block_FileXz(blockoffset),
            FileType::Journal{ archival_type: _ } => panic!(
                "BlockReader::read_block unsupported filetype {:?}; path {:?}",
                self.filetype, self.path,
            ),
            FileType::Text{ archival_type: FileTypeArchive::Normal, .. } => self.read_block_File(blockoffset),
            FileType::Text{ archival_type: FileTypeArchive::Bz2, .. } => self.read_block_FileBz2(blockoffset),
            FileType::Text{ archival_type: FileTypeArchive::Gz, .. } => self.read_block_FileGz(blockoffset),
            FileType::Text{ archival_type: FileTypeArchive::Lz4, .. } => self.read_block_FileLz4(blockoffset),
            FileType::Text{ archival_type: FileTypeArchive::Tar, .. } => self.read_block_FileTar(blockoffset),
            FileType::Text{ archival_type: FileTypeArchive::Xz, .. } => self.read_block_FileXz(blockoffset),
            FileType::Unparsable
            => panic!(
                "BlockReader::read_block bad filetype {:?}; path {:?}",
                self.filetype, self.path,
            ),
        }
    }

    /// return data \[`fileoffset_beg`, `fileoffset_end`), inclusive range to
    /// exclusive range, as sequence of
    /// [`BlockP`] with associated start [`BlockIndex`] and end `BlockIndex`.
    ///
    /// The returned "start `BlockIndex`" pertains to the first `BlockP` in the
    /// returned sequence. The returned "end `BlockIndex`" pertains to the
    /// last `BlockP` in the returned sequence.
    ///
    /// The return value contains enum [`ReadDataParts`]. This enum  allows the
    /// most common cases of one or
    /// two [`BlockP`] to be returned without allocation of a [`Vec`].
    /// In the rare case of three or more `BlockP`s then a `Vec` is allocated.
    ///
    /// Argument `oneblock` when `true` requires the [`Found`] referred data
    /// to reside within one [`Block`].
    /// If the data is not contained within one `Block` then returns [`Done`].
    ///
    /// Users that just need some not-too-huge slice of bytes should call
    /// [`read_data_to_buffer`] instead of this function.
    ///
    /// [`BlockP`]: BlockP
    /// [`Block`]: Block
    /// [`BlockIndex`s]: BlockIndex
    /// [`BlockIndex`]: BlockIndex
    /// [`ReadDataParts`]: ReadDataParts
    /// [`Vec`]: Vec
    /// [`Done`]: crate::common::ResultS3#variant.Done
    /// [`Found`]: crate::common::ResultS3#variant.Found
    pub(crate) fn read_data(
        &mut self,
        fileoffset_beg: FileOffset,
        fileoffset_end: FileOffset,
        oneblock: bool,
    ) -> ResultReadData
    {
        defn!("({}, {}, oneblock={})", fileoffset_beg, fileoffset_end, oneblock);
        debug_assert_le!(fileoffset_beg, fileoffset_end);
        let fileoffset_end: FileOffset = std::cmp::min(fileoffset_end, self.filesz());

        if fileoffset_beg >= fileoffset_end {
            defx!("offsets mismatch, return Done");
            return ResultReadData::Done;
        }

        let bo_last: BlockOffset = self.blockoffset_last();
        let mut bo1: BlockOffset = self.block_offset_at_file_offset_self(fileoffset_beg);
        let blockp1 = match self.read_block(bo1) {
            ResultS3ReadBlock::Found(blockp) => {
                defo!("read_block({}) returned Found Block len {}", bo1, (*blockp).len());

                blockp
            }
            ResultS3ReadBlock::Done => {
                defx!("read_block({}) returned Done; returning Done", bo1);
                return ResultReadData::Done;
            }
            ResultS3ReadBlock::Err(err) => {
                defx!("read_block({}) returned Error {:?}", bo1, err);
                return ResultReadData::Err(err);
            }
        };
        let bi1: BlockIndex = self.block_index_at_file_offset_self(fileoffset_beg);
        let mut bi2: BlockIndex = self.block_index_at_file_offset_self(fileoffset_end);
        let bo2: BlockOffset = match bi2 {
            0 => {
                bi2 = self.blocksz as BlockIndex;

                self.block_offset_at_file_offset_self(fileoffset_end) - 1
            }
            _ => self.block_offset_at_file_offset_self(fileoffset_end),
        };
        debug_assert_le!(bo1, bo2);

        // the input argument `fileoffset_end` is exclusive range
        // whereas BlockOffset `bo2` must be inclusive range
        // so adjust accordingly
        if bo1 == bo2 {
            // data resides within one Block
            bi2 = std::cmp::min(bi2, (*blockp1).len() as BlockIndex);
            defx!("return Found(One(len {}, {}, {}))", (*blockp1).len(), bi1, bi2);
            let rd: ReadData = (ReadDataParts::One(blockp1), bi1, bi2);
            return ResultReadData::Found(rd);
        }
        if bo1 == bo_last {
            // passed parameters that refer to data beyond the end of the file
            bi2 = std::cmp::min(bi2, (*blockp1).len() as BlockIndex);
            defx!("return Found(One(len {}, {}, {}))", (*blockp1).len(), bi1, bi2);
            let rd: ReadData = (ReadDataParts::One(blockp1), bi1, bi2);
            return ResultReadData::Found(rd);
        }

        if oneblock {
            defx!("return ResultReadData::Done; oneblock but [{}, {}) spans more than one block", fileoffset_beg, fileoffset_end);
            return ResultReadData::Done;
        }

        if bo1 + 1 == bo2 {
            // data spans two Blocks
            let blockp2 = match self.read_block(bo2) {
                ResultS3ReadBlock::Found(blockp) => {
                    defo!("read_block({}) returned Found Block len {}", bo2, (*blockp).len());

                    blockp
                }
                ResultS3ReadBlock::Done => {
                    defo!("read_block({}) returned Done", bo2);
                    let err = Error::new(
                        ErrorKind::Other,
                        format!(
                            "read_block({}) returned Done, block_last is {} for file {:?}",
                            bo2, bo_last, self.path
                        ),
                    );
                    defx!("return {:?}", err);
                    return ResultReadData::Err(err);
                }
                ResultS3ReadBlock::Err(err) => {
                    defx!("read_block({}) returned Error {:?}", bo2, err);
                    return ResultReadData::Err(err);
                }
            };

            if bo2 == bo_last {
                bi2 = std::cmp::min(bi2, (*blockp2).len() as BlockIndex);
            }
            defx!("return Found(Two(…, {}, {}))", bi1, bi2);
            let rd: ReadData = (ReadDataParts::Two(blockp1, blockp2), bi1, bi2);
            return ResultReadData::Found(rd);
        }

        // data spans more than two Blocks
        let mut blockps: Vec<BlockP> = Vec::with_capacity(3);
        defo!("blockps.push({})", bo1);
        blockps.push(blockp1);

        bo1 += 1;
        while bo1 <= bo2 {
            match self.read_block(bo1) {
                ResultS3ReadBlock::Found(blockp) => {
                    defo!("read_block({}) returned Found Block len {}", bo1, (*blockp).len());
                    defo!("blockps.push({})", bo1);
                    blockps.push(blockp);
                }
                ResultS3ReadBlock::Done => {
                    defo!("read_block({}) returned Done (fileoffset_end {} larger than filesz)", bo1, fileoffset_end);
                    break;
                }
                ResultS3ReadBlock::Err(err) => {
                    defx!("read_block({}) returned Error {:?}", bo1, err);
                    return ResultReadData::Err(err);
                }
            };
            bo1 += 1;
        }
        bi2 = std::cmp::min(bi2, blockps.last().unwrap().len() as BlockIndex);

        defx!("return Found(Many(…, {}, {}))", bi1, bi2);
        let rd: ReadData = (ReadDataParts::Many(blockps), bi1, bi2);

        ResultReadData::Found(rd)
    }

    /// Read data from the file into the passed `buffer`. Calls private
    /// function `read_data`.
    /// When successful, [`Found`] contains number of bytes read into the passed
    /// `buffer`.
    /// Data is read inclusive of `fileoffset_beg` and exclusive of
    /// `fileoffset_end`.
    ///
    /// Argument `oneblock` is passed to `read_data`.
    ///
    /// [`Found`]: ResultReadDataToBuffer
    pub fn read_data_to_buffer(
        &mut self,
        fileoffset_beg: FileOffset,
        fileoffset_end: FileOffset,
        oneblock: bool,
        buffer: &mut [u8]
    ) -> ResultReadDataToBuffer {
        defn!("({}, {}, oneblock={}, buffer len {})", fileoffset_beg, fileoffset_end, oneblock, buffer.len());
        read_data_to_buffer_len_check!(buffer.len(), 1, self.path);
        let readdata: ReadData = match self.read_data(
            fileoffset_beg,
            fileoffset_end,
            oneblock,
        ) {
            ResultReadData::Found(readdata) => readdata,
            ResultReadData::Done => {
                defx!("return ResultReadDataToBuffer::Done");
                return ResultReadDataToBuffer::Done;
            }
            ResultReadData::Err(err) => {
                defx!("return ResultReadDataToBuffer::Err({})", err);
                return ResultReadDataToBuffer::Err(err);
            }
        };

        let mut at: usize = 0;
        let bi1: usize = readdata.1;
        let bi2: usize = readdata.2;
        match readdata.0 {
            ReadDataParts::One(blockp) => {
                //let n = std::cmp::min((*blockp).len(), bi2 - bi1);
                let n = bi2 - bi1;
                defo!("copy ‥{}", at + n);
                read_data_to_buffer_len_check!(buffer.len(), at + n, self.path);
                buffer[..at + n].copy_from_slice(&(*blockp).as_slice()[bi1..bi1 + n]);
                at += n;
            }
            ReadDataParts::Two(blockp1, blockp2) => {
                debug_assert!(!oneblock);
                if oneblock {
                    defx!("return ResultReadDataToBuffer::Done; oneblock but had two blocks");
                    return ResultReadDataToBuffer::Done;
                }
                // two blocks
                // first block
                debug_assert_eq!((*blockp1).len(), self.blocksz() as usize);
                //let n = std::cmp::min((*blockp1).len(), (*blockp1).len() - bi1);
                let n = (*blockp1).len() - bi1;
                defo!("copy ‥{}", at + n);
                read_data_to_buffer_len_check!(buffer.len(), at + n, self.path);
                buffer[..at + n].copy_from_slice(&(*blockp1).as_slice()[bi1..]);
                at += n;
                // last block
                //let n = std::cmp::min((*blockp2).len() - at, bi2);
                let n = bi2;
                defo!("copy {}‥{}", at, at + n);
                read_data_to_buffer_len_check!(buffer.len(), at + n, self.path);
                buffer[at..at + n].copy_from_slice(&(*blockp2).as_slice()[..n]);
                at += n;
            }
            ReadDataParts::Many(blockps) => {
                debug_assert!(!oneblock);
                if oneblock {
                    defx!("return ResultReadDataToBuffer::Done; oneblock but had many blocks");
                    return ResultReadDataToBuffer::Done;
                }
                debug_assert_ge!(blockps.len(), 3);
                // more than two blocks
                let len_: usize = blockps.len();
                defo!("readdata (blockps.len()={}, {}, {})", len_, bi1, bi2);
                // first block
                debug_assert_eq!(blockps[0].len(), self.blocksz() as usize);
                //let n = std::cmp::min(blockps[0].len(), blockps[0].len() - bi1);
                let n = blockps[0].len() - bi1;
                defo!("copy ‥{}", at + n);
                read_data_to_buffer_len_check!(buffer.len(), at + n, self.path);
                buffer[..at + n].copy_from_slice(&blockps[0].as_slice()[bi1..bi1 + n]);
                at += n;
                // middle block(s)
                for blockp in blockps.iter().skip(1).take(len_ - 2) {
                    debug_assert_eq!((*blockp).len(), self.blocksz() as usize);
                    let n = (*blockp).len();
                    defo!("copy {}‥{}", at, at + n);
                    read_data_to_buffer_len_check!(buffer.len(), at + n, self.path);
                    buffer[at..at + n].copy_from_slice(&blockp.as_slice()[..n]);
                    at += n;
                };
                // last block
                //let n = std::cmp::min(blockps[len_ - 1].len(), bi2);
                let n = bi2;
                defo!("copy {}‥{}", at, at + n);
                read_data_to_buffer_len_check!(buffer.len(), at + n, self.path);
                buffer[at..at + n].copy_from_slice(&blockps[len_ - 1].as_slice()[..n]);
                at += n;
            }
        }

        ResultReadDataToBuffer::Found(at)
    }

    /// Helper function to open a `.tar` file.
    pub fn open_tar(path_tar: &Path) -> Result<TarHandle> {
        let mut open_options = FileOpenOptions::new();
        defo!("open_options.read(true).open({:?})", path_tar);
        let file_tar: File = match open_options
            .read(true)
            .open(path_tar)
        {
            Ok(val) => val,
            Err(err) => {
                defx!("open_options.read({:?}) Error, return {:?}", path_tar, err);
                return Err(err);
            }
        };

        Ok(TarHandle::new(file_tar))
    }

    /// For testing only, very inefficient!
    #[cfg(test)]
    pub(crate) fn get_block(
        &self,
        blockoffset: &BlockOffset,
    ) -> Option<Bytes> {
        if self
            .blocks
            .contains_key(blockoffset)
        {
            return Some((*self.blocks[blockoffset]).clone());
        }

        None
    }

    #[allow(non_snake_case)]
    pub fn summary(&self) -> SummaryBlockReader {
        let blockreader_bytes = self
            .count_bytes();
        let blockreader_bytes_total = self.filesz() as FileSz;
        let blockreader_blocks = self
            .count_blocks_processed();
        let blockreader_blocks_total = self
            .blockn;
        let blockreader_blocksz = self.blocksz();
        let blockreader_filesz = self
            .filesz;
        let blockreader_filesz_actual = self
            .filesz_actual;
        let blockreader_read_block_lru_cache_hit = self
            .read_block_cache_lru_hit;
        let blockreader_read_block_lru_cache_miss = self
            .read_block_cache_lru_miss;
        let blockreader_read_block_lru_cache_put = self
            .read_block_cache_lru_put;
        let blockreader_read_blocks_hit = self
            .read_blocks_hit;
        let blockreader_read_blocks_miss = self
            .read_blocks_miss;
        let blockreader_read_blocks_put = self
            .read_blocks_put;
        let blockreader_read_blocks_reread_error = self
            .read_blocks_reread_error;
        let blockreader_blocks_highest = self
            .blocks_highest;
        let blockreader_blocks_dropped_ok = self
            .dropped_blocks_ok;
        let blockreader_blocks_dropped_err = self
            .dropped_blocks_err;

        SummaryBlockReader {
            blockreader_bytes,
            blockreader_bytes_total,
            blockreader_blocks,
            blockreader_blocks_total,
            blockreader_blocksz,
            blockreader_filesz,
            blockreader_filesz_actual,
            blockreader_read_block_lru_cache_hit,
            blockreader_read_block_lru_cache_miss,
            blockreader_read_block_lru_cache_put,
            blockreader_read_blocks_hit,
            blockreader_read_blocks_miss,
            blockreader_read_blocks_put,
            blockreader_read_blocks_reread_error,
            blockreader_blocks_highest,
            blockreader_blocks_dropped_ok,
            blockreader_blocks_dropped_err,
        }
    }

}
