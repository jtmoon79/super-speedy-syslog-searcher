// src/data/mod.rs

//! The `data` module is specialized data containers for
//! [`Blocks`], [`Line`]s, [`Sysline`]s, [`FixedStruct`], [`Evtx`],
//! and [`JournalEntry`]s.
//!
//! The sub-module [`datetime`] defines regular expressions and functions for
//! parsing datetimes substrings from ad-hoc text log messages ("syslogs").
//!
//! ## Definitions of data
//!
//! ### Sysline
//!
//! A `Sysline` is composed of several specialized structs.
//!
//! #### Block
//!
//! A "block" is a sequence of contiguous bytes in a file that:
//!
//! * have the same length as other blocks in the file, except for the last block which has an equal
//!   or lesser length.
//!
//! A "block" is represented by a [`Block`] and retrieved by a [`BlockReader`].
//!
//! #### Line
//!
//! A "line" is sequence of bytes residing on "blocks" that:
//!
//! * begin after a prior "line" or the beginning of a file.
//! * end with a newline character `'\n'` or the end of a file.
//!
//! A "line" is represented by a [`Line`] and found by a [`LineReader`].
//!
//! #### Sysline
//!
//! A "sysline" is sequence of "lines" that:
//!
//! * have a datetime stamp on the first "line".
//! * have a datetime stamp format similar to other "sysline"s in the same file.
//!
//! A "sysline" is represented by a [`Sysline`] and found by a
//! [`SyslineReader`].
//!
//! A `Sysline` represents a "log message".
//!
//! It is not necessarily referring to an [RFC 5424] compliant log message.
//!
//! ### Syslog
//!
//! A "syslog" is a file that:
//!
//! * has [a certain minimum] of "sysline"s.
//!
//! A "syslog" is processed by a [`SyslogProcessor`].
//!
//! In this project and source code, "syslog" is used loosely; it is not
//! necessarily referring to an [RFC 5424] compliant log file.
//!
//! ### FixedStruct
//!
//! A [`FixedStruct`] is information about a processed `acct`, `lastlog`,
//! `lastlogx`, `utmp`, or `utmpx` record-keeping C struct.
//! It is processed by a [`FixedStructReader`].
//! It uses an underlying [`BlockReader`] to read from the file.
//!
//! A `FixedStruct` entry represents a "log message".
//!
//! ### Evtx
//!
//! A [`Evtx`] is information about a processed [`evtx`] structure
//! processed from a file. It is processed by a [`EvtxReader`].
//!
//! An `Evtx` represents a "log message".
//!
//! ### Journal
//!
//! A [`JournalEntry`] is information about a processed [systemd journal entry].
//! It is processed by a [`JournalReader`].
//!
//! A `JournalEntry` represents a "log message".
//!
//! <br/>
//! <br/>
//!
//! _The "Readers" are not rust "Readers"; "_Reader_" structs do not implement
//! the trait [`Read`]. These are "readers" in an informal sense._
//!
//! Also see [_Overview of readers_].
//!
//! [_Overview of readers_]: crate::readers
//! [`BlockReader`]: crate::readers::blockreader::BlockReader
//! [`LineReader`]: crate::readers::linereader::LineReader
//! [`SyslineReader`]: crate::readers::syslinereader::SyslineReader
//! [`JournalReader`]: crate::readers::journalreader::JournalReader
//! [`EvtxReader`]: crate::readers::evtxreader::EvtxReader
//! [`FixedStructReader`]: crate::readers::fixedstructreader::FixedStructReader
//! [`Block`]: crate::readers::blockreader::Block
//! [`Blocks`]: crate::readers::blockreader::Block
//! [`Line`]: crate::data::line::Line
//! [`Sysline`]: crate::data::sysline::Sysline
//! [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor
//! [RFC 5424]: https://www.rfc-editor.org/rfc/rfc5424.html
//! [a certain minimum]: static@crate::readers::syslogprocessor::BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP
//! [`FixedStruct`]: crate::data::fixedstruct::FixedStruct
//! [`Evtx`]: crate::data::evtx::Evtx
//! [`evtx`]: crate::data::evtx::SerializedEvtxRecord
//! [`JournalEntry`]: crate::data::journal::JournalEntry
//! [systemd journal entry]: https://systemd.io/JOURNAL_FILE_FORMAT/
//! [`Read`]: std::io::Read

pub mod common;
pub mod datetime;
pub mod evtx;
pub mod fixedstruct;
pub mod journal;
pub mod line;
pub mod sysline;
