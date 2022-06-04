// Readers/filepreprocessor_tests.rs
//

use std::fs::File;

use crate::common::{
    FPath,
    Bytes,
};

use crate::Readers::blockreader::{
    BlockSz,
};

use crate::Readers::linereader::{
    FileOffset,
    ResultS4_LineFind,
    LineReader,
    LineIndex,
    enum_BoxPtrs,
};

use crate::Readers::helpers::{
    randomize,
    fill,
};

use crate::Readers::filepreprocessor::{
    FilePreProcessor,
};

use crate::dbgpr::helpers::{
    NamedTempFile,
    tempdir,
    create_temp_file,
    create_temp_file_with_name,
    create_temp_file_with_name_exact,
    NTF_Path,
    eprint_file,
};

use crate::dbgpr::printers::{
    //Color,
    //print_colored_stdout,
    byte_to_char_noraw,
    buffer_to_String_noraw,
};

use crate::dbgpr::stack::{
    sn,
    so,
    sx,
    stack_offset_set,
};

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate more_asserts;
use more_asserts::{
    assert_lt,
    assert_ge,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// helper to wrap the match and panic checks
#[cfg(test)]
fn new_FilePreprocessor(path: &FPath) -> FilePreProcessor {
    match FilePreProcessor::new(path.clone()) {
        Ok(val) => {
            eprintln!("{:?}", val);
            val
        },
        Err(err) => {
            panic!("ERROR: FilePreProcessor::new({:?}) failed {}", path, err);
        }
    }
}

#[test]
fn test_FilePreProcessor_new1() {
    let ntf = create_temp_file_with_name_exact("", String::from("foo.txt"));
    let path = NTF_Path(&ntf);
    new_FilePreprocessor(&path);
}
