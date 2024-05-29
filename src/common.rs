// src/common.rs

//! Common imports, type aliases, and other globals for _s4lib_.

use std::collections::HashSet;
#[doc(hidden)]
pub use std::fs::File;
use std::io::{Error, Result};
use std::fmt::Debug;
#[doc(hidden)]
pub use std::path::Path;

use ::chrono::FixedOffset;
use ::kinded::Kinded;
use ::lazy_static::lazy_static;

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

lazy_static! {
    pub static ref FIXEDOFFSET0: FixedOffset = FixedOffset::east_opt(0).unwrap();
}

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
// TODO: rename from `ResultS3` to `ResultFind`
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultS3<T, E> {
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

impl<T, E> ResultS3<T, E> {
    // Querying the contained values

    /// Returns `true` if the result is [`Found`], [`Done`].
    ///
    /// [`Found`]: self::ResultS3#variant.Found
    /// [`Done`]: self::ResultS3#variant.Done
    /// [`Err`]: self::ResultS3#variant.Err
    #[allow(dead_code)]
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline(always)]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, ResultS3::Found(_) | ResultS3::Done)
    }

    /// Returns `true` if the result is [`Found`].
    ///
    /// [`Found`]: self::ResultS3#variant.Found
    #[inline(always)]
    pub const fn is_found(&self) -> bool {
        matches!(*self, ResultS3::Found(_))
    }

    /// Returns `true` if the result is [`Err`].
    ///
    /// [`Err`]: self::ResultS3#variant.Err
    #[allow(dead_code)]
    #[must_use = "if you intended to assert that this is err, consider `.unwrae_err()` instead"]
    #[inline(always)]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Returns `true` if the result is [`Done`].
    ///
    /// [`Done`]: self::ResultS3#variant.Done
    #[inline(always)]
    pub const fn is_done(&self) -> bool {
        matches!(*self, ResultS3::Done)
    }

    /// Returns `true` if the result is an [`Found`] value containing the given
    /// value.
    ///
    /// [`Found`]: self::ResultS3#variant.Found
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
            ResultS3::Found(y) => x == y,
            ResultS3::Done => false,
            ResultS3::Err(_) => false,
        }
    }

    /// Returns `true` if the result is an [`Err`] value containing the given
    /// value.
    ///
    /// [`Err`]: self::ResultS3#variant.Err
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
            ResultS3::Err(e) => f == e,
            _ => false,
        }
    }

    // Adapter for each variant

    /// Converts from `ResultS3<T, E>` to [`Option<T>`].
    ///
    /// Converts `self` into an [`Option<T>`], consuming `self`,
    /// and discarding the error, if any.
    #[allow(dead_code)]
    #[inline(always)]
    pub fn ok(self) -> Option<T> {
        match self {
            ResultS3::Found(x) => Some(x),
            ResultS3::Done => None,
            ResultS3::Err(_) => None,
        }
    }

    /// Converts from `ResultS3<T, E>` to [`Option<E>`].
    ///
    /// Converts `self` into an [`Option<E>`], consuming `self`,
    /// and discarding the success value, if any.
    #[allow(dead_code)]
    #[inline(always)]
    pub fn err(self) -> Option<E> {
        match self {
            ResultS3::Found(_) => None,
            ResultS3::Done => None,
            ResultS3::Err(x) => Some(x),
        }
    }
}

impl<T, E> std::fmt::Display for ResultS3<T, E>
where
    E: std::fmt::Display,
{
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ResultS3::Found(_) => {
                write!(f, "ResultS3::Found")
            }
            ResultS3::Done => {
                write!(f, "ResultS3::Done")
            }
            ResultS3::Err(err) => {
                write!(f, "ResultS3::Err({})", err)
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
            _ => None
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
    /// a compressed gzipped file, e.g. `log.gz`
    ///
    /// Presumed to contain one regular file; see Issue #8
    Gz,
    /// a compressed LZIP file, e.g. `log.lzma`
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
            FileTypeArchive::Gz => write!(f, "gzip"),
            FileTypeArchive::Lz4 => write!(f, "lz4"),
            FileTypeArchive::Tar => write!(f, "tar"),
            FileTypeArchive::Xz => write!(f, "xz"),
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

/// A file's major type to distinguish which _Reader_ struct should use it and
/// how the _Readers_ processes it.
// TODO: [2023/04] types Unset, Unparsable, Unknown are confusing to keep around
//       and make extra work for all match statements.
//       Can they be removed?
#[derive(Clone, Copy, Debug, Eq, PartialEq, Kinded)]
pub enum FileType {
    /// a [Windows XML EventLog] file
    ///
    /// [Windows XML EventLog]: https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc
    Evtx { archival_type: FileTypeArchive },
    /// a binary acct/lastlog/lastlogx/utmp/umtpx format file
    FixedStruct { archival_type: FileTypeArchive, fixedstruct_type: FileTypeFixedStruct },
    /// a [systemd Journal file]
    ///
    /// [systemd Journal file]: https://systemd.io/JOURNAL_FILE_FORMAT/
    Journal { archival_type: FileTypeArchive },
    /// a plain vanilla file, e.g. `file.log`. Presumed to be a "syslog" file
    /// as the term is loosely used in this project.
    Text { archival_type: FileTypeArchive, encoding_type: FileTypeTextEncoding },
    /// a file type known to be unparsable
    Unparsable,
    // #[default]
    // Unset
}

/*

For copy+pasta convenience:

    FileType::Evtx{ archival_type: FileTypeArchive::Normal } => false,
    FileType::Evtx{ archival_type: FileTypeArchive::Gz } => false,
    FileType::Evtx{ archival_type: FileTypeArchive::Tar } => false,
    FileType::Evtx{ archival_type: FileTypeArchive::Xz } => false,
    FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, fixedstruct_type: _ } => false,
    FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, fixedstruct_type: _ } => false,
    FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, fixedstruct_type: _ } => false,
    FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, fixedstruct_type: _ } => false,
    FileType::Journal{ archival_type: FileTypeArchive::Normal } => false,
    FileType::Journal{ archival_type: FileTypeArchive::Gz } => false,
    FileType::Journal{ archival_type: FileTypeArchive::Tar } => false,
    FileType::Journal{ archival_type: FileTypeArchive::Xz } => false,
    FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: FileTypeTextEncoding::Utf8Ascii } => false,
    FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: FileTypeTextEncoding::Utf16 } => false,
    FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: FileTypeTextEncoding::Utf32 } => false,
    FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: FileTypeTextEncoding::Utf8Ascii } => false,
    FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: FileTypeTextEncoding::Utf16 } => false,
    FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: FileTypeTextEncoding::Utf32 } => false,
    FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: FileTypeTextEncoding::Utf8Ascii } => false,
    FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: FileTypeTextEncoding::Utf16 } => false,
    FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: FileTypeTextEncoding::Utf32 } => false,
    FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: FileTypeTextEncoding::Utf8Ascii } => false,
    FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: FileTypeTextEncoding::Utf16 } => false,
    FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: FileTypeTextEncoding::Utf32 } => false,
    FileType::Unknown => false,
    FileType::Unparsable => false,
    FileType::Unset => false,

    FileType::Evtx{ archival_type: FileTypeArchive::Normal }
    | FileType::Evtx{ archival_type: FileTypeArchive::Gz }
    | FileType::Evtx{ archival_type: FileTypeArchive::Tar }
    | FileType::Evtx{ archival_type: FileTypeArchive::Xz }
    | FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, fixedstruct_type: _ }
    | FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, fixedstruct_type: _ }
    | FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, fixedstruct_type: _ }
    | FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, fixedstruct_type: _ }
    | FileType::Journal{ archival_type: FileTypeArchive::Normal }
    | FileType::Journal{ archival_type: FileTypeArchive::Gz }
    | FileType::Journal{ archival_type: FileTypeArchive::Tar }
    | FileType::Journal{ archival_type: FileTypeArchive::Xz }
    | FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: FileTypeTextEncoding::Utf8Ascii }
    | FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: FileTypeTextEncoding::Utf16 }
    | FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: FileTypeTextEncoding::Utf32 }
    | FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: FileTypeTextEncoding::Utf8Ascii }
    | FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: FileTypeTextEncoding::Utf16 }
    | FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: FileTypeTextEncoding::Utf32 }
    | FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: FileTypeTextEncoding::Utf8Ascii }
    | FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: FileTypeTextEncoding::Utf16 }
    | FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: FileTypeTextEncoding::Utf32 }
    | FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: FileTypeTextEncoding::Utf8Ascii }
    | FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: FileTypeTextEncoding::Utf16 }
    | FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: FileTypeTextEncoding::Utf32 }
    | FileType::Unknown
    | FileType::Unparsable
    | FileType::Unset

    FileType::Evtx{ archival_type: FileTypeArchive::Normal }
    | FileType::Evtx{ archival_type: FileTypeArchive::Gz }
    | FileType::Evtx{ archival_type: FileTypeArchive::Tar }
    | FileType::Evtx{ archival_type: FileTypeArchive::Xz }
    | FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, fixedstruct_type: _ }
    | FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, fixedstruct_type: _ }
    | FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, fixedstruct_type: _ }
    | FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, fixedstruct_type: _ }
    | FileType::Journal{ archival_type: FileTypeArchive::Normal }
    | FileType::Journal{ archival_type: FileTypeArchive::Gz }
    | FileType::Journal{ archival_type: FileTypeArchive::Tar }
    | FileType::Journal{ archival_type: FileTypeArchive::Xz }
    | FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: _ }
    | FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: _ }
    | FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: _ }
    | FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: _ }
    | FileType::Unknown
    | FileType::Unparsable
    | FileType::Unset
*/

impl std::fmt::Display for FileType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        match self {
            FileType::Evtx{ .. } => write!(f, "EVTX"),
            FileType::FixedStruct{ fixedstruct_type:FileTypeFixedStruct::Acct, .. } => write!(f, "ACCT"),
            FileType::FixedStruct{ fixedstruct_type:FileTypeFixedStruct::AcctV3, .. } => write!(f, "ACCT_V3"),
            FileType::FixedStruct{ fixedstruct_type:FileTypeFixedStruct::Lastlog, .. } => write!(f, "LASTLOG"),
            FileType::FixedStruct{ fixedstruct_type:FileTypeFixedStruct::Lastlogx, .. } => write!(f, "LASTLOGX"),
            FileType::FixedStruct{ fixedstruct_type:FileTypeFixedStruct::Utmp, .. } => write!(f, "UTMP/WTMP"),
            FileType::FixedStruct{ fixedstruct_type:FileTypeFixedStruct::Utmpx, .. } => write!(f, "UTMPX/WTMPX"),
            FileType::Journal{ .. } => write!(f, "JOURNAL"),
            FileType::Text{ .. } => write!(f, "TEXT"),
            FileType::Unparsable => write!(f, "UNPARSABLE"),
        }
    }
}

impl FileType {
    /// Returns `true` if this is a compressed file
    pub const fn is_compressed(&self) -> bool {
        match self {
            FileType::Evtx{ archival_type: FileTypeArchive::Normal } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Gz } => true,
            FileType::Evtx{ archival_type: FileTypeArchive::Lz4 } => true,
            FileType::Evtx{ archival_type: FileTypeArchive::Tar } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Xz } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, .. } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, .. } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, .. } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, .. } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, .. } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Normal } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Gz } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Lz4 } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Tar } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Xz } => true,
            FileType::Text{ archival_type: FileTypeArchive::Normal, .. } => false,
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
            FileType::Evtx{ archival_type: FileTypeArchive::Normal } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Gz } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Lz4 } => false,
            FileType::Evtx{ archival_type: FileTypeArchive::Tar } => true,
            FileType::Evtx{ archival_type: FileTypeArchive::Xz } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, .. } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, .. } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, .. } => false,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, .. } => true,
            FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, .. } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Normal } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Gz } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Lz4 } => false,
            FileType::Journal{ archival_type: FileTypeArchive::Tar } => true,
            FileType::Journal{ archival_type: FileTypeArchive::Xz } => false,
            FileType::Text{ archival_type: FileTypeArchive::Normal, ..} => false,
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
            FileType::Evtx{ .. } => true,
            FileType::FixedStruct{ .. } => true,
            FileType::Journal{ .. } => true,
            FileType::Text{ .. } => true,
            FileType::Unparsable => false,
        }
    }

    /// convert a `FileType` to it's corresponding `LogMessageType`
    pub const fn to_logmessagetype(&self) -> LogMessageType {
        match self {
            FileType::Evtx{ .. } => LogMessageType::Evtx,
            FileType::FixedStruct{ .. } => LogMessageType::FixedStruct,
            FileType::Journal{ .. } => LogMessageType::Journal,
            FileType::Text{ .. } => LogMessageType::Sysline,
            FileType::Unparsable => {
                debug_panic!("FileType::Unparsable should not be converted to LogMessageType");

                LogMessageType::All
            },
        }
    }

    /// convert a `FileType` to it's inner `FileTypeArchive`
    pub const fn to_filetypearchive(&self) -> FileTypeArchive {
        match self {
            FileType::Evtx{ archival_type } => *archival_type,
            FileType::FixedStruct{ archival_type, .. } => *archival_type,
            FileType::Journal{ archival_type } => *archival_type,
            FileType::Text{ archival_type, .. } => *archival_type,
            FileType::Unparsable => {
                debug_panic!("FileType::Unparsable should not be converted to FileTypeArchive");

                FileTypeArchive::Normal
            },
        }
    }

    pub const fn is_evtx (&self) -> bool {
        match self {
            FileType::Evtx{ .. } => true,
            _ => false,
        }
    }

    pub const fn is_fixedstruct (&self) -> bool {
        match self {
            FileType::FixedStruct{ .. } => true,
            _ => false,
        }
    }

    pub const fn is_journal (&self) -> bool {
        match self {
            FileType::Journal{ .. } => true,
            _ => false,
        }
    }

    pub const fn is_text (&self) -> bool {
        match self {
            FileType::Text{ .. } => true,
            _ => false,
        }
    }

    pub const fn is_unparsable (&self) -> bool {
        match self {
            FileType::Unparsable => true,
            _ => false,
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
            LogMessageType::Sysline => write!(f, "syslog lines"),
            LogMessageType::FixedStruct => write!(f, "fixedstruct entries (acct/lastlog/lastlogx/utmp/utmpx)"),
            LogMessageType::Evtx => write!(f, "evtx entries"),
            LogMessageType::Journal => write!(f, "journal entries"),
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
pub fn err_from_err_path(error: &Error, fpath: &FPath, mesg: Option<&str>) -> Error
{
    match mesg {
        Some(s) => Error::new(
            error.kind(),
            format!(
                "{} {} file {:?}", error, s, fpath
            )
        ),
        None => Error::new(
            error.kind(),
            format!(
                "{} file {:?}", error, fpath
            )
        )
    }
}

/// Helper to `BlockReadeer::new`; create a `Result::Err` with an error
/// string that includes the file path
pub fn err_from_err_path_result<T>(error: &Error, fpath: &FPath, mesg: Option<&str>) -> Result<T>
{
    Result::Err(err_from_err_path(error, fpath, mesg))
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
pub const fn max2(a: usize, b: usize) -> usize
{
    [a, b][(a < b) as usize]
}

#[allow(clippy::too_many_arguments)]
pub const fn max3(a: usize, b: usize, c: usize) -> usize {
    max2(max2(a, b), c)
}

#[allow(clippy::too_many_arguments)]
pub const fn max4(a: usize, b: usize, c: usize, d: usize) -> usize {
    max2(max3(a, b, c), d)
}

#[allow(clippy::too_many_arguments)]
pub const fn max5(a: usize, b: usize, c: usize, d: usize, e: usize) -> usize {
    max2(max4(a, b, c, d), e)
}

#[allow(clippy::too_many_arguments)]
pub const fn max6(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    max2(max5(a, b, c, d, e), f)
}

#[allow(clippy::too_many_arguments)]
pub const fn max7(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize, g: usize) -> usize {
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
pub const fn min2(a: usize, b: usize) -> usize {
    [a, b][(a > b) as usize]
}

#[allow(clippy::too_many_arguments)]
pub const fn min3(a: usize, b: usize, c: usize) -> usize {
    min2(min2(a, b), c)
}

#[allow(clippy::too_many_arguments)]
pub const fn min4(a: usize, b: usize, c: usize, d: usize) -> usize {
    min2(min3(a, b, c), d)
}

#[allow(clippy::too_many_arguments)]
pub const fn min5(a: usize, b: usize, c: usize, d: usize, e: usize) -> usize {
    min2(min4(a, b, c, d), e)
}

#[allow(clippy::too_many_arguments)]
pub const fn min6(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    min2(min5(a, b, c, d, e), f)
}

#[allow(clippy::too_many_arguments)]
pub const fn min7(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize, g: usize) -> usize {
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
    h: usize
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
