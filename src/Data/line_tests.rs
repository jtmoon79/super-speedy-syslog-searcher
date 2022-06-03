// Data/line_tests.rs
//

use crate::common::{
    FPath,
    FileOffset,
};

use crate::dbgpr::stack::{
    stack_offset_set,
    sn,
    snx,
    so,
    sx,
};

use crate::dbgpr::helpers::{
    NamedTempFile,
    create_temp_file,
    create_temp_file_bytes,
    create_temp_file_with_name_exact,
    NTF_Path,
    eprint_file,
};

use crate::Data::line::{
    LineIndex,
    Line,
    LinePart,
};

use crate::Readers::blockreader::{
    BlockSz,
};

use crate::Readers::datetime::{
    FixedOffset,
    TimeZone,
};

pub use crate::Readers::syslogprocessor::{
    SyslogProcessor,
};

use std::io::{
    Error,
    Result,
    ErrorKind,
};

extern crate debug_print;
use debug_print::{debug_eprint, debug_eprintln};

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    assert_ge,
    assert_gt,
    debug_assert_le,
    debug_assert_lt,
    debug_assert_ge,
};

extern crate lazy_static;
use lazy_static::lazy_static;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_line_new1() {
    let l1 = Line::new();
}

// TODO: [2022/06/02] needs more tests of `Data/line.rs`
