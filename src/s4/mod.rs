// src/bin/mod.rs

#[cfg(feature = "alloc_tracker")]
pub mod alloc_tracker;
pub mod s4;
#[cfg(test)]
pub mod s4_tests;

use std::process::ExitCode;

#[inline(always)]
pub fn main() -> ExitCode {
    s4::main()
}
