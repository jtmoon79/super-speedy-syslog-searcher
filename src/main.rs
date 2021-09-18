// main.rs
/* …
Successful `cat`. Passes all tests in run-tests including utf-8 with high-order characters.

(export RUST_BACKTRACE=1; cargo run -- --filepath Cargo.toml)
(cargo build && rust-gdb -ex 'layout split' -ex 'b src/main.rs:2062' -ex 'r' --args target/debug/block_reader_speedy --filepath /mnt/c/Users/ulug/Projects/syslog-datetime-searcher/logs/other/tests/basic-dt.log 2>/dev/null)
(export RUST_BACKTRACE=1; cargo run -- --filepath /mnt/c/Users/ulug/Projects/syslog-datetime-searcher/logs/other/tests/test3-hex.log)

Test this with shell command: run-test.sh

A good library `fselect` for finding files:
https://docs.rs/crate/fselect/0.7.6

This would be fun: flamegraph
https://github.com/flamegraph-rs/flamegraph

Would this be helpful for datetime_searcher(&String)?
https://lib.rs/crates/strcursor

This looks helpful for searching `Vec[u8]` without requiring conversion to `str`.
https://lib.rs/crates/bstr

Slices and references refresher:
    https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=0fe005a84f341848c491a92615288bad

Stack Offset refresher
    https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=2d870ad0b835ffc8499f7a16b1c424ec

TODO: [2021/09/01] what about mmap? https://stackoverflow.com/questions/45972/mmap-vs-reading-blocks

IDEA: [2021/09/17]
      termcolor each different file. Allow user to constrain colors too (in case some colors display poorly on their terminal)
            CLI options like:
               --color={all,none}
               --colors="black,red,green,yellow"
            Good first step.
            Later could allow user to determine colors for particular files.
            Maybe an "advanced" --path option that allows also passing color for the file:
               --path-color=/var/log/syslog:red

LAST WORKING ON 2021/09/05
    seems to work as a replacement `cat`! :-)
    Add special debug helper function to `BLockReader` and `LineReader` to print
    current known data but in correct file order (not in the order it was accessed): `fn print_known_data`
    Then do similar test but only print some section of the input file. Like first quarter, then middle, then last quarter.
    Consider hiding all these test functions behind a `--test` option. If `--test` is not passed, then just
    behave like `cat`.
    After all that, I think the `SyslogReader` can be started.

LAST WORKING ON 2021/09/09
    got run-tests.sh to pass!
    Add to `LineReader`
       pub fn print_line(fileoffset)
       pub fn fileoffsets() -> Vec<FileOffset> { [x for x in self.lines.keys()] }
       pub fn print_lines()
       fn scan_lines(blockoffset)  # this will be used for analyzing the first block
                                   # do not use `next_line`, write from scratch
    Then implement `SyslogReader`.

    I JUST REALIZED!!!
    The best way to write this, is to have a cursor for each file.
    For each file {
      find the datetime to start at according to filters (beginning of file if no filter)
      set a FileCursor
    }
    Wait for all FileCursors
    loop {
        comparing all known current FileCursors
        print earliest FileCursor, advance that cursor
    }
    ... which is sort of what I'm doing.... but in actuality, I did not need
    manually worry about Blocks. I could have limited search length
    arbitrarily, and used built-in line-searching algorithms.
    DAMN...
    Though, it's the repetitive file reads that would cause slowness...
    so grabbing big Block chunks then analyzing in memory *is* the right approach.
    The tricky part will be throwing Blocks away as soon as they are done with.
    HMMM...
    A good next thing to implement would be a "print and throw away" that
    can print a Sysline based on offset, then checks if the Sysline and underlying
    Lines and Blocks can be deleted. `print` is already implemented, just need
    the "throw away" function. Would need a way to mark Sysline, Line, Block
    as "ready for garbage collection".

LAST WORKING ON 2021/09/15
    Finished Sysline and SyslineReader.
    Now what? See TODO about `get_slice`. That may be next best thing.
    After `get_slice`, compare runtime to prior iteration `try7`, compiled as `block_reader_speedy_try7`
    //       Add `fn get_slice(FileOffset) -> (FileOffset, &[u8], FileOffset)`
    //       gives all relevant Line slices of [u8] directly from underlying Block(s),
    //       no copies or new [u8] or anything else.
    //       Passing value 0 returns
    //           (FileOffset of returned slice, first slice, FileOffset of next slice)
    //       call again with "FileOffset of next slice" to get
    //            (FileOffset of returned slice, next slice, FileOffset of next next slice)
    //       Call until "FileOffset of next next slice" is FO_NULL.
    //       Would need to add `Sysline.get_slice` that calls underlying `Line.get_slice`.
    //       This will allow to create a specialized `struct Printer` that calls
    //       `while Sysline.get_slice` (or should it be a `trait Printer`?)
    //       Then remove all `print` stuff from `Line` and `Sysline`.
    --               
    Then need to implement a basic but useful `find_datetime`.
    Just have it handle a few easy patterns `^YYYY-MM-DD HH:MM:SS`, etc.
    Then `find_datetime` needs to store the processed value as a formal datetime thingy.
    Ignore TZ for now, but add a TODO for handling TZs.
    Will need to look into the best rust datetime crate, must be comparable, and handle differeing TZ.
    Then much after that, will need to implement binary search for syslines based on datetime range.
    Then ... multi-threaded file processing? This leads into proper stages of program:
    1. analyze first block, is it syslog? what is encoding? 2. if yes, begin printing syslogs

LAST WORKING ON 2021/09/16
    Now runs about 3% to 5% faster than prior try7-syslinereader.rs implementation.
    About 110% the time of plain `cat` the file.
    Added more stub code to `find_datetime`.
    Added `get_slices`. Different than above idea and simpler to think about.
    Above `get_slice` idea requires `Iterator` Trait and/or closures, but would be very efficient.
    But hold off for now. (might be helpful https://hermanradtke.com/2015/06/22/effectively-using-iterators-in-rust.html)
    Then resume ideas at "LAST WORKING ON 2021/09/15":
    1. `find_datetime` should also transform string to datetime thingy. return (index, index, datetime_thingy)
    2. add a few more hardcoded patterns to `find_datetime` that parse down to H:M:S.f
    3. implement binary search with datetime filtering.
    Item 3. is a big one, and the last step to complete the proof of concept; to answer the question:
    can this run faster than a Unix script version? `cat`, `sort`, `grep`, etc.
    - 
    Big milestones after that, in recommended order:
    - multi-threaded processing of multiple files
      - shared task queue of files to process
      - "datetime cursor" leads printing of syslines
      - "throw away" all printed syslines and related resources (heap measurement crate?)
        (definitely read this https://nnethercote.github.io/perf-book/heap-allocations.html)
    - passing directory paths (directory walks)
    - determine if file is syslog file
    - robust datetime matching
    - gz archived single log file
    - xz archived single log file
    - ssh URLs (and accessed)
    - multi-byte encoded files
      - use of `bstr` (is it faster?)
    - tar archived single log file
    - tar archived multiple log file
    - tar.gz archives
    - datetime pattern matching at variable line index

TODO: clean up the confusing use Result. Create your own Result Enum that copies what is necessary
      from built-in code.
*/

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// uses and types
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

use std::collections::BTreeMap;
use std::fmt;
use std::fs::{File, Metadata, OpenOptions};
use std::io;
use std::io::prelude::Read;
use std::io::{Result, Error, ErrorKind, Seek, SeekFrom, Write};
use std::path::Path;
use std::rc::Rc;
use std::str;


extern crate atty;

extern crate backtrace;

extern crate clap;
use clap::{App, Arg};

extern crate debug_print;
#[allow(unused_imports)]
use debug_print::{debug_eprint, debug_eprintln, debug_print, debug_println};

extern crate lru;
use lru::LruCache;

#[macro_use]
extern crate more_asserts;

extern crate rand;
use rand::random;

extern crate termcolor;
use termcolor::{Color, ColorSpec, WriteColor};

// XXX: if `Block` is a fixed size array, may be even faster because compile time optimizations?

// type aliases
/// Block Size in bytes
type BlockSz = u64;
/// Byte offset (Index) into a `Block` from beginning of `Block`
type BlockIndex = usize;
/// Offset into a file in `Block`s, depends on `BlockSz` runtime value
type BlockOffset = u64;
/// Offset into a file in bytes
type FileOffset = u64;
/// Block of bytes data read from some file storage
type Block = Vec<u8>;
/// Reference Counting Pointer to a `Block`
type BlockP = Rc<Block>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// globals
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// global constants

/// NewLine as char
#[allow(non_upper_case_globals, dead_code)]
static NLc: char = '\n';
/// NewLine as u8
#[allow(non_upper_case_globals)]
static NLu8: u8 = 10;
#[allow(non_upper_case_globals)]
static FileOffset_NULL: FileOffset = FileOffset::MAX;

// globals

/// stackdepth in `main`, set once, use `stackdepth_main` to read
static mut _STACKDEPTH_MAIN: usize = 0;

/// wrapper for accessing `_STACKDEPTH_MAIN`
fn stackdepth_main () -> usize {
    unsafe {
        _STACKDEPTH_MAIN
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// custom errors
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// for case where reading blocks, lines, or syslines reaches end of file, the value `WriteZero` will
/// be used here ot mean "_end of file reached, nothing new_"
/// XXX: this is a hack
#[allow(non_upper_case_globals)]
static EndOfFile: ErrorKind = ErrorKind::WriteZero;
#[allow(non_upper_case_globals)]
static NoLinesFound: ErrorKind = ErrorKind::InvalidInput;

// TODO: see https://stackoverflow.com/questions/69218879/rust-extend-enum-resultt
//       if it's not possible to cleanly extend `Result` then just make-up my own
//       enums, "modelling" from `std::result::Result`.
//       maybe review https://learning-rust.github.io/docs/e7.custom_error_types.html ?
//       also see https://stackoverflow.com/a/69220986/471376 Answer
/// `Result` `Ext`ended
/// sometimes things are not `Ok` but a value needs to be returned
#[derive(Debug)]
enum ResultExt<T> {
    Result(Result<T>),
    EndOfFile(T),
    NoLinesFound(T),
}

/*
// https://learning-rust.github.io/docs/e7.custom_error_types.html

struct EndOfFile {}

impl fmt::Display for EndOfFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "End Of File!")
    }
}

/// helper to create an EndOfFile Error
fn EndOfFile_new() -> Result<()> {
    Err(EndOfFile("End of file!"))
}
*/

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// misc.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// turn passed u8 into char, for any char values that are CLI formatting instructions transform
/// them to pictoral representations, e.g. '\n' returns a pictoral unicode representation '␊'
fn char_to_nonraw_char(c: char) -> char {
    if c.is_ascii_graphic() {
        return c;
    }
    // https://www.fileformat.info/info/unicode/block/control_pictures/images.htm
    // https://en.wikipedia.org/wiki/C0_and_C1_control_codes#C0_controls
    let val: u32 = c as u32;
    match val {
        0 => '␀',
        1 => '␁',
        2 => '␂',
        3 => '␃',
        4 => '␄',
        5 => '␅',
        6 => '␆',
        7 => '␇',  // '\a'
        8 => '␈',  // '\b'
        9 => '␉',  // '\t'
        10 => '␊', // '\n'
        11 => '␋', // '\v'
        12 => '␌', // '\f'
        13 => '␍', // '\r'
        14 => '␎',
        15 => '␏',
        16 => '␐',
        17 => '␑',
        18 => '␒',
        19 => '␓',
        20 => '␔',
        21 => '␕',
        22 => '␖',
        23 => '␗',
        24 => '␘',
        25 => '␙',
        26 => '␚',
        27 => '␛', // '\e'
        28 => '␜',
        29 => '␝',
        30 => '␞',
        31 => '␟',
        127 => '␡',
        _ => ' ',
    }
}

fn byte_to_nonraw_char(byte: u8) -> char {
    return char_to_nonraw_char(byte as char);
}

/// helper for humans debugging Blocks, very inefficient
#[allow(dead_code)]
fn printblock(buffer: &Block, blockoffset: BlockOffset, fileoffset: FileOffset, blocksz: BlockSz, _mesg: String) {
    const LN: usize = 64;
    println!("╔════════════════════════════════════════════════════════════════════════════╕");
    println!(
        "║File block offset {:4}, byte offset {:4}, block length {:4} (0x{:04X}) (max {:4})",
        blockoffset,
        fileoffset,
        buffer.len(),
        buffer.len(),
        blocksz
    );
    println!("║          ┌────────────────────────────────────────────────────────────────┐");
    let mut done = false;
    let mut i = 0;
    let mut buf = Vec::<char>::with_capacity(LN);
    while i < buffer.len() && !done {
        buf.clear();
        for j in 0..LN {
            if i + j >= buffer.len() {
                done = true;
                break;
            };
            // print line number at beginning of line
            if j == 0 {
                let at: usize = i + j + ((blockoffset * blocksz) as usize);
                print!("║@0x{:06x} ", at);
            };
            let v = buffer[i + j];
            let cp = byte_to_nonraw_char(v);
            buf.push(cp);
        }
        // done reading line, print buf
        i += LN;
        {
            //let s_: String = buf.into_iter().collect();
            let s_ = buf.iter().cloned().collect::<String>();
            println!("│{}│", s_);
        }
    }
    println!("╚══════════╧════════════════════════════════════════════════════════════════╛");
}

/// return current stack depth according to `backtrace::trace`, including this 
/// function
fn stack_depth() -> usize {
    let mut sd: usize = 0;
    backtrace::trace(|_| {
        sd += 1;
        true
    });
    sd
}

/// return stack offset compared to stack depth `_STACKDEPTH_MAIN` recorded in `main`
fn stack_offset() -> usize {
    let mut sd: usize = stack_depth() - 1;
    unsafe {
        if sd < _STACKDEPTH_MAIN {
            return 0;
        }
        sd -= _STACKDEPTH_MAIN;
    }
    return sd;
}

fn test_stack_offset() {
    debug_eprintln!("{}test_stack_offset", sn());
    debug_eprintln!("{}stackdepth_main {}", so(), stackdepth_main());
    debug_eprintln!("{}stack_offset() in test_stack_offset {}", so(), stack_offset());
    fn test1a() {
        debug_eprintln!("{}stack_offset() in test_stack_offset in test1a {}", so(), stack_offset());
    }
    test1a();
    fn test1b() {
        debug_eprintln!("{}stack_offset() in test_stack_offset in test1b {}", so(), stack_offset());
        fn test2a() {
            debug_eprintln!("{}stack_offset() in test_stack_offset in test1b in test2a {}", so(), stack_offset());
        }
        test2a();
        fn test2b(_a: u128, _b: u128, _c: u128) {
            debug_eprintln!("{}stack_offset() in test_stack_offset in test1b in test2b {}", so(), stack_offset());
        }
        test2b(1, 2, 3);
        fn test2c() {
            debug_eprintln!("{}stack_offset() in test_stack_offset in test1b in test2c {}", so(), stack_offset());
        }
        test2c();
        test2b(1, 2, 3);
    }
    test1b();
    debug_eprintln!("{}test_stack_offset", sx());
}

/// return a string of spaces as long as `stack_offset`
/// for use in `print` calls, so short function name and not perfect
fn so() -> &'static str {
    let so_ = stack_offset();
    return match so_ {
        0 => "  ",
        1 => "    ",
        2 => "      ",
        3 => "        ",
        4 => "          ",
        5 => "            ",
        6 => "              ",
        7 => "                ",
        8 => "                  ",
        9 => "                    ",
        10 => "                      ",
        11 => "                        ",
        12 => "                          ",
        13 => "                            ",
        14 => "                              ",
        15 => "                                ",
        16 => "                                  ",
        17 => "                                    ",
        18 => "                                      ",
        19 => "                                        ",
        _ => "                                          ",
    }
}

/// `print` helper, a `s`tring for e`n`tering a function
fn sn() -> &'static str {
    let so_ = stack_offset();
    return match so_ {
        0 => "  →",
        1 => "  →",
        2 => "      →",
        3 => "        →",
        4 => "          →",
        5 => "            →",
        6 => "              →",
        7 => "                →",
        8 => "                  →",
        9 => "                    →",
        10 => "                      →",
        11 => "                        →",
        12 => "                          →",
        13 => "                            →",
        14 => "                              →",
        15 => "                                →",
        16 => "                                  →",
        17 => "                                    →",
        18 => "                                      →",
        19 => "                                        →",
        _ => "                                          →",
    }
}

/// `print` helper, a `s`tring for e`x`iting a function
fn sx() -> &'static str {
    let so_ = stack_offset();
    return match so_ {
        0 => "  ←",
        1 => "    ←",
        2 => "      ←",
        3 => "        ←",
        4 => "          ←",
        5 => "            ←",
        6 => "              ←",
        7 => "                ←",
        8 => "                  ←",
        9 => "                    ←",
        10 => "                      ←",
        11 => "                        ←",
        12 => "                          ←",
        13 => "                            ←",
        14 => "                              ←",
        15 => "                                ←",
        16 => "                                  ←",
        17 => "                                    ←",
        18 => "                                      ←",
        19 => "                                        ←",
        _ => "                                          ←",
    }
}

/// print colored output to terminal if possible
/// otherwise, print plain output
/// taken from https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal
fn print_colored(color: Color, value: &[u8]) -> Result<()> {
    let mut choice: termcolor::ColorChoice = termcolor::ColorChoice::Never;
    if atty::is(atty::Stream::Stdout) {
        choice = termcolor::ColorChoice::Always;
    }
    let mut stdout = termcolor::StandardStream::stdout(choice);
    match stdout.set_color(ColorSpec::new().set_fg(Some(color))) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("print_colored: stdout.set_color({:?}) returned error {}", color, err);
            return Err(err);
        },
    };
    match stdout.write(value) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("print_colored: stdout.write(…) returned error {}", err);
            return Err(err);
        },
    }
    match stdout.reset() {
        Ok(_) => {},
        Err(err) => {
            eprintln!("print_colored: stdout.reset() returned error {}", err);
            return Err(err);
        },
    }
    stdout.flush()?;
    Ok(())
}

/// write something to stdout
pub fn write(buffer: &[u8]) {
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    match stdout_lock.write(buffer) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: write StdoutLock.write({:?}) error {}", buffer, err);
        }
    }
    match stdout_lock.flush() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: write stdout flushing error {}", err);
        }
    }

}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// main
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn main() {
    let matches = App::new("syslog searcher super speedy")
        .version("0.1")
        .author("JTM")
        .about("Reads a file block")
        .arg(
            Arg::with_name("filepath")
                .short("f")
                .long("filepath")
                .value_name("FILE")
                .help("Path of file to read")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("blocksz")
                .help("Block Size")
                .required(false)
                .index(1)
                .takes_value(true)
                .default_value("1024"),
        )
        .get_matches();
    let fpath = String::from(matches.value_of("filepath").unwrap());
    let blockszs = String::from(matches.value_of("blocksz").unwrap());

    // parse input number as either hexadecimal or decimal
    let bsize: BlockSz;
    if blockszs.starts_with("0x") {
        bsize = match BlockSz::from_str_radix(&blockszs.trim_start_matches("0x"), 16) {
            Ok(val) => val,
            Err(_e) => 0,
        };
    } else {
        bsize = match blockszs.parse::<BlockSz>() {
            Ok(val) => val,
            Err(_e) => 0,
        };
    }

    // set `_STACKDEPTH_MAIN` once, use `stackdepth_main` to access `_STACKDEPTH_MAIN`
    unsafe {
        _STACKDEPTH_MAIN = stack_offset();
    }
    //test_stack_offset();
    //test_BlockReader_offsets();
    //test_BlockReader(&fpath, bsize);
    //test_LineReader(&fpath, bsize);
    //test_LineReader_rand(&fpath, bsize);
    test_SyslineReader(&fpath, bsize);
    //test_SyslineReader_rand(&fpath, bsize);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Blocks and BlockReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

type Slices<'a> = Vec<&'a [u8]>;
// Consider this user library which claims to be faster than std::collections::BTreeMap
// https://docs.rs/cranelift-bforest/0.76.0/cranelift_bforest/
type Blocks = BTreeMap<BlockOffset, BlockP>;
type BlocksLRUCache = LruCache<BlockOffset, BlockP>;
// TODO: consider adding a LinkedList to link neighboring... hmm... though that might amount to too many LinkedLists
//       if searches are done randomly.
//       This need will come up again, I suspect... maybe wait on it.

/// Cached file reader that stores data in `BlockSz` byte-sized blocks.
/// A `BlockReader` corresponds to one file.
pub struct BlockReader<'blockreader> {
    /// Path to file
    pub path: &'blockreader Path,
    /// File handle, set in `open`
    file: Option<File>,
    /// File.metadata(), set in `open`
    file_metadata: Option<Metadata>,
    /// File size in bytes, set in `open`
    filesz: u64,
    /// File size in blocks, set in `open`
    blockn: u64,
    /// BlockSz used for read operations
    pub blocksz: BlockSz,
    /// cached storage of blocks
    blocks: Blocks,
    /// internal stats tracking
    stats_read_block_cache_lru_hit: u32,
    /// internal stats tracking
    stats_read_block_cache_lru_miss: u32,
    /// internal stats tracking
    stats_read_block_cache_hit: u32,
    /// internal stats tracking
    stats_read_block_cache_miss: u32,
    /// internal LRU cache for `read_block`
    _read_block_lru_cache: BlocksLRUCache,
}

impl fmt::Debug for BlockReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //let f_ = match &self.file_metadata {
        //    None => format!("None"),
        //    Some(val) => format!("{:?}", val.file_type()),
        //};
        f.debug_struct("BlockReader")
            .field("path", &self.path)
            .field("file", &self.file)
            .field("filesz", &self.filesz)
            .field("blockn", &self.blockn)
            .field("blocksz", &self.blocksz)
            .field("blocks cached", &self.blocks.len())
            .field("cache LRU hit", &self.stats_read_block_cache_lru_hit)
            .field("cache LRU miss", &self.stats_read_block_cache_lru_miss)
            .field("cache hit", &self.stats_read_block_cache_hit)
            .field("cache miss", &self.stats_read_block_cache_miss)
            .finish()
    }
}

/// implement the BlockReader things
impl<'blockreader> BlockReader<'blockreader> {
    /// create a new `BlockReader`
    pub fn new(path_: &'blockreader String, blocksz: BlockSz) -> BlockReader<'blockreader> {
        // TODO: why not open the file here? change `open` to a "static class wide" (or equivalent)
        //       that does not take a `self`. This would simplify some things about `BlockReader`
        // TODO: how to make some fields `blockn` `blocksz` `filesz` immutable?
        //       https://stackoverflow.com/questions/23743566/how-can-i-force-a-structs-field-to-always-be-immutable-in-rust
        assert_ne!(0, blocksz, "Block Size cannot be 0");
        return BlockReader {
            path: &Path::new(path_),
            file: None,
            file_metadata: None,
            filesz: 0,
            blockn: 0,
            blocksz: blocksz,
            blocks: Blocks::new(),
            stats_read_block_cache_lru_hit: 0,
            stats_read_block_cache_lru_miss: 0,
            stats_read_block_cache_hit: 0,
            stats_read_block_cache_miss: 0,
            _read_block_lru_cache: BlocksLRUCache::new(4),
        };
    }

    // TODO: make a `self` version of the following helpers that does not require
    //       passing `BlockSz`. Save the user some trouble.
    //       But keep the public static version available for testing.

    /// return preceding block offset at given file byte offset
    pub fn block_offset_at_file_offset(file_offset: FileOffset, blocksz: BlockSz) -> BlockOffset {
        return (file_offset / blocksz) as BlockOffset;
    }

    /// return file_offset (byte offset) at given `BlockOffset`
    pub fn file_offset_at_block_offset(block_offset: BlockOffset, blocksz: BlockSz) -> FileOffset {
        return (block_offset * blocksz) as BlockOffset;
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub fn file_offset_at_block_offset_index(
        blockoffset: BlockOffset,
        blocksz: BlockSz,
        blockindex: BlockIndex,
    ) -> FileOffset {
        assert_lt!(
            (blockindex as BlockSz),
            blocksz,
            "BlockIndex {} should not be greater or equal to BlockSz {}",
            blockindex,
            blocksz
        );
        BlockReader::file_offset_at_block_offset(blockoffset, blocksz) + (blockindex as FileOffset)
    }

    /// return block_index (byte offset into a `Block`) for `Block` that corresponds to `FileOffset`
    pub fn block_index_at_file_offset(file_offset: FileOffset, blocksz: BlockSz) -> BlockIndex {
        return (file_offset
            - BlockReader::file_offset_at_block_offset(
                BlockReader::block_offset_at_file_offset(file_offset, blocksz),
                blocksz,
            )) as BlockIndex;
    }

    /// return count of blocks in a file
    pub fn file_blocks_count(filesz: FileOffset, blocksz: BlockSz) -> u64 {
        return (filesz / blocksz + (if filesz % blocksz > 0 { 1 } else { 0 })) as u64;
    }

    /// return last valid BlockOffset
    pub fn blockoffset_last(&self) -> BlockOffset {
        (BlockReader::file_blocks_count(self.filesz, self.blocksz) as BlockOffset) - 1
    }

    /// open the `self.path` file, set other field values after opening.
    /// propagates any `Err`, success returns `Ok(())`
    pub fn open(&mut self) -> Result<()> {
        assert!(
            match self.file {
                None => true,
                Some(_) => false,
            },
            "ERROR: the file is already open"
        );
        let mut open_options = OpenOptions::new();
        match open_options.read(true).open(&self.path) {
            Ok(val) => self.file = Some(val),
            Err(err) => {
                eprintln!("ERROR: File::open('{:?}') error {}", &self.path, err);
                return Err(err);
            }
        };
        let file_ = self.file.as_ref().unwrap();
        match file_.metadata() {
            Ok(val) => {
                self.filesz = val.len();
                self.file_metadata = Some(val);
            }
            Err(err) => {
                eprintln!("ERROR: File::metadata() error {}", err);
                return Err(err);
            }
        };
        self.blockn = BlockReader::file_blocks_count(self.filesz, self.blocksz);
        self.blocks = Blocks::new();
        Ok(())
    }

    /// read a `Block` of data of max size `self.blocksz` from a prior `open`ed data source
    /// when successfully read returns `Ok(BlockP)`
    /// when reached the end of the file, and no data was read returns `Err(EndOfFile)`
    /// all other `File` and `std::io` errors are propagated to the caller
    pub fn read_block(&mut self, blockoffset: BlockOffset) -> Result<BlockP> {
        debug_eprintln!("{}read_block: @{:p}.read_block({})", sn(), self, blockoffset);
        assert!(self.file.is_some(), "File has not been opened '{:?}'", self.path);
        // check LRU cache
        match self._read_block_lru_cache.get(&blockoffset) {
            Some(bp) => {
                self.stats_read_block_cache_lru_hit += 1;
                debug_eprintln!(
                    "{}read_block: return Ok(@{:p} LRU cached Block[{}] len {})",
                    sx(),
                    &*bp,
                    &blockoffset,
                    (*bp).len()
                );
                return Ok(bp.clone());
            }
            None => {
                self.stats_read_block_cache_lru_miss += 1;
            }
        }
        // check hash map cache
        if self.blocks.contains_key(&blockoffset) {
            self.stats_read_block_cache_hit += 1;
            debug_eprintln!(
                "{}read_block: return Ok(@{:p} cached Block[{}] len {})",
                sx(),
                &*self.blocks[&blockoffset],
                &blockoffset,
                self.blocks[&blockoffset].len()
            );
            let bp: &BlockP = &self.blocks[&blockoffset];
            self._read_block_lru_cache.put(blockoffset, bp.clone());
            return Ok(bp.clone());
        }
        self.stats_read_block_cache_miss += 1;
        let seek = (self.blocksz * blockoffset) as u64;
        let mut file_ = self.file.as_ref().unwrap();
        match file_.seek(SeekFrom::Start(seek)) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: file.SeekFrom({}) error {}", seek, err);
                debug_eprintln!("{}read_block: return Err({})", sx(), err);
                return Err(err);
            }
        };
        let mut reader = file_.take(self.blocksz as u64);
        // here is where the `Block` is created then set with data.
        // It should never change after this. Is there a way to mark it as "frozen"?
        // I guess just never use `mut`.
        // XXX: currently does not handle a partial read. From the docs (https://doc.rust-lang.org/std/io/trait.Read.html#method.read_to_end)
        //      > If any other read error is encountered then this function immediately returns. Any
        //      > bytes which have already been read will be appended to buf.
        //
        let mut buffer = Block::with_capacity(self.blocksz as usize);
        match reader.read_to_end(&mut buffer) {
            Ok(val) => {
                if val == 0 {
                    // special case of `Err` that caller should handle
                    debug_eprintln!(
                        "{}read_block: return Err(EndOfFile) EndOfFile blockoffset {} {:?}",
                        sx(),
                        blockoffset,
                        self.path.to_str()
                    );
                    return Err(Error::new(EndOfFile, "End Of File"));
                }
            }
            Err(err) => {
                eprintln!("ERROR: reader.read_to_end(buffer) error {}", err);
                debug_eprintln!("      ←read_block: return Err({})", err);
                return Err(err);
            }
        };
        let bp = BlockP::new(buffer);
        // store in cache
        self.blocks.insert(blockoffset, bp.clone());
        debug_eprintln!(
            "{}read_block: return Ok({:p} new Block[{}] len {})",
            sx(),
            &*self.blocks[&blockoffset],
            &blockoffset,
            (*self.blocks[&blockoffset]).len()
        );
        // store in LRU cache
        self._read_block_lru_cache.put(blockoffset, bp.clone());
        Ok(bp)
    }
}

/// basic test of BlockReader things
#[allow(non_snake_case, dead_code)]
fn test_BlockReader(path_: &String, blocksz: BlockSz) {
    debug_println!("test_BlockReader({:?}, {})", &path_, blocksz);

    // testing BlockReader basics

    let mut br1 = BlockReader::new(&path_, blocksz);
    debug_println!("new {:?}", &br1);
    match br1.open() {
        Ok(_) => {
            debug_eprintln!("opened '{}'", path_);
        }
        Err(err) => {
            eprintln!("ERROR: BlockReader.open('{:?}') {}", path_, err);
            return;
        }
    }
    debug_println!("opened {:?}", &br1);
    let last_blk = BlockReader::block_offset_at_file_offset(br1.filesz, blocksz);
    for offset in [0, 1, 5, 1, 99, 1, last_blk].iter() {
        {
            let rbp = br1.read_block(*offset);
            match rbp {
                Ok(val) => {
                    let boff: FileOffset = BlockReader::file_offset_at_block_offset(*offset, blocksz);
                    printblock(val.as_ref(), *offset, boff, blocksz, format!(""));
                }
                Err(err) => {
                    if err.kind() == EndOfFile {
                        continue;
                    } else {
                        eprintln!("ERROR: blockreader.read({}) error {}", offset, err);
                    }
                }
            };
        }
    }
    debug_println!("after reads {:?}", &br1);
}

/// quick self-test
#[allow(dead_code)]
fn test_file_blocks_count() {
    debug_eprintln!("test_file_blocks_count()");
    assert_eq!(1, BlockReader::file_blocks_count(1, 1));
    assert_eq!(2, BlockReader::file_blocks_count(2, 1));
    assert_eq!(3, BlockReader::file_blocks_count(3, 1));
    assert_eq!(4, BlockReader::file_blocks_count(4, 1));
    assert_eq!(1, BlockReader::file_blocks_count(1, 2));
    assert_eq!(1, BlockReader::file_blocks_count(2, 2));
    assert_eq!(2, BlockReader::file_blocks_count(3, 2));
    assert_eq!(2, BlockReader::file_blocks_count(4, 2));
    assert_eq!(3, BlockReader::file_blocks_count(5, 2));
    assert_eq!(1, BlockReader::file_blocks_count(1, 3));
    assert_eq!(1, BlockReader::file_blocks_count(2, 3));
    assert_eq!(1, BlockReader::file_blocks_count(3, 3));
    assert_eq!(2, BlockReader::file_blocks_count(4, 3));
    assert_eq!(1, BlockReader::file_blocks_count(1, 4));
    assert_eq!(1, BlockReader::file_blocks_count(4, 4));
    assert_eq!(2, BlockReader::file_blocks_count(5, 4));
    assert_eq!(1, BlockReader::file_blocks_count(4, 5));
    assert_eq!(1, BlockReader::file_blocks_count(5, 5));
    assert_eq!(2, BlockReader::file_blocks_count(6, 5));
    assert_eq!(2, BlockReader::file_blocks_count(10, 5));
    assert_eq!(3, BlockReader::file_blocks_count(11, 5));
    assert_eq!(3, BlockReader::file_blocks_count(15, 5));
    assert_eq!(4, BlockReader::file_blocks_count(16, 5));
}

/// quick self-test
#[allow(dead_code)]
fn test_file_offset_at_block_offset() {
    debug_eprintln!("test_file_offset_at_block_offset()");
    assert_eq!(0, BlockReader::file_offset_at_block_offset(0, 1));
    assert_eq!(0, BlockReader::file_offset_at_block_offset(0, 2));
    assert_eq!(0, BlockReader::file_offset_at_block_offset(0, 4));
    assert_eq!(1, BlockReader::file_offset_at_block_offset(1, 1));
    assert_eq!(2, BlockReader::file_offset_at_block_offset(1, 2));
    assert_eq!(4, BlockReader::file_offset_at_block_offset(1, 4));
    assert_eq!(2, BlockReader::file_offset_at_block_offset(2, 1));
    assert_eq!(4, BlockReader::file_offset_at_block_offset(2, 2));
    assert_eq!(8, BlockReader::file_offset_at_block_offset(2, 4));
    assert_eq!(3, BlockReader::file_offset_at_block_offset(3, 1));
    assert_eq!(6, BlockReader::file_offset_at_block_offset(3, 2));
    assert_eq!(12, BlockReader::file_offset_at_block_offset(3, 4));
    assert_eq!(4, BlockReader::file_offset_at_block_offset(4, 1));
    assert_eq!(8, BlockReader::file_offset_at_block_offset(4, 2));
    assert_eq!(16, BlockReader::file_offset_at_block_offset(4, 4));
    assert_eq!(5, BlockReader::file_offset_at_block_offset(5, 1));
    assert_eq!(10, BlockReader::file_offset_at_block_offset(5, 2));
    assert_eq!(20, BlockReader::file_offset_at_block_offset(5, 4));
    assert_eq!(8, BlockReader::file_offset_at_block_offset(8, 1));
    assert_eq!(16, BlockReader::file_offset_at_block_offset(8, 2));
    assert_eq!(32, BlockReader::file_offset_at_block_offset(8, 4));
}

/// quick self-test
#[allow(dead_code)]
fn test_block_offset_at_file_offset() {
    debug_eprintln!("test_block_offset_at_file_offset()");
    assert_eq!(0, BlockReader::block_offset_at_file_offset(0, 1));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(1, 1));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(2, 1));
    assert_eq!(3, BlockReader::block_offset_at_file_offset(3, 1));
    assert_eq!(4, BlockReader::block_offset_at_file_offset(4, 1));
    assert_eq!(5, BlockReader::block_offset_at_file_offset(5, 1));
    assert_eq!(8, BlockReader::block_offset_at_file_offset(8, 1));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(0, 2));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(1, 2));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(2, 2));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(3, 2));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(4, 2));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(5, 2));
    assert_eq!(4, BlockReader::block_offset_at_file_offset(8, 2));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(0, 3));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(1, 3));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(2, 3));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(3, 3));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(4, 3));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(6, 3));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(7, 3));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(8, 3));
    assert_eq!(3, BlockReader::block_offset_at_file_offset(9, 3));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(0, 4));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(1, 4));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(2, 4));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(3, 4));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(4, 4));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(5, 4));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(8, 4));
}

/// quick self-test
#[allow(dead_code)]
fn test_block_index_at_file_offset() {
    debug_eprintln!("test_block_index_at_file_offset()");
    assert_eq!(0, BlockReader::block_index_at_file_offset(0, 1));
    assert_eq!(0, BlockReader::block_index_at_file_offset(1, 1));
    assert_eq!(0, BlockReader::block_index_at_file_offset(2, 1));
    assert_eq!(0, BlockReader::block_index_at_file_offset(3, 1));
    assert_eq!(0, BlockReader::block_index_at_file_offset(0, 2));
    assert_eq!(1, BlockReader::block_index_at_file_offset(1, 2));
    assert_eq!(0, BlockReader::block_index_at_file_offset(2, 2));
    assert_eq!(1, BlockReader::block_index_at_file_offset(3, 2));
    assert_eq!(0, BlockReader::block_index_at_file_offset(0, 3));
    assert_eq!(1, BlockReader::block_index_at_file_offset(1, 3));
    assert_eq!(2, BlockReader::block_index_at_file_offset(2, 3));
    assert_eq!(0, BlockReader::block_index_at_file_offset(3, 3));
    assert_eq!(1, BlockReader::block_index_at_file_offset(4, 3));
    assert_eq!(2, BlockReader::block_index_at_file_offset(5, 3));
    assert_eq!(0, BlockReader::block_index_at_file_offset(6, 3));
    assert_eq!(1, BlockReader::block_index_at_file_offset(7, 3));
}

/// quick self-test
#[allow(dead_code)]
fn test_file_offset_at_block_offset_index() {
    debug_eprintln!("test_file_offset_at_block_offset_index()");
    assert_eq!(0, BlockReader::file_offset_at_block_offset_index(0, 1, 0));
    assert_eq!(1, BlockReader::file_offset_at_block_offset_index(1, 1, 0));
    assert_eq!(2, BlockReader::file_offset_at_block_offset_index(2, 1, 0));
    assert_eq!(3, BlockReader::file_offset_at_block_offset_index(3, 1, 0));
    assert_eq!(4, BlockReader::file_offset_at_block_offset_index(4, 1, 0));
    assert_eq!(0, BlockReader::file_offset_at_block_offset_index(0, 2, 0));
    assert_eq!(2, BlockReader::file_offset_at_block_offset_index(1, 2, 0));
    assert_eq!(4, BlockReader::file_offset_at_block_offset_index(2, 2, 0));
    assert_eq!(6, BlockReader::file_offset_at_block_offset_index(3, 2, 0));
    assert_eq!(8, BlockReader::file_offset_at_block_offset_index(4, 2, 0));
    assert_eq!(0, BlockReader::file_offset_at_block_offset_index(0, 3, 0));
    assert_eq!(3, BlockReader::file_offset_at_block_offset_index(1, 3, 0));
    assert_eq!(6, BlockReader::file_offset_at_block_offset_index(2, 3, 0));
    assert_eq!(9, BlockReader::file_offset_at_block_offset_index(3, 3, 0));
    assert_eq!(12, BlockReader::file_offset_at_block_offset_index(4, 3, 0));
    assert_eq!(0, BlockReader::file_offset_at_block_offset_index(0, 4, 0));
    assert_eq!(4, BlockReader::file_offset_at_block_offset_index(1, 4, 0));
    assert_eq!(8, BlockReader::file_offset_at_block_offset_index(2, 4, 0));
    assert_eq!(12, BlockReader::file_offset_at_block_offset_index(3, 4, 0));
    assert_eq!(16, BlockReader::file_offset_at_block_offset_index(4, 4, 0));

    assert_eq!(1, BlockReader::file_offset_at_block_offset_index(0, 2, 1));
    assert_eq!(3, BlockReader::file_offset_at_block_offset_index(1, 2, 1));
    assert_eq!(5, BlockReader::file_offset_at_block_offset_index(2, 2, 1));
    assert_eq!(7, BlockReader::file_offset_at_block_offset_index(3, 2, 1));
    assert_eq!(9, BlockReader::file_offset_at_block_offset_index(4, 2, 1));
    assert_eq!(1, BlockReader::file_offset_at_block_offset_index(0, 3, 1));
    assert_eq!(4, BlockReader::file_offset_at_block_offset_index(1, 3, 1));
    assert_eq!(7, BlockReader::file_offset_at_block_offset_index(2, 3, 1));
    assert_eq!(10, BlockReader::file_offset_at_block_offset_index(3, 3, 1));
    assert_eq!(13, BlockReader::file_offset_at_block_offset_index(4, 3, 1));
    assert_eq!(1, BlockReader::file_offset_at_block_offset_index(0, 4, 1));
    assert_eq!(5, BlockReader::file_offset_at_block_offset_index(1, 4, 1));
    assert_eq!(9, BlockReader::file_offset_at_block_offset_index(2, 4, 1));
    assert_eq!(13, BlockReader::file_offset_at_block_offset_index(3, 4, 1));
    assert_eq!(17, BlockReader::file_offset_at_block_offset_index(4, 4, 1));
}

#[allow(non_snake_case,dead_code)]
fn test_BlockReader_offsets() {
    test_file_blocks_count();
    test_file_offset_at_block_offset();
    test_block_offset_at_file_offset();
    test_block_index_at_file_offset();
    test_file_offset_at_block_offset_index();
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LinePart, Line, and LineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Struct describing a part or all of a line within a `Block`
/// A "line" can span more than one `Block`. This tracks part or all of a line within
/// one `Block`. One `LinePart` to one `Block`.
/// But one or more `LinePart` are necessary to represent an entire "line".
/// TODO: rename LinePart
pub struct LinePart {
    /// index into the `blockp`, index at beginning
    pub blocki_beg: BlockIndex,
    /// index into the `blockp`, index at one after ending '\n' (may refer to one past end of `Block`)
    pub blocki_end: BlockIndex,
    /// the `Block` pointer
    pub blockp: BlockP,
    /// the byte offset into the file where this `LinePart` begins
    pub fileoffset: FileOffset,
    /// debug helper, might be good to get rid of this?
    pub blockoffset: BlockOffset,
    /// debug helper, might be good to get rid of this?
    pub blocksz: BlockSz,
    // TODO: add size of *this* block
}

impl fmt::Debug for LinePart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LinePart")
            .field("LinePart @", &format_args!("{:p}", &self))
            .field("blocki_beg", &self.blocki_beg)
            .field("blocki_end", &self.blocki_end)
            .field("len", &self.len())
            .field("blockp @", &format_args!("{:p}", &(*self.blockp)))
            .field("fileoffset", &self.fileoffset)
            .field("blockoffset", &self.blockoffset)
            .finish()
    }
}

impl LinePart {
    pub fn new(
        blocki_beg: BlockIndex,
        blocki_end: BlockIndex,
        blockp: BlockP,
        fileoffset: FileOffset,
        blockoffset: BlockOffset,
        blocksz: BlockSz,
    ) -> LinePart {
        debug_eprintln!(
            "{}LinePart::new(blocki_beg {}, blocki_end {}, Block @{:p}, fileoffset {}, blockoffset {}, blocksz {})",
            so(),
            blocki_beg,
            blocki_end,
            &*blockp,
            fileoffset,
            blockoffset,
            blocksz
        );
        // some sanity checks
        assert_ne!(fileoffset, FileOffset_NULL, "Bad fileoffset NULL");
        assert_ne!(fileoffset, FileOffset::MAX, "Bad fileoffset MAX");
        assert_ne!(blockoffset, BlockOffset::MAX, "Bad blockoffset MAX");
        let fo1 = BlockReader::file_offset_at_block_offset(blockoffset, blocksz);
        assert_le!(fo1, fileoffset, "Bad FileOffset {}, must ≥ {}", fileoffset, fo1);
        let fo2 = BlockReader::file_offset_at_block_offset(blockoffset + 1, blocksz);
        assert_le!(fileoffset, fo2, "Bad FileOffset {}, must ≤ {}", fileoffset, fo2);
        let bo = BlockReader::block_offset_at_file_offset(fileoffset, blocksz);
        assert_eq!(blockoffset, bo, "Bad BlockOffset {}, expected {}", blockoffset, bo);
        let bi = BlockReader::block_index_at_file_offset(fileoffset, blocksz);
        assert_eq!(
            blocki_beg, bi,
            "blocki_beg {} ≠ {} block_index_at_file_offset({}, {})",
            blocki_beg, bi, fileoffset, blocksz
        );
        assert_ne!(blocki_end, 0, "Bad blocki_end 0, expected > 0");
        assert_lt!(blocki_beg, blocki_end, "blocki_beg {} should be < blocki_end {}", blocki_beg, blocki_end);
        assert_lt!((blocki_beg as BlockSz), blocksz, "blocki_beg {} should be < blocksz {}", blocki_beg, blocksz);
        assert_le!((blocki_end as BlockSz), blocksz, "blocki_end {} should be ≤ blocksz {}", blocki_end, blocksz);
        LinePart {
            blocki_beg: blocki_beg,
            blocki_end: blocki_end,
            blockp: blockp,
            fileoffset: fileoffset,
            blockoffset: blockoffset,
            blocksz: blocksz,
        }
    }

    /// length of line starting at index `blocki_beg`
    pub fn len(&self) -> usize {
        return (self.blocki_end - self.blocki_beg) as usize;
    }
}

/// A sequence to track a `Line`.
/// A "line" may span multiple `Block`s. One `LinePart` is needed for each `Block`.
type LineParts = Vec<LinePart>;

// XXX: Vector slices would be ideal. But slicing a vector returns an array.
//      The size of that slice must be known at compile time. So slices will not work.

/// A `Line` has information about a "line" that may or may not span more than one `Block`
pub struct Line {
    lineparts: LineParts,
}

impl fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut li_s = String::new();
        for li in self.lineparts.iter() {
            li_s.push_str(&format!(
                " @{:p} (blocki_beg {}, blocki_end {}, len() {}, BlockP.len() {}, fileoffset {}, blockoffset {})",
                &li,
                &li.blocki_beg,
                &li.blocki_end,
                &li.len(),
                &li.blockp.len(),
                &li.fileoffset,
                &li.blockoffset
            ));
        }
        f.debug_struct("Line")
            .field("line.fileoffset_begin()", &self.fileoffset_begin())
            .field("line.fileoffset_end()", &self.fileoffset_end())
            .field("lineparts @", &format_args!("{:p}", &self))
            .field("lineparts.len", &self.lineparts.len())
            .field("lineparts", &li_s)
            .finish()
    }
}

impl Line {
    /// default `with_capacity` for a `LineParts`, most often will only need 1 capacity
    /// as the found "line" will likely reside within one `Block`
    const LINE_PARTS_WITH_CAPACITY: usize = 1;

    pub fn new() -> Line {
        return Line {
            lineparts: LineParts::with_capacity(Line::LINE_PARTS_WITH_CAPACITY),
        };
    }

    pub fn new_from_linepart(info: LinePart) -> Line {
        let mut v = LineParts::with_capacity(Line::LINE_PARTS_WITH_CAPACITY);
        v.push(info);
        return Line { lineparts: v };
    }

    pub fn push(&mut self, linepart: LinePart) {
        let l_ = self.lineparts.len();
        if l_ > 0 {
            // sanity checks; each `LinePart` should be stored in same order as it appears in the file
            // only need to compare to last `LinePart`
            let li = &self.lineparts[l_ - 1];
            assert_le!(
                li.blockoffset,
                linepart.blockoffset,
                "Prior stored LinePart at blockoffset {} is after passed LinePart at blockoffset {}",
                li.blockoffset,
                linepart.blockoffset,
            );
            assert_lt!(
                li.fileoffset,
                linepart.fileoffset,
                "Prior stored LinePart at fileoffset {} is at or after passed LinePart at fileoffset {}",
                li.fileoffset,
                linepart.fileoffset,
            );
        }
        // TODO: add sanity checks of all prior `linepart` that all `blocki_end` match `*blockp.len()`
        self.lineparts.push(linepart);
    }

    /// the byte offset into the file where this `Line` begins
    pub fn fileoffset_begin(self: &Line) -> FileOffset {
        assert_ne!(self.lineparts.len(), 0, "This Line has no `LinePart`");
        self.lineparts[0].fileoffset
    }

    /// the byte offset into the file where this `Line` ends, inclusive (not one past ending)
    pub fn fileoffset_end(self: &Line) -> FileOffset {
        assert_ne!(self.lineparts.len(), 0, "This Line has no `LinePart`");
        let last_li = self.lineparts.len() - 1;
        self.lineparts[last_li].fileoffset + (self.lineparts[last_li].len() as FileOffset) - 1
    }

    /// XXX: is this correct?
    pub fn len(self: &Line) -> usize {
        (self.fileoffset_end() - self.fileoffset_begin() + 1) as usize
    }

    /// count of `LinePart` in `self.lineparts.len()`
    pub fn count(self: &Line) -> usize {
        self.lineparts.len()
    }

    /// return all slices that make up this `Line`
    pub fn get_slices(self: &Line) -> Slices {
        // short-circuit this case
        let sz = self.lineparts.len();
        let mut slices = Slices::with_capacity(sz);
        for linepart in self.lineparts.iter() {
            let slice = &linepart.blockp[linepart.blocki_beg..linepart.blocki_end];
            slices.push(slice);
        }
        return slices;
    }

    /// return a count of slices that would be returned by `get_slices`
    pub fn get_slices_count(self: &Line) -> usize {
        return self.lineparts.len();
    }

    /// `raw` true will write directly to stdout from the stored `Block`
    /// `raw` false will write transcode each bute to a character and use pictoral representations
    /// XXX: `raw==false` does not handle multi-byte encodings
    pub fn print(self: &Line, raw: bool) {
        // is this an expensive command? should `stdout` be cached?
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();
        for linepart in &self.lineparts {
            // TODO: can this be done without creating an entirely new array slice?
            //       Or is slicing being smart and not creating a new array on the stack?
            //       I'm somewhat sure slicing is smart, but I should verify with `gdb-rust`.
            let slice = &linepart.blockp[linepart.blocki_beg..linepart.blocki_end];
            if raw {
                match stdout_lock.write(slice) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!(
                            "ERROR: StdoutLock.write(@{:p}[{}..{}]) error {}",
                            &*linepart.blockp, linepart.blocki_beg, linepart.blocki_end, err
                        );
                    }
                }
            } else {
                // XXX: only handle single-byte encodings
                // XXX: this is not efficient
                //let s = match str::from_utf8_lossy(slice) {
                let s = match str::from_utf8(&slice) {
                    Ok(val) => val,
                    Err(err) => {
                        eprintln!("ERROR: Invalid UTF-8 sequence during from_utf8_lossy: '{}'", err);
                        continue;
                    }
                };
                let mut dst: [u8; 4] = [0, 0, 0, 0];
                for c in s.chars() {
                    let c_ = char_to_nonraw_char(c);
                    let _cs = c_.encode_utf8(&mut dst);
                    match stdout_lock.write(&dst) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("ERROR: StdoutLock.write({:?}) error {}", &dst, err);
                        }
                    }
                }
            }
        }
        match stdout_lock.flush() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: stdout flushing error {}", err);
            }
        }
    }

    /// create `String` from known bytes referenced by `self.lineparts`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_nonraw_char`)
    /// TODO: this would be more efficient returning `&str`
    ///       https://bes.github.io/blog/rust-strings
    #[allow(non_snake_case)]
    fn _to_String_raw(self: &Line, raw: bool) -> String {
        let mut sz: usize = 0;
        for linepart in &self.lineparts {
            sz += linepart.len();
        }
        let mut s_ = String::with_capacity(sz);
        // iterate over each LinePart and over each u8, copy u8 as char to `s_`
        // TODO: can these be copied in chunks?
        for linepart in &self.lineparts {
            let stop = linepart.len();
            let block_iter = (&*linepart.blockp).iter();
            for (bi, b) in block_iter.skip(linepart.blocki_beg).enumerate() {
                if bi >= stop {
                    break;
                }
                if raw {
                    // allow whatever formatting characters were found, i.e. '\n'
                    s_.push(*b as char);
                } else {
                    // turn formatting characters into pictoral representation, i.e. '\n' becomes '␊'
                    let c = byte_to_nonraw_char(*b);
                    s_.push(c);
                }
            }
        }
        return s_;
    }

    // XXX: rust does not support function overloading which is really surprising and disappointing
    /// `Line` to `String`
    #[allow(non_snake_case)]
    pub fn to_String(self: &Line) -> String {
        return self._to_String_raw(true);
    }

    #[allow(non_snake_case)]
    pub fn to_String_from(self: &Line, _from: usize) -> String {
        assert!(false, "not implemented");
        return String::from("stub!");
    }

    #[allow(non_snake_case)]
    pub fn to_String_from_to(self: &Line, _from: usize, _to: usize) -> String {
        assert!(false, "not implemented");
        return String::from("stub!");
    }

    /// `Line` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    pub fn to_String_noraw(self: &Line) -> String {
        return self._to_String_raw(false);
    }

    /// slice that represents the entire `Line`
    /// if `Line` does not cross a Block then this returns slice into the `Block`,
    /// otherwise it requires a copy of `Block`s data
    /// TODO: should use `&[char]`?
    /// XXX: cannot return slice because 1. size not known at compile time so cannot
    ///      place on stack 2. slice is an array which is not an "owned type"
    pub fn as_slice(self: &Line) -> Block {
        assert_gt!(self.lineparts.len(), 0, "This Line has no LineParts");
        // efficient case, Line does not cross any Blocks
        if self.lineparts.len() == 1 {
            let bi_beg = self.lineparts[0].blocki_beg;
            let bi_end = self.lineparts[0].blocki_end;
            assert_eq!(bi_end - bi_beg, self.len(), "bi_end-bi_beg != line.len()");
            return Block::from(&(*(self.lineparts[0].blockp))[bi_beg..bi_end]);
        }
        // not efficient case, Line crosses stored Blocks so have to create a new array
        let sz = self.len();
        assert_ne!(sz, 0, "self.len() is zero!?");
        let mut data = Block::with_capacity(sz);
        for lp in self.lineparts.iter() {
            let bi_beg = lp.blocki_beg;
            let bi_end = lp.blocki_end;
            data.extend_from_slice(&(*(lp.blockp))[bi_beg..bi_end]);
        }
        assert_eq!(data.len(), self.len(), "Line.as_slice: data.len() != self.len()");
        return data;
    }
}

type CharSz = usize;
/// reference counting pointer to a `Line`
type LineP = Rc<Line>;
/// storage for Lines found from the underlying `BlockReader`
type Lines = BTreeMap<FileOffset, LineP>;

/// Specialized Reader that uses BlockReader to find Lines
pub struct LineReader<'linereader> {
    blockreader: BlockReader<'linereader>,
    /// track `Line` found among blocks in `blockreader`, tracked by line beginning `FileOffset`
    /// key value `FileOffset` should agree with `(*LineP).fileoffset_begin()`
    pub lines: Lines,
    /// track `Line` found among blocks in `blockreader`, tracked by line ending `FileOffset`
    /// key value `FileOffset` should agree with `(*LineP).fileoffset_end()`
    lines_end: Lines,
    /// char size in bytes
    /// TODO: handle char sizes > 1 byte
    /// TODO: handle multi-byte encodings
    charsz: CharSz,
    // internal state of `next_line` (sans generators which are experimental as of 2021 https://archive.ph/wNgZp)
    // TODO: [2021/08/31] go ahead and use generators, it's far less messy https://doc.rust-lang.org/beta/unstable-book/language-features/generators.html
    //_next_line_blockoffset: BlockOffset,
    // / internal state of `next_line`
    // / this is a one element cache of the `BlockP`
    //_next_line_blockp_opt: Option<BlockP>,
    // / index into `Block` at `*(_next_line_blockp_opt.unwrap())`
    //_next_line_blocki: BlockIndex,
}

impl fmt::Debug for LineReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //let f_ = match &self.file_metadata {
        //    None => format!("None"),
        //    Some(val) => format!("{:?}", val.file_type()),
        //};
        f.debug_struct("LineReader")
            //.field("@", format!("{:p}", &self))
            .field("blockreader", &self.blockreader)
            .field("charsz", &self.charsz)
            .field("lines", &self.lines)
            .finish()
    }
}

// XXX: cannot place these within `impl LineReader`?
/// minimum char storage size in bytes
static CHARSZ_MIN: CharSz = 1;
/// maximum char storage size in bytes
static CHARSZ_MAX: CharSz = 4;
/// default char storage size in bytes
/// XXX: does not handle multi-byte encodings (e.g. UTF-8) or multi-byte character storage (e.g. UTF-32)
static CHARSZ: CharSz = CHARSZ_MIN;

/// implement the LineReader things
impl<'linereader> LineReader<'linereader> {
    pub fn new(path: &'linereader String, blocksz: BlockSz) -> Result<LineReader<'linereader>> {
        // XXX: multi-byte
        assert_ge!(
            blocksz,
            (CHARSZ_MIN as BlockSz),
            "BlockSz {} is too small, must be greater than or equal {}",
            blocksz,
            CHARSZ_MAX
        );
        assert_ne!(blocksz, 0, "BlockSz is zero");
        let mut br = BlockReader::new(&path, blocksz);
        match br.open() {
            Err(err) => {
                return Err(err);
            }
            Ok(_) => {}
        };
        Ok(LineReader {
            blockreader: br,
            lines: Lines::new(),
            lines_end: Lines::new(),
            charsz: CHARSZ,
            // give impossible value to start with
            //_next_line_blockoffset: FileOffset::MAX,
            //_next_line_blockp_opt: None,
            //_next_line_blocki: 0,
        })
    }

    pub fn blocksz(&self) -> BlockSz {
        self.blockreader.blocksz
    }

    pub fn filesz(&self) -> BlockSz {
        self.blockreader.filesz
    }

    pub fn path(&self) -> &str {
        return self
            .blockreader
            .path
            .to_str()
            .unwrap_or("ERROR UNWRAPPING self.blockreader.path");
    }

    /// print `Line` at `fileoffset`
    /// return `false` if `fileoffset` not found
    pub fn print(&self, fileoffset: &FileOffset) -> bool {
        if !self.lines.contains_key(fileoffset) {
            return false;
        }
        let lp = &self.lines[fileoffset];
        lp.print(true);
        return true;
    }

    /// Testing helper only
    /// print all known `Line`s
    pub fn print_all(&self) {
        for fo in self.lines.keys() {
            self.print(&fo);
        }
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    pub fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        BlockReader::block_offset_at_file_offset(fileoffset, self.blocksz())
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    pub fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz())
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        BlockReader::file_offset_at_block_offset_index(blockoffset, self.blocksz(), blockindex)
    }

    /// return block index at given `FileOffset`
    pub fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        BlockReader::block_index_at_file_offset(fileoffset, self.blocksz())
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    pub fn file_blocks_count(&self) -> u64 {
        BlockReader::file_blocks_count(self.filesz(), self.blocksz())
    }

    pub fn blockoffset_last(&self) -> BlockOffset {
        self.blockreader.blockoffset_last()
    }

    /// find next "line" starting from `fileoffset`, create and store a `Line`
    /// successful read of a line returns _`FileOffset` of end of the line + 1*charsz_
    /// reaching end of file (and no new line) returns `(true, ...)` otherwise `(false, ...)`
    /// reaching end of file returns `FileOffset` value that is one byte past the actual end of file (and should not be used)
    /// all other `Result::Err` errors are propagated
    /// XXX: presumes single-byte to one '\n', does not handle UTF-16 or UTF-32 or other (`charsz` hardcoded to 1)
    /// TODO: [2021/08/30] handle different encodings
    /// XXX: this function is fragile and cumbersome, any tweaks require extensive retesting
    /// TODO: the the bool `eof` and the `Err(EndOfFile)` achieve the same purpose,
    ///       but the tri-state is necessary for one case where a `Line` is found and also the `EndOfFile` is reached
    ///       How to have a tri-state `Result` type? `Ok(LineP)` `EndOfFile(LineP)` `Err(err)`
    ///       or a four-state `Result` type? `Ok(LineP)` `EndOfFile(LineP)` `EndOfFile()` `Err(err)`
    ///       or should `FileOffset` be overloaded to allow signifying some of this?
    ///       or should `FileOffset` travel with `LineP` when there is a `Line` found? (might be cleaner)
    ///       or `Result` type `Ok((FileOffset, LineP))` `EndOfFile((FileOffset, LineP))` `EndOfFile(NULL)` `Err(err)`
    pub fn find_line(&mut self, fileoffset: FileOffset) -> (bool, FileOffset, Result<LineP>) {
        debug_eprintln!("{}find_line(LineReader@{:p}, {})", sn(), self, fileoffset);

        // some helpful constants
        let charsz_fo = self.charsz as FileOffset;
        let charsz_bi = self.charsz as BlockIndex;
        let filesz = self.filesz();
        let blockoffset_last = self.blockoffset_last();

        // XXX: this function not a candidate for small LRU cache because callers will not
        //      likely call with the same `FileOffset`

        // handle special case
        if filesz == 0 {
            let err = Err(Error::new(EndOfFile, "File Is Empty"));
            debug_eprintln!("{}find_line: return ({}, {}, Err({:?})) file is empty", sx(), true, FileOffset_NULL, err);
            return (true, FileOffset_NULL, err);
        }
        // handle special case
        if fileoffset >= filesz {
            let err = Err(Error::new(EndOfFile, "End Of File"));
            debug_eprintln!(
                "{}find_line: return ({}, {}, Err({:?})) passed offset {} at or past end of file {}",
                sx(),
                true,
                FileOffset_NULL,
                err,
                fileoffset,
                filesz
            );
            return (true, FileOffset_NULL, err);
        }

        // first check if there is a line already known at this fileoffset
        if self.lines.contains_key(&fileoffset) {
            debug_eprintln!("{}find_line: hit cache for FileOffset {}", so(), fileoffset);
            let lp = self.lines[&fileoffset].clone();
            let fo_next = (*lp).fileoffset_end() + charsz_fo;
            // TODO: determine if `fileoffset` is the last line of the file
            //       should add a private helper function for this task `is_line_last(FileOffset)` ... something like that
            // TODO: add stats like BockReader
            debug_eprintln!("{}find_line: return ({}, {}, Ok(@{:p}))", sx(), false, fo_next, &*lp);
            return (false, fo_next, Ok(lp));
        }
        debug_eprintln!("{}find_line: no cache hit, searching for first newline found_nl_a …", so());

        //
        // walk through blocks and bytes looking for beginning of a line (a newline character; part A)
        //

        // block pointer to the current block of interest
        let mut bp: BlockP;
        // found newline part A? Line begins after that newline
        let mut found_nl_a = false;
        // should point to beginning of `Line` (one char after found newline A)
        let mut fo_nl_a: FileOffset = 0;
        // if at first byte of file no need to search for first newline
        if fileoffset == 0 {
            found_nl_a = true;
            debug_eprintln!("{}find_line: newline A is {} because at beginning of file!", so(), fo_nl_a);
        }
        // if prior char at fileoffset-1 has newline then use that
        // caller's commonly call this function `find_line` in a sequence so it's an easy check
        // with likely success
        if !found_nl_a {
            // XXX: single-byte encoding
            let fo1 = fileoffset - charsz_fo;
            if self.lines_end.contains_key(&fo1) {
                found_nl_a = true;
                debug_eprintln!(
                    "{}find_line: found newline A {} from lookup of passed fileoffset-1 {}",
                    so(),
                    fo1,
                    fileoffset - 1
                );
                // `fo_nl_a` should refer to first char past newline A
                // XXX: single-byte encoding
                fo_nl_a = fo1 + charsz_fo;
            }
        }

        let mut eof = false;
        let mut bo = self.block_offset_at_file_offset(fileoffset);
        let mut bin_beg_init_a = self.block_index_at_file_offset(fileoffset);
        while !found_nl_a && bo <= blockoffset_last {
            match self.blockreader.read_block(bo) {
                Ok(val) => {
                    debug_eprintln!(
                        "{}find_line: read_block returned Block @{:p} len {} while searching for found_nl_a",
                        so(),
                        &(*val),
                        (*val).len()
                    );
                    bp = val;
                }
                Err(err) => {
                    if err.kind() == EndOfFile {
                        debug_eprintln!("{}find_line: read_block returned EndOfFile {:?} searching for found_nl_a failed (IS THIS AN ERROR?)", so(), self.path());
                        // reached end of file, no beginning newlines found
                        // TODO: Is this an error state? should this be handled differently?
                        debug_eprintln!("{}find_line: return ({}, {}, Err({}))", sx(), true, FileOffset_NULL, err);
                        return (true, FileOffset_NULL, Err(err));
                    }
                    debug_eprintln!("{}find_line: return ({}, {}, Err({}))", sx(), false, FileOffset_NULL, err);
                    return (false, FileOffset_NULL, Err(err));
                }
            }
            let blen = (*bp).len() as BlockIndex;
            let mut bin_beg = bin_beg_init_a;
            while bin_beg < blen {
                // XXX: single-byte encoding
                if (*bp)[bin_beg] == NLu8 {
                    found_nl_a = true;
                    fo_nl_a = self.file_offset_at_block_offset_index(bo, bin_beg);
                    debug_eprintln!(
                        "{}find_line: found newline A from byte search fileoffset {} ≟ blockoffset {} blockindex {}",
                        so(),
                        fo_nl_a,
                        bo,
                        bin_beg
                    );
                    // `fo_nl_a` should refer to first char past newline A
                    // XXX: single-byte encoding
                    fo_nl_a += charsz_fo;
                    break;
                }
                // XXX: single-byte encoding
                bin_beg += charsz_bi;
            }
            if found_nl_a {
                break;
            }
            bin_beg_init_a = 0;
            bo += 1;
            if bo > blockoffset_last {
                debug_eprintln!("{}find_line: EOF blockoffset {} > {} blockoffset_last", so(), bo, blockoffset_last);
                eof = true;
                break;
            }
            if fo_nl_a >= filesz {
                debug_eprintln!("{}find_line: EOF newline A fileoffset {} > {} file size", so(), fo_nl_a, filesz);
                eof = true;
                break;
            }
        } // ! found_nl_a

        assert_lt!(fo_nl_a, filesz + 1, "ERROR: newline A {} is past end of file {}", fo_nl_a, filesz + 1);
        if eof {
            // the last character in the file is a newline
            // XXX: is this correct?
            let err = Err(Error::new(EndOfFile, "End Of File"));
            debug_eprintln!(
                "{}find_line: return ({}, {}, Err({:?})) - newline A is at last char in file {}, not a line",
                sx(),
                eof,
                FileOffset_NULL,
                err,
                filesz - 1
            );
            return (eof, FileOffset_NULL, err);
        }

        //
        // walk through blocks and bytes looking for ending of line (a newline character; part B)
        //
        debug_eprintln!(
            "{}find_line: found first newline A, searching for second B newline starting at {} …",
            so(),
            fo_nl_a
        );

        // found newline part B? Line ends at this
        let mut found_nl_b: bool = false;
        // this is effectively the cursor that is being analyzed
        let mut fo_nl_b: FileOffset = fo_nl_a;
        // set for the first loop (first block), then is zero
        let mut bin_beg_init_b: BlockIndex = self.block_index_at_file_offset(fo_nl_b);
        // append LinePart to this `Line`
        let mut line: Line = Line::new();
        bo = self.block_offset_at_file_offset(fo_nl_b);
        while !found_nl_b && bo <= blockoffset_last {
            match self.blockreader.read_block(bo) {
                Ok(val) => {
                    debug_eprintln!(
                        "{}find_line: read_block returned Block @{:p} len {} while searching for newline B",
                        so(),
                        &(*val),
                        (*val).len()
                    );
                    bp = val;
                }
                Err(err) => {
                    if err.kind() == EndOfFile {
                        debug_eprintln!(
                            "{}find_line: read_block returned EndOfFile {:?} while searching for newline B",
                            so(),
                            self.path()
                        );
                        let rl = self.insert_line(line);
                        let fo_ = (*rl).fileoffset_end() + charsz_fo;
                        debug_eprintln!("{}find_line: return ({}, {}, Ok(@{:p}))", sx(), true, fo_, &*rl);
                        return (true, fo_, Ok(rl));
                    }
                    debug_eprintln!("{}find_line: return ({}, {}, Err({:?}))", sx(), false, FileOffset_NULL, err);
                    return (false, FileOffset_NULL, Err(err));
                }
            }
            let blen = (*bp).len() as BlockIndex;
            let bin_beg = bin_beg_init_b;
            let mut bin_end = bin_beg;
            while bin_end < blen {
                // XXX: single-byte encoding
                if (*bp)[bin_end] == NLu8 {
                    found_nl_b = true;
                    fo_nl_b = self.file_offset_at_block_offset_index(bo, bin_end);
                    bin_end += charsz_bi; // refer to one past end
                    debug_eprintln!(
                        "{}find_line: newline B found by byte search fileoffset {} ≟ blockoffset {} blockindex {}",
                        sx(),
                        fo_nl_b,
                        bo,
                        bin_end
                    );
                    break;
                }
                // XXX: single-byte encoding
                bin_end += charsz_bi;
            }
            let fo_beg = self.file_offset_at_block_offset_index(bo, bin_beg);
            // sanity check
            if fo_beg == filesz {
                assert_eq!(bin_end - bin_beg, 0, "fileoffset of beginning of line {} is at end of file, yet found a linepart of length {} (expected zero)", fo_beg, bin_end - bin_beg);
            }
            // sanity check
            if bin_end - bin_beg == 0 {
                assert_eq!(fo_beg, filesz, "fileoffset of beginning of line {} is at end of file, yet found a linepart of length {} (expected zero)", fo_beg, bin_end - bin_beg);
            }
            // at end of file, "zero length" LinePart, skip creating a `LinePart`
            if bin_end - bin_beg == 0 && fo_beg == filesz {
                debug_eprintln!("{}find_line: no newline B, at end of file", so());
                eof = true;
                break;
            }
            let li = LinePart::new(bin_beg, bin_end, bp.clone(), fo_beg, bo, self.blocksz());
            debug_eprintln!("{}find_line: Line.push({:?})", so(), &li);
            line.push(li);
            if found_nl_b {
                break;
            }
            bin_beg_init_b = 0;
            bo += 1;
            if bo > blockoffset_last {
                eof = true;
                break;
            }
        } // ! found_nl_b

        // occurs in files with single newline
        if line.count() == 0 {
            let err = Err(Error::new(NoLinesFound, "No Lines Found!"));
            debug_eprintln!("{}find_line: return ({}, {}, {:?}) no LinePart found!", sx(), eof, fo_nl_b, err);
            return (false, fo_nl_b, err);
        }

        // sanity check
        debug_eprintln!("{}find_line: returning {:?}", so(), line);
        let fo_beg = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        //assert_eq!(fo_beg, fo_nl_a, "line.fileoffset_begin() {} ≠ {} searched fo_nl_a", fo_beg, fo_nl_a);
        //assert_eq!(fo_end, fo_nl_b, "line.fileoffset_end() {} ≠ {} searched fo_nl_b", fo_end, fo_nl_b);
        if fo_beg != fo_nl_a {
            debug_eprintln!("WARNING: line.fileoffset_begin() {} ≠ {} searched fo_nl_a", fo_beg, fo_nl_a);
        }
        if fo_end != fo_nl_b {
            debug_eprintln!("WARNING: line.fileoffset_end() {} ≠ {} searched fo_nl_b", fo_end, fo_nl_b);
        }
        assert_lt!(fo_end, filesz, "line.fileoffset_end() {} is past file size {}", fo_end, filesz);

        let rl = self.insert_line(line);
        debug_eprintln!("{}find_line: return ({}, {}, Ok(@{:p}))", sx(), eof, fo_end + 1, &*rl);
        return (eof, fo_end + 1, Ok(rl));
    }

    fn insert_line(&mut self, line: Line) -> LineP {
        let fo_beg = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let rl = LineP::new(line);
        debug_eprintln!("{}find_line: lines.insert({}, Line @{:p})", so(), fo_beg, &(*rl));
        debug_eprintln!("{}find_line: lines.insert_end({}, Line @{:p})", so(), fo_end, &(*rl));
        self.lines.insert(fo_beg, rl.clone());
        self.lines_end.insert(fo_end, rl.clone());
        return rl;
    }
}

/// basic test of LineReader things
/// simple read of file offsets in order, should print to stdout an identical file
#[allow(non_snake_case, dead_code)]
fn test_LineReader(path_: &String, blocksz: BlockSz) {
    debug_eprintln!("{}test_LineReader({:?}, {})", sn(), &path_, blocksz);
    let mut lr1 = match LineReader::new(path_, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: LineReader::new({}, {}) failed {}", path_, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}LineReader {:?}", so(), lr1);

    let mut fo1: FileOffset = 0;
    loop {
        let (eof, fo2, rlp) = lr1.find_line(fo1);
        fo1 = fo2;
        match rlp {
            Ok(lp) => {
                let _ln = lr1.lines.len();
                debug_eprintln!(
                    "{}FileOffset {} line num {} Line @{:p}: len {} '{}'\n",
                    so(),
                    fo2,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
                (*lp).print(true);
            }
            Err(err) => {
                debug_eprintln!("{}Err {}", so(), err);
                if err.kind() == EndOfFile {
                    assert!(eof, "find_line returned Err(EndOfFile) yet EOF is false (should be true)");
                    break;
                } else if err.kind() == NoLinesFound {
                    break;
                }
                eprintln!("ERROR: {}", err);
                break;
            }
        }
        if eof {
            debug_eprintln!("\n{}EOF!", so());
            break;
        }
    }
    debug_eprintln!("\n{}{:?}", so(), lr1);
    debug_eprintln!("{}test_LineReader({:?}, {})", sx(), &path_, blocksz);
}

fn randomize(v_: &mut Vec<FileOffset>) {
    // XXX: can also use `rand::shuffle` https://docs.rs/rand/0.8.4/rand/seq/trait.SliceRandom.html#tymethod.shuffle
    let sz = v_.len();
    let mut i = 0;
    while i < sz {
        let r = random::<usize>() % sz;
        let tmp = v_[r];
        v_[r] = v_[i];
        v_[i] = tmp;
        i += 1;
    }
}

fn fill(v_: &mut Vec<FileOffset>) {
    let sz = v_.capacity();
    let mut i = 0;
    while i < sz {
        v_.push(i as FileOffset);
        i += 1;
    }
}

/// basic test of LineReader things
/// read all file offsets but randomly
#[allow(non_snake_case, dead_code)]
fn test_LineReader_rand(path_: &String, blocksz: BlockSz) {
    debug_eprintln!("{}test_LineReader_rand({:?}, {})", sn(), &path_, blocksz);
    let mut lr1 = match LineReader::new(path_, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: LineReader::new({}, {}) failed {}", path_, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}LineReader {:?}", so(), lr1);
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(lr1.filesz() as usize);
    fill(&mut offsets_rand);
    debug_eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);
    randomize(&mut offsets_rand);
    debug_eprintln!("{}offsets_rand: {:?}", so(),offsets_rand);

    for fo1 in offsets_rand {
        let (_eof, _fo2, r) = lr1.find_line(fo1);
        match r {
            Ok(_) => {}
            Err(err) => {
                debug_eprintln!("{}Err {}", so(), err);
                if err.kind() != EndOfFile && err.kind() != NoLinesFound {
                    eprintln!("ERROR: {}", err);
                    break;
                }
            }
        }
    }
    // should print the file as-is and not be affected by random reads
    lr1.print_all();
    debug_eprintln!("\n{}{:?}", so(), lr1);
    debug_eprintln!("{}test_LineReader_rand({:?}, {})", sx(), &path_, blocksz);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sysline and SyslogReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A sequence to track one or more `Line` that make up a `Sysline`
/// TODO: rename this `Lines`
type SyslineParts = Vec<LineP>;
/// An offset into a `Line`
type LineIndex = usize;

/// A `Sysline` has information about a "syslog line" that spans one or more `Line`s
/// a "syslog line" is one or more lines, where the first line starts with a
/// datetime stamp. That datetime stamp is consistent format of other nearby syslines.
pub struct Sysline {
    /// the one or more `Line` that make up a Sysline
    /// TODO: rename this lines
    syslineparts: SyslineParts,
    /// index into `Line` where datetime string starts
    /// byte-based count
    /// TODO: does not handle offset into `Line` that is not first `Line`
    dt_beg: LineIndex,
    /// index into `Line` where datetime string ends, one char past last character of datetime string
    /// byte-based count
    /// TODO: does not handle offset into `Line` that is not first `Line`
    dt_end: LineIndex,
}

/// a signifier value for "not set" or "null"
const LI_NULL: LineIndex = LineIndex::MAX;

impl fmt::Debug for Sysline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut li_s = String::new();
        for lp in self.syslineparts.iter() {
            li_s.push_str(
                &format!("Line @{:p} (fileoffset_beg {}, fileoffset_end {}, len() {}, count() {}",
                         &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end(), (*lp).len(), (*lp).count())
            );
        }
        f.debug_struct("Sysline")
            .field("fileoffset_begin()", &self.fileoffset_begin())
            .field("fileoffset_end()", &self.fileoffset_end())
            .field("syslineparts @", &format_args!("{:p}", &self.syslineparts))
            .field("syslineparts.len", &self.syslineparts.len())
            .field("dt_beg", &self.dt_beg)
            .field("dt_end", &self.dt_end)
            .field("lines", &li_s)
            .finish()
    }
}

impl Sysline {
    /// default `with_capacity` for a `SyslineParts`, most often will only need 1 capacity
    /// as the found "sysline" will likely be one `Line`
    const SYSLINE_PARTS_WITH_CAPACITY: usize = 1;
    // XXX: does not handle multi-byte encodings
    const CHARSZ: usize = 1;

    pub fn new() -> Sysline {
        return Sysline {
            syslineparts: SyslineParts::with_capacity(Sysline::SYSLINE_PARTS_WITH_CAPACITY),
            dt_beg: LI_NULL,
            dt_end: LI_NULL,
        };
    }

    pub fn new_from_line(linep: LineP) -> Sysline {
        let mut v = SyslineParts::with_capacity(Sysline::SYSLINE_PARTS_WITH_CAPACITY);
        v.push(linep);
        return Sysline {
            syslineparts: v,
            dt_beg: LI_NULL,
            dt_end: LI_NULL,
        };
    }

    pub fn push(&mut self, linep: LineP) {
        if self.syslineparts.len() > 0 {
            // TODO: sanity check lines are in sequence
        }
        debug_eprintln!("{}syslinereader.push(@{:p}), self.syslineparts.len() is now {}", so(), &*linep, self.syslineparts.len() + 1);
        self.syslineparts.push(linep);
    }

    /// the byte offset into the file where this `Sysline` begins
    pub fn fileoffset_begin(self: &Sysline) -> FileOffset {
        assert_ne!(self.syslineparts.len(), 0, "This Sysline has no Line");
        (*self.syslineparts[0]).fileoffset_begin()
    }

    /// the byte offset into the file where this `Sysline` ends, inclusive (not one past ending)
    pub fn fileoffset_end(self: &Sysline) -> FileOffset {
        assert_ne!(self.syslineparts.len(), 0, "This Sysline has no Line");
        let last_ = self.syslineparts.len() - 1;
        (*self.syslineparts[last_]).fileoffset_end()
    }

    pub fn charsz(self: &Sysline) -> usize {
        Sysline::CHARSZ
    }

    /// length in bytes of the Sysline
    pub fn len(self: &Sysline) -> usize {
        (self.fileoffset_end() - self.fileoffset_begin() + 1) as usize
    }

    /// count of `Line` in `self.syslineparts`
    pub fn count(self: &Sysline) -> usize {
        self.syslineparts.len()
    }

    /// a `String` copy of the demarcating datetime string found in the Sysline
    /// TODO: does not handle datetime spanning multiple lines... is that even possible? No...
    #[allow(non_snake_case)]
    pub fn datetime_String(self: &Sysline) -> String {
        assert_ne!(self.dt_beg, LI_NULL, "dt_beg has not been set");
        assert_ne!(self.dt_end, LI_NULL, "dt_end has not been set");
        assert_lt!(self.dt_beg, self.dt_end, "bad values dt_end {} dt_beg {}", self.dt_end, self.dt_beg);
        let slice_ = self.syslineparts[0].as_slice();
        assert_lt!(self.dt_beg, slice_.len(), "dt_beg {} past end of slice[{}..{}]?", self.dt_beg, self.dt_beg, self.dt_end);
        assert_le!(self.dt_end, slice_.len(), "dt_end {} past end of slice[{}..{}]?", self.dt_end, self.dt_beg, self.dt_end);
        // TODO: here is a place to use `bstr`
        let buf: &[u8] = &slice_[self.dt_beg..self.dt_end];
        let s_ = match str::from_utf8(buf) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Error in datetime_String() during str::from_utf8 {} buffer {:?}", err, buf);
                ""
            },
        };
        String::from(s_)
    }
    
    /// return all the slices that make up this `Sysline`
    pub fn get_slices(self: &Sysline) -> Slices {
        let mut sz: usize = 0;
        for lp in &self.syslineparts {
            sz += lp.get_slices_count();
        }
        let mut slices = Slices::with_capacity(sz);
        for lp in &self.syslineparts {
            slices.extend(lp.get_slices().iter());
        }
        return slices;
    }

    /// `raw` true will write directly to stdout from the stored `Block`
    /// `raw` false will write transcode each byte to a character and use pictoral representations
    /// XXX: `raw==false` does not handle multi-byte encodings
    pub fn print(self: &Sysline, raw: bool) {
        for lp in &self.syslineparts {
            (*lp).print(raw);
        }
    }
    
    /// create `String` from `self.syslineparts`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_nonraw_char`)
    /// TODO: this would be more efficient returning `&str`
    ///       https://bes.github.io/blog/rust-strings
    #[allow(non_snake_case)]
    fn _to_String_raw(self: &Sysline, raw: bool) -> String {
        let mut sz: usize = 0;
        for lp in &self.syslineparts {
            sz += (*lp).len();
        }
        let mut s_ = String::with_capacity(sz);
        for lp in &self.syslineparts {
            s_ += (*lp)._to_String_raw(raw).as_str();
        }
        return s_;
    }

    // XXX: rust does not support function overloading which is really surprising and disappointing
    /// `Line` to `String`
    #[allow(non_snake_case)]
    pub fn to_String(self: &Sysline) -> String {
        return self._to_String_raw(true);
    }

    #[allow(non_snake_case)]
    pub fn to_String_from(self: &Sysline, _from: usize) -> String {
        assert!(false, "not implemented");
        return String::from("stub!");
    }

    #[allow(non_snake_case)]
    pub fn to_String_from_to(self: &Sysline, _from: usize, _to: usize) -> String {
        assert!(false, "not implemented");
        return String::from("stub!");
    }

    /// `Sysline` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    pub fn to_String_noraw(self: &Sysline) -> String {
        return self._to_String_raw(false);
    }

}

/// reference counting pointer to a `Sysline`
type SyslineP = Rc<Sysline>;
/// storage for `Sysline`
type Syslines = BTreeMap<FileOffset, SyslineP>;

/// Specialized Reader that uses `LineReader` to find syslog lines
pub struct SyslineReader<'syslinereader> {
    linereader: LineReader<'syslinereader>,
    syslines: Syslines,
}

impl fmt::Debug for SyslineReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyslineReader")
            .field("linereader", &self.linereader)
            .field("syslines", &self.syslines)
            .finish()
    }
}

/// implement SyslineReader things
impl<'syslinereader> SyslineReader<'syslinereader> {
    // XXX: does not handle multi-byte encodings
    const CHARSZ: usize = 1;

    pub fn new(path: &'syslinereader String, blocksz: BlockSz) -> Result<SyslineReader<'syslinereader>> {
        let lr = match LineReader::new(&path, blocksz) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("ERROR: LineReader::new({}, {}) failed {}", path, blocksz, err);
                return Err(err);
            }
        };
        Ok(SyslineReader { linereader: lr, syslines: Syslines::new(), })
    }

    pub fn blocksz(&self) -> BlockSz {
        self.linereader.blocksz()
    }

    pub fn filesz(&self) -> BlockSz {
        self.linereader.filesz()
    }

    pub fn path(&self) -> &str {
        self.linereader.path()
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    pub fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        self.linereader.block_offset_at_file_offset(fileoffset)
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    pub fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        self.linereader.file_offset_at_block_offset(blockoffset)
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        self.linereader
            .file_offset_at_block_offset_index(blockoffset, blockindex)
    }

    /// return block index at given `FileOffset`
    pub fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        self.linereader.block_index_at_file_offset(fileoffset)
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    pub fn file_blocks_count(&self) -> u64 {
        self.linereader.file_blocks_count()
    }

    pub fn blockoffset_last(&self) -> BlockOffset {
        self.linereader.blockoffset_last()
    }

    pub fn print(&self, fileoffset: FileOffset, raw: bool) {
        let syslinep: &SyslineP = match self.syslines.get(&fileoffset) {
            Some(val) => {val},
            None => {
                eprintln!("ERROR: in print, self.syslines.get({}) returned None", fileoffset);
                return;
            },
        };
        for linep in &(*syslinep).syslineparts {
            (*linep).print(raw);
        }
    }
    
    /// Testing helper only
    /// print all known `Sysline`s
    pub fn print_all(&self, raw: bool) {
        for fo in self.syslines.keys() {
            self.print(*fo, raw);
        }
    }
    
    /// store passed `Sysline` in `self.syslines`
    fn insert_line(&mut self, line: Sysline) -> SyslineP {
        let fo_beg = line.fileoffset_begin();
        let slp = SyslineP::new(line);
        debug_eprintln!("{}syslinereader.insert_line: syslines.insert({}, Sysline @{:p})", so(), fo_beg, &*slp);
        self.syslines.insert(fo_beg, slp.clone());
        return slp;
    }
    
    /// return indexes into `line` of found datetime string `(start of string, end of string)`
    /// XXX: stub implementation
    pub fn find_datetime_in_line(&self, line: &Line) -> (LineIndex, LineIndex) {
        debug_eprintln!("{}find_datetime_in_line(Line@{:p})", sn(), &line);
        // stub search, look for year within first few bytes
        if line.len() < 4 {
            return (LI_NULL, LI_NULL);
        }
        // TODO: allow `as_slice_first_X` that return slice of first X bytes, most cases only need first 30 or so bytes of line
        let slice_ = line.as_slice();
        // TODO: here is a place to use `bstr`
        let mut bia: usize = 0;
        // pattern '^[2018|2019|2020|2021]'
        while bia < 8 && bia < (slice_.len() - 4) {
            let buf: &[u8] = &slice_[bia..bia+4];
            let s = match str::from_utf8(buf) {
                Ok(val) => val,
                Err(_) => continue,
            };
            if s == "2018" || s == "2019" || s == "2020" || s == "2021" {
                debug_eprintln!("{}find_datetime_in_line returning ({}, {})", sx(), bia, bia + 4);
                return (
                    bia as LineIndex,
                    (bia + 4) as LineIndex,
                );
            }
            bia += 1;
        }
        if slice_.len() < 11 {
            debug_eprintln!("{}find_datetime_in_line returning (0x{:x}, 0x{:x})", sx(), LI_NULL, LI_NULL);
            return (LI_NULL, LI_NULL);
        }
        // pattern '^\[2020/03/05'
        // example
        // [2020/03/05 12:17:59.631000,  3] ../source3/smbd/oplock.c:1340(init_oplocks)
        //    init_oplocks: initializing messages.
        //
        // 012345678901
        loop {
            let buf: &[u8] = &slice_[0..11];
            let s = match str::from_utf8(buf) {
                Ok(val) => val,
                Err(_) => break,
            };
            let year = &s[1..5];
            if ! (year == "2018" || year == "2019" || year == "2020" || year == "2021") {
                break;
            }
            let slash = &s[5..6];
            if slash != "/" {
                break;
            }
            let m = &s[6..8];  // month
            if ! ( m == "01" || m == "02" || m == "03" || m == "04" || m == "05" || m == "06" ||
                   m == "07" || m == "08" || m == "09" || m == "10" || m == "11" || m == "23" ) {
                break;
            }
            let slash = &s[8..9];
            if slash != "/" {
                break;
            }
            let d0 = &s[9..10];  // day0
            if ! ( m == "0" || m == "1" || m == "2" || m == "3" ) {
                break;
            }
            let d1 = &s[10..11];  // day1
            if ! ( m == "0" || m == "1" || m == "2" || m == "3" || m == "4" || 
                   m == "5" || m == "6" || m == "7" || m == "8" || m == "9") {
                break;
            }
            debug_eprintln!("{}find_datetime_in_line returning ({}, {})", sx(), 1, 10);
            return (1, 10);
        }
        debug_eprintln!("{}find_datetime_in_line returning (0x{:x}, 0x{:x})", sx(), LI_NULL, LI_NULL);
        return (LI_NULL, LI_NULL);
    }
    
    /// find next sysline at or after `fileoffset`
    /// return (eof?, fileoffset of start of next sysline, new Sysline)
    pub fn find_sysline(&mut self, fileoffset: FileOffset) -> (bool, FileOffset, Result<SyslineP>) {
        debug_eprintln!("{}find_sysline(SyslingReader@{:p}, {})", sn(), self, fileoffset);

        // first check if there is a sysline already known at this fileoffset
        if self.syslines.contains_key(&fileoffset) {
            debug_eprintln!("{}find_sysline: hit cache for FileOffset {}", so(), fileoffset);
            let slp = self.syslines[&fileoffset].clone();
            // XXX: multi-byte character encoding
            let fo_next = (*slp).fileoffset_end() + (SyslineReader::CHARSZ as FileOffset);
            // TODO: determine if `fileoffset` is the last sysline of the file
            //       should add a private helper function for this task `is_sysline_last(FileOffset)` ... something like that
            debug_eprintln!("{}find_sysline: return ({}, {}, Ok(@{:p}))", sx(), false, fo_next, &*slp);
            return (false, fo_next, Ok(slp));
        }
        debug_eprintln!("{}find_sysline: no cache hit, searching for first sysline datetime A", so());
        
        //
        // find line with datetime A
        //

        let mut fo_a: FileOffset = 0;
        let mut fo1: FileOffset = fileoffset;
        let mut sl = Sysline::new();
        let mut eof: bool = false;
        loop {
            let (eof_, fo2, rlp) = self.linereader.find_line(fo1);
            // TODO: the bool `eof` and the `Err(EndOfFile)` achieve the same purpose, get rid of one?
            eof = eof_;
            let lp: LineP = match rlp {
                Ok(lp_) => {
                    debug_eprintln!(
                        "{}find_sysline: A FileOffset {} Line @{:p} len {} parts {} '{}'",
                        so(),
                        fo2,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    lp_
                }
                Err(err) => {
                    debug_eprintln!("{}find_sysline: A Err {}", so(), err);
                    if err.kind() == EndOfFile {
                        assert!(eof, "linereader.find_line returned Err(EndOfFile) yet EOF is false (should be true)");
                        debug_eprintln!("{}find_sysline: A return ({}, {}, Err({:?}))", sx(), true, fo2, err);
                        return (true, fo2, Err(err));
                    } else if err.kind() == NoLinesFound {
                        debug_eprintln!("{}find_sysline: A return ({}, {}, Err({:?}))", sx(), false, fo2, err);
                        return (false, fo2, Err(err));
                    }
                    eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    break;
                }
            };
            if eof {
                debug_eprintln!("WARNING: find_line returned EOF as true yet did not return Err(EndOfFile)");
            }
            let (dt_beg, dt_end) = self.find_datetime_in_line(&*lp);
            debug_eprintln!("{}find_sysline: A find_datetime_in_line returned ({}, {})", so(), dt_beg, dt_end);
            if dt_beg != LI_NULL {
                // a datetime was found! beginning of a sysline
                fo_a = fo1;
                sl.dt_beg = dt_beg;
                sl.dt_end = dt_end;
                debug_eprintln!("{}find_sysline: A sl.push('{}')", so(), (*lp).to_String_noraw());
                sl.push(lp);
                fo1 = sl.fileoffset_end();
                break;
            }
            debug_eprintln!("{}find_sysline: A skip push Line '{}'", so(), (*lp).to_String_noraw());
            fo1 = fo2;
        }

        debug_eprintln!("{}find_sysline: found line with datetime A at FileOffset {}, serach for datetime B starting at {}", so(), fo_a, fo1);

        //
        // find line with datetime B
        //

        let mut fo_b: FileOffset = fo1;
        loop {
            let (eof_, fo2, rlp) = self.linereader.find_line(fo1);
            // TODO: the bool `eof` and the `Err(EndOfFile)` achieve the same purpose, get rid of one?
            eof = eof_;
            let lp: LineP = match rlp {
                Ok(lp_) => {
                    debug_eprintln!(
                        "{}find_sysline: B FileOffset {} Line @{:p} len {} parts {} '{}'\n",
                        so(),
                        fo2,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    //assert!(!eof, "ERROR: find_line returned EOF as true yet returned Ok()");
                    lp_
                }
                Err(err) => {
                    debug_eprintln!("{}find_sysline: Err {}", so(), err);
                    if err.kind() == EndOfFile {
                        assert!(eof, "linereader.find_line returned Err(EndOfFile) yet EOF is false (should be true)");
                        eof = true;
                        break;
                        //debug_eprintln!("{}find_sysline: B return ({}, {}, Err({:?}))", sx(), true, fo2, err);
                        //return (true, fo2, Err(err));
                    } else if err.kind() == NoLinesFound {
                        eof = true;
                        break;
                        //debug_eprintln!("{}find_sysline: B return ({}, {}, Err({:?}))", sx(), false, fo2, err);
                        //return (false, fo2, Err(err));
                    }
                    eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    assert!(!eof, "ERROR: find_line returned EOF true yet the Error was {}, not EndOfFile", err);
                    debug_eprintln!("{}find_sysline: B return ({}, {}, Err({:?}))", sx(), false, fo2, err);
                    return (false, fo2, Err(err));
                }
            };
            let (dt_beg, dt_end) = self.find_datetime_in_line(&*lp);
            debug_eprintln!("{}find_sysline: B find_datetime_in_line returned ({}, {})", so(), dt_beg, dt_end);
            if dt_beg != LI_NULL {
                // a datetime was found! end of this sysline, beginning of a new sysline
                debug_eprintln!("{}find_sysline: B skip push Line '{}'", so(), (*lp).to_String_noraw());
                fo_b = fo1;
                break;
            } else {
                debug_eprintln!("{}find_sysline: B sl.push('{}')", so(), (*lp).to_String_noraw());
                sl.push(lp);
            }
            fo1 = fo2;
        }

        debug_eprintln!("{}find_sysline: found line with datetime B at FileOffset {}", so(), fo_b);

        debug_eprintln!("{}find_sysline: self.insert_line({:p})", so(), &sl);
        let slp = self.insert_line(sl);
        debug_eprintln!("{}find_sysline: return ({}, {}, Ok(@{:p}))", sx(), eof, fo_b, &*slp);
        return (eof, fo_b, Ok(slp));
    }
}

/// basic test of SyslineReader things
#[allow(non_snake_case, dead_code)]
fn test_SyslineReader(path: &String, blocksz: BlockSz) {
    debug_eprintln!("{}test_SyslineReader({:?}, {})", sn(), &path, blocksz);
    let mut slr = match SyslineReader::new(path, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({}, {}) failed {}", path, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}{:?}", so(), slr);

    fn print_slp(slp_: &SyslineP) {
        if cfg!(debug_assertions) {
            let out = (*slp_).to_String_noraw();
            match print_colored(Color::Green, &out.as_bytes()) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("ERROR: print_colored returned error {}", err);
                }
            };
            println!();
        } else {
            //print!("Sysline {:02} [{}]: ", i, (*slp).datetime_String());
            (*slp_).print(true);
        }
    }
    
    let mut fo1: FileOffset = 0;
    loop {
        let (eof, fo2, rslp) = slr.find_sysline(fo1);
        fo1 = fo2;
        match rslp {
            Ok(slp) => {
                debug_eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} '{}'\n",
                    so(),
                    fo2,
                    &(*slp),
                    slp.syslineparts.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                //print_slp(&slp);
                let slices = (*slp).get_slices();
                for slice in slices.iter() {
                    write(slice);
                }
            }
            Err(err) => {
                debug_eprintln!("{}Err {}", so(), err);
                if err.kind() == EndOfFile {
                    assert!(eof, "find_sysline returned Err(EndOfFile) yet EOF is false (should be true)");
                    break;
                } else if err.kind() == NoLinesFound {
                    break;
                }
                eprintln!("ERROR: {}", err);
                break;
            }
        }
        if eof {
            debug_eprintln!("\n{}EOF!", so());
            break;
        }
    }
    debug_eprintln!("\n{}{:#?}", so(), slr);

    debug_eprintln!("{}test_SyslineReader({:?}, {})", sx(), &path, blocksz);
}

/// basic test of SyslineReader things
/// read all file offsets but randomly
#[allow(non_snake_case, dead_code)]
fn test_SyslineReader_rand(path_: &String, blocksz: BlockSz) {
    debug_eprintln!("{}test_SyslineReader_rand({:?}, {})", sn(), &path_, blocksz);
    let mut slr1 = match SyslineReader::new(path_, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: LineReader::new({}, {}) failed {}", path_, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}LineReader {:?}", so(), slr1);
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(slr1.filesz() as usize);
    fill(&mut offsets_rand);
    debug_eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);
    randomize(&mut offsets_rand);
    debug_eprintln!("{}offsets_rand: {:?}", so(),offsets_rand);

    for fo1 in offsets_rand {
        let (_eof, _fo2, r) = slr1.find_sysline(fo1);
        match r {
            Ok(_) => {}
            Err(err) => {
                debug_eprintln!("{}Err {}", so(), err);
                if err.kind() != EndOfFile && err.kind() != NoLinesFound {
                    eprintln!("ERROR: {}", err);
                    break;
                }
            }
        }
    }
    // should print the file as-is and not be affected by random reads
    slr1.print_all(true);
    debug_eprintln!("\n{}{:?}", so(), slr1);
    debug_eprintln!("{}test_SyslineReader_rand({:?}, {})", sx(), &path_, blocksz);
}