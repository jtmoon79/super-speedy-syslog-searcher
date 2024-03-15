// src/tests/mod.rs

//! Tests for _s4lib_.
//!
//! Tests are placed at `src/tests/`, inside the `s4lib`. The author concluded
//! this is a reasonable trade-off of separation and access.
//!
//! Tests placed at top-level path `tests/` do not have crate-internal
//! visibility. While it is recommended to not require internal visibility for
//! testing, in practice that often makes tests difficult or impossible to
//! implement.

pub mod blockreader_tests;
pub mod common;
pub mod datetime_tests;
pub mod evtx_tests;
pub mod evtxreader_tests;
pub mod filepreprocessor_tests;
pub mod fixedstruct_tests;
pub mod fixedstructreader_tests;
pub mod journal_tests;
pub mod journalreader_tests;
pub mod line_tests;
pub mod linereader_tests;
pub mod printers_tests;
pub mod sysline_tests;
pub mod syslinereader_tests;
pub mod syslogprocessor_tests;
