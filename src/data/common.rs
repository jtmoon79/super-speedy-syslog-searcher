// src/data/common.rs

//! Common types and constants for `readers`.

use crate::data::evtx::Evtx;
use crate::data::sysline::SyslineP;
use crate::data::utmpx::Utmpx;
use crate::data::journal::JournalEntry;

/// The type of log message sent from file processing thread to the main
/// printing thread enclosing the specific message.
#[derive(Debug)]
pub enum LogMessage {
    Sysline(SyslineP),
    Utmpx(Utmpx),
    Evtx(Evtx),
    Journal(JournalEntry),
}
pub type LogMessageOpt = Option<LogMessage>;

/// Type alias for bytes offsets of the beginning and end of the
/// datetime substring within a `String`.
pub type DtBegEndPair = (usize, usize);
/// Type alias for [`Option`] of [`DtBegEndPair`].
pub type DtBegEndPairOpt = Option<DtBegEndPair>;
