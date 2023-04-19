// src/printer/mod.rs

//! The `printer` module is for printing user-facing log messages
//! ([`Sysline`s], [`Utmpx`s], [`Evtx`s], and [`JournalEntry`s]).
//! with various text effects (color, underline, etc.)
//! and per-line prepended data (datetime, file name, etc.).
//!
//! [`Sysline`s]: crate::data::sysline::Sysline
//! [`Utmpx`s]: crate::data::utmpx::Utmpx
//! [`Evtx`s]: crate::data::evtx::Evtx
//! [`JournalEntry`s]: crate::data::journal::JournalEntry

pub mod printers;
