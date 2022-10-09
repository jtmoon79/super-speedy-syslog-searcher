// src/lib.rs

//! _Super Speedy Syslog Searcher_ library, _s4lib_!
//!
//! This is the library implementation used by binary program _s4_.
//! This library is documented in part to have a presence on _crates.io_ and
//! _docs.rs_.
//!
//! The _s4lib_ library was not designed for use outside of program _s4_,
//! and it was not designed to be an especially user-friendly API.
//!
//! The term "syslog" within code context is used refers to a log file
//! where each log message has some parsesable datetimestamp. It is not
//! necessarily an [RFC 5424] compliant message.
//! Also see [_Definitions of data_] and [`Sysline`].
//!
//! [RFC 5424]: https://www.rfc-editor.org/rfc/rfc5424.html
//! [_Definitions of data_]: crate::data
//! [`Sysline`]: crate::data::sysline::Sysline

pub mod common;

pub mod data;

#[doc(hidden)]
pub mod debug;

pub mod printer;

pub mod readers;

#[cfg(test)]
pub mod tests;

#[doc(hidden)]
pub fn main() {}
