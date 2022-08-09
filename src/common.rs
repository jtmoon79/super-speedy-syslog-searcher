// src/common.rs
//
// common imports, type aliases, and other globals (avoids circular imports)

pub use std::fs::File;
use std::fmt::Debug;
pub use std::path::Path;

/// `F`ake `Path` or `F`ile `Path`
//
// XXX: ideal would be using `std::path::Path`, but that does not have trait `Sized` which means
//      instances must be passed-by-reference ("size is not known at compile time"). This
//      introduces too much difficulty (have to start marking lifetimes everywhere, no way!)
//      Use this type alias as a stand-in.
pub type FPath = String;
/// a sequence of `FPath`
pub type FPaths = Vec::<FPath>;
pub type FileMetadata = std::fs::Metadata;
pub type FileOpenOptions = std::fs::OpenOptions;
/// File Size in bytes
pub type FileSz = u64;
// general purpose counting variables
pub type Count = u64;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// custom Results enums for various *Reader functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#481-495

/// `Result` extended for `s4` to 3 types
///
/// for line searching and sysline searching functions
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResultS4<T, E> {
    /// Contains the success data
    Found(T),
    /// File is empty, or other condition that means "Done", nothing to return, but no bad errors happened
    Done,
    /// Contains the error value, something bad happened
    Err(E),
}

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#501-659

impl<T, E> ResultS4<T, E> {
    // Querying the contained values

    /// Returns `true` if the result is [`Found`, 'Done`].
    #[allow(dead_code)]
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline(always)]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, ResultS4::Found(_) | ResultS4::Done)
    }

    /// Returns `true` if the result is [`Found`].
    #[inline(always)]
    pub const fn is_found(&self) -> bool {
        matches!(*self, ResultS4::Found(_))
    }

    /// Returns `true` if the result is [`Err`].
    #[allow(dead_code)]
    #[must_use = "if you intended to assert that this is err, consider `.unwrap_err()` instead"]
    #[inline(always)]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Returns `true` if the result is [`Done`].
    #[inline(always)]
    pub const fn is_done(&self) -> bool {
        matches!(*self, ResultS4::Done)
    }

    /// Returns `true` if the result is an [`Found`] value containing the given value.
    #[allow(dead_code)]
    #[must_use]
    #[inline(always)]
    pub fn contains<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            ResultS4::Found(y) => x == y,
            ResultS4::Done => false,
            ResultS4::Err(_) => false,
        }
    }

    /// Returns `true` if the result is an [`Err`] value containing the given value.
    #[allow(dead_code)]
    #[must_use]
    #[inline(always)]
    pub fn contains_err<F>(&self, f: &F) -> bool
    where
        F: PartialEq<E>,
    {
        match self {
            ResultS4::Err(e) => f == e,
            _ => false,
        }
    }

    // Adapter for each variant

    /// Converts from `ResultS4<T, E>` to [`Option<T>`].
    ///
    /// Converts `self` into an [`Option<T>`], consuming `self`,
    /// and discarding the error, if any.
    #[allow(dead_code)]
    #[inline(always)]
    pub fn ok(self) -> Option<T> {
        match self {
            ResultS4::Found(x) => Some(x),
            ResultS4::Done => None,
            ResultS4::Err(_) => None,
        }
    }

    /// Converts from `ResultS4<T, E>` to [`Option<E>`].
    ///
    /// Converts `self` into an [`Option<E>`], consuming `self`,
    /// and discarding the success value, if any.
    #[allow(dead_code)]
    #[inline(always)]
    pub fn err(self) -> Option<E> {
        match self {
            ResultS4::Found(_) => None,
            ResultS4::Done => None,
            ResultS4::Err(x) => Some(x),
        }
    }
}

impl<T, E> std::fmt::Display for ResultS4<T, E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResultS4::Found(_) => { write!(f, "ResultS4::Found") },
            ResultS4::Done => { write!(f, "ResultS4::Done") },
            ResultS4::Err(err) => { write!(f, "ResultS4::Err({})", err) },
        }
    }
}

// TODO: [2022/08] `ResultS4` was refactored. It can be merged with `ResultS3`.
//       No need for both.

/// `Result` extended to 3 types
///
/// for block searching functions
#[derive(Debug, PartialEq)]
pub enum ResultS3<T, E> {
    /// Contains the success data
    Found(T),
    /// File is empty, or other condition that means "Done", nothing to return, but no bad errors happened
    Done,
    /// Contains the error value, something bad happened
    Err(E),
}

impl<T, E> ResultS3<T, E> {
    // Querying the contained values

    /// Returns `true` if the result is [`Found`, 'Done`].
    #[allow(dead_code)]
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline(always)]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, ResultS3::Found(_) | ResultS3::Done)
    }

    /// Returns `true` if the result is [`Err`].
    #[allow(dead_code)]
    #[must_use = "if you intended to assert that this is err, consider `.unwrap_err()` instead"]
    #[inline(always)]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Returns `true` if the result is [`Done`].
    #[inline(always)]
    pub const fn is_done(&self) -> bool {
        matches!(*self, ResultS3::Done)
    }

    /// Returns `true` if the result is [`Found`].
    #[inline(always)]
    pub const fn is_found(&self) -> bool {
        matches!(*self, ResultS3::Found(_))
    }

    /// Returns `true` if the result is an [`Found`] value containing the given value.
    #[allow(dead_code)]
    #[must_use]
    #[inline(always)]
    pub fn contains<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            ResultS3::Found(y) => x == y,
            ResultS3::Done => false,
            ResultS3::Err(_) => false,
        }
    }

    /// Returns `true` if the result is an [`Err`] value containing the given value.
    #[allow(dead_code)]
    #[must_use]
    #[inline(always)]
    pub fn contains_err<F>(&self, f: &F) -> bool
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

    /// Converts from `Result<T, E>` to [`Option<E>`].
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResultS3::Found(_) => { write!(f, "ResultS3::Found") },
            ResultS3::Done => { write!(f, "ResultS3::Done") },
            ResultS3::Err(err) => { write!(f, "ResultS3::Err({})", err) },
        }
    }
}

/// results given by the filepreprocessor function(s)
#[derive(Debug)]
pub enum FileProcessingResult<E> {
    FileErrEmpty,
    FileErrNoLinesFound,
    FileErrNoSyslinesFound,
    FileErrNoSyslinesInDtRange,
    FileErrIo(E),
    FileErrWrongType,
    FileErrDecompress,
    /// TODO: [2022/08] stub value, redesign bin.rs data passing channels to pass the actual
    ///       error, not via Summary._Error
    FileErrStub,
    FileOk,
}

impl<E> FileProcessingResult<E> {
    /// Returns `true` if the result is [`FileOk`].
    #[inline(always)]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, FileProcessingResult::FileOk)
    }

    /// Returns `true` if the result is [`FileErrIo`].
    #[inline(always)]
    pub const fn has_err(&self) -> bool {
        matches!(*self, FileProcessingResult::FileErrIo(_))
    }
}

/// manually implement `PartialEq` as `#[derive(PartialEq)]` does not seem to work
impl<E> PartialEq for FileProcessingResult<E> {
    fn eq(&self, other: &Self) -> bool {
        if self.has_err() && other.has_err() {
            return true;
        }
        matches!(self, _other)
    }
}
impl<E> Eq for FileProcessingResult<E> {}

/// file types that can be processed by `SyslogProcessor` (and underlying modules)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileType {
    FileUnset,
    /// a regular file `file.log`
    File,
    /// a gzipped file `.gz`, presumed to contain one regular file
    FileGz,
    /// a regular file within a `.tar` file
    FileTar,
    FileTarGz,
    /// a xz'd file `.xz`, presumed to contain one regular file
    FileXz,
    /// unknown
    FileUnknown,
}

// XXX: Deriving `Default` on enums is experimental.
//      See issue #86985 <https://github.com/rust-lang/rust/issues/86985>
//      When `Default` is integrated then this `impl Default` can be removed.
//      Issue #18
impl Default for FileType {
    fn default() -> Self { FileType::FileUnset }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FileType::FileUnset => write!(f, "UNSET"),
            FileType::File => write!(f, "TEXT"),
            FileType::FileGz => write!(f, "GZIP"),
            FileType::FileTar => write!(f, "TAR"),
            FileType::FileTarGz => write!(f, "TAR GZIP"),
            FileType::FileXz => write!(f, "XZ"),
            FileType::FileUnknown => write!(f, "UNKNOWN"),
        }
    }
}

impl FileType {
    /// Returns `true` if this is a compressed file
    #[inline(always)]
    pub const fn is_compressed(&self) -> bool {
        matches!(*self,
            FileType::FileGz | FileType::FileTarGz | FileType::FileXz
        )
    }

    /// Returns `true` if the file is within an archived file
    #[inline(always)]
    pub const fn is_archived(&self) -> bool {
        matches!(*self,
            FileType::FileTar | FileType::FileTarGz
        )
    }

    /// Returns the tarred version of the `FileType`
    /// XXX: only supports `FileType::File` right now
    /// Relates to Issue #7
    /// Relates to Issue #14
    pub const fn to_tar(&self) -> FileType {
        if matches!(*self, FileType::File) {
            return FileType::FileTar;
        }

        FileType::FileUnknown
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Blocks and BlockReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Offset into a file in bytes
///
/// zero-based
pub type FileOffset = u64;

/// A `Vec` of `u8`
pub type Bytes = Vec<u8>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Lines and LineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// minimum size of characters of file in bytes
/// UTF-8 would be value `1`, UTF-16 would be value `2`, etc.
pub type CharSz = usize;
/// NewLine as char
#[allow(non_upper_case_globals)]
pub const NLc: char = '\n';
/// Single-byte newLine char as u8
#[allow(non_upper_case_globals)]
pub const NLu8: u8 = 10;
/// Newline in a byte buffer
// XXX: Issue #16 only handles UTF-8/ASCII encoding
#[allow(non_upper_case_globals)]
pub const NLu8a: [u8; 1] = [NLu8];

/// maximum size of a syslog message, 8096 octets (0x1FA0)
///
/// taken from https://stackoverflow.com/a/41822232/471376
pub const SYSLOG_SZ_MAX: usize = 8096;
