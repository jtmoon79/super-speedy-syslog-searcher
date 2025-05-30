// src/printer/mod.rs

//! The `printer` module is for printing user-facing log messages
//! ([`Sysline`s], [`FixedStruct`s], [`Evtx`s], and [`JournalEntry`s]).
//! with various text effects (color, underline, etc.)
//! and per-line prepended data (datetime, file name, etc.).
//!
//! [`summary`] module is functions to handle the user passed `--summary`
//! option.
//!
//! [`Sysline`s]: crate::data::sysline::Sysline
//! [`FixedStruct`s]: crate::data::fixedstruct::FixedStruct
//! [`Evtx`s]: crate::data::evtx::Evtx
//! [`JournalEntry`s]: crate::data::journal::JournalEntry

pub mod printers;
pub mod summary;
