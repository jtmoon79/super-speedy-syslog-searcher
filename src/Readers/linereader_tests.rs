// Readers/linereader_tests.rs
//

use crate::common::{
    FPath,
    Bytes,
};

use crate::Readers::blockreader::{
    BlockSz,
};

use crate::Readers::linereader::{
    FileOffset,
    LineP,
    ResultS4_LineFind,
    LineReader,
    LineIndex,
    enum_BoxPtrs,
};

use crate::Readers::helpers::{
    randomize,
    fill,
};

use crate::dbgpr::helpers::{
    create_temp_file,
};

use crate::dbgpr::printers::{
    Color,
    print_colored_stdout,
    byte_to_char_noraw,
    buffer_to_String_noraw,
};

use crate::dbgpr::stack::{
    sn,
    so,
    sx,
    stack_offset_set,
};

extern crate more_asserts;
use more_asserts::{
    assert_lt,
    assert_ge
};

/// helper to wrap the match and panic checks
#[cfg(test)]
fn new_LineReader(path: &FPath, blocksz: BlockSz) -> LineReader {
    match LineReader::new(path, blocksz) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: LineReader::new({:?}, {}) failed {}", path, blocksz, err);
        }
    }
}

/// loop on `LineReader.find_line` until it is done
/// this is the most straightforward use of `LineReader`
#[cfg(test)]
fn process_LineReader(lr1: &mut LineReader) {
    eprintln!("{}process_LineReader()", sn());
    let mut fo1: FileOffset = 0;
    loop {
        eprintln!("{}LineReader.find_line({})", so(), fo1);
        let result = lr1.find_line(fo1);
        match result {
            ResultS4_LineFind::Found((fo, lp)) => {
                let _ln = lr1.count();
                eprintln!(
                    "{}ResultS4_LineFind::Found!    FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
                fo1 = fo;
                /*
                if cfg!(debug_assertions) {
                    match print_colored_stdout(Color::Green, &(*lp).as_bytes()) {
                        Ok(_) => {}
                        Err(err) => {
                            panic!("ERROR: print_colored_stdout returned error {}", err);
                        }
                    }
                } else {
                    (*lp).print(true);
                }
                */
                (*lp).print(false);
            }
            ResultS4_LineFind::Found_EOF((fo, lp)) => {
                let _ln = lr1.count();
                eprintln!(
                    "{}ResultS4_LineFind::EOF!  FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
                fo1 = fo;
                (*lp).print(false);
            }
            ResultS4_LineFind::Done => {
                eprintln!("{}ResultS4_LineFind::Done!", so());
                break;
            }
            ResultS4_LineFind::Err(err) => {
                eprintln!("{}ResultS4_LineFind::Err {}", so(), err);
                panic!("ERROR: {}", err);
            }
        }
    }
    eprintln!("{}process_LineReader()", sx());
}

/// have `LineReader` instnace read `content`
/// assert the line count
#[allow(non_snake_case)]
#[cfg(test)]
fn do_test_LineReader_count(content: &str, line_count: usize) {
    eprintln!("{}do_test_LineReader_count()", sn());
    let ntf = create_temp_file(content);
    let blocksz: BlockSz = 64;
    let path = String::from(ntf.path().to_str().unwrap());
    let mut lr1 = new_LineReader(&path, blocksz);
    let bufnoraw = buffer_to_String_noraw(content.as_bytes());
    eprintln!("{}File {:?}", so(), bufnoraw);
    process_LineReader(&mut lr1);
    let lc = lr1.count();
    assert_eq!(line_count as u64, lc, "Expected {} count of lines, found {}", line_count, lc);
    //match print_colored_stdout(
    //    Color::Green,
    //    format!("{}PASS Found {} Lines as expected from {:?}\n", so(), lc, bufnoraw).as_bytes(),
    //) { Ok(_) => {}, Err(_) => {}, };
    eprintln!("{}{:?}", so(), content.as_bytes());
    eprintln!("{}do_test_LineReader_count()", sx());
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count0() {
    do_test_LineReader_count("", 0);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count1_() {
    do_test_LineReader_count(" ", 1);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count1__() {
    do_test_LineReader_count("  ", 1);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count1_n() {
    do_test_LineReader_count(" \n", 1);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count2_n_() {
    do_test_LineReader_count(" \n ", 2);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count2__n__() {
    do_test_LineReader_count("  \n  ", 2);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count2_n_n() {
    do_test_LineReader_count(" \n \n", 2);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count2__n__n() {
    do_test_LineReader_count("  \n  \n", 2);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count3_n_n_() {
    do_test_LineReader_count(" \n \n ", 3);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count3__n__n__() {
    do_test_LineReader_count("  \n  \n  ", 3);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count3__n__n__n() {
    do_test_LineReader_count("  \n  \n  \n", 3);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count1() {
    do_test_LineReader_count("  \n  \n  \n  ", 4);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count4__n__n_n__n() {
    do_test_LineReader_count("  \n  \n  \n  \n", 4);
}

#[allow(non_snake_case)]
#[test]
fn test_LineReader_count4_uhi_n__n__n__n() {
    do_test_LineReader_count("two unicode points é\n  \n  \n  \n", 4);
}

/// call `LineReader.find_line` for all `FileOffset` in passed `offsets`
#[cfg(test)]
fn find_line_all(linereader: &mut LineReader, offsets: &Vec::<FileOffset>) {
    for fo1 in offsets {
        eprintln!("{}LineReader.find_line({})", so(), fo1);
        let result = linereader.find_line(*fo1);
        match result {
            ResultS4_LineFind::Found((fo, lp)) => {
                let _ln = linereader.count();
                eprintln!(
                    "{}ResultS4_LineFind::Found!    FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
            }
            ResultS4_LineFind::Found_EOF((fo, lp)) => {
                let _ln = linereader.count();
                eprintln!(
                    "{}ResultS4_LineFind::EOF!  FileOffset {} line num {} Line @{:p}: len {} {:?}",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
            }
            ResultS4_LineFind::Done => {
                eprintln!("{}ResultS4_LineFind::Done!", so());
            }
            ResultS4_LineFind::Err(err) => {
                eprintln!("{}ResultS4_LineFind::Err {}", so(), err);
                panic!("ERROR: find_line({:?}) {:?}", fo1, err);
            }
        }
    }
}

/// compare contents of a file `path` to contents of `linereader`
/// assert they are the same
/// presumes the linereader has processed the entire file
#[cfg(test)]
fn compare_file_linereader(path: &FPath, linereader: &LineReader) {
    eprintln!("{}_compare_file_linereader({:?})", sn(), path);
    let contents_file: String = std::fs::read_to_string(path).unwrap();
    let contents_file_count = contents_file.lines().count();
    eprintln!(
        "{}contents_file ({} lines):\n{}{:?}\n",
        so(), contents_file_count, so(), contents_file,
    );
    let mut contents_lr: String = String::with_capacity(0);
    linereader.copy_all_lines(&mut contents_lr);
    eprintln!(
        "{}contents_lr ({} lines processed):\n{}{:?}\n",
        so(), linereader.count(), so(), contents_lr,
    );
    let mut i: usize = 0;
    for lines_file_lr1 in contents_file.lines().zip(contents_lr.lines()) {
        i += 1;
        eprintln!(
            "{}compare {}\n{}{:?}\n{}{:?}\n",
            so(), i, so(), lines_file_lr1.0, so(), lines_file_lr1.1,
        );
        assert_eq!(
            lines_file_lr1.0, lines_file_lr1.1,
            "Lines {:?} differ\nFile      : {:?}\nLineReader: {:?}\n",
            i, lines_file_lr1.0, lines_file_lr1.1,
        );
    }
    assert_eq!(
        contents_file_count, i, "Expected to compare {} lines, only compared {}",
        contents_file_count, i
    );
    eprintln!("{}_compare_file_linereader({:?})", sx(), &path);
}

/// have `LineReader` read all file offsets
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_LineReader_all(path: &FPath, blocksz: BlockSz) {
    eprintln!("{}_test_LineReader_all({:?}, {:?})", sn(), &path, blocksz);
    let mut lr1 = new_LineReader(&path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    let fillsz: usize = match lr1.filesz() as usize { 0 => 1, x => x };
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
    let data: &str = "";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}

#[test]
fn test_LineReader_all1() {
    let data: &str = "\
test_LineReader_all1 line 1";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}


#[test]
fn test_LineReader_all1n() {
    let data: &str = "\
test_LineReader_all1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}

#[test]
fn test_LineReader_all2() {
    let data: &str = "\
test_LineReader_all2 line 1
test_LineReader_all2 line 2";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0xFF);
}


#[test]
fn test_LineReader_all2n() {
    let data: &str = "\
test_LineReader_all2n line 1
test_LineReader_all2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}

#[test]
fn test_LineReader_all3_empty() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}

#[test]
fn test_LineReader_all3() {
    let data: &str = "\
test_LineReader_all3 line 1
test_LineReader_all3 line 2
test_LineReader_all3 line 3";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}

#[test]
fn test_LineReader_all3n() {
    let data: &str = "\
test_LineReader_all3n line 1
test_LineReader_all3n line 2
test_LineReader_all3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}


/// have `LineReader` read all file offsets but in reverse
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_LineReader_all_reversed(path: &FPath, blocksz: BlockSz) {
    eprintln!("{}_test_LineReader_all_reversed({:?}, {:?})", sn(), &path, blocksz);
    let mut lr1 = new_LineReader(&path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    let fillsz: usize = match lr1.filesz() as usize { 0 => 1, x => x };
    let mut offsets_all_rev = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_all_rev);
    offsets_all_rev.sort_by(|a, b| b.cmp(a));

    eprintln!("{}offsets_all_rev: {:?}", so(), offsets_all_rev);
    find_line_all(&mut lr1, &offsets_all_rev);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}_test_LineReader_all_reversed({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_all_reversed0_empty() {
    let data: &str = "";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}

#[test]
fn test_LineReader_all_reversed1() {
    let data: &str = "\
test_LineReader_all_reversed1 line 1";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}


#[test]
fn test_LineReader_all_reversed1n() {
    let data: &str = "\
test_LineReader_all_reversed1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}

#[test]
fn test_LineReader_all_reversed2() {
    let data: &str = "\
test_LineReader_all_reversed2 line 1
test_LineReader_all_reversed2 line 2";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0xFF);
}


#[test]
fn test_LineReader_all_reversed2n() {
    let data: &str = "\
test_LineReader_all_reversed2n line 1
test_LineReader_all_reversed2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}

#[test]
fn test_LineReader_all_reversed3_empty() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}

#[test]
fn test_LineReader_all_reversed3() {
    let data: &str = "\
test_LineReader_all_reversed3 line 1
test_LineReader_all_reversed3 line 2
test_LineReader_all_reversed3 line 3";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}

#[test]
fn test_LineReader_all_reversed3n() {
    let data: &str = "\
test_LineReader_all_reversed3n line 1
test_LineReader_all_reversed3n line 2
test_LineReader_all_reversed3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_all(&fpath, 0x4);
}

/// have `LineReader` read all file offsets but in random order
/// TODO: `randomize` should be predictable
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_LineReader_rand(path: &FPath, blocksz: BlockSz) {
    stack_offset_set(None);
    eprintln!("{}_test_LineReader_rand({:?}, {:?})", sn(), &path, blocksz);

    let mut lr1 = new_LineReader(path, blocksz);
    eprintln!("{}LineReader {:?}", so(), lr1);
    let fillsz: usize = match lr1.filesz() as usize { 0 => 1, x => x };
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(fillsz);
    fill(&mut offsets_rand);
    eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);
    randomize(&mut offsets_rand);
    offsets_rand.insert(0, 0);
    eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);

    find_line_all(&mut lr1, &offsets_rand);

    eprintln!("\n{}{:?}\n", so(), lr1);

    compare_file_linereader(path, &lr1);

    eprintln!("{}_test_LineReader_rand({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_rand0_empty() {
    let data: &str = "";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_rand(&fpath, 0x4);
}

#[test]
fn test_LineReader_rand1() {
    let data: &str = "\
test_LineReader_rand1 line 1";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_rand(&fpath, 0x4);
}


#[test]
fn test_LineReader_rand1n() {
    let data: &str = "\
test_LineReader_rand1n line 1n
";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_rand(&fpath, 0x4);
}

#[test]
fn test_LineReader_rand2() {
    let data: &str = "\
test_LineReader_rand2 line 1
test_LineReader_rand2 line 2";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_rand(&fpath, 0xFF);
}


#[test]
fn test_LineReader_rand2n() {
    let data: &str = "\
test_LineReader_rand2n line 1
test_LineReader_rand2n line 2
";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_rand(&fpath, 0x4);
}

#[test]
fn test_LineReader_rand3_empty() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_rand(&fpath, 0x4);
}

#[test]
fn test_LineReader_rand3() {
    let data: &str = "\
test_LineReader_rand3 line 1
test_LineReader_rand3 line 2
test_LineReader_rand3 line 3";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_rand(&fpath, 0x4);
}

#[test]
fn test_LineReader_rand3n() {
    let data: &str = "\
test_LineReader_rand3n line 1
test_LineReader_rand3n line 2
test_LineReader_rand3n line 3
";
    let ntf = create_temp_file(data);
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_rand(&fpath, 0x4);
}

/// have `LineReader` read all file offsets but in a precise order
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_LineReader_precise_order(path: &FPath, blocksz: BlockSz, offsets: &Vec::<FileOffset>) {
    stack_offset_set(None);
    eprintln!("{}_test_LineReader_rand({:?}, {:?}, {:?})", sn(), &path, blocksz, offsets);
    let mut lr1: LineReader = new_LineReader(path, blocksz);

    find_line_all(&mut lr1, offsets);

    eprintln!("\n{}{:?}\n", so(), lr1);
    for (fo, linep) in lr1.lines.iter() {
        eprintln!("{}  Line@{:02}: {:?}", so(), fo, linep);
        for linepart in (*linep).lineparts.iter() {
            eprintln!("{}    LinePart: {:?} {:?}", so(), linepart, linepart.to_String_noraw());
        }
    }

    compare_file_linereader(path, &lr1);

    eprintln!("{}_test_LineReader_rand({:?}, {:?})", sx(), &path, blocksz);
}

#[test]
fn test_LineReader_precise_order_2_0_1() {
    let data: &str = "\
test_LineReader_precise_order_2 line 1
test_LineReader_precise_order_2 line 2
";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![0, 1];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_2_1_0() {
    let data: &str = "\
test_LineReader_precise_order_2 line 1
test_LineReader_precise_order_2 line 2
";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![1, 0];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}


#[test]
fn test_LineReader_precise_order_empty3_0_1_2() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![0, 1, 2];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty3_0_2_1() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![0, 2, 1];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty3_1_2_0() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![1, 2, 0];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty3_1_0_2() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![1, 0, 2];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty3_2_0_1() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![2, 0, 1];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty3_2_1_0() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![2, 1, 0];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty3_1_0_2_1_2() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![1, 0, 2, 1, 2];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty3_1_2_1_2_0() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![1, 2, 1, 2, 0];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty3_0_1_2_2() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![0, 1, 2, 2];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty3_0_2_1_1() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![0, 2, 1, 1];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty3_1_2_0_0() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![1, 2, 0, 0];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_3_0_2_1() {
    let data: &str = "\
test_LineReader_precise_order_3 line 1
test_LineReader_precise_order_3 line 2
test_LineReader_precise_order_3 line 3
";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![0, 2, 1];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

#[test]
fn test_LineReader_precise_order_empty3() {
    let data: &str = "\n\n\n";
    let ntf = create_temp_file(data);
    let offsets: Vec::<FileOffset> = vec![0, 2, 1];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_LineReader_precise_order(&fpath, 0xF, &offsets);
}

type test_Line_get_boxptrs_check = Vec<(FileOffset, (LineIndex, LineIndex), Bytes)>;

/// test `Line.get_boxptrs`
/// assert result equals passed `checks`
fn _test_Line_get_boxptrs(path: &FPath, blocksz: BlockSz, checks: &test_Line_get_boxptrs_check) {
    let fn_: &str = "_test_Line_get_boxptrs";
    eprintln!("{}{}({:?}, {}, checks)", sn(), fn_, path, blocksz);
    // create a `LineReader` and read all the lines in the file
    let mut lr = new_LineReader(path, blocksz);
    let mut fo: FileOffset = 0;
    loop {
        match lr.find_line(fo) {
            ResultS4_LineFind::Found((fo_, _)) => {
                fo = fo_;
            },
            ResultS4_LineFind::Found_EOF((fo_, _)) => {
                fo = fo_;
            },
            ResultS4_LineFind::Done => {
                break;
            },
            ResultS4_LineFind::Err(err) => {
                panic!("LineReader::new({:?}, {:?}) ResultS4_LineFind::Err {}", path, blocksz, err);
            },
        }
    }

    // then test the `Line.get_boxptrs`
    // get_boxptrs(self: &Line, a: LineIndex, mut b: LineIndex) -> Vec<Box<&[u8]>>
    for (linenum, (a, b), bytes_check) in checks.iter() {
        assert_lt!(a, b, "bad check args a {} b {}", a, b);
        assert_ge!(b-a, bytes_check.len(), "Bad check args ({}-{})={} < {} bytes_check.len()", b, a, b-a, bytes_check.len());
        eprintln!("{}{}: linereader.get_linep({})", so(), fn_, linenum);
        let line = lr.get_linep(linenum).unwrap();
        eprintln!("{}{}: returned {:?}", so(), fn_, line.to_String_noraw());
        eprintln!("{}{}: line.get_boxptrs({}, {})", so(), fn_, a, b);
        let boxptrs = match line.get_boxptrs(*a, *b) {
            enum_BoxPtrs::SinglePtr(box_) => {
                vec![box_,]
            },
            enum_BoxPtrs::MultiPtr(boxes) => {
                boxes
            }
        };
        let mut at: usize = 0;
        for boxptr in boxptrs.iter() {
            for byte_ in (*boxptr).iter() {
                let byte_check = &bytes_check[at];
                eprintln!("{}{}: {:3?} ≟ {:3?} ({:?} ≟ {:?})", so(), fn_, byte_, byte_check, byte_to_char_noraw(*byte_), byte_to_char_noraw(*byte_check));
                assert_eq!(byte_, byte_check, "byte {} from boxptr {:?} ≠ {:?} ({:?} ≠ {:?}) check value; returned boxptr segement {:?} Line {:?}", at, byte_, byte_check, byte_to_char_noraw(*byte_), byte_to_char_noraw(*byte_check), buffer_to_String_noraw(&(*boxptr)), line.to_String_noraw());
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
    let checks: test_Line_get_boxptrs_check = vec![
        (0, (0, 1), vec![b't']),
    ];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_Line_get_boxptrs(&fpath, 0xFF, &checks);
}

/// for given `blocksz`, get `LineReader.get_boxptrs` for a predetermined
/// inputs and assert the comparison to `checks` outputs
#[cfg(test)]
fn _test_Line_get_boxptrs_2_(blocksz: BlockSz) {
    eprintln!("{}_test_Line_get_boxptrs_2_({:?})", sn(), blocksz);
    let data: &str = "\
One 1
Two 2";
    let ntf = create_temp_file(data);
    let checks: test_Line_get_boxptrs_check = vec![
        (6, (0, 1), vec![b'T',]),
        (6, (0, 2), vec![b'T', b'w']),
        (7, (0, 2), vec![b'T', b'w']),
        (7, (0, 5), vec![b'T', b'w', b'o', b' ', b'2']),
        (8, (0, 6), vec![b'T', b'w', b'o', b' ', b'2', b'\n']),
        (8, (0, 7), vec![b'T', b'w', b'o', b' ', b'2', b'\n']),
        (9, (0, 6), vec![b'T', b'w', b'o', b' ', b'2', b'\n']),
        (10, (0, 6), vec![b'T', b'w', b'o', b' ', b'2', b'\n']),
        (10, (1, 6), vec![b'w', b'o', b' ', b'2', b'\n']),
        (10, (2, 6), vec![b'o', b' ', b'2', b'\n']),
        (10, (3, 6), vec![b' ', b'2', b'\n']),
        (10, (4, 6), vec![b'2', b'\n']),
        (10, (5, 6), vec![b'\n']),
    ];
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_Line_get_boxptrs(&fpath, blocksz, &checks);
    eprintln!("{}_test_Line_get_boxptrs_2_({:?})", sx(), blocksz);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xF() {
    _test_Line_get_boxptrs_2_(0xF);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xE() {
    _test_Line_get_boxptrs_2_(0xE);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xD() {
    _test_Line_get_boxptrs_2_(0xD);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xC() {
    _test_Line_get_boxptrs_2_(0xC);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xB() {
    _test_Line_get_boxptrs_2_(0xB);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0xA() {
    _test_Line_get_boxptrs_2_(0xA);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x9() {
    _test_Line_get_boxptrs_2_(0x9);
}


#[test]
fn test_Line_get_boxptrs_2_bsz_0x8() {
    _test_Line_get_boxptrs_2_(0x8);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x7() {
    _test_Line_get_boxptrs_2_(0x7);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x6() {
    _test_Line_get_boxptrs_2_(0x6);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x5() {
    _test_Line_get_boxptrs_2_(0x5);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x4() {
    _test_Line_get_boxptrs_2_(0x4);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x3() {
    _test_Line_get_boxptrs_2_(0x3);
}

#[test]
fn test_Line_get_boxptrs_2_bsz_0x2() {
    _test_Line_get_boxptrs_2_(0x2);
}
