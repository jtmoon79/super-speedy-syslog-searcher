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
      This is a resource intensive hack that *would* work.

TODO: 2022/03/31 add option --assume-TZ-offset that allows passing TZ offset that will be assumed for syslog
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

DONE: TODO: 2022/04/07 need to handle formats with explicit timezone offset.
      see example `access.log`

DONE: TODO: 2022/04/09 in `find_datetime_in_line`, the `slice_.contains(&b'1')`
      use much runtime. Would doing this search manually be faster?
      Create a benchmark to find out.

*/

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// uses
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::collections::{
    //BTreeMap,
    HashMap
};
use std::fmt;
//use std::fs::{File, Metadata, OpenOptions};
//use std::io;
//use std::io::prelude::Read;
//use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};
//use std::path::Path;
//use std::ops::RangeInclusive;
use std::str;
//use std::str::FromStr;  // attaches `from_str` to various built-in types
//use std::sync::Arc;
use std::thread;

//extern crate arrayref;
//use arrayref::array_ref;

extern crate atty;

extern crate backtrace;

extern crate chain_cmp;
use chain_cmp::chmp;

extern crate clap;
use clap::Parser;
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
    //Bytes,
    //NLu8,
    NLu8a,
};

mod dbgpr;
use dbgpr::stack::{so, sn, sx, snx, stack_offset_set};
use crate::dbgpr::printers::{
    print_colored_stdout,
    print_colored_stderr,
    write_stdout,
    Color,
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
use Readers::datetime::{
    DateTimeL_Opt,
    DateTime_Parse_Data_str,
    //Local,
    //Utc,
    //Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    DateTime_Parse_Data_str_to_DateTime_Parse_Data,
    str_datetime,
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// misc. globals
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// global constants

/// global test initializer to run once
/// see https://stackoverflow.com/a/58006287/471376
//static _Test_Init_Once: Once = Once::new();

static COLOR_DATETIME: Color = Color::Green;

static COLORS_TEXT: [Color; 29] = [
    Color::Yellow,
    Color::Cyan,
    Color::Red,
    Color::Magenta,
    // decent reference https://www.rapidtables.com/web/color/RGB_Color.html
    // XXX: colors with low pixel values are difficult to see on dark console backgrounds
    //      recommend at least one pixel value of 102 or greater
    Color::Rgb(102, 0, 0),
    Color::Rgb(102, 102, 0),
    Color::Rgb(127, 0, 0),
    Color::Rgb(0, 0, 127),
    Color::Rgb(127, 0, 127),
    Color::Rgb(153, 76, 0),
    Color::Rgb(153, 153, 0),
    Color::Rgb(0, 153, 153),
    Color::Rgb(127, 127, 127),
    Color::Rgb(127, 153, 153),
    Color::Rgb(127, 153, 127),
    Color::Rgb(127, 127, 230),
    Color::Rgb(127, 230, 127),
    Color::Rgb(230, 127, 127),
    Color::Rgb(127, 230, 230),
    Color::Rgb(230, 230, 127),
    Color::Rgb(230, 127, 230),
    Color::Rgb(230, 230, 230),
    Color::Rgb(153, 153, 255),
    Color::Rgb(153, 255, 153),
    Color::Rgb(255, 153, 153),
    Color::Rgb(153, 255, 255),
    Color::Rgb(255, 255, 153),
    Color::Rgb(255, 153, 255),
    Color::Rgb(255, 255, 255),
];

/// "cached" indexing value for `color_rand`
/// not thread aware
#[allow(non_upper_case_globals)]
static mut _color_at: usize = 0;

/// return a random color from `COLORS`
fn color_rand() -> Color {
    let ci: usize;
    unsafe {
        _color_at += 1;
        if _color_at == COLORS_TEXT.len() {
            _color_at = 0;
        }
        ci = _color_at;
    }

    COLORS_TEXT[ci]
}


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// command-line parsing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const CLI_DT_FILTER_PATTERN1: &DateTime_Parse_Data_str = &("%Y%m%dT%H%M%S", true, false, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN2: &DateTime_Parse_Data_str = &("%Y-%m-%d %H:%M:%S", true, false, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN3: &DateTime_Parse_Data_str = &("%Y-%m-%dT%H:%M:%S", true, false, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN4: &DateTime_Parse_Data_str = &("%Y%m%dT%H%M%S%z", true, true, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN5: &DateTime_Parse_Data_str = &("%Y-%m-%d %H:%M:%S %z", true, true, 0, 0, 0, 0);
const CLI_DT_FILTER_PATTERN6: &DateTime_Parse_Data_str = &("%Y-%m-%dT%H:%M:%S %z", true, true, 0, 0, 0, 0);
const CLI_FILTER_PATTERNS_COUNT: usize = 6;
const CLI_FILTER_PATTERNS: [&DateTime_Parse_Data_str; CLI_FILTER_PATTERNS_COUNT] = [
    CLI_DT_FILTER_PATTERN1,
    CLI_DT_FILTER_PATTERN2,
    CLI_DT_FILTER_PATTERN3,
    CLI_DT_FILTER_PATTERN4,
    CLI_DT_FILTER_PATTERN5,
    CLI_DT_FILTER_PATTERN6,
];
const CLI_HELP_AFTER: &str = "\
DateTime Filter patterns may be:
    '%Y%m%dT%H%M%S'
    '%Y-%m-%d %H:%M:%S'
    '%Y-%m-%dT%H:%M:%S'
    '%Y%m%dT%H%M%S%z'
    '%Y-%m-%d %H:%M:%S %z'
    '%Y-%m-%dT%H:%M:%S %z'

Without a timezone offset (%z), the datetime is presumed to be the system timezone.

DateTime Filter formatting is described at
https://docs.rs/chrono/latest/chrono/format/strftime/

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
    /// Path(s) of syslog files.
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
        group = "prepend",
    )]
    prepend_utc: bool,

    /// Prepend DateTime in the Local Timezone for every sysline.
    #[clap(
        short = 'l',
        long = "prepend-local",
        group = "prepend",
    )]
    prepend_local: bool,

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
fn cli_validate_blocksz(blockszs: &str) -> std::result::Result<(), String> {
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

/// process passed CLI arguments into types
/// this function will `std::process::exit` if there is an `Err`
fn cli_process_args() -> (FPaths, BlockSz, DateTimeL_Opt, DateTimeL_Opt, FixedOffset, bool, bool, bool) {
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

    fn process_dt(dts: Option<String>, tz_offset: &FixedOffset) -> DateTimeL_Opt {
        // parse datetime filters after
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

    (fpaths, blocksz, filter_dt_after, filter_dt_before, tz_offset, args.prepend_utc, args.prepend_local, args.summary)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// command-line parsing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// process user-passed command-line arguments
pub fn main() -> std::result::Result<(), chrono::format::ParseError> {
    // set once, use `stackdepth_main` to access `_STACKDEPTH_MAIN`
    if cfg!(debug_assertions) {
        stack_offset_set(Some(-2));
    }
    debug_eprintln!("{}main()", sn());
    let (
        fpaths,
        blocksz,
        filter_dt_after,
        filter_dt_before,
        tz_offset,
        cli_opt_prepend_utc,
        cli_opt_prepend_local,
        cli_opt_summary,
    ) = cli_process_args();

    // TODO: for each fpath, remove non-existent files non-readable files, walk directories and expand to fpaths

    run_4(
        fpaths,
        blocksz,
        &filter_dt_after,
        &filter_dt_before,
        tz_offset,
        cli_opt_prepend_utc,
        cli_opt_prepend_local,
        cli_opt_summary
    );

    debug_eprintln!("{}main()", sx());

    Ok(())
}

// -------------------------------------------------------------------------------------------------
// threading try #4
// -------------------------------------------------------------------------------------------------

// TODO: leave a long code comment explaining  why I chose this threading pub-sub approach
//       see old archived code to see previous attempts

type Thread_Init_Data4 = (FPath, BlockSz, DateTimeL_Opt, DateTimeL_Opt, FixedOffset);
type IsSyslineLast = bool;
type Chan_Datum = (SyslineP_Opt, Summary_Opt, IsSyslineLast);
type Chan_Send_Datum = crossbeam_channel::Sender<Chan_Datum>;
type Chan_Recv_Datum = crossbeam_channel::Receiver<Chan_Datum>;
type Map_FPath_Datum = HashMap<FPath, Chan_Datum>;

/// thread entry point for processing a file
/// this creates `SyslineReader` and processes the `Syslines`
fn exec_4(chan_send_dt: Chan_Send_Datum, thread_init_data: Thread_Init_Data4) -> thread::ThreadId {
    stack_offset_set(Some(2));
    let (path, blocksz, filter_dt_after_opt, filter_dt_before_opt, tz_offset) = thread_init_data;
    debug_eprintln!("{}exec_4({:?})", sn(), path);
    let thread_cur: thread::Thread = thread::current();
    let tid: thread::ThreadId = thread_cur.id();
    let tname: &str = <&str>::clone(&thread_cur.name().unwrap_or(""));

    let mut slr = match SyslineReader::new(&path, blocksz, tz_offset) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({:?}, {}) failed {}", path, blocksz, err);
            return tid;
        }
    };

    // find first sysline acceptable to the passed filters
    let mut fo1: FileOffset = 0;
    let mut search_more = true;
    let result = slr.find_sysline_between_datetime_filters(fo1, &filter_dt_after_opt, &filter_dt_before_opt);
    match result {
        ResultS4_SyslineFind::Found((fo, slp)) => {
            fo1 = fo;
            let is_last = slr.is_sysline_last(&slp);
            debug_eprintln!("{}{:?}({}): Found, chan_send_dt.send({:p}, None, {});", so(), tid, tname, slp, is_last);
            match chan_send_dt.send((Some(slp), None, is_last)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: A chan_send_dt.send(…) failed {}", err);
                }
            }
        }
        ResultS4_SyslineFind::Found_EOF((_, slp)) => {
            let is_last = slr.is_sysline_last(&slp);
            debug_eprintln!("{}{:?}({}): Found_EOF, chan_send_dt.send(({:p}, None, {}));", so(), tid, tname, slp, is_last);
            match chan_send_dt.send((Some(slp), None, is_last)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: B chan_send_dt.send(…) failed {}", err);
                }
            }
            search_more = false;
        }
        ResultS4_SyslineFind::Done => {
            search_more = false;
        }
        ResultS4_SyslineFind::Err(err) => {
            debug_eprintln!("{}{:?}({}): find_sysline_at_datetime_filter returned Err({:?});", so(), tid, tname, err);
            eprintln!("ERROR: SyslineReader@{:p}.find_sysline_at_datetime_filter({}, {:?}) {}", &slr, fo1, filter_dt_after_opt, err);
            search_more = false;
        }
    }
    if !search_more {
        let summary_opt = Some(slr.summary());
        let is_last = false;  // XXX: is_last does not matter here
        debug_eprintln!("{}{:?}({}): !search_more chan_send_dt.send((None, {:?}, {}));", so(), tid, tname, summary_opt, is_last);
        match chan_send_dt.send((None, summary_opt, is_last)) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: C chan_send_dt.send(…) failed {}", err);
            }
        }
        debug_eprintln!("{}exec_4({:?})", sx(), path);
        return tid;
    }
    // find all proceeding syslines acceptable to the passed filters
    let mut fo2: FileOffset = fo1;
    loop {
        let result = slr.find_sysline(fo2);
        let eof = result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                fo2 = fo;
                match SyslineReader::sysline_pass_filters(&slp, &filter_dt_after_opt, &filter_dt_before_opt) {
                    Result_Filter_DateTime2::InRange => {
                        let is_last = slr.is_sysline_last(&slp);
                        assert_eq!(eof, is_last, "from find_sysline, ResultS4_SyslineFind.is_eof is {:?} (EOF), yet the returned SyslineP.is_sysline_last is {:?}; they should always agree", eof, is_last);
                        debug_eprintln!("{}{:?}({}): InRange, chan_send_dt.send(({:p}, None, {}));", so(), tid, tname, slp, is_last);
                        match chan_send_dt.send((Some(slp), None, is_last)) {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("ERROR: D chan_send_dt.send(…) failed {}", err);
                            }
                        }
                    }
                    Result_Filter_DateTime2::BeforeRange => {
                        debug_eprintln!("{}{:?}{} ERROR: Sysline out of order: {:?}", so(), tid, tname, (*slp).to_String_noraw());
                        eprintln!("ERROR: Encountered a Sysline that is out of order; will abandon processing of file {:?}", path);
                        break;
                    }
                    Result_Filter_DateTime2::AfterRange => {
                        break;
                    }
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
                eprintln!("ERROR: SyslineReader@{:p}.find_sysline({}) {}", &slr, fo2, err);
                break;
            }
        }
    }

    let summary = slr.summary();
    let is_last = false;  // XXX: is_last should not matter here
    debug_eprintln!("{}{:?}({}): last chan_send_dt.send((None, {:?}, {}));", so(), tid, tname, summary, is_last);
    match chan_send_dt.send((None, Some(summary), is_last)) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: E chan_send_dt.send(…) failed {}", err);
        }
    }

    debug_eprintln!("{}exec_4({:?})", sx(), path);
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
    pub fn print_colored_stderr(&self, summary_opt: &Summary_Opt) {
        let clrerr = Color::Red;
        
        let sumd = Summary::default();
        let sum_: &Summary = match summary_opt {
            Some(s) => s,
            None => {
                &sumd
            }
        };
        eprint!("{{ bytes: ");
        if self.bytes == 0 && sum_.bytes != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(clrerr, self.bytes.to_string().as_bytes()) {
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
        if self.lines == 0 && sum_.bytes != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(clrerr, self.lines.to_string().as_bytes()) {
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
        if self.syslines == 0 && sum_.lines != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(clrerr, self.syslines.to_string().as_bytes()) {
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
        if self.dt_first.is_none() && sum_.lines != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(clrerr, "None".as_bytes()) {
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
        if self.dt_last.is_none() && sum_.lines != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(clrerr, "None".as_bytes()) {
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
fn summaryprint_update(slp: &SyslineP, sp: &mut SummaryPrinted) -> SummaryPrinted {
    sp.syslines += 1;
    sp.lines += slp.count();
    sp.bytes += slp.count_bytes();
    if let Some(dt) = slp.dt {
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
fn summaryprint_map_update(slp: &SyslineP, fpath_: &FPath, map_: &mut Map_FPath_SummaryPrint) {
    debug_eprintln!("{}summaryprint_map_update", snx());
    let result = map_.get_mut(fpath_);
    match result {
        Some(sp) => {
            summaryprint_update(slp, sp);
        },
        None => {
            let mut sp = SummaryPrinted::default();
            summaryprint_update(slp, &mut sp);
            map_.insert(fpath_.clone(), sp);
        }
    };
}

type Map_FPath_Summary = HashMap::<FPath, Summary>;

#[inline(always)]
fn summary_update(fpath_: &FPath, summary: Summary, map_: &mut Map_FPath_Summary) {
    debug_eprintln!("{}summary_update {:?};", snx(), summary);
    if let Some(val) = map_.insert(fpath_.clone(), summary) {
        //debug_eprintln!("{}run_4: Error: map_path_summary already contains key {:?} with {:?}", so(), fpath_min, val);
        eprintln!("Error: run4: map_path_summary already contains key {:?} with {:?}, overwritten", fpath_, val);
    };
}

// TODO: use std::path::Path
fn basename(path: &FPath) -> FPath {
    let mut riter = path.rsplit(std::path::MAIN_SEPARATOR);

    FPath::from(riter.next().unwrap_or(""))
}

const OPT_SUMMARY_DEBUG_STATS: bool = true;

#[allow(clippy::too_many_arguments)]
fn run_4(
    paths: FPaths,
    blocksz: BlockSz,
    filter_dt_after_opt: &DateTimeL_Opt,
    filter_dt_before_opt: &DateTimeL_Opt,
    tz_offset: FixedOffset,
    cli_opt_prepend_utc: bool,
    cli_opt_prepend_local: bool,
    cli_opt_summary: bool,
) {
    debug_eprintln!("{}run_4({:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?})", sn(), paths, blocksz, filter_dt_after_opt, filter_dt_before_opt, cli_opt_prepend_local, cli_opt_prepend_utc, cli_opt_summary);

    if paths.is_empty() {
        return;
    }

    let queue_sz_dt: usize = 10;
    let file_count = paths.len();

    //
    // create a single ThreadPool with one thread per file path, each thread named for the file basename
    //
    let mut paths_basen = paths.clone();
    for p_ in paths_basen.iter_mut() {
        (*p_) = basename(p_);
    }
    debug_eprintln!("{}run_4: rayon::ThreadPoolBuilder::new().num_threads({}).build()", so(), file_count);
    let pool = rayon::ThreadPoolBuilder::new().
        num_threads(file_count).
        thread_name(move |i| paths_basen[i].clone()).
        build().
        unwrap();

    //
    // prepare per-thread data keyed by `FPath`
    // create necessary channels for each thread
    // launch each thread
    //
    type FPath_ChanRecvDatum<'a> = (&'a FPath, &'a Chan_Recv_Datum);
    type Map_FPath_ChanRecvDatum<'a> = HashMap<&'a FPath, Chan_Recv_Datum>;
    let mut map_path_recv_dt = Map_FPath_ChanRecvDatum::with_capacity(file_count);
    let mut map_path_color = HashMap::<&FPath, Color>::with_capacity(file_count);
    let mut map_path_summary = Map_FPath_Summary::with_capacity(file_count);
    let color_datetime: Color = COLOR_DATETIME;

    // initialize
    let (chan_send_1, _chan_recv_1) = std::sync::mpsc::channel();
    for fpath in paths.iter() {
        map_path_color.insert(fpath, color_rand());
    }
    for fpath in paths.iter() {
        let thread_data: Thread_Init_Data4 =
            (fpath.clone().to_owned(), blocksz, *filter_dt_after_opt, *filter_dt_before_opt, tz_offset);
        let (chan_send_dt, chan_recv_dt): (Chan_Send_Datum, Chan_Recv_Datum) = crossbeam_channel::bounded(queue_sz_dt);
        map_path_recv_dt.insert(fpath, chan_recv_dt);
        let chan_send_1_thread = chan_send_1.clone();
        // TODO: how to name the threads? The provided example is not clear
        //       https://docs.rs/rayon/1.5.1/rayon/struct.ThreadPoolBuilder.html#examples-1
        pool.spawn(move || match chan_send_1_thread.send(exec_4(chan_send_dt, thread_data)) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: chan_send_1_thread.send(exec_3(chan_send_dt, thread_data)) failed {}", err);
            }
        });
    }

    // XXX: is this needed?
    debug_eprintln!("{}run_4: drop({:?});", so(), chan_send_1);
    drop(chan_send_1); // close sender so chan.into_iter.collect does not block

    type Recv_Result4 = std::result::Result<Chan_Datum, crossbeam_channel::RecvError>;
    /// run `.recv` on many Receiver channels simultaneously with the help of `crossbeam_channel::Select`
    /// https://docs.rs/crossbeam-channel/0.5.1/crossbeam_channel/struct.Select.html
    /// XXX: I would like to return a `&FPath` to avoid one `FPath.clone()` but it causes
    ///      compiler error about mutable and immutable borrows of `map_path_slp` occurring simultaneously
    ///      cannot borrow `map_path_slp` as mutable because it is also borrowed as immutable
    /// TODO: [2022/03/26] to avoid sending a new `FPath` on each channel send, instead have a single
    ///       Map<u32, FPath> that is referred to on "each side". The `u32` is the lightweight key sent
    ///       along the channel.
    ///       This mapping <u32, FPath> could be used for all other maps with keys `FPath`...
    ///       would a global static lookup map make this easier? No need to pass around instances of `Map<u32, FPath>`.
    fn recv_many_chan(
        fpath_chans: &Map_FPath_ChanRecvDatum, filter_: &Map_FPath_Datum,
    ) -> (FPath, Recv_Result4) {
        debug_eprintln!("{}run_4:recv_many_chan();", sn());
        // "mapping" of index to data; required for various `Select` and `SelectedOperation` procedures,
        // order should match index numeric value returned by `select`
        let mut imap = Vec::<FPath_ChanRecvDatum>::with_capacity(fpath_chans.len());
        // Build a list of operations
        let mut select = crossbeam_channel::Select::new();
        for fp_chan in fpath_chans.iter() {
            // if there is already a DateTime "on hand" for the given fpath then
            // skip receiving on the associated channel
            if filter_.contains_key(*fp_chan.0) {
                continue;
            }
            imap.push((fp_chan.0, fp_chan.1));
            debug_eprintln!("{}run_4:recv_many_chan: select.recv({:?});", so(), fp_chan.1);
            // load `select` with `recv` operations, to be run during later `.select()`
            select.recv(fp_chan.1);
        }
        assert_gt!(imap.len(), 0, "No channel recv operations to select on.");
        debug_eprintln!("{}run_4:recv_many_chan: v: {:?}", so(), imap);
        // Do the `select` operation
        let soper = select.select();
        // get the index of the chosen "winner" of the `select` operation
        let index = soper.index();
        debug_eprintln!("{}run_4:recv_many_chan: soper.recv(&v[{:?}]);", so(), index);
        let fpath = imap[index].0;
        let chan = &imap[index].1;
        debug_eprintln!("{}run_4:recv_many_chan: chan: {:?}", so(), chan);
        // Get the result of the `recv` done during `select`
        let result = soper.recv(chan);
        debug_eprintln!("{}run_4:recv_many_chan() return ({:?}, {:?});", sx(), fpath, result);
        (fpath.clone(), result)
    }

    //
    // main coordination loop (e.g. "main game loop")
    // process the "receiving sysline" channels from the running threads
    // print the soonest available sysline
    //
    // XXX: BTreeMap does not implement `with_capacity`
    //let mut map_path_slp = Map_FPath_SLP::new();
    // TODO: [2022/03/24] change `map_path_datum` to `HashMap<FPath, (SylineP, is_last)>` (`map_path_slp`); currently it's confusing.
    //       that there is a special handler for `Summary` (`map_path_summary`), but not an equivalent `map_path_slp`.
    //       In other words, break apart the passed `Chan_Datum` to the more specific maps.
    let mut map_path_datum = Map_FPath_Datum::new();
    let mut map_path_sumpr = Map_FPath_SummaryPrint::new();
    // crude debugging stats
    let mut _count_recv_ok: usize = 0;
    let mut _count_recv_di: usize = 0;
    let mut sp_total: SummaryPrinted = SummaryPrinted::default();

    let clr_default = Color::White;

    let cli_opt_prepend_fmt = &"%Y%m%dT%H%M%S%.6f %z";
    let tz_utc = Utc::from_offset(&Utc);
    let tz_local = Local.timestamp(0, 0).timezone();

    loop {
        // channels that should be disconnected this loop iteration
        let mut disconnected = Vec::<FPath>::with_capacity(map_path_recv_dt.len());

        // if there is a DateTime available for every FPath channel (one channel is one FPath) then
        // they can all be compared. The soonest DateTime selected then printed.
        if map_path_recv_dt.len() == map_path_datum.len() {
            let fp1: FPath;
            // XXX: arbitrary code block here to allow later `map_path_datum.remove`;
            //      hacky workaround for a difficult error:
            //          "cannot borrow `map_path_datum` as mutable more than once at a time"
            {
                if cfg!(debug_assertions) {
                    for (i, (k, v)) in map_path_recv_dt.iter().enumerate() {
                        debug_eprintln!("{} A1 map_path_recv_dt[{:?}] = {:?}", i, k, v);
                    }
                    for (i, (k, v)) in map_path_datum.iter().enumerate() {
                        debug_eprintln!("{} A1 map_path_datum[{:?}] = {:?}", i, k, v);
                    }
                }
                // XXX: my small investigation into `min`, `max`, `min_by`, `max_by`
                //      https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=a6d307619a7797b97ef6cfc1635c3d33
                let (fpath_min, chan_datum) =
                    // here is part of the "sorting" of syslines process by datetime.
                    // In case of tie datetime values, the tie-breaker will be order of `BTreeMap::iter_mut` which
                    // iterates in order of key sort. https://doc.rust-lang.org/stable/std/collections/struct.BTreeMap.html#method.iter_mut
                    // XXX: could these `unwrap` ever fail?
                    match map_path_datum.iter_mut().min_by(
                        |x, y|
                            x.1.0.as_ref().unwrap().dt.cmp(&(y.1.0.as_ref().unwrap().dt))
                    ) {
                        Some(val) => (
                            val.0, val.1
                        ),
                        None => {
                            eprintln!("ERROR map_path_datum.iter_mut().min_by() returned None");
                            // XXX: not sure what else to do here
                            continue;
                        }
                    };
                if let Some(summary) = chan_datum.1 {
                    debug_eprintln!("{}run_4: A2 chan_datum has Summary {:?}", so(), fpath_min);
                    assert!(chan_datum.0.is_none(), "Chan_Datum Some(Summary) and Some(SyslineP); should only have one Some(). {:?}", fpath_min);
                    if cli_opt_summary {
                        summary_update(fpath_min, summary, &mut map_path_summary);
                    }
                    debug_eprintln!("{}run_4: A2 will disconnect channel {:?}", so(), fpath_min);
                    // receiving a Summary must be the last data sent on the channel
                    disconnected.push(fpath_min.clone());
                } else {
                    let is_last: bool = chan_datum.2;
                    let slp_min: &SyslineP = chan_datum.0.as_ref().unwrap();
                    let clr: &Color = map_path_color.get(fpath_min).unwrap_or(&clr_default);
                    // print the sysline!
                    debug_eprintln!("{}run_4: A3 printing SyslineP@{:p} @[{}, {}] {:?}", so(), slp_min, slp_min.fileoffset_begin(), slp_min.fileoffset_end(), fpath_min);
                    if cfg!(debug_assertions) {
                        let out = fpath_min.to_string()
                            + &String::from(": ")
                            + &(slp_min.to_String_noraw())
                            + &String::from("\n");
                        #[allow(clippy::match_single_binding)]
                        match print_colored_stdout(*clr, out.as_bytes()) { _ => {}};
                    } else {
                        if cli_opt_prepend_utc || cli_opt_prepend_local {
                            #[allow(clippy::single_match)]
                            match (*slp_min).dt {
                                Some(dt) => {
                                    #[allow(clippy::needless_late_init)]
                                    let fmt_;
                                    if cli_opt_prepend_utc {
                                        let dt_ = dt.with_timezone(&tz_utc);
                                        fmt_ = dt_.format(cli_opt_prepend_fmt);
                                    } else { // cli_opt_prepend_local
                                        let dt_ = dt.with_timezone(&tz_local);
                                        fmt_ = dt_.format(cli_opt_prepend_fmt);
                                    }
                                    write_stdout(fmt_.to_string().as_bytes());
                                    write_stdout(" ".as_bytes());
                                },
                                _ => {},
                            }
                        }
                        match (*slp_min).print_color(*clr, color_datetime) {
                            Ok(_) => {},
                            Err(_err) => {
                                eprintln!("ERROR: failed to print; TODO abandon processing for {:?}", fpath_min);
                                // TODO: 2022/04/09 remove this `fpath` from queues, tell it's thread to shutdown
                            }
                        }
                    }
                    if is_last {
                        write_stdout(&NLu8a);
                        sp_total.bytes += 1;
                    }
                    if cli_opt_summary {
                        summaryprint_map_update(slp_min, fpath_min, &mut map_path_sumpr);
                        summaryprint_update(slp_min, &mut sp_total);
                    }
                }
                fp1 = (*fpath_min).clone();
            }
            assert!(!fp1.is_empty(), "Empty filepath.");
            map_path_datum.remove(&fp1);
        } else {
            // else waiting on a (datetime, syslinep) from a file
            debug_eprintln!("{}run_4: B recv_many_chan(map_path_recv_dt: {:?}, map_path_datum: {:?})", so(), map_path_recv_dt, map_path_datum);
            let (fpath1, result1) = recv_many_chan(&map_path_recv_dt, &map_path_datum);
            match result1 {
                Ok(chan_datum) => {
                    debug_eprintln!("{}run_4: B crossbeam_channel::Found for FPath {:?};", so(), fpath1);
                    if let Some(summary) = chan_datum.1 {
                        debug_eprintln!("{}run_4: B chan_datum has Summary {:?}", so(), fpath1);
                        assert!(chan_datum.0.is_none(), "Chan_Datum Some(Summary) and Some(SyslineP); should only have one Some(). FPath {:?}", fpath1);
                        summary_update(&fpath1, summary, &mut map_path_summary);
                        debug_eprintln!("{}run_4: B will disconnect channel {:?}", so(), fpath1);
                        // receiving a Summary must be the last data sent on the channel
                        disconnected.push(fpath1.clone());
                    } else {
                        assert!(chan_datum.0.is_some(), "Chan_Datum None(Summary) and None(SyslineP); should have one Some(). FPath {:?}", fpath1);
                        map_path_datum.insert(fpath1, chan_datum);
                    }
                    _count_recv_ok += 1;
                }
                Err(crossbeam_channel::RecvError) => {
                    debug_eprintln!("{}run_4: B crossbeam_channel::RecvError, will disconnect channel for FPath {:?};", so(), fpath1);
                    // this channel was closed by the sender
                    disconnected.push(fpath1);
                    _count_recv_di += 1;
                }
            }
        }
        // remove channels that have been disconnected
        for fpath in disconnected.into_iter() {
            debug_eprintln!("{}run_4: C map_path_recv_dt.remove({:?});", so(), fpath);
            map_path_recv_dt.remove(&fpath);
        }
        // are there any channels to receive from?
        if map_path_recv_dt.is_empty() {
            debug_eprintln!("{}run_4: D map_path_recv_dt.is_empty(); no more channels to receive from!", so());
            break;
        }
        debug_eprintln!("{}run_4: D map_path_recv_dt: {:?}", so(), map_path_recv_dt);
        debug_eprintln!("{}run_4: D map_path_datum: {:?}", so(), map_path_datum);
    } // end loop

    if cli_opt_summary {
        eprintln!("\nSummary:");
        for fpath in paths.iter() {
            eprint!("File: ");
            let clr = map_path_color.get(fpath).unwrap_or(&clr_default);
            match print_colored_stderr(*clr, fpath.as_bytes()) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("ERROR: {:?}", err);
                    continue;
                }
            };
            eprintln!();

            let summary_opt: Summary_Opt = map_path_summary.remove(fpath);
            match summary_opt {
                Some(summary) => {
                    eprintln!("   Summary Processed:{:?}", summary);
                },
                None => {
                    eprintln!("   Summary Processed: None");
                }
            }
            let summary_print_opt: SummaryPrinted_Opt = map_path_sumpr.remove(fpath);
            match summary_print_opt {
                Some(summary_print) => {
                    eprint!("   Summary Printed  : ");
                    summary_print.print_colored_stderr(&summary_opt);
                },
                None => {
                    eprint!("   Summary Printed  : ");
                    SummaryPrinted::default().print_colored_stderr(&summary_opt);
                }
            }
            eprintln!();
            if OPT_SUMMARY_DEBUG_STATS && summary_opt.is_some() {
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
                let mut ratio = ratio64(&summary._parse_datetime_in_line_lru_cache_hit, &summary._parse_datetime_in_line_lru_cache_miss);
                eprintln!("   caching: SyslineReader::parse_datetime_in_line_lru_cache: hit {:2}, miss {:2}, ratio: {:1.2}", summary._parse_datetime_in_line_lru_cache_hit, summary._parse_datetime_in_line_lru_cache_miss, ratio);
                // LineReader
                ratio = ratio64(&summary._find_line_lru_cache_hit, &summary._find_line_lru_cache_miss);
                eprintln!("   caching: LineReader::find_line_cache: hit {:2}, miss: {:2}, ratio: {:1.2}", summary._find_line_lru_cache_hit, summary._find_line_lru_cache_miss, ratio);
                // BlockReader
                ratio = ratio32(&summary._read_blocks_hit, &summary._read_blocks_miss);
                eprintln!("   caching: BlockReader::read_block_blocks   : hit {:2}, miss {:2}, ratio: {:1.2}", summary._read_blocks_hit, summary._read_blocks_miss, ratio);
                ratio = ratio32(&summary._read_block_cache_lru_hit, &summary._read_block_cache_lru_miss);
                eprintln!("   caching: BlockReader::read_block_cache_lru: hit {:2}, miss {:2}, ratio: {:1.2}", summary._read_block_cache_lru_hit, summary._read_block_cache_lru_miss, ratio);
            }
        }
        eprintln!("{:?}", sp_total);
    }

    debug_eprintln!("{}run_4: E _count_recv_ok {:?} _count_recv_di {:?}", so(), _count_recv_ok, _count_recv_di);
    debug_eprintln!("{}run_4()", sx());
}
