// src/bin/mod.rs

//! `s4` module for the _s4_ binary program.

use std::process::ExitCode;

pub mod s4;


pub fn main() -> ExitCode {
    s4::main()
}
