// src/common.rs

//! Common imports, type aliases, and other globals for _s4lib_.

use std::collections::HashSet;
use std::fmt::Debug;
#[doc(hidden)]
pub use std::fs::File;
use std::io::{
    Error,
    Result,
};
#[doc(hidden)]
pub use std::path::Path;
use std::thread;

use ::chrono::FixedOffset;
use ::kinded::Kinded;

/// `F`ake `Path` or `F`ile `Path`.
///
/// Type alias `FPath` is a simpler stand-in for formalized file system path
/// [`std::path::Path`].
///
/// An `FPath`can handle "subpaths" (paths into archives or compressed files)
/// by adding an ad-hoc separator.
///
/// It's easier to pass around a [`String`] than a `Path`.
/// `std::path::Path` does not have trait `Sized` so
/// instances of `std::path::Path` must be passed-by-reference
/// (rustc error "`size is not known at compile time`").
/// Some code areas then require marking explicit lifetimes 😩<br/>
/// It's much easier to use a [`String`] and convert the `Path` as needed.
pub type FPath = String;

/// A sequence of [`FPath`]s
pub type FPaths = Vec<FPath>;

#[doc(hidden)]
pub type FileMetadata = std::fs::Metadata;

#[doc(hidden)]
pub type FileOpenOptions = std::fs::OpenOptions;

/// File Size in bytes
pub type FileSz = u64;

/// A general-purpose counting type, typically used for internal statistics
/// (Summary) counting.
pub type Count = u64;

/// File paths are needed as keys. Many such keys are passed around among
/// different threads.
/// Instead of passing clones of `FPath`, pass around a relatively light-weight
/// `usize` as a key.
/// The main processing thread uses the `PathId` key for various lookups,
/// including the file path.
pub type PathId = usize;

/// a set of [`PathId`]
pub type SetPathId = HashSet<PathId>;

/// status of whether summary statistics are enabled
///
/// only access via `summary_stats_enabled()` function. unsafe global variable!
///
/// _XXX:_ skips use `LazyStatic`, `OnceCell`, etc. for better performance as it
///      is accessed often.
///
/// TODO: prove the prior statement with a benchmark
static mut SUMMARY_STATS_ENABLED: bool = false;
/// sanity check
static mut SUMMARY_STATS_INITIALIZED: bool = false;

/// enable summary statistics
///
/// only call this from the main thread once; multi-thread unsafe!
pub fn summary_stats_enable() {
    unsafe {
        SUMMARY_STATS_ENABLED = true;
        if ! cfg!(test) && SUMMARY_STATS_INITIALIZED {
            panic!("summary_stats_enable() called more than once");
        }
        SUMMARY_STATS_INITIALIZED = true;
    }
}

/// Check if summary statistics are enabled.
#[inline(always)]
pub fn summary_stats_enabled() -> bool {
    unsafe {
        SUMMARY_STATS_ENABLED
    }
}

/// Only execute the given expression if summary statistics are enabled.
///
/// This macro wraps expressions that update summary statistics. This
/// reduces a trivial amount of overhead and more
/// importantly clarifies which expressions are related to summary statistics.
#[macro_export]
macro_rules! summary_stat {
    ($($arg:expr)*) => (
        if $crate::common::summary_stats_enabled() {
            $($arg)*;
        }
    )
}
pub use summary_stat;

/// If summary statistics are enabled, do the first expression, else do the second.
///
/// This macro wraps expressions that update summary statistics. This
/// reduces a trivial amount of overhead and more
/// importantly clarifies which expressions are related to summary statistics.
#[macro_export]
macro_rules! summary_stat_set {
    ($($arg_if_true:expr)*, $($arg_if_false:expr)*) => (
        if $crate::common::summary_stats_enabled() {
            $($arg_if_true)*
        } else {
            $($arg_if_false)*
        }
    )
}
pub use summary_stat_set;

/// signifier of which allocator was chosen
pub enum AllocatorChosen {
    System = 1,
    Jemalloc = 2,
    Mimalloc = 3,
    TCMalloc = 4,
}

impl std::fmt::Display for AllocatorChosen {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        match self {
            AllocatorChosen::System => write!(f, "system"),
            AllocatorChosen::Jemalloc => write!(f, "jemalloc"),
            AllocatorChosen::Mimalloc => write!(f, "mimalloc"),
            AllocatorChosen::TCMalloc => write!(f, "tcmalloc"),
        }
    }
}

/// panics in debug builds, otherwise a no-op
#[macro_export]
macro_rules! debug_panic {
    ($($arg:tt)*) => (
        // XXX: use `if cfg!` instead of `#[cfg(...)]` to avoid
        //      `unreachable_code` warning
        if cfg!(any(debug_assertions, test))
        {
            panic!($($arg)*);
        }
    )
}
pub use debug_panic;

/// Assert if the the argument is `None`, allow optional message, only in debug builds.
#[macro_export]
macro_rules! debug_assert_none {
    ($arg:expr) => {
        if cfg!(any(debug_assertions, test))
        {
            if ! $arg.is_none() {
                panic!("'{}' is not None", stringify!($arg));
            }
        }
    };
    ($arg:expr, $($extra_message:tt)*) => {
        if cfg!(any(debug_assertions, test))
        {
            if ! $arg.is_none() {
                panic!("'{}' is not None; {}", stringify!($arg), format_args!($($extra_message)*));
            }
        }
    };
}
pub use debug_assert_none;

/// Assert if the any of the arguments are `None`, only in debug builds.
#[macro_export]
macro_rules! debug_assert_nones {
    // TODO: change this to allow passing format arguments
    ($($arg:expr),+) => {
        $(
            $crate::debug_assert_none!($arg);
        )+
    };
}
pub use debug_assert_nones;

pub const FIXEDOFFSET0: FixedOffset = FixedOffset::east_opt(0).unwrap();

// --------------------------------------------------
// custom Results enums for various *Reader functions

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#481-495

/// [`Result`]-like result extended for `s4` to 3 types.
///
/// For various "find" functions implemented by [Readers].
///
/// [`Result`]: std::result::Result
/// [Readers]: crate::readers
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultFind<T, E> {
    /// Contains the found data.
    Found(T),
    /// File is empty, or a request reached the end of the file or beyond the
    /// end, or some other condition that means "_Nothing to do, done_".
    ///
    /// Does not imply an error occurred.
    Done,
    /// Something bad happened. Contains the `E` error data.
    Err(E),
}

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#501-659

impl<T, E> ResultFind<T, E> {
    // Querying the contained values

    /// Returns `true` if the result is [`Found`], [`Done`].
    ///
    /// [`Found`]: self::ResultFind#variant.Found
    /// [`Done`]: self::ResultFind#variant.Done
    /// [`Err`]: self::ResultFind#variant.Err
    #[allow(dead_code)]
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline(always)]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, ResultFind::Found(_) | ResultFind::Done)
    }

    /// Returns `true` if the result is [`Found`].
    ///
    /// [`Found`]: self::ResultFind#variant.Found
    #[inline(always)]
    pub const fn is_found(&self) -> bool {
        matches!(*self, ResultFind::Found(_))
    }

    /// Returns `true` if the result is [`Err`].
    ///
    /// [`Err`]: self::ResultFind#variant.Err
    #[allow(dead_code)]
    #[must_use = "if you intended to assert that this is err, consider `.unwrae_err()` instead"]
    #[inline(always)]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Returns `true` if the result is [`Done`].
    ///
    /// [`Done`]: self::ResultFind#variant.Done
    #[inline(always)]
    pub const fn is_done(&self) -> bool {
        matches!(*self, ResultFind::Done)
    }

    /// Returns `true` if the result is an [`Found`] value containing the given
    /// value.
    ///
    /// [`Found`]: self::ResultFind#variant.Found
    #[allow(dead_code)]
    #[must_use]
    #[inline(always)]
    pub fn contains<U>(
        &self,
        x: &U,
    ) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            ResultFind::Found(y) => x == y,
            ResultFind::Done => false,
            ResultFind::Err(_) => false,
        }
    }

    /// Returns `true` if the result is an [`Err`] value containing the given
    /// value.
    ///
    /// [`Err`]: self::ResultFind#variant.Err
    #[allow(dead_code)]
    #[must_use]
    #[inline(always)]
    pub fn contains_err<F>(
        &self,
        f: &F,
    ) -> bool
    where
        F: PartialEq<E>,
    {
        match self {
            ResultFind::Err(e) => f == e,
            _ => false,
        }
    }

    // Adapter for each variant

    /// Converts from `ResultFind<T, E>` to [`Option<T>`].
    ///
    /// Converts `self` into an [`Option<T>`], consuming `self`,
    /// and discarding the error, if any.
    #[allow(dead_code)]
    #[inline(always)]
    pub fn ok(self) -> Option<T> {
        match self {
            ResultFind::Found(x) => Some(x),
            ResultFind::Done => None,
            ResultFind::Err(_) => None,
        }
    }

    /// Converts from `ResultFind<T, E>` to [`Option<E>`].
    ///
    /// Converts `self` into an [`Option<E>`], consuming `self`,
    /// and discarding the success value, if any.
    #[allow(dead_code)]
    #[inline(always)]
    pub fn err(self) -> Option<E> {
        match self {
            ResultFind::Found(_) => None,
            ResultFind::Done => None,
            ResultFind::Err(x) => Some(x),
        }
    }
}

impl<T, E> std::fmt::Display for ResultFind<T, E>
where
    E: std::fmt::Display,
{
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ResultFind::Found(_) => {
                write!(f, "ResultFind::Found")
            }
            ResultFind::Done => {
                write!(f, "ResultFind::Done")
            }
            ResultFind::Err(err) => {
                write!(f, "ResultFind::Err({})", err)
            }
        }
    }
}

/// [`Result`]-like result extended for `s4` to 4 types, distinguishing between
/// errors that should halt file processing and errors that should not.
///
/// For various "find" functions implemented by [Readers].
///
/// [`Result`]: std::result::Result
/// [Readers]: crate::readers
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultFind4<T, E> {
    /// Contains the found data.
    Found(T),
    /// File is empty, or a request reached the end of the file or beyond the
    /// end, or some other condition that means "_Nothing to do, done_".
    ///
    /// Does not imply an error occurred.
    Done,
    /// Something bad happened. Contains the `E` error data. Further file
    /// processing should be halted.
    Err(E),
    /// Something bad happened. Contains the `E` error data. File processing
    /// can continue.
    ErrIgnore(E),
}

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#501-659

impl<T, E> ResultFind4<T, E> {
    // Querying the contained values

    /// Returns `true` if the result is [`Found`], [`Done`].
    ///
    /// [`Found`]: self::ResultFind4#variant.Found
    /// [`Done`]: self::ResultFind4#variant.Done
    /// [`Err`]: self::ResultFind4#variant.Err
    #[allow(dead_code)]
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline(always)]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, ResultFind4::Found(_) | ResultFind4::Done)
    }

    /// Returns `true` if the result is [`Found`].
    ///
    /// [`Found`]: self::ResultFind4#variant.Found
    #[inline(always)]
    pub const fn is_found(&self) -> bool {
        matches!(*self, ResultFind4::Found(_))
    }

    /// Returns `true` if the result is [`Err`].
    ///
    /// [`Err`]: self::ResultFind4#variant.Err
    #[allow(dead_code)]
    #[must_use = "if you intended to assert that this is err, consider `.unwrae_err()` instead"]
    #[inline(always)]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Returns `true` if the result is [`Done`].
    ///
    /// [`Done`]: self::ResultFind4#variant.Done
    #[inline(always)]
    pub const fn is_done(&self) -> bool {
        matches!(*self, ResultFind4::Done)
    }

    /// Returns `true` if the result is an [`Found`] value containing the given
    /// value.
    ///
    /// [`Found`]: self::ResultFind4#variant.Found
    #[allow(dead_code)]
    #[must_use]
    #[inline(always)]
    pub fn contains<U>(
        &self,
        x: &U,
    ) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            ResultFind4::Found(y) => x == y,
            _ => false,
        }
    }

    /// Returns `true` if the result is an [`Err`] value containing the given
    /// value.
    ///
    /// [`Err`]: self::ResultFind4#variant.Err
    #[allow(dead_code)]
    #[must_use]
    #[inline(always)]
    pub fn contains_err<F>(
        &self,
        f: &F,
    ) -> bool
    where
        F: PartialEq<E>,
    {
        match self {
            ResultFind4::Err(e) => f == e,
            ResultFind4::ErrIgnore(e) => f == e,
            _ => false,
        }
    }

    // Adapter for each variant

    /// Converts from `ResultFind4<T, E>` to [`Option<T>`].
    ///
    /// Converts `self` into an [`Option<T>`], consuming `self`,
    /// and discarding the error, if any.
    #[allow(dead_code)]
    #[inline(always)]
    pub fn ok(self) -> Option<T> {
        match self {
            ResultFind4::Found(x) => Some(x),
            _ => None,
        }
    }

    /// Converts from `ResultFind4<T, E>` to [`Option<E>`].
    ///
    /// Converts `self` into an [`Option<E>`], consuming `self`,
    /// and discarding the success value, if any.
    #[allow(dead_code)]
    #[inline(always)]
    pub fn err(self) -> Option<E> {
        match self {
            ResultFind4::Found(_) => None,
            ResultFind4::Done => None,
            ResultFind4::ErrIgnore(x)
            | ResultFind4::Err(x) => Some(x),
        }
    }
}

impl<T, E> std::fmt::Display for ResultFind4<T, E>
where
    E: std::fmt::Display,
{
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ResultFind4::Found(_) => {
                write!(f, "ResultFind4::Found")
            }
            ResultFind4::Done => {
                write!(f, "ResultFind4::Done")
            }
            ResultFind4::Err(err) => {
                write!(f, "ResultFind4::Err({})", err)
            }
            ResultFind4::ErrIgnore(err) => {
                write!(f, "ResultFind4::ErrIgnore({})", err)
            }
        }
    }
}

/// [`Result`]-like result extended 3 types, distinguishing between
/// errors that may be reprinted and errors that should not.
///
/// [`Result`]: std::result::Result
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Result3<T, E> {
    /// Success
    Ok(T),
    /// Error that may be reprinted
    Err(E),
    /// Error that should not be reprinted
    ErrNoReprint(E),
}

pub type Result3E<T> = Result3<T, std::io::Error>;

impl<T, E> Result3<T, E> {
    /// Returns `true` if the result is [`Ok`].
    ///
    /// [`Ok`]: self::Result3#variant.Ok
    #[inline(always)]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, Result3::Ok(_))
    }

    /// Returns `true` if the result is [`Err`] or [`ErrNoReprint`].
    ///
    /// [`Err`]: self::Result3#variant.Err
    /// [`ErrNoReprint`]: self::Result3#variant.ErrNoReprint
    #[inline(always)]
    pub const fn is_err(&self) -> bool {
        matches!(*self, Result3::Err(_) | Result3::ErrNoReprint(_))
    }
}

/// A file size in bytes equal or less is too small and will not be processed
pub const FILE_TOO_SMALL_SZ: FileSz = 5;

/// Enum return value for various [filepreprocessor] functions.
///
/// [filepreprocessor]: crate::readers::filepreprocessor
#[derive(Debug)]
pub enum FileProcessingResult<E> {
    FileErrEmpty,
    FileErrTooSmall,
    /// `FileErrTooSmall` but with a custom message
    FileErrTooSmallS(String),
    FileErrNullBytes,
    FileErrNoLinesFound,
    FileErrNoSyslinesFound,
    FileErrNoSyslinesInDtRange,
    FileErrNoValidFixedStruct,
    FileErrNoFixedStructInDtRange,
    /// Carries the `E` error data. This is how an [`Error`] is carried between
    /// a processing thread and the main printing thread.
    ///
    /// [`Error`]: std::io::Error
    FileErrIo(E),
    /// Like `FileErrIo` but the message string includes the path of the
    /// processed file that elicited the error. Later processing of this
    /// error will not append the path of the processed file.
    FileErrIoPath(E),
    FileErrWrongType,
    FileErrDecompress,
    /// Do not use this error. Merely a stand-in.
    /// The real error should have been processed elsewhere.
    FileErrStub,
    FileErrChanSend,
    FileOk,
}

impl<E> FileProcessingResult<E> {
    /// Returns `true` if the result is [`FileOk`].
    ///
    /// [`FileOk`]: self::FileProcessingResult#variant.FileOk
    #[inline(always)]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, FileProcessingResult::FileOk)
    }

    /// Returns `true` if the result is [`FileErrStub`].
    ///
    /// [`FileErrStub`]: self::FileProcessingResult#variant.FileErrStub
    #[inline(always)]
    pub const fn is_stub(&self) -> bool {
        matches!(*self, FileProcessingResult::FileErrStub)
    }

    /// Returns `true` if the result is [`FileErrIo`].
    ///
    /// [`FileErrIo`]: self::FileProcessingResult#variant.FileErrIo
    #[inline(always)]
    pub const fn has_err(&self) -> bool {
        matches!(*self, FileProcessingResult::FileErrIo(_) | FileProcessingResult::FileErrIoPath(_))
    }
}

impl<E> std::fmt::Display for FileProcessingResult<E>
where
    E: std::fmt::Display,
{
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            FileProcessingResult::FileErrEmpty => {
                write!(f, "file empty")
            }
            FileProcessingResult::FileErrTooSmall => {
                write!(f, "file too small")
            }
            FileProcessingResult::FileErrTooSmallS(msg) => {
                write!(f, "file too small: {}", msg)
            }
            FileProcessingResult::FileErrNullBytes => {
                write!(f, "file contains null bytes")
            }
            FileProcessingResult::FileErrNoLinesFound => {
                write!(f, "no lines found in file")
            }
            FileProcessingResult::FileErrNoSyslinesFound => {
                write!(f, "no syslog lines found in file")
            }
            FileProcessingResult::FileErrNoSyslinesInDtRange => {
                write!(f, "no syslog lines found in date/time range")
            }
            FileProcessingResult::FileErrNoValidFixedStruct => {
                write!(f, "no valid fixed-structure records found in file")
            }
            FileProcessingResult::FileErrNoFixedStructInDtRange => {
                write!(f, "no fixed-structure records found in date/time range")
            }
            FileProcessingResult::FileErrIo(err) => {
                write!(f, "{}", err)
            }
            FileProcessingResult::FileErrIoPath(err) => {
                write!(f, "{}", err)
            }
            FileProcessingResult::FileErrWrongType => {
                write!(f, "wrong file type")
            }
            FileProcessingResult::FileErrDecompress => {
                write!(f, "decompression error")
            }
            FileProcessingResult::FileErrStub => {
                write!(f, "stub error")
            }
            FileProcessingResult::FileErrChanSend => {
                write!(f, "channel send error")
            }
            FileProcessingResult::FileOk => {
                write!(f, "file OK")
            }
        }
    }
}

/// Manually implement [`PartialEq`].
///
/// `#[derive(PartialEq)]` often fails because `E` does not implement it.
///
/// [PartialEq]: std::cmp::PartialEq
impl<E> PartialEq for FileProcessingResult<E> {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        match self {
            FileProcessingResult::FileErrEmpty => {
                matches!(*other, FileProcessingResult::FileErrEmpty)
            }
            FileProcessingResult::FileErrTooSmall => {
                matches!(*other, FileProcessingResult::FileErrTooSmall)
            }
            FileProcessingResult::FileErrTooSmallS(_) => {
                matches!(*other, FileProcessingResult::FileErrTooSmallS(_))
            }
            FileProcessingResult::FileErrNullBytes => {
                matches!(*other, FileProcessingResult::FileErrNullBytes)
            }
            FileProcessingResult::FileErrNoLinesFound => {
                matches!(*other, FileProcessingResult::FileErrNoLinesFound)
            }
            FileProcessingResult::FileErrNoSyslinesFound => {
                matches!(*other, FileProcessingResult::FileErrNoSyslinesFound)
            }
            FileProcessingResult::FileErrNoSyslinesInDtRange => {
                matches!(*other, FileProcessingResult::FileErrNoSyslinesInDtRange)
            }
            FileProcessingResult::FileErrNoValidFixedStruct => {
                matches!(*other, FileProcessingResult::FileErrNoValidFixedStruct)
            }
            FileProcessingResult::FileErrNoFixedStructInDtRange => {
                matches!(*other, FileProcessingResult::FileErrNoFixedStructInDtRange)
            }
            FileProcessingResult::FileErrIo(_) => {
                matches!(*other, FileProcessingResult::FileErrIo(_))
            }
            FileProcessingResult::FileErrIoPath(_) => {
                matches!(*other, FileProcessingResult::FileErrIoPath(_))
            }
            FileProcessingResult::FileErrWrongType => {
                matches!(*other, FileProcessingResult::FileErrWrongType)
            }
            FileProcessingResult::FileErrDecompress => {
                matches!(*other, FileProcessingResult::FileErrDecompress)
            }
            FileProcessingResult::FileErrStub => {
                matches!(*other, FileProcessingResult::FileErrStub)
            }
            FileProcessingResult::FileErrChanSend => {
                matches!(*other, FileProcessingResult::FileErrChanSend)
            }
            FileProcessingResult::FileOk => {
                matches!(*other, FileProcessingResult::FileOk)
            }
        }
    }
}
impl<E> Eq for FileProcessingResult<E> {}

/// the various kinds of fixedstruct files, i.e. C-struct records
// BUG: `Kinded` claims to automatically derive multiple traits but does not.
//      See claims in <https://crates.io/crates/kinded/0.3.0>
//      > By default the kind type implements the following traits:
//      > Debug, Clone, Copy, PartialEq, Eq, Display, FromStr, From<T>, From<&T>.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Kinded)]
pub enum FileTypeFixedStruct {
    Acct,
    AcctV3,
    Lastlog,
    Lastlogx,
    Utmp,
    Utmpx,
}

/// Text file encoding type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Kinded)]
pub enum FileTypeTextEncoding {
    /// UTF8 or ASCII encoding
    Utf8Ascii,
    // UTF16 encoding
    Utf16,
    // UTF32 encoding
    Utf32,
}

impl std::fmt::Display for FileTypeTextEncoding {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        match self {
            FileTypeTextEncoding::Utf8Ascii => write!(f, "UTF8/ASCII"),
            FileTypeTextEncoding::Utf16 => write!(f, "UTF16"),
            FileTypeTextEncoding::Utf32 => write!(f, "UTF32"),
        }
    }
}

/// a file's storage format
#[derive(Clone, Copy, Debug, Eq, PartialEq, Kinded)]
pub enum FileTypeArchive {
    /// An normal file, not explicitly compressed or archived
    Normal,
    /// a compressed bzip2 file, e.g. `log.bz2`
    ///
    /// Presumed to contain one regular file
    Bz2,
    /// a compressed gzipped file, e.g. `log.gz`
    ///
    /// Presumed to contain one regular file; see Issue #8
    Gz,
    /// a compressed LZMA4 file, e.g. `log.lz4`
    Lz4,
    /// a file within a `.tar` archive file
    Tar,
    /// a file compressed "xz'd" file, e.g. `log.xz`
    ///
    /// Presumed to contain one regular file; see Issue #11
    Xz,
}

impl std::fmt::Display for FileTypeArchive {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        match self {
            FileTypeArchive::Normal => write!(f, "Normal"),
            FileTypeArchive::Bz2 => write!(f, "BZIP2"),
            FileTypeArchive::Gz => write!(f, "GZIP"),
            FileTypeArchive::Lz4 => write!(f, "LZMA4"),
            FileTypeArchive::Tar => write!(f, "TAR"),
            FileTypeArchive::Xz => write!(f, "XZ"),
        }
    }
}

impl FileTypeArchive {
    /// Returns `true` if this is a `Tar`
    #[inline(always)]
    pub const fn is_tar(&self) -> bool {
        matches!(*self, FileTypeArchive::Tar)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Kinded)]
pub enum OdlSubType {
    /// suffix `.aodl`
    Aodl,
    /// suffix `.odl`
    Odl,
    /// suffix `.odlgz`
    ///
    /// This is not the same as a standard gzip-compressed file. It is gzipped by the Microsoft
    /// ODL writer. However the gzip data begins after the OneDrive Log header.
    Odlgz,
    /// suffix `.odlsent`
    Odlsent,
}

impl std::fmt::Display for OdlSubType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        match self {
            OdlSubType::Aodl => write!(f, "AODL"),
            OdlSubType::Odl => write!(f, "ODL"),
            OdlSubType::Odlgz => write!(f, "ODLGZ"),
            OdlSubType::Odlsent => write!(f, "ODLSENT"),
        }
    }
}

/// A file's major type to distinguish which _Reader_ struct should use it and
/// how the _Readers_ processes it.
// TODO: [2023/04] types Unset, Unparsable, Unknown are confusing to keep around
//       and make extra work for all match statements.
//       Can they be removed?
#[derive(Clone, Copy, Debug, Eq, PartialEq, Kinded)]
pub enum FileType {
    /// a Windows [Event Trace Log] file
    ///
    /// [Event Trace Log]: https://learn.microsoft.com/en-us/windows-hardware/test/wpt/opening-and-analyzing-etl-files-in-wpa
    Etl { archival_type: FileTypeArchive },
    /// a [Windows XML EventLog] file
    ///
    /// [Windows XML EventLog]: https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc
    Evtx { archival_type: FileTypeArchive },
    /// a binary acct/lastlog/lastlogx/utmp/umtpx format file
    FixedStruct {
        archival_type: FileTypeArchive,
        fixedstruct_type: FileTypeFixedStruct,
    },
    /// a [systemd Journal file]
    ///
    /// [systemd Journal file]: https://systemd.io/JOURNAL_FILE_FORMAT/
    Journal { archival_type: FileTypeArchive },
    Odl { archival_type: FileTypeArchive, odl_sub_type: OdlSubType },
    /// a plain vanilla file, e.g. `file.log`. Presumed to be a "syslog" file
    /// as the term is loosely used in this project.
    Text {
        archival_type: FileTypeArchive,
        encoding_type: FileTypeTextEncoding,
    },
    /// a file type known to be unparsable
    Unparsable,
}

impl std::fmt::Display for FileType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        match self {
            FileType::Etl{ .. } => write!(f, "ETL"),
            FileType::Evtx{ .. } => write!(f, "EVTX"),
            FileType::FixedStruct{ fixedstruct_type:FileTypeFixedStruct::Acct, .. } => write!(f, "ACCT"),
            FileType::FixedStruct{ fixedstruct_type:FileTypeFixedStruct::AcctV3, .. } => write!(f, "ACCT_V3"),
            FileType::FixedStruct{ fixedstruct_type:FileTypeFixedStruct::Lastlog, .. } => write!(f, "LASTLOG"),
            FileType::FixedStruct{ fixedstruct_type:FileTypeFixedStruct::Lastlogx, .. } => write!(f, "LASTLOGX"),
            FileType::FixedStruct{ fixedstruct_type:FileTypeFixedStruct::Utmp, .. } => write!(f, "UTMP/WTMP"),
            FileType::FixedStruct{ fixedstruct_type:FileTypeFixedStruct::Utmpx, .. } => write!(f, "UTMPX/WTMPX"),
            FileType::Journal{ .. } => write!(f, "JOURNAL"),
            FileType::Odl{ .. } => write!(f, "ODL"),
            FileType::Text{ .. } => write!(f, "TEXT"),
            FileType::Unparsable => write!(f, "UNPARSABLE"),
        }
    }
}

impl FileType {
    /// Returns `true` if this is a compressed file
    pub const fn is_compressed(&self) -> bool {
        match self {
            FileType::Etl{ archival_type: FileTypeArchive::Normal } => false,
            FileType::Etl{ archival_type: FileTypeArchive::Bz2 } => true,
            FileType::Etl{ archival_type: FileTypeArchive::Gz } => true,
            FileType::Etl{ archival_type: FileTypeArchive::Lz4 } => true,
            FileType::Etl{ archival_type: FileTypeArchive::Tar } => false,
            FileType::Etl{ archival_type: FileTypeArchive::Xz } => true,
            FileType::Evtx{ archival_type: FileTypeArchive::Normal } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Bz2 } => true,
            FileType::Evtx{ archival_type: FileTypeArchive::Gz } => true,
            FileType::Evtx{ archival_type: FileTypeArchive::Lz4 } => true,
            FileType::Evtx{ archival_type: FileTypeArchive::Tar } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Xz } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, .. } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Bz2, .. } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, .. } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, .. } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, .. } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, .. } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Normal } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Bz2 } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Gz } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Lz4 } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Tar } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Xz } => true,
            FileType::Odl{ archival_type: FileTypeArchive::Normal, .. } => false,
            FileType::Odl{ archival_type: FileTypeArchive::Bz2, .. } => true,
            FileType::Odl{ archival_type: FileTypeArchive::Gz, .. } => true,
            FileType::Odl{ archival_type: FileTypeArchive::Lz4, .. } => true,
            FileType::Odl{ archival_type: FileTypeArchive::Tar, .. } => true,
            FileType::Odl{ archival_type: FileTypeArchive::Xz, .. } => true,
            FileType::Text{ archival_type: FileTypeArchive::Normal, .. } => false,
            FileType::Text{ archival_type: FileTypeArchive::Bz2, .. } => true,
            FileType::Text{ archival_type: FileTypeArchive::Gz, .. } => true,
            FileType::Text{ archival_type: FileTypeArchive::Lz4, .. } => true,
            FileType::Text{ archival_type: FileTypeArchive::Tar, .. } => false,
            FileType::Text{ archival_type: FileTypeArchive::Xz, .. } => true,
            FileType::Unparsable => false,
        }
    }

    /// Returns `true` if the file is within an archived file
    pub const fn is_archived(&self) -> bool {
        match self {
            FileType::Etl{ archival_type: FileTypeArchive::Normal } => false,
            FileType::Etl{ archival_type: FileTypeArchive::Bz2 } => false,
            FileType::Etl{ archival_type: FileTypeArchive::Gz } => false,
            FileType::Etl{ archival_type: FileTypeArchive::Lz4 } => false,
            FileType::Etl{ archival_type: FileTypeArchive::Tar } => true,
            FileType::Etl{ archival_type: FileTypeArchive::Xz } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Normal } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Bz2 } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Gz } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Lz4 } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Tar } => true,
            FileType::Evtx{ archival_type: FileTypeArchive::Xz } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, .. } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Bz2, .. } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, .. } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, .. } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, .. } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, .. } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Normal } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Bz2 } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Gz } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Lz4 } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Tar } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Xz } => false,
            FileType::Odl{ archival_type: FileTypeArchive::Normal, .. } => false,
            FileType::Odl{ archival_type: FileTypeArchive::Bz2, .. } => false,
            FileType::Odl{ archival_type: FileTypeArchive::Gz, .. } => false,
            FileType::Odl{ archival_type: FileTypeArchive::Lz4, .. } => false,
            FileType::Odl{ archival_type: FileTypeArchive::Tar, .. } => true,
            FileType::Odl{ archival_type: FileTypeArchive::Xz, .. } => false,
            FileType::Text{ archival_type: FileTypeArchive::Normal, ..} => false,
            FileType::Text{ archival_type: FileTypeArchive::Bz2, .. } => false,
            FileType::Text{ archival_type: FileTypeArchive::Gz, .. } => false,
            FileType::Text{ archival_type: FileTypeArchive::Lz4, .. } => false,
            FileType::Text{ archival_type: FileTypeArchive::Tar, .. } => true,
            FileType::Text{ archival_type: FileTypeArchive::Xz, .. } => false,
            FileType::Unparsable => false,
        }
    }

    /// Returns `true` if the file is processable
    pub const fn is_supported(&self) -> bool {
        match self {
            FileType::Etl { .. } => true,
            FileType::Evtx { .. } => true,
            FileType::FixedStruct { .. } => true,
            FileType::Journal { .. } => true,
            FileType::Odl { .. } => true,
            FileType::Text { .. } => true,
            FileType::Unparsable => false,
        }
    }

    /// convert a `FileType` to it's corresponding `LogMessageType`
    pub const fn to_logmessagetype(&self) -> LogMessageType {
        match self {
            FileType::Etl { .. } => LogMessageType::PyEvent,
            FileType::Evtx { .. } => LogMessageType::Evtx,
            FileType::FixedStruct { .. } => LogMessageType::FixedStruct,
            FileType::Journal { .. } => LogMessageType::Journal,
            FileType::Odl { .. } => LogMessageType::PyEvent,
            FileType::Text { .. } => LogMessageType::Sysline,
            FileType::Unparsable => {
                debug_panic!("FileType::Unparsable should not be converted to LogMessageType");

                LogMessageType::All
            }
        }
    }

    /// convert a `FileType` to it's inner `FileTypeArchive`
    pub const fn to_filetypearchive(&self) -> FileTypeArchive {
        match self {
            FileType::Etl { archival_type } => *archival_type,
            FileType::Evtx { archival_type } => *archival_type,
            FileType::FixedStruct { archival_type, .. } => *archival_type,
            FileType::Journal { archival_type } => *archival_type,
            FileType::Odl { archival_type, .. } => *archival_type,
            FileType::Text { archival_type, .. } => *archival_type,
            FileType::Unparsable => {
                debug_panic!("FileType::Unparsable should not be converted to FileTypeArchive");

                FileTypeArchive::Normal
            }
        }
    }

    pub const fn is_etl(&self) -> bool {
        matches!(self, FileType::Etl { .. })
    }

    pub const fn is_evtx(&self) -> bool {
        matches!(self, FileType::Evtx { .. })
    }

    pub const fn is_fixedstruct(&self) -> bool {
        matches!(self, FileType::FixedStruct { .. })
    }

    pub const fn is_journal(&self) -> bool {
        matches!(self, FileType::Journal { .. })
    }

    pub const fn is_odl(&self) -> bool {
        matches!(self, FileType::Odl { .. })
    }

    pub const fn is_text(&self) -> bool {
        matches!(self, FileType::Text { .. })
    }

    pub const fn is_unparsable(&self) -> bool {
        matches!(self, FileType::Unparsable)
    }

    pub const fn archival_type(&self) -> FileTypeArchive {
        match self {
            FileType::Etl { archival_type } => *archival_type,
            FileType::Evtx { archival_type } => *archival_type,
            FileType::FixedStruct { archival_type, .. } => *archival_type,
            FileType::Journal { archival_type } => *archival_type,
            FileType::Odl { archival_type, .. } => *archival_type,
            FileType::Text { archival_type, .. } => *archival_type,
            FileType::Unparsable => FileTypeArchive::Normal,
        }
    }

    pub const fn encoding_type(&self) -> Option<FileTypeTextEncoding> {
        match self {
            FileType::Text { encoding_type, .. } => Some(*encoding_type),
            _ => None,
        }
    }

    pub const fn pretty_name(&self) -> &'static str {
        match self {
            FileType::Etl { .. } => "Windows Event Trace Log",
            FileType::Evtx { .. } => "Windows XML EventLog",
            FileType::FixedStruct { .. } => "Unix accounting log (acct/lastlog/lastlogx/utmp/utmpx)",
            FileType::Journal { .. } => "systemd Journal",
            FileType::Odl { .. } => "OneDrive Log",
            FileType::Text { .. } => "text log",
            FileType::Unparsable => "Unparsable",
        }
    }
}

/// The type of message sent from file processing thread to the main printing
/// thread. Similar to [`LogMessage`] but without the enclosed data.
///
/// [`LogMessage`]: crate::data::common::LogMessage
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LogMessageType {
    /// Typical line-oriented log file text message. Such a log file may be
    /// referred to as a "syslog file" in program parlance but that phrase is
    /// used loosely.
    ///
    /// Relates to a [`Sysline`].
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    Sysline,
    // / A binary [lastlog/lastlogx/utmp/utmpx format] file.
    // /
    // / Relates to a [`FixedStruct`].
    // /
    // / [lastlog/lastlogx/utmp/utmpx format]: https://web.archive.org/web/20231216015325/https://man.freebsd.org/cgi/man.cgi?query=lastlog&sektion=5&manpath=NetBSD+9.3
    // / [`FixedStruct`]: crate::data::fixedstruct::FixedStruct
    FixedStruct,
    /// A [Windows XML EventLog] file.
    ///
    /// [Windows XML EventLog]: https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc
    Evtx,
    /// A [systemd Journal file].
    ///
    /// Relates to a [`JournalEntry`].
    ///
    /// [systemd Journal file]: https://systemd.io/JOURNAL_FILE_FORMAT/
    /// [`JournalEntry`]: crate::data::journal::JournalEntry
    Journal,
    /// a Windows [Event Trace Log] file, or Windows OneDrive Log (ODL) file.
    ///
    /// [Event Trace Log]: https://learn.microsoft.com/en-us/windows-hardware/test/wpt/opening-and-analyzing-etl-files-in-wpa
    PyEvent,
    /// Special case, used to indicate "ALL" or "ANY" message type.
    /// Useful for code objects tracking multiple files.
    #[default]
    All,
}

impl std::fmt::Display for LogMessageType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        match self {
            LogMessageType::Evtx => write!(f, "EVTX entries (Windows XML EventLog)"),
            LogMessageType::FixedStruct => write!(f, "fixedstruct entries (Unix acct/lastlog/lastlogx/utmp/utmpx)"),
            LogMessageType::Journal => write!(f, "systemd journal entries"),
            LogMessageType::PyEvent => write!(f, "Python parsed events (ETL/ODL)"),
            LogMessageType::Sysline => write!(f, "text log lines"),
            LogMessageType::All => write!(f, "ALL"),
        }
    }
}

// ----------------------
// Blocks and BlockReader

/// Offset into a file in bytes. Zero-based.
pub type FileOffset = u64;

/// A [`Vec`](std::vec::Vec) of `u8`.
pub type Bytes = Vec<u8>;

/// Separator `char` symbol for a filesystem path and subpath within a
/// compressed file or an archive file. Used by an [`FPath`].
///
/// e.g. `path/logs.tar|logs/syslog`<br/>
/// e.g. `log.xz|syslog`
///
/// [`FPath`]: crate::common::FPath
pub const SUBPATH_SEP: char = '\0';
/// when printing a subpath separator, use this character
pub const SUBPATH_SEP_DISPLAY: char = '|';
/// `SUBPATH_SEP_DISPLAY` as a `str`
pub const SUBPATH_SEP_DISPLAY_STR: &str = "|";

// --------------------
// Lines and LineReader

/// Minimum size of characters in bytes.
/// UTF-8 would be value `1`, UTF-16 would be value `2`, etc.
pub type CharSz = usize;
/// *N*ew*L*ine as a [`char`].
#[allow(non_upper_case_globals)]
pub const NLc: char = '\n';
/// *N*ew*L*ine as a [`str`].
#[allow(non_upper_case_globals)]
pub const NLs: &str = "\n";
/// Single-byte *N*ew*L*ine `char` as [`u8`].
#[allow(non_upper_case_globals)]
pub const NLu8: u8 = 10;
/// *N*ew*L*ine in a byte buffer.
// XXX: Issue #16 only handles UTF-8/ASCII encoding
#[allow(non_upper_case_globals)]
pub const NLu8a: [u8; 1] = [NLu8];

/// Maximum size of a syslog message, 8096 octets (0x1FA0).
///
/// According to <https://stackoverflow.com/a/41822232/471376>.
pub const SYSLOG_SZ_MAX: usize = 8096;

/// Create a `Error` with an error string that includes the file path.
pub fn err_from_err_path(
    error: &Error,
    fpath: &FPath,
    mesg: Option<&str>,
) -> Error {
    match mesg {
        Some(s) => Error::new(error.kind(), format!("{} {} file {:?}", error, s, fpath)),
        None => Error::new(error.kind(), format!("{} file {:?}", error, fpath)),
    }
}

/// Helper to `BlockReadeer::new`; create a `Result::Err` with an error
/// string that includes the file path
pub fn err_from_err_path_result<T>(
    error: &Error,
    fpath: &FPath,
    mesg: Option<&str>,
) -> Result<T> {
    Result::Err(err_from_err_path(error, fpath, mesg))
}

/// convert a `thread::ThreadId` to `u64`
// TRACKING: Tracking Issue for `ThreadId` to u64 conversion <https://github.com/rust-lang/rust/issues/67939>
pub fn threadid_to_u64(tid: thread::ThreadId) -> u64 {
    let tid_u64: u64;
    unsafe {
        tid_u64 = std::mem::transmute::<thread::ThreadId, u64>(tid);
    }

    tid_u64
}

// TRACKING: Tracking Issue for comparing values in const items <https://github.com/rust-lang/rust/issues/92391>
//pub const MAX_SZ: usize = core::cmp::max(linux_x86::UTMPX_SZ, openbsd_x86::UTMPX_SZ);

// TRACKING: Tracking Issue for comparing Trait objects in const context <https://github.com/rust-lang/rust/issues/67792>
//           this would allow making the `max` and `min` functions into supporting all numbers
//           and other comparables.
//           i.e.
//                 pub const max2(a: T, b: T) -> T
//                 where T: ~const PartialOrd, ~const Ord

/// local `const` helper to return the maximum of two `usize` values
///
/// Credit to <https://stackoverflow.com/a/53646925/471376>
#[allow(clippy::too_many_arguments)]
pub const fn max2(
    a: usize,
    b: usize,
) -> usize {
    [a, b][(a < b) as usize]
}

#[allow(clippy::too_many_arguments)]
pub const fn max3(
    a: usize,
    b: usize,
    c: usize,
) -> usize {
    max2(max2(a, b), c)
}

#[allow(clippy::too_many_arguments)]
pub const fn max4(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
) -> usize {
    max2(max3(a, b, c), d)
}

#[allow(clippy::too_many_arguments)]
pub const fn max5(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
) -> usize {
    max2(max4(a, b, c, d), e)
}

#[allow(clippy::too_many_arguments)]
pub const fn max6(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
) -> usize {
    max2(max5(a, b, c, d, e), f)
}

#[allow(clippy::too_many_arguments)]
pub const fn max7(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
) -> usize {
    max2(max6(a, b, c, d, e, f), g)
}

#[allow(clippy::too_many_arguments)]
pub const fn max8(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
) -> usize {
    max2(max7(a, b, c, d, e, f, g), h)
}

#[allow(clippy::too_many_arguments)]
pub const fn max9(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
) -> usize {
    max2(max8(a, b, c, d, e, f, g, h), i)
}

#[allow(clippy::too_many_arguments)]
pub const fn max10(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
) -> usize {
    max2(max9(a, b, c, d, e, f, g, h, i), j)
}

#[allow(clippy::too_many_arguments)]
pub const fn max11(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
    k: usize,
) -> usize {
    max2(max10(a, b, c, d, e, f, g, h, i, j), k)
}

#[allow(clippy::too_many_arguments)]
pub const fn max12(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
    k: usize,
    l: usize,
) -> usize {
    max2(max11(a, b, c, d, e, f, g, h, i, j, k), l)
}

#[allow(clippy::too_many_arguments)]
pub const fn max13(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
    k: usize,
    l: usize,
    m: usize,
) -> usize {
    max2(max12(a, b, c, d, e, f, g, h, i, j, k, l), m)
}

#[allow(clippy::too_many_arguments)]
pub const fn max14(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
    k: usize,
    l: usize,
    m: usize,
    n: usize,
) -> usize {
    max2(max13(a, b, c, d, e, f, g, h, i, j, k, l, m), n)
}

#[allow(clippy::too_many_arguments)]
pub const fn max15(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
    k: usize,
    l: usize,
    m: usize,
    n: usize,
    o: usize,
) -> usize {
    max2(max14(a, b, c, d, e, f, g, h, i, j, k, l, m, n), o)
}

#[allow(clippy::too_many_arguments)]
pub const fn max16(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
    k: usize,
    l: usize,
    m: usize,
    n: usize,
    o: usize,
    p: usize,
) -> usize {
    max2(max15(a, b, c, d, e, f, g, h, i, j, k, l, m, n, o), p)
}

/// local `const` helper to return the minimum of two `usize` values
///
/// Credit to <https://stackoverflow.com/a/53646925/471376>
#[allow(clippy::too_many_arguments)]
pub const fn min2(
    a: usize,
    b: usize,
) -> usize {
    [a, b][(a > b) as usize]
}

#[allow(clippy::too_many_arguments)]
pub const fn min3(
    a: usize,
    b: usize,
    c: usize,
) -> usize {
    min2(min2(a, b), c)
}

#[allow(clippy::too_many_arguments)]
pub const fn min4(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
) -> usize {
    min2(min3(a, b, c), d)
}

#[allow(clippy::too_many_arguments)]
pub const fn min5(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
) -> usize {
    min2(min4(a, b, c, d), e)
}

#[allow(clippy::too_many_arguments)]
pub const fn min6(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
) -> usize {
    min2(min5(a, b, c, d, e), f)
}

#[allow(clippy::too_many_arguments)]
pub const fn min7(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
) -> usize {
    min2(min6(a, b, c, d, e, f), g)
}

#[allow(clippy::too_many_arguments)]
pub const fn min8(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
) -> usize {
    min2(min7(a, b, c, d, e, f, g), h)
}

#[allow(clippy::too_many_arguments)]
pub const fn min9(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
) -> usize {
    min2(min8(a, b, c, d, e, f, g, h), i)
}

#[allow(clippy::too_many_arguments)]
pub const fn min10(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
) -> usize {
    min2(min9(a, b, c, d, e, f, g, h, i), j)
}

#[allow(clippy::too_many_arguments)]
pub const fn min11(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
    k: usize,
) -> usize {
    min2(min10(a, b, c, d, e, f, g, h, i, j), k)
}

#[allow(clippy::too_many_arguments)]
pub const fn min12(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
    k: usize,
    l: usize,
) -> usize {
    min2(min11(a, b, c, d, e, f, g, h, i, j, k), l)
}

#[allow(clippy::too_many_arguments)]
pub const fn min13(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
    k: usize,
    l: usize,
    m: usize,
) -> usize {
    min2(min12(a, b, c, d, e, f, g, h, i, j, k, l), m)
}

#[allow(clippy::too_many_arguments)]
pub const fn min14(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
    k: usize,
    l: usize,
    m: usize,
    n: usize,
) -> usize {
    min2(min13(a, b, c, d, e, f, g, h, i, j, k, l, m), n)
}

#[allow(clippy::too_many_arguments)]
pub const fn min15(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
    k: usize,
    l: usize,
    m: usize,
    n: usize,
    o: usize,
) -> usize {
    min2(min14(a, b, c, d, e, f, g, h, i, j, k, l, m, n), o)
}

#[allow(clippy::too_many_arguments)]
pub const fn min16(
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
    g: usize,
    h: usize,
    i: usize,
    j: usize,
    k: usize,
    l: usize,
    m: usize,
    n: usize,
    o: usize,
    p: usize,
) -> usize {
    min2(min15(a, b, c, d, e, f, g, h, i, j, k, l, m, n, o), p)
}
