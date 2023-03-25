// src/data/common.rs

//! Common types and constants for `readers`.

use crate::data::evtx::Evtx;
use crate::data::sysline::SyslineP;
use crate::data::utmpx::Utmpx;

/// The type of log message sent from file processing thread to the main
/// printing thread enclosing the specific message.
#[derive(Debug)]
pub enum LogMessage {
    Sysline(SyslineP),
    Utmpx(Utmpx),
    Evtx(Evtx),
}
pub type LogMessageOpt = Option<LogMessage>;
