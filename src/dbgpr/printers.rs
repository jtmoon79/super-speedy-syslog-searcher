// dgbpr/printers.rs
//
// debug printing - printer functions and helpers for test and debug builds
//
// TODO: [2022/04/14] needs consolidation of overlapping functions. many were written in haste.
//

#[allow(unused_imports)]  // XXX: clippy errantly marks this as unused
#[cfg(any(debug_assertions,test))]
use crate::common::{
    FileOpenOptions,
    FPath,
};

use crate::printer::printers::{
    Color,
    ColorSpec,
    WriteColor,
    COLOR_DATETIME,
    color_rand,
    print_colored_stdout,
    print_colored_stderr,
    write_stdout,
};

use std::io::Write;  // for `std::io::Stdout.flush`
#[allow(unused_imports)]  // XXX: clippy errantly marks this as unused
#[cfg(any(debug_assertions,test))]
use std::io::prelude::*;  // for `std::fs::File.read_to_string`
//use std::io::Result;

// see https://docs.rs/strum_macros/0.24.0/strum_macros/derive.AsRefStr.html
//use std::convert::AsRef;
//extern crate strum_macros;
//use strum_macros::EnumString;
//use std::str::FromStr;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions - various print and write
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// turn passed u8 into char, for any char values that are CLI formatting instructions transform
/// them to pictoral representations, e.g. '\n' returns a pictoral unicode representation '␊'
///
/// only intended for debugging
//#[cfg(any(debug_assertions,test))]
pub fn char_to_char_noraw(c: char) -> char {
    if c.is_ascii_graphic() {
        return c;
    }
    // https://www.fileformat.info/info/unicode/block/control_pictures/images.htm
    // https://en.wikipedia.org/wiki/C0_and_C1_control_codes#C0_controls
    let val: u32 = c as u32;
    match val {
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
        _ => ' ',
    }
}

/// transform utf-8 byte (presumably) to non-raw char
/// 
/// only intended for debugging
//#[cfg(any(debug_assertions,test))]
pub fn byte_to_char_noraw(byte: u8) -> char {
    char_to_char_noraw(byte as char)
}

/// transform buffer of utf-8 chars (presumably) to a non-raw String
/// 
/// only intended for debugging
#[allow(non_snake_case)]
#[cfg(any(debug_assertions,test))]
pub fn buffer_to_String_noraw(buffer: &[u8]) -> String {
    let s1 = match core::str::from_utf8(buffer) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: buffer_to_String_noraw: Invalid UTF-8 sequence during from_utf8: {}", err);
            return String::with_capacity(0);
        }
    };
    let mut s2 = String::with_capacity(s1.len() + 10);
    for c in s1.chars() {
        let c_ = char_to_char_noraw(c);
        s2.push(c_);
    }
    s2
}

/// transform str to non-raw String version
/// 
/// only intended for debugging
#[allow(non_snake_case)]
#[cfg(any(debug_assertions,test))]
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
/// only intended for debugging]
#[allow(dead_code, non_snake_case)]
#[cfg(test)]
pub fn file_to_String_noraw(path: &FPath) -> String {
    let path_ = std::path::Path::new(path);
    let mut open_options = FileOpenOptions::new();
    let mut file_ = match open_options.read(true).open(&path_) {
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

/// helper flush stdout and stderr
#[allow(dead_code)]
#[cfg(any(debug_assertions,test))]
pub fn flush_stdouterr() {
    #[allow(clippy::match_single_binding)]
    match std::io::stdout().flush() { _ => {} };
    #[allow(clippy::match_single_binding)]
    match std::io::stderr().flush() { _ => {} };
}

/// write to console, `raw` as `true` means "as-is"
/// else use `char_to_char_noraw` to replace chars in `buffer` (inefficient)
///
/// only intended for debugging
#[allow(dead_code)]
#[cfg(any(debug_assertions,test))]
pub fn pretty_print(buffer: &[u8], raw: bool) {
    if raw {
        return write_stdout(buffer);
    }
    // is this an expensive command? should `stdout` be cached?
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();
    // XXX: only handle single-byte encodings
    // XXX: doing this char by char is probably not efficient
    //let s = match str::from_utf8_lossy(buffer) {
    let s = match core::str::from_utf8(buffer) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: pretty_print: Invalid UTF-8 sequence during from_utf8: {}", err);
            return;
        }
    };
    let mut dst: [u8; 4] = [0, 0, 0, 0];
    for c in s.chars() {
        let c_ = char_to_char_noraw(c);
        let _cs = c_.encode_utf8(&mut dst);
        match stdout_lock.write(&dst) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: pretty_print: StdoutLock.write({:?}) error {}", &dst, err);
            }
        }
    }
    match stdout_lock.flush() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: pretty_print: stdout flushing error {}", err);
        }
    }
}
