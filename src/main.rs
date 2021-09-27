// main.rs
/* …
Successful `cat`. Passes all tests in run-tests including utf-8 with high-order characters.

(export RUST_BACKTRACE=1; cargo run -- --filepath Cargo.toml)
(cargo build && rust-gdb -ex 'layout split' -ex 'b src/main.rs:2062' -ex 'r' --args target/debug/block_reader_speedy --filepath /mnt/c/Users/ulug/Projects/syslog-datetime-searcher/logs/other/tests/basic-dt.log 2>/dev/null)
(export RUST_BACKTRACE=1; cargo run -- --filepath /mnt/c/Users/ulug/Projects/syslog-datetime-searcher/logs/other/tests/test3-hex.log)

(export CARGO_PROFILE_RELEASE_DEBUG=true;
 export PERF=/usr/lib/linux-tools/5.4.0-84-generic/perf;
 set -x;
 cargo build --release && flamegraph -o flame-S4.cvg ./target/release/super_speedy_syslog_searcher -f ./logs/Ubuntu18/samba/log.10.7.190.134 1024
)

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

LAST WORKING ON [2021/09/05]
    seems to work as a replacement `cat`! :-)
    Add special debug helper function to `BLockReader` and `LineReader` to print
    current known data but in correct file order (not in the order it was accessed): `fn print_known_data`
    Then do similar test but only print some section of the input file. Like first quarter, then middle, then last quarter.
    Consider hiding all these test functions behind a `--test` option. If `--test` is not passed, then just
    behave like `cat`.
    After all that, I think the `SyslogReader` can be started.

LAST WORKING ON [2021/09/09]
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

LAST WORKING ON [2021/09/15]
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

LAST WORKING ON [2021/09/16]
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
    can this run faster than the Unix script version? `cat`, `sort`, `grep`, etc.
    -
    Big milestones, in recommended order:
    - datetime filtering
    - datetime binary search processing
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

TODO: [2021/09/16]
      clean up the confusing use Result. Create your own Result Enum that copies what is necessary
      from built-in code.

LAST WORKING ON [2021/09/17]
    Fixing `find_datetime_in_line`, and then store the `DateTime` instances.
    Then need to think about how to use the `DateTime` instances. Maybe a BTreeMap<&DateTime, SyslineP> ?
    I might want to remove `test_find_datetime_in_line` and just use `test_SyslineReader`.

TODO: [2021/09/17]
    If a function does not need `self` then remove it. Simpler, testable.

TODO: [2021/09/20]
      Better distinguish "byte lengths" and "character lengths".
      i.e. rename functions like `len` to `byte_len` or `char_len`.
      or to `size` (bytes) and `len` (characters).
      Also rename various `*Index` to `ByteIndex` or `CharIndex`.
      Also rename various `Offset` to `ByteOffset` or `CharOffset`.

LAST WORKING ON [2021/09/20]
     Tried out flamegraph for fun.
     Now to convert `BlockReader.read_block` to use it's own typed `ResultS4`.
     Then fix the zero size bug, then resume work on function called by `test_SyslineReader_w_filtering`.

BUG: [2021/09/20] file of zero size, or single line causes a crash.

LAST WORKING ON [2021/09/22]
     Revised debug indent printing.
     First implement the `fname` macro (search for it) mentioned, then replace debug prints.
     Then resume implementing `find_sysline_at_datetime_filter`. It's only job is to find one sysline
     closest to passed datetime filter and fileoffset. No need to loop on it.

*/

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// uses and types
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

use std::collections::BTreeMap;
use std::fmt;
use std::fs::{File, Metadata, OpenOptions};
use std::io;
use std::io::prelude::Read;
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};
use std::path::Path;
use std::rc::Rc;
use std::str;

extern crate atty;

extern crate backtrace;

extern crate clap;
use clap::{App, Arg};

extern crate chrono;
use chrono::{DateTime, Local, TimeZone};

extern crate debug_print;
#[allow(unused_imports)]
use debug_print::{debug_eprint, debug_eprintln, debug_print, debug_println};

extern crate lru;
use lru::LruCache;

#[macro_use]
extern crate more_asserts;

extern crate rand;
use rand::random;

extern crate rangemap;
use rangemap::RangeMap;

extern crate tempfile;
use tempfile::NamedTempFile;

extern crate termcolor;
use termcolor::{Color, ColorSpec, WriteColor};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// misc. globals
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// global constants

/// NewLine as char
#[allow(non_upper_case_globals, dead_code)]
static NLc: char = '\n';
/// Single-byte newLine char as u8
#[allow(non_upper_case_globals)]
static NLu8: u8 = 10;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// custom Results enums for various *Reader functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#481-495

/// `Result` `Ext`ended
/// sometimes things are not `Ok` but a value needs to be returned
#[derive(Debug)]
pub enum ResultS4<T, E> {
    /// Contains the success value (LineP, SyslineP, etc.)
    /// TODO: change to "Found"
    Ok(T),

    /// Contains the success value (LineP, SyslineP, etc.), reached End Of File, but things are okay
    /// TODO: change to "Found_EOF"
    #[allow(non_camel_case_types)]
    Ok_EOF(T),

    /// File is empty, or other condition that means "Done", nothing to return, but things no bad errors happened
    /// TODO: change to "Done"
    #[allow(non_camel_case_types)]
    Ok_Done,

    /// Contains the error value, something went wrong
    Err(E),
}

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#501-659
// XXX: how to link to specific version of `result.rs`?

impl<T, E> ResultS4<T, E> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the result is [`Ok`, `Ok_EOF`, 'Ok_Done`].
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, ResultS4::Ok(_) | ResultS4::Ok_EOF(_) | ResultS4::Ok_Done)
    }

    /// Returns `true` if the result is [`Err`].
    #[must_use = "if you intended to assert that this is err, consider `.unwrap_err()` instead"]
    #[inline]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Returns `true` if the result is [`Ok_EOF`].
    #[inline]
    pub const fn is_eof(&self) -> bool {
        matches!(*self, ResultS4::Ok_EOF(_))
    }

    /// Returns `true` if the result is [`Ok_EOF`, `Ok_Done`].
    #[inline]
    pub const fn is_done(&self) -> bool {
        matches!(*self, ResultS4::Ok_Done)
    }

    /// Returns `true` if the result is an [`Ok`, `Ok_EOF`] value containing the given value.
    #[must_use]
    #[inline]
    pub fn contains<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            ResultS4::Ok(y) => x == y,
            ResultS4::Ok_EOF(y) => x == y,
            ResultS4::Ok_Done => false,
            ResultS4::Err(_) => false,
        }
    }

    /// Returns `true` if the result is an [`Err`] value containing the given value.
    #[must_use]
    #[inline]
    pub fn contains_err<F>(&self, f: &F) -> bool
    where
        F: PartialEq<E>,
    {
        match self {
            ResultS4::Err(e) => f == e,
            _ => false,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Adapter for each variant
    /////////////////////////////////////////////////////////////////////////

    /// Converts from `Result<T, E>` to [`Option<T>`].
    ///
    /// Converts `self` into an [`Option<T>`], consuming `self`,
    /// and discarding the error, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.ok(), Some(2));
    ///
    /// let x: Result<u32, &str> = Err("Nothing here");
    /// assert_eq!(x.ok(), None);
    /// ```
    #[inline]
    pub fn ok(self) -> Option<T> {
        match self {
            ResultS4::Ok(x) => Some(x),
            ResultS4::Ok_EOF(x) => Some(x),
            ResultS4::Ok_Done => None,
            ResultS4::Err(_) => None,
        }
    }

    /// Converts from `Result<T, E>` to [`Option<E>`].
    ///
    /// Converts `self` into an [`Option<E>`], consuming `self`,
    /// and discarding the success value, if any.
    #[inline]
    pub fn err(self) -> Option<E> {
        match self {
            ResultS4::Ok(_) => None,
            ResultS4::Ok_EOF(_) => None,
            ResultS4::Ok_Done => None,
            ResultS4::Err(x) => Some(x),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions - debug printing indentation
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// stackdepth in `main`, should set once, use `stackdepth_main` to read
static mut _STACKDEPTH_MAIN: usize = usize::MAX;

/// wrapper for accessing `_STACKDEPTH_MAIN`
fn stackdepth_main() -> usize {
    unsafe { _STACKDEPTH_MAIN }
}

/// return current stack depth according to `backtrace::trace`, including this
/// function
#[allow(dead_code)]
fn stack_depth() -> usize {
    let mut sd: usize = 0;
    backtrace::trace(|_| {
        sd += 1;
        true
    });
    sd
}

/// return stack offset compared to stack depth `_STACKDEPTH_MAIN` recorded in `main`
#[allow(dead_code)]
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

/// set _STACKDEPTH_MAIN, once do this once
fn stackdepth_main_set() {
    unsafe {
        assert_eq!(usize::MAX, _STACKDEPTH_MAIN, "_STACKDEPTH_MAIN has already been set; must only be set once");
        _STACKDEPTH_MAIN = stack_offset();
    }
}

#[allow(dead_code)]
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
#[allow(dead_code)]
fn so() -> &'static str {
    let so_ = stack_offset();
    return match so_ {
        0 => " ",
        1 => "     ",
        2 => "         ",
        3 => "             ",
        4 => "                 ",
        5 => "                     ",
        6 => "                         ",
        7 => "                             ",
        8 => "                                 ",
        9 => "                                     ",
        10 => "                                        ",
        11 => "                                            ",
        12 => "                                                ",
        13 => "                                                    ",
        14 => "                                                        ",
        15 => "                                                            ",
        16 => "                                                                ",
        17 => "                                                                    ",
        18 => "                                                                        ",
        19 => "                                                                            ",
        _ => "                                                                                ",
    };
}

/// `print` helper, a `s`tring for e`n`tering a function
#[allow(dead_code)]
fn sn() -> &'static str {
    let so_ = stack_offset();
    return match so_ {
        0 => "→",
        1 => "    →",
        2 => "        →",
        3 => "            →",
        4 => "                →",
        5 => "                    →",
        6 => "                        →",
        7 => "                            →",
        8 => "                                →",
        9 => "                                    →",
        10 => "                                       →",
        11 => "                                           →",
        12 => "                                               →",
        13 => "                                                   →",
        14 => "                                                       →",
        15 => "                                                           →",
        16 => "                                                               →",
        17 => "                                                                   →",
        18 => "                                                                       →",
        19 => "                                                                           →",
        _ => "                                                                               →",
    };
}

/// `print` helper, a `s`tring for e`x`iting a function
#[allow(dead_code)]
fn sx() -> &'static str {
    let so_ = stack_offset();
    return match so_ {
        0 => "←",
        1 => "    ←",
        2 => "        ←",
        3 => "            ←",
        4 => "                ←",
        5 => "                    ←",
        6 => "                        ←",
        7 => "                            ←",
        8 => "                                ←",
        9 => "                                    ←",
        10 => "                                        ←",
        11 => "                                            ←",
        12 => "                                                ←",
        13 => "                                                    ←",
        14 => "                                                        ←",
        15 => "                                                            ←",
        16 => "                                                                ←",
        17 => "                                                                    ←",
        18 => "                                                                        ←",
        19 => "                                                                            ←",
        _ => "                                                                                ←",
    };
}

// TODO: [2021/09/22]
//       create new macro for current function name `fname`
//       macro function_name!() prints all parents `A::B::my_func`, just print `my_func`.
//       can be ripped from https://github.com/popzxc/stdext-rs/blob/2179f94475f925a2eacdc2f2408d7ab352d0052c/src/macros.rs#L44-L74
//       could possibly use `backtrace::trace` and return this as part of `so`, `sn`, `sx` ???
/*
fn fno() -> () {
    let bt = backtrace::Backtrace::new();
    let frames = bt.frames();
    dbg!(frames);
    for f in frames.iter() {
        dbg!(f);
        debug_eprintln!("\n");
        for s in f.symbols() {
            dbg!(s);
        }
        debug_eprintln!("\n\n\n");
    }
    frames[1].symbols()[0];
    debug_eprintln!("\n\n\n");
    panic!();
}
*/

/// quickie test for debug helpers `sn`, `so`, `sx`
#[allow(dead_code)]
pub fn test_sn_so_sx() {
    fn depth1() {
        debug_eprintln!("{}depth1 enter", sn());
        fn depth2() {
            debug_eprintln!("{}depth2 enter", sn());
            fn depth3() {
                debug_eprintln!("{}depth3 enter", sn());
                fn depth4() {
                    debug_eprintln!("{}depth4 enter", sn());
                    debug_eprintln!("{}depth4 middle", so());
                    debug_eprintln!("{}depth4 exit", sx());
                }
                debug_eprintln!("{}depth3 middle before", so());
                depth4();
                debug_eprintln!("{}depth3 middle after", so());
                debug_eprintln!("{}depth3 exit", sx());
            }
            debug_eprintln!("{}depth2 middle before", so());
            depth3();
            debug_eprintln!("{}depth2 middle after", so());
            debug_eprintln!("{}depth2 exit", sx());
        }
        debug_eprintln!("{}depth1 middle before", so());
        depth2();
        debug_eprintln!("{}depth1 middle after", so());
        debug_eprintln!("{}depth1 exit", sx());
    }
    depth1();
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions - various print and write
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// turn passed u8 into char, for any char values that are CLI formatting instructions transform
/// them to pictoral representations, e.g. '\n' returns a pictoral unicode representation '␊'
/// only intended for debugging
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

/// tranform utf-8 byte (presumably) to non-raw char
/// only intended for debugging
#[allow(dead_code)]
fn byte_to_nonraw_char(byte: u8) -> char {
    return char_to_nonraw_char(byte as char);
}

/// transform buffer of utf-8 chars (presumably) to a non-raw String
/// inefficient
/// only intended for debugging
#[allow(non_snake_case, dead_code)]
fn buffer_to_nonraw_String(buffer: &[u8]) -> String {
    let s1 = match str::from_utf8(&buffer) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: buffer_to_nonraw_String: Invalid UTF-8 sequence during from_utf8: {}", err);
            return String::with_capacity(0);
        }
    };
    let mut s2 = String::with_capacity(s1.len() + 10);
    for c in s1.chars() {
        let c_ = char_to_nonraw_char(c);
        s2.push(c_);
    }
    return s2;
}

/// transform str to non-raw String version
/// only intended for debugging
#[allow(non_snake_case, dead_code)]
fn str_to_nonraw_String(str_buf: &str) -> String {
    let mut s2 = String::with_capacity(str_buf.len() + 1);
    for c in str_buf.chars() {
        let c_ = char_to_nonraw_char(c);
        s2.push(c_);
    }
    return s2;
}

/// return contents of file utf-8 chars (presumably) at `path` as non-raw String
/// inefficient
/// only intended for debugging
#[allow(non_snake_case, dead_code)]
fn file_to_nonraw_String(path: &String) -> String {
    let path_ = Path::new(path);
    let mut open_options = OpenOptions::new();
    let mut file_ = match open_options.read(true).open(&path_) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: File::open('{:?}') error {}", path_, err);
            return String::with_capacity(0);
        }
    };
    let filesz = match file_.metadata() {
        Ok(val) => val.len() as usize,
        Err(err) => {
            eprintln!("ERROR: File::metadata() error {}", err);
            return String::with_capacity(0);
        }
    };
    let mut s2 = String::with_capacity(filesz + 1);
    let s2read = match file_.read_to_string(&mut s2) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: File::read_to_string() error {}", err);
            return String::with_capacity(0);
        }
    };
    assert_eq!(
        s2read, filesz,
        "Read {} bytes but expected to read file size count of bytes {} for file '{}'",
        s2read, filesz, path
    );
    let mut s3 = String::with_capacity(filesz + 1);
    for c in s2.chars() {
        let c_ = char_to_nonraw_char(c);
        s3.push(c_);
    }
    return s3;
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
        Ok(_) => {}
        Err(err) => {
            eprintln!("print_colored: stdout.set_color({:?}) returned error {}", color, err);
            return Err(err);
        }
    };
    match stdout.write(value) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("print_colored: stdout.write(…) returned error {}", err);
            return Err(err);
        }
    }
    match stdout.reset() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("print_colored: stdout.reset() returned error {}", err);
            return Err(err);
        }
    }
    stdout.flush()?;
    Ok(())
}

/// write the `buffer` to stdout
pub fn write(buffer: &[u8]) {
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    match stdout_lock.write(buffer) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: write: StdoutLock.write(@{:?}) error {}", buffer, err);
        }
    }
    match stdout_lock.flush() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: write: stdout flushing error {}", err);
        }
    }
}

/// write to console, `raw` as `true` means "as-is"
/// else use `char_to_nonraw_char` to replace chars in `buffer` (inefficient)
/// only intended for debugging
pub fn pretty_print(buffer: &[u8], raw: bool) {
    if raw {
        return write(buffer);
    }
    // is this an expensive command? should `stdout` be cached?
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    // XXX: only handle single-byte encodings
    // XXX: doing this char by char is probably not efficient
    //let s = match str::from_utf8_lossy(buffer) {
    let s = match str::from_utf8(&buffer) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: pretty_print: Invalid UTF-8 sequence during from_utf8: {}", err);
            return;
        }
    };
    let mut dst: [u8; 4] = [0, 0, 0, 0];
    for c in s.chars() {
        let c_ = char_to_nonraw_char(c);
        let _cs = c_.encode_utf8(&mut dst);
        match stdout_lock.write(&dst) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: pretty_print: StdoutLock.write({:?}) error {}", &dst, err);
            }
        }
    }
    match stdout_lock.flush() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: pretty_print: stdout flushing error {}", err);
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// main
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub fn main() -> std::result::Result<(), chrono::format::ParseError> {
    let dt_filter_pattern: &str = "%Y%m%dT%H%M%S";
    let dt_example: &str = "20200102T123000";
    let matches = App::new("super speedy syslog searcher")
        .version("0.0.1")
        .author("JTM")
        .about("Reads a file block")
        .arg(
            Arg::with_name("filepath")
                .short("f")
                .long("filepath")
                .value_name("FILE")
                .help("Path of file to read")
                .takes_value(true)
                .required(true)
        )
        .arg(
            Arg::with_name("blocksz")
                .help("Block Size")
                .required(false)
                .index(1)
                .takes_value(true)
                .default_value("1024")
                .value_name("BLOCKSZ")
        )
        .arg(
            Arg::with_name("dt-after")
                .help(
                    &format!("DateTime After filter - print syslog lines with a datetime that is at or after this datetime. Format {} (for example, '{}')", dt_filter_pattern, dt_example)
                )
                .required(false)
                //.index(2)
                .takes_value(true)
                .default_value("")
                .value_name("DT_AFTER")
        )
        .arg(
            Arg::with_name("dt-before")
                .help(
                    &format!("DateTime Before filter - print syslog lines with a datetime that is at or before this datetime. Format {} (for example, '{}')", dt_filter_pattern, dt_example)
                )
                .required(false)
                .index(3)
                .takes_value(true)
                .default_value("")
                .value_name("DT_BEFORE")
        )
        .get_matches();
    let fpath = String::from(matches.value_of("filepath").unwrap());
    let blockszs = String::from(matches.value_of("blocksz").unwrap());
    let filter_dt_after_s: &str = matches.value_of("dt-after").unwrap();
    let filter_dt_before_s: &str = matches.value_of("dt-before").unwrap();

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

    // parse datetime filters after
    let mut filter_dt_after: DateTimeL_Opt = None;
    if filter_dt_after_s != "" {
        filter_dt_after = match Local.datetime_from_str(filter_dt_after_s, &dt_filter_pattern) {
            Ok(val) => Some(val),
            Err(err) => {
                eprintln!("ERROR: failed to parse --dt-after {}", err);
                return Err(err);
            }
        };
    }
    //dbg!(filter_dt_after);
    // parse datetime filters before
    let mut filter_dt_before: DateTimeL_Opt = None;
    if filter_dt_before_s != "" {
        filter_dt_before = match Local.datetime_from_str(filter_dt_before_s, &dt_filter_pattern) {
            Ok(val) => Some(val),
            Err(err) => {
                eprintln!("ERROR: failed to parse --dt-before {}", err);
                return Err(err);
            }
        };
    }
    //dbg!(filter_dt_before);

    if filter_dt_after.is_some() && filter_dt_before.is_some() {
        let dta = filter_dt_after.unwrap();
        let dtb = filter_dt_before.unwrap();
        if dta > dtb {
            eprintln!("ERROR: Datetime --dt-after ({}) is after Datetime --dt-before ({})", dta, dtb);
            // TODO: return an Error
            return Ok(());
        }
    }

    // set `_STACKDEPTH_MAIN` once, use `stackdepth_main` to access `_STACKDEPTH_MAIN`
    if cfg!(debug_assertions) {
        stackdepth_main_set();
    }
    debug_eprintln!("{}main()", sn());

    //test_sn_so_sx();
    //test_stack_offset();
    //test_BlockReader_offsets();
    //test_BlockReader(&fpath, bsize);
    //test_find_datetime_in_line(bsize);
    //test_LineReader(&fpath, bsize);
    //test_LineReader_rand(&fpath, bsize);
    //test_sysline_pass_filters();
    //test_SyslineReader(&fpath, bsize);
    //test_SyslineReader_rand(&fpath, bsize);
    //test_SyslineReader_w_filtering_1(&fpath, bsize, filter_dt_after, filter_dt_before);
    test_SyslineReader_w_filtering_2(&fpath, bsize, filter_dt_after, filter_dt_before);

    debug_eprintln!("{}main()", sx());
    return Ok(());
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Blocks and BlockReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
/// Sequence of Bytes
type Bytes = Vec<u8>;
/// Reference Counting Pointer to a `Block`
type BlockP = Rc<Block>;

type Slices<'a> = Vec<&'a [u8]>;
// Consider this user library which claims to be faster than std::collections::BTreeMap
// https://docs.rs/cranelift-bforest/0.76.0/cranelift_bforest/
type Blocks = BTreeMap<BlockOffset, BlockP>;
type BlocksLRUCache = LruCache<BlockOffset, BlockP>;
// TODO: consider adding a LinkedList to link neighboring... hmm... though that might amount to too many LinkedLists
//       if searches are done randomly.
//       This need will come up again, I suspect... maybe wait on it.
/// for case where reading blocks, lines, or syslines reaches end of file, the value `WriteZero` will
/// be used here ot mean "_end of file reached, nothing new_"
/// XXX: this is a hack
#[allow(non_upper_case_globals)]
static EndOfFile: ErrorKind = ErrorKind::WriteZero;

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
    //       Can also `assert` that passed `FileOffset` is not larger than filesz, greater than zero.
    //       But keep the public static version available for testing.
    //       Change the LineReader calls to call `self.blockreader....`

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
        if self.filesz == 0 {
            return 0;
        }
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

    /// get byte at FileOffset
    /// `None` means the data at `FileOffset` was not available
    /// Does not request any `read_block`! Only copies from what is currently available.
    /// debug helper only
    fn _get_byte(&self, fo: FileOffset) -> Option<u8> {
        let bo = BlockReader::block_offset_at_file_offset(fo, self.blocksz);
        let bi = BlockReader::block_index_at_file_offset(fo, self.blocksz);
        if self.blocks.contains_key(&bo) {
            return Some((*self.blocks[&bo])[bi]);
        }
        return None;
    }

    /// return `Bytes` at `[fo_a, fo_b)`.
    /// uses `_get_byte` which does not request any reads!
    /// debug helper only
    fn _vec_from(&self, fo_a: FileOffset, fo_b: FileOffset) -> Bytes {
        assert_le!(fo_a, fo_b, "bad fo_a {} fo_b {}", fo_a, fo_b);
        assert_le!(fo_b, self.filesz, "bad fo_b {} but filesz {}", fo_b, self.filesz);
        if fo_a == fo_b {
            return Bytes::with_capacity(0);
        }
        let bo_a = BlockReader::block_offset_at_file_offset(fo_a, self.blocksz);
        let bo_b = BlockReader::block_offset_at_file_offset(fo_b, self.blocksz);
        let bo_a_i = BlockReader::block_index_at_file_offset(fo_a, self.blocksz);
        let bo_b_i = BlockReader::block_index_at_file_offset(fo_b, self.blocksz);
        if bo_a == bo_b {
            return Bytes::from(&(*self.blocks[&bo_a])[bo_a_i..bo_b_i]);
        }
        let mut fo_at = fo_a;
        let sz = (fo_b - fo_a) as usize;
        // XXX: inefficient!
        let mut vec_ = Bytes::with_capacity(sz);
        let mut at: usize = 0;
        while fo_at < fo_b {
            let b = match self._get_byte(fo_at) {
                Some(val) => val,
                None => {
                    break;
                }
            };
            vec_.push(b);
            fo_at += 1;
            at += 1;
        }
        return vec_;
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

#[allow(non_snake_case, dead_code)]
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
            // TODO: I'm somewhat sure this is not creating anything new but I should verify with `gdb-rust`.
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
        let mut s1 = String::with_capacity(sz);

        for linepart in &self.lineparts {
            if raw {
                // transform slices to `str`, can this be done more efficiently?
                // XXX: here is a good place to use `bstr`
                let s2 = &(&*linepart.blockp)[linepart.blocki_beg..linepart.blocki_end];
                let s3 = match str::from_utf8(s2) {
                    Ok(val) => val,
                    Err(err) => {
                        let fo1 = self.fileoffset_begin() + (linepart.blocki_beg as FileOffset);
                        let fo2 = self.fileoffset_begin() + (linepart.blocki_end as FileOffset);
                        eprintln!("ERROR: failed to convert [u8] at FileOffset[{}..{}] to utf8 str; {}", fo1, fo2, err);
                        continue;
                    }
                };
                s1.push_str(s3);
            } else {
                // copy u8 as char to `s1`
                let stop = linepart.len();
                let block_iter = (&*linepart.blockp).iter();
                for (bi, b) in block_iter.skip(linepart.blocki_beg).enumerate() {
                    if bi >= stop {
                        break;
                    }
                    let c = byte_to_nonraw_char(*b);
                    s1.push(c);
                }
            }
        }
        return s1;
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
/// Line Searching error
#[allow(non_camel_case_types)]
type ResultS4_LineFind = ResultS4<(FileOffset, LineP), Error>;
type LinesLRUCache = LruCache<FileOffset, ResultS4_LineFind>;

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
    // TODO: [2021/09/21] add efficiency stats
    // TODO: [2021/09/26] replace LruCache with a Rangemap
    /// internal LRU cache for `find_line`
    _find_line_lru_cache: LinesLRUCache,
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
            _find_line_lru_cache: LinesLRUCache::new(8),
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
    pub fn find_line(&mut self, fileoffset: FileOffset) -> ResultS4_LineFind {
        debug_eprintln!("{}find_line(LineReader@{:p}, {})", sn(), self, fileoffset);

        // some helpful constants
        let charsz_fo = self.charsz as FileOffset;
        let charsz_bi = self.charsz as BlockIndex;
        let filesz = self.filesz();
        let blockoffset_last = self.blockoffset_last();

        // check LRU cache
        match self._find_line_lru_cache.get(&fileoffset) {
            Some(rlp) => {
                // self.stats_read_block_cache_lru_hit += 1;
                debug_eprint!("{}find_line: found LRU cached for offset {}", sx(), fileoffset);
                match rlp {
                    ResultS4_LineFind::Ok(val) => {
                        debug_eprintln!(" return ResultS4_LineFind::Ok(({}, …))", val.0);
                        return ResultS4_LineFind::Ok((val.0, val.1.clone()));
                    }
                    ResultS4_LineFind::Ok_EOF(val) => {
                        debug_eprintln!(" return ResultS4_LineFind::Ok_EOF(({}, …))", val.0);
                        return ResultS4_LineFind::Ok_EOF((val.0, val.1.clone()));
                    }
                    ResultS4_LineFind::Ok_Done => {
                        debug_eprintln!(" return ResultS4_LineFind::Ok_Done");
                        return ResultS4_LineFind::Ok_Done;
                    }
                    _ => {
                        debug_eprintln!(" Err");
                        eprintln!("ERROR: unexpected value store in _find_line_lru_cache, fileoffset {}", fileoffset);
                    }
                }
            }
            None => {
                //self.stats_read_block_cache_lru_miss += 1;
                debug_eprintln!("{}find_line: fileoffset {} not found in LRU cache", so(), fileoffset);
            }
        }

        // handle special cases
        if filesz == 0 {
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Ok_Done; file is empty", sx());
            return ResultS4_LineFind::Ok_Done;
        } else if fileoffset > filesz {
            // TODO: need to decide on consistent behavior for passing fileoffset > filesz
            //       should it really Error or be Ok_Done?
            //       Make that consisetent among all LineReader and SyslineReader `find_*` functions
            let err = Error::new(
                ErrorKind::AddrNotAvailable,
                format!("Passed fileoffset {} past file size {}", fileoffset, filesz),
            );
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Err({}); fileoffset was too big!", sx(), err);
            return ResultS4_LineFind::Err(err);
        } else if fileoffset == filesz {
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Ok_Done(); fileoffset is at end of file!", sx());
            return ResultS4_LineFind::Ok_Done;
        }

        // TODO: add a RangeMap like SyslineReader has

        // first check if there is a line already known at this fileoffset
        if self.lines.contains_key(&fileoffset) {
            debug_eprintln!("{}find_line: hit cache for FileOffset {}", so(), fileoffset);
            let lp = self.lines[&fileoffset].clone();
            let fo_next = (*lp).fileoffset_end() + charsz_fo;
            // TODO: determine if `fileoffset` is the last line of the file
            //       should add a private helper function for this task `is_line_last(FileOffset)` ... something like that
            // TODO: add stats like BockReader
            self._find_line_lru_cache
                .put(fileoffset, ResultS4_LineFind::Ok((fo_next, lp.clone())));
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Ok({}, {:p})", sx(), fo_next, &*lp);
            return ResultS4_LineFind::Ok((fo_next, lp));
        }
        debug_eprintln!(
            "{}find_line: file offset {} not found in self.lines, searching for first newline found_nl_a …",
            so(),
            fileoffset
        );

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
                        debug_eprintln!("{}find_line: return ResultS4_LineFind::Ok_Done; EOF from read_block; NOT SURE IF THIS IS CORRECT", sx());
                        return ResultS4_LineFind::Ok_Done;
                    }
                    self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Ok_Done);
                    debug_eprintln!(
                        "{}find_line: return ResultS4_LineFind::Ok_Done; NOT SURE IF THIS IS CORRECT",
                        sx()
                    );
                    return ResultS4_LineFind::Ok_Done;
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
            debug_eprintln!(
                "{}find_line: return ResultS4_LineFind::Ok_Done; newline A is at last char in file {}, not a line IS THIS CORRECT?",
                sx(),
                filesz - 1
            );
            self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Ok_Done);
            return ResultS4_LineFind::Ok_Done;
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
                        debug_eprintln!(
                            "{}find_line: return ResultS4_LineFind::Ok_EOF(({}, {:p})); '{}'",
                            sx(),
                            fo_,
                            &*rl,
                            (*rl).to_String_noraw()
                        );
                        self._find_line_lru_cache
                            .put(fileoffset, ResultS4_LineFind::Ok_EOF((fo_, rl.clone())));
                        return ResultS4_LineFind::Ok_EOF((fo_, rl));
                    }
                    debug_eprintln!("{}find_line: return ResultS4_LineFind::Err({:?});", sx(), err);
                    return ResultS4_LineFind::Err(err);
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
                        so(),
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
                break;
            }
        } // ! found_nl_b

        /*
        This code was never called. Get rid of extra ResultS4 enum.

        // occurs in files with single newline
        if line.count() == 0 {
            let err = Err(Error::new(NoLinesFound, "No Lines Found!"));
            debug_eprintln!("{}find_line: return ({}, {}, {:?}) no LinePart found!", sx(), eof, fo_nl_b, err);
            return (false, fo_nl_b, err);
        }
        */

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
        debug_eprintln!(
            "{}find_line: return ResultS4_LineFind::Ok(({}, @{:p})); '{}'",
            sx(),
            fo_end + 1,
            &*rl,
            (*rl).to_String_noraw()
        );
        self._find_line_lru_cache
            .put(fileoffset, ResultS4_LineFind::Ok((fo_end + 1, rl.clone())));
        return ResultS4_LineFind::Ok((fo_end + 1, rl));
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
        debug_eprintln!("{}LineReader.find_line({})", so(), fo1);
        let result = lr1.find_line(fo1);
        match result {
            ResultS4_LineFind::Ok((fo, lp)) => {
                let _ln = lr1.lines.len();
                debug_eprintln!(
                    "{}ResultS4_LineFind::Ok!    FileOffset {} line num {} Line @{:p}: len {} '{}'",
                    so(),
                    fo,
                    _ln,
                    &*lp,
                    (*lp).len(),
                    (*lp).to_String_noraw()
                );
                fo1 = fo;
                if cfg!(debug_assertions) {
                    match print_colored(Color::Green, &(*lp).as_slice()) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("ERROR: print_colored returned error {}", err);
                        }
                    }
                } else {
                    (*lp).print(true);
                }
            }
            ResultS4_LineFind::Ok_EOF((fo, lp)) => {
                let _ln = lr1.lines.len();
                debug_eprintln!(
                    "{}ResultS4_LineFind::EOF!  FileOffset {} line num {} Line @{:p}: len {} '{}'",
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
            ResultS4_LineFind::Ok_Done => {
                debug_eprintln!("{}ResultS4_LineFind::Ok_Done!", so());
                break;
            }
            ResultS4_LineFind::Err(err) => {
                debug_eprintln!("{}ResultS4_LineFind::Err {}", so(), err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
    }
    //debug_eprintln!("\n{}{:?}", so(), lr1);

    if cfg!(debug_assertions) {
        debug_eprintln!("{}Found {} Lines", so(), lr1.lines.len())
    }
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
    debug_eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);

    for fo1 in offsets_rand {
        debug_eprintln!("{}LineReader.find_line({})", so(), fo1);
        let result = lr1.find_line(fo1);
        match result {
            ResultS4_LineFind::Ok((fo, lp)) => {
                let _ln = lr1.lines.len();
                debug_eprintln!(
                    "{}ResultS4_LineFind::Ok!    FileOffset {} line num {} Line @{:p}: len {} '{}'",
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
            ResultS4_LineFind::Ok_EOF((fo, lp)) => {
                let _ln = lr1.lines.len();
                debug_eprintln!(
                    "{}ResultS4_LineFind::EOF!  FileOffset {} line num {} Line @{:p}: len {} '{}'",
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
            ResultS4_LineFind::Ok_Done => {
                debug_eprintln!("{}ResultS4_LineFind::Ok_Done!", so());
                break;
            }
            ResultS4_LineFind::Err(err) => {
                debug_eprintln!("{}ResultS4_LineFind::Err {}", so(), err);
                eprintln!("ERROR: {}", err);
                break;
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
/// typical DateTime with TZ type
type DateTimeL = DateTime<Local>;
#[allow(non_camel_case_types)]
type DateTimeL_Opt = Option<DateTimeL>;
/// Sysline Searching error
#[allow(non_camel_case_types)]
type ResultS4_SyslineFind = ResultS4<(FileOffset, SyslineP), Error>;

/// A `Sysline` has information about a "syslog line" that spans one or more `Line`s
/// a "syslog line" is one or more lines, where the first line starts with a
/// datetime stamp. That datetime stamp is consistent format of other nearby syslines.
pub struct Sysline {
    /// the one or more `Line` that make up a Sysline
    /// TODO: rename this lines
    syslineparts: SyslineParts,
    /// index into `Line` where datetime string starts
    /// byte-based count
    /// datetime is presumed to be on first Line
    dt_beg: LineIndex,
    /// index into `Line` where datetime string ends, one char past last character of datetime string
    /// byte-based count
    /// datetime is presumed to be on first Line
    dt_end: LineIndex,
    /// parsed DateTime instance
    /// TODO: assumes `Local` TZ, how to create an "any TZ" chrono DateTime instance?
    dt: DateTimeL_Opt,
}

/// a signifier value for "not set" or "null" - because sometimes Option is a PitA
const LI_NULL: LineIndex = LineIndex::MAX;

impl fmt::Debug for Sysline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut li_s = String::new();
        for lp in self.syslineparts.iter() {
            li_s.push_str(&format!(
                "Line @{:p} (fileoffset_beg {}, fileoffset_end {}, len() {}, count() {}",
                &*lp,
                (*lp).fileoffset_begin(),
                (*lp).fileoffset_end(),
                (*lp).len(),
                (*lp).count()
            ));
        }
        f.debug_struct("Sysline")
            .field("fileoffset_begin()", &self.fileoffset_begin())
            .field("fileoffset_end()", &self.fileoffset_end())
            .field("syslineparts @", &format_args!("{:p}", &self.syslineparts))
            .field("syslineparts.len", &self.syslineparts.len())
            .field("dt_beg", &self.dt_beg)
            .field("dt_end", &self.dt_end)
            .field("dt", &self.dt)
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
            dt: None,
        };
    }

    pub fn new_from_line(linep: LineP) -> Sysline {
        let mut v = SyslineParts::with_capacity(Sysline::SYSLINE_PARTS_WITH_CAPACITY);
        v.push(linep);
        return Sysline {
            syslineparts: v,
            dt_beg: LI_NULL,
            dt_end: LI_NULL,
            dt: None,
        };
    }

    pub fn push(&mut self, linep: LineP) {
        if self.syslineparts.len() > 0 {
            // TODO: sanity check lines are in sequence
        }
        debug_eprintln!(
            "{}syslinereader.push(@{:p}), self.syslineparts.len() is now {}",
            so(),
            &*linep,
            self.syslineparts.len() + 1
        );
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

    /// the byte offset into the next sysline
    /// however, this Sysline does not know if it is at the end of a file
    pub fn fileoffset_next(self: &Sysline) -> FileOffset {
        self.fileoffset_end() + (self.charsz() as FileOffset)
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
        assert_lt!(
            self.dt_beg,
            slice_.len(),
            "dt_beg {} past end of slice[{}..{}]?",
            self.dt_beg,
            self.dt_beg,
            self.dt_end
        );
        assert_le!(
            self.dt_end,
            slice_.len(),
            "dt_end {} past end of slice[{}..{}]?",
            self.dt_end,
            self.dt_beg,
            self.dt_end
        );
        // TODO: here is a place to use `bstr`
        let buf: &[u8] = &slice_[self.dt_beg..self.dt_end];
        let s_ = match str::from_utf8(buf) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Error in datetime_String() during str::from_utf8 {} buffer {:?}", err, buf);
                ""
            }
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
        // XXX: intermixing byte lengths and character lengths
        // XXX: does not handle multi-byte
        let mut s_ = String::with_capacity(sz + 1);
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
        panic!("not implemented");
        return String::from("stub!");
    }

    #[allow(non_snake_case)]
    pub fn to_String_from_to(self: &Sysline, _from: usize, _to: usize) -> String {
        panic!("not implemented");
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
/// range map where key is sysline begin to end `[ Sysline.fileoffset_begin(), Sysline.fileoffset_end()]`
/// and where value is sysline begin (`Sysline.fileoffset_begin()`). Use the value to lookup associated `Syslines` map
type SyslinesRangeMap = RangeMap<FileOffset, FileOffset>;
/// return type for `SyslineReader::find_datetime_in_line`
#[allow(non_camel_case_types)]
type Result_FindDateTime = Result<(LineIndex, LineIndex, DateTimeL)>;
type SyslinesLRUCache = LruCache<FileOffset, ResultS4_SyslineFind>;

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Result_Filter_DateTime1 {
    Pass,
    OccursAtOrAfter,
    OccursBefore,
}

impl Result_Filter_DateTime1 {
    /// Returns `true` if the result is [`OccursAfter`].
    #[inline]
    pub const fn is_after(&self) -> bool {
        matches!(*self, Result_Filter_DateTime1::OccursAtOrAfter)
    }

    /// Returns `true` if the result is [`OccursBefore`].
    #[inline]
    pub const fn is_before(&self) -> bool {
        matches!(*self, Result_Filter_DateTime1::OccursBefore)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Result_Filter_DateTime2 {
    /// PASS
    OccursInRange,
    /// FAIL
    OccursBeforeRange,
    /// FAIL
    OccursAfterRange,
}

impl Result_Filter_DateTime2 {
    #[inline]
    pub const fn is_pass(&self) -> bool {
        matches!(*self, Result_Filter_DateTime2::OccursInRange)
    }

    #[inline]
    pub const fn is_fail(&self) -> bool {
        matches!(*self, Result_Filter_DateTime2::OccursAfterRange | Result_Filter_DateTime2::OccursBeforeRange)
    }
}

/// Specialized Reader that uses `LineReader` to find syslog lines
pub struct SyslineReader<'syslinereader> {
    linereader: LineReader<'syslinereader>,
    /// Syslines by fileoffset_begin
    syslines: Syslines,
    /// Syslines fileoffset by sysline fileoffset range, i.e. `[Sysline.fileoffset_begin(), Sysline.fileoffset_end()]`
    syslines_by_range: SyslinesRangeMap,
    // TODO: [2021/09/21] add efficiency stats
    // TODO: get rid of LRU cache
    /// internal LRU cache for `find_sysline`
    _find_sysline_lru_cache: SyslinesLRUCache,
}

// TODO: [2021/09/19]
//       put all filter data into one struct `SyslineFilter`, simpler to pass around

impl fmt::Debug for SyslineReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyslineReader")
            .field("linereader", &self.linereader)
            .field("syslines", &self.syslines)
            .finish()
    }
}

/// quick debug helper
#[allow(non_snake_case, dead_code)]
fn debug_eprint_LRU_cache<K, V>(cache: &LruCache<K, V>)
where
    K: std::fmt::Debug,
    K: std::hash::Hash,
    K: Eq,
    V: std::fmt::Debug,
{
    if !cfg!(debug_assertions) {
        return;
    }
    debug_eprint!("[");
    for (key, val) in cache.iter() {
        debug_eprint!(" Key: {:?}, Value: {:?};", key, val);
    }
    debug_eprint!("]");
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
        Ok(SyslineReader {
            linereader: lr,
            syslines: Syslines::new(),
            syslines_by_range: SyslinesRangeMap::new(),
            _find_sysline_lru_cache: SyslinesLRUCache::new(4),
        })
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

    pub fn charsz(&self) -> usize {
        SyslineReader::CHARSZ
    }

    pub fn print(&self, fileoffset: FileOffset, raw: bool) {
        let syslinep: &SyslineP = match self.syslines.get(&fileoffset) {
            Some(val) => val,
            None => {
                eprintln!("ERROR: in print, self.syslines.get({}) returned None", fileoffset);
                return;
            }
        };
        for linep in &(*syslinep).syslineparts {
            (*linep).print(raw);
        }
    }

    /// Testing helper only
    /// print all known `Sysline`s
    pub fn print_all(&self, raw: bool) {
        debug_eprintln!("{}print_all(true)", sn());
        for fo in self.syslines.keys() {
            self.print(*fo, raw);
        }
        debug_eprintln!("\n{}print_all(true)", sx());
    }

    /// is given `SyslineP` last in the file?
    fn is_sysline_last(&self, syslinep: &SyslineP) -> bool {
        let filesz = self.filesz();
        let fo_end = (*syslinep).fileoffset_end();
        if (fo_end + 1) == filesz {
            return true;
        }
        assert_lt!(fo_end + 1, filesz, "fileoffset_end() {} is at or after filesz() fileoffset {}", fo_end, filesz);
        return false;
    }

    /// store passed `Sysline` in `self.syslines`
    fn insert_line(&mut self, line: Sysline) -> SyslineP {
        let fo_beg: FileOffset = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let slp = SyslineP::new(line);
        debug_eprintln!("{}syslinereader.insert_line: syslines.insert({}, Sysline @{:p})", so(), fo_beg, &*slp);
        self.syslines.insert(fo_beg, slp.clone());
        debug_eprintln!(
            "{}syslinereader.insert_line: syslines_by_range.insert({}..{}, {})",
            so(),
            fo_beg,
            fo_end,
            fo_beg
        );
        // XXX: multi-byte character
        self.syslines_by_range.insert(fo_beg..fo_end, fo_beg);
        return slp;
    }

    /// if datetime found in `Line` returns `Ok` around
    /// indexes into `line` of found datetime string `(start of string, end of string)`
    /// else returns `Err`
    /// XXX: stub implementation; a few patterns, Local TZ assumptions
    pub fn find_datetime_in_line(line: &Line) -> Result_FindDateTime {
        debug_eprintln!("{}find_datetime_in_line(Line@{:p}) '{}'", sn(), &line, line.to_String_noraw());
        // no possible datetime
        if line.len() < 4 {
            debug_eprintln!("{}find_datetime_in_line returning Err(ErrorKind::InvalidInput)", sx());
            return Err(Error::new(ErrorKind::InvalidInput, "Line is too short"));
        }
        // TODO: create `pub fnas_slice_first_X` that return slice of first X bytes,
        //       most cases only need first 30 or so bytes of line, and this would be less likely to cross block boundaries
        let slice_ = line.as_slice();

        let mut i = 0;
        // end_i is one past last char
        // XXX: it might be faster to skip the special formatting and look directly for the datetime stamp.
        //      calls to chrono are long according to the flamegraph.
        //      however, using the demarcating characters ("[", "]") does give better assurance.
        // TODO: [2021/09/20] That is why another function is needed after the datetime format is determined.
        //       parsing the remainder of the file should use the already determined datetime format.
        //       And maybe for the common datetime formats, should use premade DateTime creation helper?
        //       (and skip use of chrono::datetime_from_str, it is incredibly slow!)
        for (pattern, beg_i, end_i, actual_beg_i, actual_end_i) in [
            //
            // from file `.\logs\Ubuntu18\samba\log.10.7.190.134`
            // example with offset:
            //
            //               1         2
            //     012345678901234567890123456789
            //     [2020/03/05 12:17:59.631000,  3] ../source3/smbd/oplock.c:1340(init_oplocks)
            //        init_oplocks: initializing messages.
            //
            ("[%Y/%m/%d %H:%M:%S%.6f,", 0, 28, 1, 27),
            //
            // from file `.\logs\Ubuntu18\xrdp.log`
            // example with offset:
            //
            //               1
            //     01234567890123456789
            //     [20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
            //
            ("[%Y%m%d-%H:%M:%S]", 0, 19, 1, 18),
            //
            // from file `logs/other/archives/proftpd/xferlog`
            // example with offset:
            //
            //               1         2
            //     0123456789012345678901234
            //     Sat Oct 03 11:26:12 2020 0 192.168.1.12 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c
            //
            ("%a %b %d %H:%M:%S %Y", 0, 24, 0, 24),
            //
            // example with offset:
            //
            //               1         2
            //     012345678901234567890
            //     2020-01-01 00:00:01xyz
            //
            ("%Y-%m-%d %H:%M:%S", 0, 19, 0, 19),
            //
            // example with offset:
            //
            //               1         2
            //     012345678901234567890
            //     2020-01-01T00:00:01xyz
            //
            ("%Y-%m-%dT%H:%M:%S", 0, 19, 0, 19),
            //
            // example with offset:
            //
            //               1
            //     012345678901234567
            //     20200101T000001xyz
            //
            ("%Y%m%dT%H%M%S", 0, 15, 0, 15),
            //
        ] {
            i += 1;
            let len_ = end_i - beg_i;
            debug_assert_lt!(beg_i, end_i, "Bad values beg_i end_i");
            debug_assert_ge!(actual_beg_i, beg_i, "Bad values actual_beg_i beg_i");
            debug_assert_le!(actual_end_i, end_i, "Bad values actual_end_i end_i");
            debug_eprintln!("{}find_datetime_in_line searching for pattern {} '{}'", so(), i, pattern);
            if slice_.len() < len_ {
                debug_eprintln!(
                    "{}find_datetime_in_line slice.len() {} is too short for pattern {} len {}",
                    so(),
                    i,
                    slice_.len(),
                    len_
                );
                continue;
            }
            // TODO: here is a place to use `bstr`
            let dts = match str::from_utf8(&slice_[beg_i..len_]) {
                Ok(val) => val,
                Err(_) => {
                    debug_eprintln!("{}ERROR: find_datetime_in_line from_utf8 failed during pattern {}", so(), i);
                    continue;
                }
            };
            debug_eprintln!(
                "{}find_datetime_in_line searching for pattern {} '{}' in slice '{}'",
                so(),
                i,
                pattern,
                str_to_nonraw_String(dts),
            );
            let dt = match Local.datetime_from_str(dts, pattern) {
                Ok(val) => {
                    debug_eprintln!(
                        "{}find_datetime_in_line matched pattern {} '{}' to String '{}' extrapolated datetime '{}'",
                        so(),
                        i,
                        pattern,
                        dts,
                        val
                    );
                    val
                }
                Err(_) => {
                    debug_eprintln!("{}find_datetime_in_line failed to match pattern '{}'", so(), i);
                    continue;
                }
            };
            debug_eprintln!("{}find_datetime_in_line returning Ok({}, {}, {})", sx(), beg_i, end_i, &dt);
            return Ok((actual_beg_i as LineIndex, actual_end_i as LineIndex, dt));
        }

        debug_eprintln!("{}find_datetime_in_line returning Err(ErrorKind::NotFound)", sx());
        return Err(Error::new(ErrorKind::NotFound, "No datetime found!"));
    }

    /// find first sysline at or after `fileoffset`
    /// return (fileoffset of start of _next_ sysline, new Sysline at or after `fileoffset`)
    pub fn find_sysline(&mut self, fileoffset: FileOffset) -> ResultS4_SyslineFind {
        debug_eprintln!("{}find_sysline(SyslineReader@{:p}, {})", sn(), self, fileoffset);

        // check LRU cache
        match self._find_sysline_lru_cache.get(&fileoffset) {
            Some(rlp) => {
                // self.stats_read_block_cache_lru_hit += 1;
                debug_eprint!("{}find_sysline: found LRU cached for fileoffset {}", sx(), fileoffset);
                match rlp {
                    ResultS4_SyslineFind::Ok(val) => {
                        debug_eprintln!(" return ResultS4_SyslineFind::Ok(({}, …))", val.0);
                        return ResultS4_SyslineFind::Ok((val.0, val.1.clone()));
                    }
                    ResultS4_SyslineFind::Ok_EOF(val) => {
                        debug_eprintln!(" return ResultS4_SyslineFind::Ok_EOF(({}, …))", val.0);
                        return ResultS4_SyslineFind::Ok_EOF((val.0, val.1.clone()));
                    }
                    ResultS4_SyslineFind::Ok_Done => {
                        debug_eprintln!(" return ResultS4_SyslineFind::Ok_Done");
                        return ResultS4_SyslineFind::Ok_Done;
                    }
                    _ => {
                        debug_eprintln!(" Err");
                        eprintln!("ERROR: unexpected value store in _find_line_lru_cache, fileoffset {}", fileoffset);
                    }
                }
            }
            None => {
                //self.stats_read_block_cache_lru_miss += 1;
                debug_eprintln!("{}find_sysline: fileoffset {} not found in LRU cache", so(), fileoffset);
            }
        }

        /*
        // check if there is a sysline already known at this fileoffset
        if self.syslines.contains_key(&fileoffset) {
            debug_eprintln!("{}find_sysline: hit syslines cache for FileOffset {}", so(), fileoffset);
            let slp = self.syslines[&fileoffset].clone();
            // XXX: multi-byte character encoding
            let fo_next = (*slp).fileoffset_end() + (SyslineReader::CHARSZ as FileOffset);
            // TODO: determine if `fileoffset` is the last sysline of the file
            //       should add a private helper function for this task `is_sysline_last(FileOffset)` ... something like that
            debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Ok(({}, @{:p})) '{}'",
                sx(),
                fo_next,
                &*slp,
                (*slp).to_String_noraw()
            );
            self._find_sysline_lru_cache.put(fileoffset, ResultS4_SyslineFind::Ok((fo_next, slp.clone())));
            return ResultS4_SyslineFind::Ok((fo_next, slp));
        }
        */
        // TODO: test that retrieving by cache always returns the same ResultS4 enum value as without a cache
        // check if the offset is already in a known range
        match self.syslines_by_range.get_key_value(&fileoffset) {
            Some(range_fo) => {
                let range = range_fo.0;
                debug_eprintln!(
                    "{}find_sysline: hit syslines_by_range cache for FileOffset {} (found in range {:?})",
                    so(),
                    fileoffset,
                    range
                );
                let fo = range_fo.1;
                let slp = self.syslines[fo].clone();
                // XXX: multi-byte character encoding
                let fo_next = (*slp).fileoffset_next();
                if self.is_sysline_last(&slp) {
                    debug_eprintln!(
                        "{}find_sysline: return ResultS4_SyslineFind::Ok_EOF(({}, @{:p})) '{}'",
                        sx(),
                        fo_next,
                        &*slp,
                        (*slp).to_String_noraw()
                    );
                    self._find_sysline_lru_cache
                        .put(fileoffset, ResultS4_SyslineFind::Ok_EOF((fo_next, slp.clone())));
                    return ResultS4_SyslineFind::Ok_EOF((fo_next, slp.clone()));
                }
                debug_eprintln!(
                    "{}find_sysline: return ResultS4_SyslineFind::Ok(({}, @{:p})) '{}'",
                    sx(),
                    fo_next,
                    &*slp,
                    (*slp).to_String_noraw()
                );
                self._find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Ok((fo_next, slp.clone())));
                return ResultS4_SyslineFind::Ok((fo_next, slp.clone()));
            }
            None => {}
        }
        debug_eprintln!(
            "{}find_sysline: fileoffset {} not found in self.syslines_by_range, searching for first sysline datetime A",
            so(),
            fileoffset
        );

        //
        // find line with datetime A
        //

        let mut fo_a: FileOffset = 0;
        let mut fo1: FileOffset = fileoffset;
        let mut sl = Sysline::new();
        loop {
            let result: ResultS4_LineFind = self.linereader.find_line(fo1);
            let eof = result.is_eof();
            let (fo2, lp) = match result {
                ResultS4_LineFind::Ok((fo_, lp_)) | ResultS4_LineFind::Ok_EOF((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: A FileOffset {} Line @{:p} len {} parts {} '{}'",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    (fo_, lp_)
                }
                ResultS4_LineFind::Ok_Done => {
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Ok_Done; A", sx());
                    self._find_sysline_lru_cache
                        .put(fileoffset, ResultS4_SyslineFind::Ok_Done);
                    return ResultS4_SyslineFind::Ok_Done;
                }
                ResultS4_LineFind::Err(err) => {
                    eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Err({}); A", sx(), err);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            let result = SyslineReader::find_datetime_in_line(&*lp);
            debug_eprintln!("{}find_sysline: A find_datetime_in_line returned {:?}", so(), result);
            match result {
                Err(_) => {}
                Ok((dt_beg, dt_end, dt)) => {
                    // a datetime was found! beginning of a sysline
                    fo_a = fo1;
                    sl.dt_beg = dt_beg;
                    sl.dt_end = dt_end;
                    sl.dt = Some(dt);
                    debug_eprintln!("{}find_sysline: A sl.push('{}')", so(), (*lp).to_String_noraw());
                    sl.push(lp);
                    fo1 = sl.fileoffset_end();
                    if eof {
                        debug_eprintln!(
                            "{}find_sysline: return ResultS4_SyslineFind::Ok_EOF({}, {:p}); A",
                            sx(),
                            fo1,
                            &sl
                        );
                        let slp = SyslineP::new(sl);
                        self._find_sysline_lru_cache
                            .put(fileoffset, ResultS4_SyslineFind::Ok_EOF((fo1, slp.clone())));
                        return ResultS4_SyslineFind::Ok_EOF((fo1, slp));
                    }
                    break;
                }
            }
            debug_eprintln!("{}find_sysline: A skip push Line '{}'", so(), (*lp).to_String_noraw());
            fo1 = fo2;
        }

        debug_eprintln!(
            "{}find_sysline: found line with datetime A at FileOffset {}, searching for datetime B starting at fileoffset {}",
            so(),
            fo_a,
            fo1
        );

        //
        // find line with datetime B
        //

        let mut fo_b: FileOffset = fo1;
        let mut eof = false;
        loop {
            let result = self.linereader.find_line(fo1);
            let (fo2, lp) = match result {
                ResultS4_LineFind::Ok((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: B FileOffset {} Line @{:p} len {} parts {} '{}'",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    //assert!(!eof, "ERROR: find_line returned EOF as true yet returned Ok()");
                    (fo_, lp_)
                }
                ResultS4_LineFind::Ok_EOF((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: B FileOffset {} Line @{:p} len {} parts {} '{}'",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    eof = true;
                    //assert!(!eof, "ERROR: find_line returned EOF as true yet returned Ok()");
                    (fo_, lp_)
                }
                ResultS4_LineFind::Ok_Done => {
                    //debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Ok_Done; B", sx());
                    debug_eprintln!("{}find_sysline: break; B", sx());
                    eof = true;
                    break;
                }
                ResultS4_LineFind::Err(err) => {
                    eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Err({}); B", sx(), err);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            //let (dt_beg, dt_end, op_dt) = SyslineReader::find_datetime_in_line(&*lp);
            let result = SyslineReader::find_datetime_in_line(&*lp);
            debug_eprintln!("{}find_sysline: B find_datetime_in_line returned {:?}", so(), result);
            match result {
                Err(_) => {
                    debug_eprintln!("{}find_sysline: B sl.push('{}')", so(), (*lp).to_String_noraw());
                    sl.push(lp);
                }
                Ok(_) => {
                    // a datetime was found! end of this sysline, beginning of a new sysline
                    debug_eprintln!("{}find_sysline: B skip push Line '{}'", so(), (*lp).to_String_noraw());
                    fo_b = fo1;
                    break;
                }
            }
            fo1 = fo2;
        }

        debug_eprintln!("{}find_sysline: found line with datetime B at FileOffset {}", so(), fo_b);

        debug_eprintln!("{}find_sysline: self.insert_line({:p})", so(), &sl);
        let slp = self.insert_line(sl);
        if eof {
            debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Ok_EOF(({}, Ok(@{:p})) '{}'",
                sx(),
                fo_b,
                &*slp,
                (*slp).to_String_noraw()
            );
            self._find_sysline_lru_cache
                .put(fileoffset, ResultS4_SyslineFind::Ok_EOF((fo_b, slp.clone())));
            return ResultS4_SyslineFind::Ok_EOF((fo_b, slp));
        } else {
            debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Ok(({}, Ok(@{:p})) '{}'",
                sx(),
                fo_b,
                &*slp,
                (*slp).to_String_noraw()
            );
            self._find_sysline_lru_cache
                .put(fileoffset, ResultS4_SyslineFind::Ok((fo_b, slp.clone())));
            return ResultS4_SyslineFind::Ok((fo_b, slp));
        }
    }

    /// find first sysline at or after `fileoffset` that is at or after `dt_filter`
    /// for example, given syslog file with datetimes:
    ///     20010101
    ///     20010102
    ///     20010103
    /// where the newline ending the first line is the ninth byte (fileoffset 9)
    /// calling
    ///     syslinereader.find_sysline_at_datetime_filter(0, Some(20010102 00:00:00-0000))
    /// will return
    ///     ResultS4::Ok(19, SyslineP(data='20010102␊'))
    /// TODO: complete the examples
    ///
    pub fn find_sysline_at_datetime_filter(
        &mut self,
        fileoffset: FileOffset,
        dt_filter: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        let _fname = "find_sysline_at_datetime_filter";
        debug_eprintln!("{}{}(SyslingReader@{:p}, {}, {:?})", sn(), _fname, self, fileoffset, dt_filter,);
        let filesz = self.filesz();
        let fo_end: FileOffset = filesz as FileOffset;
        let mut try_fo: FileOffset = fileoffset;
        let mut try_fo_last: FileOffset = try_fo;
        let mut fo_last: FileOffset = fileoffset;
        let mut slp_opt: Option<SyslineP> = None;
        let mut fo_a: FileOffset = fileoffset; // begin "range cursor" marker
        let mut fo_b: FileOffset = fo_end; // end "range cursor" marker
        // LAST WORKING HERE [2021/09/26]
        //      BUG does not handle multiple sequential syslines with same datetime
        //      sometimes it is correct, sometimes not, depends on the particular file.
        //      test with file basic-basic-dt30-repeats.log
        loop {
            // TODO: [2021/09/26]
            //       this could be faster.
            //       currently it narrowing down to byte offset
            //       but it only needs to narrow down to range of a sysline
            //       so if `fo_a` and `fo_b` are in same sysline range, then this can return that sysline.
            //       Also, add stats for this function and debug print those stats before exiting.
            //       i.e. count of loops, count of calls to sysline_dt_before_after, etc.
            //       do this before tweaking function so can be compared
            debug_eprintln!("{}{}: loop(…)!", so(), _fname);
            let result = self.find_sysline(try_fo);
            let eof = result.is_eof();
            match result {
                ResultS4_SyslineFind::Ok((fo, slp)) | ResultS4_SyslineFind::Ok_EOF((fo, slp)) => {
                    if !eof {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline({}) returned ResultS4_SyslineFind::Ok({}, …)",
                            so(),
                            _fname,
                            try_fo,
                            fo
                        );
                    } else {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline({}) returned ResultS4_SyslineFind::Ok_EOF({}, …)",
                            so(),
                            _fname,
                            try_fo,
                            fo
                        );
                    }
                    debug_eprintln!(
                        "{}{}: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} '{}'",
                        so(),
                        _fname,
                        fo,
                        &(*slp),
                        slp.syslineparts.len(),
                        (*slp).len(),
                        (*slp).to_String_noraw(),
                    );
                    // here is the binary search algorithm in action
                    debug_eprintln!(
                        "{}{}: sysline_dt_after_or_before(@{:p} ({:?}), {:?})",
                        so(),
                        _fname,
                        &*slp,
                        (*slp).dt,
                        dt_filter
                    );
                    match SyslineReader::sysline_dt_after_or_before(&slp, dt_filter) {
                        Result_Filter_DateTime1::Pass => {
                            debug_eprintln!(
                                "{}{}: Pass => fo {} fo_last {} try_fo {} try_fo_last {} (fo_end {})",
                                so(),
                                _fname,
                                fo,
                                fo_last,
                                try_fo,
                                try_fo_last,
                                fo_end
                            );
                            debug_eprintln!(
                                "{}{}: return ResultS4_SyslineFind::Ok(({}, @{:p})); A",
                                sx(),
                                _fname,
                                fo,
                                &*slp
                            );
                            return ResultS4_SyslineFind::Ok((fo, slp));
                        }
                        Result_Filter_DateTime1::OccursAtOrAfter => {
                            // the Sysline found by `find_sysline(try_fo)` occurs at or after filter `dt_filter`, so search backward
                            // i.e. move end marker `fo_b` backward
                            debug_eprintln!("{}{}: OccursAtOrAfter => fo {} fo_last {} try_fo {} try_fo_last {} fo_b {} fo_a {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_b, fo_a, fo_end);
                            // short-circuit a common case, passed fileoffset is past the `dt_filter`, can immediately return
                            // XXX: does this mean my algorithm sucks?
                            if try_fo == fileoffset {
                                // first loop iteration
                                debug_eprintln!(
                                    "{}{}:                    try_fo {} == {} try_fo_last; early return",
                                    so(),
                                    _fname,
                                    try_fo,
                                    try_fo_last
                                );
                                debug_eprintln!(
                                    "{}{}: return ResultS4_SyslineFind::Ok(({}, @{:p})); B fileoffset {} '{}'",
                                    sx(),
                                    _fname,
                                    fo,
                                    &*slp,
                                    (*slp).fileoffset_begin(),
                                    (*slp).to_String_noraw()
                                );
                                return ResultS4_SyslineFind::Ok((fo, slp));
                            }
                            fo_b = (*slp).fileoffset_begin();
                            try_fo_last = try_fo;
                            assert_le!(fo_a, fo_b, "Unexpected values for fo_a {} fo_b {}", fo_a, fo_b);
                            debug_eprintln!(
                                "{}{}:                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                so(),
                                _fname,
                                fo_a,
                                fo_b,
                                fo_a
                            );
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        }
                        Result_Filter_DateTime1::OccursBefore => {
                            // the Sysline found by `find_sysline(try_fo)` occurs before filter `dt_filter`, so search forthward
                            // i.e. move begin marker `fo_a` forthward
                            debug_eprintln!("{}{}: OccursBefore =>    fo {} fo_last {} try_fo {} try_fo_last {} fo_b {} fo_a {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_b, fo_a, fo_end);
                            let slp_foe = (*slp).fileoffset_end();
                            assert_le!(slp_foe, fo, "unexpected values (*SyslineP).fileoffset_end() {}, fileoffset returned by find_sysline {}", slp_foe, fo);
                            fo_a = slp_foe;
                            try_fo_last = try_fo;
                            assert_le!(fo_a, fo_b, "Unexpected values for fo_a {} fo_b {}", fo_a, fo_b);
                            debug_eprintln!(
                                "{}{}:                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                so(),
                                _fname,
                                fo_a,
                                fo_b,
                                fo_a
                            );
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        }
                    }
                    debug_eprintln!("{}{}:                    try_fo {} try_fo_last {} fo_b {} fo_a {} fo {} fo_last {} (fo_end {})", so(), _fname, try_fo, try_fo_last, fo_b, fo_a, fo, fo_last, fo_end);
                    fo_last = fo;
                    slp_opt = Some(slp);
                    // TODO: [2021/09/26]
                    //       I think could do an early check and skip a few loops:
                    //       if `fo_a` and `fo_b` are offsets into the same Sysline
                    //       then that Sysline is the candidate, so return Ok(...)
                    //       unless `fo_a` and `fo_b` are past last Sysline.fileoffset_begin of the file then return Ok_Done
                    //       However, before implemetning that, implement the stats tracking of this function mentioned above,
                    //       be sure some improvement really occurs.
                }
                ResultS4_SyslineFind::Ok_Done => {
                    debug_eprintln!("{}{}: SyslineReader.find_sysline({}) returned Ok_Done", so(), _fname, try_fo);
                    debug_eprintln!(
                        "{}{}:                 try_fo {} try_fo_last {} fo_b {} fo_a {} (fo_end {})",
                        so(),
                        _fname,
                        try_fo,
                        try_fo_last,
                        fo_b,
                        fo_a,
                        fo_end
                    );
                    try_fo_last = try_fo;
                    debug_eprintln!(
                        "{}{}:                 ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                        so(),
                        _fname,
                        fo_a,
                        fo_b,
                        fo_a
                    );
                    try_fo = fo_a + ((fo_b - fo_a) / 2);
                    debug_eprintln!(
                        "{}{}:                 try_fo {} try_fo_last {} fo_b {} fo_a {} (fo_end {})",
                        so(),
                        _fname,
                        try_fo,
                        try_fo_last,
                        fo_b,
                        fo_a,
                        fo_end
                    );
                }
                ResultS4_SyslineFind::Err(err) => {
                    debug_eprintln!("{}{}: SyslineReader.find_sysline({}) returned Err({})", so(), _fname, try_fo, err);
                    eprintln!("ERROR: {}", err);
                    break;
                }
            } // match result
            debug_eprintln!("{}{}: next loop will try offset {} (fo_end {})", so(), _fname, try_fo, fo_end);
            if slp_opt.is_some() && try_fo == try_fo_last {
                // if the new offset calculation `try_fo` has not changed since last loop (`try_fo_last`) then
                // searching is exhausted
                debug_eprintln!("{}{}: try_fo {} == {} try_fo_last;", so(), _fname, try_fo, try_fo_last);
                let slp = slp_opt.unwrap();
                if slp.fileoffset_begin() < try_fo {
                    // binary search stopped at fileoffset past start of last Sysline
                    // so entirely past all acceptable syslines
                    debug_eprintln!(
                        "{}{}: return ResultS4_SyslineFind::Ok_Done; C",
                        sx(),
                        _fname,
                    );
                    return ResultS4_SyslineFind::Ok_Done;
                }
                // binary search stopped at fileoffset that refers to an acceptable sysline
                debug_eprintln!(
                    "{}{}: return ResultS4_SyslineFind::Ok(({}, @{:p})); D fileoffset {} '{}'",
                    sx(),
                    _fname,
                    fo_last,
                    &*slp,
                    (*slp).fileoffset_begin(),
                    (*slp).to_String_noraw()
                );
                return ResultS4_SyslineFind::Ok((fo_last, slp));
            } else if slp_opt.is_none() && try_fo == try_fo_last {
                debug_eprintln!(
                    "{}{}: try_fo {} == {} try_fo_last; SyslineP is None, break!",
                    so(),
                    _fname,
                    try_fo,
                    try_fo_last
                );
                break;
            }
        } // loop

        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Ok_Done; E", sx(), _fname);
        return ResultS4_SyslineFind::Ok_Done;
    }

    /// if `syslinep.dt` is at or after `dt_filter` then return `OccursAtOrAfter`
    /// if `syslinep.dt` is before `dt_filter` then return `OccursBefore`
    /// else return `Pass` (including if `dt_filter` is `None`)
    /// TODO: create more testable implementation that takes only DateTimeL_Opt. Say `fn dt_after_or_before`
    ///       Let `sysline_dt_after_or_before` check the SyslineP then call `dt_after_or_before`.
    pub fn sysline_dt_after_or_before(syslinep: &SyslineP, dt_filter: &DateTimeL_Opt) -> Result_Filter_DateTime1 {
        debug_eprintln!("{}sysline_dt_after_or_before(SyslineP@{:p}, {:?})", sn(), &*syslinep, dt_filter,);
        assert!((*syslinep).dt.is_some(), "Sysline @{:p} does not have a datetime set.", &*syslinep);

        if dt_filter.is_none() {
            debug_eprintln!(
                "{}sysline_dt_after_or_before(…) return Result_Filter_DateTime1::Pass; (no dt filters)",
                sx(),
            );
            return Result_Filter_DateTime1::Pass;
        }

        let dt = (*syslinep).dt.unwrap();
        let dt_a = dt_filter.unwrap();
        debug_eprintln!(
            "{}sysline_dt_after_or_before comparing Sysline datetime {:?} to filter datetime {:?}",
            so(),
            dt,
            dt_a
        );
        //if dt_a == dt {
        //    debug_eprintln!("{}sysline_dt_after_or_before(…) return Result_Filter_DateTime1::OccursInRange; ==", sx(),);
        //    return Result_Filter_DateTime1::OccursInRange;
        //}
        if dt < dt_a {
            debug_eprintln!("{}sysline_dt_after_or_before(…) return Result_Filter_DateTime1::OccursBefore; (Sysline datetime {:?} is before filter {:?})", sx(), dt, dt_a);
            return Result_Filter_DateTime1::OccursBefore;
        }
        debug_eprintln!("{}sysline_dt_after_or_before(…) return Result_Filter_DateTime1::OccursAtOrAfter; (Sysline datetime {:?} is at or after filter {:?})", sx(), dt, dt_a);
        return Result_Filter_DateTime1::OccursAtOrAfter;
    }

    /// If both filters are `Some` and `syslinep.dt` is "between" the filters then return `Pass`
    /// comparison is "inclusive" i.e. `dt` == `dt_filter_after` will return `Pass`
    /// TODO: finish this
    /// If both filters are `None` then return `Pass`
    pub fn dt_pass_filters(
        dt: &DateTimeL,
        dt_filter_after: &DateTimeL_Opt,
        dt_filter_before: &DateTimeL_Opt,
    ) -> Result_Filter_DateTime2 {
        debug_eprintln!(
            "{}dt_pass_filters({:?}, {:?}, {:?})",
            sn(),
            dt,
            dt_filter_after,
            dt_filter_before,
        );
        if dt_filter_after.is_none() && dt_filter_before.is_none() {
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange; (no dt filters)", sx(),);
            return Result_Filter_DateTime2::OccursInRange;
        }
        if dt_filter_after.is_some() && dt_filter_before.is_some() {
            debug_eprintln!("{}dt_pass_filters comparing datetime dt_filter_after {:?} < {:?} dt < {:?} dt_fiter_before ???", so(), &dt_filter_after.unwrap(), dt, &dt_filter_before.unwrap());
            let da = &dt_filter_after.unwrap();
            let db = &dt_filter_before.unwrap();
            assert_le!(da, db, "Bad datetime range values filter_after {:?} {:?} filter_before", da, db);
            if dt < da {
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursBeforeRange;", sx());
                return Result_Filter_DateTime2::OccursBeforeRange;
            }
            if db < dt {
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursAfterRange;", sx());
                return Result_Filter_DateTime2::OccursAfterRange;
            }
            // assert da < dt && dt < db
            assert_le!(da, dt, "Unexpected range values da dt");
            assert_le!(dt, db, "Unexpected range values dt db");
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange;", sx());
            return Result_Filter_DateTime2::OccursInRange;
        } else if dt_filter_after.is_some() {
            debug_eprintln!("{}dt_pass_filters comparing datetime dt_filter_after {:?} < {:?} dt ???", so(), &dt_filter_after.unwrap(), dt);
            let da = &dt_filter_after.unwrap();
            if dt < da {
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursBeforeRange;", sx());
                return Result_Filter_DateTime2::OccursBeforeRange;
            }
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange;", sx());
            return Result_Filter_DateTime2::OccursInRange;
        } else {
            debug_eprintln!("{}dt_pass_filters comparing datetime dt {:?} < {:?} dt_filter_before ???", so(), dt, &dt_filter_before.unwrap());
            let db = &dt_filter_before.unwrap();
            if db < dt {
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursAfterRange;", sx());
                return Result_Filter_DateTime2::OccursAfterRange;
            }
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange;", sx());
            return Result_Filter_DateTime2::OccursInRange;
        }
    }

    /// wrapper for call to `dt_pass_filters`
    pub fn sysline_pass_filters(
        syslinep: &SyslineP,
        dt_filter_after: &DateTimeL_Opt,
        dt_filter_before: &DateTimeL_Opt,
    ) -> Result_Filter_DateTime2 {
        debug_eprintln!(
            "{}sysline_pass_filters(SyslineP@{:p}, {:?}, {:?})",
            sn(),
            &*syslinep,
            dt_filter_after,
            dt_filter_before,
        );
        assert!((*syslinep).dt.is_some(), "Sysline @{:p} does not have a datetime set.", &*syslinep);
        let dt = (*syslinep).dt.unwrap();
        let result = SyslineReader::dt_pass_filters(&dt, dt_filter_after, dt_filter_before);
        debug_eprintln!("{}sysline_pass_filters(…) return {:?};", sx(), result);
        return result;
    }
}

/// testing helper to write a `str` to a temporary file
/// `NamedTempFile` will be automatically deleted when `NamedTempFile` is dropped.
fn create_temp_file(content: &str) -> NamedTempFile {
    let mut ntf1 = match NamedTempFile::new() {
        Ok(val) => val,
        Err(err) => {
            //eprintln!("NamedTempFile::new() return Err {}", err);
            panic!("NamedTempFile::new() return Err {}", err);
        }
    };
    match ntf1.write_all(content.as_bytes()) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("NamedTempFile::write_all() return Err {}", err);
        }
    }

    return ntf1;
}

/// basic test of `SyslineReader.find_datetime_in_line`
#[allow(non_snake_case, dead_code)]
fn test_find_datetime_in_line(blocksz: BlockSz) {
    debug_eprintln!("{}test_find_datetime_in_line()", sn());

    let ntf1 = create_temp_file(
        "\
[20200113-11:03:06] [DEBUG] Testing if xrdp can listen on 0.0.0.0 port 3389.
[20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
CLOSED!
[20200113-11:03:08] [INFO ] starting xrdp with pid 23198
[20200113-11:03:08] [INFO ] listening to port 3389 on 0.0.0.0
[20200113-11:13:59] [INFO ] Socket 12: AF_INET6 connection received from ::ffff:127.0.0.1 port 55426
[20200113-11:13:59] [DEBUG] Closed socket 12 (AF_INET6 ::ffff:127.0.0.1 port 3389)
[20200113-11:13:59] [DEBUG] Closed socket 11 (AF_INET6 :: port 3389)
[20200113-11:13:59] [INFO ] Using default X.509 certificate: /etc/xrdp/cert.pem
[20200113-11:13:59] [INFO ] Using default X.509 key file: /etc/xrdp/key.pem
[20200113-11:13:59] [ERROR] Cannot read private key file /etc/xrdp/key.pem: Permission denied
[20200113-11:13:59] [ERROR] Certification error:
    UNABLE TO READ CERTIFICATE!
[20200113-11:13:59] [ERROR] Certification failed.
",
    );
    let path = String::from(ntf1.path().to_str().unwrap());

    let mut slr = match SyslineReader::new(&path, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new('{}', {}) failed {}", &path, blocksz, err);
            return;
        }
    };

    let mut fo1: FileOffset = 0;
    loop {
        let result = slr.find_sysline(fo1);
        let done = result.is_done() || result.is_eof();
        match result {
            ResultS4_SyslineFind::Ok((fo, slp)) | ResultS4_SyslineFind::Ok_EOF((fo, slp)) => {
                debug_eprintln!("{}slr.find_sysline({}) returned Ok({}, @{:p})", so(), fo1, fo, &*slp);
                debug_eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} '{}'",
                    so(),
                    fo,
                    &(*slp),
                    slp.syslineparts.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print_slp(&slp);
                fo1 = fo;
            }
            ResultS4_SyslineFind::Ok_Done => {
                debug_eprintln!("{}slr.find_sysline({}) returned Ok_Done", so(), fo1);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!("{}slr.find_sysline({}) returned Err({})", so(), fo1, err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
        if done {
            break;
        }
    }

    debug_eprintln!("{}test_find_datetime_in_line()", sx());
}

/// basic test of `SyslineReader.sysline_pass_filters`
#[allow(non_snake_case, dead_code)]
fn test_sysline_pass_filters() {
    debug_eprintln!("{}test_sysline_pass_filters()", sn());

    fn DTL(s: &str) -> DateTimeL {
        return Local.datetime_from_str(s, &"%Y%m%dT%H%M%S").unwrap();
    }

    for (da, dt, db, exp_result) in [
        (
            Some(DTL(&"20000101T010105")),
            DTL(&"20000101T010106"),
            Some(DTL(&"20000101T010107")),
            Result_Filter_DateTime2::OccursInRange,
        ),
        (
            Some(DTL(&"20000101T010107")),
            DTL(&"20000101T010106"),
            Some(DTL(&"20000101T010108")),
            Result_Filter_DateTime2::OccursBeforeRange,
        ),
        (
            Some(DTL(&"20000101T010101")),
            DTL(&"20000101T010106"),
            Some(DTL(&"20000101T010102")),
            Result_Filter_DateTime2::OccursAfterRange,
        ),
        (
            Some(DTL(&"20000101T010101")),
            DTL(&"20000101T010106"),
            None,
            Result_Filter_DateTime2::OccursInRange,
        ),
        (
            Some(DTL(&"20000101T010102")),
            DTL(&"20000101T010101"),
            None,
            Result_Filter_DateTime2::OccursBeforeRange,
        ),
        (
            Some(DTL(&"20000101T010101")),
            DTL(&"20000101T010101"),
            None,
            Result_Filter_DateTime2::OccursInRange,
        ),
        (
            None,
            DTL(&"20000101T010101"),
            Some(DTL(&"20000101T010106")),
            Result_Filter_DateTime2::OccursInRange,
        ),
        (
            None,
            DTL(&"20000101T010101"),
            Some(DTL(&"20000101T010100")),
            Result_Filter_DateTime2::OccursAfterRange,
        ),
        (
            None,
            DTL(&"20000101T010101"),
            Some(DTL(&"20000101T010101")),
            Result_Filter_DateTime2::OccursInRange,
        ),
    ] {
        let result = SyslineReader::dt_pass_filters(
            &dt,
            &da,
            &db,
        );
        assert_eq!(exp_result, result, "Expected {:?} Got {:?} for ({:?}, {:?}, {:?})", exp_result, result, dt, da, db);
        print_colored(Color::Green,
                      format!("{}({:?}, {:?}, {:?}) returned expected {:?}\n", so(), dt, da, db, result).as_bytes());
    }
    debug_eprintln!("{}test_sysline_pass_filters()", sx());
}

/// testing helper
/// if debug then print with color
/// else print efficiently
fn print_slp(slp: &SyslineP) {
    if cfg!(debug_assertions) {
        let out = (*slp).to_String_noraw();
        // XXX: presumes single-byte character encoding, does not handle multi-byte
        let a = &out[0..(*slp).dt_beg];
        match print_colored(Color::Green, &a.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: print_colored a returned error {}", err);
            }
        };
        let b = &out[(*slp).dt_beg..(*slp).dt_end];
        match print_colored(Color::Yellow, &b.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: print_colored b returned error {}", err);
            }
        };
        let c = &out[(*slp).dt_end..out.len()];
        match print_colored(Color::Green, &c.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: print_colored c returned error {}", err);
            }
        };
        println!();
    } else {
        //(*slp_).print(true);
        let slices = (*slp).get_slices();
        for slice in slices.iter() {
            write(slice);
        }
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

    let mut fo1: FileOffset = 0;
    loop {
        let result = slr.find_sysline(fo1);
        let done = result.is_done() || result.is_eof();
        match result {
            ResultS4_SyslineFind::Ok((fo, slp)) => {
                debug_eprintln!("{}slr.find_sysline({}) returned Ok({}, @{:p})", so(), fo1, fo, &*slp);
                debug_eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} '{}'",
                    so(),
                    fo,
                    &(*slp),
                    slp.syslineparts.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print_slp(&slp);
                assert!(!slr.is_sysline_last(&slp), "returned Ok yet this Sysline is last!? Should have returned Ok_EOF or this Sysline is really not last.");
                fo1 = fo;
            }
            ResultS4_SyslineFind::Ok_EOF((fo, slp)) => {
                debug_eprintln!("{}slr.find_sysline({}) returned Ok_EOF({}, @{:p})", so(), fo1, fo, &*slp);
                debug_eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} '{}'",
                    so(),
                    fo,
                    &(*slp),
                    slp.syslineparts.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print_slp(&slp);
                assert!(slr.is_sysline_last(&slp), "returned Ok_EOF yet this Sysline is not last!?");
                fo1 = fo;
            }
            ResultS4_SyslineFind::Ok_Done => {
                debug_eprintln!("{}slr.find_sysline({}) returned Ok_Done", so(), fo1);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!("{}slr.find_sysline({}) returned Err({})", so(), fo1, err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
        if done {
            break;
        }
    }

    debug_eprintln!("{}Found {} Lines, {} Syslines", so(), slr.linereader.lines.len(), slr.syslines.len());
    //print_colored(Color::Green, "DOES ANYTHING COLORED PRINT?\n".as_bytes())?;
    debug_eprintln!("{}test_SyslineReader({:?}, {})", sx(), &path, blocksz);
}

/// basic test of SyslineReader things
#[allow(non_snake_case, dead_code)]
fn test_SyslineReader_w_filtering_1(
    path: &String,
    blocksz: BlockSz,
    filter_dt_after_opt: DateTimeL_Opt,
    filter_dt_before_opt: DateTimeL_Opt,
) {
    debug_eprintln!(
        "{}test_SyslineReader_w_filtering_1({:?}, {}, {:?}, {:?})",
        sn(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );

    if cfg!(debug_assertions) {
        let s1 = file_to_nonraw_String(path);
        print_colored(Color::Yellow, s1.as_bytes());
        println!();
    }

    let mut slr = match SyslineReader::new(path, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({}, {}) failed {}", path, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}{:?}", so(), slr);

    let mut fo1: FileOffset = 0;
    let filesz = slr.filesz();
    while fo1 < filesz {
        debug_eprintln!("{}slr.find_sysline_at_datetime_filter({}, {:?})", so(), fo1, filter_dt_after_opt);
        let result = slr.find_sysline_at_datetime_filter(fo1, &filter_dt_after_opt);
        match result {
            ResultS4_SyslineFind::Ok((fo, slp)) | ResultS4_SyslineFind::Ok_EOF((fo, slp)) => {
                debug_eprintln!(
                    "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Ok({}, @{:p})",
                    so(),
                    fo1,
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                    fo,
                    &*slp
                );
                debug_eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} '{}'",
                    so(),
                    fo,
                    &(*slp),
                    slp.syslineparts.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print!("FileOffset {:3} {:?} '", fo1, filter_dt_after_opt);
                let snippet = slr
                    .linereader
                    .blockreader
                    ._vec_from(fo1, std::cmp::min(fo1 + 40, filesz));
                print_colored(Color::Yellow, buffer_to_nonraw_String(snippet.as_slice()).as_bytes());
                print!("' ");
                //print_slp(&slp);
                let slices = (*slp).get_slices();
                for slice in slices.iter() {
                    print_colored(Color::Green, slice);
                }
                println!();
            }
            ResultS4_SyslineFind::Ok_Done => {
                debug_eprintln!(
                    "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Ok_Done",
                    so(),
                    fo1,
                    filter_dt_after_opt,
                    filter_dt_before_opt
                );
            }
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!(
                    "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Err({})",
                    so(),
                    fo1,
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                    err
                );
                eprintln!("ERROR: {}", err);
            }
        }
        fo1 += 1;
        debug_eprintln!("\n");
    }

    debug_eprintln!("{}Found {} Lines, {} Syslines", so(), slr.linereader.lines.len(), slr.syslines.len());
    debug_eprintln!(
        "{}test_SyslineReader_w_filtering_1({:?}, {}, {:?}, {:?})",
        sx(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
}

/// basic test of SyslineReader things
#[allow(non_snake_case, dead_code)]
fn test_SyslineReader_w_filtering_2(
    path: &String,
    blocksz: BlockSz,
    filter_dt_after_opt: DateTimeL_Opt,
    filter_dt_before_opt: DateTimeL_Opt,
) {
    debug_eprintln!(
        "{}test_SyslineReader_w_filtering_2({:?}, {}, {:?}, {:?})",
        sn(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );

    let mut slr = match SyslineReader::new(path, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({}, {}) failed {}", path, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}{:?}", so(), slr);

    let mut fo1: FileOffset = 0;
    let mut search_more = true;
    debug_eprintln!("{}slr.find_sysline_at_datetime_filter({}, {:?})", so(), fo1, filter_dt_after_opt);
    let result = slr.find_sysline_at_datetime_filter(fo1, &filter_dt_after_opt);
    match result {
        ResultS4_SyslineFind::Ok((fo, slp)) | ResultS4_SyslineFind::Ok_EOF((fo, slp)) => {
            debug_eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Ok({}, @{:p})",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt,
                fo,
                &*slp
            );
            debug_eprintln!(
                "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} '{}'",
                so(),
                fo,
                &(*slp),
                slp.syslineparts.len(),
                (*slp).len(),
                (*slp).to_String_noraw(),
            );
            fo1 = fo;
            print_slp(&slp);
            //let slices = (*slp).get_slices();
            //for slice in slices.iter() {
            //    print_colored(Color::Green, slice);
            //}
            //println!();
        }
        ResultS4_SyslineFind::Ok_Done => {
            debug_eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Ok_Done",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt
            );
            search_more = false;
        }
        ResultS4_SyslineFind::Err(err) => {
            debug_eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Err({})",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt,
                err
            );
            eprintln!("ERROR: {}", err);
            search_more = false;
        }
    }
    if !search_more {
        debug_eprintln!("{}! search_more", so());
        debug_eprintln!("{}test_SyslineReader_w_filtering_2(…)", sx());
        return;
    }
    let mut fo2: FileOffset = fo1;
    loop {
        let result = slr.find_sysline(fo2);
        let eof = result.is_eof();
        match result {
            ResultS4_SyslineFind::Ok((fo, slp)) | ResultS4_SyslineFind::Ok_EOF((fo, slp)) => {
                if eof {
                    debug_eprintln!("{}slr.find_sysline({}) returned Ok_EOF({}, @{:p})", so(), fo2, fo, &*slp);
                } else {
                    debug_eprintln!("{}slr.find_sysline({}) returned Ok({}, @{:p})", so(), fo2, fo, &*slp);
                }
                fo2 = fo;
                debug_eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} '{}'",
                    so(),
                    fo,
                    &(*slp),
                    slp.syslineparts.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                debug_eprintln!(
                    "{}sysline_pass_filters({:?}, {:?}, {:?})",
                    so(),
                    (*slp).dt,
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                );
                match SyslineReader::sysline_pass_filters(&slp, &filter_dt_after_opt, &filter_dt_before_opt) {
                    Result_Filter_DateTime2::OccursBeforeRange | Result_Filter_DateTime2::OccursAfterRange => {
                        debug_eprintln!(
                            "{}sysline_pass_filters returned not Result_Filter_DateTime2::OccursInRange; continue!",
                            so()
                        );
                        continue;
                    }
                    Result_Filter_DateTime2::OccursInRange => {
                        print_slp(&slp);
                        if eof {
                            assert!(slr.is_sysline_last(&slp), "returned Ok_EOF yet this Sysline is not last!?");
                        } else {
                            assert!(!slr.is_sysline_last(&slp), "returned Ok yet this Sysline is last!? Should have returned Ok_EOF or this Sysline is really not last.");
                        }
                    }
                }
            }
            ResultS4_SyslineFind::Ok_Done => {
                debug_eprintln!("{}slr.find_sysline({}) returned Ok_Done", so(), fo2);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!("{}slr.find_sysline({}) returned Err({})", so(), fo2, err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
    }

    debug_eprintln!("{}Found {} Lines, {} Syslines", so(), slr.linereader.lines.len(), slr.syslines.len());
    /*
    for (i, line) in slr.linereader.lines.iter().enumerate() {
        print_colored(Color::Cyan, format!("{:2} {:3} '{}'\n", i, line.0, line.1.to_String_noraw()).as_bytes());
    }
    */
    debug_eprintln!("{}test_SyslineReader_w_filtering_2(…)", sx());
}

/// basic test of SyslineReader things
/// read all file offsets but randomly
#[allow(non_snake_case, dead_code)]
fn test_SyslineReader_rand(path_: &String, blocksz: BlockSz) {
    debug_eprintln!("{}test_SyslineReader_rand({:?}, {})", sn(), &path_, blocksz);
    let mut slr1 = match SyslineReader::new(path_, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({}, {}) failed {}", path_, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}SyslineReader {:?}", so(), slr1);
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(slr1.filesz() as usize);
    fill(&mut offsets_rand);
    debug_eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);
    randomize(&mut offsets_rand);
    debug_eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);

    for fo1 in offsets_rand {
        let result = slr1.find_sysline(fo1);
        match result {
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!("{}slr1.find_sysline({}) returned Err({})", so(), fo1, err);
                eprintln!("ERROR: {}", err);
            }
            _ => {}
        }
    }
    // should print the file as-is and not be affected by random reads
    slr1.print_all(true);
    debug_eprintln!("\n{}{:?}", so(), slr1);
    debug_eprintln!("{}test_SyslineReader_rand(…)", sx());
}
