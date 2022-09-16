// src/debug/mod.rs

//! The `debug` module is functions for printing in debug builds and
//! test builds.

#[cfg(test)]
pub mod helpers;

pub mod printers;
