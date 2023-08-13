// src/common.rs

//! Common imports, type aliases, and other globals for _s4lib_.

#[doc(hidden)]
pub use std::fs::File;
use std::fmt::Debug;
#[doc(hidden)]
pub use std::path::Path;

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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// custom Results enums for various *Reader functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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

/// Enum return value for various [filepreprocessor] functions.
///
/// [filepreprocessor]: crate::readers::filepreprocessor
#[derive(Debug)]
pub enum FileProcessingResult<E> {
    FileErrEmpty,
    FileErrTooSmall,
    FileErrNullBytes,
    FileErrNoLinesFound,
    FileErrNoSyslinesFound,
    FileErrNoSyslinesInDtRange,
    /// Carries the `E` error data. This is how an [`Error`] is carried between
    /// a processing thread and the main printing thread.
    ///
    /// [`Error`]: std::io::Error
    FileErrIo(E),
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
        matches!(*self, FileProcessingResult::FileErrIo(_))
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
            FileProcessingResult::FileErrIo(_) => {
                matches!(*other, FileProcessingResult::FileErrIo(_))
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

/// File types that can be processed by [`SyslogProcessor`]
/// (and underlying "structs"; [`SyslineReader`], etc.).
///
/// [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor
/// [`SyslineReader`]: crate::readers::syslinereader::SyslineReader
// TODO: [2023/04] types Unset, Unparseable, Unknown are confusing to keep around
//       and make extra work for all match statements.
//       Can they be removed?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileType {
    /// an unset value, the default, encountering this value is an error
    Unset,
    /// a plain vanilla file, e.g. `file.log`
    File,
    /// a compressed gzipped file, e.g. `log.gz`,
    /// (presumed to contain one regular file; see Issue #8)
    Gz,
    /// a plain file within a `.tar` archive file
    Tar,
    /// a file within a compressed gzipped `.tar` or `.tgz` archive file
    /// Currently unparseable. See [Issue #14]
    ///
    /// This ended up here from overzealous planning. It is not currently used.
    ///
    /// [Issue #14]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/14
    TarGz,
    /// a file compressed "xz'd" file, e.g. `log.xz`
    /// (presumed to contain one regular file; see Issue #11)
    Xz,
    /// a binary [utmp/umtpx format] file
    ///
    /// [utmp/umtpx format]: https://man7.org/linux/man-pages/man5/utmp.5.html
    Utmpx,
    /// a [Windows XML EventLog] file
    ///
    /// [Windows XML EventLog]: https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc
    Evtx,
    /// a [systemd Journal file]
    ///
    /// [systemd Journal file]: https://systemd.io/JOURNAL_FILE_FORMAT/
    Journal,
    /// a file type known to be unparseable
    // TODO: [2023/03] fix misspelling, `Unparseable` -> `Unparsable`
    Unparseable,
    /// an unknown file type (catch all)
    Unknown,
}

// XXX: Deriving `Default` on enums is experimental.
//      See issue #86985 <https://github.com/rust-lang/rust/issues/86985>
//      When `Default` is integrated then this `impl Default` can be removed.
//      Issue #18
impl Default for FileType {
    fn default() -> Self {
        FileType::Unset
    }
}

impl std::fmt::Display for FileType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        match self {
            FileType::Unset => write!(f, "UNSET"),
            FileType::File => write!(f, "TEXT"),
            FileType::Gz => write!(f, "GZIP"),
            FileType::Tar => write!(f, "TAR"),
            FileType::TarGz => write!(f, "TAR GZIP"),
            FileType::Xz => write!(f, "XZ"),
            FileType::Utmpx => write!(f, "UTMP"),
            FileType::Evtx => write!(f, "EVTX"),
            FileType::Journal => write!(f, "JOURNAL"),
            FileType::Unparseable => write!(f, "UNPARSABLE"),
            FileType::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl FileType {
    /// Returns `true` if this is a compressed file
    #[inline(always)]
    pub const fn is_compressed(&self) -> bool {
        matches!(*self, FileType::Gz | FileType::TarGz | FileType::Xz)
    }

    /// Returns `true` if the file is within an archived file
    #[inline(always)]
    pub const fn is_archived(&self) -> bool {
        matches!(*self, FileType::Tar | FileType::TarGz)
    }

    /// Returns `true` if the file is processable
    #[inline(always)]
    pub const fn is_supported(&self) -> bool {
        matches!(*self,
            FileType::File
            | FileType::Gz
            | FileType::Tar
            | FileType::Xz
            | FileType::Utmpx
            | FileType::Evtx
            | FileType::Journal
            | FileType::Unknown
        )
    }
}


/// The type of message sent from file processing thread to the main printing
/// thread. Similar to [`LogMessage`] but without the enclosed data.
///
/// [`LogMessage`]: crate::data::common::LogMessage
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LogMessageType {
    /// Typical line-oriented log file; a "syslog" file in program parlance.
    ///
    /// Relates to a [`Sysline`].
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    Sysline,
    /// A binary [utmp/umtpx format] file.
    ///
    /// Relates to a [`Utmpx`].
    ///
    /// [utmp/umtpx format]: https://man7.org/linux/man-pages/man5/utmp.5.html
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    Utmpx,
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
            LogMessageType::Utmpx => write!(f, "utmpx entries"),
            LogMessageType::Evtx => write!(f, "evtx entries"),
            LogMessageType::Journal => write!(f, "journal entries"),
            LogMessageType::All => write!(f, "ALL"),
        }
    }
}

/// convert a [`FileType`] to a [`LogMessageType`]
pub fn filetype_to_logmessagetype(filetype: FileType) -> LogMessageType {
    match filetype {
        FileType::Utmpx => LogMessageType::Utmpx,
        FileType::Evtx => LogMessageType::Evtx,
        FileType::Journal => LogMessageType::Journal,
        FileType::File
        | FileType::Gz
        | FileType::Tar
        | FileType::Xz
        | FileType::Unknown
        => LogMessageType::Sysline,
        FileType::TarGz => panic!("unexpected filetype TarGz"),
        FileType::Unparseable => panic!("unexpected filetype Unparseable"),
        FileType::Unset => panic!("unexpected filetype Unset"),
    }
}

#[macro_export]
macro_rules! debug_panic {
    ($($arg:tt)*) => (
        if cfg!(debug_assertions) {
            panic!($($arg)*);
        }
    )
}
pub use debug_panic;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Blocks and BlockReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Offset into a file in bytes. Zero-based.
pub type FileOffset = u64;

/// A [`Vec`](std::vec::Vec) of `u8`.
pub type Bytes = Vec<u8>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Lines and LineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
