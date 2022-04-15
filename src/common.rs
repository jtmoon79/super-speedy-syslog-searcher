// common.rs
//
// common imports, type aliases, and other globals (also avoids circular imports)

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// file-handling, command-line parsing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub use std::fs::File;
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

/// `Result` `Ext`ended
/// sometimes things are not `Ok` but a value needs to be returned
#[derive(Debug)]
pub enum ResultS4<T, E> {
    /// Contains the success data
    Found(T),

    /// Contains the success data and reached End Of File and things are okay
    #[allow(non_camel_case_types)]
    Found_EOF(T),

    /// File is empty, or other condition that means "Done", nothing to return, but no bad errors happened
    #[allow(non_camel_case_types)]
    Done,

    /// Contains the error value, something unexpected happened
    Err(E),
}

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#501-659
// XXX: how to link to specific version of `result.rs`?

impl<T, E> ResultS4<T, E> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the result is [`Ok`, `Found_EOF`, 'Done`].
    #[allow(dead_code)]
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline(always)]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, ResultS4::Found(_) | ResultS4::Found_EOF(_) | ResultS4::Done)
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

    /// Returns `true` if the result is [`Found_EOF`, `Done`].
    #[inline(always)]
    pub const fn is_done(&self) -> bool {
        matches!(*self, ResultS4::Done)
    }

    /// Returns `true` if the result is an [`Ok`, `Found_EOF`] value containing the given value.
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

    /////////////////////////////////////////////////////////////////////////
    // Adapter for each variant
    /////////////////////////////////////////////////////////////////////////

    /// Converts from `Result<T, E>` to [`Option<T>`].
    ///
    /// Converts `self` into an [`Option<T>`], consuming `self`,
    /// and discarding the error, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.ok(), Some(2));
    ///
    /// let x: Result<u32, &str> = Err("Nothing here");
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

    /// Converts from `Result<T, E>` to [`Option<E>`].
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
