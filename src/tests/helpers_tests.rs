// src/tests/helpers_tests.rs

//! tests for `helpers.rs` functions

use std::path::Path;

use ::test_case::test_case;

use crate::readers::helpers::{
    count_char_in_str,
    filename_count_extensions,
};


#[test_case("", '.', 0; "empty string")]
#[test_case("a.b.c", '.', 2; "a.b.c")]
#[test_case(".....", '.', 5; "5.....")]
fn test_count_char_in_str(input: &str, c: char, expected: usize) {
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
fn test_filename_count_extensions(path: &str, expected: usize) {
    let path_: &Path = Path::new(path);
    let result = filename_count_extensions(path_);
    assert_eq!(result, expected);
}
