// src/subprojects/rust_lzxpress/mod.rs

//! Partial rip of crate [`rust-lzxpress`] v0.7.1 (MIT); only the MS-XCA
//! plain LZ77 `decompress` is kept. See `README.md` in this directory.
//!
//! [`rust-lzxpress`]: https://github.com/MagnetForensics/rust-lzxpress

// vendored code; keep the diff against upstream minimal
#![allow(clippy::all)]

pub mod data;
pub mod error;

pub use data::decompress;
pub use error::Error;
