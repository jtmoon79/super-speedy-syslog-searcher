// src/printer/mod.rs

//! The `printer` module is for printing user-facing [`Sysline`s] with various
//! text effects (color, underline, etc.) and per-[`Line`] prepended data
//! (datetime, file name, etc.).
//!
//! [`Line`]: crate::data::line::Line
//! [`Sysline`s]: crate::data::sysline::Sysline

pub mod printers;
