// main.rs
/* … ≤ ≥ ≠ ≟
Successful `sort`. Passes all tests in run-tests including utf-8 with high-order characters.

(export RUST_BACKTRACE=1; cargo run -- --filepath Cargo.toml)
(cargo build && rust-gdb -ex 'layout split' -ex 'b src/main.rs:2062' -ex 'r' --args target/debug/super_speedy_syslog_searcher --filepath ./logs/other/tests/basic-dt.log 2>/dev/null)
(export RUST_BACKTRACE=1; cargo run -- --filepath ./logs/other/tests/test3-hex.log)

# compare performance to `sort`
/usr/bin/time -v -- ./target/release/super_speedy_syslog_searcher --path ./logs/other/tests/gen-*.log -- 0x1000 '20000101T000000'
/usr/bin/time -v -- sort -n -- ./logs/other/tests/gen-*.log

(
 # install:
 #   apt install -y linux-perf linux-tools-generic
 #
 # add to Cargo.toml
 #   [profile.bench]
 #   debug = true
 #   [profile.release]
 #   debug = true
 set -eu
 export CARGO_PROFILE_RELEASE_DEBUG=true;
 export PERF=$(realpath /usr/lib/linux-tools/5*-generic/perf)
 set -x;
 cargo build --release
 flamegraph -o flame-S4.svg ./target/release/super_speedy_syslog_searcher --path ./logs/other/tests/gen-*.log '20000101T000100'
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
This looks really fast for small strings (like DateTime Substrings)
https://docs.rs/arraystring/latest/arraystring/

Slices and references refresher:
    https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=0fe005a84f341848c491a92615288bad

Stack Offset refresher
    https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=2d870ad0b835ffc8499f7a16b1c424ec

"Easy Rust" book
https://erasin.wang/books/easy-rust/

The Rust Programing Book
https://doc.rust-lang.org/book/

testing Clone and Copy and Pointer
https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=203ff518e004f62a959ac4697daa24a5

TODO: [2022/05/01] would Perfect Hash Table module speed up anything?
      https://docs.rs/phf/latest/phf/

DROP: TODO: [2021/09/01] what about mmap? https://stackoverflow.com/questions/45972/mmap-vs-reading-blocks

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

DONE: [2021/09/16]
      clean up the confusing use Result. Create your own Result Enum that copies what is necessary
      from built-in code.

DONE: LAST WORKING ON [2021/09/17]
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

DONE: LAST WORKING ON [2021/09/20]
     Tried out flamegraph for fun.
     Now to convert `BlockReader.read_block` to use it's own typed `ResultS4`.
     Then fix the zero size bug, then resume work on function called by `test_SyslineReader_w_filtering`.

FIXED: BUG: [2021/09/20] file of zero size, or single line causes a crash.

LAST WORKING ON [2021/09/22]
     Revised debug indent printing.
     First implement the `fname` macro (search for it) mentioned, then replace debug prints.
     Then resume implementing `find_sysline_at_datetime_filter`. It's only job is to find one sysline
     closest to passed datetime filter and fileoffset. No need to loop on it.

LAST WORKING ON [2021/09/28 02:00:00]
     Just implemented `test_LineReader_1`. Now to resume implementing `test_SyslineReader_w_filtering_3`
     dealing with multiple files.... which I think I'm done with for now.
     Actually want to move on to basic implementation of multi-threaded file reading. No need to print
     in synchrony, just read in different threads, return something (what data can be returned from a finishing thread?).
     Later, work on synchronized printing based on datetime and filters.
     Oh, create a copy of all "TODO" up in this header comment area so I can precede with "DONE"

LAST WORKING ON [2021/10/01 01:10:00]
     Got `test_threading_3` running. It just prints syslog files among shared threads. No coordination.
     Next is to coordinate among the different threads.
     Each file processing threads must:
         processing thread (many):
           find a sysline datetime
               if no datetime found, send Done to `channel_dt`, exit.
           send datetime to channel `channel_dt`
           Wait on recv channel `print`
         coordinating thread (one):
            has map[???, channel_dt]
            waits to recv on several `channel_dt`
              if receive datetime then associates recieved datetime with a send channel
              if receive Done then removes the associated channel
            compares currently had datetimes, for winning datetime (soonest),
               find send channel, send to channel `print`
         processing thread (many):
           receives signal on channel `print`
           prints sysline
           (loop)
     ... carp, this ain't even all of it... there can be many files but only N processing threads.
     So given limited threads but one worker per file, need share a few threads among the many workers,
     like some sort of work pipeline.
     yet need to coordinate among all workers...
     Next implementation step should create one thread per passed file, then implement the datetime printing
     coordination mechanism.
     After that, work on the limited threads mechanism.

LAST WORKING ON [2021/10/02 02:26:00]
     Simplified `test_threading_3` much. One thread per file.
     Need to implement the main thread loop that reads the Sync_Receiver channels, and then
     chooses to print the soonest datetime.

TODO: [2021/10/02]
      need to add a SyslinePrinter that prints a given `Sysline`.
      A `Sysline` can print itself, however, one little troublesome detail:
      the last `Sysline` of a file often has no terminating '\n'
      When printing `Sysline` from many different files, it'll result in some Syslines
      getting printed on the same row in the CLI. Looks bad, is unexpected.
      To avoid that, the `SyslinePrinter` must be aware of when it is handling the last `Sysline`
      of a file, and write it's own '\n' to stdout.
      Alternative is to append a '\n' to the last Sysline during processing, but I really don't
      like that solution; breaks presumptions that `Sysline` (and underlying `Line`, `Block`)
      hold exactly what was read from the file.

DONE: TODO: [2021/10/02]
      Need to save the parsed datetime, very efficient without it.

TODO: [2021/10/03]
     Offer CLI option to fallback to some TZ. i.e. `--fallback-TZ -0800`
     Use this when TZ cannot be parsed.
     Consider, are there other TZ hints? Like, TZ of the hosting system?
     Or, is it embedded in any file attributes?
     Inform user of default fallback in `--help`.

TODO: [2021/10/03]
     Offer CLI option to fallback to current year. i.e. `--fallback-year 2018`
     For when year cannot be parsed.
     Consider, are there other Year hints?
     The file modified attribute may be very helpful here.
     Could other information be scraped from the file?
     Inform user of default fallback in `--help`.

LAST WORKING ON [2021/10/03 00:20:00]
     Have a simple multi-threaded reader, one thread per file.
     Next, need to improve coordination main thread to allow limited threads per storage source (i.e. x2 for "C:\", x2 for "D:\", etc.)
     May want to move `basic_threading_3` into a it's own proper function.
     I made a skeleton for SyslogWriter, but I'm not sure what to do with it.
     Perhaps that could be the thing that handles all the threading? Rename `SyslogsPrinter` ?

FIXED: [2021/10/04 01:05:00]
     Fails to parse datetime in datetime from file `logs/Ubuntu18/vmware-installer`, example sysline
         [2019-05-06 11:24:34,033] Installer running.
     Debug output shows an attempt to parse it, all parameters looks correct.
     Not sure what's happening here.
     Is there some odd character not visually obvious in the file or in the pattern? (i.e. a different "hyphen" character?)

TODO: [2021/10/05]
      Some sysline datetime variations not yet possible:
      - variable preceding string; in addition to a datetime pattern with offsets, add an optional
        preceding regexp pattern to try, then match datetime pattern after matched regexp
      - missing year
      - missing TZ (currently, always presumes `Local` TZ but after fixing that to allow for any TZ...)

BUG: [2021/10/06 00:03:00]
     fails for file `basic-basic-dt20.log`:
          for i in 00 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16 17 18 19 20 21; do
            (set -x;
            ./target/release/super_speedy_syslog_searcher --path ./logs/other/tests/basic-basic-dt20.log -- 65536 "2000-01-01 00:00:${i}"
            );
            echo; echo;
          done
     Bug appears to be in `find_sysline_at_datetime_filter`.
     As opposed to manually retesting on one file...

TODO: [2022/03/11]
      The concept of static datetime pattern lengths (beg_i, end_i, actual_beg_i, actual_end_i) won't work for
      variable length datetime patterns, i.e. full month names 'July 1, 2020' and 'December 1, 2020'
      See longer note in `find_datetime_in_line`

LAST WORKING ON [2022/03/18]
      Got many tests `test_find_sysline_at_datetime_filter*` Looks pretty solid.
      Now to review all recent "LAST WORKING ON/HERE" embedded in this.
      Then backtrack other recent TODOs.
      IIRC, the next small milestone was getting basic interleaved logs read.
      Running a smorgasborg of log files failed to sort them correctly:
          ./target/release/super_speedy_syslog_searcher --path $(find ./logs/other/tests/ ./logs/debian9/ ./logs/OpenSUSE15/ ./logs/synology/ ./logs/Ubuntu18/ -type f -not \( -name '*.gz' -o -name '*.xz' -o -name '*.tar' -o -name '*.zip' \) )
      :-(
      More unfortunately, this bug was only apparent for large numbers of large files.
      [2022/03/22]
      I'm beginning to think the files are not entirely parsed for datetime on syslines, and so
      I'm seeing the remnants.
      I experimented with processing many small files.
      first take the `head` of all the test log files, and create a temporary file that.
          $ find ./logs/other/tests/ ./logs/debian9/ ./logs/OpenSUSE15/ ./logs/synology/ ./logs/Ubuntu18/ -type f \
            -not \( -name '*.gz' -o -name '*.xz' -o -name '*.tar' -o -name '*.zip' \) \
            -exec ./tools/out-10.sh {} \;
      then process those temporary files:
          $ ./target/release/super_speedy_syslog_searcher --path ./tmp/ *
     the 100+ files in ./tmp/ were printed in correct datetime order.
     Perhaps it's a matter of file size? Could try to test with two or three very large files with
     interleaved datetimes. These should be generated so they are easy to visually verify, perhaps using
     odd/even numbers?
     ... after running various tests, I'm pretty sure the out of order is because the files daettime
     are not matched because the patterns are not known.

TODO: [2022/03/18] before opening a file, attempt to retrieve the file attributes.
      The Last Modified Time can be used when the datetime format does not include a year.
      e.g. sysline like
          Feb 23 06:37:40 server1 CRON[1242]: pam_unix(cron:session): session closed for user root
      this Last Modified Time could also be used to determine times in a dmesg or Xorg.log file
      e.g. syslines like
          [    91.203] Build Operating System: Linux 4.4.0-170-generic x86_64 Ubuntu

TODO: [2022/03/18] need summary of files not read, or message to user about files that had
      no syslines found. A simple check would be: if !file.is_empty && file.syslines.count()==0 then print warning.
      May want to add an option to print summary of findings after all files.
            --summary
      This might help me debug right now... 

TODO: [2022/03/22] need to track errors decoding UTF8, too many errors many means the file is not a text
      file, and should be abandoned for further processing.
      This error occurs as `Utf8Error` within `SyslineReader::find_datetime_in_line` call `str::from_utf8`.

TODO: [2022/03/22 01:53:10] easy speed up is to add functions to handle typical case of
      reading sysline by sysline. These would skip checks of LRU Cache and stuff like that, just try to
      get the line at the given offset, immediately stop if it's not found.
          fn next_line(fileoffset) 

TODO: [2022/03/22 02:10:12] add command-line option to bold/color the datetime string
      helpful for visual verifications.

FIXED: [2022/03/22 02:26:15] see above LAST WORKING ON; large files are not sorted correctly.

TODO: [2022/03] try cargo-nexttest https://github.com/nextest-rs/nextest/releases/tag/cargo-nextest-0.9.12

TODO: [2022/03/23 02:07:51] might be interesting to add a debug printing mode --datetime-only
      that prints *only* the datetime portion of sysline.
      This would help visually verify datetimes for a variety of files.

DONE: 2022/03/23 02:09:37 need to add the summary to help with debug inspections
        count of bytes (taken from blocks),
        count of lines (counted in insert_line),
        count of syslines (counted in insert_sysline),
        count of lines printed,
        count of syslines printed,
        first datetime printed (Datetime object and found string),
        last datetime printed (Datetime object and found string),
        datetime formats found (and count)
     make sure to print about files that did not find process *any* Lines or Syslines (binary files, empty file).
     Then add a bunch of found datetimes formats from various found files, see how things improve.

TODO: update my github profile with an introduction like here https://github.com/yuk7

TODO: submit PR to https://github.com/chronotope/chrono/issues/660 

TODO: [2022/03/23] add "streaming" built-in feature. This would conserve memory, and would be necessary
      for very large log files.
      The `SyslineReader` would delete processed `Syslines` (and the contained `Lines`, etc.), i.e.
      printed syslines.
      Have this behavior controlled by a global constant so before and after comparisons
      of memory usage can be done.

TODO: [2022/03/24] allow processing the same file more than once, like typical other Unix tools allow.
      Currently, various areas key by `FPath`, forcing a file to only be processed once.
      Instead, key by `(command-line index, FPath)`, e.g.
        $ super_speedy_syslog_search file1 file1
      would be keys
        { (1, file1), (2, file1) } 
      The duplication of work would be acceptable as it's an uncommon case.

TODO: [2022/03/24] add debug macro to print in color, `debug_eprintln_clr!(...)`
      this could be used for printing with color from different threads.
      Would very much help debug output readability.

NOTE: I attempted to use named records or structural records. Both crates were unsatisfactory.
      crate `named_tuple` wants to make copies of the fields for each `get`. I also abandoned compile
      problems with it.
         https://docs.rs/named_tuple/0.1.3/named_tuple/index.html#attributes
      crate `structx` seemed too incomplete
         https://crates.io/crates/structx/0.1.5

TODO: [2022/03/24] need option to dump files that have no found Syslines, as-is
      this would allow output testing compared to `cat`, i.e. script `tools/run-cat-cmp-tests.sh`
          --cat
      but that would get too weird, goes against purpose of program.
      however, a simple solution:
      add CLI option --nonl "no newline" mode that would refrain printing extra newlines
      at file ending. This would behave more like `cat` and allow testing script
      `tools/run-cat-cmp-tests.sh` to succeed.

LAST WORKING ON 2022/03/25 00:53:21 crashing when passed dt_filter
            ‣ ./target/release/super_speedy_syslog_searcher --path ./logs/other/tests/dtf5-2c.log ./logs/other/tests/dtf5-6c.log -- 0xFFFFF 20000101T000100
            thread '<unnamed>' panicked at 'thread 'assertion failed: `(left <= right)`
              left: `279`,
             right: `254`: unexpected values (*SyslineP).fileoffset_end() 279, fileoffset returned by find_sysline 254 FPath "./logs/other/tests/dtf5-6c.log"<unnamed>', ' panicked at 'src/main.rsassertion failed: `(left <= right)`
              left: `65`,
             right: `63`: unexpected values (*SyslineP).fileoffset_end() 65, fileoffset returned by find_sysline 63 FPath "./logs/other/tests/dtf5-2c.log":', 5005src/main.rs::295005
            :note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
            29
            Rayon: detected unexpected panic; aborting
            Aborted
        To see
            ./target/debug/super_speedy_syslog_searcher --path ./logs/other/tests/dtf5-2c.log -- 0xFFFFF 20000101T000100

TODO: [2022/03/25] is it possible to further limit the places where `Bytes` are decoded to `str` or `String`?
      Might help. Should also write an explanatory NOTE about this when completed.

NOTE: example of error[E0716] temporary value dropped while borrowed
                 creates a temporary which is freed while still in use
      https://play.rust-lang.org/?version=beta&mode=debug&edition=2021&gist=644fa5db11aebc49d66a7d25478e3893

DONE: LAST WORKING ON 2022/03/26 02:13:44 suble problem when dt_filter_after is passed; first line of file is printed twice.
      the WARNING is obvious place to start, and use a debug build.
      Also, why didn't #[test] cases catch this? It's probably not covered by them, and it should be.
      here is the output of test file ./logs/other/tests/dtf5-6b.log
                    ▶ bat ./logs/other/tests/dtf5-6b.log
                    ───────┬──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
                           │ File: ./logs/other/tests/dtf5-6b.log
                           │ Size: 215 B
                    ───────┼──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
                       1   │ 2000-01-01 00:00:00 [dtf5-6b]
                       2   │
                       3   │ 2000-01-01 00:00:01 [dtf5-6b]a
                       4   │ a
                       5   │ 2000-01-01 00:00:02 [dtf5-6b]ab
                       6   │ ab
                       7   │ 2000-01-01 00:00:03 [dtf5-6b]abc
                       8   │ abc
                       9   │ 2000-01-01 00:00:04 [dtf5-6b]abcd
                      10   │ abcd
                      11   │ 2000-01-01 00:00:05 [dtf5-6b]abcde
                      12   │ abcde
                    ───────┴──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
                    
                    ▶ ./target/release/super_speedy_syslog_searcher --path ./logs/other/tests/dtf5-6b.log -- 0xFFFFF 20000101T000001
                    WARNING: fo_last 32 != 64 slp.fileoffset_next() (fo_end is 215)
                    2000-01-01 00:00:01 [dtf5-6b]a
                    a
                    2000-01-01 00:00:01 [dtf5-6b]a
                    a
                    2000-01-01 00:00:02 [dtf5-6b]ab
                    ab
                    2000-01-01 00:00:03 [dtf5-6b]abc
                    abc
                    2000-01-01 00:00:04 [dtf5-6b]abcd
                    abcd
                    2000-01-01 00:00:05 [dtf5-6b]abcde
                    abcde
                    
                    Summary:
                    File: "./logs/other/tests/dtf5-6b.log"
                       Summary: { bytes: 215, lines: 12, syslines: 6, blocks: 1, blocksz: 1048575 }
                       Printed: { bytes: 217, lines: 12, syslines: 6 }

TODO: 2022/03/26 19:49:01 for using `&FPath` or `FPath` for a key value, instead try
      `Box<FPath>`.  This would also allow passing the `FPath` between channels. No need for
      my other TODO idea of adding an a lookup table.
      There could be one global static list of files that this `Box` refers to.
      See https://stackoverflow.com/a/71626319/471376

LAST WORKING ON 2022/03/27 03:09:44 fixed prior bug.
      Now to resume... should I add some test cases for find_sysline_by_datetime?
      Or reach for the next milestone ASAP? What was teh next milestone?
      Just read back upwards, see what's next best to work on...
      Should I write a shell script to `sort` log file datetimes, then compare performances (obv. no filtering)?
      What about comparing to `cat`?

DONE: TODO: [2022/03/27]
      Summary printing should print file names in same colors the lines were printed.
      Should the printed datetime be a variation on the file color? Visually, that would help much.

DONE: [2022/03/27]
      add a Summary total at end of Summary,
      i.e. count of lines printed for *all* files processed, syslines, bytes, etc.
           first found datetime, last found datetime

DONE: TODO: [2022/03/29]
      add a --summary command-line option for controlling print of Summary.

FIXED: [2022/03/27 18:29:03] passing both datetime filters results in a printed error
        ▶ ./target/release/super_speedy_syslog_searcher --path ./logs/other/tests/dtf5-6b.log -- 0xFFFFF 20000101T000001 20000101T000002
        ...
        ERROR: SyslineReader@0x7f9ca0a6fe90.find_sysline(216) Passed fileoffset 216 past file size 215

TODO: [2022/03/28 00:11:00] need mutex for writing to stdout, stderr. There are unexpected color changes.
      Also would add some certainty about the prints occurring.
      This would only be used in debug mode.

FIXED: BUG: [2022/03/28 00:36:27] `find_sysline_by_datetime` is searching linearly, not binary search.
        and it errantly prints the first sysline found.
            ▶ head -n 1 ./gen-100-10-FOOBAR.log
            20000101T080000 FOOBAR
            ▶ tail -n 1 ./gen-100-10-FOOBAR.log
            20000101T080139 9abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWZYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìí FOOBAR
            ▶ ./target/release/super_speedy_syslog_searcher --path  ./gen-100-10-FOOBAR.log -- 0xFFF 20000101T000129 20000101T000145
            20000101T080000 FOOBAR
            
            Summary:
            File: ./gen-100-10-FOOBAR.log
               Summary Processed: { bytes: 97050, bytes total: 97050, lines: 1000, syslines: 1000, blocks: 24, blocksz: 4095 (0xFFF), blocks total: 24 }
               Summary Printed  : { bytes: 23, lines: 1, syslines: 1, dt_first: 2000-01-01 08:00:00 +00:00, dt_last: 2000-01-01 08:00:00 +00:00 }
            Summary Printed  : { bytes: 23, lines: 1, syslines: 1, dt_first: 2000-01-01 08:00:00 +00:00, dt_last: 2000-01-01 08:00:00 +00:00 }
            ▶ ./target/debug/super_speedy_syslog_searcher --path  ./gen-100-10-FOOBAR.log -- 0xFF 20000101T000129 20000101T000145 2>&1 | head -n 999

TODO: 2022/03/29 add `BlockReader.read_block` semaphore https://crates.io/crates/async-semaphore 
      For first attempt, just add one global mutex for all `BlockerReader` paths processed.
      Later, can get specific by examining the path sources, and creating mutexes for each source (`C:\`, `D:\`, etc.).
      Would be cool to have some stats on total wait time, but that's a feature creep goal.
      Keep in mind, it's necessary for all BlockReaders to read, since all files need simultaneous processing.
      And each of them pauses when waiting in the `exec_4`... (right?)
      However, an underlying BlockReader async-semaphore could service the read requests.
      Though... does not the underlying OS handle this already? This might be of little importance.

LAST WORKING ON [2022/03/30 23:45:30] according to tools/flamegraph.sh output, a good thing to
       optimize next would be `find_datetime_in_line`, specifically, avoid unnecessary calls to
       `datetime_from_str`.
       Or lessen calls to `str::from_utf8`, they are time-consuming. Use `bstr` instead.
       Perhaps write a little test program for this comparison.
       Also curious how much time is shaved off by printing without color. Should add a --color={auto, always, none}
       option next.

DONE: write a quickie script to run `cat | sort` and compare runtimes.

TODO: 2022/03/31 for files with datetime format lacking year, it will be necessary
      to determine if year rollover occurred *before printing or creating syslines* (or some mechanism to revise
      previously created syslines). This will mean beginning a backwards search for syslines
      *at the end* of the files (no matter the passed `dt_filter_before`).
      This would cause much reading of data that would otherwise want to avoid reading.
      It would cool to add this, though it would be much delicate work.
      Here are some related implementation options:
      Syslog datetime formats without a year should emit a printed warning:
        "Warning: syslog file uses datetime format that does not include a year. Cannot be processed."
      and then stop processing that file.
      or, if the file metadata "Last Modified Time" can be retrieved, then could print
        "Warning: syslog file uses datetime format that does not include a year. Assuming year to be Last Modified Time XXXX".
      or, do one huge iteration from the last line of the file, tracking years, but never keeping any data (so it doesn't blowout memory).
      This is a time resource intensive hack that *would* work.

DONE: TODO: 2022/03/31 add option --assume-TZ-offset that allows passing TZ offset that will be assumed for syslog
      files that use datetime format without a timezone offset.
      Would default to `DateTime.Local` offset.

DONE: LAST WORKING ON 2022/04/01 trying to improve on `RangeMap`. It takes a surprising amount of runtime according to flamegraph.
      I wonder if I could roll my own? It only needs two special functions, `contains` and `get`.
      I tried this with help of `std::collections::BTreeMap.range`. The one problem is `.range` cannot search *backwards*
      for the nearest matching key (it does search forwards). i.e. can't do a `map.range(23).prev()`
      *thinking face*
      I wonder if I could get away from using the `lines_by_range` at all? Definitely worth a try.
      See https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=e2f8254986338694b3ec467e606b73e7

TODO: 2022/04/06 add script to run `tools/compare-...sh` and `tools/flamegraph.sh`, save outputs
      based on commit and/or tag. This way it's easy to save performance data
      for each new commit/tag.
      Write output to path `./performance-data/$(hostname)/$(rust --print ...)/`
      files:
        `flamegraph-$HASH.svg`
        `compare-$HASH.out`

TODO: 2022/04/07 add directory walks and file finding

TODO: 2022/04/07 need to handle formats with missing year and year rollover.
      not easy!

TODO: 2022/04 add streaming capability/mode.
      this would be manually enabled by file processing thread. do this after
      file is analysed as acceptable syslog file.
      As syslines are found, old sysline `data` and other associated data is `drop`.
      not easy.
      need memory profiler to verify it's working.

DONE: TODO: 2022/04/07 need to handle formats with explicit timezone offset.
      see example `access.log`

DONE: TODO: 2022/04/09 in `find_datetime_in_line`, the `slice_.contains(&b'1')`
      use much runtime. Would doing this search manually be faster?
      Create a benchmark to find out.

DONE: TODO: 2022/04/29 easy addition:
      add CLI option for color: --color={none,always,auto} default to `auto`

SKIP: TODO: 2022/04/29 easy addition:
      check `filesz` is less than `u64::MAX`, return Error `unsupported` if it is.
      do this in BlockReader::open
      ANSWER: std::fs:File.metadata().len() returns `u64`

TODO: [2022/04/30]
      efficiency addition:
      if
        processing last file (one file)
        found sysline with datetime after dt_time_after
        dt_time_before is None
      then
        dump the file from fileoffset A
        (no need to parse Lines, Syslines)
      Implementation requires sending info back to file processing thread.

TODO: [2022/04/30]
      efficiency addition:
      if
        processing last file (one file)
        found sysline with datetime after dt_time_after
        found sysline with datetime before dt_time_after
      then
        dump the file from fileoffset A to fileoffset B

TODO: [2022/05/01] add parsing of Windows Startup files (which are in XML)?
      See examples in "C:\Windows\System32\WDI\LogFiles\StartupInfo\"
      Would require O(n) analysis. This might be acceptable for the
      benefit.
      However...
      I could hardcode schemas for well-known XML files (like Windows startup logs)
      but how to allow user to pass in schemas for unknown XML formats? Might
      become too much work, and feature creep... but would be cool.
      See https://github.com/RazrFalcon/roxmltree

TODO: [2022/05/10] does the `SyslineReader::find_sysline` binary search use
      similar range check as done in `LineReader::find_line` ?

TODO: [2022/05/31] stats counters should be `type uStat = u64`;

DONE: TODO: [2022/06/02] remove the lifetime specifier from
      SyslineReader, Linereader, BlockReader.

DONE: TODO: [2022/06/02] move `Sysline` to file `data/sysline.rs`
      move `Line` to file `data/line.rs`

TODO: 2022/06/01 put mimetype into crate-defined enums
      (`TEXT`, `GZ_TEXT`, `XZ_TEXT`, `TAR_TEXT`, `TAR_DIR_FILES`, `GZ_TAR`, ...)
      then implement handling of just `TEXT` and `GZ_TEXT`, others should return `Err("not yet supported")`.
      but before that, handle teh different processing modes mentioned elsewhere

TODO: 2022/06/06 enums can contain instances. Consider making more enums
      contain the thing of interest. The is a better representation of meaning.

TODO: 2022/06/06 add CLI option to not cross filesystem boundaries during recurse
      https://docs.rs/walkdir/latest/walkdir/struct.WalkDir.html#method.same_file_system

TODO: 2022/06/07 add CLI option to print filename, filepath, align them
      --prepend-name --prepend-path --prepend-align

TODO: 2022/06/06 move SyslineReader datetime state chaanges into SyslogProcessor
      The SyslogProcessor should set the allowed datetime patterns to use, and should understand
      state changes. SyslineReader should not know about state changes.

TODO: 2022/06/11 consistent naming:
      there is `File_Offset`, `FileOffset` `file_offset` `fileoffset, same for `blockoffset` `lineindex` etc.
      for variable names, function names, and types.
      pick with or without middle `_` and always use that

TODO: 2022/06/13 consistent naming:
      function names are `count_this` and `this_count`. Same for field names.
      Typical rust phraseology is `verb_object`

TODO: 2022/06/14 add typing for file size, put in common.rs and use
         pub type FileSz = u64;

LAST WORKING ON 2022/06/14 00:09:03
      Next two items:
      1. Streaming mode:
        a. Must improve blockzero analysis steps, including when to truncate datetime
           formats to check.
        b. Must drop processed+printed data.
      2. Improve printing efficiency.
        Change `Sysline::print_color` and helper functions.
        to pass around `&[&[u8]]` instead of `Vec<&[u8]>`.
        Probably just want to clean them up, they're all over the place.
        Consider,
        Delete current hackjob of functions (leave the print functions for debug builds), and instead:
        Add to `printer/printers.rs`.
        A new struct that holds stdout_lock and various colorization settings.
        Has specialized functions for each specific printing situation (avoid general purpose stuff):
            // prints without color
            pub fn print_sysline(&SyslineP) -> Result<()>
            pub fn print_prependdate_sysline(prepend_date: &String, &SyslineP) -> Result<()>
            pub fn print_prependname_prependdate_sysline(prepend_name: &String, prepend_date: &String, &SyslineP) -> Result<()>
            // prints with color
            pub fn print_color_sysline(&SyslineP, color_sysline: Color) -> Result<()>
            pub fn print_color_prependdate_sysline(prepend_date: &String, color_date: Color, &SyslineP, color_sysline: Color) -> Result<()>
            pub fn print_color_prependname_prependdate_sysline(prepend_name: &String, color_name: Color, prepend_date: &String, color_date: Color, &SyslineP, color_sysline: Color) -> Result<()>
        those functions could be called by something that analyzes passed `termcolor::ColorChoice::Never`
        *or*
        the processing_loop could have a many-armed match statement that calls the
        function based on settings it knows (and has precalculated).
        This should be placed in `#[inline] fn print(&settings, ...)`
        or whatever avoids extraneous lookups of colors when not needed.
        ...
        Does `enum_BoxPtrs` fit into more efficient iterating over `&[u8]` ?
      - Colorizing the datetime in the sysline should be a CLI option.

TODO: BUG: 2022/06/14 passing same file twice on CLI results in channels getting disconnected too soon

*/

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// uses
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::collections::{
    HashMap,
};
use std::fmt;
//use std::fs::{File, Metadata, OpenOptions};
//use std::io;
//use std::io::prelude::Read;
//use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};
//use std::path::Path;
//use std::ops::RangeInclusive;
use std::str;
use std::str::FromStr;  // attaches `from_str` to various built-in types
//use std::sync::Arc;
use std::thread;

//extern crate arrayref;
//use arrayref::array_ref;

extern crate atty;

extern crate backtrace;

extern crate chain_cmp;
use chain_cmp::chmp;

extern crate clap;
use clap::{
    ArgEnum,
    PossibleValue,
    Parser,
};

extern crate crossbeam_channel;

extern crate chrono;
use chrono::{
    //DateTime,
    FixedOffset,
    Local,
    //NaiveDateTime,
    TimeZone,
    Utc,
};
use chrono::offset::{
    Offset,
};

extern crate debug_print;
use debug_print::{
    //debug_eprint,
    debug_eprintln
};
#[allow(unused_imports)]
use debug_print::{
    debug_print,
    debug_println
};

extern crate encoding_rs;

//extern crate enum_utils;

//extern crate enum_display_derive;
//use enum_display_derive::Display;
// XXX: I do not understand why importing the same name does not cause problems.
use std::fmt::Display;

//extern crate lru;
//use lru::LruCache;

//extern crate lazy_static;
//use lazy_static::lazy_static;

extern crate more_asserts;
use more_asserts::{
    //assert_le,
    //assert_lt,
    //assert_ge,
    assert_gt,
    //debug_assert_le,
    //debug_assert_lt,
    //debug_assert_ge,
    //debug_assert_gt
};

//extern crate rand;

//extern crate rangemap;
//use rangemap::RangeMap;

//extern crate mut_static;

//extern crate unroll;
//use unroll::unroll_for_loops;

mod common;
use crate::common::{
    FPath,
    FPaths,
    FileOffset,
    FileType,
    //Bytes,
    //NLu8,
    NLu8a,
};

mod Data;
use crate::Data::line::{
    LineIndex,
};

use Data::datetime::{
    DateTimeL_Opt,
    DateTime_Parse_Data_str,
    //Local,
    //Utc,
    //Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    DateTime_Parse_Data_str_to_DateTime_Parse_Data,
    str_datetime,
};

mod dbgpr;
use dbgpr::stack::{
    so,
    sn,
    sx,
    snx,
    stack_offset_set,
};

mod printer;
use printer::printers::{
    Color,
    COLOR_DATETIME,
    color_rand,
    print_colored_stdout,
    print_colored_stderr,
    write_stdout,
};

mod Readers;

use Readers::blockreader::{
    BlockSz,
    //BlockIndex,
    //BlockOffset,
    //Block,
    //BlockP,
    //Slices,
    //Blocks,
    //BlocksLRUCache,
    //EndOfFile,
    BLOCKSZ_MIN,
    BLOCKSZ_MAX,
    BLOCKSZ_DEFs,
    //BlockReader,
};

use Readers::filepreprocessor::{
    ProcessPathResult,
    ProcessPathResults,
    process_path,
};

use Readers::summary::{
    Summary,
    Summary_Opt,
};

use Readers::syslinereader::{
    SyslineP,
    SyslineP_Opt,
    ResultS4_SyslineFind,
    SyslineReader,
};

use Readers::syslogprocessor::{
    SyslogProcessor,
    FileProcessingResult_BlockZero,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// misc. globals
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// global constants

/// global test initializer to run once
/// see https://stackoverflow.com/a/58006287/471376
//static _Test_Init_Once: Once = Once::new();

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// command-line parsing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// XXX: unable to get `strum_macros::EnumString` to compile
/// CLI enum that is mapped to `termcolor::ColorChoice`
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    ArgEnum,  // clap
    //strum_macros::Display,
    //strum_macros::FromRepr,
    //strum_macros::AsRefStr,
)]
enum CLI_Color_Choice {
    always,
    auto,
    never,
}

const CLI_DT_FILTER_PATTERN1: &DateTime_Parse_Data_str = &("%Y%m%dT%H%M%S", true, false, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN2: &DateTime_Parse_Data_str = &("%Y%m%dT%H%M%S%z", true, true, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN3: &DateTime_Parse_Data_str = &("%Y-%m-%d %H:%M:%S", true, false, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN4: &DateTime_Parse_Data_str = &("%Y-%m-%d %H:%M:%S %z", true, true, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN5: &DateTime_Parse_Data_str = &("%Y-%m-%dT%H:%M:%S", true, false, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN6: &DateTime_Parse_Data_str = &("%Y-%m-%dT%H:%M:%S %z", true, true, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN7: &DateTime_Parse_Data_str = &("%Y/%m/%d %H:%M:%S", true, false, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN8: &DateTime_Parse_Data_str = &("%Y/%m/%d %H:%M:%S%z", true, true, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN9: &DateTime_Parse_Data_str = &("%Y/%m/%d %H:%M:%S %z", true, true, 0, 0, 0, 0);
// TODO: [2022/06/07] allow passing only a date, fills HMS with 000
//const CLI_DT_FILTER_PATTERN10: &DateTime_Parse_Data_str = &("%Y/%m/%d", true, false, 0, 0, 0, 0);
//const CLI_DT_FILTER_PATTERN11: &DateTime_Parse_Data_str = &("%Y-%m-%d", true, false, 0, 0, 0, 0);
//const CLI_DT_FILTER_PATTERN12: &DateTime_Parse_Data_str = &("%Y%m%d", true, false, 0, 0, 0, 0);
const CLI_FILTER_PATTERNS_COUNT: usize = 9;
/// acceptable datetime filter patterns for the user-passed `-a` or `-b`
const CLI_FILTER_PATTERNS: [&DateTime_Parse_Data_str; CLI_FILTER_PATTERNS_COUNT] = [
    CLI_DT_FILTER_PATTERN1,
    CLI_DT_FILTER_PATTERN2,
    CLI_DT_FILTER_PATTERN3,
    CLI_DT_FILTER_PATTERN4,
    CLI_DT_FILTER_PATTERN5,
    CLI_DT_FILTER_PATTERN6,
    CLI_DT_FILTER_PATTERN7,
    CLI_DT_FILTER_PATTERN8,
    CLI_DT_FILTER_PATTERN9,
    //CLI_DT_FILTER_PATTERN10,
    //CLI_DT_FILTER_PATTERN11,
    //CLI_DT_FILTER_PATTERN12,
];
/// datetime format printed for CLI options `-u` or `-l`
const CLI_OPT_PREPEND_FMT: &str = "%Y%m%dT%H%M%S%.6f %z:";

const CLI_HELP_AFTER: &str = "\
DateTime Filter patterns may be:
    '%Y%m%dT%H%M%S'
    '%Y%m%dT%H%M%S%z'
    '%Y-%m-%d %H:%M:%S'
    '%Y-%m-%d %H:%M:%S %z'
    '%Y-%m-%dT%H:%M:%S'
    '%Y-%m-%dT%H:%M:%S %z'
    '%Y/%m/%d %H:%M:%S'
    '%Y/%m/%d %H:%M:%S%z'
    '%Y/%m/%d %H:%M:%S %z'

Without a timezone offset (%z), the datetime is presumed to be the system timezone.

DateTime Filter formatting is described at
https://docs.rs/chrono/latest/chrono/format/strftime/

Prepended datetime, -u or -l, is printed in format '%Y%m%dT%H%M%S%.6f %z'.

DateTimes supported are only of the Gregorian calendar.
DateTimes languages is English.";

// TODO: change OPTIONS listing ordering (https://github.com/clap-rs/clap/discussions/3593)
// references:
// inference types https://github.com/clap-rs/clap/blob/v3.1.6/examples/derive_ref/README.md#arg-types
// other `clap::App` options https://docs.rs/clap/latest/clap/struct.App.html
// the `about` is taken from `Cargo.toml:[package]:description`
#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about,
    after_help = CLI_HELP_AFTER,
    before_help = "Super Speedy Syslog Searcher will search syslog files and sort entries by datetime. It aims to be very fast. DateTime filters can passed to narrow the search.",
    setting = clap::AppSettings::DeriveDisplayOrder,
)]
/// this is the `CLI_Args` docstring, is it captured by clap?
struct CLI_Args {
    /// Path(s) of syslog files or directories.
    /// Directories will be recursed, remaining on the same filesystem.
    /// Symlinks will be followed.
    #[clap(required = true)]
    paths: Vec::<String>,
    //#[clap(parse(from_os_str))]
    //paths: Vec::<std::path::PathBuf>,

    /// DateTime Filter after.
    #[clap(
        short = 'a',
        long,
        help = "DateTime After filter - print syslog lines with a datetime that is at or after this datetime. For example, '20200102T123000'",
    )]
    dt_after: Option<String>,

    /// DateTime Filter before.
    #[clap(
        short = 'b',
        long,
        help = "DateTime Before filter - print syslog lines with a datetime that is at or before this datetime. For example, '20200102T123001'",
    )]
    dt_before: Option<String>,

    /// Default timezone offset for naive datetimes (without timezone offset)
    #[clap(
        short = 't',
        long,
        help = "DateTime Timezone offset - for syslines with a datetime that does not include a timezone, this will be used. For example, '-0800' '+0200'. If passing a value with leading '-', use the '=' to explicitly set the argument, e.g. '-t=-0800'. Otherwise the CLI argument parsing will fail.",
        validator = cli_validate_tz_offset,
    )]
    tz_offset: Option<String>,

    /// Prepend DateTime in the UTC Timezone for every sysline.
    #[clap(
        short = 'u',
        long = "prepend-utc",
        group = "prepend_dt",
    )]
    prepend_utc: bool,

    /// Prepend DateTime in the Local Timezone for every sysline.
    #[clap(
        short = 'l',
        long = "prepend-local",
        group = "prepend_dt",
    )]
    prepend_local: bool,

    /// Prepend file basename to every sysline.
    #[clap(
        short = 'n',
        long = "prepend-filename",
        group = "prepend_file",
    )]
    prepend_filename: bool,

    /// Prepend file full path to every sysline.
    #[clap(
        short = 'p',
        long = "prepend-filepath",
        group = "prepend_file",
    )]
    prepend_filepath: bool,

    /// Aligh column width of prepended file basename or file path.
    #[clap(
        short = 'w',
        long = "prepend-file-align",
    )]
    prepend_file_align: bool,

    /// Choose to print to terminal using colors.
    #[clap(
        required = false,
        short = 'c',
        long = "--color",
        arg_enum,
        default_value_t=CLI_Color_Choice::auto,
    )]
    color_choice: CLI_Color_Choice,

    /// Read blocks of this size. May pass decimal or hexadecimal numbers.
    #[clap(
        required = false,
        short = 'z',
        long,
        default_value_t = BLOCKSZ_DEFs.to_string(),
        validator = cli_validate_blocksz,
    )]
    blocksz: String,

    /// Print ending summary of files processed. Printed to stderr.
    #[clap(
        short,
        long,
    )]
    summary: bool,
}

/// CLI argument processing
fn cli_process_blocksz(blockszs: &String) -> std::result::Result<u64, String> {
    // TODO: there must be a more concise way to parse numbers with radix formatting
    let blocksz_: u64;
    let errs = format!("Unable to parse a number for --blocksz {:?}", blockszs);

    if blockszs.starts_with("0x") {
        blocksz_ = match BlockSz::from_str_radix(blockszs.trim_start_matches("0x"), 16) {
            Ok(val) => val,
            Err(err) => { return Err(format!("{} {}", errs, err)) }
        };
    } else if blockszs.starts_with("0o") {
        blocksz_ = match BlockSz::from_str_radix(blockszs.trim_start_matches("0o"), 8) {
            Ok(val) => val,
            Err(err) => { return Err(format!("{} {}", errs, err)) }
        };
    } else if blockszs.starts_with("0b") {
        blocksz_ = match BlockSz::from_str_radix(blockszs.trim_start_matches("0b"), 2) {
            Ok(val) => val,
            Err(err) => { return Err(format!("{} {}", errs, err)) }
        };
    } else {
        blocksz_ = match blockszs.parse::<BlockSz>() {
            Ok(val) => val,
            Err(err) => { return Err(format!("{} {}", errs, err)) }
        };
    }

    if ! chmp!(BLOCKSZ_MIN <= blocksz_ <= BLOCKSZ_MAX) {
        return Err(format!("--blocksz must be {} ≤ BLOCKSZ ≤ {}, it was {:?}", BLOCKSZ_MIN, BLOCKSZ_MAX, blockszs));
    }

    Ok(blocksz_)
}

/// argument validator for clap
/// see https://github.com/clap-rs/clap/blob/v3.1.6/examples/tutorial_derive/04_02_validate.rs
fn cli_validate_blocksz(blockszs: &str) -> clap::Result<(), String> {
    match cli_process_blocksz(&String::from(blockszs)) {
        Ok(_) => {},
        Err(err) => { return Err(err); }
    }
    Ok(())
}

/// CLI argument processing
/// TODO: move some of this into small testable helper functions
fn cli_process_tz_offset(tzo: &String) -> std::result::Result<FixedOffset, String> {
    let tzo_: String;
    if tzo.as_str() == "" {
        // ripped from https://stackoverflow.com/a/59603899/471376
        let local_offs = Local.timestamp(0, 0).offset().fix().local_minus_utc();
        let hours = local_offs / 3600;
        let mins = local_offs % 3600;
        tzo_ = format!("{:+03}{:02}", hours, mins);
    } else {
        tzo_ = tzo.clone();
    }
    let fo_val = match i32::from_str_radix(tzo_.as_str(), 10) {
        Ok(val) => val,
        Err(err) => {
            return Err(err.to_string());
        }
    };
    let hours: i32 = fo_val / 100;
    let mins: i32 = fo_val % 100;
    let east: i32 = (hours * 3600) + (mins * 60);
    let fo = match FixedOffset::east_opt(east) {
        Some(val) => val,
        None => {
            return Err(format!("Unable to parse a timezone FixedOffset for -t {:?} (value {:?})", tzo, east));
        }
    };

    Ok(fo)
}

/// argument validator for clap
fn cli_validate_tz_offset(blockszs: &str) -> std::result::Result<(), String> {
    match cli_process_tz_offset(&String::from(blockszs)) {
        Ok(_) => { Ok(()) },
        Err(err) => { Err(err) },
    }
}

/// helper to `cli_process_args`
fn process_dt(dts: Option<String>, tz_offset: &FixedOffset) -> DateTimeL_Opt {
    // parse datetime filters
    match dts {
        Some(dts) => {
            let mut dto: DateTimeL_Opt = None;
            for dtpds in CLI_FILTER_PATTERNS.iter() {
                let dtpd = DateTime_Parse_Data_str_to_DateTime_Parse_Data(dtpds);
                debug_eprintln!("{}str_datetime({:?}, {:?}, {:?}, {:?})", so(), dts, dtpd.pattern, dtpd.tz, tz_offset);
                #[allow(clippy::single_match)]
                match str_datetime(dts.as_str(), &dtpd.pattern, dtpd.tz, tz_offset) {
                    Some(val) => {
                        dto = Some(val);
                        break;
                    }
                    _ => {}
                };
            };
            if dto.is_none() {
                eprintln!("ERROR: Unable to parse a datetime for --dt-after {:?}", dts);
                std::process::exit(1);
            }
            dto
        },
        None => { None },
    }
}

/// process passed CLI arguments into types
/// this function will `std::process::exit` if there is an `Err`
fn cli_process_args() -> (
    FPaths,
    BlockSz,
    DateTimeL_Opt,
    DateTimeL_Opt,
    FixedOffset,
    termcolor::ColorChoice,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
) {
    let args = CLI_Args::parse();

    debug_eprintln!("{} {:?}", so(), args);

    //
    // process string arguments into specific types
    //
    
    let blockszs: String = args.blocksz;
    let blocksz: BlockSz = match cli_process_blocksz(&blockszs) {
        Ok(val) => { val },
        Err(err) => {
            eprintln!("ERROR: {}", err);
            std::process::exit(1);
        }
    };
    debug_eprintln!("{} blocksz {:?}", so(), blocksz);

    let mut fpaths: Vec<FPath> = Vec::<FPath>::new();
    for path in args.paths.iter() {
        fpaths.push(path.clone());
    }

    let tz_offset: FixedOffset = match cli_process_tz_offset(
        &args.tz_offset.unwrap_or_default()
    ) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            std::process::exit(1);
        }
    };
    debug_eprintln!("{} tz_offset {:?}", so(), tz_offset);

    let filter_dt_after: DateTimeL_Opt = process_dt(args.dt_after, &tz_offset);
    debug_eprintln!("{} filter_dt_after {:?}", so(), filter_dt_after);
    let filter_dt_before: DateTimeL_Opt = process_dt(args.dt_before, &tz_offset);
    debug_eprintln!("{} filter_dt_before {:?}", so(), filter_dt_before);

    #[allow(clippy::single_match)]
    match (filter_dt_after, filter_dt_before) {
        (Some(dta), Some(dtb)) => {
            if dta > dtb {
                eprintln!("ERROR: Datetime --dt-after ({}) is after Datetime --dt-before ({})", dta, dtb);
                std::process::exit(1);
            }
        },
        _ => {},
    }

    // map `CLI_Color_Choice` to `termcolor::ColorChoice`
    let color_choice: termcolor::ColorChoice = match args.color_choice {
        CLI_Color_Choice::always => termcolor::ColorChoice::Always,
        CLI_Color_Choice::auto => termcolor::ColorChoice::Auto,
        CLI_Color_Choice::never => termcolor::ColorChoice::Never,
    };

    (
        fpaths,
        blocksz,
        filter_dt_after,
        filter_dt_before,
        tz_offset,
        color_choice,
        args.prepend_utc,
        args.prepend_local,
        args.prepend_filename,
        args.prepend_filepath,
        args.prepend_file_align,
        args.summary
    )
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// command-line parsing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// process user-passed command-line arguments
pub fn main() -> std::result::Result<(), chrono::format::ParseError> {
    // set once, use `stackdepth_main` to access `_STACKDEPTH_MAIN`
    if cfg!(debug_assertions) {
        stack_offset_set(Some(0 ));
    }
    debug_eprintln!("{}main()", sn());
    let (
        paths,
        blocksz,
        filter_dt_after,
        filter_dt_before,
        tz_offset,
        color_choice,
        cli_opt_prepend_utc,
        cli_opt_prepend_local,
        cli_opt_prepend_filename,
        cli_opt_prepend_filepath,
        cli_opt_prepend_file_align,
        cli_opt_summary,
    ) = cli_process_args();

    let mut processed_paths: ProcessPathResults = ProcessPathResults::with_capacity(paths.len() * 4);
    for fpath in paths.iter() {
        let ppaths: ProcessPathResults = process_path(fpath);
        for ppresult in ppaths.into_iter() {
            processed_paths.push(ppresult);
        }
        /*
        // TODO: [2022/06/06] carry forward invalid paths for printing with the `--summary`
        // XXX: can this be done in a one-liner?
        for processpathresult in ppaths.iter()
            .filter(|x| matches!(x,  ProcessPathResult::FILE_VALID(_)))
        {
            let path: FPath = match filetype_path {
                ProcessPathResult::FILE_VALID(val) => val.1,
                _ => { continue; },
            };
            processed_paths.push(path.clone());
        }
        */
    }

    processing_loop(
        processed_paths,
        blocksz,
        &filter_dt_after,
        &filter_dt_before,
        tz_offset,
        color_choice,
        cli_opt_prepend_utc,
        cli_opt_prepend_local,
        cli_opt_prepend_filename,
        cli_opt_prepend_filepath,
        cli_opt_prepend_file_align,
        cli_opt_summary,
    );

    debug_eprintln!("{}main() return Ok(())", sx());

    Ok(())
}

// -------------------------------------------------------------------------------------------------
// processing threads
// -------------------------------------------------------------------------------------------------

// TODO: leave a long code comment explaining  why I chose this threading pub-sub approach
//       see old archived code to see previous attempts

/// Paths are needed as keys. Many such keys are passed around among different threads.
/// This requires many `FPath::clone()`. Instead of clones, pass around a relatively light-weight
/// `usize` as a key.
/// The main processing thread can use a `PathId` key to lookup the `FPath` as-needed.
type PathId = usize;
/// data to initialize a file processing thread
type Thread_Init_Data4 = (
    FPath,
    PathId,
    FileType,
    BlockSz,
    DateTimeL_Opt,
    DateTimeL_Opt,
    FixedOffset,
);
type IsSyslineLast = bool;
/// the data sent from file processing thread to the main processing thread
type Chan_Datum = (SyslineP_Opt, Summary_Opt, IsSyslineLast);
type Map_PathId_Datum = HashMap<PathId, Chan_Datum>;
type Chan_Send_Datum = crossbeam_channel::Sender<Chan_Datum>;
type Chan_Recv_Datum = crossbeam_channel::Receiver<Chan_Datum>;

/// Thread entry point for processing a file
/// this creates `SyslogProcessor` and processes the syslog file `Syslines`.
/// Sends each processed sysline back across channel to main thread.
fn exec_4(chan_send_dt: Chan_Send_Datum, thread_init_data: Thread_Init_Data4) -> thread::ThreadId {
    stack_offset_set(Some(2));
    let (path, pathid, filetype, blocksz, filter_dt_after_opt, filter_dt_before_opt, tz_offset) = thread_init_data;
    debug_eprintln!("{}exec_4({:?})", sn(), path);

    let thread_cur: thread::Thread = thread::current();
    let tid: thread::ThreadId = thread_cur.id();
    let tname: &str = <&str>::clone(&thread_cur.name().unwrap_or(""));

    let mut syslogproc = match SyslogProcessor::new(
        path.clone(),
        filetype,
        blocksz,
        tz_offset,
        filter_dt_after_opt,
        filter_dt_before_opt,
    ) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslogProcessor::new({:?}, {:?}) failed {}", path.as_str(), blocksz, err);
            // TODO: [2022/06/07] send error through channel back to main loop
            return tid;
        }
    };
    debug_eprintln!("{}exec_4 syslogproc {:?}", so(), syslogproc);

    syslogproc.process_stage0_valid_file_check();

    let result = syslogproc.process_stage1_blockzero_analysis();
    match result {
        FileProcessingResult_BlockZero::FILE_ERR_NO_LINES_FOUND => {
            eprintln!("WARNING: no lines found {:?}", path);
            return tid;
        },
        FileProcessingResult_BlockZero::FILE_ERR_NO_SYSLINES_FOUND => {
            eprintln!("WARNING: no syslines found {:?}", path);
            return tid;
        },
        FileProcessingResult_BlockZero::FILE_ERR_DECOMPRESS => {
            eprintln!("WARNING: could not decompress {:?}", path);
            return tid;
        },
        FileProcessingResult_BlockZero::FILE_ERR_WRONG_TYPE => {
            eprintln!("WARNING: bad path {:?}", path);
            return tid;
        },
        FileProcessingResult_BlockZero::FILE_ERR_IO(err) => {
            eprintln!("ERROR: Error {} for {:?}", err, path);
            return tid;
        },
        FileProcessingResult_BlockZero::FILE_OK => {},
        FileProcessingResult_BlockZero::FILE_ERR_EMPTY => {},
        FileProcessingResult_BlockZero::FILE_ERR_NO_SYSLINES_IN_DT_RANGE => {},
    }

    // find first sysline acceptable to the passed filters
    syslogproc.process_stage2_find_dt();

    // sanity check sending of `is_last`
    let mut sent_is_last: bool = false;
    let mut fo1: FileOffset = 0;
    let search_more: bool;
    let result: ResultS4_SyslineFind = syslogproc.find_sysline_between_datetime_filters(0);
    let eof: bool = result.is_eof();
    match result {
        ResultS4_SyslineFind::Found((fo, syslinep)) | ResultS4_SyslineFind::Found_EOF((fo, syslinep)) => {
            fo1 = fo;
            let is_last: IsSyslineLast = syslogproc.is_sysline_last(&syslinep) as IsSyslineLast;
            // XXX: yet another reason to get rid of `Found_EOF` (`Found` and `Done` are enough)
            assert_eq!(eof, is_last, "result.is_eof() {}, syslogproc.is_sysline_last(…) {}; they should match; Sysline @{:?}", eof, is_last, (*syslinep).fileoffset_begin());
            debug_eprintln!("{}{:?}({}): Found, chan_send_dt.send({:p}, None, {});", so(), tid, tname, syslinep, is_last);
            match chan_send_dt.send((Some(syslinep), None, is_last)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: A chan_send_dt.send(…) failed {}", err);
                }
            }
            // XXX: sanity check
            if is_last {
                assert!(!sent_is_last, "is_last {}, yet sent_is_last was also {} (is_last was already sent!)", is_last, sent_is_last);
                sent_is_last = true;
            }
            search_more = !eof;
        },
        ResultS4_SyslineFind::Done => {
            search_more = false;
        },
        ResultS4_SyslineFind::Err(err) => {
            debug_eprintln!("{}{:?}({}): find_sysline_at_datetime_filter returned Err({:?});", so(), tid, tname, err);
            eprintln!("ERROR: SyslogProcessor.process_stage2_find_dt() Error {} for {:?}", err, path);
            search_more = false;
        },
    }

    if !search_more {
        debug_eprintln!("{}{:?}({}): quit searching…", so(), tid, tname);
        let result = syslogproc.process_stage4_summary();
        let summary_opt = Some(syslogproc.summary());
        debug_eprintln!("{}{:?}({}): !search_more chan_send_dt.send((None, {:?}, {}));", so(), tid, tname, summary_opt, false);
        match chan_send_dt.send((None, summary_opt, false)) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: C chan_send_dt.send(…) failed {}", err);
            }
        }
        debug_eprintln!("{}exec_4({:?})", sx(), path);
        return tid;
    }

    // find all proceeding syslines acceptable to the passed filters
    syslogproc.process_stage3_stream_syslines();

    loop {
        let result: ResultS4_SyslineFind = syslogproc.find_sysline(fo1);
        let eof: bool = result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, syslinep)) | ResultS4_SyslineFind::Found_EOF((fo, syslinep)) =>
            {
                let is_last = syslogproc.is_sysline_last(&syslinep);
                // XXX: yet another reason to get rid of `Found_EOF` (`Found` and `Done` are enough)
                assert_eq!(eof, is_last, "from find_sysline({}), ResultS4_SyslineFind.is_eof is {:?} (EOF), yet the returned SyslineP.is_sysline_last is {:?}; they should always agree, for file {:?}", fo, eof, is_last, path);
                debug_eprintln!("{}{:?}({}): chan_send_dt.send(({:p}, None, {}));", so(), tid, tname, syslinep, is_last);
                match chan_send_dt.send((Some(syslinep), None, is_last)) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("ERROR: D chan_send_dt.send(…) failed {}", err);
                    }
                }
                fo1 = fo;
                // XXX: sanity check
                if is_last {
                    assert!(!sent_is_last, "is_last {}, yet sent_is_last was also {} (is_last was already sent!)", is_last, sent_is_last);
                    sent_is_last = true;
                }
                if eof {
                    break;
                }
            }
            ResultS4_SyslineFind::Done => {
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!("{}{:?}({}): find_sysline_at_datetime_filter returned Err({:?});", so(), tid, tname, err);
                eprintln!("ERROR: SyslogProcessor@{:p}.find_sysline({}) {}", &syslogproc, fo1, err);
                break;
            }
        }
    }

    syslogproc.process_stage4_summary();

    let summary = syslogproc.summary();
    debug_eprintln!("{}{:?}({}): last chan_send_dt.send((None, {:?}, {}));", so(), tid, tname, summary, false);
    match chan_send_dt.send((None, Some(summary), false)) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: E chan_send_dt.send(…) failed {}", err);
        }
    }

    debug_eprintln!("{}exec_4({:?})", sx(), path);

// LAST WORKING HERE 2022/06/16 the file processing threads are not ending.
// run with many files in one console window, in a different console window watch `htop`
// in the main thread, try `pool.current_num_threads` to verify if the count is changing.
// might need a test program to figure this out.

    tid
}

/// statistics to print about printing
#[derive(Copy, Clone, Default)]
pub struct SummaryPrinted {
    /// count of bytes printed
    pub bytes: u64,
    /// count of `Lines` printed
    pub lines: u64,
    /// count of `Syslines` printed
    pub syslines: u64,
    /// last datetime printed
    pub dt_first: DateTimeL_Opt,
    pub dt_last: DateTimeL_Opt,
}

impl fmt::Debug for SummaryPrinted {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Summary Printed:")
            .field("bytes", &self.bytes)
            .field("lines", &self.lines)
            .field("syslines", &self.syslines)
            .field("dt_first", &format_args!("{}",
                match self.dt_first {
                        Some(dt) => {
                            dt.to_string()
                        },
                        None => { String::from("None") },
                    }
                )
            )
            .field("dt_last", &format_args!("{}",
                match self.dt_last {
                        Some(dt) => {
                            dt.to_string()
                        },
                        None => { String::from("None") },
                    }
                )
            )
            .finish()
    }
}

impl SummaryPrinted {
    /// mimics debug print but with colorized zero values
    /// only colorize if associated `Summary_Opt` has corresponding
    /// non-zero values
    pub fn print_colored_stderr(&self, color_choice_opt: Option<termcolor::ColorChoice>, summary_opt: &Summary_Opt) {
        let clrerr = Color::Red;
        
        let sumd = Summary::default();
        let sum_: &Summary = match summary_opt {
            Some(s) => s,
            None => {
                &sumd
            }
        };
        eprint!("{{ bytes: ");
        if self.bytes == 0 && sum_.BlockReader_bytes != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(clrerr, color_choice_opt, self.bytes.to_string().as_bytes()) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                },
                _ => {},
            }
        } else {
            eprint!("{}", self.bytes);
        }

        eprint!(", lines: ");
        if self.lines == 0 && sum_.BlockReader_bytes != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(clrerr, color_choice_opt, self.lines.to_string().as_bytes()) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                },
                _ => {},
            }
        } else {
            eprint!("{}", self.lines);
        }

        eprint!(", syslines: ");
        if self.syslines == 0 && sum_.LineReader_lines != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(clrerr, color_choice_opt, self.syslines.to_string().as_bytes()) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                },
                _ => {},
            }
        } else {
            eprint!("{}", self.syslines);
        }

        eprint!(", dt_first: ");
        if self.dt_first.is_none() && sum_.LineReader_lines != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(clrerr, color_choice_opt, "None".as_bytes()) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                },
                _ => {},
            }
        } else {
            eprint!("{:?}", self.dt_first);
        }

        eprint!(", dt_last: ");
        if self.dt_last.is_none() && sum_.LineReader_lines != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(clrerr, color_choice_opt, "None".as_bytes()) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                },
                _ => {},
            }
        } else {
            eprint!("{:?}", self.dt_first);
        }
        eprint!(" }}");
    }
}

type SummaryPrinted_Opt = Option<SummaryPrinted>;

type Map_FPath_SummaryPrint = HashMap::<FPath, SummaryPrinted>;

// TODO: move into impl SummaryPrinted
/// update the passed 
fn summaryprint_update(syslinep: &SyslineP, sp: &mut SummaryPrinted) -> SummaryPrinted {
    sp.syslines += 1;
    sp.lines += (*syslinep).count_lines();
    sp.bytes += (*syslinep).count_bytes();
    if let Some(dt) = (*syslinep).dt {
        match sp.dt_first {
            Some(dt_first) => {
                if dt < dt_first {
                    sp.dt_first = Some(dt);
                };
            },
            None => {
                sp.dt_first = Some(dt);
            },
        };
        match sp.dt_last {
            Some(dt_last) => {
                if dt > dt_last {
                    sp.dt_last = Some(dt);
                };
            },
            None => {
                sp.dt_last = Some(dt);
            },
        };
    };
    *sp
}

// TODO: move into SummaryPrinted
#[inline(always)]
fn summaryprint_map_update(syslinep: &SyslineP, path: &FPath, map_: &mut Map_FPath_SummaryPrint) {
    debug_eprintln!("{}summaryprint_map_update", snx());
    let result = map_.get_mut(path);
    match result {
        Some(sp) => {
            summaryprint_update(syslinep, sp);
        },
        None => {
            let mut sp = SummaryPrinted::default();
            summaryprint_update(syslinep, &mut sp);
            map_.insert(path.clone(), sp);
        }
    };
}

type Map_FPath_Summary = HashMap::<FPath, Summary>;

#[inline(always)]
fn summary_update(path: &FPath, summary: Summary, map_: &mut Map_FPath_Summary) {
    debug_eprintln!("{}summary_update {:?};", snx(), summary);
    if let Some(val) = map_.insert(path.clone(), summary) {
        eprintln!("Error: processing_loop: map_path_summary already contains key {:?} with {:?}, overwritten", path, val);
    };
}

// TODO: use std::path::Path
//// return basename of a file
fn basename(path: &FPath) -> FPath {
    let mut riter = path.rsplit(std::path::MAIN_SEPARATOR);

    FPath::from(riter.next().unwrap_or(""))
}

/// print the various caching statistics
const OPT_SUMMARY_PRINT_CACHE_STATS: bool = true;

/// for printing `--summary` lines, indentation
const SPACING_LEAD: &str = "  ";

/// the main processing loop:
/// 1. creates threads to process each file
/// 2. waits on each thread to receive processed `Sysline` _or_ end
///    a. prints received `Sysline` in order
///    b. goto 2.
/// 3. prints summary (if requestd by user)
///
/// The main thread is the only thread that prints to stdout. In --release
/// builds, other file processing threads may rarely print messages to stderr.
#[allow(clippy::too_many_arguments)]
fn processing_loop(
    mut paths_results: ProcessPathResults,
    blocksz: BlockSz,
    filter_dt_after_opt: &DateTimeL_Opt,
    filter_dt_before_opt: &DateTimeL_Opt,
    tz_offset: FixedOffset,
    color_choice: termcolor::ColorChoice,
    cli_opt_prepend_utc: bool,
    cli_opt_prepend_local: bool,
    cli_opt_prepend_filename: bool,
    cli_opt_prepend_filepath: bool,
    cli_opt_prepend_file_align: bool,
    cli_opt_summary: bool,
) {
    debug_eprintln!("{}processing_loop({:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?})", sn(), paths_results, blocksz, filter_dt_after_opt, filter_dt_before_opt, color_choice, cli_opt_prepend_local, cli_opt_prepend_utc, cli_opt_summary);

    // XXX: sanity check
    assert!(!(cli_opt_prepend_filename && cli_opt_prepend_filepath), "Cannot both cli_opt_prepend_filename && cli_opt_prepend_filepath");

    if paths_results.is_empty() {
        return;
    }

    // separate `ProcessPathResult`s into different storage, valid and invalid
    let mut paths_valid_results: ProcessPathResults = ProcessPathResults::with_capacity(paths_results.len());
    let mut paths_invalid_results: ProcessPathResults = ProcessPathResults::with_capacity(paths_results.len());
    let mut paths_valid: FPaths = FPaths::with_capacity(paths_results.len());
    for processpathresult in paths_results.drain(..)
    {
        match processpathresult {
            ProcessPathResult::FILE_VALID(ref val) =>
            {
                paths_valid.push(val.1.clone());
                paths_valid_results.push(processpathresult);
            }
            _ =>
            {
                paths_valid_results.push(processpathresult);
            },
        };
        
    }
    // XXX: sanity checks
    assert_eq!(paths_valid.len(), paths_valid_results.len(), "mismatching paths_valid {} paths_valid_results {}", paths_valid.len(), paths_valid_results.len());
    assert!(paths_results.is_empty(), "paths_results was not cleared, {} elements remain", paths_results.len());

    let queue_sz_dt: usize = 10;
    let file_count = paths_valid.len();

    // TODO: [2022/06/02] this point needs a PathToPaths thingy that expands user-passed Paths to all possible paths_valid,
    //       e.g.
    //       given a directory path, returns paths_valid of possible syslog files found recursively.
    //       given a symlink, follows the symlink
    //       given a path to a tar file, returns paths_valid of possible syslog files within that .tar file.
    //       given a plain valid file path, just returns that path
    //       would return `Vec<(path: FPath, subpath: FPath, type_: FILE_TYPE, Option<result>: common::FileProcessingResult)>`
    //         where `path` is actual path,
    //         `subpath` is path within a .tar/.zip file
    //         `type_` is enum for `FILE` `FILE_IN_ARCHIVE_TAR`, `FILE_IN_ARCHIVE_TAR_COMPRESS_GZ`, 
    //           `FILE_COMPRESS_GZ`, etc.
    //          `result` of `Some(FileProcessingResult)` if processing has completed or just `None`
    //       (this might be a better place for mimeguess and mimeanalysis?)
    //       Would be best to first implment `FILE`, then `FILE_COMPRESS_GZ`, then `FILE_IN_ARCHIVE_TAR`

    // create PathId->FPath lookup vector (effectively a map)
    // create FPath->PathId lookup map
    let mut map_pathid_path = Vec::<FPath>::with_capacity(file_count);
    let mut map_path_pathid = HashMap::<FPath, PathId>::with_capacity(file_count);
    for (pathid, path) in paths_valid.iter().enumerate() {
        map_pathid_path.insert(pathid as PathId, path.clone());
        map_path_pathid.insert(path.clone(), pathid as PathId);
    }

    // preprint the prepended name or path
    //
    // TODO: [2022/06/15] how to count "column width" of each `char` in the `String`?
    //       SO Q: https://stackoverflow.com/questions/72612510/get-console-width-of-string
    //       A: use https://crates.io/crates/unicode-width
    //
    type PathId_PrependName = HashMap<PathId, String>;
    let mut pathid_to_prependname: PathId_PrependName;
    let mut prependname_width: usize = 0;
    if cli_opt_prepend_filename {
        if cli_opt_prepend_file_align {
            for path in paths_valid.iter() {
                let bname: String = basename(path);
                prependname_width = std::cmp::max(prependname_width, bname.chars().count())
            }
        }
        pathid_to_prependname = PathId_PrependName::with_capacity(file_count);
        for path in paths_valid.iter() {
            let bname: String = basename(path);
            let prepend: String = format!("{0:<1$}:", bname, prependname_width);
            let pathid: &PathId = map_path_pathid.get(path).unwrap();
            pathid_to_prependname.insert(*pathid, prepend);
        }
    } else if cli_opt_prepend_filepath {
        if cli_opt_prepend_file_align {
            for path in paths_valid.iter() {
                prependname_width = std::cmp::max(prependname_width, path.chars().count())
            }
        }
        pathid_to_prependname = PathId_PrependName::with_capacity(file_count);
        for path in paths_valid.iter() {
            let prepend: String = format!("{0:<1$}:", path, prependname_width);
            let pathid: &PathId = map_path_pathid.get(path).unwrap();
            pathid_to_prependname.insert(*pathid, prepend);
        }
    }
    else {
        pathid_to_prependname = PathId_PrependName::with_capacity(0);
    }

    //
    // create a single ThreadPool with one thread per file path, each thread named for the file basename
    //
    let mut paths_valid_basen = paths_valid.clone();
    for p_ in paths_valid_basen.iter_mut() {
        (*p_) = basename(p_);
    }
    debug_eprintln!("{}processing_loop: rayon::ThreadPoolBuilder::new().num_threads({}).build()", so(), file_count);
    let pool: rayon::ThreadPool = rayon::ThreadPoolBuilder::new()
        .num_threads(file_count)
        .thread_name(move |i| paths_valid_basen[i].clone())
        .build()
        .unwrap();

    //
    // prepare per-thread data keyed by `FPath`
    // create necessary channels for each thread
    // launch each thread
    //
    type PathId_ChanRecvDatum<'a> = (PathId, &'a Chan_Recv_Datum);
    type Map_PathId_ChanRecvDatum = HashMap<PathId, Chan_Recv_Datum>;
    let mut map_pathid_recv_dt = Map_PathId_ChanRecvDatum::with_capacity(file_count);
    let mut map_pathid_color = HashMap::<PathId, Color>::with_capacity(file_count);
    let mut map_path_summary = Map_FPath_Summary::with_capacity(file_count);
    let color_datetime: Color = COLOR_DATETIME;

    // initialize processing channels/threads, one per file path
    let (chan_send_1, _chan_recv_1) = std::sync::mpsc::channel();
    for path in paths_valid.iter() {
        let pathid: &PathId = map_path_pathid.get(path).unwrap();
        map_pathid_color.insert(*pathid, color_rand());
    }
    for procespathresult in paths_valid_results.iter() {
        let (filtype, path) = match procespathresult {
            ProcessPathResult::FILE_VALID((f_, p_)) => (f_, p_),
            result => { panic!("bad ProcessPathResult in paths_valid_results: {:?}", result); }
        };
        let pathid: &PathId = map_path_pathid.get(path).unwrap();
        let thread_data: Thread_Init_Data4 = (
            path.clone().to_owned(),
            *pathid,
            *filtype,
            blocksz,
            *filter_dt_after_opt,
            *filter_dt_before_opt,
            tz_offset,
        );
        let (chan_send_dt, chan_recv_dt): (Chan_Send_Datum, Chan_Recv_Datum) = crossbeam_channel::bounded(queue_sz_dt);
        map_pathid_recv_dt.insert(*pathid, chan_recv_dt);
        let chan_send_1_thread = chan_send_1.clone();
        pool.spawn(move || match chan_send_1_thread.send(exec_4(chan_send_dt, thread_data)) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: chan_send_1_thread.send(exec_4(chan_send_dt, thread_data)) failed {}", err);
            }
        });
    }

    // XXX: is this `drop` needed? it was copied from the docs example.
    debug_eprintln!("{}processing_loop: drop({:?});", so(), chan_send_1);
    drop(chan_send_1); // close sender so chan.into_iter.collect does not block

    type Recv_Result4 = std::result::Result<Chan_Datum, crossbeam_channel::RecvError>;

    /// run `.recv` on many Receiver channels simultaneously using `crossbeam_channel::Select`
    /// https://docs.rs/crossbeam-channel/0.5.1/crossbeam_channel/struct.Select.html
    /// 
    /// DONE: TODO: [2022/03/26] to avoid sending a new `FPath` on each channel send, instead have a single
    ///       Map<u32, FPath> that is referred to on "each side". The `u32` is the lightweight key sent
    ///       along the channel.
    ///       This mapping <u32, FPath> could be used for all other maps with keys `FPath`...
    ///       would a global static lookup map make this easier? No need to pass around instances of `Map<u32, FPath>`.
    ///
    #[inline]
    fn recv_many_chan(
        pathid_chans: &Map_PathId_ChanRecvDatum, filter_: &Map_PathId_Datum,
    ) -> (PathId, Recv_Result4) {
        debug_eprintln!("{}processing_loop:recv_many_chan();", sn());
        // "mapping" of index to data; required for various `Select` and `SelectedOperation` procedures,
        // order should match index numeric value returned by `select`
        // TODO: [2022/06] alloc this `imap: Vec` once, outside this function and pass it in as refernce
        //       this function will `clear` and then use it
        let mut imap = Vec::<PathId_ChanRecvDatum>::with_capacity(pathid_chans.len());
        // Build a list of operations
        let mut select = crossbeam_channel::Select::new();
        for pathid_chan in pathid_chans.iter() {
            // if there is already a DateTime "on hand" for the given pathid then
            // skip receiving on the associated channel
            if filter_.contains_key(pathid_chan.0) {
                continue;
            }
            imap.push((*(pathid_chan.0), pathid_chan.1));
            debug_eprintln!("{}processing_loop:recv_many_chan: select.recv({:?});", so(), pathid_chan.1);
            // load `select` with `recv` operations, to be run during later `.select()`
            select.recv(pathid_chan.1);
        }
        assert_gt!(imap.len(), 0, "No channel recv operations to select on.");
        debug_eprintln!("{}processing_loop:recv_many_chan: v: {:?}", so(), imap);
        // Do the `select` operation
        let soper = select.select();
        // get the index of the chosen "winner" of the `select` operation
        let index = soper.index();
        debug_eprintln!("{}processing_loop:recv_many_chan: soper.index() returned {}", so(), index);
        let pathid = imap[index].0;
        let chan = &imap[index].1;
        debug_eprintln!("{}processing_loop:recv_many_chan: soper.recv({:?})", so(), chan);
        // Get the result of the `recv` done during `select`
        let result = soper.recv(chan);
        debug_eprintln!("{}processing_loop:recv_many_chan: soper.recv returned {:?}", so(), result);
        debug_eprintln!("{}processing_loop:recv_many_chan() return ({:?}, {:?})", sx(), pathid, chan);

        (pathid, result)
    }

    //
    // main coordination loop (e.g. "main game loop")
    // process the "receiving sysline" channels from the running threads
    // print the soonest available sysline
    //
    // TODO: [2022/03/24] change `map_pathid_datum` to `HashMap<FPath, (SylineP, is_last)>` (`map_path_slp`);
    //       currently it's confusing that there is a special handler for `Summary` (`map_path_summary`),
    //       but not an equivalent `map_path_slp`.
    //       In other words, break apart the passed `Chan_Datum` to the more specific maps.
    //
    let mut map_pathid_datum = Map_PathId_Datum::with_capacity(file_count);
    let mut map_path_sumpr = Map_FPath_SummaryPrint::with_capacity(file_count);
    // crude debugging stats
    let mut _count_recv_ok: usize = 0;
    let mut _count_recv_di: usize = 0;
    let mut sp_total: SummaryPrinted = SummaryPrinted::default();

    let color_default = Color::White;

    let tz_utc = Utc::from_offset(&Utc);
    let tz_local = Local.timestamp(0, 0).timezone();

    // XXX: workaround for missing Default for `&String`
    let string_default: &String = &String::from("");

    // channels that should be disconnected per loop iteration
    let mut disconnect = Vec::<PathId>::with_capacity(file_count);

    // any prepended writes to do?
    let do_prepend: bool = cli_opt_prepend_filename
        || cli_opt_prepend_filepath
        || cli_opt_prepend_utc
        || cli_opt_prepend_local;

    // main thread "game loop"
    loop {
        disconnect.clear();

        if cfg!(debug_assertions) {
            debug_eprintln!("{}processing_loop: pool.current_num_threads {}", so(), pool.current_num_threads(),);
            for (pathid, recv) in map_pathid_recv_dt.iter() {
                let path: &FPath = map_pathid_path.get(*pathid).unwrap();
                debug_eprintln!("{}processing_loop: thread {} {} messages {}", so(), path, pathid, recv.len());
            }
        }

        // if there is a DateTime available for *every* FPath channel (one channel is one FPath)
        // then those datetimes can all be compared. The sysline with the soonest DateTime is
        // selected then printed.
        if map_pathid_recv_dt.len() == map_pathid_datum.len() {
            // debug prints
            if cfg!(debug_assertions) {
                for (i, (k, v)) in map_pathid_recv_dt.iter().enumerate() {
                    debug_eprintln!("{} A1 map_pathid_recv_dt[{:?}] = {:?}", i, k, v);
                }
                for (i, (k, v)) in map_pathid_datum.iter().enumerate() {
                    debug_eprintln!("{} A1 map_pathid_datum[{:?}] = {:?}", i, k, v);
                }
            }

            // (path, channel data) for the sysline with soonest datetime ("minimum" datetime)
            //
            // here is part of the "sorting" of syslines process by datetime.
            // In case of tie datetime values, the tie-breaker will be order of `BTreeMap::iter_mut` which
            // iterates in order of key sort. https://doc.rust-lang.org/stable/std/collections/struct.BTreeMap.html#method.iter_mut
            //
            // XXX: assume `unwrap` will never fail
            //
            // XXX: my small investigation into `min`, `max`, `min_by`, `max_by`
            //      https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=a6d307619a7797b97ef6cfc1635c3d33
            //
            let pathid: &PathId;
            let chan_datum: &mut Chan_Datum;
            (pathid, chan_datum) = match map_pathid_datum.iter_mut().min_by(
                |x, y|
                    x.1.0.as_ref().unwrap().dt.cmp(&(y.1.0.as_ref().unwrap().dt))
            ) {
                Some(val) => (
                    val.0, val.1
                ),
                None => {
                    eprintln!("ERROR map_pathid_datum.iter_mut().min_by() returned None");
                    // XXX: not sure what else to do here
                    continue;
                }
            };

            if let Some(summary) = chan_datum.1.clone() {
                debug_eprintln!("{}processing_loop: A2 chan_datum has Summary, PathId: {:?}", so(), pathid);
                assert!(chan_datum.0.is_none(), "Chan_Datum Some(Summary) and Some(SyslineP); should only have one Some(). PathId: {:?}", pathid);
                if cli_opt_summary {
                    let path: &FPath = map_pathid_path.get(*pathid).unwrap();
                    summary_update(path, summary, &mut map_path_summary);
                }
                debug_eprintln!("{}processing_loop: A2 will disconnect channel {:?}", so(), pathid);
                // receiving a Summary implies the last data was sent on the channel
                disconnect.push(*pathid);
            } else {
                // is last sysline of the file?
                let is_last: bool = chan_datum.2;
                // Sysline of interest
                let syslinep: &SyslineP = chan_datum.0.as_ref().unwrap();
                // color for printing
                let clr: &Color = map_pathid_color.get(pathid).unwrap_or(&color_default);
                // print the sysline line-by-line!
                debug_eprintln!("{}processing_loop: A3 printing SyslineP@{:p} @[{}, {}] PathId: {:?}", so(), syslinep, syslinep.fileoffset_begin(), syslinep.fileoffset_end(), pathid);
                if do_prepend {
                    // print one `Line` from `Sysline` at a time
                    // so each `Line` is prepended as requested
                    let line_count: usize = (*syslinep).count_lines() as usize;
                    let mut line_at: usize = 0;
                    while line_at < line_count {
                        if cli_opt_prepend_filename || cli_opt_prepend_filepath {
                            let path: &FPath = map_pathid_path.get(*pathid).unwrap();
                            let prepend: &String = pathid_to_prependname
                                .get(pathid)
                                .unwrap_or(string_default);
                            write_stdout(prepend.as_bytes());
                        }
                        if cli_opt_prepend_utc || cli_opt_prepend_local {
                            #[allow(clippy::single_match)]
                            match (*syslinep).dt {
                                Some(dt) => {
                                    #[allow(clippy::needless_late_init)]
                                    let fmt_;
                                    if cli_opt_prepend_utc {
                                        let dt_ = dt.with_timezone(&tz_utc);
                                        fmt_ = dt_.format(CLI_OPT_PREPEND_FMT);
                                    } else { // cli_opt_prepend_local
                                        let dt_ = dt.with_timezone(&tz_local);
                                        fmt_ = dt_.format(CLI_OPT_PREPEND_FMT);
                                    }
                                    write_stdout(fmt_.to_string().as_bytes());
                                },
                                _ => {},
                            }
                        }
                        match (*syslinep).print_color(Some(line_at), Some(color_choice), *clr, color_datetime) {
                            Ok(_) => {},
                            Err(_err) => {
                                eprintln!("ERROR: failed to print; TODO abandon processing for PathId {:?}", pathid);
                                // TODO: 2022/04/09 remove this `pathid` from maps and queues, shutdown it's thread
                            }
                        }
                        line_at += 1;
                    }
                } else  {
                    // no prepends request so print all `Line`s within one call
                    match (*syslinep).print_color(None, Some(color_choice), *clr, color_datetime) {
                        Ok(_) => {},
                        Err(_err) => {
                            eprintln!("ERROR: failed to print; TODO abandon processing for PathId {:?}", pathid);
                            // TODO: 2022/04/09 remove this `pathid` from maps and queues, shutdown it's thread
                        }
                    }
                }
                if is_last {
                    write_stdout(&NLu8a);
                    sp_total.bytes += 1;
                }
                if cli_opt_summary {
                    let path: &FPath = map_pathid_path.get(*pathid).unwrap();
                    summaryprint_map_update(syslinep, path, &mut map_path_sumpr);
                    summaryprint_update(syslinep, &mut sp_total);
                }
            }
            // create a copy of the borrowed key `pathid`, this avoids rustc error:
            //     cannot borrow `map_pathid_datum` as mutable more than once at a time
            let pathid_: PathId = *pathid;
            map_pathid_datum.remove(&pathid_);
        } else {
            // else waiting on a (datetime, syslinep) from at least one channel
            // so call `recv_many_chan` and store the data
            debug_eprintln!("{}processing_loop: B recv_many_chan(map_pathid_recv_dt: {:?}, map_pathid_datum: {:?})", so(), map_pathid_recv_dt, map_pathid_datum);
            let (pathid, result1) = recv_many_chan(&map_pathid_recv_dt, &map_pathid_datum);
            match result1 {
                Ok(chan_datum) => {
                    debug_eprintln!("{}processing_loop: B crossbeam_channel::Found for PathId {:?};", so(), pathid);
                    if let Some(summary) = chan_datum.1 {
                        let path: &FPath = map_pathid_path.get(pathid).unwrap();
                        debug_eprintln!("{}processing_loop: B chan_datum has Summary {:?}", so(), path);
                        assert!(chan_datum.0.is_none(), "Chan_Datum Some(Summary) and Some(SyslineP); should only have one Some(). PathId {:?}", pathid);
                        summary_update(&path, summary, &mut map_path_summary);
                        debug_eprintln!("{}processing_loop: B will disconnect channel {:?}", so(), path);
                        // receiving a Summary must be the last data sent on the channel
                        disconnect.push(pathid);
                    } else {
                        assert!(chan_datum.0.is_some(), "Chan_Datum None(Summary) and None(SyslineP); should have one Some(). PathId {:?}", pathid);
                        map_pathid_datum.insert(pathid, chan_datum);
                    }
                    _count_recv_ok += 1;
                }
                Err(crossbeam_channel::RecvError) => {
                    debug_eprintln!("{}processing_loop: B crossbeam_channel::RecvError, will disconnect channel for PathId {:?};", so(), pathid);
                    // this channel was closed by the sender
                    disconnect.push(pathid);
                    _count_recv_di += 1;
                }
            }
        }
        // remove channels (and keys) that have been disconnected
        for pathid in disconnect.iter() {
            debug_eprintln!("{}processing_loop: C map_pathid_recv_dt.remove({:?});", so(), pathid);
            map_pathid_recv_dt.remove(pathid);
            debug_eprintln!("{}processing_loop: C pathid_to_prependname.remove({:?});", so(), pathid);
            pathid_to_prependname.remove(pathid);
        }
        // are there any channels to receive from?
        if map_pathid_recv_dt.is_empty() {
            debug_eprintln!("{}processing_loop: D map_pathid_recv_dt.is_empty(); no more channels to receive from!", so());
            break;
        }
        debug_eprintln!("{}processing_loop: D map_pathid_recv_dt: {:?}", so(), map_pathid_recv_dt);
        debug_eprintln!("{}processing_loop: D map_pathid_datum: {:?}", so(), map_pathid_datum);
    } // end loop

    // Getting here means main program processing has completed.
    // Now to print the `--summary` (if it was requested).

    /// helper function to print the `summary.patterns` Vec (requires it's own line)
    pub fn patterns_dbg(summary: &Summary) -> String {
        // `cap` is a rough capacity estimation
        let cap: usize = summary.SyslineReader_patterns.len() * 150;
        let mut out: String = String::with_capacity(cap);
        for patt in summary.SyslineReader_patterns.iter() {
            // XXX: magic knowledge of blank prepend
            let a = format!("                   {:?}", patt);
            out.push_str(a.as_ref());
        }

        out
    }

    /// helper function to print the filepath name (one line)
    pub fn print_filepath(path: &FPath, color: &Color, color_choice: &termcolor::ColorChoice) {
        eprint!("File: ");
        match print_colored_stderr(*color, Some(*color_choice), path.as_bytes()) {
            Ok(()) => {},
            Err(err) => {
                eprintln!("ERROR: {:?}", err);
            }
        };
        eprintln!();
    }

    /// helper function to print the &Summary_Opt (one line)
    pub fn print_summary_opt_processed(summary_opt: &Summary_Opt) {
        match summary_opt {
            Some(summary) => {
                eprintln!("{}Summary Processed:{:?}", SPACING_LEAD, summary);
                let out = patterns_dbg(summary);
                eprintln!("{}{}", SPACING_LEAD, out);
            },
            None => {
                // TODO: [2022/06/07] print filesz
                eprintln!("{}Summary Processed: None", SPACING_LEAD);
            }
        }
    }

    /// helper function to print the &SummaryPrinted_Opt (one line)
    pub fn print_summary_opt_printed(
        summary_print_opt: &SummaryPrinted_Opt,
        summary_opt: &Summary_Opt,
        color_choice: &termcolor::ColorChoice,
    ) {
        match summary_print_opt {
            Some(summary_print) => {
                eprint!("{}Summary Printed  : ", SPACING_LEAD);
                summary_print.print_colored_stderr(Some(*color_choice), summary_opt);
            },
            None => {
                eprint!("{}Summary Printed  : ", SPACING_LEAD);
                SummaryPrinted::default().print_colored_stderr(Some(*color_choice), summary_opt);
            }
        }
        eprintln!();
    }

    /// helper function to print the various caching statistics (several lines)
    pub fn print_cache_stats(summary_opt: Summary_Opt) {
        if summary_opt.is_none() {
            return;
        }

        fn ratio64(a: &u64, b: &u64) -> f64 {
            if b == &0 {
                return 0.0;
            }
            (*a as f64) / (*b as f64)
        }

        fn ratio32(a: &u32, b: &u32) -> f64 {
            ratio64(&(*a as u64), &(*b as u64))
        }

        // SyslineReader
        let summary: &Summary = &summary_opt.unwrap_or_default();
        // SyslineReader::_parse_datetime_in_line_lru_cache
        let mut ratio = ratio64(
            &summary.SyslineReader_parse_datetime_in_line_lru_cache_hit,
            &summary.SyslineReader_parse_datetime_in_line_lru_cache_miss
        );
        eprintln!(
            "{}caching: SyslineReader::parse_datetime_in_line_lru_cache: hit {:2}, miss {:2}, ratio: {:1.2}, put {:2}",
            SPACING_LEAD,
            summary.SyslineReader_parse_datetime_in_line_lru_cache_hit,
            summary.SyslineReader_parse_datetime_in_line_lru_cache_miss,
            ratio,
            summary.SyslineReader_parse_datetime_in_line_lru_cache_put,
        );
        // SyslineReader::_syslines_by_range
        ratio = ratio64(
            &summary.SyslineReader_syslines_by_range_hit,
            &summary.SyslineReader_syslines_by_range_miss
        );
        eprintln!(
            "{}caching: SyslineReader::syslines_by_range_map           : hit {:2}, miss {:2}, ratio: {:1.2}, insert {:2}",
            SPACING_LEAD,
            summary.SyslineReader_syslines_by_range_hit,
            summary.SyslineReader_syslines_by_range_miss,
            ratio,
            summary.SyslineReader_syslines_by_range_insert,
        );
        // SyslineReader::_find_sysline_lru_cache
        ratio = ratio64(
            &summary.SyslineReader_find_sysline_lru_cache_hit,
            &summary.SyslineReader_find_sysline_lru_cache_miss
        );
        eprintln!(
            "{}caching: SyslineReader::find_sysline_lru_cache          : hit {:2}, miss {:2}, ratio: {:1.2}, put {:2}",
            SPACING_LEAD,
            summary.SyslineReader_find_sysline_lru_cache_hit,
            summary.SyslineReader_find_sysline_lru_cache_miss,
            ratio,
            summary.SyslineReader_find_sysline_lru_cache_put,
        );
        // LineReader::_find_line_lru_cache
        ratio = ratio64(
            &summary.LineReader_find_line_lru_cache_hit,
            &summary.LineReader_find_line_lru_cache_miss
        );
        eprintln!(
            "{}caching: LineReader::find_line_lru_cache: hit {:2}, miss: {:2}, ratio: {:1.2}, put {:2}",
            SPACING_LEAD,
            summary.LineReader_find_line_lru_cache_hit,
            summary.LineReader_find_line_lru_cache_miss,
            ratio,
            summary.LineReader_find_line_lru_cache_put,
        );
        // BlockReader::_read_blocks
        ratio = ratio32(
            &summary.BlockReader_read_blocks_hit,
            &summary.BlockReader_read_blocks_miss
        );
        eprintln!(
            "{}caching: BlockReader::read_block_blocks   : hit {:2}, miss {:2}, ratio: {:1.2}, insert {:2}",
            SPACING_LEAD,
            summary.BlockReader_read_blocks_hit,
            summary.BlockReader_read_blocks_miss,
            ratio,
            summary.BlockReader_read_blocks_insert,
        );
        // BlockReader::_read_blocks_cache
        ratio = ratio32(
            &summary.BlockReader_read_block_lru_cache_hit,
            &summary.BlockReader_read_block_lru_cache_miss
        );
        eprintln!(
            "{}caching: BlockReader::read_block_lru_cache: hit {:2}, miss {:2}, ratio: {:1.2}, put {:2}",
            SPACING_LEAD,
            summary.BlockReader_read_block_lru_cache_hit,
            summary.BlockReader_read_block_lru_cache_miss,
            ratio,
            summary.BlockReader_read_block_lru_cache_put,
        );
    }

    if cli_opt_summary {
        eprintln!("\nSummary:");
        for path in paths_valid.iter() {
            let pathid: &PathId = map_path_pathid.get(path).unwrap();
            let color_ = map_pathid_color.get(pathid).unwrap_or(&color_default);
            print_filepath(path, color_, &color_choice);

            let summary_opt: Summary_Opt = map_path_summary.remove(path);
            print_summary_opt_processed(&summary_opt);

            let summary_print_opt: SummaryPrinted_Opt = map_path_sumpr.remove(path);
            print_summary_opt_printed(&summary_print_opt, &summary_opt, &color_choice);

            if OPT_SUMMARY_PRINT_CACHE_STATS {
                print_cache_stats(summary_opt);
            }
        }
        eprintln!("{:?}", sp_total);
        eprintln!("channel recv ok {}, channel recv err {}", _count_recv_ok, _count_recv_di);
    }

    debug_eprintln!("{}processing_loop: E _count_recv_ok {:?} _count_recv_di {:?}", so(), _count_recv_ok, _count_recv_di);
    debug_eprintln!("{}processing_loop()", sx());
}
