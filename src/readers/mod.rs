// src/readers/mod.rs

//! "Readers" for _s4lib_.
//!
//! ## Overview of readers
//!
//! * A [`SyslogProcessor`] drives a [`SyslineReader`] to derive [`Sysline`s].
//! * A `SyslineReader` drives a [`LineReader`] to derive [`Line`s].
//! * A `LineReader` drives a [`BlockReader`] to derive [`Block`s].
//!
//! <br/>
//!
//! * A `BlockReader` only handles `u8` bytes.
//! * A `LineReader` and a `SyslineReader` strongly prefer to handle `u8`
//!   bytes but converts to `char` when necessary.<br/>
//!   Avoiding `u8` to `char` conversion avoids potential errors and
//!   significantly improves program performance.
//! * A `LineReader` does the majority of `u8` to `char` conversions.
//!
//! <br/>
//!
//! Also see [_Definitions of data_].
//!
//! <br/>
//!
//! ---
//!
//! The _s4_ binary program uses a [`SyslogProcessor`] instance, one per file,
//! to drive processing for a file.
//!
//! <br/>
//!
//! _These are not rust "Readers"; these structs do not implement the trait
//! [`Read`]. These are "readers" in an informal sense._
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

pub mod blockreader;
pub mod filepreprocessor;
pub mod helpers;
pub mod linereader;
pub mod summary;
pub mod syslinereader;
pub mod syslogprocessor;
