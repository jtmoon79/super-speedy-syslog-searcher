// src/tests/rust_lzxpress_tests.rs

use std::io::{
    Cursor,
    ErrorKind,
};

use crate::readers::etlparser::{
    BUFFER_HEADER_SZ,
    EtlParser,
};
use crate::subprojects::rust_lzxpress::{
    self,
    Error,
};

const BUFFER_FLAG_COMPRESSED: u16 = 0x0040;
const BUFFER_SZ_MAX: usize = 64 * 1024 * 1024;

fn extended_length_stream() -> Vec<u8> {
    vec![
        0x00, 0x00, 0x00, 0x40, // literal followed by a match
        b'A', 0x07, 0x00, // offset 1, extended length
        0x0F, // extended-length nibble
        0xFF, // read the following u16
        0x00, 0x00, // read the following u32
        0xFF, 0xFF, 0xFF, 0xFF,
    ]
}

fn etl_buffer(
    data: &[u8],
    filled: usize,
) -> Vec<u8> {
    let buffer_size: usize = BUFFER_HEADER_SZ + data.len();
    let mut buffer: Vec<u8> = vec![0; buffer_size];
    buffer[0x00..0x04].copy_from_slice(&(buffer_size as u32).to_le_bytes());
    buffer[0x30..0x34].copy_from_slice(&(filled as u32).to_le_bytes());
    buffer[0x34..0x36].copy_from_slice(&BUFFER_FLAG_COMPRESSED.to_le_bytes());
    buffer[BUFFER_HEADER_SZ..].copy_from_slice(data);
    buffer
}

#[test]
fn decompress_with_sufficient_limit() {
    let compressed: [u8; 7] = [
        0x00, 0x00, 0x00, 0x40, b'A', 0x00, 0x00,
    ];

    let decompressed: Vec<u8> = rust_lzxpress::decompress(&compressed, 4).unwrap();

    assert_eq!(decompressed, b"AAAA");
}

#[test]
fn decompress_rejects_literal_over_limit() {
    let compressed: [u8; 5] = [0, 0, 0, 0, b'A'];

    let result = rust_lzxpress::decompress(&compressed, 0);

    assert!(matches!(result, Err(Error::MemLimit)));
}

#[test]
fn decompress_rejects_extended_length_over_limit() {
    let result = rust_lzxpress::decompress(&extended_length_stream(), 32);

    assert!(matches!(result, Err(Error::MemLimit)));
}

#[test]
fn etl_parser_rejects_filled_bytes_above_buffer_limit() {
    let buffer: Vec<u8> = etl_buffer(&[], BUFFER_SZ_MAX + 1);

    let error = EtlParser::new(Cursor::new(buffer)).unwrap_err();

    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(
        error
            .to_string()
            .contains("invalid FilledBytes")
    );
}

#[test]
fn etl_parser_bounds_lzxpress_output_to_filled_bytes() {
    let compressed: Vec<u8> = extended_length_stream();
    let buffer: Vec<u8> = etl_buffer(&compressed, BUFFER_HEADER_SZ + 32);

    let error = EtlParser::new(Cursor::new(buffer)).unwrap_err();

    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(
        error
            .to_string()
            .contains("LZXPRESS decompression failed: MemLimit")
    );
}
