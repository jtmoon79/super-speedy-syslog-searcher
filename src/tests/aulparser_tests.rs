//! Smoke tests for the Apple Unified Log parser.

#![allow(non_snake_case)]

use std::io::Read;
use std::path::PathBuf;

use crate::readers::aulparser::AulParser;

#[test]
fn test_new_aul_parser_empty() {
    let _parser =
        AulParser::new(
            PathBuf::new(),
            PathBuf::new(),
            Box::new(|_| Ok(Box::new(std::io::empty()) as Box<dyn Read>))
        );
}
