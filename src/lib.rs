// src/lib.rs

//! _Super Speedy Syslog Searcher_ library, _s4lib_!
//!
//! ## Introduction
//!
//! This is the library implementation used by binary program _s4_.
//! This library is documented in part to have a presence on _docs.rs_.
//!
//! The _s4lib_ library was not designed for use outside of program _s4_,
//! and it was not designed to be an especially user-friendly API.
//!
//! The term "syslog" within code context is used refers to a file
//! where a text-encoded message has some parsesable datetimestamp. It includes
//! but it not limited to an [RFC 5424] compliant message.
//!
//! The term "log message" is for any type log message, including
//! ad-hoc log messages, formal syslog RFC 5424 messages, fixedstruct entries
//! (acct/lastlog/lastlogx/utmp/utmpx/wtmp/wtmpx), systemd journal messages,
//! evtx entries, and other types of log messages.
//!
//! ## Overview of modules
//!
//! Broadly, there are definietions of data, under the [`data`] module, and
//! there
//! are Readers, under [`readers`] module. Note that the "Reader"s do not
//! implement the Rust `Read` trait; it is merely a general phrase. These are
//! where this tool's specific features are implemented.
//! <br/>
//! The [`printer`] module handles printing log messages to standard output,
//! along with user-passed command-line printing options
//! (e.g. `--color=always`, `--prepend-utc`, etc.).<br/>
//! The [`debug`] module is for helper functions and features for debug builds
//! and testing (it may not appear in these generated docs).<br/>
//! The [`libload`] module is for loading shared
//! libraries at runtime (e.g. `libsystemd.so` to parse journal files).<br/>
//! The [`common`] module is for shared constants, definitions, and helper
//! functions. There are also sub-module `common.rs` specific to that module,
//! e.g. [`data/common.rs`].<br/>
//! And finally the driver is under the [`bin/s4.rs`].
//!
//! Also see [_Definitions of data_] and [`Sysline`].
//!
//! [RFC 5424]: https://www.rfc-editor.org/rfc/rfc5424.html
//! [`bin/s4.rs`]: ../s4/index.html
//! [`data/common.rs`]: crate::data::common
//! [_Definitions of data_]: crate::data
//! [`Sysline`]: crate::data::sysline::Sysline

pub mod bindings;
pub mod common;
pub mod data;
#[doc(hidden)]
pub mod debug;
pub mod libload;
pub mod printer;
pub mod readers;
#[cfg(test)]
pub mod tests;

#[doc(hidden)]
pub fn main() {}
