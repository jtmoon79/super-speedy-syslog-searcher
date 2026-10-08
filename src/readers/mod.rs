// src/readers/mod.rs

//! "Readers" for _s4lib_.
//!
//! ## Overview of readers
//!
//! ### Reading log and syslog files
//!
//! * A [`SyslogProcessor`] drives a [`SyslineReader`] to derive [`Sysline`s].
//! * A `SyslineReader` drives a [`LineReader`] to derive [`Line`s].
//! * A `LineReader` drives a [`BlockReader`] to derive [`Block`s].
//!
//! <br/>
//!
//! * A `BlockReader` only handles `u8` bytes.
//! * A `LineReader` and a `SyslineReader` strongly prefer to handle `u8` bytes but converts to
//!   `char` when necessary.<br/> Avoiding `u8` to `char` conversion avoids potential errors and
//!   significantly improves program performance.
//! * A `LineReader` does the majority of `u8` to `char` conversions.
//!
//! ### Reading C-struct record-keeping files; acct, lastlog, utmp, etc.
//!
//! * A [`FixedStructReader`] drives a [`BlockReader`] to derive
//! [`FixedStruct`s].
//!
//! <br/>
//!
//! ### Reading [evtx files]; Windows Event Log XML files
//!
//! * A [`EvtxReader`] drives a [`EvtxParser`] to derive [`Evtx`s].
//!
//! <br/>
//!
//! ### Reading [etl files]; Windows Event Trace Log files
//!
//! * A [`EtlReader`] drives a [`EtlParser`] to derive [`Etl`s].
//!
//! <br/>
//!
//! ### Reading Apple System Log files
//!
//! * An [`AslReader`] drives an [`AslParser`] to derive [`Asl`] events.
//!
//! <br/>
//!
//! ### Reading [`systemd` journal files]
//!
//! * A [`JournalReader`] drives a [`JournalApiPtr`] to derive
//! [Journal entries].
//!
//! <br/>
//! <br/>
//!
//! Also see [_Definitions of data_].
//!
//! <br/>
//!
//! ---
//!
//! The _s4_ binary program uses a [`SyslogProcessor`], a [`FixedStructReader`],
//! a [`EvtxReader`], a [`EtlReader`], an [`AslReader`], or a [`JournalReader`], instance,
//! one per file, to drive processing of the file.
//!
//! <br/>
//!
//! _Most of these are "readers" only in an informal sense; `AslReader`
//! additionally implements [`Read`] over rendered event bytes._
//!
//! [_Definitions of data_]: crate::data
//! [`Read`]: std::io::Read
//! [`Block`s]: crate::readers::blockreader::Block
//! [`Line`s]: crate::data::line::Line
//! [`Sysline`s]: crate::data::sysline::Sysline
//! [`BlockReader`]: crate::readers::blockreader::BlockReader
//! [`LineReader`]: crate::readers::linereader::LineReader
//! [`SyslineReader`]: crate::readers::syslinereader::SyslineReader
//! [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor
//! [`FixedStructReader`]: crate::readers::fixedstructreader::FixedStructReader
//! [`EvtxReader`]: crate::readers::evtxreader::EvtxReader
//! [`EtlReader`]: crate::readers::etlreader::EtlReader
//! [`EtlParser`]: crate::readers::etlparser::EtlParser
//! [`Etl`s]: crate::data::etl::Etl
//! [`AslReader`]: crate::readers::aslreader::AslReader
//! [`AslParser`]: crate::readers::aslparser::AslParser
//! [`Asl`]: crate::data::asl::Asl
//! [etl files]: https://learn.microsoft.com/en-us/windows/win32/etw/about-event-tracing
//! [`systemd` journal files]: https://systemd.io/JOURNAL_FILES/
//! [`JournalReader`]: crate::readers::journalreader::JournalReader
//! [`JournalApiPtr`]: crate::libload::systemd_dlopen2::JournalApiPtr
//! [Journal entries]: https://systemd.io/JOURNAL_FILE_FORMAT/
//! [`FixedStruct`s]: crate::data::fixedstruct::FixedStruct
//! [`EvtxParser`]: https://docs.rs/evtx/0.8.1/evtx/struct.EvtxParser.html
//! [`Evtx`s]: crate::data::evtx::Evtx
//! [evtx files]: https://en.wikipedia.org/w/index.php?title=Event_Viewer&oldid=1130075772#Windows_Vista

pub mod aslparser;
pub mod aslreader;
pub mod blockreader;
pub mod etlparser;
pub mod etlreader;
pub mod evtxreader;
pub mod filedecompressor;
pub mod filehandlemanager;
pub mod filepreprocessor;
pub mod fixedstructreader;
pub mod helpers;
pub mod journalreader;
pub mod linereader;
pub mod odlparser;
pub mod odlreader;
pub mod summary;
pub mod syslinereader;
pub mod syslogprocessor;
