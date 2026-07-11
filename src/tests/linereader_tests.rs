// src/tests/linereader_tests.rs

//! tests for `linereader.rs`

// TODO: [2023/01/14] replace eprintln! with si_trace_print macros

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::too_many_arguments)]

use std::io::ErrorKind;

use ::lazy_static::lazy_static;
use ::more_asserts::{
    assert_ge,
    assert_le,
};
use ::si_trace_print::printers::{
    defn,
    defo,
    defx,
};
use ::si_trace_print::stack::{
    sn,
    so,
    stack_offset_set,
    sx,
};
use ::test_case::{
    test_case,
    test_matrix,
};

use crate::common::{
    Bytes,
    Count,
    FPath,
    FileOffset,
    FileTypeTextEncoding,
    summary_stats_enable,
};
use crate::data::line::{
    LineIndex,
    LineP,
    LinePartPtrs,
};
use crate::debug::helpers::{
    create_temp_file,
    create_temp_file_no_permissions,
    ntf_fpath,
};
use crate::debug::printers::{
    buffer_to_string_noraw,
    byte_to_char_noraw,
    str_to_string_noraw,
};
use crate::readers::blockreader::BlockSz;
use crate::readers::filepreprocessor::{
    fpath_to_filetype,
    PathToFiletypeResult,
};
use crate::readers::helpers::{
    path_to_fpath,
    randomize,
};
use crate::readers::linereader::{
    LineReader,
    LINE_SEARCH_MAX,
    ResultFindLine,
    SummaryLineReader,
};
use crate::tests::common::{
    path_id_generator,
    eprint_file,
    fill,
    encode_utf16be,
    encode_utf16le,
    encode_utf32be,
    encode_utf32le,
    FILETYPE_UTF8,
    NTF_LOG_EMPTY_FPATH,
    NTF_NL_1_PATH,
    NTF_NL_2_PATH,
    NTF_NL_3_PATH,
    NTF_NL_4_PATH,
    NTF_NL_5_PATH,
    NTF_LINE_TOO_LONG_UTF8_FPATH,
    NTF_LINE_TOO_LONG_UTF8_LONG_LINE_OFFSET,
    NTF_SYSLINE_2_PATH,
    FILE_UTF16LE_ABC_FPATH,
    FILE_UTF16LE_ABC_NL_FPATH,
    FILE_UTF16LE_ABC_NL_DEF_FPATH,
    FILE_UTF16LE_ABC_NL_DEF_NL_FPATH,
    FILE_UTF16BE_ABC_FPATH,
    FILE_UTF16BE_ABC_NL_FPATH,
    FILE_UTF16BE_ABC_NL_DEF_FPATH,
    FILE_UTF16BE_ABC_NL_DEF_NL_FPATH,
    FILE_UTF32BE_ABC_FPATH,
    FILE_UTF32BE_ABC_NL_FPATH,
    FILE_UTF32BE_ABC_NL_DEF_FPATH,
    FILE_UTF32BE_ABC_NL_DEF_NL_FPATH,
    FILE_UTF32LE_ABC_FPATH,
    FILE_UTF32LE_ABC_NL_FPATH,
    FILE_UTF32LE_ABC_NL_DEF_FPATH,
    FILE_UTF32LE_ABC_NL_DEF_NL_FPATH,
    FILE_UTF32LE_BOM_ABC_FPATH,
    FILE_UTF32BE_BOM_ABC_FPATH,
    FILE_UTF16LE_BOM_ABC_FPATH,
    FILE_UTF16BE_BOM_ABC_FPATH,
    FILE_UTF8_ABC_FPATH,
    FILE_UTF8_ABC_NL_FPATH,
    FILE_UTF8_ABC_NL_DEF_FPATH,
    FILE_UTF8_ABC_NL_DEF_NL_FPATH,
    FILE_UTF8_BOM_ABC_FPATH,
    FILE_UTF16LE_SYSLINE_2,
};

/// dummy version of `ResultFindLine` for asserting return enum of
/// `LineReader::find_line`
#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq)]
enum ResultFindLine_Test {
    Found,
    Done,
}

// helpful abbreviations
const RS3T_DONE: ResultFindLine_Test = ResultFindLine_Test::Done;
const RS3T_FOUND: ResultFindLine_Test = ResultFindLine_Test::Found;

const UTF_8: FileTypeTextEncoding = FileTypeTextEncoding::Utf8Ascii;
const UTF_8_BOM: FileTypeTextEncoding = FileTypeTextEncoding::Utf8BOM;
const UTF_16LE: FileTypeTextEncoding = FileTypeTextEncoding::Utf16le;
const UTF_16LE_BOM: FileTypeTextEncoding = FileTypeTextEncoding::Utf16leBOM;
const UTF_16BE: FileTypeTextEncoding = FileTypeTextEncoding::Utf16be;
const UTF_16BE_BOM: FileTypeTextEncoding = FileTypeTextEncoding::Utf16beBOM;
const UTF_32LE: FileTypeTextEncoding = FileTypeTextEncoding::Utf32le;
const UTF_32LE_BOM: FileTypeTextEncoding = FileTypeTextEncoding::Utf32leBOM;
const UTF_32BE: FileTypeTextEncoding = FileTypeTextEncoding::Utf32be;
const UTF_32BE_BOM: FileTypeTextEncoding = FileTypeTextEncoding::Utf32beBOM;

// -------------------------------------------------------------------------------------------------

/// Helper to wrap the match and panic checks
fn new_LineReader(
    path: &FPath,
    blocksz: BlockSz,
) -> LineReader {
    let result = fpath_to_filetype(path, true);
    let filetype = match result {
        PathToFiletypeResult::Filetype(ft) => ft,
        PathToFiletypeResult::Archive(..) => {
            panic!("ERROR: fpath_to_filetype({:?}) returned an PathToFiletypeResult::Archive", path);
        }
    };
    match LineReader::new(path_id_generator(), path.clone(), filetype, blocksz) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: LineReader::new({:?}, {}) failed {}", path, blocksz, err);
        }
    }
}

#[test]
fn test_new_LineReader_1() {
    new_LineReader(&NTF_NL_1_PATH, 1024);
}

#[test]
#[should_panic]
fn test_new_LineReader_2_bad_path_panics() {
    new_LineReader(&FPath::from("THIS/PATH_DOES/NOT///EXIST!!!"), 1024);
}

#[cfg(target_family = "unix")]
#[test]
fn test_new_LineReader_no_file_permissions() {
    let ntf = create_temp_file_no_permissions(".log");
    let path = ntf.path();
    let fpath = path_to_fpath(path);
    match LineReader::new(
        path_id_generator(),
        fpath.clone(),
        FILETYPE_UTF8,
        1024,
    ) {
        Ok(_) => {
            panic!("no permissions to read {:?}", path);
        }
        Err(err) => {
            defo!("no permissions to read {:?}", path);
            defo!("error (expected): {}", err);
        }
    }
}

#[test_case(&NTF_NL_1_PATH)]
#[test_case(&NTF_SYSLINE_2_PATH)]
fn test_mtime(path: &FPath) {
    let lr1 = new_LineReader(path, 0x100);
    // merely run the function
    _ = lr1.mtime();
}

// -------------------------------------------------------------------------------------------------

/// loop on `LineReader.find_line` until it is done
/// this is the most straightforward use of `LineReader`
fn process_LineReader(lr1: &mut LineReader) {
    defn!();
    let mut fo1: FileOffset = 0;
    loop {
        defo!("fileoffset {}", fo1);
        let result = lr1.find_line(fo1);
        match result {
            ResultFindLine::Found((fo, lp)) => {
                let count = lr1.count_lines_processed();
                defo!(
                    "ResultFindLine::Found!    FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    fo,
                    count,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_string_noraw()
                );
                fo1 = fo;
                (*lp).print(false);
            }
            ResultFindLine::Done => {
                defo!("ResultFindLine::Done!");
                break;
            }
            ResultFindLine::Err(err) => {
                defo!("ResultFindLine::Err {}", err);
                panic!("ERROR: {}", err);
            }
        }
    }
    defx!();
}

// -----------------------------------------------------------------------------

/// test `LineReader::find_line`
///
/// the `LineReader` instance reads `data`
/// assert the line count
fn do_test_LineReader_count(
    data: &str,
    line_count: usize,
) {
    summary_stats_enable();
    defn!("do_test_LineReader_count(…, {:?})", line_count);
    let blocksz: BlockSz = 64;
    let ntf = create_temp_file(data);
    let path = ntf_fpath(&ntf);
    let mut lr1 = new_LineReader(&path, blocksz);
    let bufnoraw = buffer_to_string_noraw(data.as_bytes());
    defo!("File {:?}", bufnoraw);
    process_LineReader(&mut lr1);
    let lc = lr1.count_lines_processed();
    assert_eq!(line_count as u64, lc, "Expected {} count of lines, found {}", line_count, lc);
    defo!("{:?}", data.as_bytes());
    defx!("do_test_LineReader_count()");
}

#[test]
fn test_LineReader_count0() {
    do_test_LineReader_count("", 0);
}

#[test]
fn test_LineReader_count1_() {
    do_test_LineReader_count(" ", 1);
}

#[test]
fn test_LineReader_count1__() {
    do_test_LineReader_count("  ", 1);
}

#[test]
fn test_LineReader_count1_n() {
    do_test_LineReader_count(" \n", 1);
}

#[test]
fn test_LineReader_count2_n_() {
    do_test_LineReader_count(" \n ", 2);
}

#[test]
fn test_LineReader_count2__n__() {
    do_test_LineReader_count("  \n  ", 2);
}

#[test]
fn test_LineReader_count2_n_n() {
    do_test_LineReader_count(" \n \n", 2);
}

#[test]
fn test_LineReader_count2__n__n() {
    do_test_LineReader_count("  \n  \n", 2);
}

#[test]
fn test_LineReader_count3_n_n_() {
    do_test_LineReader_count(" \n \n ", 3);
}

#[test]
fn test_LineReader_count3__n__n__() {
    do_test_LineReader_count("  \n  \n  ", 3);
}

#[test]
fn test_LineReader_count3__n__n__n() {
    do_test_LineReader_count("  \n  \n  \n", 3);
}

#[test]
fn test_LineReader_count1() {
    do_test_LineReader_count("  \n  \n  \n  ", 4);
}

#[test]
fn test_LineReader_count4__n__n_n__n() {
    do_test_LineReader_count("  \n  \n  \n  \n", 4);
}

#[test]
fn test_LineReader_count4_uhi_n__n__n__n() {
    do_test_LineReader_count("two unicode points é\n  \n  \n  \n", 4);
}

// -------------------------------------------------------------------------------------------------

/// testing helper
/// call `LineReader.find_line` for all `FileOffset` in passed `offsets`
fn find_line_all(
    linereader: &mut LineReader,
    offsets: &Vec<FileOffset>,
) {
    for fo1 in offsets {
        eprintln!("{}LineReader.find_line({})", so(), fo1);
        let result = linereader.find_line(*fo1);
        match result {
            ResultFindLine::Found((fo, lp)) => {
                let _ln = linereader.count_lines_processed();
                eprintln!(
                    "{}ResultFindLine::Found!    FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_string_noraw()
                );
            }
            ResultFindLine::Done => {
                eprintln!("{}ResultFindLine::Done!", so());
            }
            ResultFindLine::Err(err) => {
                eprintln!("{}ResultFindLine::Err {}", so(), err);
                panic!("ERROR: find_line({:?}) {:?}", fo1, err);
            }
        }
    }
}

/// compare contents of a file `path` to contents of `linereader`
/// assert they are the same
/// presumes the linereader has processed the entire file
fn compare_file_linereader(
    path: &FPath,
    linereader: &LineReader,
) {
    eprintln!("{}compare_file_linereader({:?})", sn(), path);
    let contents_file: String = std::fs::read_to_string(path).unwrap();
    let contents_file_count: usize = contents_file.lines().count();
    eprint_file(path);

    let mut buffer_lr: Vec<u8> = Vec::<u8>::with_capacity(contents_file.len() * 2);
    for fo in linereader
        .get_fileoffsets()
        .iter()
    {
        let linep = linereader
            .get_linep(fo)
            .unwrap();
        for slice_ in (*linep).get_slices() {
            for byte_ in slice_.iter() {
                buffer_lr.push(*byte_);
            }
        }
    }
    let contents_lr: String = String::from_utf8_lossy(&buffer_lr).to_string();

    eprintln!(
        "{}contents_file ({} lines):\n───────────────────────\n{}\n───────────────────────\n",
        so(),
        contents_file_count,
        str_to_string_noraw(contents_file.as_str()),
    );

    eprintln!(
        "{}contents_lr ({} lines processed):\n───────────────────────\n{}\n───────────────────────\n",
        so(),
        linereader.count_lines_processed(),
        str_to_string_noraw(contents_lr.as_str()),
    );

    let mut i: usize = 0;
    for lines_file_lr1 in contents_file
        .lines()
        .zip(contents_lr.lines())
    {
        i += 1;
        eprintln!("{}compare {}\n{}{:?}\n{}{:?}\n", so(), i, so(), lines_file_lr1.0, so(), lines_file_lr1.1,);
        assert_eq!(
            lines_file_lr1.0, lines_file_lr1.1,
            "Lines {:?} differ\nFile      : {:?}\nLineReader: {:?}\n",
            i, lines_file_lr1.0, lines_file_lr1.1,
        );
    }
    assert_eq!(
        contents_file_count, i,
        "Expected to compare {} lines, only compared {}",
        contents_file_count, i
    );
    eprintln!("{}compare_file_linereader({:?})", sx(), &path);
}

/// test `LineReader::find_line` read all file offsets
#[allow(non_snake_case)]
fn _test_LineReader_all(
    path: &FPath,
    cache_enabled: bool,
    blocksz: BlockSz,
) {
    stack_offset_set(None);
    eprintln!("{}_test_LineReader_all({:?}, {:?})", sn(), path, blocksz);
    eprint_file(path);
    let mut lr1 = new_LineReader(path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    if !cache_enabled {
        lr1.LRU_cache_disable();
    }
    let fillsz: usize = match lr1.filesz() as usize {
        0 => 1,
        x => x,
    };
    let mut offsets_all = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_all);
    eprintln!("{}offsets_all: {:?}", so(), offsets_all);
    find_line_all(&mut lr1, &offsets_all);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}_test_LineReader_all({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_all0_empty() {
    _test_LineReader_all(&*NTF_LOG_EMPTY_FPATH, true, 0x4);
}

#[test]
fn test_LineReader_all1() {
    let data: &str = "\
test_LineReader_all1 line 1";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    _test_LineReader_all(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all1n() {
    let data: &str = "\
test_LineReader_all1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    _test_LineReader_all(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all2() {
    let data: &str = "\
test_LineReader_all2 line 1
test_LineReader_all2 line 2";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    _test_LineReader_all(&fpath, true, 0xFF);
}

#[test]
fn test_LineReader_all2n() {
    let data: &str = "\
test_LineReader_all2n line 1
test_LineReader_all2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    _test_LineReader_all(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all2n_noLRUcache() {
    let data: &str = "\
test_LineReader_all2n line 1
test_LineReader_all2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    _test_LineReader_all(&fpath, false, 0x4);
}

#[test]
fn test_LineReader_all3_empty() {
    _test_LineReader_all(&NTF_NL_3_PATH, true, 0x4);
}

#[test]
fn test_LineReader_all3() {
    let data: &str = "\
test_LineReader_all3 line 1
test_LineReader_all3 line 2
test_LineReader_all3 line 3";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    _test_LineReader_all(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all3_noLRUcache() {
    let data: &str = "\
test_LineReader_all3 line 1
test_LineReader_all3 line 2
test_LineReader_all3 line 3";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    _test_LineReader_all(&fpath, false, 0x4);
}

#[test]
fn test_LineReader_all3n() {
    let data: &str = "\
test_LineReader_all3n line 1
test_LineReader_all3n line 2
test_LineReader_all3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    _test_LineReader_all(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all3n_noLRUcache() {
    let data: &str = "\
test_LineReader_all3n line 1
test_LineReader_all3n line 2
test_LineReader_all3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    _test_LineReader_all(&fpath, false, 0x4);
}

/// test `LineReader::find_line` read all file offsets but in reverse
#[allow(non_snake_case)]
fn test_LineReader_all_reversed(
    path: &FPath,
    cache_enabled: bool,
    blocksz: BlockSz,
) {
    stack_offset_set(None);
    eprintln!("{}test_LineReader_all_reversed({:?}, {:?})", sn(), &path, blocksz);
    let mut lr1 = new_LineReader(path, blocksz);
    if !cache_enabled {
        lr1.LRU_cache_disable();
    }
    eprintln!("{}LineReader {:?}", so(), lr1);
    let fillsz: usize = match lr1.filesz() as usize {
        0 => 1,
        x => x,
    };
    let mut offsets_all_rev = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_all_rev);
    offsets_all_rev.sort_by(|a, b| b.cmp(a));

    eprintln!("{}offsets_all_rev: {:?}", so(), offsets_all_rev);
    find_line_all(&mut lr1, &offsets_all_rev);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}test_LineReader_all_reversed({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_all_reversed0_empty() {
    test_LineReader_all_reversed(&*NTF_LOG_EMPTY_FPATH, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed1() {
    let data: &str = "\
test_LineReader_all_reversed1 line 1";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_all_reversed(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed1n() {
    let data: &str = "\
test_LineReader_all_reversed1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_all_reversed(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed2() {
    let data: &str = "\
test_LineReader_all_reversed2 line 1
test_LineReader_all_reversed2 line 2";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_all_reversed(&fpath, true, 0xFF);
}

#[test]
fn test_LineReader_all_reversed2n() {
    let data: &str = "\
test_LineReader_all_reversed2n line 1
test_LineReader_all_reversed2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_all_reversed(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed3_empty() {
    test_LineReader_all_reversed(&NTF_NL_3_PATH, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed3() {
    let data: &str = "\
test_LineReader_all_reversed3 line 1
test_LineReader_all_reversed3 line 2
test_LineReader_all_reversed3 line 3";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_all_reversed(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed3n() {
    let data: &str = "\
test_LineReader_all_reversed3n line 1
test_LineReader_all_reversed3n line 2
test_LineReader_all_reversed3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_all_reversed(&fpath, true, 0x4);
}

#[test]
fn test_LineReader_all_reversed3n_noLRUcache() {
    let data: &str = "\
test_LineReader_all_reversed3n line 1
test_LineReader_all_reversed3n line 2
test_LineReader_all_reversed3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_all_reversed(&fpath, false, 0x4);
}

/// test `LineReader::find_line` read all file offsets but only the even ones
#[allow(non_snake_case)]
fn test_LineReader_half_even(
    path: &FPath,
    blocksz: BlockSz,
) {
    eprintln!("{}test_LineReader_half_even({:?}, {:?})", sn(), &path, blocksz);
    let mut lr1 = new_LineReader(path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    let fillsz: usize = match lr1.filesz() as usize {
        0 => 1,
        x => x,
    };
    let mut offsets_half_even = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_half_even);
    offsets_half_even.retain(|x| *x % 2 == 0);

    eprintln!("{}offsets_half: {:?}", so(), offsets_half_even);
    find_line_all(&mut lr1, &offsets_half_even);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}test_LineReader_half_even({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_half_even_0_empty() {
    test_LineReader_half_even(&*NTF_LOG_EMPTY_FPATH, 0x4);
}

#[test]
fn test_LineReader_half_even_1() {
    let data: &str = "\
test_LineReader_half_even_1 line 1";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_even(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_even_1n() {
    let data: &str = "\
test_LineReader_half_even_1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_even(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_even_2() {
    let data: &str = "\
test_LineReader_half_even_2 line 1
test_LineReader_half_even_2 line 2";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_even(&fpath, 0xFF);
}

#[test]
fn test_LineReader_half_even_2n() {
    let data: &str = "\
test_LineReader_half_even_2n line 1
test_LineReader_half_even_2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_even(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_even_4_sparse1_0x4() {
    let data: &str = "a\nb\nc\nd";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_even(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_even_4_sparse2_0x4() {
    let data: &str = "\na\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_even(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_even_4_sparse2_0x6() {
    let data: &str = "\na\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_even(&fpath, 0x6);
}

#[test]
fn test_LineReader_half_even_4_sparse2_0x8() {
    let data: &str = "\na\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_even(&fpath, 0x8);
}

#[test]
fn test_LineReader_half_even_4_sparse2_0xA() {
    let data: &str = "\na\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_even(&fpath, 0xA);
}

#[test]
fn test_LineReader_half_even_4_sparse2_0x5() {
    let data: &str = "\na\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_even(&fpath, 0x5);
}

#[test]
fn test_LineReader_half_even_3() {
    let data: &str = "\
test_LineReader_half_even_3 line 1
test_LineReader_half_even_3 line 2
test_LineReader_half_even_3 line 3";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_even(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_even_3n() {
    let data: &str = "\
test_LineReader_half_even_3n line 1
test_LineReader_half_even_3n line 2
test_LineReader_half_even_3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_even(&fpath, 0x4);
}

/// test `LineReader::find_line` read all file offsets but only the even ones
#[allow(non_snake_case)]
fn test_LineReader_half_odd(
    path: &FPath,
    blocksz: BlockSz,
) {
    eprintln!("{}test_LineReader_half_odd({:?}, {:?})", sn(), &path, blocksz);
    let mut lr1 = new_LineReader(path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    let fillsz: usize = match lr1.filesz() as usize {
        0 => 1,
        x => x,
    };
    let mut offsets_half_odd = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_half_odd);
    offsets_half_odd.retain(|x| *x % 2 != 0);

    eprintln!("{}offsets_half: {:?}", so(), offsets_half_odd);
    find_line_all(&mut lr1, &offsets_half_odd);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}test_LineReader_half_odd({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_half_odd_0_empty() {
    test_LineReader_half_odd(&*NTF_LOG_EMPTY_FPATH, 0x4);
}

#[test]
fn test_LineReader_half_odd_1() {
    let data: &str = "\
test_LineReader_half_odd_1 line 1";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_odd(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_odd_1n() {
    let data: &str = "\
test_LineReader_half_odd_1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_odd(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_odd_2() {
    let data: &str = "\
test_LineReader_half_odd_2 line 1
test_LineReader_half_odd_2 line 2";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_odd(&fpath, 0xFF);
}

#[test]
fn test_LineReader_half_odd_2n() {
    let data: &str = "\
test_LineReader_half_odd_2n line 1
test_LineReader_half_odd_2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_odd(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_odd_3_sparse1() {
    let data: &str = "a\nb\nc\nd ";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_odd(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_odd_3() {
    let data: &str = "\
test_LineReader_half_odd_3 line 1
test_LineReader_half_odd_3 line 2
test_LineReader_half_odd_3 line 3";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_odd(&fpath, 0x4);
}

#[test]
fn test_LineReader_half_odd_3n() {
    let data: &str = "\
test_LineReader_half_odd_3n line 1
test_LineReader_half_odd_3n line 2
test_LineReader_half_odd_3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_half_odd(&fpath, 0x4);
}

/// test `LineReader::find_line` read all file offsets but in random order
/// TODO: `randomize` should be predictable
#[allow(non_snake_case)]
fn test_LineReader_rand(
    path: &FPath,
    blocksz: BlockSz,
) {
    stack_offset_set(None);
    eprintln!("{}test_LineReader_rand({:?}, {:?})", sn(), &path, blocksz);

    let mut lr1 = new_LineReader(path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    let fillsz: usize = match lr1.filesz() as usize {
        0 => 1,
        x => x,
    };
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_rand);
    eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);
    randomize(&mut offsets_rand);
    offsets_rand.insert(0, 0);
    eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);

    find_line_all(&mut lr1, &offsets_rand);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}test_LineReader_rand({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_rand0_empty() {
    test_LineReader_rand(&*NTF_LOG_EMPTY_FPATH, 0x4);
}

#[test]
fn test_LineReader_rand1() {
    let data: &str = "\
test_LineReader_rand1 line 1";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_rand(&fpath, 0x4);
}

#[test]
fn test_LineReader_rand1n() {
    let data: &str = "\
test_LineReader_rand1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_rand(&fpath, 0x4);
}

#[test]
fn test_LineReader_rand2() {
    let data: &str = "\
test_LineReader_rand2 line 1
test_LineReader_rand2 line 2";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_rand(&fpath, 0xFF);
}

#[test]
fn test_LineReader_rand2n() {
    let data: &str = "\
test_LineReader_rand2n line 1
test_LineReader_rand2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_rand(&fpath, 0x4);
}

#[test]
fn test_LineReader_rand3_nl3() {
    test_LineReader_rand(&NTF_NL_3_PATH, 0x4);
}

#[test]
fn test_LineReader_rand3() {
    let data: &str = "\
test_LineReader_rand3 line 1
test_LineReader_rand3 line 2
test_LineReader_rand3 line 3";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_rand(&fpath, 0x4);
}

#[test]
fn test_LineReader_rand3n() {
    let data: &str = "\
test_LineReader_rand3n line 1
test_LineReader_rand3n line 2
test_LineReader_rand3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_LineReader_rand(&fpath, 0x4);
}

// -------------------------------------------------------------------------------------------------

/// test `LineReader::find_line` read all file offsets in a precise order
#[allow(non_snake_case)]
fn test_LineReader_precise_order(
    path: &FPath,
    cache_enabled: bool,
    blocksz: BlockSz,
    offsets: &Vec<FileOffset>,
) {
    stack_offset_set(None);
    eprintln!("{}test_LineReader_rand({:?}, {:?}, {:?})", sn(), &path, blocksz, offsets);
    eprint_file(path);
    let mut lr1: LineReader = new_LineReader(path, blocksz);
    if !cache_enabled {
        lr1.LRU_cache_disable();
    }

    find_line_all(&mut lr1, offsets);

    eprintln!("\n{}{:?}\n", so(), lr1);
    for (fo, linep) in lr1.lines.iter() {
        eprintln!("{}  Line@{:02}: {:?}", so(), fo, linep);
        let slices_ = (*linep).get_slices();
        for (count_, slice_) in slices_.iter().enumerate() {
            eprintln!("{}    LinePart {}: {:?}", so(), count_, buffer_to_string_noraw(slice_));
        }
    }

    compare_file_linereader(path, &lr1);

    eprintln!("{}test_LineReader_rand({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_precise_order_2__0_44__0xF() {
    let data: &str = "\
test_LineReader_precise_order_2 line 1 of 2
test_LineReader_precise_order_2 line 2 of 2
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let offsets: Vec<FileOffset> = vec![0, 44];
    test_LineReader_precise_order(&fpath, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_2__0_44__0xFF() {
    let data: &str = "\
test_LineReader_precise_order_2 line 1 of 2
test_LineReader_precise_order_2 line 2 of 2
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let offsets: Vec<FileOffset> = vec![0, 44];
    test_LineReader_precise_order(&fpath, true, 0xFF, &offsets);
}

#[test]
fn test_LineReader_precise_order_2__44_0() {
    let data: &str = "\
test_LineReader_precise_order_2 line 1 of 2
test_LineReader_precise_order_2 line 2 of 2
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let offsets: Vec<FileOffset> = vec![44, 0];
    test_LineReader_precise_order(&fpath, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty0__0_1() {
    let offsets: Vec<FileOffset> = vec![0, 1];
    test_LineReader_precise_order(&*NTF_LOG_EMPTY_FPATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl1__0_1() {
    let offsets: Vec<FileOffset> = vec![0, 1];
    test_LineReader_precise_order(&NTF_NL_1_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__0_1_2() {
    let offsets: Vec<FileOffset> = vec![0, 1, 2];
    test_LineReader_precise_order(&NTF_NL_3_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__0_2_1() {
    let offsets: Vec<FileOffset> = vec![0, 2, 1];
    test_LineReader_precise_order(&NTF_NL_3_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__1_2_0() {
    let offsets: Vec<FileOffset> = vec![1, 2, 0];
    test_LineReader_precise_order(&NTF_NL_3_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__1_0_2() {
    let offsets: Vec<FileOffset> = vec![1, 0, 2];
    test_LineReader_precise_order(&NTF_NL_3_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__2_0_1() {
    let offsets: Vec<FileOffset> = vec![2, 0, 1];
    test_LineReader_precise_order(&NTF_NL_3_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__2_1_0() {
    let offsets: Vec<FileOffset> = vec![2, 1, 0];
    test_LineReader_precise_order(&NTF_NL_3_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__1_0_2_1_2() {
    let offsets: Vec<FileOffset> = vec![1, 0, 2, 1, 2];
    test_LineReader_precise_order(&NTF_NL_3_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__1_2_1_2_0() {
    let offsets: Vec<FileOffset> = vec![1, 2, 1, 2, 0];
    test_LineReader_precise_order(&NTF_NL_3_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__0_1_2_2() {
    let offsets: Vec<FileOffset> = vec![0, 1, 2, 2];
    test_LineReader_precise_order(&NTF_NL_3_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__0_2_1_1() {
    let offsets: Vec<FileOffset> = vec![0, 2, 1, 1];
    test_LineReader_precise_order(&NTF_NL_3_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl3__1_2_0_0() {
    let offsets: Vec<FileOffset> = vec![1, 2, 0, 0];
    test_LineReader_precise_order(&NTF_NL_3_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl4__0_1_2_3() {
    let offsets: Vec<FileOffset> = vec![0, 1, 2, 3];
    test_LineReader_precise_order(&NTF_NL_4_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl4__1_2_3_0() {
    let offsets: Vec<FileOffset> = vec![1, 2, 3, 0];
    test_LineReader_precise_order(&NTF_NL_4_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl4__2_3_0_1() {
    let offsets: Vec<FileOffset> = vec![2, 3, 0, 1];
    test_LineReader_precise_order(&NTF_NL_4_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl4__3_0_1_2() {
    let offsets: Vec<FileOffset> = vec![3, 0, 1, 2];
    test_LineReader_precise_order(&NTF_NL_4_PATH, true, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_nl4__3_0_1_2_3_0_1_2__noLRUcache() {
    let offsets: Vec<FileOffset> = vec![
        3, 0, 1, 2, 3, 0, 1, 2,
    ];
    test_LineReader_precise_order(&NTF_NL_4_PATH, false, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_3__0_88_44() {
    let data: &str = "\
test_LineReader_precise_order_3 line 1 of 3
test_LineReader_precise_order_3 line 2 of 3
test_LineReader_precise_order_3 line 3 of 3
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let offsets: Vec<FileOffset> = vec![0, 88, 44];
    test_LineReader_precise_order(&fpath, true, 0x8, &offsets);
}

#[test]
fn test_LineReader_precise_order_3__0_100_50() {
    let data: &str = "\
test_LineReader_precise_order_3 line 1 of 3
test_LineReader_precise_order_3 line 2 of 3
test_LineReader_precise_order_3 line 3 of 3
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let offsets: Vec<FileOffset> = vec![0, 100, 50];
    test_LineReader_precise_order(&fpath, true, 0x8, &offsets);
}

#[test]
fn test_LineReader_precise_order_3__50_0_100() {
    let data: &str = "\
test_LineReader_precise_order_3 line 1 of 3
test_LineReader_precise_order_3 line 2 of 3
test_LineReader_precise_order_3 line 3 of 3
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let offsets: Vec<FileOffset> = vec![50, 0, 100];
    test_LineReader_precise_order(&fpath, true, 0x8, &offsets);
}

#[test]
fn test_LineReader_precise_order_3__100_50_0() {
    let data: &str = "\
test_LineReader_precise_order_3 line 1 of 3
test_LineReader_precise_order_3 line 2 of 3
test_LineReader_precise_order_3 line 3 of 3
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let offsets: Vec<FileOffset> = vec![100, 50, 0];
    test_LineReader_precise_order(&fpath, true, 0x8, &offsets);
}

// -------------------------------------------------------------------------------------------------

/// call `LineReader.find_line_in_block` for all `FileOffset` in passed
/// `offsets`
fn find_line_in_block_all(
    linereader: &mut LineReader,
    offsets: &Vec<FileOffset>,
) {
    for fo1 in offsets {
        defo!("LineReader.find_line_in_block({})", fo1);
        let result = linereader.find_line_in_block(*fo1);
        match result {
            (ResultFindLine::Found((fo, lp)), partial) => {
                let _ln = linereader.count_lines_processed();
                defo!(
                    "ResultFindLine::Found!    FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_string_noraw()
                );
                assert!(partial.is_none());
            }
            (ResultFindLine::Done, partial) => {
                defo!("ResultFindLine::Done! partial {:?}", partial);
            }
            (ResultFindLine::Err(err), _) => {
                defo!("ResultFindLine::Err {}", err);
                panic!("ERROR: find_line({:?}) {:?}", fo1, err);
            }
        }
    }
}

/// test `LineReader::find_line_in_block` read all file offsets
fn test_find_line_in_block_all(
    path: &FPath,
    cache_enabled: bool,
    blocksz: BlockSz,
) {
    stack_offset_set(None);
    eprintln!("{}test_find_line_in_block_all({:?}, {:?})", sn(), path, blocksz);
    eprint_file(path);
    let mut lr1 = new_LineReader(path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    if !cache_enabled {
        lr1.LRU_cache_disable();
    }
    let fillsz: usize = match lr1.filesz() as usize {
        0 => 1,
        x => x,
    };
    let mut offsets_all = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_all);
    eprintln!("{}offsets_all: {:?}", so(), offsets_all);
    find_line_in_block_all(&mut lr1, &offsets_all);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}test_find_line_in_block_all({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_find_line_in_block_all_empty0() {
    test_find_line_in_block_all(&*NTF_LOG_EMPTY_FPATH, true, 0xF);
}

#[test]
fn test_find_line_in_block_all_empty0_nocache() {
    test_find_line_in_block_all(&*NTF_LOG_EMPTY_FPATH, false, 0xF);
}

#[test]
fn test_find_line_in_block_all_nl1() {
    test_find_line_in_block_all(&NTF_NL_1_PATH, true, 0xF);
}

#[test]
fn test_find_line_in_block_all_nl2() {
    test_find_line_in_block_all(&NTF_NL_2_PATH, true, 0xF);
}

#[test]
fn test_find_line_in_block_all_nl3() {
    test_find_line_in_block_all(&NTF_NL_3_PATH, true, 0xF);
}

#[test]
fn test_find_line_in_block_all_nl4() {
    test_find_line_in_block_all(&NTF_NL_4_PATH, true, 0xF);
}

#[test]
fn test_find_line_in_block_all_nl5() {
    test_find_line_in_block_all(&NTF_NL_5_PATH, true, 0xF);
}

#[test]
fn test_find_line_in_block_all_nl2_4() {
    test_find_line_in_block_all(&NTF_NL_2_PATH, true, 4);
}

#[test]
fn test_find_line_in_block_all_nl3_4() {
    test_find_line_in_block_all(&NTF_NL_3_PATH, true, 4);
}

#[test]
fn test_find_line_in_block_all_nl4_4() {
    test_find_line_in_block_all(&NTF_NL_4_PATH, true, 4);
}

#[test]
fn test_find_line_in_block_all_nl5_4() {
    test_find_line_in_block_all(&NTF_NL_5_PATH, true, 4);
}

#[test]
fn test_find_line_in_block_all_5_8() {
    let data: &str = "a\nb\nc\nd\ne\n";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_find_line_in_block_all(&fpath, true, 8);
}

#[test]
fn test_find_line_in_block_all_5_4() {
    let data: &str = "a\nb\nc\nd\ne\n";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    test_find_line_in_block_all(&fpath, true, 4);
}

// -------------------------------------------------------------------------------------------------

/// each entry is
/// `(find_line_in_block fileoffset, (Result Expect, Partial Value Expect, Value Expect)`
type TestFindLineInBlockCheck = Vec<(FileOffset, (ResultFindLine_Test, Option<&'static str>), String)>;

/// test `LineReader::find_line_in_block` reads passed file offsets
#[allow(non_snake_case)]
fn test_find_line_in_block(
    path: &FPath,
    cache_enabled: bool,
    blocksz: BlockSz,
    encoding: FileTypeTextEncoding,
    in_out: &TestFindLineInBlockCheck,
) {
    stack_offset_set(Some(2));
    eprintln!(
        "test_find_line_in_block({:?}, {:?}, {:?}, {:?})",
        &path,
        cache_enabled,
        blocksz,
        in_out
    );
    eprint_file(path);
    let mut lr1: LineReader = new_LineReader(path, blocksz);
    if !cache_enabled {
        lr1.LRU_cache_disable();
    }
    lr1.filetype_text_encoding_update(encoding);

    let mut i = 0;
    for (fo_in, (rs4_expect, partial_expect), str_expect) in in_out.iter() {
        i += 1;
        eprintln!("LineReader.find_line_in_block({}) (check {i})", fo_in);
        eprintln!("Blocksz       {:?}", blocksz);
        eprintln!("expect Result {:?}", rs4_expect);
        eprintln!("expect Value  {:?}", str_expect);
        eprintln!("expect Partial Result {:?}", partial_expect);
        let result = lr1.find_line_in_block(*fo_in);
        match result {
            (ResultFindLine::Found((fo, lp)), partial_actual) => {
                let _ln = lr1.count_lines_processed();
                eprintln!(
                    "{}ResultFindLine::Found! (check {i})    FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_string_noraw()
                );
                let str_actual = (*lp).to_String();
                assert_eq!(
                    &str_actual, str_expect,
                    "find_line_in_block({}) (check {i})\nexpect {:?}\nactual {:?}\n",
                    *fo_in, str_expect, str_actual,
                );
                assert_eq!(rs4_expect, &RS3T_FOUND, "Expected {:?}, got Found (check {i})", rs4_expect);
                assert!(partial_actual.is_none(), "unexpected partial for result Found (check {i})");
                assert!(partial_expect.is_none(), "bad test check for partial (check {i})");
            }
            (ResultFindLine::Done, partial_actual) => {
                eprintln!("ResultFindLine::Done, partial_actual={:?}", partial_actual);
                assert_eq!(
                    &"",
                    &str_expect.as_str(),
                    "find_line_in_block({}) returned Done (check {i})\nexpected value {:?}\ngot value      \"\"\n",
                    *fo_in,
                    str_expect,
                );
                assert_eq!(rs4_expect, &RS3T_DONE, "Expected {:?}, got Done (check {i})", rs4_expect);
                match partial_actual {
                    Some(line) => {
                        assert!(partial_expect.is_some(),
                            "expected partial None but actual partial is Some(line: {:?}) (check {i})",
                            line.to_string_noraw(),
                        );
                        let sa = line.to_String();
                        let se = partial_expect.unwrap();
                        assert_eq!(sa.as_str(), se,
                            "\n  expected partial {:?}\n  actual {:?} (check {i})\n",
                            se, sa,
                        );
                    }
                    None => {
                        assert!(partial_expect.is_none(), "result partial is None but expected {:?} (check {i})", partial_expect);
                    }
                }
            }
            (ResultFindLine::Err(err), _) => {
                eprintln!("ResultFindLine::Err {} (check {i})", err);
                panic!("ERROR: find_line_in_block({:?}) {:?} (check {i})", fo_in, err);
            }
        }
        eprintln!("\n");
    }

    eprintln!("\n{:?}\n", lr1);
}

#[test]
fn test_find_line_in_block_empty0_bszFF() {
    let in_out: TestFindLineInBlockCheck = vec![(0, (RS3T_DONE, None), ES.clone())];
    test_find_line_in_block(&*NTF_LOG_EMPTY_FPATH, true, 0xFF, UTF_8, &in_out);
}

#[test]
fn test_find_line_in_block_nl1_bszFF() {
    let in_out: TestFindLineInBlockCheck = vec![
        (0, (RS3T_FOUND, None), String::from("\n")),
        (1, (RS3T_DONE, None), ES.clone()),
    ];
    test_find_line_in_block(&NTF_NL_1_PATH, true, 0xFF, UTF_8, &in_out);
}

#[test]
fn test_find_line_in_block_nl2_bszFF() {
    let in_out: TestFindLineInBlockCheck = vec![
        (0, (RS3T_FOUND, None), String::from("\n")),
        (1, (RS3T_FOUND, None), String::from("\n")),
        (2, (RS3T_DONE, None), ES.clone()),
    ];
    test_find_line_in_block(&NTF_NL_2_PATH, true, 0xFF, UTF_8, &in_out);
}

#[test]
fn test_find_line_in_block_1_bszFF() {
    let data: &str = "abcdef";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let in_out: TestFindLineInBlockCheck = vec![
        (0, (RS3T_FOUND, None), String::from("abcdef")),
        (1, (RS3T_FOUND, None), String::from("abcdef")),
        (2, (RS3T_FOUND, None), String::from("abcdef")),
        (3, (RS3T_FOUND, None), String::from("abcdef")),
        (4, (RS3T_FOUND, None), String::from("abcdef")),
        (5, (RS3T_FOUND, None), String::from("abcdef")),
        (6, (RS3T_DONE, None), ES.clone()),
        (7, (RS3T_DONE, None), ES.clone()),
    ];
    test_find_line_in_block(&fpath, true, 0xFF, UTF_8, &in_out);
}

#[test]
fn test_find_line_in_block_2_bszFF() {
    let data: &str = "a\nb";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let in_out: TestFindLineInBlockCheck = vec![
        (0, (RS3T_FOUND, None), String::from("a\n")),
        (1, (RS3T_FOUND, None), String::from("a\n")),
        (2, (RS3T_FOUND, None), String::from("b")),
        (3, (RS3T_DONE, None), ES.clone()),
    ];
    test_find_line_in_block(&fpath, true, 0xFF, UTF_8, &in_out);
}

#[test]
fn test_find_line_in_block_3_bszFF() {
    let data: &str = "a\nb\nc";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let in_out: TestFindLineInBlockCheck = vec![
        (0, (RS3T_FOUND, None), String::from("a\n")),
        (1, (RS3T_FOUND, None), String::from("a\n")),
        (2, (RS3T_FOUND, None), String::from("b\n")),
        (3, (RS3T_FOUND, None), String::from("b\n")),
        (4, (RS3T_FOUND, None), String::from("c")),
        (5, (RS3T_DONE, None), ES.clone()),
    ];
    test_find_line_in_block(&fpath, true, 0xFF, UTF_8, &in_out);
}

#[test]
fn test_find_line_in_block_4_bszFF() {
    let data: &str = "a\nb\nc\nd\n";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let in_out: TestFindLineInBlockCheck = vec![
        (0, (RS3T_FOUND, None), String::from("a\n")),
        (1, (RS3T_FOUND, None), String::from("a\n")),
        (2, (RS3T_FOUND, None), String::from("b\n")),
        (3, (RS3T_FOUND, None), String::from("b\n")),
        (4, (RS3T_FOUND, None), String::from("c\n")),
        (5, (RS3T_FOUND, None), String::from("c\n")),
        (6, (RS3T_FOUND, None), String::from("d\n")),
        (7, (RS3T_FOUND, None), String::from("d\n")),
        (8, (RS3T_DONE, None), ES.clone()),
    ];
    test_find_line_in_block(&fpath, true, 0xFF, UTF_8, &in_out);
}

#[test]
fn test_find_line_in_block_4x2_bsz4() {
    let data: &str = "abc\ndef\n";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let in_out: TestFindLineInBlockCheck = vec![
        (0, (RS3T_FOUND, None), String::from("abc\n")),
        (1, (RS3T_FOUND, None), String::from("abc\n")),
        (2, (RS3T_FOUND, None), String::from("abc\n")),
        (3, (RS3T_FOUND, None), String::from("abc\n")),
        (4, (RS3T_FOUND, None), String::from("def\n")),
        (5, (RS3T_FOUND, None), String::from("def\n")),
        (6, (RS3T_FOUND, None), String::from("def\n")),
        (7, (RS3T_FOUND, None), String::from("def\n")),
    ];
    test_find_line_in_block(&fpath, true, 4, UTF_8, &in_out);
}

#[test]
fn test_find_line_in_block_4x2_bsz5() {
    let data: &str = "abc\ndef\n";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let in_out: TestFindLineInBlockCheck = vec![
        (0, (RS3T_FOUND, None), String::from("abc\n")),
        (1, (RS3T_FOUND, None), String::from("abc\n")),
        (2, (RS3T_FOUND, None), String::from("abc\n")),
        (3, (RS3T_FOUND, None), String::from("abc\n")),
        (4, (RS3T_FOUND, None), String::from("def\n")),
        (5, (RS3T_FOUND, None), String::from("def\n")),
        (6, (RS3T_FOUND, None), String::from("def\n")),
        (7, (RS3T_FOUND, None), String::from("def\n")),
    ];
    test_find_line_in_block(&fpath, true, 5, UTF_8, &in_out);
}

// -------------------------------------------------------------------------------------------------

type TestLineGetBoxPtrsCheck = Vec<(FileOffset, (LineIndex, LineIndex), Bytes)>;

/// test `Line.get_boxptrs`
/// assert result equals passed `checks`
fn test_Line_get_boxptrs(
    path: &FPath,
    blocksz: BlockSz,
    checks: &TestLineGetBoxPtrsCheck,
) {
    let fn_: &str = "test_Line_get_boxptrs";
    eprintln!("{}{}({:?}, {}, checks)", sn(), fn_, path, blocksz);
    // create a `LineReader` and read all the lines in the file
    let mut lr = new_LineReader(path, blocksz);
    let mut fo: FileOffset = 0;
    loop {
        match lr.find_line(fo) {
            ResultFindLine::Found((fo_, _)) => {
                fo = fo_;
            }
            ResultFindLine::Done => {
                break;
            }
            ResultFindLine::Err(err) => {
                panic!("LineReader::new({:?}, {:?}) ResultFindLine::Err {}", path, blocksz, err);
            }
        }
    }

    // then test the `Line.get_boxptrs`
    for (fileoffset, (a, b), bytes_check) in checks.iter() {
        assert_le!(a, b, "bad check args a {} b {}", a, b);
        assert_ge!(
            b - a,
            bytes_check.len(),
            "Bad check args ({}-{})={} < {} bytes_check.len()",
            b,
            a,
            b - a,
            bytes_check.len()
        );
        eprintln!("{}{}: linereader.get_linep({})", so(), fn_, fileoffset);
        // get the `LineP` at `fileoffset`
        let linep: LineP = lr
            .get_linep(fileoffset)
            .unwrap();
        eprintln!("{}{}: returned {:?}", so(), fn_, (*linep).to_string_noraw());
        eprintln!("{}{}: line.get_boxptrs({}, {})", so(), fn_, a, b);
        let boxptrs = match (*linep).get_boxptrs(*a, *b) {
            LinePartPtrs::NoPtr => {
                assert!(
                    bytes_check.is_empty(),
                    "Expected bytes_check {:?}, received NoPtr (no bytes)",
                    bytes_check
                );
                continue;
            }
            LinePartPtrs::SinglePtr(box_) => {
                vec![box_]
            }
            LinePartPtrs::DoublePtr(box1, box2) => {
                vec![box1, box2]
            }
            LinePartPtrs::MultiPtr(boxes) => boxes,
        };
        // check the results, comparing byte by byte
        let mut at: usize = 0;
        for boxptr in boxptrs.iter() {
            for byte_ in (*boxptr).iter() {
                let byte_check = &bytes_check[at];
                eprintln!(
                    "{}{}: {:3?} ≟ {:3?} ({:?} ≟ {:?})",
                    so(),
                    fn_,
                    byte_,
                    byte_check,
                    byte_to_char_noraw(*byte_),
                    byte_to_char_noraw(*byte_check)
                );
                assert_eq!(byte_, byte_check, "byte {} from boxptr {:?} ≠ {:?} ({:?} ≠ {:?}) check value; returned boxptr segment {:?} Line {:?}", at, byte_, byte_check, byte_to_char_noraw(*byte_), byte_to_char_noraw(*byte_check), buffer_to_string_noraw(boxptr), (*linep).to_string_noraw());
                at += 1;
            }
        }
    }
    eprintln!("{}{}", sx(), fn_);
}

#[test]
fn test_Line_get_boxptrs_1() {
    let data: &str = "\
this is line 1";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let checks: TestLineGetBoxPtrsCheck = vec![(0, (0, 1), vec![b't'])];
    test_Line_get_boxptrs(&fpath, 0xFF, &checks);
}

/// for given `blocksz`, get `LineReader.get_boxptrs` for a predetermined
/// inputs and assert the comparison to `checks` outputs
fn test_Line_get_boxptrs_2_(blocksz: BlockSz) {
    eprintln!("{}test_Line_get_boxptrs_2_({:?})", sn(), blocksz);
    let data: &str = "\
One 1
Two 2
";
    let ntf = create_temp_file(data);
    let fpath = ntf_fpath(&ntf);
    let checks: TestLineGetBoxPtrsCheck = vec![
        // fileoffset, (a, b), check
        //
        (0, (0, 1), vec![b'O']),
        (0, (0, 2), vec![b'O', b'n']),
        (0, (0, 3), vec![b'O', b'n', b'e']),
        (
            0,
            (0, 4),
            vec![
                b'O', b'n', b'e', b' ',
            ],
        ),
        (
            0,
            (0, 5),
            vec![
                b'O', b'n', b'e', b' ', b'1',
            ],
        ),
        (
            0,
            (0, 6),
            vec![
                b'O', b'n', b'e', b' ', b'1', b'\n',
            ],
        ),
        //
        (0, (1, 2), vec![b'n']),
        (0, (1, 3), vec![b'n', b'e']),
        (0, (1, 4), vec![b'n', b'e', b' ']),
        (
            0,
            (1, 5),
            vec![
                b'n', b'e', b' ', b'1',
            ],
        ),
        (
            0,
            (1, 6),
            vec![
                b'n', b'e', b' ', b'1', b'\n',
            ],
        ),
        //
        (0, (2, 3), vec![b'e']),
        (0, (2, 4), vec![b'e', b' ']),
        (0, (2, 5), vec![b'e', b' ', b'1']),
        (
            0,
            (2, 6),
            vec![
                b'e', b' ', b'1', b'\n',
            ],
        ),
        //
        (0, (3, 4), vec![b' ']),
        (0, (3, 5), vec![b' ', b'1']),
        (0, (3, 6), vec![b' ', b'1', b'\n']),
        //
        (0, (4, 5), vec![b'1']),
        (0, (4, 6), vec![b'1', b'\n']),
        //
        (0, (5, 5), vec![]),
        (0, (5, 6), vec![b'\n']),
        //
        (0, (6, 6), vec![]),
        //
        (1, (0, 1), vec![b'O']),
        (2, (0, 2), vec![b'O', b'n']),
        (3, (0, 3), vec![b'O', b'n', b'e']),
        (
            4,
            (0, 4),
            vec![
                b'O', b'n', b'e', b' ',
            ],
        ),
        (
            5,
            (0, 5),
            vec![
                b'O', b'n', b'e', b' ', b'1',
            ],
        ),
        (
            5,
            (0, 6),
            vec![
                b'O', b'n', b'e', b' ', b'1', b'\n',
            ],
        ),
        //
        (6, (0, 1), vec![b'T']),
        (6, (0, 2), vec![b'T', b'w']),
        (7, (0, 2), vec![b'T', b'w']),
        (
            7,
            (0, 5),
            vec![
                b'T', b'w', b'o', b' ', b'2',
            ],
        ),
        (
            8,
            (0, 6),
            vec![
                b'T', b'w', b'o', b' ', b'2', b'\n',
            ],
        ),
        (
            8,
            (0, 7),
            vec![
                b'T', b'w', b'o', b' ', b'2', b'\n',
            ],
        ),
        (
            9,
            (0, 6),
            vec![
                b'T', b'w', b'o', b' ', b'2', b'\n',
            ],
        ),
        (
            10,
            (0, 6),
            vec![
                b'T', b'w', b'o', b' ', b'2', b'\n',
            ],
        ),
        (
            10,
            (1, 6),
            vec![
                b'w', b'o', b' ', b'2', b'\n',
            ],
        ),
        (
            10,
            (2, 6),
            vec![
                b'o', b' ', b'2', b'\n',
            ],
        ),
        (10, (3, 6), vec![b' ', b'2', b'\n']),
        (10, (4, 6), vec![b'2', b'\n']),
        (10, (5, 6), vec![b'\n']),
    ];
    test_Line_get_boxptrs(&fpath, blocksz, &checks);
    eprintln!("{}test_Line_get_boxptrs_2_({:?})", sx(), blocksz);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xF() {
    test_Line_get_boxptrs_2_(0xF);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xE() {
    test_Line_get_boxptrs_2_(0xE);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xD() {
    test_Line_get_boxptrs_2_(0xD);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xC() {
    test_Line_get_boxptrs_2_(0xC);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xB() {
    test_Line_get_boxptrs_2_(0xB);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xA() {
    test_Line_get_boxptrs_2_(0xA);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x9() {
    test_Line_get_boxptrs_2_(0x9);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x8() {
    test_Line_get_boxptrs_2_(0x8);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x7() {
    test_Line_get_boxptrs_2_(0x7);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x6() {
    test_Line_get_boxptrs_2_(0x6);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x5() {
    test_Line_get_boxptrs_2_(0x5);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x4() {
    test_Line_get_boxptrs_2_(0x4);
}

/* file dtf5-6b.log content:

2000-01-01 00:00:00 [dtf5-6b]6b
6b
2000-01-01 00:00:01 [dtf5-6b]6b-a
6b-a
2000-01-01 00:00:02 [dtf5-6b]6b-ab
6b-ab
2000-01-01 00:00:03 [dtf5-6b]6b-abc
6b-abc
2000-01-01 00:00:04 [dtf5-6b]6b-abcd
6b-abcd
2000-01-01 00:00:05 [dtf5-6b]6b-abcde
6b-abcde

*/

#[test]
fn test_simple_processing_different_utf() {
    let fpath: FPath = path_to_fpath(&FILE_UTF16LE_SYSLINE_2);
    let mut linereader = new_LineReader(&fpath, 0x4);
    linereader.filetype_text_encoding_update(FileTypeTextEncoding::Utf16le);
    let mut fo: FileOffset = 0;
    loop {
        match linereader.find_line(fo) {
            ResultFindLine::Found((fo_, _)) => {
                fo = fo_;
            }
            ResultFindLine::Done => {
                break;
            }
            ResultFindLine::Err(err) => {
                panic!("LineReader::new({:?}, {:?}) ResultFindLine::Err {}", fpath, 0x2, err);
            }
        }
    }
}

lazy_static::lazy_static! {
    static ref ES: String = String::with_capacity(0);
}

// FILE_UTF8_ABC_FPATH
#[test_case(UTF_8, &FILE_UTF8_ABC_FPATH, 4, 0, RS3T_FOUND, None, String::from("abc"); "UTF8 ABC 4 0")]
#[test_case(UTF_8, &FILE_UTF8_ABC_FPATH, 4, 1, RS3T_FOUND, None, String::from("abc"); "UTF8 ABC 4 1")]
#[test_case(UTF_8, &FILE_UTF8_ABC_FPATH, 4, 2, RS3T_FOUND, None, String::from("abc"); "UTF8 ABC 4 2")]
#[test_case(UTF_8, &FILE_UTF8_ABC_FPATH, 4, 3, RS3T_DONE, None, ES.clone(); "UTF8 ABC 4 3")]
#[test_case(UTF_8, &FILE_UTF8_ABC_FPATH, 4, 4, RS3T_DONE, None, ES.clone(); "UTF8 ABC 4 4")]
// FILE_UTF8_ABC_NL_FPATH
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_FPATH, 4, 0, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL 4 0")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_FPATH, 4, 1, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL 4 1")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_FPATH, 4, 2, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL 4 2")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_FPATH, 4, 3, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL 4 3")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_FPATH, 4, 4, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL 4 4")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_FPATH, 4, 5, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL 4 5")]
// FILE_UTF8_ABC_NL_DEF_FPATH
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 4, 0, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 4 0")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 4, 1, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 4 1")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 4, 2, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 4 2")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 4, 3, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 4 3")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 4, 4, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 4 4")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 4, 5, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 4 5")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 4, 6, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 4 6")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 4, 7, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 4 7")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 4, 8, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 4 8")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 5, 0, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 5 0")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 5, 1, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 5 1")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 5, 2, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 5 2")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 5, 3, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 5 3")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 5, 4, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 5 4")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 5, 5, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 5 5")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 5, 6, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 5 6")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 5, 7, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 5 7")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 5, 8, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 5 8")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 6, 0, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 6 0")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 6, 1, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 6 1")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 6, 2, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 6 2")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 6, 3, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 6 3")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 6, 4, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 6 4")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 6, 5, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 6 5")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 6, 6, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 6 6")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 6, 7, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 6 7")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 6, 8, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 6 8")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 7, 0, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 7 0")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 7, 1, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 7 1")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 7, 2, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 7 2")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 7, 3, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 7 3")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 7, 4, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 7 4")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 7, 5, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 7 5")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 7, 6, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 7 6")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 7, 7, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 7 7")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 7, 8, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 7 8")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 8, 0, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 8 0")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 8, 1, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 8 1")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 8, 2, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 8 2")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 8, 3, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF 8 3")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 8, 4, RS3T_FOUND, None, String::from("def"); "UTF8 ABC NL DEF 8 4")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 8, 5, RS3T_FOUND, None, String::from("def"); "UTF8 ABC NL DEF 8 5")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 8, 6, RS3T_FOUND, None, String::from("def"); "UTF8 ABC NL DEF 8 6")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 8, 7, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 8 7")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, 8, 8, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF 8 8")]
// FILE_UTF8_ABC_NL_DEF_NL_FPATH
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 4, 0, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 4 0")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 4, 1, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 4 1")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 4, 2, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 4 2")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 4, 3, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 4 3")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 4, 4, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 4 4")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 4, 5, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 4 5")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 4, 6, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 4 6")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 4, 7, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 4 7")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 4, 8, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 4 8")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 4, 9, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 4 9")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 5, 0, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 5 0")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 5, 1, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 5 1")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 5, 2, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 5 2")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 5, 3, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 5 3")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 5, 4, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 5 4")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 5, 5, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 5 5")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 5, 6, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 5 6")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 5, 7, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 5 7")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 5, 8, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 5 8")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 5, 9, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 5 9")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 6, 0, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 6 0")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 6, 1, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 6 1")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 6, 2, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 6 2")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 6, 3, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 6 3")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 6, 4, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 6 4")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 6, 5, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 6 5")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 6, 6, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 6 6")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 6, 7, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 6 7")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 6, 8, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 6 8")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 6, 9, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 6 9")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 7, 0, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 7 0")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 7, 1, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 7 1")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 7, 2, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 7 2")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 7, 3, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 7 3")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 7, 4, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 7 4")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 7, 5, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 7 5")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 7, 6, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 7 6")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 7, 7, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 7 7")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 7, 8, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 7 8")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 7, 9, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 7 9")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 8, 0, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 8 0")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 8, 1, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 8 1")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 8, 2, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 8 2")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 8, 3, RS3T_FOUND, None, String::from("abc\n"); "UTF8 ABC NL DEF NL 8 3")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 8, 4, RS3T_FOUND, None, String::from("def\n"); "UTF8 ABC NL DEF NL 8 4")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 8, 5, RS3T_FOUND, None, String::from("def\n"); "UTF8 ABC NL DEF NL 8 5")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 8, 6, RS3T_FOUND, None, String::from("def\n"); "UTF8 ABC NL DEF NL 8 6")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 8, 7, RS3T_FOUND, None, String::from("def\n"); "UTF8 ABC NL DEF NL 8 7")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 8, 8, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 8 8")]
#[test_case(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, 8, 9, RS3T_DONE, None, ES.clone(); "UTF8 ABC NL DEF NL 8 9")]
fn test_find_line_in_block_utf(
    encoding: FileTypeTextEncoding,
    fpath: &FPath,
    blocksz: BlockSz,
    fileoffset: FileOffset,
    expect_result: ResultFindLine_Test,
    expect_partial: Option<&'static str>,
    expect_str: String,
) {
    let in_out: TestFindLineInBlockCheck = vec![
        (fileoffset, (expect_result, expect_partial), expect_str),
    ];
    test_find_line_in_block(fpath, true, blocksz, encoding, &in_out);
}

const UTF8_BOM_ABC_BYTES: [u8; 6] = [0xEF, 0xBB, 0xBF, b'a', b'b', b'c'];
const UTF16BE_BOM_ABC_BYTES: [u8; 8] = [0xFE, 0xFF, 0x00, b'a', 0x00, b'b', 0x00, b'c'];
const UTF16LE_BOM_ABC_BYTES: [u8; 8] = [0xFF, 0xFE, b'a', 0x00, b'b', 0x00, b'c', 0x00];
const UTF32BE_BOM_ABC_BYTES: [u8; 16] = [0x00, 0x00, 0xFE, 0xFF, 0x00, 0x00, 0x00, b'a', 0x00, 0x00, 0x00, b'b', 0x00, 0x00, 0x00, b'c'];
const UTF32LE_BOM_ABC_BYTES: [u8; 16] = [0xFF, 0xFE, 0x00, 0x00, b'a', 0x00, 0x00, 0x00, b'b', 0x00, 0x00, 0x00, b'c', 0x00, 0x00, 0x00];

lazy_static! {
    // UTF-16BE
    static ref ABC_UTF16BE: String = encode_utf16be("abc");
    static ref ABC_NL_UTF16BE: String = encode_utf16be("abc\n");
    static ref DEF_UTF16BE: String = encode_utf16be("def");
    static ref DEF_NL_UTF16BE: String = encode_utf16be("def\n");
    // UTF-16LE
    static ref ABC_UTF16LE: String = encode_utf16le("abc");
    static ref ABC_NL_UTF16LE: String = encode_utf16le("abc\n");
    static ref DEF_UTF16LE: String = encode_utf16le("def");
    static ref DEF_NL_UTF16LE: String = encode_utf16le("def\n");
    // UTF-32BE
    static ref ABC_UTF32BE: String = encode_utf32be("abc");
    static ref ABC_NL_UTF32BE: String = encode_utf32be("abc\n");
    static ref DEF_UTF32BE: String = encode_utf32be("def");
    static ref DEF_NL_UTF32BE: String = encode_utf32be("def\n");
    // UTF-32LE
    static ref ABC_UTF32LE: String = encode_utf32le("abc");
    static ref ABC_NL_UTF32LE: String = encode_utf32le("abc\n");
    static ref DEF_UTF32LE: String = encode_utf32le("def");
    static ref DEF_NL_UTF32LE: String = encode_utf32le("def\n");
}

// UTF-8
#[test_matrix(UTF_8, &FILE_UTF8_ABC_FPATH, (4, 8, 12, 16, 20), (0, 1, 2), RS3T_FOUND, "abc", &[])]
#[test_matrix(UTF_8, &FILE_UTF8_ABC_FPATH, (4, 20), 3, RS3T_DONE, "", &[])]
#[test_matrix(UTF_8, &FILE_UTF8_ABC_NL_FPATH, (4, 8, 12, 16, 20), (0, 1, 2, 3), RS3T_FOUND, "abc\n", &[])]
#[test_matrix(UTF_8, &FILE_UTF8_ABC_NL_FPATH, (4, 20), 4, RS3T_DONE, "", &[])]
#[test_matrix(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, (4, 12, 20), (0, 3), RS3T_FOUND, "abc\n", &[])]
#[test_matrix(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, (4, 12, 20), (5, 6), RS3T_FOUND, "def\n", &[])]
#[test_matrix(UTF_8, &FILE_UTF8_ABC_NL_DEF_FPATH, (4, 50), 7, RS3T_DONE, "", &[])]
#[test_matrix(UTF_8, &FILE_UTF8_ABC_NL_DEF_NL_FPATH, (4, 20), (5, 6), RS3T_FOUND, "def\n", &[])]
// UTF-8 BOM
#[test_matrix(UTF_8_BOM, &FILE_UTF8_BOM_ABC_FPATH, (4, 8, 20), (0, 5), RS3T_FOUND, "", &UTF8_BOM_ABC_BYTES)]
#[test_matrix(UTF_8_BOM, &FILE_UTF8_BOM_ABC_FPATH, 20, 7, RS3T_DONE, "", &[])]
// UTF-16BE
#[test_matrix(UTF_16BE, &FILE_UTF16BE_ABC_FPATH, (4, 30), (0, 5), RS3T_FOUND, ABC_UTF16BE.as_str(), &[])]
#[test_matrix(UTF_16BE, &FILE_UTF16BE_ABC_NL_FPATH, (4, 30), (0, 7), RS3T_FOUND, ABC_NL_UTF16BE.as_str(), &[])]
#[test_matrix(UTF_16BE, &FILE_UTF16BE_ABC_NL_DEF_FPATH, (4, 30), (9, 10), RS3T_FOUND, DEF_UTF16BE.as_str(), &[])]
#[test_matrix(UTF_16BE, &FILE_UTF16BE_ABC_NL_DEF_NL_FPATH, (4, 30), 10, RS3T_FOUND, DEF_NL_UTF16BE.as_str(), &[])]
// UTF-16BE BOM
#[test_matrix(UTF_16BE_BOM, &FILE_UTF16BE_BOM_ABC_FPATH, (4, 30), (0, 5), RS3T_FOUND, "", &UTF16BE_BOM_ABC_BYTES)]
// UTF-16LE
#[test_matrix(UTF_16LE, &FILE_UTF16LE_ABC_FPATH, (4, 30), (0, 5), RS3T_FOUND, ABC_UTF16LE.as_str(), &[])]
#[test_matrix(UTF_16LE, &FILE_UTF16LE_ABC_NL_FPATH, (4, 30), (0, 7), RS3T_FOUND, ABC_NL_UTF16LE.as_str(), &[])]
#[test_matrix(UTF_16LE, &FILE_UTF16LE_ABC_NL_DEF_FPATH, (4, 30), (9, 10), RS3T_FOUND, DEF_UTF16LE.as_str(), &[])]
#[test_matrix(UTF_16LE, &FILE_UTF16LE_ABC_NL_DEF_NL_FPATH, (4, 30), 10, RS3T_FOUND, DEF_NL_UTF16LE.as_str(), &[])]
// UTF-16LE BOM
#[test_matrix(UTF_16LE_BOM, &FILE_UTF16LE_BOM_ABC_FPATH, (4, 30), (0, 5), RS3T_FOUND, "", &UTF16LE_BOM_ABC_BYTES)]
// UTF-32BE
#[test_matrix(UTF_32BE, &FILE_UTF32BE_ABC_FPATH, (4, 30), (0, 5), RS3T_FOUND, ABC_UTF32BE.as_str(), &[])]
#[test_matrix(UTF_32BE, &FILE_UTF32BE_ABC_NL_FPATH, (4, 30), (0, 7), RS3T_FOUND, ABC_NL_UTF32BE.as_str(), &[])]
#[test_matrix(UTF_32BE, &FILE_UTF32BE_ABC_NL_DEF_FPATH, (4, 30), (16, 17, 18, 19, 20), RS3T_FOUND, DEF_UTF32BE.as_str(), &[])]
#[test_matrix(UTF_32BE, &FILE_UTF32BE_ABC_NL_DEF_NL_FPATH, (4, 30), 21, RS3T_FOUND, DEF_NL_UTF32BE.as_str(), &[])]
// UTF-32BE BOM
#[test_matrix(UTF_32BE_BOM, &FILE_UTF32BE_BOM_ABC_FPATH, (4, 30), (0, 9), RS3T_FOUND, "", &UTF32BE_BOM_ABC_BYTES)]
// UTF-32LE
#[test_matrix(UTF_32LE, &FILE_UTF32LE_ABC_FPATH, (4, 30), (0, 5), RS3T_FOUND, ABC_UTF32LE.as_str(), &[])]
#[test_matrix(UTF_32LE, &FILE_UTF32LE_ABC_NL_FPATH, (4, 30), (0, 7), RS3T_FOUND, ABC_NL_UTF32LE.as_str(), &[])]
#[test_matrix(UTF_32LE, &FILE_UTF32LE_ABC_NL_DEF_FPATH, (4, 30), (16, 17, 18, 19, 20), RS3T_FOUND, DEF_UTF32LE.as_str(), &[])]
#[test_matrix(UTF_32LE, &FILE_UTF32LE_ABC_NL_DEF_NL_FPATH, (4, 30), 21, RS3T_FOUND, DEF_NL_UTF32LE.as_str(), &[])]
// UTF-32LE BOM
#[test_matrix(UTF_32LE_BOM, &FILE_UTF32LE_BOM_ABC_FPATH, (4, 30), (0, 9), RS3T_FOUND, "", &UTF32LE_BOM_ABC_BYTES)]
fn test_find_line(
    encoding: FileTypeTextEncoding,
    fpath: &FPath,
    blocksz: BlockSz,
    fileoffset: FileOffset,
    expect_result: ResultFindLine_Test,
    expect_str: &'static str,
    expect_bytes: &'static [u8],
) {
    eprintln!("test_find_line_in_block({encoding:?}, {blocksz:?}, {fileoffset:?})");
    eprint_file(fpath);
    let mut lr1: LineReader = new_LineReader(fpath, blocksz);
    lr1.filetype_text_encoding_update(encoding);

    match lr1.find_line(fileoffset) {
        ResultFindLine::Found((_fo, linep)) => {
            eprintln!("expect_result = {expect_result:?}");
            eprintln!("expect_str = {:?}", str_to_string_noraw(expect_str));
            eprintln!("actual_str = {:?}", linep.to_string_noraw());
            eprintln!("expect_bytes = {:?}", expect_bytes);
            assert!(expect_str.is_empty() || expect_bytes.is_empty(), "must have either expect_str or expect_bytes, but not both; got expect_str {:?} and expect_bytes {:?}", expect_str, expect_bytes);
            assert_eq!(expect_result, RS3T_FOUND, "actual FOUND, expected FOUND");
            let expect_compare: &[u8] = if !expect_bytes.is_empty() {
                expect_bytes
            } else {
                expect_str.as_bytes()
            };
            let mut i = 0;
            for linepart in &linep.lineparts {
                eprintln!("linepart = {:?}", linepart.as_slice());
                for b in linepart.as_slice() {
                    let c = match expect_compare.get(i) {
                        Some(c) => *c,
                        None => {
                            panic!("expect_compare {expect_compare:?} should have more bytes to match linepart; failed at byte index {i}");
                        }
                    };
                    eprintln!("expect {:?} = {:?} actual", c as char, *b as char);
                    assert_eq!(*b, c, "failed to match at byte index {i}; expected byte 0x{c:02X} ({c}) but got 0x{b:02X} ({b})");
                    i += 1;
                }
            }
        }
        ResultFindLine::Done => {
            eprintln!("expect_result = {expect_result:?}");
            eprintln!("expect_str = {:?}", str_to_string_noraw(expect_str));
            assert_eq!(expect_result, RS3T_DONE, "got DONE expected FOUND");
            assert!(expect_str.is_empty(), "expect_str should be empty when expect_result is RS3T_DONE");
        }
        ResultFindLine::Err(err) => {
            panic!("LineReader::new({:?}, {:?}) ResultFindLine::Err {}", fpath, blocksz, err);
        }
    }
}

#[test_case(
    NTF_LINE_TOO_LONG_UTF8_LONG_LINE_OFFSET,
    &NTF_LINE_TOO_LONG_UTF8_FPATH,
    ErrorKind::InvalidData;
    "long line start"
)]
#[test_case(
    NTF_LINE_TOO_LONG_UTF8_LONG_LINE_OFFSET + (LINE_SEARCH_MAX / 2),
    &NTF_LINE_TOO_LONG_UTF8_FPATH,
    ErrorKind::InvalidData;
    "long line middle"
)]
#[test_case(
    NTF_LINE_TOO_LONG_UTF8_LONG_LINE_OFFSET + LINE_SEARCH_MAX,
    &NTF_LINE_TOO_LONG_UTF8_FPATH,
    ErrorKind::InvalidData;
    "long line beyond search max"
)]
fn test_LineReader_find_line_line_too_long(
    fileoffset: FileOffset,
    fpath: &FPath,
    expected_error_kind: ErrorKind,
) {
    let mut lr1 = new_LineReader(fpath, 0x100);
    match lr1.find_line(fileoffset) {
        ResultFindLine::Err(err) => {
            assert_eq!(err.kind(), expected_error_kind);
        }
        ResultFindLine::Found((fo, linep)) => {
            panic!(
                "find_line({fileoffset}) returned Found({}, Line len {}) for {:?}, expected {:?}",
                fo,
                (*linep).len(),
                fpath,
                expected_error_kind,
            );
        }
        ResultFindLine::Done => {
            panic!("find_line({fileoffset}) returned Done for {:?}, expected {:?}", fpath, expected_error_kind);
        }
    }
}

// SummaryLineReader

#[test_case(&*NTF_LOG_EMPTY_FPATH)]
#[test_case(&NTF_NL_1_PATH)]
fn test_LineReader_summary_empty(
    path: &FPath,
) {
    let linereader = new_LineReader(path, 4);
    _ = linereader.summary();
}

#[test_case(
    &NTF_NL_1_PATH,
    4,
    1,
    1,
    1,
    0,
    1,
    1,
    0,
    1,
    1,
    0,
    0
)]
#[test_case(
    &NTF_SYSLINE_2_PATH,
    4,
    46,
    2,
    2,
    0,
    3,
    24,
    0,
    2,
    2,
    0,
    0
)]
/// test `LineReader.Summary()`
// TODO: [2024/03/10] copy design of similar function
//       `fixedstructreader_tests.rs:test_FixedStructReader_summary`
fn test_SummaryLineReader(
    path: &FPath,
    blocksz: BlockSz,
    linereader_line_longest_processed: Count,
    linereader_lines: Count,
    linereader_lines_stored_highest: usize,
    linereader_lines_hits: Count,
    linereader_lines_miss: Count,
    linereader_line_parts_created: Count,
    linereader_find_line_lru_cache_hit: Count,
    linereader_find_line_lru_cache_miss: Count,
    linereader_find_line_lru_cache_put: Count,
    linereader_drop_line_ok: Count,
    linereader_drop_line_errors: Count,
) {
    summary_stats_enable();
    // create a `LineReader` and read all the lines in the file
    let mut lr = new_LineReader(path, blocksz);
    let mut fo: FileOffset = 0;
    loop {
        match lr.find_line(fo) {
            ResultFindLine::Found((fo_, _)) => {
                fo = fo_;
            }
            ResultFindLine::Done => {
                break;
            }
            ResultFindLine::Err(err) => {
                panic!("LineReader::new({:?}, {:?}) ResultFindLine::Err {}", path, blocksz, err);
            }
        }
    }

    let summary: SummaryLineReader = lr.summary();
    assert_eq!(
        summary.linereader_line_longest_processed,
        linereader_line_longest_processed,
        "linereader_line_longest_processed 0"
    );
    assert_eq!(
        summary.linereader_lines,
        linereader_lines,
        "linereader_lines 1"
    );
    assert_eq!(
        summary.linereader_lines_stored_highest,
        linereader_lines_stored_highest,
        "linereader_lines_stored_highest 2"
    );
    assert_eq!(
        summary.linereader_lines_hits,
        linereader_lines_hits,
        "linereader_lines_hits 3"
    );
    assert_eq!(
        summary.linereader_lines_miss,
        linereader_lines_miss,
        "linereader_lines_miss 4"
    );
    assert_eq!(
        summary.linereader_line_parts_created,
        linereader_line_parts_created,
        "linereader_line_parts_created 5"
    );
    assert_eq!(
        summary.linereader_find_line_lru_cache_hit,
        linereader_find_line_lru_cache_hit,
        "linereader_find_line_lru_cache_hit 6"
    );
    assert_eq!(
        summary.linereader_find_line_lru_cache_miss,
        linereader_find_line_lru_cache_miss,
        "linereader_find_line_lru_cache_miss 7"
    );
    assert_eq!(
        summary.linereader_find_line_lru_cache_put,
        linereader_find_line_lru_cache_put,
        "linereader_find_line_lru_cache_put 8"
    );
    assert_eq!(
        summary.linereader_drop_line_ok,
        linereader_drop_line_ok,
        "linereader_drop_line_ok 9"
    );
    assert_eq!(
        summary.linereader_drop_line_errors,
        linereader_drop_line_errors,
        "linereader_drop_line_errors 10"
    );
}
