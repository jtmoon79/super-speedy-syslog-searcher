// src/tests/printers_tests.rs

//! tests for `src/printer/printers.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::printer::printers::{
    Color,
    ColorChoice,
    PrinterSysline,
};

extern crate si_trace_print;
#[allow(unused_imports)]
use si_trace_print::printers::{defo, defn, defx};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_PrinterSysline_new() {
    PrinterSysline::new(
        ColorChoice::Never,
        Color::Red,
        None,
        None,
        None,
    );
}
