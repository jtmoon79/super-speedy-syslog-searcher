//! Smoke tests for the Apple Unified Log reader.

#![allow(non_snake_case)]

use std::io::Read;
use std::path::PathBuf;

use crate::readers::aulreader::AulReader;

#[test]
fn test_new_aul_reader_empty() {
    let _reader =
        AulReader::with_opener(
            PathBuf::new(),
            PathBuf::new(),
            |_| Ok(Box::new(std::io::empty()) as Box<dyn Read>)
        );
}
