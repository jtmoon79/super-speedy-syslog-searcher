// tests/s4_tests/mod.rs

pub mod common;

#[cfg(test)]
pub mod blockreader_tests;

#[cfg(test)]
pub mod filepreprocessor_tests;

#[cfg(test)]
pub mod line_tests;

#[cfg(test)]
pub mod sysline_tests;

#[cfg(test)]
pub mod syslogprocessor_tests;
