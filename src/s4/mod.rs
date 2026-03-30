// src/bin/mod.rs

pub mod s4;
#[cfg(test)]
pub mod s4_tests;

use std::process::ExitCode;

#[inline(always)]
pub fn main() -> ExitCode {
    s4::main()
}
