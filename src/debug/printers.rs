// src/debug/printers.rs

//! A hodge-podge of printer functions and helpers for test and debug builds.

#[cfg(test)]
use crate::common::{FPath, FileOpenOptions};

#[cfg(any(debug_assertions, test))]
use std::io::Write; // for `std::io::Stdout.flush`
#[cfg(test)]
use std::io::prelude::*; // for `std::fs::File.read_to_string`

#[doc(hidden)]
pub use ::termcolor::{Color, ColorChoice, ColorSpec, WriteColor};
#[doc(hidden)]
#[allow(unused_imports)]
use ::utf8_iter::Utf8CharsEx; // provides `.chars()` on `&[u8]`

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// `d`ebug `e`println! an `err`or
#[macro_export]
macro_rules! de_err {
    (
        $($args:tt)*
    ) => {
        {
            #[cfg(any(debug_assertions,test))]
            eprint!("ERROR: ");
            #[cfg(any(debug_assertions,test))]
            eprintln!($($args)*)
        }
    }
}
pub use de_err;

/// `d`ebug `e`println! an `warn`ing
#[macro_export]
macro_rules! de_wrn {
    (
        $($args:tt)*
    ) => {
        {
            #[cfg(any(debug_assertions,test))]
            eprint!("WARNING: ");
            #[cfg(any(debug_assertions,test))]
            eprintln!($($args)*)
        }
    }
}
pub use de_wrn;

/// `e`println! an `err`or
#[macro_export]
macro_rules! e_err {
    (
        $($args:tt)*
    ) => {
        {
            eprint!("ERROR: ");
            eprintln!($($args)*)
        }
    }
}
pub use e_err;

/// `e`println! a `warn`ing
#[macro_export]
macro_rules! e_wrn {
    (
        $($args:tt)*
    ) => {
        {
            eprint!("WARNING: ");
            eprintln!($($args)*)
        }
    }
}
pub use e_wrn;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions - various print and write
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// turn passed u8 into char, for any char values that are CLI formatting instructions transform
/// them to pictoral representations, e.g. '\n' returns a pictoral unicode representation '␊'.
///
/// This is intended as an improvement of `fmt::Debug` display of `str` which control codes with
/// backslash-escape sequences, e.g. '\n'. This function keeps the printing width of a control
/// character to 1. This helps humans visually review various debug outputs.
///
/// only intended to aid visual debugging
///
/// XXX: is this implemented in std or in a crate?
#[cfg(any(debug_assertions, test))]
pub const fn char_to_char_noraw(c: char) -> char {
    // https://en.wikipedia.org/wiki/C0_and_C1_control_codes#C0_controls
    match c as u32 {
        0 => '␀',
        1 => '␁',
        2 => '␂',
        3 => '␃',
        4 => '␄',
        5 => '␅',
        6 => '␆',
        7 => '␇',  // '\a'
        8 => '␈',  // '\b'
        9 => '␉',  // '\t'
        10 => '␊', // '\n'
        11 => '␋', // '\v'
        12 => '␌', // '\f'
        13 => '␍', // '\r'
        14 => '␎',
        15 => '␏',
        16 => '␐',
        17 => '␑',
        18 => '␒',
        19 => '␓',
        20 => '␔',
        21 => '␕',
        22 => '␖',
        23 => '␗',
        24 => '␘',
        25 => '␙',
        26 => '␚',
        27 => '␛', // '\e'
        28 => '␜',
        29 => '␝',
        30 => '␞',
        31 => '␟',
        127 => '␡',
        _ => c,
    }
}

/// transform utf-8 byte (presumably) to non-raw char
///
/// only intended for debugging
#[doc(hidden)]
#[cfg(any(debug_assertions, test))]
pub const fn byte_to_char_noraw(byte: u8) -> char {
    char_to_char_noraw(byte as char)
}

/// transform buffer of chars to a non-raw String
/// chars may be invalid utf-8
///
/// only intended for debugging
#[doc(hidden)]
#[allow(non_snake_case)]
#[cfg(any(debug_assertions, test))]
pub fn buffer_to_String_noraw(buffer: &[u8]) -> String {
    let mut s2: String = String::with_capacity(buffer.len() + 1);
    for c in buffer.chars() {
        let c_ : char = char_to_char_noraw(c);
        s2.push(c_);
    }
    s2
}

/// transform valid UTF8 str to non-raw String version
///
/// only intended for debugging
#[doc(hidden)]
#[allow(non_snake_case)]
#[cfg(any(debug_assertions, test))]
pub fn str_to_String_noraw(str_buf: &str) -> String {
    let mut s2 = String::with_capacity(str_buf.len() + 1);
    for c in str_buf.chars() {
        let c_ = char_to_char_noraw(c);
        s2.push(c_);
    }
    s2
}

/// return contents of file utf-8 chars (presumably) at `path` as non-raw String
///
/// only intended for debugging
#[allow(non_snake_case)]
#[cfg(test)]
pub fn file_to_String_noraw(path: &FPath) -> String {
    let path_ = std::path::Path::new(path);
    let mut open_options = FileOpenOptions::new();
    let mut file_ = match open_options
        .read(true)
        .open(path_)
    {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: File::open('{:?}') error {}", path_, err);
            return String::with_capacity(0);
        }
    };
    let filesz = match file_.metadata() {
        Ok(val) => val.len() as usize,
        Err(err) => {
            eprintln!("ERROR: File::metadata() error {}", err);
            return String::with_capacity(0);
        }
    };
    let mut s2 = String::with_capacity(filesz + 1);
    let s2read = match file_.read_to_string(&mut s2) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: File::read_to_string() error {}", err);
            return String::with_capacity(0);
        }
    };
    assert_eq!(
        s2read, filesz,
        "Read {} bytes but expected to read file size count of bytes {} for file {:?}",
        s2read, filesz, path
    );
    let mut s3 = String::with_capacity(filesz + 1);
    for c in s2.chars() {
        let c_ = char_to_char_noraw(c);
        s3.push(c_);
    }

    s3
}

/// Helper flush stdout and stderr
#[doc(hidden)]
#[allow(dead_code)]
#[cfg(any(debug_assertions, test))]
pub fn flush_stdouterr() {
    #[allow(clippy::match_single_binding)]
    match std::io::stdout().flush() {
        _ => {}
    };
    #[allow(clippy::match_single_binding)]
    match std::io::stderr().flush() {
        _ => {}
    };
}
