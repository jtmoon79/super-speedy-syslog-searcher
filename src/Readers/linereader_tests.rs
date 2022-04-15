// Readers/linereader_tests.rs
//

#[cfg(test)]
use crate::common::{
    FPath,
    Bytes,
};

#[cfg(test)]
use crate::Readers::blockreader::{
    BlockSz,
};

use crate::Readers::linereader::{
    FileOffset,
    ResultS4_LineFind,
    LineReader,
};

#[cfg(test)]
use crate::Readers::linereader::{
    LineIndex,
    enum_BoxPtrs,
};

#[cfg(test)]
use crate::Readers::syslinereader::{
    randomize,
    fill,
};

#[cfg(test)]
use crate::dbgpr::helpers::{
    create_temp_file,
};

use crate::dbgpr::printers::{
    Color,
    print_colored_stdout,
};
#[cfg(test)]
use crate::dbgpr::printers::{
    byte_to_char_noraw,
    buffer_to_String_noraw,
};

use crate::dbgpr::stack::{
    sn,
    so,
    sx,
};

extern crate more_asserts;
#[cfg(test)]
use more_asserts::{assert_lt, assert_ge};

/// loop on `LineReader.find_line` until it is done
/// prints to stdout
/// testing helper
#[cfg(any(debug_assertions,test))]
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
                (*lp).print(true);
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

/// basic test of LineReader things with premade tests
/// simple read of file offsets in order, should print to stdout an identical file
#[allow(non_snake_case)]
#[test]
fn test_LineReader_1() {
    eprintln!("{}test_LineReader_1()", sn());

    for (content, line_count) in [
        ("", 0),
        (" ", 1),
        ("  ", 1),
        (" \n", 1),
        (" \n ", 2),
        ("  \n  ", 2),
        (" \n \n", 2),
        ("  \n  \n", 2),
        (" \n \n ", 3),
        ("  \n  \n  ", 3),
        ("  \n  \n  \n", 3),
        ("  \n  \n  \n  ", 4),
        ("  \n  \n  \n  \n", 4),
        ("two unicode points é\n  \n  \n  \n", 4),
    ] {
        let ntf = create_temp_file(content);
        let blocksz: BlockSz = 64;
        let path = String::from(ntf.path().to_str().unwrap());
        let mut lr1 = match LineReader::new(&path, blocksz) {
            Ok(val) => val,
            Err(err) => {
                panic!("ERROR: LineReader::new({:?}, {}) failed {}", path, blocksz, err);
            }
        };
        let bufnoraw = buffer_to_String_noraw(content.as_bytes());
        eprintln!("{}File {:?}", so(), bufnoraw);
        process_LineReader(&mut lr1);
        let lc = lr1.count();
        assert_eq!(line_count, lc, "Expected {} count of lines, found {}", line_count, lc);
        //match print_colored_stdout(
        //    Color::Green,
        //    format!("{}PASS Found {} Lines as expected from {:?}\n", so(), lc, bufnoraw).as_bytes(),
        //) { Ok(_) => {}, Err(_) => {}, };
        eprintln!("{}{:?}", so(), content.as_bytes());
    }
    eprintln!("{}test_LineReader_1()", sx());
}

/// basic test of LineReader things using user passed file
/// simple read of file offsets in order, should print to stdout an identical file
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_LineReader(path_: &FPath, blocksz: BlockSz) {
    eprintln!("{}test_LineReader({:?}, {})", sn(), &path_, blocksz);
    let mut lr1 = match LineReader::new(path_, blocksz) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: LineReader::new({:?}, {:?}) failed {}", path_, blocksz, err);
        }
    };
    eprintln!("{}LineReader {:?}", so(), lr1);

    process_LineReader(&mut lr1);

    if cfg!(debug_assertions) {
        eprintln!("{}Found {} Lines", so(), lr1.count())
    }
    eprintln!("{}test_LineReader({:?}, {})", sx(), &path_, blocksz);
}

/// basic test of LineReader things using user passed file
/// read all file offsets but randomly
#[allow(non_snake_case)]
#[cfg(test)]
fn test_LineReader_rand(path_: &FPath, blocksz: BlockSz) {
    eprintln!("{}test_LineReader_rand({:?}, {})", sn(), &path_, blocksz);
    let mut lr1 = match LineReader::new(path_, blocksz) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: LineReader::new({}, {}) failed {}", path_, blocksz, err);
        }
    };
    eprintln!("{}LineReader {:?}", so(), lr1);
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(lr1.filesz() as usize);
    fill(&mut offsets_rand);
    eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);
    randomize(&mut offsets_rand);
    eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);

    for fo1 in offsets_rand {
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
                //fo1 = fo;
                //(*lp).print(true);
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
                //fo1 = fo;
                //(*lp).print(true);
            }
            ResultS4_LineFind::Done => {
                eprintln!("{}ResultS4_LineFind::Done!", so());
                break;
            }
            ResultS4_LineFind::Err(err) => {
                eprintln!("{}ResultS4_LineFind::Err {}", so(), err);
                panic!("ERROR: find_line({:?}) {:?}", fo1, err);
            }
        }
    }
    // should print the file as-is and not be affected by random reads
    lr1.print_all();
    eprintln!("\n{}{:?}", so(), lr1);
    eprintln!("{}test_LineReader_rand({:?}, {})", sx(), &path_, blocksz);
}

// TODO: add tests for `test_LineReader_rand`

#[cfg(test)]
type test_Line_get_boxptrs_check = Vec<(FileOffset, (LineIndex, LineIndex), Bytes)>;

/// test `Line.get_boxptrs`
#[cfg(test)]
fn _test_Line_get_boxptrs(fpath: &FPath, blocksz: BlockSz, checks: &test_Line_get_boxptrs_check) {
    eprintln!("{}_test_Line_get_boxptrs({:?}, {}, checks)", sn(), fpath, blocksz);
    // create a `LineReader` and read all the lines in the file
    let mut lr = LineReader::new(fpath, blocksz).unwrap();
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
                panic!("LineReader::new({:?}, {:?}) ResultS4_LineFind::Err {}", fpath, blocksz, err);
            },
        }
    }

    // then test the `Line.get_boxptrs`
    // get_boxptrs(self: &Line, a: LineIndex, mut b: LineIndex) -> Vec<Box<&[u8]>>
    for (linenum, (a, b), bytes_check) in checks.iter() {
        assert_lt!(a, b, "bad check args a {} b {}", a, b);
        assert_ge!(b-a, bytes_check.len(), "Bad check args ({}-{})={} < {} bytes_check.len()", b, a, b-a, bytes_check.len());
        eprintln!("{}_test_Line_get_boxptrs: linereader.get_linep({})", so(), linenum);
        let line = lr.get_linep(linenum).unwrap();
        eprintln!("{}_test_Line_get_boxptrs: returned {:?}", so(), line.to_String_noraw());
        eprintln!("{}_test_Line_get_boxptrs: line.get_boxptrs({}, {})", so(), a, b);
        let boxptrs = match line.get_boxptrs(*a, *b) {
            enum_BoxPtrs::SinglePtr(box_) => {
                let mut v = Vec::<Box<&[u8]>>::with_capacity(1);
                v.push(box_);
                v
            },
            enum_BoxPtrs::MultiPtr(boxes) => {
                boxes
            }
        };
        let mut at: usize = 0;
        for boxptr in boxptrs.iter() {
            for byte_ in (*boxptr).iter() {
                let byte_check = &bytes_check[at];
                eprintln!("{}_test_Line_get_boxptrs: {:3?} ≟ {:3?} ({:?} ≟ {:?})", so(), byte_, byte_check, byte_to_char_noraw(*byte_), byte_to_char_noraw(*byte_check));
                assert_eq!(byte_, byte_check, "byte {} from boxptr {:?} ≠ {:?} ({:?} ≠ {:?}) check value; returned boxptr segement {:?} Line {:?}", at, byte_, byte_check, byte_to_char_noraw(*byte_), byte_to_char_noraw(*byte_check), buffer_to_String_noraw(&(*boxptr)), line.to_String_noraw());
                at += 1;
            }
        }
    }
    eprintln!("{}_test_Line_get_boxptrs", sx());
}

#[test]
fn test_Line_get_boxptrs_1() {
    let data: &str = "\
this is line 1";
    let ntf = create_temp_file(data);
    let mut checks: test_Line_get_boxptrs_check = test_Line_get_boxptrs_check::new();
    checks.push((0, (0, 1), vec![b't']));
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_Line_get_boxptrs(&fpath, 0xFF, &checks);
}

#[cfg(test)]
fn _test_Line_get_boxptrs_2_(blocksz: BlockSz) {
    eprintln!("{}_test_Line_get_boxptrs_2_({:?})", sn(), blocksz);
    let data: &str = "\
One 1
Two 2";
    let ntf = create_temp_file(data);
    let mut checks: test_Line_get_boxptrs_check = test_Line_get_boxptrs_check::new();
    checks.push((6, (0, 1), vec![b'T',]));
    checks.push((6, (0, 2), vec![b'T', b'w']));
    checks.push((7, (0, 2), vec![b'T', b'w']));
    checks.push((7, (0, 5), vec![b'T', b'w', b'o', b' ', b'2']));
    checks.push((8, (0, 6), vec![b'T', b'w', b'o', b' ', b'2', b'\n']));
    checks.push((8, (0, 7), vec![b'T', b'w', b'o', b' ', b'2', b'\n']));
    checks.push((9, (0, 6), vec![b'T', b'w', b'o', b' ', b'2', b'\n']));
    checks.push((10, (0, 6), vec![b'T', b'w', b'o', b' ', b'2', b'\n']));
    checks.push((10, (1, 6), vec![b'w', b'o', b' ', b'2', b'\n']));
    checks.push((10, (2, 6), vec![b'o', b' ', b'2', b'\n']));
    checks.push((10, (3, 6), vec![b' ', b'2', b'\n']));
    checks.push((10, (4, 6), vec![b'2', b'\n']));
    checks.push((10, (5, 6), vec![b'\n']));
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
