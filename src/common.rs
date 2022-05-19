// common.rs
//
// common imports, type aliases, and other globals (avoids circular imports)

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// file-handling, command-line parsing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub use std::fs::File;
use std::fmt::Debug;
pub use std::path::Path;

// TODO: use `std::path::Path` for `FPath`
/// `F`ake `Path` or `F`ile `Path`
pub type FPath = String;
pub type FPaths = Vec::<FPath>;
pub type FileMetadata = std::fs::Metadata;
pub type FileOpenOptions = std::fs::OpenOptions;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// custom Results enums for various *Reader functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#481-495

/// `Result` Extended
/// for line and sysline searching functions
///
/// TODO: [2022/05/03] getting rid of `Found_EOF` would simplify a lot of code
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResultS4<T, E> {
    /// Contains the success data
    Found(T),

    /// Contains the success data and reached End Of File and things are okay
    #[allow(non_camel_case_types)]
    Found_EOF(T),

    /// File is empty, or other condition that means "Done", nothing to return, but no bad errors happened
    Done,

    /// Contains the error value, something bad happened
    Err(E),
}

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#501-659
// XXX: how to link to specific version of `result.rs`?

impl<T, E> ResultS4<T, E> {
    // Querying the contained values

    /// Returns `true` if the result is [`Found`, `Found_EOF`, 'Done`].
    #[allow(dead_code)]
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline(always)]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, ResultS4::Found(_) | ResultS4::Found_EOF(_) | ResultS4::Done)
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

    /// Returns `true` if the result is [`Found_EOF`].
    #[inline(always)]
    pub const fn is_eof(&self) -> bool {
        matches!(*self, ResultS4::Found_EOF(_))
    }

    /// Returns `true` if the result is [`Done`].
    #[inline(always)]
    pub const fn is_done(&self) -> bool {
        matches!(*self, ResultS4::Done)
    }

    /// Returns `true` if the result is an [`Found`, `Found_EOF`] value containing the given value.
    #[allow(dead_code)]
    #[must_use]
    #[inline(always)]
    pub fn contains<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            ResultS4::Found(y) => x == y,
            ResultS4::Found_EOF(y) => x == y,
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
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: ResultS4<u32, &str> = Ok(2);
    /// assert_eq!(x.ok(), Some(2));
    ///
    /// let x: ResultS4<u32, &str> = Err("Nothing here");
    /// assert_eq!(x.ok(), None);
    /// ```
    #[allow(dead_code)]
    #[inline(always)]
    pub fn ok(self) -> Option<T> {
        match self {
            ResultS4::Found(x) => Some(x),
            ResultS4::Found_EOF(x) => Some(x),
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
            ResultS4::Found_EOF(_) => None,
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
            ResultS4::Found_EOF(_) => { write!(f, "ResultS4::Found_EOF") },
            ResultS4::Done => { write!(f, "ResultS4::Done") },
            ResultS4::Err(err) => { write!(f, "ResultS4::Err({})", err) },
        }
    }
}

/// `Result` Extended
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
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: ResultS3<u32, &str> = Ok(2);
    /// assert_eq!(x.ok(), Some(2));
    ///
    /// let x: ResultS3<u32, &str> = Err("Nothing here");
    /// assert_eq!(x.ok(), None);
    /// ```
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Blocks and BlockReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Offset into a file in bytes
pub type FileOffset = u64;

/// Sequence of Bytes
pub type Bytes = Vec<u8>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Lines and LineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// minimum size of characters of file in bytes
/// UTF-8 would be value `1`, UTF-16 would be value `2`, etc.
pub type CharSz = usize;
/// NewLine as char
#[allow(dead_code, non_upper_case_globals)]
pub const NLc: char = '\n';
/// Single-byte newLine char as u8
#[allow(non_upper_case_globals)]
pub const NLu8: u8 = 10;
/// Newline in a byte buffer
#[allow(non_upper_case_globals)]
pub const NLu8a: [u8; 1] = [NLu8];
