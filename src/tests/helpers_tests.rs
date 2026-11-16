// src/tests/helpers_tests.rs

//! tests for `helpers.rs` functions

use std::io::Cursor;
use std::path::Path;

use ::test_case::test_case;

use crate::readers::helpers::{
    count_char_in_str,
    filename_count_extensions,
    gzip_header_metadata,
};

#[test_case("", '.', 0; "empty string")]
#[test_case("a.b.c", '.', 2; "a.b.c")]
#[test_case(".....", '.', 5; "5.....")]
fn test_count_char_in_str(
    input: &str,
    c: char,
    expected: usize,
) {
    let result = count_char_in_str(input, c);
    assert_eq!(result, expected);
}

#[test_case("/path/to/file.log", 1)]
#[test_case("/pa.th/to/file.log", 1)]
#[test_case("/path/to./file.tar.gz", 2)]
#[test_case("/path/to./.file", 1)]
#[test_case("/path/to./file", 0)]
#[test_case("../..", 0; "2dots2")]
#[test_case("", 0; "empty path")]
fn test_filename_count_extensions(
    path: &str,
    expected: usize,
) {
    let path_: &Path = Path::new(path);
    let result = filename_count_extensions(path_);
    assert_eq!(result, expected);
}

#[test]
fn test_gzip_header_metadata_extracts_fields_and_restores_position() {
    let mut gzip = vec![0x1f, 0x8b, 8, 0x1e];
    gzip.extend_from_slice(&0x1234_5678u32.to_le_bytes());
    gzip.extend_from_slice(&[0, 255]);
    gzip.extend_from_slice(&2u16.to_le_bytes());
    gzip.extend_from_slice(b"xyname.log\0a comment\0\x01\x02payload");
    let mut reader = Cursor::new([b"prefix".as_slice(), gzip.as_slice()].concat());
    reader.set_position(6);

    assert_eq!(
        gzip_header_metadata(&mut reader).unwrap(),
        ("name.log".to_owned(), 0x1234_5678),
    );
    assert_eq!(reader.position(), 6);
}
