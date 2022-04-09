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

TODO: 2022/04/07 need to handle formats with explicit timezone offset.
      see example `access.log`

*/

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// uses
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs::{File, Metadata, OpenOptions};
use std::io;
use std::io::prelude::Read;
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};
use std::path::Path;
use std::ops::RangeInclusive;
use std::str;
use std::str::FromStr;  // attaches `from_str` to various built-in types
use std::sync::Arc;

extern crate atty;

extern crate backtrace;

//extern crate bstr;
//use bstr::ByteSlice;

extern crate chain_cmp;
use chain_cmp::chmp;

extern crate clap;
use clap::Parser;

extern crate chrono;
use chrono::{DateTime, FixedOffset, Local, Offset, NaiveDateTime, TimeZone, Utc};

extern crate crossbeam_channel;

extern crate debug_print;
#[allow(unused_imports)]
use debug_print::{debug_eprint, debug_eprintln, debug_print, debug_println};

extern crate encoding_rs;

extern crate lru;
use lru::LruCache;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate more_asserts;

extern crate rand;

extern crate rangemap;
use rangemap::{RangeMap,RangeSet};

extern crate mut_static;

extern crate tempfile;
use tempfile::NamedTempFile;

extern crate termcolor;
use termcolor::{Color, ColorSpec, WriteColor};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// misc. globals
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// global constants

/// global test initializer to run once
/// see https://stackoverflow.com/a/58006287/471376
//static _Test_Init_Once: Once = Once::new();

/// NewLine as char
#[allow(non_upper_case_globals, dead_code)]
static NLc: char = '\n';
/// Single-byte newLine char as u8
#[allow(non_upper_case_globals)]
static NLu8: u8 = 10;
/// Newline in a byte buffer
#[allow(non_upper_case_globals)]
static NLu8a: [u8; 1] = [NLu8];

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// custom Results enums for various *Reader functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#481-495

/// `Result` `Ext`ended
/// sometimes things are not `Ok` but a value needs to be returned
#[derive(Debug)]
pub enum ResultS4<T, E> {
    /// Contains the success data
    Found(T),

    /// Contains the success data and reached End Of File and things are okay
    #[allow(non_camel_case_types)]
    Found_EOF(T),

    /// File is empty, or other condition that means "Done", nothing to return, but no bad errors happened
    #[allow(non_camel_case_types)]
    Done,

    /// Contains the error value, something unexpected happened
    Err(E),
}

// XXX: ripped from '\.rustup\toolchains\beta-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs'
//      https://doc.rust-lang.org/src/core/result.rs.html#501-659
// XXX: how to link to specific version of `result.rs`?

impl<T, E> ResultS4<T, E> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the result is [`Ok`, `Found_EOF`, 'Done`].
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline(always)]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, ResultS4::Found(_) | ResultS4::Found_EOF(_) | ResultS4::Done)
    }

    /// Returns `true` if the result is [`Err`].
    #[must_use = "if you intended to assert that this is err, consider `.unwrap_err()` instead"]
    #[inline(always)]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Returns `true` if the result is [`Found_EOF`].
    #[inline(always)]
    pub const fn is_eof(&self) -> bool {
        matches!(*self, ResultS4::Found_EOF(_))
    }

    /// Returns `true` if the result is [`Found_EOF`, `Done`].
    #[inline(always)]
    pub const fn is_done(&self) -> bool {
        matches!(*self, ResultS4::Done)
    }

    /// Returns `true` if the result is an [`Ok`, `Found_EOF`] value containing the given value.
    #[must_use]
    #[inline(always)]
    pub fn contains<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            ResultS4::Found(y) => x == y,
            ResultS4::Found_EOF(y) => x == y,
            ResultS4::Done => false,
            ResultS4::Err(_) => false,
        }
    }

    /// Returns `true` if the result is an [`Err`] value containing the given value.
    #[must_use]
    #[inline(always)]
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
    #[inline(always)]
    pub fn ok(self) -> Option<T> {
        match self {
            ResultS4::Found(x) => Some(x),
            ResultS4::Found_EOF(x) => Some(x),
            ResultS4::Done => None,
            ResultS4::Err(_) => None,
        }
    }

    /// Converts from `Result<T, E>` to [`Option<E>`].
    ///
    /// Converts `self` into an [`Option<E>`], consuming `self`,
    /// and discarding the success value, if any.
    #[inline(always)]
    pub fn err(self) -> Option<E> {
        match self {
            ResultS4::Found(_) => None,
            ResultS4::Found_EOF(_) => None,
            ResultS4::Done => None,
            ResultS4::Err(x) => Some(x),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions - debug printing indentation (stack depths)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

type Map_ThreadId_SD<'a> = HashMap<thread::ThreadId, usize>;

// use `stack_offset_set` to set `_STACK_OFFSET_TABLE` once, use `stack_offset` to get
// XXX: no mutex to guard access; it's rarely written to 🤞
// XXX: a mutable static reference for "complex types" is not allowed in rust
//      use `lazy_static` and `mut_static` to create one
//      see https://github.com/tyleo/mut_static#quickstart
lazy_static! {
    static ref _STACK_OFFSET_TABLE: mut_static::MutStatic<Map_ThreadId_SD<'static>> =
        mut_static::MutStatic::new();
        //Map_ThreadId_SD::new();
}

/// return current stack depth according to `backtrace::trace`, including this function
//#[cfg(any(debug_asserions,test))]
#[inline(always)]
fn stack_depth() -> usize {
    let mut sd: usize = 0;
    backtrace::trace(|_| {
        sd += 1;
        true
    });
    sd
}


/// return current stack offset compared to "original" stack depth. The "original" stack depth
/// should have been recorded at the beginning of the thread by calling `stack_offset_set`.
//#[cfg(any(debug_asserions,test))]
#[inline(always)]
fn stack_offset() -> usize {
    let mut sd: usize = stack_depth() - 1;
    let sd2 = sd; // XXX: copy `sd` to avoid borrow error
    let tid = thread::current().id();
    let so: &usize;
    // XXX: for tests, just set on first call
    if !_STACK_OFFSET_TABLE.is_set().unwrap() {
        match _STACK_OFFSET_TABLE.set(Map_ThreadId_SD::new()) {
            Err(err) => {
                eprintln!("ERROR: stack_offset: _STACK_OFFSET_TABLE.set failed {:?}", err);
            },
            _ => {},
        }
    }
    let so_table = _STACK_OFFSET_TABLE.read().unwrap();
    so = so_table.get(&tid).unwrap_or(&sd2);
    if &sd < so {
        return 0;
    }
    sd -= so;
    sd
}

/// set once in each thread near the beginning of the thread.
/// a positive value `correction` will move the printed output to the right.
/// if the `correction` is too negative then it will print to the left-most column
/// of the terminal. Negative values are useful for when most of a program runs in
/// a function that is several calls deep.
///
/// For example, the `main` function might
/// call an `intialize` function which might call a `run` function. The `run` function
/// might do the majority of work (and debug printing). In that case, from `main`,
/// pass a negative offset of 4 to `stack_offset_set`, i.e. `stack_offset_set(Some(-4))`
/// This way, debug printing from function `run` will start at the left-most column (and not
/// be indented to the right). This may improve readability.
//#[cfg(any(debug_asserions,test))]
fn stack_offset_set(correction: Option<isize>) {
    let sd_ = stack_depth();
    let sdi: isize = (sd_ as isize) - correction.unwrap_or(0);
    let so = std::cmp::max(sdi, 0) as usize;
    let thread_cur = thread::current();
    let tid = thread_cur.id();
    let tname = thread_cur.name().unwrap_or(&"");
    if !_STACK_OFFSET_TABLE.is_set().unwrap() {
        match _STACK_OFFSET_TABLE.set(Map_ThreadId_SD::new()) {
            Err(err) => {
                eprintln!("ERROR: stack_offset_set: _STACK_OFFSET_TABLE.set failed {:?}", err);
            },
            _ => {},
        }
    }
    if _STACK_OFFSET_TABLE.read().unwrap().contains_key(&tid) {
        //eprintln!("WARNING: _STACK_OFFSET_TABLE has already been set for this thread {:?}; stack_offset_set() will be ignored", tid);
        return;
    }
    _STACK_OFFSET_TABLE.write().unwrap().insert(tid, so);
    debug_eprintln!("stack_offset_set({:?}): {:?}({}) stack_offset set to {}, stack_depth {}", correction, tid, tname, so, sd_);
}

/// XXX: `test_stack_offset` requires human visual inspection
#[test]
fn test_stack_offset() {
    debug_eprintln!("{}test_stack_offset", sn());
    debug_eprintln!("{}stack_offset {}", so(), stack_offset());
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
//#[cfg(any(debug_asserions,test))]
fn so() -> &'static str {
    let so_ = stack_offset();
    match so_ {
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
    }
}

/// `print` helper, a `s`tring for e`n`tering a function
//#[cfg(any(debug_assertions,test))]
fn sn() -> &'static str {
    let so_ = stack_offset();
    match so_ {
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
    }
}

/// `print` helper, a `s`tring for e`x`iting a function
//#[cfg(any(debug_assertions,test))]
fn sx() -> &'static str {
    let so_ = stack_offset();
    match so_ {
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
    }
}

/// `print` helper, a `s`tring for e`n`tering and e`x`iting a function
/// (like a small function that only needs a one-liner)
//#[cfg(any(debug_assertions,test))]
fn snx() -> &'static str {
    let so_ = stack_offset();
    match so_ {
        0 => "↔",
        1 => "    ↔",
        2 => "        ↔",
        3 => "            ↔",
        4 => "                ↔",
        5 => "                    ↔",
        6 => "                        ↔",
        7 => "                            ↔",
        8 => "                                ↔",
        9 => "                                    ↔",
        10 => "                                        ↔",
        11 => "                                            ↔",
        12 => "                                                ↔",
        13 => "                                                    ↔",
        14 => "                                                        ↔",
        15 => "                                                            ↔",
        16 => "                                                                ↔",
        17 => "                                                                    ↔",
        18 => "                                                                        ↔",
        19 => "                                                                            ↔",
        _ => "                                                                                ↔",
    }
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
#[test]
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
#[cfg(any(debug_assertions,test))]
fn char_to_char_noraw(c: char) -> char {
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

/// transform utf-8 byte (presumably) to non-raw char
/// only intended for debugging
#[cfg(any(debug_assertions,test))]
fn byte_to_char_noraw(byte: u8) -> char {
    char_to_char_noraw(byte as char)
}

/// transform buffer of utf-8 chars (presumably) to a non-raw String
/// inefficient
/// only intended for debugging
#[allow(non_snake_case)]
#[cfg(any(debug_assertions,test))]
fn buffer_to_String_noraw(buffer: &[u8]) -> String {
    let s1 = match str::from_utf8(buffer) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: buffer_to_String_noraw: Invalid UTF-8 sequence during from_utf8: {}", err);
            return String::with_capacity(0);
        }
    };
    let mut s2 = String::with_capacity(s1.len() + 10);
    for c in s1.chars() {
        let c_ = char_to_char_noraw(c);
        s2.push(c_);
    }
    s2
}

/// transform str to non-raw String version
/// only intended for debugging
#[allow(non_snake_case)]
#[cfg(any(debug_assertions,test))]
fn str_to_String_noraw(str_buf: &str) -> String {
    let mut s2 = String::with_capacity(str_buf.len() + 1);
    for c in str_buf.chars() {
        let c_ = char_to_char_noraw(c);
        s2.push(c_);
    }
    s2
}

/// return contents of file utf-8 chars (presumably) at `path` as non-raw String
/// inefficient
/// only intended for debugging
#[allow(non_snake_case)]
#[cfg(any(test))]
fn file_to_String_noraw(path: &FPath) -> String {
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
        "Read {} bytes but expected to read file size count of bytes {} for file {:?}",
        s2read, filesz, path
    );
    let mut s3 = String::with_capacity(filesz + 1);
    for c in s2.chars() {
        let c_ = char_to_char_noraw(c);
        s3.push(c_);
    }
    return s3;
}

/// print colored output to terminal if possible choosing using passed stream
/// otherwise, print plain output
/// taken from https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal
fn print_colored(color: Color, value: &[u8], std_: &mut termcolor::StandardStream) -> Result<()> {
    match std_.set_color(ColorSpec::new().set_fg(Some(color))) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: print_colored: std.set_color({:?}) returned error {}", color, err);
            return Err(err);
        }
    };
    //let mut stderr_lock:Option<io::StderrLock> = None;
    //if cfg!(debug_assertions) {
    //    stderr_lock = Some(io::stderr().lock());
    //}
    match std_.write(value) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: print_colored: std_.write(…) returned error {}", err);
            return Err(err);
        }
    }
    match std_.reset() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: print_colored: std_.reset() returned error {}", err);
            return Err(err);
        }
    }
    std_.flush()?;
    Ok(())
}

/// print colored output to terminal on stdout
/// taken from https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal
fn print_colored_stdout(color: Color, value: &[u8]) -> Result<()> {
    let mut choice: termcolor::ColorChoice = termcolor::ColorChoice::Never;
    if atty::is(atty::Stream::Stdout) || cfg!(debug_assertions) {
        choice = termcolor::ColorChoice::Always;
    }
    let mut stdout = termcolor::StandardStream::stdout(choice);
    print_colored(color, value, &mut stdout)
}

/// print colored output to terminal on stderr
/// taken from https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal
fn print_colored_stderr(color: Color, value: &[u8]) -> Result<()> {
    let mut choice: termcolor::ColorChoice = termcolor::ColorChoice::Never;
    if atty::is(atty::Stream::Stderr) || cfg!(debug_assertions) {
        choice = termcolor::ColorChoice::Always;
    }
    let mut stderr = termcolor::StandardStream::stderr(choice);
    print_colored(color, value, &mut stderr)
}

/// safely write the `buffer` to stdout with help of `StdoutLock`
pub fn write_stdout(buffer: &[u8]) {
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    match stdout_lock.write(buffer) {
        Ok(_) => {}
        Err(err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            eprintln!("ERROR: write: StdoutLock.write(buffer@{:p} (len {})) error {}", buffer, buffer.len(), err);
        }
    }
    match stdout_lock.flush() {
        Ok(_) => {}
        Err(err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            eprintln!("ERROR: write: stdout flushing error {}", err);
        }
    }
    if cfg!(debug_assertions) {
        match io::stderr().flush() {
            Ok(_) => {},
            Err(_) => {},
        }
    }
}

/// helper flush stdout and stderr
pub fn flush_stdouterr() {
    match io::stdout().flush() {
        Ok(_) => {},
        Err(_) => {},
    };
    match io::stderr().flush() {
        Ok(_) => {},
        Err(_) => {},
    };
}

/// write to console, `raw` as `true` means "as-is"
/// else use `char_to_char_noraw` to replace chars in `buffer` (inefficient)
/// only intended for debugging
#[cfg(any(debug_assertions,test))]
pub fn pretty_print(buffer: &[u8], raw: bool) {
    if raw {
        return write_stdout(buffer);
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
        let c_ = char_to_char_noraw(c);
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
// helper functions - misc.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// testing helper to write a `str` to a temporary file
/// The temporary file will be automatically deleted when returned `NamedTempFile` is dropped.
#[cfg(test)]
fn create_temp_file(content: &str) -> NamedTempFile {
    let mut ntf1 = match NamedTempFile::new() {
        Ok(val) => val,
        Err(err) => {
            panic!("NamedTempFile::new() return Err {}", err);
        }
    };
    match ntf1.write_all(content.as_bytes()) {
        Ok(_) => {}
        Err(err) => {
            panic!("NamedTempFile::write_all() return Err {}", err);
        }
    }

    return ntf1;
}

/// testing helper to write a `[u8]` to a temporary file
/// The temporary file will be automatically deleted when returned `NamedTempFile` is dropped.
#[cfg(test)]
fn create_temp_file_bytes(content: &[u8]) -> NamedTempFile {
    let mut ntf1 = match NamedTempFile::new() {
        Ok(val) => val,
        Err(err) => {
            panic!("NamedTempFile::new() return Err {}", err);
        }
    };
    match ntf1.write_all(content) {
        Ok(_) => {}
        Err(err) => {
            panic!("NamedTempFile::write_all() return Err {}", err);
        }
    }

    return ntf1;
}

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
    return COLORS_TEXT[ci];
}

/// does chrono datetime pattern have a timezone
/// see https://docs.rs/chrono/latest/chrono/format/strftime/
#[inline(always)]
#[cfg(test)]
fn dt_pattern_has_tz(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%Z") ||
    pattern.contains("%z") ||
    pattern.contains("%:z") ||
    pattern.contains("%#z")
}

/// does chrono datetime pattern have a year
/// see https://docs.rs/chrono/latest/chrono/format/strftime/
#[inline(always)]
#[cfg(test)]
fn dt_pattern_has_year(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%Y") ||
    pattern.contains("%y")
}

/// convert `&str` to a chrono `Option<DateTime<FixedOffset>>` instance
#[inline(always)]
fn str_datetime(dts: &str, pattern: &DateTimePattern_str, patt_has_tz: bool, tz_offset: &FixedOffset) -> DateTimeL_Opt {
    debug_eprintln!("{}str_datetime({:?}, {:?}, {:?}, {:?})", sn(), str_to_String_noraw(dts), pattern, patt_has_tz, tz_offset);
    // BUG: [2022/03/21] chrono Issue #660 https://github.com/chronotope/chrono/issues/660
    //      ignoring surrounding whitespace in the passed `fmt`
    // LAST WORKING HERE 2022/04/07 22:07:34 see scrap experiments in `Projects/rust-tests/test8-tz/`
    // TODO: 2022/04/07
    //       if dt_pattern has TZ then create a `DateTime`
    //       if dt_pattern does not have TZ then create a `NaiveDateTime`
    //       then convert that to `DateTime` with aid of crate `chrono_tz`
    //       TZ::from_local_datetime();
    //       How to determine TZ to use? Should it just use Local?
    //       Defaulting to local TZ would be an adequate start.
    //       But pass around as `chrono::DateTime`, not `chrono::Local`.
    //       Replace use of `Local` with `DateTime. Change typecast `DateTimeL`
    //       type. Can leave the name in place for now.
    if patt_has_tz {
        match DateTime::parse_from_str(dts, pattern) {
            Ok(val) => {
                debug_eprintln!(
                    "{}str_datetime: DateTime::parse_from_str({:?}, {:?}) extrapolated DateTime {:?}",
                    so(),
                    str_to_String_noraw(dts),
                    pattern,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `dts`
                //       and `pattern`
                if !SyslineReader::datetime_from_str_workaround_Issue660(dts, pattern) {
                    debug_eprintln!("{}str_datetime: skip match due to chrono Issue #660", sx());
                    return None;
                }
                debug_eprintln!("{}str_datetime return {:?}", sx(), Some(val));
                return Some(val);
            }
            Err(err) => {
                debug_eprintln!("{}str_datetime: DateTime::parse_from_str({:?}, {:?}) failed ParseError {}", sx(), dts, pattern, err);
                return None;
            }
        };
    }

    // no timezone in pattern, first convert to NaiveDateTime
    //let tz_offset = FixedOffset::west(3600 * 8);
    let dt_naive = match NaiveDateTime::parse_from_str(dts, pattern) {
        Ok(val) => {
            debug_eprintln!(
                "{}str_datetime: NaiveDateTime.parse_from_str({:?}, {:?}) extrapolated NaiveDateTime {:?}",
                so(),
                str_to_String_noraw(dts),
                pattern,
                val,
            );
            // HACK: workaround chrono Issue #660 by checking for matching begin, end of `dts`
            //       and `dtpd.pattern`
            if !SyslineReader::datetime_from_str_workaround_Issue660(dts, pattern) {
                debug_eprintln!("{}str_datetime: skip match due to chrono Issue #660", sx());
                return None;
            }
            val
        }
        Err(err) => {
            debug_eprintln!("{}str_datetime: NaiveDateTime.parse_from_str({:?}, {:?}) failed ParseError {}", sx(), dts, pattern, err);
            return None;
        }
    };
    // second convert the NaiveDateTime to FixedOffset
    match tz_offset.from_local_datetime(&dt_naive).earliest() {
        Some(val) => {
            debug_eprintln!(
                "{}str_datetime: tz_offset.from_local_datetime({:?}).earliest() extrapolated NaiveDateTime {:?}",
                so(),
                dt_naive,
                val,
            );
            // HACK: workaround chrono Issue #660 by checking for matching begin, end of `dts`
            //       and `dtpd.pattern`
            if !SyslineReader::datetime_from_str_workaround_Issue660(dts, pattern) {
                debug_eprintln!("{}str_datetime: skip match due to chrono Issue #660, return None", sx());
                return None;
            }
            debug_eprintln!("{}str_datetime return {:?}", sx(), Some(val));
            return Some(val);
        }
        None => {
            debug_eprintln!("{}str_datetime: NaiveDateTime.parse_from_str({:?}, {:?}) returned None, return None", sx(), dts, pattern);
            return None;
        }
    };
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// command-line parsing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// TODO: use `std::path::Path`
/// `F`ake `Path` or `F`ile `Path`
type FPath = String;
type FPaths = Vec::<FPath>;

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
const CLI_HELP_DT_EXAMPLE: &str = "20200102T123000";
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
    let mut blocksz_: u64 = 0;
    let errs = format!("Unable to parse a number for --blocksz {:?}", blockszs);

    if blockszs.starts_with("0x") {
        blocksz_ = match BlockSz::from_str_radix(&blockszs.trim_start_matches("0x"), 16) {
            Ok(val) => val,
            Err(err) => { return Err(format!("{} {}", errs, err)) }
        };
    } else if blockszs.starts_with("0o") {
        blocksz_ = match BlockSz::from_str_radix(&blockszs.trim_start_matches("0o"), 8) {
            Ok(val) => val,
            Err(err) => { return Err(format!("{} {}", errs, err)) }
        };
    } else if blockszs.starts_with("0b") {
        blocksz_ = match BlockSz::from_str_radix(&blockszs.trim_start_matches("0b"), 2) {
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
        let local_offs = Local.timestamp(0, 0).offset().fix().local_minus_utc();
        let hours = local_offs / 3600;
        let mins = local_offs % 3600;
        tzo_ = format!("{:+03}{:02}", hours, mins);
    } else {
        tzo_ = tzo.clone();
    }
    let fo_val = match i32::from_str_radix(&tzo_.as_str(), 10) {
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
        &args.tz_offset.unwrap_or(String::from(""))
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
                    match str_datetime(&dts.as_str(), &dtpd.pattern, dtpd.tz, &tz_offset) {
                        Some(val) => {
                            dto = Some(val);
                            break;
                        }
                        None => {}
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

    if filter_dt_after.is_some() && filter_dt_before.is_some() {
        let dta = filter_dt_after.unwrap();
        let dtb = filter_dt_before.unwrap();
        if dta > dtb {
            eprintln!("ERROR: Datetime --dt-after ({}) is after Datetime --dt-before ({})", dta, dtb);
            std::process::exit(1);
        }
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

    run_4(fpaths, blocksz, &filter_dt_after, &filter_dt_before, tz_offset, cli_opt_prepend_utc, cli_opt_prepend_local, cli_opt_summary);

    debug_eprintln!("{}main()", sx());
    return Ok(());
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Blocks and BlockReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// type aliases
/// Block Size in bytes
type BlockSz = u64;
/// Byte offset (Index) _into_ a `Block` from beginning of `Block`
type BlockIndex = usize;
/// Offset into a file in `Block`s, depends on `BlockSz` runtime value
type BlockOffset = u64;
/// Offset into a file in bytes
type FileOffset = u64;
/// Block of bytes data read from some file storage
type Block = Vec<u8>;
/// Sequence of Bytes
type Bytes = Vec<u8>;
/// thread-safe Atomic Reference Counting Pointer to a `Block`
type BlockP = Arc<Block>;

type Slices<'a> = Vec<&'a [u8]>;
type Blocks = BTreeMap<BlockOffset, BlockP>;
type BlocksLRUCache = LruCache<BlockOffset, BlockP>;
/// for case where reading blocks, lines, or syslines reaches end of file, the value `WriteZero` will
/// be used here ot mean "_end of file reached, nothing new_"
/// XXX: this is a hack
#[allow(non_upper_case_globals)]
const EndOfFile: ErrorKind = ErrorKind::WriteZero;
/// minimum Block Size (inclusive)
const BLOCKSZ_MIN: BlockSz = 1;
/// maximum Block Size (inclusive)
const BLOCKSZ_MAX: BlockSz = 0xFFFFFF;
/// default Block Size
const BLOCKSZ_DEF: BlockSz = 0xFFFF;
#[allow(non_upper_case_globals)]
const BLOCKSZ_DEFs: &str = &"0xFFFF";
/// minimum and maximum Block Size as a `RangeInclusive`
const BLOCKSZ_RANGE: RangeInclusive<BlockSz> = BLOCKSZ_MIN..=BLOCKSZ_MAX;

/// Cached file reader that stores data in `BlockSz` byte-sized blocks.
/// A `BlockReader` corresponds to one file.
/// TODO: make a copy of `path`, no need to hold a reference, it just complicates things by introducing explicit lifetimes
pub struct BlockReader<'blockreader> {
    /// Path to file
    pub path: &'blockreader FPath,
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
    /// count of bytes stored by the `BlockReader`
    _count_bytes: u64,
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
            .field("count_bytes", &self._count_bytes)
            .field("blocks cached", &self.blocks.len())
            .field("cache LRU hit", &self.stats_read_block_cache_lru_hit)
            .field("cache LRU miss", &self.stats_read_block_cache_lru_miss)
            .field("cache hit", &self.stats_read_block_cache_hit)
            .field("cache miss", &self.stats_read_block_cache_miss)
            .finish()
    }
}

/// helper for humans debugging Blocks, very inefficient
#[cfg(any(debug_assertions,test))]
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
            let cp = byte_to_char_noraw(v);
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
    pub fn new(path_: &'blockreader FPath, blocksz: BlockSz) -> BlockReader<'blockreader> {
        // TODO: why not open the file here? change `open` to a "static class wide" (or equivalent)
        //       that does not take a `self`. This would simplify some things about `BlockReader`
        // TODO: how to make some fields `blockn` `blocksz` `filesz` immutable?
        //       https://stackoverflow.com/questions/23743566/how-can-i-force-a-structs-field-to-always-be-immutable-in-rust
        assert_ne!(0, blocksz, "Block Size cannot be 0");
        assert_ge!(blocksz, BLOCKSZ_MIN, "Block Size too small");
        assert_le!(blocksz, BLOCKSZ_MAX, "Block Size too big");
        return BlockReader {
            path: path_,
            file: None,
            file_metadata: None,
            filesz: 0,
            blockn: 0,
            blocksz,
            _count_bytes: 0,
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
        blockoffset: BlockOffset, blocksz: BlockSz, blockindex: BlockIndex,
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

    /// count of blocks stored by this `BlockReader` (during calls to `BlockReader::read_block`)
    pub fn count(&self) -> u64 {
        return self.blocks.len() as u64;
    }

    /// count of bytes stored by this `BlockReader` (during calls to `BlockReader::read_block`)
    pub fn count_bytes(&self) -> u64 {
        return self._count_bytes;
    }

    /// open the `self.path` file, set other field values after opening.
    /// propagates any `Err`, success returns `Ok(())`
    pub fn open(&mut self) -> Result<()> {
        assert!(self.file.is_none(), "ERROR: the file is already open {:?}", &self.path);
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
    /// TODO: create custom `ResultS4` for this analogous to `LineReader` `SyslineReader
    ///       get rid of hack `EndOfFile`
    pub fn read_block(&mut self, blockoffset: BlockOffset) -> Result<BlockP> {
        debug_eprintln!("{}read_block: @{:p}.read_block({})", sn(), self, blockoffset);
        assert!(self.file.is_some(), "File has not been opened {:?}", self.path);
        // check LRU cache
        match self._read_block_lru_cache.get(&blockoffset) {
            Some(bp) => {
                self.stats_read_block_cache_lru_hit += 1;
                debug_eprintln!(
                    "{}read_block: return Ok(BlockP@{:p}); hit LRU cache Block[{}] @[{}, {}) len {}",
                    sx(),
                    &*bp,
                    &blockoffset,
                    BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz),
                    BlockReader::file_offset_at_block_offset(blockoffset+1, self.blocksz),
                    (*bp).len(),
                );
                return Ok(bp.clone());
            }
            None => {
                debug_eprintln!("{}read_block: blockoffset {} not found LRU cache", so(), blockoffset);
                self.stats_read_block_cache_lru_miss += 1;
            }
        }
        // check hash map cache
        if self.blocks.contains_key(&blockoffset) {
            debug_eprintln!("{}read_block: blocks.contains_key({})", so(), blockoffset);
            self.stats_read_block_cache_hit += 1;
            let bp: &BlockP = &self.blocks[&blockoffset];
            debug_eprintln!("{}read_block: LRU cache put({}, BlockP@{:p})", so(), blockoffset, bp);
            self._read_block_lru_cache.put(blockoffset, bp.clone());
            debug_eprintln!(
                "{}read_block: return Ok(BlockP@{:p}); cached Block[{}] @[{}, {}) len {}",
                sx(),
                &*self.blocks[&blockoffset],
                &blockoffset,
                BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz),
                BlockReader::file_offset_at_block_offset(blockoffset+1, self.blocksz),
                self.blocks[&blockoffset].len(),
            );
            return Ok(bp.clone());
        }
        self.stats_read_block_cache_miss += 1;
        let seek = (self.blocksz * blockoffset) as u64;
        let mut file_ = self.file.as_ref().unwrap();
        match file_.seek(SeekFrom::Start(seek)) {
            Ok(_) => {}
            Err(err) => {
                debug_eprintln!("{}read_block: return Err({})", sx(), err);
                eprintln!("ERROR: file.SeekFrom({}) Error {}", seek, err);
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
        debug_eprintln!("{}read_block: reader.read_to_end(@{:p})", so(), &buffer);
        match reader.read_to_end(&mut buffer) {
            Ok(val) => {
                if val == 0 {
                    // special case of `Err` that caller should handle
                    debug_eprintln!(
                        "{}read_block: return Err(EndOfFile) EndOfFile blockoffset {} {:?}",
                        sx(),
                        blockoffset,
                        self.path
                    );
                    return Err(Error::new(EndOfFile, "End Of File"));
                }
            }
            Err(err) => {
                eprintln!("ERROR: reader.read_to_end(buffer) error {}", err);
                debug_eprintln!("{}read_block: return Err({})", sx(), err);
                return Err(err);
            }
        };
        let blen64 = buffer.len() as u64;
        let bp = BlockP::new(buffer);
        // store block
        debug_eprintln!("{}read_block: blocks.insert({}, BlockP@{:p})", so(), blockoffset, bp);
        match self.blocks.insert(blockoffset, bp.clone()) {
            Some(bp_) => {
                eprintln!("WARNING: blocks.insert({}, BlockP@{:p}) already had a entry BlockP@{:p}", blockoffset, bp, bp_);
            },
            _ => {},
        }
        self._count_bytes += blen64;
        // store in LRU cache
        debug_eprintln!("{}read_block: LRU cache put({}, BlockP@{:p})", so(), blockoffset, bp);
        self._read_block_lru_cache.put(blockoffset, bp.clone());
        debug_eprintln!(
            "{}read_block: return Ok(BlockP@{:p}); new Block[{}] @[{}, {}) len {}",
            sx(),
            &*self.blocks[&blockoffset],
            &blockoffset,
            BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz),
            BlockReader::file_offset_at_block_offset(blockoffset+1, self.blocksz),
            (*self.blocks[&blockoffset]).len()
        );
        Ok(bp)
    }

    /// get byte at FileOffset
    /// `None` means the data at `FileOffset` was not available
    /// Does not request any `read_block`! Only copies from what is currently available from prior
    /// calls to `read_block`.
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
    /// uses `self._get_byte` which does not request any reads!
    /// debug helper only
    fn _vec_from(&self, fo_a: FileOffset, fo_b: FileOffset) -> Bytes {
        assert_le!(fo_a, fo_b, "bad fo_a {} fo_b {} FPath {:?}", fo_a, fo_b, self.path);
        assert_le!(fo_b, self.filesz, "bad fo_b {} but filesz {} FPath {:?}", fo_b, self.filesz, self.path);
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
        while fo_at < fo_b {
            let b = match self._get_byte(fo_at) {
                Some(val) => val,
                None => {
                    break;
                }
            };
            vec_.push(b);
            fo_at += 1;
        }
        return vec_;
    }
}

#[test]
fn test_BlockReader1() {
    test_BlockReader(&FPath::from("./logs/other/tests/basic-basic-dt10-repeats.log"), 2);
}

/// basic test of BlockReader things
#[allow(non_snake_case, dead_code)]
#[cfg(test)]
fn test_BlockReader(path_: &FPath, blocksz: BlockSz) {
    debug_println!("test_BlockReader()");

    // testing BlockReader basics

    let mut br1 = BlockReader::new(&path_, blocksz);
    debug_println!("new {:?}", &br1);
    match br1.open() {
        Ok(_) => {
            debug_eprintln!("opened {:?}", path_);
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
#[test]
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
#[test]
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
#[test]
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
#[test]
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
#[test]
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LinePart, Line, and LineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Struct describing a part or all of a line within a `Block`
/// A "line" can span more than one `Block`. This tracks part or all of a line within
/// one `Block`. One `LinePart` to one `Block`.
/// But one or more `LinePart` are necessary to represent an entire "line".
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
    // XXX: does not handle multi-byte encodings
    const CHARSZ: usize = 1;

    pub fn new(
        blocki_beg: BlockIndex, blocki_end: BlockIndex, blockp: BlockP, fileoffset: FileOffset,
        blockoffset: BlockOffset, blocksz: BlockSz,
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
            blocki_beg,
            blocki_end,
            blockp,
            fileoffset,
            blockoffset,
            blocksz,
        }
    }

    /// length of line starting at index `blocki_beg`
    pub fn len(&self) -> usize {
        (self.blocki_end - self.blocki_beg) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// count of bytes of this `LinePart`
    /// XXX: `count_bytes` and `len` is overlapping and confusing.
    pub fn count_bytes(&self) -> u64 {
        (self.len() * LinePart::CHARSZ) as u64
    }

    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    fn _to_String_raw(self: &LinePart, raw: bool) -> String {
        // XXX: intermixing byte lengths and character lengths
        // XXX: does not handle multi-byte
        let s1: String;
        let slice_ = &(*self.blockp)[self.blocki_beg..self.blocki_end];
        if raw {
            unsafe {
                s1 = String::from_utf8_unchecked(Vec::<u8>::from(slice_));
            }
            return s1;
        }
        s1 = buffer_to_String_noraw(slice_);
        s1
    }

    pub fn contains(self: &LinePart, byte_: &u8) -> bool {
        (*self.blockp).contains(&byte_)
    }

    /// `Line` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &LinePart) -> String {
        return self._to_String_raw(false);
    }

    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &LinePart) -> String {
        return self._to_String_raw(true);
    }

    /// return Box pointer to slice of bytes that make up this `LinePart`
    pub fn block_boxptr(&self) -> Box<&[u8]> {
        let slice_ = &(*self.blockp).as_slice()[self.blocki_beg..self.blocki_end];
        //let slice_ptr: *const &[u8] = **slice_;
        let slice_boxp = Box::new(slice_);
        slice_boxp
    }

    /// return Box pointer to slice of bytes in this `LinePart` from `a` to end
    pub fn block_boxptr_a(&self, a: &LineIndex) -> Box<&[u8]> {
        debug_assert_lt!(self.blocki_beg+a, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed a {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, a, self.blocki_beg + a, self.blocki_end);
        let slice1 = &(*self.blockp).as_slice()[(self.blocki_beg+a)..self.blocki_end];
        //let slice2 = &slice1[*a..];
        let slice_boxp = Box::new(slice1);
        slice_boxp
    }

    /// return Box pointer to slice of bytes in this `LinePart` from beginning to `b`
    pub fn block_boxptr_b(&self, b: &LineIndex) -> Box<&[u8]> {
        debug_assert_lt!(self.blocki_beg+b, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed b {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, b, self.blocki_beg + b, self.blocki_end);
        let slice1 = &(*self.blockp).as_slice()[..self.blocki_beg+b];
        //let slice2 = &slice1[..*b];
        let slice_boxp = Box::new(slice1);
        slice_boxp
    }
    

    /// return Box pointer to slice of bytes in this `LinePart` from `a` to `b`
    pub fn block_boxptr_ab(&self, a: &LineIndex, b: &LineIndex) -> Box<&[u8]> {
        debug_assert_lt!(a, b, "bad LineIndex");
        debug_assert_lt!(self.blocki_beg+a, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed a {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, a, self.blocki_beg + a, self.blocki_end);
        debug_assert_lt!(self.blocki_beg+b, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed b {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, b, self.blocki_beg + b, self.blocki_end);
        debug_assert_lt!(b - a, self.len(), "Passed LineIndex {}..{} (diff {}) are larger than this LinePart 'slice' {}", a, b, b - a, self.len());
        let slice1 = &(*self.blockp).as_slice()[(self.blocki_beg+a)..(self.blocki_beg+b)];
        //let slice2 = &slice1[*a..*b];
        let slice_boxp = Box::new(slice1);
        slice_boxp
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
        let mut fo_b = 0;
        if !self.lineparts.is_empty() {
            fo_b = self.lineparts[0].fileoffset;
        }
        let mut fo_e = 0;
        if !self.lineparts.is_empty() {
            let last_li = self.lineparts.len() - 1;
            fo_e = self.lineparts[last_li].fileoffset + (self.lineparts[last_li].len() as FileOffset) - 1;
        }
        f.debug_struct("Line")
            .field("line.fileoffset_begin()", &fo_b)
            .field("line.fileoffset_end()", &fo_e)
            .field("lineparts @", &format_args!("{:p}", &self))
            .field("lineparts.len", &self.lineparts.len())
            .field("lineparts", &li_s)
            .finish()
    }
}

/// return value for `Line::get_boxptrs`
pub enum enum_BoxPtrs <'a> {
    SinglePtr(Box<&'a [u8]>),
    MultiPtr(Vec<Box<&'a [u8]>>),
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

    pub fn new_from_linepart(linepart: LinePart) -> Line {
        let mut v = LineParts::with_capacity(Line::LINE_PARTS_WITH_CAPACITY);
        v.push(linepart);
        return Line { lineparts: v };
    }

    //pub fn charsz(self: &Line) {
    //    self.lineparts.first().unwrap().
    //}

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
    /// "points" to first character of `Line`
    pub fn fileoffset_begin(self: &Line) -> FileOffset {
        debug_assert_ne!(self.lineparts.len(), 0, "This Line has no `LinePart`");
        self.lineparts[0].fileoffset
    }

    /// the byte offset into the file where this `Line` ends, inclusive (not one past ending)
    pub fn fileoffset_end(self: &Line) -> FileOffset {
        debug_assert_ne!(self.lineparts.len(), 0, "This Line has no `LinePart`");
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

    /// sum of `LinePart.count_bytes`
    pub fn count_bytes(self: &Line) -> u64 {
        let mut cb: u64 = 0;
        for lp in self.lineparts.iter() {
            cb += lp.count_bytes();
        }
        cb
    }

    pub fn get_linepart(self: &Line, mut a: LineIndex) -> &LinePart {
        for linepart in self.lineparts.iter() {
            let len_ = linepart.len();
            if a < len_ {
                return &linepart;
            }
            a -= len_;
        }
        // XXX: not sure if this is the best choice
        &(self.lineparts.last().unwrap())
    }

    /// does the `Line` contain the byte value?
    pub fn contains(self: &Line, byte_: &u8) -> bool {
        for linepart in self.lineparts.iter() {
            if linepart.contains(byte_) {
                return true;
            }
        }
        false
    }

    /// does the `Line` contain the byte value?
    pub fn contains_at(self: &Line, byte_: &u8, a: &LineIndex, b: &LineIndex) -> bool {
        debug_assert_le!(a, b, "passed bad LineIndex pair");
        for linepart in self.lineparts.iter() {
            if linepart.contains(byte_) {
                return true;
            }
        }
        false
    }

    /// return all slices that make up this `Line`
    /// CANDIDATE FOR REMOVAL?
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
    /// CANDIDATE FOR REMOVAL?
    pub fn get_slices_count(self: &Line) -> usize {
        return self.lineparts.len();
    }


    /// get Box pointers to the underlying `&[u8]` slice that makes up this `Line`.
    /// There may be more than one slice as the `Line` may cross block boundaries. So
    /// return the sequence of Box pointers in a `Vec`.
    /// TODO: the `Vec<Box<&[u8]>>` creation is expensive
    ///       consider allowing a mut &Vec to be passed in. However, this will require declaring lifetimes!
    ///       LAST WORKING HERE 2022/04/03 23:54:00
    // TODO: due to unstable feature `Sized` in `Box`, cannot do
    //           fn get_boxptrs(...) -> either::Either<Box<&[u8]>, Vec<Box<&[u8]>>>
    //       causes error `experimental Sized`
    pub fn get_boxptrs(self: &Line, mut a: LineIndex, mut b: LineIndex) -> enum_BoxPtrs<'_> {
        debug_assert_le!(a, b, "passed bad LineIndex pair");
        // do the simple case first (single `Box` pointer required)
        // doing this here, as opposed to intermixing with multiple case, avoids compiler complaint of "use of possibly-uninitialized `ptrs`"
        let mut a1: LineIndex = a;
        let mut b1: LineIndex = b;
        for linepart_ in &self.lineparts {
            let len_ = linepart_.len();
            if a1 < len_ && b1 < len_ {
                return enum_BoxPtrs::SinglePtr(linepart_.block_boxptr_ab(&a1, &b1));
            } else if a1 < len_ && len_ <= b1 {
                break;
            }
            a1 -= len_;
            b1 -= len_;
        }
        // do the harder case (multiple `Box` pointers required)
        let mut a_found = false;
        let mut b_search = false;
        let mut ptrs: Vec<Box<&[u8]>> = Vec::<Box::<&[u8]>>::new();
        for linepart_ in &self.lineparts {
            debug_eprintln!("{}get_boxptrs: linepart {:?}", so(), linepart_.to_String_noraw());
            let len_ = linepart_.len();
            if !a_found && a < len_ {
                a_found = true;
                b_search = true;
                if b < len_ {
                    debug_eprintln!("{}get_boxptrs: ptrs.push(linepart_.block_boxptr_ab({}, {}))", so(), a, b);
                    ptrs.push(linepart_.block_boxptr_ab(&a, &b));  // store [a..b]  (entire slice, entire `Line`)
                    debug_assert_gt!(ptrs.len(), 1, "ptrs is {} elements, expected >= 1; this should have been handled earlier", ptrs.len());
                    return enum_BoxPtrs::MultiPtr(ptrs);
                }
                debug_eprintln!("{}get_boxptrs: ptrs.push(linepart_.block_boxptr_a({}))", so(), a);
                ptrs.push(linepart_.block_boxptr_a(&a));  // store [a..]  (first slice of `Line`)
                b -= len_;
                continue;
            } else if !a_found {
                a -= len_;
                continue;
            }
            if b_search && b < len_ {
                debug_eprintln!("{}get_boxptrs: ptrs.push(linepart_.block_boxptr_b({}))", so(), b);
                ptrs.push(linepart_.block_boxptr_b(&b));  // store [..b] (last slice of `Line`)
                break;
            } else  {
                debug_eprintln!("{}get_boxptrs: ptrs.push(linepart_.block_boxptr())", so());
                ptrs.push(linepart_.block_boxptr());  // store [..] (entire slice, middle part of `Line`)
                b -= len_;
            }
        }
        enum_BoxPtrs::MultiPtr(ptrs)
    }

    /// `raw` true will write directly to stdout from the stored `Block`
    /// `raw` false will write transcode each byte to a character and use pictoral representations
    /// XXX: `raw==false` does not handle multi-byte encodings
    #[cfg(any(debug_assertions,test))]
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
                            "ERROR: StdoutLock.write(@{:p}[{}‥{}]) error {}",
                            &*linepart.blockp, linepart.blocki_beg, linepart.blocki_end, err
                        );
                    }
                }
            } else {
                // XXX: only handle single-byte encodings
                // XXX: this is not efficient
                //let s = match str::from_utf8_lossy(slice) {
                let s = match str::from_utf8(slice) {
                    Ok(val) => val,
                    Err(err) => {
                        eprintln!("ERROR: Invalid UTF-8 sequence during from_utf8: {:?}", err);
                        continue;
                    }
                };
                let mut dst: [u8; 4] = [0, 0, 0, 0];
                for c in s.chars() {
                    let c_ = char_to_char_noraw(c);
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
    /// with pictoral representation (i.e. `byte_to_char_noraw`)
    /// XXX: not efficient!
    /// TODO: this would be more efficient returning `&str`
    ///       https://bes.github.io/blog/rust-strings
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
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
                        eprintln!("ERROR: failed to convert [u8] at FileOffset[{}‥{}] to utf8 str; {}", fo1, fo2, err);
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
                    let c = byte_to_char_noraw(*b);
                    s1.push(c);
                }
            }
        }
        return s1;
    }

    // XXX: rust does not support function overloading which is really surprising and disappointing
    /// `Line` to `String`
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &Line) -> String {
        return self._to_String_raw(true);
    }

    #[allow(non_snake_case)]
    pub fn to_String_from(self: &Line, _from: usize) -> String {
        unimplemented!("to_String_from");
    }

    #[allow(non_snake_case)]
    pub fn to_String_from_to(self: &Line, _from: usize, _to: usize) -> String {
        unimplemented!("to_String_from_to");
    }

    /// `Line` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &Line) -> String {
        return self._to_String_raw(false);
    }

    /// slice that represents the entire `Line`
    /// if `Line` does not cross a Block then this returns slice into the `Block`,
    /// otherwise it requires a copy of `Block`s data
    /// XXX: naive implementation
    /// XXX: cannot return slice because
    ///      1. size not known at compile time so cannot place on stack
    ///      2. slice is an array which is not an "owned type"
    /// TODO: add tests
    /// CANDIDATE FOR REMOVAL
    pub fn as_bytes(self: &Line) -> Bytes {
        assert_gt!(self.lineparts.len(), 0, "This Line has no LineParts");
        // efficient case, Line does not cross any Blocks
        if self.lineparts.len() == 1 {
            let bi_beg = self.lineparts[0].blocki_beg;
            let bi_end = self.lineparts[0].blocki_end;
            assert_eq!(bi_end - bi_beg, self.len(), "bi_end-bi_beg != line.len()");
            return Bytes::from(&(*(self.lineparts[0].blockp))[bi_beg..bi_end]);
        }
        // not efficient case, Line crosses stored Blocks so have to create a new vec
        let sz = self.len();
        assert_ne!(sz, 0, "self.len() is zero!?");
        let mut data = Bytes::with_capacity(sz);
        for lp in self.lineparts.iter() {
            let bi_beg = lp.blocki_beg;
            let bi_end = lp.blocki_end;
            data.extend_from_slice(&(*(lp.blockp))[bi_beg..bi_end]);
        }
        assert_eq!(data.len(), self.len(), "Line.as_bytes: data.len() != self.len()");
        return data;
    }

    /// do be do
    /// CANDIDATE FOR REMOVAL
    //pub fn as_vec(self: &Line, beg: LineIndex, end: LineIndex) -> Bytes {
    #[warn(unreachable_code)]
    pub fn as_vec(self: &Line, beg: LineIndex, end: LineIndex) -> Bytes {
        assert_gt!(self.lineparts.len(), 0, "This Line has no LineParts");
        // efficient case, Line does not cross any Blocks
        if self.lineparts.len() == 1 {
            //let bi_beg = self.lineparts[0].blocki_beg;
            //let bi_end = self.lineparts[0].blocki_end;
            assert_le!(end - beg, self.len(), "end-beg > line.len()");

            return Bytes::from(&(*(self.lineparts[0].blockp))[beg as usize..end as usize]);
        }
        unimplemented!("as_vec does not handle multiple lineparts");
        // XXX: incredibly inefficient case, Line crosses stored Blocks so have to create a new vec
        let sz = self.len();
        assert_ne!(sz, 0, "self.len() is zero!?");
        let mut data: Bytes = Bytes::with_capacity(sz);
        for lp in self.lineparts.iter() {
            let bi_beg = lp.blocki_beg;
            let bi_end = lp.blocki_end;
            data.extend_from_slice(&(*(lp.blockp))[bi_beg..bi_end]);
        }
        assert_eq!(data.len(), self.len(), "Line.as_vec: data.len() != self.len()");
        data
    }
}

type CharSz = usize;
/// thread-safe Atomic Reference Counting pointer to a `Line`
type LineP = Arc<Line>;
/// storage for Lines found from the underlying `BlockReader`
/// FileOffset key is the first byte/offset that begins the `Line`
type FoToLine = BTreeMap<FileOffset, LineP>;
type FoToFo = BTreeMap<FileOffset, FileOffset>;
/// Line Searching error
#[allow(non_camel_case_types)]
type ResultS4_LineFind = ResultS4<(FileOffset, LineP), Error>;
type LinesLRUCache = LruCache<FileOffset, ResultS4_LineFind>;
/// range map where key is Line begin to end `[Line.fileoffset_begin(), Line.fileoffset_end()]`
/// and where value is Line begin (`Line.fileoffset_begin()`). Use the value to lookup associated `Line` map
//type LinesRangeMap = RangeMap<FileOffset, FileOffset>;
//type LinesRangeSet = RangeSet<FileOffset>;
//type LinesRangeMap32 = cranelift_bforest::Set<FileOffset>;
//type LinesRangeMap32Mem = cranelift_bforest::SetForest<FileOffset>;
//type LinesRangeHVal = std::ops::Range<FileOffset>;
//type LinesRangeHSet = std::collections::hash_set::HashSet<LinesRangeHVal>;
//type LinesRMap = BTreeMap<FileOffset, FileOffset>;

/*
/// quickie helper to wrap common check, this could be more rustic O(n)
pub fn hashset_contains(set: &LinesRangeHSet, value: &FileOffset) -> bool {
    for r1 in set.iter() {
        if r1.contains(value) {
            return true;
        }
    }
    return false;
}

/// quickie helper to wrap common check, this could be more rustic O(n)
pub fn hashset_get(set: &LinesRangeHSet, value: &FileOffset) -> Option<LinesRangeHVal> {
    for r1 in set.iter() {
        if r1.contains(value) {
            return Some(r1.clone());
        }
    }
    return None;
}
*/

/*
/// quickie helper to wrap common check, this could be more rustic O(n)
pub fn hashset_contains(map: &LinesRMap, value: &FileOffset) -> bool {
    match map.range(value..).next() {
        Some(r1, r2) => {
            // XXX: now search for `.prev` entry... how to?
            return true;
        },
        _ => {
            return false;
        },
    }
}

/// quickie helper to wrap common check, this could be more rustic O(n)
pub fn hashset_get(map: &LinesRMap, value: &FileOffset) -> Option<FileOffset> {
    
    return None;
}
*/

/// Specialized Reader that uses BlockReader to find FoToLine
pub struct LineReader<'linereader> {
    blockreader: BlockReader<'linereader>,
    /// track `Line` found among blocks in `blockreader`, tracked by line beginning `FileOffset`
    /// key value `FileOffset` should agree with `(*LineP).fileoffset_begin()`
    pub lines: FoToLine,
    /// for all `Lines`, map `Line.fileoffset_end` to `Line.fileoffset_beg`
    foend_to_fobeg: FoToFo,
    /// count of `Line`s. Tracked outside of `self.lines.len()` as that may
    /// have contents removed when --streaming
    lines_count: u64,
    /// char size in bytes
    /// TODO: handle char sizes > 1 byte
    /// TODO: handle multi-byte encodings
    _charsz: CharSz,
    /// `Line` offsets stored as Range `[fileoffset_begin..fileoffset_end+1)`. to `fileoffset_begin`.
    ///  the stored value can be used to lookup `Line` in `self.lines`
    //lines_by_range: LinesRangeMap,
    //lines_by_range: LinesRangeSet,
    //lines_by_range32: LinesRangeMap32,
    //lines_by_range32mem: LinesRangeMap32Mem,
    //lines_by_range: LinesRangeHSet,
    //lines_by_range: LinesRMap,
    /// internal LRU cache for `find_line`
    _find_line_lru_cache: LinesLRUCache,
    // TODO: [2021/09/21] add efficiency stats
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
            .field("_charsz", &self._charsz)
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
    pub fn new(path: &'linereader FPath, blocksz: BlockSz) -> Result<LineReader<'linereader>> {
        // XXX: multi-byte
        assert_ge!(
            blocksz,
            (CHARSZ_MIN as BlockSz),
            "BlockSz {} is too small, must be greater than or equal {}",
            blocksz,
            CHARSZ_MAX
        );
        assert_ne!(blocksz, 0, "BlockSz is zero");
        let mut br = BlockReader::new(path, blocksz);
        match br.open() {
            Err(err) => {
                return Err(err);
            }
            Ok(_) => {}
        };
        Ok(LineReader {
            blockreader: br,
            lines: FoToLine::new(),
            foend_to_fobeg: FoToFo::new(),
            lines_count: 0,
            _charsz: CHARSZ,
            // give impossible value to start with
            //_next_line_blockoffset: FileOffset::MAX,
            //_next_line_blockp_opt: None,
            //_next_line_blocki: 0,
            //lines_by_range: LinesRangeMap::new(),
            //lines_by_range: LinesRangeSet::new(),
            //lines_by_range32: LinesRangeMap32::new(),
            //lines_by_range32mem: LinesRangeMap32Mem::new(),
            //lines_by_range: LinesRangeHSet::new(),
            //lines_by_range: LinesRMap::new(),
            _find_line_lru_cache: LinesLRUCache::new(8),
        })
    }

    /// smallest size character in bytes
    pub fn charsz(&self) -> usize {
        self._charsz
    }

    pub fn blocksz(&self) -> BlockSz {
        self.blockreader.blocksz
    }

    pub fn filesz(&self) -> BlockSz {
        self.blockreader.filesz
    }

    pub fn path(&self) -> &FPath {
        return &self.blockreader.path;
    }

    /// print `Line` at `fileoffset`
    /// return `false` if `fileoffset` not found
    #[cfg(any(debug_assertions,test))]
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
    #[cfg(any(debug_assertions,test))]
    pub fn print_all(&self) {
        for fo in self.lines.keys() {
            self.print(fo);
        }
    }

    /// count of lines processed by this LineReader
    pub fn count(&self) -> u64 {
        self.lines_count
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
    
    fn insert_line(&mut self, line: Line) -> LineP {
        debug_eprintln!("{}LineReader.insert_line(Line @{:p})", sn(), &line);
        let fo_beg = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let rl = LineP::new(line);
        debug_eprintln!("{}LineReader.insert_line: lines.insert({}, Line @{:p})", so(), fo_beg, &(*rl));
        debug_assert!(!self.lines.contains_key(&fo_beg), "self.lines already contains key {}", fo_beg);
        self.lines.insert(fo_beg, rl.clone());
        debug_eprintln!("{}LineReader.insert_line: foend_to_fobeg.insert({}, {})", so(), fo_end, fo_beg);
        debug_assert!(!self.foend_to_fobeg.contains_key(&fo_end), "self.foend_to_fobeg already contains key {}", fo_end);
        self.foend_to_fobeg.insert(fo_end, fo_beg);
        self.lines_count += 1;
        // XXX: multi-byte character encoding
        let fo_end1 = fo_end + (self.charsz() as FileOffset);
        // TODO: this `RangeMap::insert` takes a very large amount of processing time, 8% of processing time for the tests
        //       in script `./tools/compare-grep-sort1.sh`. In this special case, it case be replaced with
        //       a `RangeSet`. The `V` in this `RangeMap` use is the same as `K.last`.

        //debug_eprintln!("{}LineReader.insert_line: lines_by_range.insert({}‥{}, {})", so(), fo_beg, fo_end1, fo_beg);
        //debug_assert!(!self.lines_by_range.contains_key(&fo_beg), "self.lines_by_range already contains range with fo_beg {}", fo_beg);
        //debug_assert!(!self.lines_by_range.contains_key(&fo_end), "self.lines_by_range already contains range with fo_end {}", fo_end);
        //self.lines_by_range.insert(fo_beg..fo_end1, fo_beg);

        //debug_eprintln!("{}LineReader.insert_line: lines_by_range.insert({}‥{})", so(), fo_beg, fo_end1);
        //debug_assert!(!self.lines_by_range.contains(&fo_beg), "self.lines_by_range already contains range with fo_beg {}", fo_beg);
        //debug_assert!(!self.lines_by_range.contains(&fo_end1), "self.lines_by_range already contains range with fo_end1 {}", fo_end1);
        //self.lines_by_range.insert(fo_beg..fo_end1);

        //debug_eprintln!("{}LineReader.insert_line: lines_by_range.insert({}‥{})", so(), fo_beg, fo_end1);
        //debug_assert!(!hashset_contains(&self.lines_by_range, &fo_beg), "self.lines_by_range already contains range with fo_beg {}", fo_beg);
        //debug_assert!(!hashset_contains(&self.lines_by_range, &fo_end), "self.lines_by_range already contains range with fo_end {}", fo_end);
        //self.lines_by_range.insert(fo_beg..fo_end1);

        debug_eprintln!("{}LineReader.insert_line() returning @{:p}", sx(), rl);
        return rl;
    }

    /// does `self` "contain" this `fileoffset`? That is, already know about it?
    /// the `fileoffset` can be any value (does not have to be begining or ending of
    /// a `Line`).
    fn lines_contains(&self, fileoffset: &FileOffset) -> bool {
        let fo_beg = match self.foend_to_fobeg.range(fileoffset..).next() {
            Some((_, fo_beg_)) => {
                fo_beg_
            },
            None => { return false; },
        };
        if fileoffset < fo_beg {
            return false;
        }
        self.lines.contains_key(&fo_beg)
    }

    /// for any `FileOffset`, get the `Line` (if available)
    /// The passed `FileOffset` can be any value (does not have to be begining or ending of
    /// a `Line`).
    fn get_linep(&self, fileoffset: &FileOffset) -> Option<LineP> {
        let fo_beg = match self.foend_to_fobeg.range(fileoffset..).next() {
            Some((_, fo_beg_)) => {
                fo_beg_
            },
            None => { return None; },
        };
        if fileoffset < fo_beg {
            return None;
        }
        match self.lines.get(&fo_beg) {
            Some(slp) => { Some(slp.clone()) }
            None => { None }
        }
    }

    /// find next `Line` starting from `fileoffset`
    /// in the process of finding, creates and stores the `Line` from underlying `Block` data
    /// returns `Found`(`FileOffset` of beginning of the _next_ line, found `LineP`)
    /// reaching end of file (and no new line) returns `Found_EOF`
    /// reaching end of file returns `FileOffset` value that is one byte past the actual end of file (and should not be used)
    /// otherwise `Err`, all other `Result::Err` errors are propagated
    /// 
    /// similar to `find_sysline`, `read_block`
    ///
    /// XXX: presumes single-byte to one '\n', does not handle UTF-16 or UTF-32 or other (`charsz` hardcoded to 1)
    /// TODO: [2021/08/30] handle different encodings
    /// XXX: this function is fragile and cumbersome, any tweaks require extensive retesting
    pub fn find_line(&mut self, fileoffset: FileOffset) -> ResultS4_LineFind {
        debug_eprintln!("{}find_line(LineReader@{:p}, {})", sn(), self, fileoffset);

        // some helpful constants
        let charsz_fo = self._charsz as FileOffset;
        let charsz_bi = self._charsz as BlockIndex;
        let filesz = self.filesz();
        let blockoffset_last = self.blockoffset_last();

        // check LRU cache first (this is very fast)
        match self._find_line_lru_cache.get(&fileoffset) {
            Some(rlp) => {
                // self.stats_read_block_cache_lru_hit += 1;
                debug_eprint!("{}find_line: found LRU cached for offset {}", sx(), fileoffset);
                match rlp {
                    ResultS4_LineFind::Found(val) => {
                        debug_eprintln!(" return ResultS4_LineFind::Found(({}, …)) @[{}, {}]", val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                        return ResultS4_LineFind::Found((val.0, val.1.clone()));
                    }
                    ResultS4_LineFind::Found_EOF(val) => {
                        debug_eprintln!(" return ResultS4_LineFind::Found_EOF(({}, …)) @[{}, {}]", val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                        return ResultS4_LineFind::Found_EOF((val.0, val.1.clone()));
                    }
                    ResultS4_LineFind::Done => {
                        debug_eprintln!(" return ResultS4_LineFind::Done");
                        return ResultS4_LineFind::Done;
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
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Done; file is empty", sx());
            return ResultS4_LineFind::Done;
        } else if fileoffset > filesz {
            // TODO: [2021/10] need to decide on consistent behavior for passing fileoffset > filesz
            //       should it really Error or be Done?
            //       Make that consisetent among all LineReader and SyslineReader `find_*` functions
            /*
            let err = Error::new(
                ErrorKind::AddrNotAvailable,
                format!("Passed fileoffset {} past file size {}", fileoffset, filesz),
            );
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Err({}); fileoffset {} was too big filesz {}!", sx(), err, fileoffset, filesz);
            return ResultS4_LineFind::Err(err);
            */
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Done; fileoffset {} was too big filesz {}!", sx(), fileoffset, filesz);
            return ResultS4_LineFind::Done;
        } else if fileoffset == filesz {
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Done(); fileoffset {} is at end of file {}!", sx(), fileoffset, filesz);
            return ResultS4_LineFind::Done;
        }

        {
            // first check if there is a `Line` already known at this fileoffset
            if self.lines.contains_key(&fileoffset) {
                debug_eprintln!("{}find_line: hit cache for FileOffset {}", so(), fileoffset);

                //debug_assert!(self.lines_by_range.contains_key(&fileoffset), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fileoffset);
                //debug_assert!(self.lines_by_range.contains(&fileoffset), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fileoffset);
                //debug_assert!(hashset_contains(&self.lines_by_range, &fileoffset), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fileoffset);
                debug_assert!(self.lines_contains(&fileoffset), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fileoffset);

                let lp = self.lines[&fileoffset].clone();
                let fo_next = (*lp).fileoffset_end() + charsz_fo;
                // TODO: add stats like BlockReader._stats*
                debug_eprintln!("{}find_line: LRU Cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_next);
                self._find_line_lru_cache
                    .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
                debug_eprintln!("{}find_line: return ResultS4_LineFind::Found({}, {:p})  @[{}, {}]", sx(), fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end());
                return ResultS4_LineFind::Found((fo_next, lp));
            }
            //match self.lines_by_range.get(&fileoffset) {
            //match hashset_get(&self.lines_by_range, &fileoffset) {
            match self.get_linep(&fileoffset) {
                //Some(fo_range) => {
                Some(lp) => {
                    debug_eprintln!(
                        "{}find_line: self.get_linep({}) returned @{:p}",
                        so(),
                        fileoffset,
                        lp
                    );
                    //debug_eprintln!(
                    //    "{}find_line: fileoffset {} refers to self.lines_by_range {:?}",
                    //    so(),
                    //    fileoffset,
                    //    fo_range
                    //);
                    //let lp = self.lines[fo_range].clone();
                    //let lp = self.lines[&fo_range.start].clone();
                    let fo_next = (*lp).fileoffset_end() + charsz_fo;
                    // TODO: add stats like BlockReader._stats*
                    debug_eprintln!("{}find_line: LRU Cache put({}, Found({}, …)) {:?}", so(), fileoffset, fo_next, (*lp).to_String_noraw());
                    self._find_line_lru_cache
                        .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
                    debug_eprintln!("{}find_line: return ResultS4_LineFind::Found({}, {:p}) @[{}, {}]", sx(), fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end());
                    return ResultS4_LineFind::Found((fo_next, lp));
                }
                None => {
                    //self.stats_read_block_cache_lru_miss += 1;
                    debug_eprintln!("{}find_line: fileoffset {} not found in self.lines_by_range", so(), fileoffset);
                }
            }
            debug_eprintln!("{}find_line: fileoffset {} not found in self.lines", so(), fileoffset);
            debug_eprintln!("{}find_line: searching for first newline newline A …", so());
        }

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
            // XXX: single-byte encoding, does not handle multi-byte
            let fo1 = fileoffset - charsz_fo;
            if self.foend_to_fobeg.contains_key(&fo1) {
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
            debug_eprintln!("{}find_line: self.blockreader.read_block({})", so(), bo);
            match self.blockreader.read_block(bo) {
                Ok(val) => {
                    debug_eprintln!(
                        "{}find_line: read_block returned Block @{:p} len {} while searching for newline A",
                        so(),
                        &(*val),
                        (*val).len()
                    );
                    bp = val;
                }
                Err(err) => {
                    if err.kind() == EndOfFile {
                        debug_eprintln!("{}find_line: read_block returned EndOfFile {:?} searching for found_nl_a failed (IS THIS AN ERROR???????)", so(), self.path());
                        // reached end of file, no beginning newlines found
                        // TODO: Is this an error state? should this be handled differently?
                        debug_eprintln!("{}find_line: return ResultS4_LineFind::Done; EOF from read_block; NOT SURE IF THIS IS CORRECT", sx());
                        return ResultS4_LineFind::Done;
                    }
                    debug_eprintln!("{}find_line: LRU cache put({}, Done)", so(), fileoffset);
                    self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Done);
                    debug_eprintln!("{}find_line: return ResultS4_LineFind::Done; NOT SURE IF THIS IS CORRECT!!!!", sx());
                    return ResultS4_LineFind::Done;
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
                        "{}find_line: found newline A from byte search at fileoffset {} ≟ blockoffset {} blockindex {}",
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
            debug_eprintln!("{}find_line: LRU Cache put({}, Done)", so(), fileoffset);
            self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Done);
            // the last character in the file is a newline
            // XXX: is this correct?
            debug_eprintln!(
                "{}find_line: return ResultS4_LineFind::Done; newline A is at last char in file {}, not a line IS THIS CORRECT?",
                sx(),
                filesz - 1
            );
            return ResultS4_LineFind::Done;
        }

        //
        // walk through blocks and bytes looking for ending of line (a newline character; part B)
        //
        debug_eprintln!(
            "{}find_line: found first newline A, searching for second B newline starting at {} …",
            so(),
            fo_nl_a
        );

        {
            // …but before doing work of discovering a new `Line` (part B), first checks various
            // maps in `self` to see if this `Line` has already been discovered and processed
            if self.lines.contains_key(&fo_nl_a) {
                debug_eprintln!("{}find_line: hit for self.lines for FileOffset {} (before part B)", so(), fo_nl_a);

                //debug_assert!(self.lines_by_range.contains_key(&fo_nl_a), "self.lines and self.lines_by_range are out of synch on key {}", fo_nl_a);
                //debug_assert!(self.lines_by_range.contains(&fo_nl_a), "self.lines and self.lines_by_range are out of synch on key {}", fo_nl_a);
                //debug_assert!(hashset_contains(&self.lines_by_range, &fo_nl_a), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fo_nl_a);
                debug_assert!(self.lines_contains(&fo_nl_a), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fo_nl_a);

                let lp = self.lines[&fo_nl_a].clone();
                let fo_next = (*lp).fileoffset_end() + charsz_fo;
                // TODO: add stats like BlockReader._stats*
                debug_eprintln!("{}find_line: LRU Cache put({}, Found_EOF({}, …))", so(), fo_nl_a, fo_next);
                self._find_line_lru_cache
                    .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
                debug_eprintln!("{}find_line: return ResultS4_LineFind::Found({}, {:p})  @[{}, {}]", sx(), fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end());
                return ResultS4_LineFind::Found((fo_next, lp));
            }
            match self.get_linep(&fo_nl_a) {
            //match hashset_get(&self.lines_by_range, &fileoffset) {
                //Some(fo_range) => {
                Some(lp) => {
                    debug_eprintln!(
                        "{}find_line: self.get_linep({}) returned {:p}",
                        so(),
                        fo_nl_a,
                        lp
                    );
                    //debug_eprintln!(
                    //    "{}find_line: fo_nl_a {} refers to self.lines_by_range {:?}",
                    //    so(),
                    //    fo_nl_a,
                    //    fo_range
                    //);
                    //let lp = self.lines[fo_range].clone();
                    //let lp = self.lines[&fo_range.start].clone();
                    let fo_next = (*lp).fileoffset_end() + charsz_fo;
                    // TODO: add stats like BlockReader._stats*
                    debug_eprintln!("{}find_line: LRU Cache put({}, Found({}, …)) {:?}", so(), fo_nl_a, fo_next, (*lp).to_String_noraw());
                    self._find_line_lru_cache
                        .put(fo_nl_a, ResultS4_LineFind::Found((fo_next, lp.clone())));
                    debug_eprintln!("{}find_line: return ResultS4_LineFind::Found({}, {:p}) @[{}, {}]", sx(), fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end());
                    return ResultS4_LineFind::Found((fo_next, lp));
                }
                None => {
                    //self.stats_read_block_cache_lru_miss += 1;
                    debug_eprintln!("{}find_line: fileoffset {} not found in self.lines_by_range", so(), fo_nl_a);
                }
            }
        }

        // getting here means this function is discovering a brand new `Line` (part B)

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
            debug_eprintln!("{}find_line: self.blockreader.read_block({})", so(), bo);
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
                        debug_eprintln!("{}find_line: LRU Cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_);
                        self._find_line_lru_cache
                            .put(fileoffset, ResultS4_LineFind::Found_EOF((fo_, rl.clone())));
                        debug_eprintln!(
                            "{}find_line: return ResultS4_LineFind::Found_EOF(({}, {:p})) @[{} , {}]; {:?}",
                            sx(),
                            fo_,
                            &*rl,
                            (*rl).fileoffset_begin(),
                            (*rl).fileoffset_end(),
                            (*rl).to_String_noraw()
                        );
                        return ResultS4_LineFind::Found_EOF((fo_, rl));
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

        // may occur in files ending on a single newline
        if line.count() == 0 {
            debug_eprintln!("{}find_line: LRU Cache put({}, Done)", so(), fileoffset);
            self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Done);
            debug_eprintln!("{}find_line: return ResultS4_LineFind::Done;", sx());
            return ResultS4_LineFind::Done;
        }

        // sanity check
        debug_eprintln!("{}find_line: return {:?};", so(), line);
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
        debug_eprintln!("{}find_line: LRU Cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_end + 1);
        self._find_line_lru_cache
            .put(fileoffset, ResultS4_LineFind::Found((fo_end + 1, rl.clone())));
        debug_eprintln!(
            "{}find_line: return ResultS4_LineFind::Found(({}, @{:p})) @[{}, {}]; {:?}",
            sx(),
            fo_end + 1,
            &*rl,
            (*rl).fileoffset_begin(),
            (*rl).fileoffset_end(),
            (*rl).to_String_noraw()
        );
        return ResultS4_LineFind::Found((fo_end + 1, rl));
    }
}

/// loop on `LineReader.find_line` until it is done
/// prints to stdout
/// testing helper
#[cfg(any(debug_assertions,test))]
fn process_LineReader(lr1: &mut LineReader) {
    debug_eprintln!("{}process_LineReader()", sn());
    let mut fo1: FileOffset = 0;
    loop {
        debug_eprintln!("{}LineReader.find_line({})", so(), fo1);
        let result = lr1.find_line(fo1);
        match result {
            ResultS4_LineFind::Found((fo, lp)) => {
                let _ln = lr1.count();
                debug_eprintln!(
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
                            eprintln!("ERROR: print_colored_stdout returned error {}", err);
                        }
                    }
                } else {
                    (*lp).print(true);
                }
            }
            ResultS4_LineFind::Found_EOF((fo, lp)) => {
                let _ln = lr1.count();
                debug_eprintln!(
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
                debug_eprintln!("{}ResultS4_LineFind::Done!", so());
                break;
            }
            ResultS4_LineFind::Err(err) => {
                debug_eprintln!("{}ResultS4_LineFind::Err {}", so(), err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
    }
    debug_eprintln!("{}process_LineReader()", sx());
}

/// basic test of LineReader things with premade tests
/// simple read of file offsets in order, should print to stdout an identical file
#[allow(non_snake_case)]
#[test]
fn test_LineReader_1() {
    debug_eprintln!("{}test_LineReader_1()", sn());

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
                eprintln!("ERROR: LineReader::new({:?}, {}) failed {}", path, blocksz, err);
                return;
            }
        };
        let bufnoraw = buffer_to_String_noraw(content.as_bytes());
        debug_eprintln!("{}File {:?}", so(), bufnoraw);
        process_LineReader(&mut lr1);
        let lc = lr1.count();
        assert_eq!(line_count, lc, "Expected {} count of lines, found {}", line_count, lc);
        match print_colored_stdout(
            Color::Green,
            format!("{}PASS Found {} Lines as expected from {:?}\n", so(), lc, bufnoraw).as_bytes(),
        ) { Ok(_) => {}, Err(_) => {}, };
        debug_eprintln!("{}{:?}", so(), content.as_bytes());
    }
    debug_eprintln!("{}test_LineReader_1()", sx());
}

/// basic test of LineReader things using user passed file
/// simple read of file offsets in order, should print to stdout an identical file
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_LineReader(path_: &FPath, blocksz: BlockSz) {
    debug_eprintln!("{}test_LineReader({:?}, {})", sn(), &path_, blocksz);
    let mut lr1 = match LineReader::new(path_, blocksz) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: LineReader::new({}, {}) failed {}", path_, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}LineReader {:?}", so(), lr1);

    process_LineReader(&mut lr1);
    //debug_eprintln!("\n{}{:?}", so(), lr1);

    if cfg!(debug_assertions) {
        debug_eprintln!("{}Found {} Lines", so(), lr1.count())
    }
    debug_eprintln!("{}test_LineReader({:?}, {})", sx(), &path_, blocksz);
}

/// testing helper
#[cfg(test)]
fn randomize(v_: &mut Vec<FileOffset>) {
    // XXX: can also use `rand::shuffle` https://docs.rs/rand/0.8.4/rand/seq/trait.SliceRandom.html#tymethod.shuffle
    let sz = v_.len();
    let mut i = 0;
    while i < sz {
        let r = rand::random::<usize>() % sz;
        v_.swap(r, i);
        i += 1;
    }
}

/// testing helper
#[cfg(test)]
fn fill(v_: &mut Vec<FileOffset>) {
    let sz = v_.capacity();
    let mut i = 0;
    while i < sz {
        v_.push(i as FileOffset);
        i += 1;
    }
}

/// basic test of LineReader things using user passed file
/// read all file offsets but randomly
#[allow(non_snake_case)]
#[cfg(test)]
fn test_LineReader_rand(path_: &FPath, blocksz: BlockSz) {
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
            ResultS4_LineFind::Found((fo, lp)) => {
                let _ln = lr1.count();
                debug_eprintln!(
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
                debug_eprintln!(
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
                debug_eprintln!("{}ResultS4_LineFind::Done!", so());
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

// TODO: add tests for `test_LineReader_rand`

#[cfg(test)]
type test_Line_get_boxptrs_check = Vec<(FileOffset, (LineIndex, LineIndex), Bytes)>;

/// test `Line.get_boxpts`
#[cfg(test)]
fn _test_Line_get_boxptrs(fpath: &FPath, blocksz: BlockSz, checks: &test_Line_get_boxptrs_check) {
    debug_eprintln!("{}_test_Line_get_boxptrs({:?}, {}, checks)", sn(), fpath, blocksz);
    // create a `LineReader` and read all the lines in the file
    let mut lr = LineReader::new(fpath, blocksz).unwrap();
    let mut done = false;
    let mut fo: FileOffset = 0;
    while !done {
        match lr.find_line(fo) {
            ResultS4_LineFind::Found((fo_, linep)) => {
                fo = fo_;
            },
            ResultS4_LineFind::Found_EOF((fo_, linep)) => {
                fo = fo_;
            },
            ResultS4_LineFind::Done => {
                break;
            },
            ResultS4_LineFind::Err(err) => {
                assert!(false, "ResultS4_LineFind::Err {}", err);
            },
        }
    }

    // then test the `Line.get_boxptrs`
    // get_boxptrs(self: &Line, a: LineIndex, mut b: LineIndex) -> Vec<Box<&[u8]>>
    for (linenum, (a, b), bytes_check) in checks.iter() {
        assert_lt!(a, b, "bad check args a {} b {}", a, b);
        assert_ge!(b-a, bytes_check.len(), "Bad check args ({}-{})={} < {} bytes_check.len()", b, a, b-a, bytes_check.len());
        debug_eprintln!("{}_test_Line_get_boxptrs: linereader.get_linep({})", so(), linenum);
        let line = lr.get_linep(linenum).unwrap();
        debug_eprintln!("{}_test_Line_get_boxptrs: returned {:?}", so(), line.to_String_noraw());
        debug_eprintln!("{}_test_Line_get_boxptrs: line.get_boxptrs({}, {})", so(), a, b);
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
                debug_eprintln!("{}_test_Line_get_boxptrs: {:3?} ≟ {:3?} ({:?} ≟ {:?})", so(), byte_, byte_check, byte_to_char_noraw(*byte_), byte_to_char_noraw(*byte_check));
                assert_eq!(byte_, byte_check, "byte {} from boxptr {:?} ≠ {:?} ({:?} ≠ {:?}) check value; returned boxptr segement {:?} Line {:?}", at, byte_, byte_check, byte_to_char_noraw(*byte_), byte_to_char_noraw(*byte_check), buffer_to_String_noraw(&(*boxptr)), line.to_String_noraw());
                at += 1;
            }
        }
    }
    debug_eprintln!("{}_test_Line_get_boxptrs", sx());
}

#[test]
fn test_Line_get_boxptrs_1() {
    let data: &str = &"\
this is line 1";
    let ntf = create_temp_file(data);
    let mut checks: test_Line_get_boxptrs_check = test_Line_get_boxptrs_check::new();
    checks.push((0, (0, 1), vec![b't']));
    let fpath = FPath::from(ntf.path().to_str().unwrap());
    _test_Line_get_boxptrs(&fpath, 0xFF, &checks);
}

#[cfg(test)]
fn _test_Line_get_boxptrs_2_(blocksz: BlockSz) {
    debug_eprintln!("{}_test_Line_get_boxptrs_2_({:?})", sn(), blocksz);
    let data: &str = &"\
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
    debug_eprintln!("{}_test_Line_get_boxptrs_2_({:?})", sx(), blocksz);
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sysline
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A sequence to track one or more `LineP` that make up a `Sysline` 
type Lines = Vec<LineP>;
/// An offset into a `Line`
type LineIndex = usize;

// DateTime typing
/// typical DateTime with TZ type
//type DateTimeL = DateTime<Local>;
type DateTimeL = DateTime<FixedOffset>;
#[allow(non_camel_case_types)]
type DateTimeL_Opt = Option<DateTimeL>;
/// Sysline Searching error
/// TODO: does SyslineFind need an `Found_EOF` state? Is it an unnecessary overlap of `Ok` and `Done`?
#[allow(non_camel_case_types)]
type ResultS4_SyslineFind = ResultS4<(FileOffset, SyslineP), Error>;

/// A `Sysline` has information about a "syslog line" that spans one or more `Line`s
/// a "syslog line" is one or more lines, where the first line starts with a
/// datetime stamp. That datetime stamp is consistent format of other nearby syslines.
pub struct Sysline {
    /// the one or more `Line` that make up a Sysline
    lines: Lines,
    /// index into `Line` where datetime string starts
    /// byte-based count
    /// datetime is presumed to be on first Line
    dt_beg: LineIndex,
    /// index into `Line` where datetime string ends, one char past last character of datetime string
    /// byte-based count
    /// datetime is presumed to be on first Line
    dt_end: LineIndex,
    /// parsed DateTime instance
    dt: DateTimeL_Opt,
}

/// a signifier value for "not set" or "null" - because sometimes Option is a PitA
const LI_NULL: LineIndex = LineIndex::MAX;

impl fmt::Debug for Sysline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut li_s = String::new();
        for lp in self.lines.iter() {
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
            .field("lines @", &format_args!("{:p}", &self.lines))
            .field("lines.len", &self.lines.len())
            .field("dt_beg", &self.dt_beg)
            .field("dt_end", &self.dt_end)
            .field("dt", &self.dt)
            .field("lines", &li_s)
            .finish()
    }
}

impl Sysline {
    /// default `with_capacity` for a `Lines`, most often will only need 1 capacity
    /// as the found "sysline" will likely be one `Line`
    const SYSLINE_PARTS_WITH_CAPACITY: usize = 1;
    // XXX: does not handle multi-byte encodings
    const CHARSZ: usize = 1;

    pub fn new() -> Sysline {
        return Sysline {
            lines: Lines::with_capacity(Sysline::SYSLINE_PARTS_WITH_CAPACITY),
            dt_beg: LI_NULL,
            dt_end: LI_NULL,
            dt: None,
        };
    }

    pub fn new_from_line(linep: LineP) -> Sysline {
        let mut v = Lines::with_capacity(Sysline::SYSLINE_PARTS_WITH_CAPACITY);
        v.push(linep);
        return Sysline {
            lines: v,
            dt_beg: LI_NULL,
            dt_end: LI_NULL,
            dt: None,
        };
    }

    pub fn charsz(self: &Sysline) -> usize {
        Sysline::CHARSZ
    }

    pub fn push(&mut self, linep: LineP) {
        if !self.lines.is_empty() {
            // TODO: sanity check lines are in sequence
        }
        debug_eprintln!(
            "{}SyslineReader.push(@{:p}), self.lines.len() is now {}",
            so(),
            &*linep,
            self.lines.len() + 1
        );
        self.lines.push(linep);
    }

    /// the byte offset into the file where this `Sysline` begins
    /// "points" to first character of `Sysline` (and underlying `Line`)
    pub fn fileoffset_begin(self: &Sysline) -> FileOffset {
        assert_ne!(self.lines.len(), 0, "This Sysline has no Line");
        (*self.lines[0]).fileoffset_begin()
    }

    /// the byte offset into the file where this `Sysline` ends, inclusive (not one past ending)
    pub fn fileoffset_end(self: &Sysline) -> FileOffset {
        assert_ne!(self.lines.len(), 0, "This Sysline has no Line");
        let last_ = self.lines.len() - 1;
        (*self.lines[last_]).fileoffset_end()
    }

    /// the fileoffset into the next sysline
    /// this Sysline does not know if that fileoffset points to the end of file (one past last actual byte)
    pub fn fileoffset_next(self: &Sysline) -> FileOffset {
        self.fileoffset_end() + (self.charsz() as FileOffset)
    }

    /// length in bytes of this Sysline
    pub fn len(self: &Sysline) -> usize {
        (self.fileoffset_end() - self.fileoffset_begin() + 1) as usize
    }

    /// count of `Line` in `self.lines`
    pub fn count(self: &Sysline) -> u64 {
        self.lines.len() as u64
    }

    /// sum of `Line.count_bytes`
    pub fn count_bytes(self: &Sysline) -> u64 {
        let mut cb = 0;
        for ln in self.lines.iter() {
            cb += ln.count_bytes();
        }
        cb
    }

    /// a `String` copy of the demarcating datetime string found in the Sysline
    #[allow(non_snake_case)]
    pub fn datetime_String(self: &Sysline) -> String {
        assert_ne!(self.dt_beg, LI_NULL, "dt_beg has not been set");
        assert_ne!(self.dt_end, LI_NULL, "dt_end has not been set");
        assert_lt!(self.dt_beg, self.dt_end, "bad values dt_end {} dt_beg {}", self.dt_end, self.dt_beg);
        let slice_ = self.lines[0].as_bytes();
        assert_lt!(
            self.dt_beg,
            slice_.len(),
            "dt_beg {} past end of slice[{}‥{}]?",
            self.dt_beg,
            self.dt_beg,
            self.dt_end
        );
        assert_le!(
            self.dt_end,
            slice_.len(),
            "dt_end {} past end of slice[{}‥{}]?",
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
    /// TODO: make an iterable trait of this struct
    pub fn get_slices(self: &Sysline) -> Slices {
        let mut sz: usize = 0;
        for lp in &self.lines {
            sz += lp.get_slices_count();
        }
        let mut slices = Slices::with_capacity(sz);
        for lp in &self.lines {
            slices.extend(lp.get_slices().iter());
        }
        slices
    }

    /// print approach #1, use underlying `Line` to `print`
    /// `raw` true will write directly to stdout from the stored `Block`
    /// `raw` false will write transcode each byte to a character and use pictoral representations
    /// XXX: `raw==false` does not handle multi-byte encodings
    /// TODO: move this into a `Printer` class
    #[cfg(any(debug_assertions,test))]
    pub fn print1(self: &Sysline, raw: bool) {
        for lp in &self.lines {
            (*lp).print(raw);
        }
    }

    // TODO: [2022/03/23] implement an `iter_slices` that does not require creating a new `vec`, just
    //       passes `&bytes` back. Call `iter_slices` from `print`

    /// print approach #2, print by slices
    /// prints raw data from underlying `Block`
    /// testing helper
    /// TODO: move this into a `Printer` class
    #[cfg(any(debug_assertions,test))]
    fn print2(&self) {
        let slices = self.get_slices();
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();
        for slice in slices.iter() {
            match stdout_lock.write(slice) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: write: StdoutLock.write(slice@{:p} (len {})) error {}", slice, slice.len(), err);
                }
            }
        }
        match stdout_lock.flush() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: write: stdout flushing error {}", err);
            }
        }
    }

    /// helper to `print_color`
    /// caller must acquire stdout.Lock, and call `stdout.flush()`
    /// TODO: move this into a `Printer` class
    fn print_color_slices(stdclr: &mut termcolor::StandardStream, colors: &[Color], values:&[&[u8]]) -> Result<()> {
        assert_eq!(colors.len(), values.len());
        for (color, value) in colors.iter().zip(values.iter())
        {
            match stdclr.set_color(ColorSpec::new().set_fg(Some(color.clone()))) {
                Err(err) => {
                    eprintln!("ERROR: print_color_slices: stdout.set_color({:?}) returned error {}", color, err);
                    //continue;
                    return Err(err);
                },
                _ => {},
            };
            match stdclr.write(value) {
                Err(err) => {
                    eprintln!("ERROR: print_color_slices: stdout.write(…) returned error {}", err);
                    //continue;
                    return Err(err);
                }
                _ => {},
            }
        }
        Ok(())
    }

    /// print with color
    /// prints raw data from underlying `Block` bytes
    /// XXX: does not handle multi-byte strings
    /// TODO: needs a global mutex
    /// TODO: move this into a `Printer` class
    pub fn print_color(&self, color_text: Color, color_datetime: Color) -> Result<()> {
        let slices = self.get_slices();
        //let mut stdout = io::stdout();
        //let mut stdout_lock = stdout.lock();
        let mut choice: termcolor::ColorChoice = termcolor::ColorChoice::Never;
        if atty::is(atty::Stream::Stdout) || cfg!(debug_assertions) {
            choice = termcolor::ColorChoice::Always;
        }
        let mut clrout = termcolor::StandardStream::stdout(choice);
        let mut at: LineIndex = 0;
        let dtb = self.dt_beg;
        let dte = self.dt_end;
        for slice in slices.iter() {
            let len_ = slice.len();
            // datetime entirely in this `slice`
            if chmp!(at <= dtb < dte < (at + len_)) {
                let a = &slice[..(dtb-at)];
                let b = &slice[(dtb-at)..(dte-at)];
                let c = &slice[(dte-at)..];
                match Sysline::print_color_slices(&mut clrout, &[color_text, color_datetime, color_text], &[a, b, c]) {
                    Ok(_) => {},
                    Err(err) => {
                        return Err(err);
                    }
                };
            } // XXX: incomplete datetime crosses into next slice
            else {
                match Sysline::print_color_slices(&mut clrout, &[color_text], &[slice]) {
                    Ok(_) => {},
                    Err(err) => {
                        return Err(err);
                    }
                };
            }
            at += len_;
        }
        let mut ret = Ok(());
        match clrout.flush() {
            Err(err) => {
                eprintln!("ERROR: write: stdout flushing error {}", err);
                ret = Err(err);
            },
            _ => {},
        }
        match clrout.reset() {
            Err(err) => {
                eprintln!("print_colored: stdout.reset() returned error {}", err);
                return Err(err);
            },
            _ => {},
        }
        ret
    }

    /// create `String` from `self.lines`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_char_noraw`)
    /// TODO: this would be more efficient returning `&str`
    ///       https://bes.github.io/blog/rust-strings
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    fn _to_String_raw(self: &Sysline, raw: bool) -> String {
        let mut sz: usize = 0;
        for lp in &self.lines {
            sz += (*lp).len();
        }
        // XXX: intermixing byte lengths and character lengths
        // XXX: does not handle multi-byte
        let mut s_ = String::with_capacity(sz + 1);
        for lp in &self.lines {
            s_ += (*lp)._to_String_raw(raw).as_str();
        }
        return s_;
    }

    /*
    /// create `str` from `self.lines`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_char_noraw`)
    /// TODO: can this be more efficient? specialized for `str`?
    #[allow(non_snake_case)]
    fn _to_str_raw(self: &Sysline, raw: bool) -> &str {
        return (&self._to_String_raw(raw)).as_str();
    }
     */

    // XXX: rust does not support function overloading which is really surprising and disappointing
    /// `Line` to `String`
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &Sysline) -> String {
        self._to_String_raw(true)
    }

    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_from(self: &Sysline, _from: usize) -> String {
        unimplemented!("yep");
    }

    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_from_to(self: &Sysline, _from: usize, _to: usize) -> String {
        unimplemented!("yep");
    }

    /// `Sysline` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &Sysline) -> String {
        self._to_String_raw(false)
    }

    #[allow(non_snake_case)]
    #[cfg(not(any(debug_assertions,test)))]
    pub fn to_String_noraw(self: &Sysline) -> String {
        panic!("should not call function 'Sysline::to_String_noraw' in release build");
        String::from("")
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DateTime typing, strings, and formatting
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// DateTime typing

/// DateTime formatting pattern, passed to `chrono::datetime_from_str`
type DateTimePattern = String;
type DateTimePattern_str = str;
/// DateTimePattern for searching a line (not the results)
/// slice index begin, slice index end of entire datetime pattern
/// slice index begin just the datetime, slice index end just the datetime
/// TODO: why not define as a `struct` instead of a tuple?
/// TODO: why not use `String` type for the datetime pattern? I don't recall why I chose `str`.
/// TODO: instead of `LineIndex, LineIndex`, use `(RangeInclusive, Offset)` for the two pairs of LineIndex ranges
///       processing functions would attempt all values within `RangeInclusive` (plus the `Offset`).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub struct DateTime_Parse_Data {
    pattern: DateTimePattern,
    /// does the `pattern` have a year? ("%Y", "%y")
    year: bool,
    /// does the `pattern` have a timezone? ("%z", "%Z", etc.)
    tz: bool,
    /// slice index begin of entire pattern
    sib: LineIndex,
    /// slice index end of entire pattern
    sie: LineIndex,
    /// slice index begin of only datetime portion of pattern
    siba: LineIndex,
    /// slice index end of only datetime portion of pattern
    siea: LineIndex,
}
//type DateTime_Parse_Data = (DateTimePattern, bool, LineIndex, LineIndex, LineIndex, LineIndex);
type DateTime_Parse_Data_str<'a> = (&'a DateTimePattern_str, bool, bool, LineIndex, LineIndex, LineIndex, LineIndex);
//type DateTime_Parse_Datas_ar<'a> = [DateTime_Parse_Data<'a>];
type DateTime_Parse_Datas_vec = Vec<DateTime_Parse_Data>;
//type DateTime_Parse_Data_BoxP<'syslinereader> = Box<&'syslinereader DateTime_Parse_Data>;
/// count of datetime format strings used
// TODO: how to do this a bit more efficiently, and not store an entire copy?
type DateTime_Pattern_Counts = HashMap<DateTime_Parse_Data, u64>;
/// return type for `SyslineReader::find_datetime_in_line`
type Result_FindDateTime = Result<(DateTime_Parse_Data, DateTimeL)>;
/// return type for `SyslineReader::parse_datetime_in_line`
type Result_ParseDateTime = Result<(LineIndex, LineIndex, DateTimeL)>;
/// used internally by `SyslineReader`
type SyslinesLRUCache = LruCache<FileOffset, ResultS4_SyslineFind>;

/// describe the result of comparing one DateTime to one DateTime Filter
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Result_Filter_DateTime1 {
    Pass,
    OccursAtOrAfter,
    OccursBefore,
}

impl Result_Filter_DateTime1 {
    /// Returns `true` if the result is [`OccursAfter`].
    #[inline(always)]
    pub const fn is_after(&self) -> bool {
        matches!(*self, Result_Filter_DateTime1::OccursAtOrAfter)
    }

    /// Returns `true` if the result is [`OccursBefore`].
    #[inline(always)]
    pub const fn is_before(&self) -> bool {
        matches!(*self, Result_Filter_DateTime1::OccursBefore)
    }
}

/// describe the result of comparing one DateTime to two DateTime Filters
/// `(after, before)`
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Result_Filter_DateTime2 {
    /// PASS
    OccursInRange,
    /// FAIL
    OccursBeforeRange,
    /// FAIL
    OccursAfterRange,
}

impl Result_Filter_DateTime2 {
    #[inline(always)]
    pub const fn is_pass(&self) -> bool {
        matches!(*self, Result_Filter_DateTime2::OccursInRange)
    }

    #[inline(always)]
    pub const fn is_fail(&self) -> bool {
        matches!(*self, Result_Filter_DateTime2::OccursAfterRange | Result_Filter_DateTime2::OccursBeforeRange)
    }
}


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// built-in Datetime formats
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const DATETIME_PARSE_DATAS_LEN: usize = 104;

/// built-in datetime parsing patterns, these are all known patterns attempted on processed files
/// first string is a chrono strftime pattern
/// https://docs.rs/chrono/latest/chrono/format/strftime/
/// first two numbers are total string slice offsets
/// last two numbers are string slice offsets constrained to *only* the datetime portion
/// offset values are [X, Y) (beginning offset is inclusive, ending offset is exclusive or "one past")
/// i.e. string `"[2000-01-01 00:00:00]"`, the pattern may begin at `"["`, the datetime begins at `"2"`
///      same rule for the endings.
/// TODO: use std::ops::RangeInclusive
const DATETIME_PARSE_DATAS: [DateTime_Parse_Data_str; DATETIME_PARSE_DATAS_LEN] = [
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/samba/log.10.7.190.134` (multi-line)
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2020/03/05 12:17:59.631000,  3] ../source3/smbd/oplock.c:1340(init_oplocks)
    //        init_oplocks: initializing messages.
    //
    ("[%Y/%m/%d %H:%M:%S%.6f,", true, false, 0, 28, 1, 27),
    //
    // similar:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2000/01/01 00:00:04.123456] foo
    //
    ("[%Y/%m/%d %H:%M:%S%.6f]", true, false, 0, 28, 1, 27),
    // ---------------------------------------------------------------------------------------------
    // prescripted datetime+tz
    //
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-01 00:00:05 -0400 foo
    //     2000-01-01 00:00:05-0400 foo
    //
    ("%Y-%m-%d %H:%M:%S %z ", true, true, 0, 26, 0, 25),
    ("%Y-%m-%d %H:%M:%S%z ", true, true, 0, 25, 0, 24),
    ("%Y-%m-%dT%H:%M:%S %z ", true, true, 0, 26, 0, 25),
    ("%Y-%m-%dT%H:%M:%S%z ", true, true, 0, 25, 0, 24),
    //
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-01 00:00:05 ACST foo
    //     2000-01-01 00:00:05ACST foo
    //
    ("%Y-%m-%d %H:%M:%S %Z ", true, true, 0, 25, 0, 24),
    ("%Y-%m-%d %H:%M:%S%Z ", true, true, 0, 24, 0, 23),
    ("%Y-%m-%dT%H:%M:%S %Z ", true, true, 0, 25, 0, 24),
    ("%Y-%m-%dT%H:%M:%S%Z ", true, true, 0, 24, 0, 23),
    //
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-01 00:00:05 -04:00 foo
    //     2000-01-01 00:00:05-04:00 foo
    //
    ("%Y-%m-%d %H:%M:%S %:z ", true, true, 0, 27, 0, 26),
    ("%Y-%m-%d %H:%M:%S%:z ", true, true, 0, 26, 0, 25),
    ("%Y-%m-%dT%H:%M:%S %:z ", true, true, 0, 27, 0, 26),
    ("%Y-%m-%dT%H:%M:%S%:z ", true, true, 0, 26, 0, 25),
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     2000-01-01 00:00:01.234-0500 foo
    //     2000-01-01 00:00:01.234-05:00 foo
    //     2000-01-01 00:00:01.234 ACST foo
    //     2000-00-01T00:00:05.123-00:00 Five
    //
    ("%Y-%m-%d %H:%M:%S%.3f%z ", true, true, 0, 29, 0, 28),
    ("%Y-%m-%d %H:%M:%S%.3f%:z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%d %H:%M:%S%.3f %z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%d %H:%M:%S%.3f %:z ", true, true, 0, 31, 0, 30),
    ("%Y-%m-%d %H:%M:%S%.3f %Z ", true, true, 0, 29, 0, 28),
    ("%Y-%m-%dT%H:%M:%S%.3f%z ", true, true, 0, 29, 0, 28),
    ("%Y-%m-%dT%H:%M:%S%.3f%:z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%dT%H:%M:%S%.3f %z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%dT%H:%M:%S%.3f %:z ", true, true, 0, 31, 0, 30),
    ("%Y-%m-%dT%H:%M:%S%.3f %Z ", true, true, 0, 29, 0, 28),
    //
    //               1         2         3
    //     0123456789012345678901234567890123456789
    //     2000-01-01 00:00:01.234567-0800 foo
    //     2000-01-01 00:00:01.234567-08:00 foo
    //     2000-01-01 00:00:01.234567 ACST foo
    //
    ("%Y-%m-%d %H:%M:%S%.6f%z ", true, true, 0, 32, 0, 31),
    ("%Y-%m-%d %H:%M:%S%.6f %z ", true, true, 0, 33, 0, 32),
    ("%Y-%m-%d %H:%M:%S%.6f%:z ", true, true, 0, 33, 0, 32),
    ("%Y-%m-%d %H:%M:%S%.6f %:z ", true, true, 0, 34, 0, 33),
    ("%Y-%m-%d %H:%M:%S%.6f %Z ", true, true, 0, 32, 0, 31),
    ("%Y-%m-%dT%H:%M:%S%.6f%z ", true, true, 0, 32, 0, 31),
    ("%Y-%m-%dT%H:%M:%S%.6f %z ", true, true, 0, 33, 0, 32),
    ("%Y-%m-%dT%H:%M:%S%.6f%:z ", true, true, 0, 33, 0, 32),
    ("%Y-%m-%dT%H:%M:%S%.6f %:z ", true, true, 0, 34, 0, 33),
    ("%Y-%m-%dT%H:%M:%S%.6f %Z ", true, true, 0, 32, 0, 31),
    //
    //               1         2         3
    //     0123456789012345678901234567890123456789
    //     20000101T000001 -0800 foo
    //     20000101T000001 -08:00 foo
    //     20000101T000001 ACST foo
    //
    ("%Y%m%dT%H%M%S %z ", true, true, 0, 22, 0, 21),
    ("%Y%m%dT%H%M%S %:z ", true, true, 0, 23, 0, 22),
    ("%Y%m%dT%H%M%S %Z ", true, true, 0, 22, 0, 21),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/vmware/hostd-62.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     2019-07-26T10:40:29.682-07:00 info hostd[03210] [Originator@6876 sub=Default] Current working directory: /usr/bin
    //
    ("%Y-%m-%dT%H:%M:%S%.3f%z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%dT%H:%M:%S%.3f%Z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%dT%H:%M:%S%.3f-", true, false, 0, 24, 0, 23),  // XXX: temporary stand-in
    ("%Y-%m-%d %H:%M:%S%.6f-", true, false, 0, 27, 0, 26),  // XXX: temporary stand-in
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/kernel.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     Mar  9 08:10:29 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode
    //
    // TODO: [2021/10/03] no support of inferring the year
    //("%b %e %H:%M:%S ", 0, 25, 0, 25),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/synology/synobackup.log` (has horizontal alignment tabs)
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     info	2017/02/21 21:50:48	SYSTEM:	[Local][Backup Task LocalBackup1] Backup task started.
    //     err	2017/02/23 02:55:58	SYSTEM:	[Local][Backup Task LocalBackup1] Exception occured while backing up data. (Capacity at destination is insufficient.) [Path: /volume1/LocalBackup1.hbk]
    // example escaped:
    //     info␉2017/02/21 21:50:48␉SYSTEM:␉[Local][Backup Task LocalBackup1] Backup task started.
    //     err␉2017/02/23 02:55:58␉SYSTEM:␉[Local][Backup Task LocalBackup1] Exception occured while backing up data. (Capacity at destination is insufficient.) [Path: /volume1/LocalBackup1.hbk]
    //
    // TODO: [2021/10/03] no support of variable offset datetime
    //       this could be done by trying random offsets into something
    //       better is to search for a preceding regexp pattern
    //("\t%Y/%m/%d %H:%M:%S\t", 5, 24, 0, 24),
    // ---------------------------------------------------------------------------------------------
    //
    // iptables warning from kernel, from file `/var/log/messages` on OpenWRT
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     Mar 30 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/xrdp.log`
    // example with offset:
    //
    //               1
    //     01234567890123456789
    //     [20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
    //
    ("[%Y%m%d-%H:%M:%S]", true, false, 0, 19, 1, 18),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/vmware-installer.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2019-05-06 11:24:34,074] Successfully loaded GTK libraries.
    //
    ("[%Y-%m-%d %H:%M:%S,%3f] ", true, false, 0, 26, 1, 24),
    // repeat prior but no trailing space
    ("[%Y-%m-%d %H:%M:%S,%3f]", true, false, 0, 25, 1, 24),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/other/archives/proftpd/xferlog`
    // example with offset:
    //
    //               1         2
    //     0123456789012345678901234
    //     Sat Oct 03 11:26:12 2020 0 192.168.1.12 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c
    //
    ("%a %b %d %H:%M:%S %Y ", true, false, 0, 25, 0, 24),
    // repeat prior but no trailing space
    ("%a %b %d %H:%M:%S %Y", true, false, 0, 24, 0, 24),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/OpenSUSE15/zypper.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2019-05-23 16:53:43 <1> trenker(24689) [zypper] main.cc(main):74 ===== Hi, me zypper 1.14.27
    //
    //("%Y-%m-%d %H:%M:%S ", 0, 20, 0, 19),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     2020-01-01 00:00:01.001 xyz
    //      2020-01-01 00:00:01.001 xyz
    //       2020-01-01 00:00:01.001 xyz
    //        2020-01-01 00:00:01.001 xyz
    //         2020-01-01 00:00:01.001 xyz
    //          2020-01-01 00:00:01.001 xyz
    //           2020-01-01 00:00:01.001 xyz
    //            2020-01-01 00:00:01.001 xyz
    //             2020-01-01 00:00:01.001 xyz
    //              2020-01-01 00:00:01.001 xyz
    //     2020-01-01 00:00:01 xyz
    //      2020-01-01 00:00:01 xyz
    //       2020-01-01 00:00:01 xyz
    //        2020-01-01 00:00:01 xyz
    //         2020-01-01 00:00:01 xyz
    //          2020-01-01 00:00:01 xyz
    //           2020-01-01 00:00:01 xyz
    //            2020-01-01 00:00:01 xyz
    //             2020-01-01 00:00:01 xyz
    //              2020-01-01 00:00:01 xyz
    //     2020-01-01 00:00:01xyz
    //      2020-01-01 00:00:01xyz
    //       2020-01-01 00:00:01xyz
    //        2020-01-01 00:00:01xyz
    //         2020-01-01 00:00:01xyz
    //          2020-01-01 00:00:01xyz
    //           2020-01-01 00:00:01xyz
    //            2020-01-01 00:00:01xyz
    //             2020-01-01 00:00:01xyz
    //              2020-01-01 00:00:01xyz
    //
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 0, 24, 0, 23),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 1, 25, 1, 24),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 2, 26, 2, 25),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 3, 27, 3, 26),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 4, 28, 4, 27),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 5, 29, 5, 28),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 6, 30, 6, 29),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 7, 31, 7, 30),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 8, 32, 8, 31),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 9, 33, 9, 32),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 10, 34, 10, 33),
    ("%Y-%m-%d %H:%M:%S ", true, false, 0, 20, 0, 19),
    ("%Y-%m-%d %H:%M:%S ", true, false, 1, 21, 1, 20),
    ("%Y-%m-%d %H:%M:%S ", true, false, 2, 22, 2, 21),
    ("%Y-%m-%d %H:%M:%S ", true, false, 3, 23, 3, 22),
    ("%Y-%m-%d %H:%M:%S ", true, false, 4, 24, 4, 23),
    ("%Y-%m-%d %H:%M:%S ", true, false, 5, 25, 5, 24),
    ("%Y-%m-%d %H:%M:%S ", true, false, 6, 26, 6, 25),
    ("%Y-%m-%d %H:%M:%S ", true, false, 7, 27, 7, 26),
    ("%Y-%m-%d %H:%M:%S ", true, false, 8, 28, 8, 27),
    ("%Y-%m-%d %H:%M:%S ", true, false, 9, 29, 9, 28),
    ("%Y-%m-%d %H:%M:%S ", true, false, 10, 30, 10, 29),
    ("%Y-%m-%d %H:%M:%S", true, false, 0, 19, 0, 19),
    ("%Y-%m-%d %H:%M:%S", true, false, 1, 20, 1, 20),
    ("%Y-%m-%d %H:%M:%S", true, false, 2, 21, 2, 21),
    ("%Y-%m-%d %H:%M:%S", true, false, 3, 22, 3, 22),
    ("%Y-%m-%d %H:%M:%S", true, false, 4, 23, 4, 23),
    ("%Y-%m-%d %H:%M:%S", true, false, 5, 24, 5, 24),
    ("%Y-%m-%d %H:%M:%S", true, false, 6, 25, 6, 25),
    ("%Y-%m-%d %H:%M:%S", true, false, 7, 26, 7, 26),
    ("%Y-%m-%d %H:%M:%S", true, false, 8, 27, 8, 27),
    ("%Y-%m-%d %H:%M:%S", true, false, 9, 28, 9, 28),
    ("%Y-%m-%d %H:%M:%S", true, false, 10, 29, 10, 29),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2020-01-01T00:00:01 xyz
    //
    ("%Y-%m-%dT%H:%M:%S ", true, false, 0, 20, 0, 19),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2020-01-01T00:00:01xyz
    //
    ("%Y-%m-%dT%H:%M:%S", true, false, 0, 19, 0, 19),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1
    //     012345678901234567
    //     20200101 000001 xyz
    //
    ("%Y%m%d %H%M%S ", true, false, 0, 16, 0, 15),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1
    //     012345678901234567
    //     20200101T000001 xyz
    //
    ("%Y%m%dT%H%M%S ", true, false, 0, 16, 0, 15),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1
    //     012345678901234567
    //     20200101T000001xyz
    //
    ("%Y%m%dT%H%M%S", true, false, 0, 15, 0, 15),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/debian9/apport.log.1`
    // example with offset:
    //
    //               1         2         3         4         5
    //     012345678901234567890123456789012345678901234567890
    //     ERROR: apport (pid 9) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 93) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 935) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 9359) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //
    (" %a %b %d %H:%M:%S %Y:", true, false, 22, 47, 22, 46),
    (" %a %b %d %H:%M:%S %Y:", true, false, 23, 48, 23, 47),
    (" %a %b %d %H:%M:%S %Y:", true, false, 24, 49, 24, 48),
    (" %a %b %d %H:%M:%S %Y:", true, false, 25, 50, 25, 49),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     INFO: Thu Feb 20 00:59:59 2020 info
    //     ERROR: Thu Feb 20 00:59:59 2020 error
    //     DEBUG: Thu Feb 20 00:59:59 2020 debug
    //     VERBOSE: Thu Feb 20 00:59:59 2020 verbose
    //
    (" %a %b %d %H:%M:%S %Y ", true, false, 5, 31, 6, 30),
    (" %a %b %d %H:%M:%S %Y ", true, false, 6, 32, 7, 31),
    (" %a %b %d %H:%M:%S %Y ", true, false, 7, 33, 8, 32),
    (" %a %b %d %H:%M:%S %Y ", true, false, 8, 34, 9, 33),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     INFO: Sat Jan 01 2000 08:00:00 info
    //     WARN: Sat Jan 01 2000 08:00:00 warn
    //     ERROR: Sat Jan 01 2000 08:00:00 error
    //     DEBUG: Sat Jan 01 2000 08:00:00 debug
    //     VERBOSE: Sat Jan 01 2000 08:00:00 verbose
    //
    (" %a %b %d %Y %H:%M:%S ", true, false, 5, 31, 6, 30),
    (" %a %b %d %Y %H:%M:%S ", true, false, 6, 32, 7, 31),
    (" %a %b %d %Y %H:%M:%S ", true, false, 7, 33, 8, 32),
    (" %a %b %d %Y %H:%M:%S ", true, false, 8, 34, 9, 33),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     [ERROR] 2000-01-01T00:00:03 foo
    //     [WARN] 2000-01-01T00:00:03 foo
    //     [DEBUG] 2000-01-01T00:00:03 foo
    //     [INFO] 2000-01-01T00:00:03 foo
    //     [VERBOSE] 2000-01-01T00:00:03 foo
    //
    ("] %Y-%m-%dT%H:%M:%S ", true, false, 5, 27, 7, 26),
    ("] %Y-%m-%dT%H:%M:%S ", true, false, 6, 28, 8, 27),
    ("] %Y-%m-%dT%H:%M:%S ", true, false, 7, 29, 9, 28),
    ("] %Y-%m-%dT%H:%M:%S ", true, false, 8, 30, 10, 29),
    ("] %Y-%m-%d %H:%M:%S ", true, false, 5, 27, 7, 26),
    ("] %Y-%m-%d %H:%M:%S ", true, false, 6, 28, 8, 27),
    ("] %Y-%m-%d %H:%M:%S ", true, false, 7, 29, 9, 28),
    ("] %Y-%m-%d %H:%M:%S ", true, false, 8, 30, 10, 29),
    // ---------------------------------------------------------------------------------------------
    // TODO: [2022/03/24] add timestamp formats seen at https://www.unixtimestamp.com/index.php
];

fn DateTime_Parse_Data_str_to_DateTime_Parse_Data(dtpds: &DateTime_Parse_Data_str) -> DateTime_Parse_Data {
    DateTime_Parse_Data {
        pattern: dtpds.0.to_string(),
        year: dtpds.1,
        tz: dtpds.2,
        sib: dtpds.3,
        sie: dtpds.4,
        siba: dtpds.5,
        siea: dtpds.6,
    }
}

lazy_static! {
    static ref DATETIME_PARSE_DATAS_VEC: DateTime_Parse_Datas_vec =
        DATETIME_PARSE_DATAS.iter().map(
            |&x| DateTime_Parse_Data_str_to_DateTime_Parse_Data(&x)
        ).collect();
}

lazy_static! {
    static ref DATETIME_PARSE_DATAS_VEC_LONGEST: usize =
        DATETIME_PARSE_DATAS.iter().max_by(|x, y| x.0.len().cmp(&y.0.len())).unwrap().0.len();
}

/// built-in sanity check of the static DATETIME_PARSE_DATAS
/// can only check for coarse errors, cannot check catch all errors
#[test]
fn test_DATETIME_PARSE_DATAS() {
    for dtpd in DATETIME_PARSE_DATAS_VEC.iter() {
        assert_lt!(dtpd.sib, dtpd.sie, "dtpd.sib < dtpd.sie failed; bad build-in DateTimeParseData {:?}", dtpd);
        assert_lt!(dtpd.siba, dtpd.siea, "dtpd.siba < dtpd.siea failed; bad build-in DateTimeParseData {:?}", dtpd);
        assert_le!(dtpd.sib, dtpd.siba, "dtpd.sib ≤ dtpd.siba failed; bad build-in DateTimeParseData {:?}", dtpd);
        assert_ge!(dtpd.sie, dtpd.siea, "dtpd.sie ≥ dtpd.siea failed; bad build-in DateTimeParseData {:?}", dtpd);
        let len_ = dtpd.pattern.len();
        // XXX: arbitrary minimum
        assert_le!(6, len_, ".pattern.len too short; bad build-in DateTimeParseData {:?}", dtpd);
        let diff_ = dtpd.sie - dtpd.sib;
        let diff_dt = dtpd.siea - dtpd.siba;
        assert_ge!(diff_, diff_dt, "len(.sib,.sie) ≥ len(.siba,.siea) failed; bad build-in DateTimeParseData {:?}", dtpd);
        assert_ge!(diff_, len_, "len(.sib,.sie) ≥ .dtp.len() failed; bad build-in DateTimeParseData {:?}", dtpd);
        //assert_le!(diff_dt, len_, "len(.3,.4) ≤ .0.len() failed; bad build-in DateTimeParseData {:?}", dtpd);
        if dtpd.year {
            assert!(dt_pattern_has_year(&dtpd.pattern), "pattern has not year {:?} but .year is true", dtpd.pattern);
        } else {
            assert!(!dt_pattern_has_year(&dtpd.pattern), "pattern has year {:?} but .year is false", dtpd.pattern);
        }
        if dtpd.tz {
            assert!(dt_pattern_has_tz(&dtpd.pattern), "pattern has not timezone {:?} but tz is true", dtpd.pattern);
        } else {
            assert!(!dt_pattern_has_tz(&dtpd.pattern), "pattern has timezone {:?} but tz is false", dtpd.pattern);
        }
    }
    // check for duplicates
    let mut check = DATETIME_PARSE_DATAS_VEC.clone();
    check.sort_unstable();
    let check_orig = check.clone();
    check.dedup();
    //let check: DateTime_Parse_Datas_vec = DATETIME_PARSE_DATAS.into_iter().unique().collect();
    if check.len() != DATETIME_PARSE_DATAS.len() {
        for (i, (co, cd)) in check_orig.iter().zip(check.iter()).enumerate() {
            debug_eprintln!("entry {} {:?} {:?}", i, co, cd);
        }
        for (co, cd) in check_orig.iter().zip(check.iter()) {
            assert_eq!(co, cd, "entry {:?} appears to be a duplicate", co);
        }
    };
    assert_eq!(check.len(), DATETIME_PARSE_DATAS.len(), "the deduplicated DATETIME_PARSE_DATAS_VEC is different len than original; there are duplicates in DATETIME_PARSE_DATAS_VEC but the test could not determine which entry.");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Summary
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// statistics to print about `SyslineReader` activity
#[derive(Copy, Clone, Default)]
pub struct Summary {
    /// count of bytes stored by `BlockReader`
    pub bytes: u64,
    /// count of bytes in file
    pub bytes_total: u64,
    /// count of `Block`s read by `BlockReader`
    pub blocks: u64,
    /// count of `Block`s in file
    pub blocks_total: u64,
    /// `BlockSz` of `BlockReader`
    pub blocksz: BlockSz,
    /// count of `Lines` processed by `LineReader`
    pub lines: u64,
    /// count of `Syslines` processed by `SyslineReader`
    pub syslines: u64,
}

impl Summary {
    pub fn new(
        bytes: u64, bytes_total: u64, blocks: u64, blocks_total: u64, blocksz: BlockSz, lines: u64, syslines: u64,
    ) -> Summary {
        // some sanity checks
        assert_ge!(bytes, blocks, "There is less bytes than Blocks");
        assert_ge!(bytes, lines, "There is less bytes than Lines");
        assert_ge!(bytes, lines, "There is less bytes than Syslines");
        assert_ge!(blocksz, BLOCKSZ_MIN, "blocksz too small");
        assert_le!(blocksz, BLOCKSZ_MAX, "blocksz too big");
        assert_ge!(lines, syslines, "There is less Lines than Syslines");
        Summary {
            bytes,
            bytes_total,
            blocks,
            blocks_total,
            blocksz,
            lines,
            syslines,
        }
    }
}

impl fmt::Debug for Summary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Summary Processed:")
            .field("bytes", &self.bytes)
            .field("bytes total", &self.bytes_total)
            .field("lines", &self.lines)
            .field("syslines", &self.syslines)
            .field("blocks", &self.blocks)
            .field("blocks total", &self.blocks_total)
            .field("blocksz", &format_args!("{0} (0x{0:X})", &self.blocksz))
            .finish()
    }
}

type Summary_Opt = Option<Summary>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// thread-safe Atomic Reference Counting Pointer to a `Sysline`
type SyslineP = Arc<Sysline>;
type SyslineP_Opt = Option<Arc<Sysline>>;
/// storage for `Sysline`
type Syslines = BTreeMap<FileOffset, SyslineP>;
/// range map where key is sysline begin to end `[ Sysline.fileoffset_begin(), Sysline.fileoffset_end()]`
/// and where value is sysline begin (`Sysline.fileoffset_begin()`). Use the value to lookup associated `Syslines` map
type SyslinesRangeMap = RangeMap<FileOffset, FileOffset>;

/// Specialized Reader that uses `LineReader` to find syslog lines
pub struct SyslineReader<'syslinereader> {
    linereader: LineReader<'syslinereader>,
    /// Syslines by fileoffset_begin
    syslines: Syslines,
    /// count of Syslines processed
    syslines_count: u64,
    // TODO: has `syslines_by_range` ever found a sysline?
    //       would be good to add a test for it.
    /// Syslines fileoffset by sysline fileoffset range, i.e. `[Sysline.fileoffset_begin(), Sysline.fileoffset_end()+1)`
    /// the stored value can be used as a key for `self.syslines`
    syslines_by_range: SyslinesRangeMap,
    /// datetime formatting data, for extracting datetime strings from Lines
    /// TODO: change to Set
    dt_patterns: DateTime_Parse_Datas_vec,
    /// internal use; counts found patterns stored in `dt_patterns`
    /// not used after `analyzed == true`
    dt_patterns_counts: DateTime_Pattern_Counts,
    /// default FixedOffset for found `DateTime` without timezone
    tz_offset: FixedOffset,
    // TODO: [2021/09/21] add efficiency stats
    // TODO: get rid of LRU cache
    /// internal LRU cache for `find_sysline`
    _find_sysline_lru_cache: SyslinesLRUCache,
    /// has `self.file_analysis` completed?
    analyzed: bool,
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
#[cfg(debug_assertions)]
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
    const DT_PATTERN_MAX_PRE_ANALYSIS: usize = 4;
    const DT_PATTERN_MAX: usize = 1;
    const ANALYSIS_THRESHOLD: u64 = 5;

    pub fn new(path: &'syslinereader FPath, blocksz: BlockSz, tz_offset: FixedOffset) -> Result<SyslineReader<'syslinereader>> {
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
            syslines_count: 0,
            syslines_by_range: SyslinesRangeMap::new(),
            dt_patterns: DateTime_Parse_Datas_vec::with_capacity(SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS),
            dt_patterns_counts: DateTime_Pattern_Counts::with_capacity(SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS),
            tz_offset: tz_offset,
            _find_sysline_lru_cache: SyslinesLRUCache::new(4),
            analyzed: false,
        })
    }

    pub fn blocksz(&self) -> BlockSz {
        self.linereader.blocksz()
    }

    pub fn filesz(&self) -> BlockSz {
        self.linereader.filesz()
    }

    pub fn path(&self) -> &FPath {
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

    /// smallest size character
    pub fn charsz(&self) -> usize {
        self.linereader._charsz
    }

    /// count of `Sysline`s processed
    pub fn count(&self) -> u64 {
        self.syslines_count
    }

    /// Testing helper only
    #[cfg(any(debug_assertions,test))]
    pub fn print(&self, fileoffset: FileOffset, raw: bool) {
        let syslinep: &SyslineP = match self.syslines.get(&fileoffset) {
            Some(val) => val,
            None => {
                eprintln!("ERROR: in print, self.syslines.get({}) returned None", fileoffset);
                return;
            }
        };
        for linep in &(*syslinep).lines {
            (*linep).print(raw);
        }
    }

    /// Testing helper only
    /// print all known `Sysline`s
    #[cfg(any(debug_assertions,test))]
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
        false
    }

    /// store passed `Sysline` in `self.syslines`, update other fields
    fn insert_sysline(&mut self, line: Sysline) -> SyslineP {
        let fo_beg: FileOffset = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let slp = SyslineP::new(line);
        debug_eprintln!("{}SyslineReader.insert_sysline: syslines.insert({}, Sysline @{:p})", so(), fo_beg, &*slp);
        self.syslines.insert(fo_beg, slp.clone());
        self.syslines_count += 1;
        // XXX: multi-byte character
        let fo_end1 = fo_end + (self.charsz() as FileOffset);
        debug_eprintln!(
            "{}SyslineReader.insert_sysline: syslines_by_range.insert(({}‥{}], {})",
            so(),
            fo_beg,
            fo_end1,
            fo_beg
        );
        self.syslines_by_range.insert(fo_beg..fo_end1, fo_beg);
        return slp;
    }

    /// workaround for chrono Issue #660 https://github.com/chronotope/chrono/issues/660
    /// match spaces at beginning and ending of inputs
    /// TODO: handle all Unicode whitespace.
    ///       This fn is essentially counteracting an errant call to `std::string:trim`
    ///       within `Local.datetime_from_str`.
    ///       `trim` removes "Unicode Derived Core Property White_Space".
    ///       This implementation handles three whitespace chars. There are twenty-five whitespace
    ///       chars according to
    ///       https://en.wikipedia.org/wiki/Unicode_character_property#Whitespace
    pub fn datetime_from_str_workaround_Issue660(value: &str, pattern: &DateTimePattern_str) -> bool {
        let spaces = " ";
        let tabs = "\t";
        let lineends = "\n\r";

        // match whitespace forwards from beginning
        let mut v_sc: u32 = 0;  // `value` spaces count
        let mut v_tc: u32 = 0;  // `value` tabs count
        let mut v_ec: u32 = 0;  // `value` line ends count
        let mut v_brk: bool = false;
        for v_ in value.chars() {
            if spaces.contains(v_) {
                v_sc += 1;
            } else if tabs.contains(v_) {
                v_tc += 1;
            } else if lineends.contains(v_) {
                v_ec += 1;
            } else {
                v_brk = true;
                break;
            }
        }
        let mut p_sc: u32 = 0;  // `pattern` space count
        let mut p_tc: u32 = 0;  // `pattern` tab count
        let mut p_ec: u32 = 0;  // `pattern` line ends count
        let mut p_brk: bool = false;
        for p_ in pattern.chars() {
            if spaces.contains(p_) {
                p_sc += 1;
            } else if tabs.contains(p_) {
                p_tc += 1;
            } else if lineends.contains(p_) {
                p_ec += 1;
            } else {
                p_brk = true;
                break;
            }
        }
        if v_sc != p_sc || v_tc != p_tc || v_ec != p_ec {
            return false;
        }

        // match whitespace backwards from ending
        v_sc = 0;
        v_tc = 0;
        v_ec = 0;
        if v_brk {
            for v_ in value.chars().rev() {
                if spaces.contains(v_) {
                    v_sc += 1;
                } else if tabs.contains(v_) {
                    v_tc += 1;
                } else if lineends.contains(v_) {
                    v_ec += 1;
                } else {
                    break;
                }
            }
        }
        p_sc = 0;
        p_tc = 0;
        p_ec = 0;
        if p_brk {
            for p_ in pattern.chars().rev() {
                if spaces.contains(p_) {
                    p_sc += 1;
                } else if tabs.contains(p_) {
                    p_tc += 1;
                } else if lineends.contains(p_) {
                    p_ec += 1;
                } else {
                    break;
                }
            }
        }
        if v_sc != p_sc || v_tc != p_tc || v_ec != p_ec {
            return false;
        }

        return true;
    }

    /// decoding `[u8]` bytes to a `str` takes a surprising amount of time, according to `tools/flamegraph.sh`.
    /// first check `u8` slice with custom simplistic checker that, in case of complications,
    /// falls back to using higher-resource and more-precise checker `encoding_rs::mem::utf8_latin1_up_to`.
    /// this uses built-in unsafe `str::from_utf8_unchecked`.
    /// See `benches/bench_decode_utf.rs` for comparison of bytes->str decode strategies
    #[inline(always)]
    fn u8_to_str(slice_: &[u8]) -> Option<&str> {
        let dts: &str;
        let mut fallback = false;
        // custom check for UTF8; fast but imperfect
        if ! slice_.is_ascii() {
            fallback = true;
        }
        if fallback {
            // found non-ASCII, fallback to checking with `utf8_latin1_up_to` which is a thorough check
            let va = encoding_rs::mem::utf8_latin1_up_to(slice_);
            if va != slice_.len() {
                return None;  // invalid UTF8
            }
        }
        unsafe {
            dts = std::str::from_utf8_unchecked(slice_);
        };
        Some(dts)
    }

    pub fn str_datetime(dts: &str, dtpd: &DateTime_Parse_Data, tz_offset: &FixedOffset) -> DateTimeL_Opt {
        str_datetime(dts, &dtpd.pattern.as_str(), dtpd.tz, tz_offset)
    }

    /// if datetime found in `Line` returns `Ok` around
    /// indexes into `line` of found datetime string `(start of string, end of string)`
    /// else returns `Err`
    /// TODO: assumes Local TZ
    /// TODO: 2022/03/11 14:30:00
    ///      The concept of static pattern lengths (beg_i, end_i, actual_beg_i, actual_end_i) won't work for
    ///      variable length datetime patterns, i.e. full month names 'July 1, 2020' and 'December 1, 2020'.
    ///      Instead of fixing the current problem of unexpected datetime matches,
    ///      fix the concept problem of passing around fixed-length datetime strings. Then redo this.
    pub fn find_datetime_in_line(
        line: &Line, parse_data: &'syslinereader DateTime_Parse_Datas_vec, fpath: &FPath, charsz: &CharSz, tz_offset: &FixedOffset,
    ) -> Result_FindDateTime {
        debug_eprintln!("{}find_datetime_in_line:(Line@{:p}, {:?}) {:?}", sn(), &line, line.to_String_noraw(), fpath);
        // skip easy case; no possible datetime
        if line.len() < 4 {
            debug_eprintln!("{}find_datetime_in_line: return Err(ErrorKind::InvalidInput);", sx());
            return Result_FindDateTime::Err(Error::new(ErrorKind::InvalidInput, "Line is too short"));
        }

        //let longest: usize = *DATETIME_PARSE_DATAS_VEC_LONGEST;
        //let mut dtsS: String = String::with_capacity(longest * (2 as usize));

        let mut i = 0;
        // `sie` and `siea` is one past last char; exclusive.
        // `actual` are more confined slice offsets of the datetime,
        // XXX: it might be faster to skip the special formatting and look directly for the datetime stamp.
        //      calls to chrono are long according to the flamegraph.
        //      however, using the demarcating characters ("[", "]") does give better assurance.
        for dtpd in parse_data.iter() {
            i += 1;
            debug_eprintln!("{}find_datetime_in_line: pattern tuple {} ({:?}, {}, {}, {}, {})", so(), i, dtpd.pattern, dtpd.sib, dtpd.sie, dtpd.siba, dtpd.siea);
            debug_assert_lt!(dtpd.sib, dtpd.sie, "Bad values dtpd.sib dtpd.sie");
            debug_assert_ge!(dtpd.siba, dtpd.sib, "Bad values dtpd.siba dtpd.sib");
            debug_assert_le!(dtpd.siea, dtpd.sie, "Bad values dtpd.siea dtpd.sie");
            //debug_eprintln!("{}find_datetime_in_line searching for pattern {} {:?}", so(), i, dtpd.pattern);
            let len_ = (dtpd.sie - dtpd.sib) as LineIndex;
            // XXX: does not support multi-byte string; assumes single-byte
            if line.len() < dtpd.sie {
                debug_eprintln!(
                    "{}find_datetime_in_line: line len {} is too short for pattern {} len {} @({}, {}] {:?}",
                    so(),
                    line.len(),
                    i,
                    len_,
                    dtpd.sib,
                    dtpd.sie,
                    dtpd.pattern,
                );
                continue;
            }
            // take a slice of the `line_as_slice` then convert to `str`
            // this is to force the parsing function `Local.datetime_from_str` to constrain where it
            // searches within the `Line`
            // TODO: to make this a bit more efficient, would be good to do a lookahead. Add a funciton like
            //       `Line.crosses_block(a: LineIndex, b: LineIndex) -> bool`. Then could set capacity of things
            //       ahead of time.
            let slice_: &[u8];
            let mut hack_slice: Bytes;
            match line.get_boxptrs(dtpd.sib, dtpd.sie) {
                enum_BoxPtrs::SinglePtr(box_slice) => {
                    slice_ = *box_slice;
                },
                enum_BoxPtrs::MultiPtr(vec_box_slice) => {
                    // XXX: really inefficient!
                    hack_slice = Bytes::new();
                    for box_ in vec_box_slice {
                        hack_slice.extend_from_slice(*box_);
                    }
                    slice_ = hack_slice.as_slice();
                },
            };
            // hack efficiency improvement, presumes all found years will have a '1' or a '2' in them
            if charsz == &1 && dtpd.year && !(slice_.contains(&b'1') || slice_.contains(&b'2')) {
                debug_eprintln!("{}find_datetime_in_line: skip slice, does not have '1' or '2'", so());
                continue;
            }
            let dts: &str = match SyslineReader::u8_to_str(slice_) {
                Some(val) => val,
                None => { continue; }
            };
            debug_eprintln!(
                "{}find_datetime_in_line: searching for pattern {} {:?} in {:?} (slice [{}‥{}] from Line {:?})",
                so(),
                i,
                dtpd.pattern,
                str_to_String_noraw(dts),
                dtpd.sib,
                dtpd.sie,
                line.to_String_noraw(),
            );
            // TODO: [2021/10/03]
            //       according to flamegraph, this function `Local::datetime_from_str` takes a very large amount of
            //       runtime, around 20% to 25% of entire process runtime. How to improve that?
            //       Could I create my own hardcoded parsing for a few common patterns?
            let dt = match SyslineReader::str_datetime(dts, dtpd, &tz_offset) {
                Some(val) => {
                    debug_eprintln!("{}find_datetime_in_line: str_datetime returned {:?}", so(), val);
                    val
                }
                None => {
                    debug_eprintln!("{}find_datetime_in_line: str_datetime returned None", so());
                    continue;
                }
            }; // end for(pattern, ...)
            debug_eprintln!("{}find_datetime_in_line: return Ok({}, {}, {});", sx(), dtpd.sib, dtpd.sie, &dt);
            return Result_FindDateTime::Ok((dtpd.clone(), dt));
        }

        debug_eprintln!("{}find_datetime_in_line: return Err(ErrorKind::NotFound);", sx());
        return Result_FindDateTime::Err(Error::new(ErrorKind::NotFound, "No datetime found!"));
    }

    /// private helper function to update `self.dt_patterns`
    fn dt_patterns_update(&mut self, datetime_parse_data: DateTime_Parse_Data) {
        if self.analyzed {
            return;
        }
        debug_eprintln!("{}dt_patterns_update(SyslineReader@{:p}, {:?})", sn(), self, datetime_parse_data);
        //
        // update `self.dt_patterns_counts`
        //
        if self.dt_patterns_counts.contains_key(&datetime_parse_data) {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns_counts.get_mut({:?}) += 1",
                so(),
                self,
                datetime_parse_data
            );
            let counter = self.dt_patterns_counts.get_mut(&datetime_parse_data).unwrap();
            *counter += 1;
        } else if self.dt_patterns_counts.len() < SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns_counts.insert({:?}, 0)",
                so(),
                self,
                datetime_parse_data
            );
            self.dt_patterns_counts.insert(datetime_parse_data.clone(), 1);
        }
        //
        // update `self.dt_patterns`
        //
        if self.dt_patterns.len() >= SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns already DT_PATTERN_MAX_PRE_ANALYSIS length {:?}",
                sx(),
                self,
                &self.dt_patterns.len()
            );
            return;
        }
        if self.dt_patterns.contains(&datetime_parse_data) {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) found DateTime_Parse_Data; skip self.dt_patterns.push",
                sx(),
                self
            );
            return;
        }
        debug_eprintln!(
            "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns.push({:?})",
            sx(),
            self,
            datetime_parse_data
        );
        self.dt_patterns.push(datetime_parse_data);
    }

    /// analyze syslines gathered
    /// when a threshold of syslines or bytes has been processed, then
    /// 1. narrow down datetime formats used. this greatly reduces resources
    /// used by `SyslineReader::find_datetime_in_line`
    /// 2. ??? I forgot what else I wanted this function to do.
    /// TODO: will break if DT_PATTERN_MAX > 1
    fn dt_patterns_analysis(&mut self) {
        if self.analyzed || self.count() < SyslineReader::ANALYSIS_THRESHOLD {
            return;
        }
        debug_eprintln!("{}dt_patterns_analysis()", sn());
        if SyslineReader::DT_PATTERN_MAX != 1 {
            unimplemented!("function dt_patterns_analysis unimplemented for DT_PATTERN_MAX > 1; it is {}", SyslineReader::DT_PATTERN_MAX);
        }
        debug_assert_eq!(self.dt_patterns.len(), self.dt_patterns_counts.len(),
            "dt_patterns.len() != dt_patterns_count.len()\nself.dt_patterns ({})     : {:?}\nself.dt_patterns_counts ({}): {:?}", self.dt_patterns.len(), self.dt_patterns, self.dt_patterns_counts.len(), self.dt_patterns_counts);
        // TODO: change pattern tuple `DateTime_Parse_Data` to use Ranges, currently this is
        //       removing valid patterns in different (beg,end) positions
        // ripped from https://stackoverflow.com/a/60134450/471376
        // test https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b8eb53f40fd89461c9dad9c976746cc3
        let max_ = (&self.dt_patterns_counts).iter().fold(
            std::u64::MIN, |a,b| a.max(*(b.1))
        );
        self.dt_patterns_counts.retain(|_, v| *v >= max_);
        self.dt_patterns_counts.shrink_to(SyslineReader::DT_PATTERN_MAX);
        if cfg!(debug_assertions) {
            for (k, v) in self.dt_patterns_counts.iter() {
                debug_eprintln!("{}dt_patterns_analysis: self.dt_patterns_counts[{:?}]={:?}", so(), k, v);
            }
        }
        // XXX: is there a simpler way to get the first element?
        let datetime_parse_data = match self.dt_patterns_counts.iter().next() {
            Some((k, _)) => { k },
            None => {
                eprintln!("ERROR: self.dt_patterns_counts.values().next() returned None, it is len {}", self.dt_patterns_counts.len());
                self.analyzed = true;
                return;
            }
        };
        debug_eprintln!("{}dt_patterns_analysis: chose dt_pattern", so());
        // effectively remove all elements by index, except for `datetime_parse_data`
        // XXX: what is the rust idiomatic way to remove all but a few elements by index?
        let mut patts = DateTime_Parse_Datas_vec::with_capacity(SyslineReader::DT_PATTERN_MAX);
        let mut index_: usize = 0;
        for datetime_parse_data_ in &self.dt_patterns {
            if datetime_parse_data_.eq(&datetime_parse_data) {
                break;
            }
            index_ += 1;
        }
        patts.push(self.dt_patterns.swap_remove(index_));
        self.dt_patterns = patts;
        //self.dt_patterns.retain(|v| v == **patt);
        self.dt_patterns.shrink_to(SyslineReader::DT_PATTERN_MAX);
        if cfg!(debug_assertions) {
            for dtpd in self.dt_patterns.iter() {
                debug_eprintln!("{}dt_patterns_analysis: self.dt_pattern {:?}", so(), dtpd);
            }
        }
        self.dt_patterns_counts.clear();
        self.dt_patterns_counts.shrink_to(0);
        self.analyzed = true;
        debug_eprintln!("{}dt_patterns_analysis()", sx());
    }

    /// attempt to parse a DateTime substring in the passed `Line`
    /// wraps call to `find_datetime_in_line` according to status of `self.dt_patterns`
    /// if `self.dt_patterns` is `None`, will set `self.dt_patterns`
    fn parse_datetime_in_line(&mut self, line: &Line, charsz: &CharSz) -> Result_ParseDateTime {
        // XXX: would prefer this at the end of this function, but borrow error occurs
        if !self.analyzed {
            self.dt_patterns_analysis();
        };
        debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}); {:?}", sn(), self, line.to_String_noraw());
        // if no `dt_patterns` have been found then try the default datetime patterns immediately
        if self.dt_patterns.is_empty() {
            debug_eprintln!("{}parse_datetime_in_line self.dt_patterns is empty", sn());
            // this `SyslineReader` has not determined it's own DateTime formatting data `self.dt_patterns`
            // so pass the built-in `DATETIME_PARSE_DATAS`.
            // Use the extra data returned by `find_datetime_in_line` to set `self.dt_patterns` once.
            // This will only happen once per `SyslineReader` (assuming a valid Syslog file)
            let result = SyslineReader::find_datetime_in_line(line, &DATETIME_PARSE_DATAS_VEC, self.path(), charsz, &self.tz_offset);
            let (datetime_parse_data, dt) = match result {
                Ok(val) => val,
                Err(err) => {
                    debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Err {};", sx(), self, err);
                    return Err(err);
                }
            };
            self.dt_patterns_update(datetime_parse_data.clone());
            debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Ok;", sx(), self);
            return Result_ParseDateTime::Ok((datetime_parse_data.siba, datetime_parse_data.siea, dt));
        }
        debug_eprintln!("{}parse_datetime_in_line self.dt_patterns has {} entries", so(), &self.dt_patterns.len());
        // have already determined DateTime formatting for this file, so
        // no need to try *all* built-in DateTime formats, just try the known good formats `self.dt_patterns`
        let result = SyslineReader::find_datetime_in_line(line, &self.dt_patterns, self.path(), charsz, &self.tz_offset);
        let (datetime_parse_data, dt) = match result {
            Ok(val) => val,
            Err(err) => {
                if self.analyzed {
                    debug_eprintln!(
                        "{}parse_datetime_in_line(SyslineReader@{:p}) return Err {};",
                        sx(),
                        self,
                        err
                    );
                    return Err(err);
                }
                // The known good format failed and this SyslineReader has not yet run `dt_format_analysis`
                // so now try other default formats. This is a resource expensive operation.
                debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Err {}; try again using default DATETIME_PARSE_DATAS_VEC", so(), self, err);
                match SyslineReader::find_datetime_in_line(line, &DATETIME_PARSE_DATAS_VEC, self.path(), charsz, &self.tz_offset) {
                    Ok((datetime_parse_data_, dt_)) => {
                        self.dt_patterns_update(datetime_parse_data_.clone());
                        (datetime_parse_data_, dt_)
                    }
                    Err(err_) => {
                        debug_eprintln!(
                            "{}parse_datetime_in_line(SyslineReader@{:p}) return Err {};",
                            sx(),
                            self,
                            err_
                        );
                        return Err(err_);
                    }
                }
            }
        };
        debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Ok;", sx(), self);
        return Result_ParseDateTime::Ok((datetime_parse_data.sib, datetime_parse_data.sie, dt));
    }

    /// Find first sysline at or after `fileoffset`.
    /// return (fileoffset of start of _next_ sysline, found Sysline at or after `fileoffset`)
    /// Similar to `find_line`, `read_block`.
    /// This is the heart of the algorithm to find a sysline within a syslog file quickly.
    /// It's simply a binary search.
    /// It could definitely use some improvements, but for now it gets the job done.
    /// XXX: this function is large and cumbersome. you've been warned.
    pub fn find_sysline(&mut self, fileoffset: FileOffset) -> ResultS4_SyslineFind {
        debug_eprintln!("{}find_sysline(SyslineReader@{:p}, {})", sn(), self, fileoffset);

        // TODO: make these comparison values into consts
        if self.linereader.blockreader.count_bytes() > 0x4000 && self.count() < 3 {
            debug_eprintln!("{}find_sysline(SyslineReader@{:p}); too many bytes analyzed {}, yet too few syslines {}", sn(), self, self.linereader.blockreader.count_bytes(), self.count());
            // TODO: [2022/04/06] need to implement a way to abandon processing a file.
            //return Result_ParseDateTime::Error("");
        }

        { // check if `fileoffset` is already known about

            // check LRU cache
            match self._find_sysline_lru_cache.get(&fileoffset) {
                Some(rlp) => {
                    // self.stats_read_block_cache_lru_hit += 1;
                    debug_eprintln!("{}find_sysline: found LRU cached for fileoffset {}", so(), fileoffset);
                    match rlp {
                        ResultS4_SyslineFind::Found(val) => {
                            debug_eprintln!("{}return ResultS4_SyslineFind::Found(({}, …)) @[{}, {}] from LRU cache", sx(), val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                            return ResultS4_SyslineFind::Found((val.0, val.1.clone()));
                        }
                        ResultS4_SyslineFind::Found_EOF(val) => {
                            debug_eprintln!("{}return ResultS4_SyslineFind::Found_EOF(({}, …)) @[{}, {}] from LRU cache", sx(), val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                            return ResultS4_SyslineFind::Found_EOF((val.0, val.1.clone()));
                        }
                        ResultS4_SyslineFind::Done => {
                            debug_eprintln!("{}return ResultS4_SyslineFind::Done from LRU cache", sx());
                            return ResultS4_SyslineFind::Done;
                        }
                        ResultS4_SyslineFind::Err(err) => {
                            debug_eprintln!("{}Error {}", so(), err);
                            eprintln!("ERROR: unexpected value store in _find_line_lru_cache, fileoffset {} error {}", fileoffset, err);
                        }
                    }
                }
                None => {
                    //self.stats_read_block_cache_lru_miss += 1;
                    debug_eprintln!("{}find_sysline: fileoffset {} not found in LRU cache", so(), fileoffset);
                }
            }

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
                    let fo_next = (*slp).fileoffset_next() + (self.charsz() as FileOffset);
                    if self.is_sysline_last(&slp) {
                        debug_eprintln!(
                        "{}find_sysline: return ResultS4_SyslineFind::Found_EOF(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                        sx(),
                        fo_next,
                        &*slp,
                        (*slp).fileoffset_begin(),
                        (*slp).fileoffset_end(),
                        (*slp).to_String_noraw()
                    );
                        self._find_sysline_lru_cache
                            .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_next, slp.clone())));
                        return ResultS4_SyslineFind::Found_EOF((fo_next, slp));
                    }
                    self._find_sysline_lru_cache
                        .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
                    debug_eprintln!(
                    "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                    sx(),
                    fo_next,
                    &*slp,
                    (*slp).fileoffset_begin(),
                    (*slp).fileoffset_end(),
                    (*slp).to_String_noraw()
                );
                    return ResultS4_SyslineFind::Found((fo_next, slp));
                }
                None => {
                    debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines_by_range", so(), fileoffset);
                }
            }
            debug_eprintln!("{}find_sysline: searching for first sysline datetime A …", so());

            // check if there is a Sysline already known at this fileoffset
            // XXX: not necessary to check `self.syslines` since `self.syslines_by_range` is checked.
            if self.syslines.contains_key(&fileoffset) {
                debug_assert!(self.syslines_by_range.contains_key(&fileoffset), "self.syslines.contains_key({}) however, self.syslines_by_range.contains_key({}) returned None (syslines_by_range out of synch)", fileoffset, fileoffset);
                debug_eprintln!("{}find_sysline: hit self.syslines for FileOffset {}", so(), fileoffset);
                let slp = self.syslines[&fileoffset].clone();
                // XXX: multi-byte character encoding
                let fo_next = (*slp).fileoffset_end() + (self.charsz() as FileOffset);
                // TODO: determine if `fileoffset` is the last sysline of the file
                //       should add a private helper function for this task `is_sysline_last(FileOffset)` ... something like that
                debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines {:?}",
                sx(),
                fo_next,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).fileoffset_end(),
                (*slp).to_String_noraw()
            );
                self._find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
                return ResultS4_SyslineFind::Found((fo_next, slp));
            } else {
                debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines", so(), fileoffset);
            }
        }

        //
        // find line with datetime A
        //

        let mut fo_a: FileOffset = 0;
        let mut fo1: FileOffset = fileoffset;
        let mut sl = Sysline::new();
        loop {
            debug_eprintln!("{}find_sysline: self.linereader.find_line({})", so(), fo1);
            let result: ResultS4_LineFind = self.linereader.find_line(fo1);
            let eof = result.is_eof();
            let (fo2, lp) = match result {
                ResultS4_LineFind::Found((fo_, lp_)) | ResultS4_LineFind::Found_EOF((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: A FileOffset {} Line @{:p} len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    (fo_, lp_)
                }
                ResultS4_LineFind::Done => {
                    debug_eprintln!("{}find_sysline: LRU cache put({}, Done)", so(), fileoffset);
                    self._find_sysline_lru_cache.put(fileoffset, ResultS4_SyslineFind::Done);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Done; A from LineReader.find_line({})", sx(), fo1);
                    return ResultS4_SyslineFind::Done;
                }
                ResultS4_LineFind::Err(err) => {
                    eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Err({}); A from LineReader.find_line({})", sx(), err, fo1);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            let result = self.parse_datetime_in_line(&*lp, &self.charsz());
            debug_eprintln!("{}find_sysline: A find_datetime_in_line returned {:?}", so(), result);
            match result {
                Err(_) => {}
                Ok((dt_beg, dt_end, dt)) => {
                    // a datetime was found! beginning of a sysline
                    fo_a = fo1;
                    sl.dt_beg = dt_beg;
                    sl.dt_end = dt_end;
                    sl.dt = Some(dt);
                    debug_eprintln!("{}find_sysline: A sl.push({:?})", so(), (*lp).to_String_noraw());
                    sl.push(lp);
                    fo1 = sl.fileoffset_end() + (self.charsz() as FileOffset);
                    // sanity check
                    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}", dt_beg, dt_end);
                    debug_assert_lt!(dt_end, fo1 as usize, "bad dt_end {} fileoffset+charsz {}", dt_end, fo1 as usize);
                    if eof {
                        let slp = SyslineP::new(sl);
                        debug_eprintln!("{}find_sysline: LRU cache put({}, Found_EOF({}, …))", so(), fileoffset, fo1);
                        self._find_sysline_lru_cache
                            .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo1, slp.clone())));
                        debug_eprintln!(
                            "{}find_sysline: return ResultS4_SyslineFind::Found_EOF({}, {:p}) @[{}, {}]; A found here and LineReader.find_line({})",
                            sx(),
                            fo1,
                            &(*slp),
                            (*slp).fileoffset_begin(),
                            (*slp).fileoffset_end(),
                            fo1,
                        );
                        return ResultS4_SyslineFind::Found_EOF((fo1, slp));
                    }
                    break;
                }
            }
            debug_eprintln!("{}find_sysline: A skip push Line {:?}", so(), (*lp).to_String_noraw());
            fo1 = fo2;
        }

        debug_eprintln!(
            "{}find_sysline: found line with datetime A at FileOffset {}, searching for datetime B starting at fileoffset {} …",
            so(),
            fo_a,
            fo1
        );

        //
        // find line with datetime B
        //

        { // check if sysline at `fo1` is already known about
        /*
            // XXX: not necessary to check `self.syslines` since `self.syslines_by_range` is checked.
            // check if there is a Sysline already known at this fileoffset
            if self.syslines.contains_key(&fo1) {
                debug_assert!(self.syslines_by_range.contains_key(&fo1), "self.syslines.contains_key({}) however, self.syslines_by_range.contains_key({}); syslines_by_range out of synch", fo1, fo1);
                debug_eprintln!("{}find_sysline: hit self.syslines for FileOffset {}", so(), fo1);
                let slp = self.syslines[&fo1].clone();
                // XXX: multi-byte character encoding
                let fo_next = (*slp).fileoffset_end() + (self.charsz() as FileOffset);
                // TODO: determine if `fileoffset` is the last sysline of the file
                //       should add a private helper function for this task `is_sysline_last(FileOffset)` ... something like that
                debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines {:?}",
                sx(),
                fo_next,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).fileoffset_end(),
                (*slp).to_String_noraw()
            );
                self._find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
                return ResultS4_SyslineFind::Found((fo_next, slp));
            } else {
                debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines", so(), fileoffset);
            }
            // check if the offset is already in a known range
            match self.syslines_by_range.get_key_value(&fo1) {
                Some(range_fo) => {
                    let range = range_fo.0;
                    debug_eprintln!(
                    "{}find_sysline: hit syslines_by_range cache for FileOffset {} (found in range {:?})",
                    so(),
                    fo1,
                    range
                );
                    let fo = range_fo.1;
                    let slp = self.syslines[fo].clone();
                    // XXX: multi-byte character encoding
                    let fo_next = (*slp).fileoffset_next() + (self.charsz() as FileOffset);
                    if self.is_sysline_last(&slp) {
                        debug_eprintln!(
                            "{}find_sysline: return ResultS4_SyslineFind::Found_EOF(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                            sx(),
                            fo_next,
                            &*slp,
                            (*slp).fileoffset_begin(),
                            (*slp).fileoffset_end(),
                            (*slp).to_String_noraw()
                        );
                        self._find_sysline_lru_cache
                            .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_next, slp.clone())));
                        return ResultS4_SyslineFind::Found_EOF((fo_next, slp));
                    }
                    self._find_sysline_lru_cache
                        .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
                    debug_eprintln!(
                        "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                        sx(),
                        fo_next,
                        &*slp,
                        (*slp).fileoffset_begin(),
                        (*slp).fileoffset_end(),
                        (*slp).to_String_noraw()
                    );
                    return ResultS4_SyslineFind::Found((fo_next, slp));
                }
                None => {
                    debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines_by_range", so(), fileoffset);
                }
            }
            debug_eprintln!("{}find_sysline: searching for first sysline datetime B …", so());
        */
        }

        let mut fo_b: FileOffset = fo1;
        let mut eof = false;
        loop {
            debug_eprintln!("{}find_sysline: self.linereader.find_line({})", so(), fo1);
            let result = self.linereader.find_line(fo1);
            let (fo2, lp) = match result {
                ResultS4_LineFind::Found((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: B got Found(FileOffset {}, Line @{:p}) len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    //assert!(!eof, "ERROR: find_line returned EOF as true yet returned Found()");
                    (fo_, lp_)
                }
                ResultS4_LineFind::Found_EOF((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: B got Found_EOF(FileOffset {} Line @{:p}) len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    eof = true;
                    //assert!(!eof, "ERROR: find_line returned EOF as true yet returned Found_EOF()");
                    (fo_, lp_)
                }
                ResultS4_LineFind::Done => {
                    //debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Done; B", sx());
                    debug_eprintln!("{}find_sysline: break; B", so());
                    eof = true;
                    break;
                }
                ResultS4_LineFind::Err(err) => {
                    eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Err({}); B from LineReader.find_line({})", sx(), err, fo1);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            let result = self.parse_datetime_in_line(&*lp, &self.charsz());
            debug_eprintln!("{}find_sysline: B find_datetime_in_line returned {:?}", so(), result);
            match result {
                Err(_) => {
                    debug_eprintln!(
                        "{}find_sysline: B append found Line to this Sysline sl.push({:?})",
                        so(),
                        (*lp).to_String_noraw()
                    );
                    sl.push(lp);
                }
                Ok(_) => {
                    // a datetime was found! end of this sysline, beginning of a new sysline
                    debug_eprintln!(
                        "{}find_sysline: B found datetime; end of this Sysline. Do not append found Line {:?}",
                        so(),
                        (*lp).to_String_noraw()
                    );
                    fo_b = fo1;
                    break;
                }
            }
            fo1 = fo2;
        }

        debug_eprintln!("{}find_sysline: found line with datetime B at FileOffset {}", so(), fo_b);

        let slp = self.insert_sysline(sl);
        if eof {
            debug_eprintln!("{}find_sysline: LRU cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_b);
            self._find_sysline_lru_cache
                .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_b, slp.clone())));
            debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Found_EOF(({}, SyslineP@{:p}) @[{}, {}] E {:?}",
                sx(),
                fo_b,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).fileoffset_end(),
                (*slp).to_String_noraw()
            );
            return ResultS4_SyslineFind::Found_EOF((fo_b, slp));
        }
        debug_eprintln!("{}find_sysline: LRU cache put({}, Found({}, …))", so(), fileoffset, fo_b);
        self._find_sysline_lru_cache
            .put(fileoffset, ResultS4_SyslineFind::Found((fo_b, slp.clone())));
        debug_eprintln!(
            "{}find_sysline: return ResultS4_SyslineFind::Found(({}, SyslineP@{:p}) @[{}, {}] E {:?}",
            sx(),
            fo_b,
            &*slp,
            (*slp).fileoffset_begin(),
            (*slp).fileoffset_end(),
            (*slp).to_String_noraw()
        );
        return ResultS4_SyslineFind::Found((fo_b, slp));
    }

    /// wrapper to call each implementation of `find_sysline_at_datetime_filter`
    pub fn find_sysline_at_datetime_filter(
        &mut self, fileoffset: FileOffset, dt_filter: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        self.find_sysline_at_datetime_filter1(fileoffset, dt_filter)
    }

    /// find first sysline at or after `fileoffset` that is at or after `dt_filter`
    ///
    /// for example, given syslog file with datetimes:
    ///     20010101
    ///     20010102
    ///     20010103
    /// where the newline ending the first line is the ninth byte (fileoffset 9)
    ///
    /// calling
    ///     syslinereader.find_sysline_at_datetime_filter(0, Some(20010102 00:00:00-0000))
    /// will return
    ///     ResultS4::Found(19, SyslineP(data='20010102␊'))
    ///
    /// TODO: add more of these examples
    ///
    /// XXX: this function is large, cumbersome, and messy
    fn find_sysline_at_datetime_filter1(
        &mut self, fileoffset: FileOffset, dt_filter: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        let _fname = "find_sysline_at_datetime_filter1";
        debug_eprintln!("{}{}(SyslineReader@{:p}, {}, {:?})", sn(), _fname, self, fileoffset, dt_filter);
        let filesz = self.filesz();
        let _fo_end: FileOffset = filesz as FileOffset;
        let mut try_fo: FileOffset = fileoffset;
        let mut try_fo_last: FileOffset = try_fo;
        let mut fo_last: FileOffset = fileoffset;
        let mut slp_opt: Option<SyslineP> = None;
        let mut slp_opt_last: Option<SyslineP> = None;
        let mut fo_a: FileOffset = fileoffset; // begin "range cursor" marker
        let mut fo_b: FileOffset = _fo_end; // end "range cursor" marker
        loop {
            // TODO: [2021/09/26]
            //       this could be faster.
            //       currently it narrowing down to a byte offset
            //       but it only needs to narrow down to offsets within range of one sysline
            //       so if `fo_a` and `fo_b` are in same sysline range, then this can return that sysline.
            //       Also, add stats for this function and debug print those stats before exiting.
            //       i.e. count of loops, count of calls to sysline_dt_before_after, etc.
            //       do this before tweaking function so can be compared
            debug_eprintln!("{}{}: loop(…)!", so(), _fname);
            let result = self.find_sysline(try_fo);
            let eof = result.is_eof();
            let done = result.is_done();
            match result {
                ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                    if !eof {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(try_fo: {}) returned ResultS4_SyslineFind::Found({}, …) A",
                            so(),
                            _fname,
                            try_fo,
                            fo
                        );
                    } else {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(try_fo: {}) returned ResultS4_SyslineFind::Found_EOF({}, …) B",
                            so(),
                            _fname,
                            try_fo,
                            fo
                        );
                    }
                    debug_eprintln!(
                        "{}{}: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?} C",
                        so(),
                        _fname,
                        fo,
                        &(*slp),
                        slp.lines.len(),
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
                                _fo_end
                            );
                            debug_eprintln!(
                                "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); A",
                                sx(),
                                _fname,
                                fo,
                                &*slp
                            );
                            return ResultS4_SyslineFind::Found((fo, slp));
                        } // end Pass
                        Result_Filter_DateTime1::OccursAtOrAfter => {
                            // the Sysline found by `find_sysline(try_fo)` occurs at or after filter `dt_filter`, so search backward
                            // i.e. move end marker `fo_b` backward
                            debug_eprintln!("{}{}: OccursAtOrAfter => fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
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
                                    "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); B fileoffset {} {:?}",
                                    sx(),
                                    _fname,
                                    fo,
                                    &*slp,
                                    (*slp).fileoffset_begin(),
                                    (*slp).to_String_noraw()
                                );
                                return ResultS4_SyslineFind::Found((fo, slp));
                            }
                            try_fo_last = try_fo;
                            fo_b = std::cmp::min((*slp).fileoffset_begin(), try_fo_last);
                            debug_eprintln!(
                                "{}{}:                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                so(),
                                _fname,
                                fo_a,
                                fo_b,
                                fo_a
                            );
                            assert_le!(fo_a, fo_b, "Unexpected values for fo_a {} fo_b {}, FPath {:?}", fo_a, fo_b, self.path());
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursAtOrAfter
                        Result_Filter_DateTime1::OccursBefore => {
                            // the Sysline found by `find_sysline(try_fo)` occurs before filter `dt_filter`, so search forthward
                            // i.e. move begin marker `fo_a` forthward
                            debug_eprintln!("{}{}: OccursBefore =>    fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                            let slp_foe = (*slp).fileoffset_end();
                            // XXX: [2022/03/25] why was this `assert_le` here? It seems wrong.
                            //assert_le!(slp_foe, fo, "unexpected values (SyslineP@{:p}).fileoffset_end() {}, fileoffset returned by self.find_sysline({}) was {} FPath {:?}", slp, slp_foe, try_fo, fo, self.path());
                            try_fo_last = try_fo;
                            assert_le!(try_fo_last, slp_foe, "Unexpected values try_fo_last {} slp_foe {}, last tried offset (passed to self.find_sysline({})) is beyond returned Sysline@{:p}.fileoffset_end() {}!? FPath {:?}", try_fo_last, slp_foe, try_fo, slp, slp_foe, self.path());
                            debug_eprintln!(
                                "{}{}:                    ∴ fo_a = min(slp_foe {}, fo_b {});",
                                so(),
                                _fname,
                                slp_foe,
                                fo_b
                            );
                            // LAST WORKING HERE [2021/10/06 00:05:00]
                            // LAST WORKING HERE [2022/03/16 01:15:00]
                            // this code passes all tests, but runs strangely. I think the problem is the first found sysline
                            // (that may or may not satisfy the passed filter) is placed into a queue and then printed by the waiting main thread.
                            fo_a = std::cmp::min(slp_foe, fo_b);
                            //fo_a = std::cmp::max(slp_foe, fo_b);
                            //fo_a = slp_foe;
                            //assert_le!(fo_a, fo_b, "Unexpected values for fo_a {} fo_b {}", fo_a, fo_b);
                            debug_eprintln!(
                                "{}{}:                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                so(),
                                _fname,
                                fo_a,
                                fo_b,
                                fo_a
                            );
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursBefore
                    } // end SyslineReader::sysline_dt_after_or_before()
                    debug_eprintln!("{}{}:                    fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                    fo_last = fo;
                    slp_opt_last = slp_opt;
                    slp_opt = Some(slp);
                    // TODO: [2021/09/26]
                    //       I think could do an early check and skip a few loops:
                    //       if `fo_a` and `fo_b` are offsets into the same Sysline
                    //       then that Sysline is the candidate, so return Ok(...)
                    //       unless `fo_a` and `fo_b` are past last Sysline.fileoffset_begin of the file then return Done
                } // end Found | Found_EOF
                ResultS4_SyslineFind::Done => {
                    debug_eprintln!("{}{}: SyslineReader.find_sysline(try_fo: {}) returned Done", so(), _fname, try_fo);
                    debug_eprintln!(
                        "{}{}:                 try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})",
                        so(),
                        _fname,
                        try_fo,
                        try_fo_last,
                        fo_a,
                        fo_b,
                        _fo_end
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
                        "{}{}:                 try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})",
                        so(),
                        _fname,
                        try_fo,
                        try_fo_last,
                        fo_a,
                        fo_b,
                        _fo_end
                    );
                } // end Done
                ResultS4_SyslineFind::Err(err) => {
                    debug_eprintln!(
                        "{}{}: SyslineReader.find_sysline(try_fo: {}) returned Err({})",
                        so(),
                        _fname,
                        try_fo,
                        err
                    );
                    eprintln!("ERROR: {}", err);
                    break;
                } // end Err
            } // match result
            debug_eprintln!("{}{}: next loop will try offset {} (fo_end {})", so(), _fname, try_fo, _fo_end);

            // TODO: 2022/03/18 this latter part hints at a check that could be done sooner,
            //       before `try_fo==try_fo_last`, that would result in a bit less loops.
            //       A simpler and faster check is to do
            //           fo_next, slp = find_sysline(fileoffset)
            //           _, slp_next = find_sysline(fo_next)
            //       do this at the top of the loop. Then call `dt_after_or_before` for each
            //       `.dt` among `slp`, `slp_next`.

            // `try_fo == try_fo_last` means binary search loop is deciding on the same fileoffset upon each loop.
            // the searching is exhausted.
            if done && try_fo == try_fo_last {
                // reached a dead-end of searching the same fileoffset `find_sysline(try_fo)` and receiving Done
                // so this function is exhausted too.
                debug_eprintln!("{}{}: Done && try_fo {} == {} try_fo_last; break!", so(), _fname, try_fo, try_fo_last);
                break;
            } else if try_fo != try_fo_last {
                continue;
            }
            debug_eprintln!("{}{}: try_fo {} == {} try_fo_last;", so(), _fname, try_fo, try_fo_last);
            let mut slp = slp_opt.unwrap();
            let fo_beg = slp.fileoffset_begin();
            if self.is_sysline_last(&slp) && fo_beg < try_fo {
                // binary search stopped at fileoffset past start of last Sysline in file
                // so entirely past all acceptable syslines
                debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done; C binary searched ended after beginning of last sysline in the file", sx(), _fname,);
                return ResultS4_SyslineFind::Done;
            }
            // binary search loop is deciding on the same fileoffset upon each loop. That fileoffset must refer to
            // an acceptable sysline. However, if that fileoffset is past `slp.fileoffset_begin` than the threshold
            // change of datetime for the `dt_filter` is the *next* Sysline.
            let fo_next = slp.fileoffset_next();
            // XXX: sanity check
            //debug_assert_eq!(fo_last, fo_next, "fo {} != {} slp.fileoffset_next()", fo_last, fo_next);
            if fo_beg < try_fo {
                debug_eprintln!("{}{}: slp.fileoffset_begin() {} < {} try_fo;", so(), _fname, fo_beg, try_fo);
                let slp_next = match self.find_sysline(fo_next) {
                    ResultS4_SyslineFind::Found_EOF((_, slp_)) => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Found_EOF(…, {:?})",
                            so(),
                            _fname,
                            fo_next,
                            slp_
                        );
                        slp_
                    }
                    ResultS4_SyslineFind::Found((_, slp_)) => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Found(…, {:?})",
                            so(),
                            _fname,
                            fo_next,
                            slp_
                        );
                        slp_
                    }
                    ResultS4_SyslineFind::Done => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) unexpectedly returned Done",
                            so(),
                            _fname,
                            fo_next
                        );
                        break;
                    }
                    ResultS4_SyslineFind::Err(err) => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Err({})",
                            so(),
                            _fname,
                            fo_next,
                            err
                        );
                        eprintln!("ERROR: {}", err);
                        break;
                    }
                };
                debug_eprintln!("{}{}: dt_filter:                   {:?}", so(), _fname, dt_filter);
                debug_eprintln!(
                    "{}{}: slp      : fo_beg {:3}, fo_end {:3} {:?} {:?}",
                    so(),
                    _fname,
                    fo_beg,
                    (*slp).fileoffset_end(),
                    (*slp).dt.unwrap(),
                    (*slp).to_String_noraw()
                );
                debug_eprintln!(
                    "{}{}: slp_next : fo_beg {:3}, fo_end {:3} {:?} {:?}",
                    so(),
                    _fname,
                    (*slp_next).fileoffset_begin(),
                    (*slp_next).fileoffset_end(),
                    (*slp_next).dt.unwrap(),
                    (*slp_next).to_String_noraw()
                );
                let slp_compare = Self::dt_after_or_before(&(*slp).dt.unwrap(), dt_filter);
                let slp_next_compare = Self::dt_after_or_before(&(*slp_next).dt.unwrap(), dt_filter);
                debug_eprintln!("{}{}: match({:?}, {:?})", so(), _fname, slp_compare, slp_next_compare);
                slp = match (slp_compare, slp_next_compare) {
                    (_, Result_Filter_DateTime1::Pass) | (Result_Filter_DateTime1::Pass, _) => {
                        debug_eprintln!("{}{}: unexpected Result_Filter_DateTime1::Pass", so(), _fname);
                        eprintln!("ERROR: unexpected Result_Filter_DateTime1::Pass result");
                        break;
                    }
                    (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursBefore) => {
                        debug_eprintln!("{}{}: choosing slp_next", so(), _fname);
                        slp_next
                    }
                    (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursAtOrAfter) => {
                        debug_eprintln!("{}{}: choosing slp_next", so(), _fname);
                        slp_next
                    }
                    (Result_Filter_DateTime1::OccursAtOrAfter, Result_Filter_DateTime1::OccursAtOrAfter) => {
                        debug_eprintln!("{}{}: choosing slp", so(), _fname);
                        slp
                    }
                    _ => {
                        debug_eprintln!(
                            "{}{}: unhandled (Result_Filter_DateTime1, Result_Filter_DateTime1) tuple",
                            so(),
                            _fname
                        );
                        eprintln!("ERROR: unhandled (Result_Filter_DateTime1, Result_Filter_DateTime1) tuple");
                        break;
                    }
                };
            } else {
                debug_eprintln!(
                    "{}{}: slp.fileoffset_begin() {} >= {} try_fo; use slp",
                    so(),
                    _fname,
                    fo_beg,
                    try_fo
                );
            }
            let fo_ = slp.fileoffset_next();
            debug_eprintln!(
                "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); D fileoffset {} {:?}",
                sx(),
                _fname,
                fo_,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).to_String_noraw()
            );
            return ResultS4_SyslineFind::Found((fo_, slp));
        } // end loop

        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done; E", sx(), _fname);
        return ResultS4_SyslineFind::Done;
    }

    fn find_sysline_at_datetime_filter2(
        &mut self, fileoffset: FileOffset, dt_filter: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        let _fname = "find_sysline_at_datetime_filter2";
        debug_eprintln!("{}{}(SyslineReader@{:p}, {}, {:?})", sn(), _fname, self, fileoffset, dt_filter);
        let filesz = self.filesz();
        let _fo_end: FileOffset = filesz as FileOffset;

        // TODO: complete this second attempt

        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done; E", sx(), _fname);
        return ResultS4_SyslineFind::Done;
    }

    /// if `dt` is at or after `dt_filter` then return `OccursAtOrAfter`
    /// if `dt` is before `dt_filter` then return `OccursBefore`
    /// else return `Pass` (including if `dt_filter` is `None`)
    pub fn dt_after_or_before(dt: &DateTimeL, dt_filter: &DateTimeL_Opt) -> Result_Filter_DateTime1 {
        if dt_filter.is_none() {
            debug_eprintln!("{}dt_after_or_before(…) return Result_Filter_DateTime1::Pass; (no dt filters)", snx(),);
            return Result_Filter_DateTime1::Pass;
        }

        let dt_a = &dt_filter.unwrap();
        debug_eprintln!("{}dt_after_or_before comparing dt datetime {:?} to filter datetime {:?}", sn(), dt, dt_a);
        if dt < dt_a {
            debug_eprintln!("{}dt_after_or_before(…) return Result_Filter_DateTime1::OccursBefore; (dt {:?} is before dt_filter {:?})", sx(), dt, dt_a);
            return Result_Filter_DateTime1::OccursBefore;
        }
        debug_eprintln!("{}dt_after_or_before(…) return Result_Filter_DateTime1::OccursAtOrAfter; (dt {:?} is at or after dt_filter {:?})", sx(), dt, dt_a);
        return Result_Filter_DateTime1::OccursAtOrAfter;
    }

    /// convenience wrapper for `dt_after_or_before`
    pub fn sysline_dt_after_or_before(syslinep: &SyslineP, dt_filter: &DateTimeL_Opt) -> Result_Filter_DateTime1 {
        debug_eprintln!("{}sysline_dt_after_or_before(SyslineP@{:p}, {:?})", snx(), &*syslinep, dt_filter,);
        assert!((*syslinep).dt.is_some(), "Sysline@{:p} does not have a datetime set.", &*syslinep);

        let dt = (*syslinep).dt.unwrap();
        return Self::dt_after_or_before(&dt, dt_filter);
    }

    /// If both filters are `Some` and `syslinep.dt` is "between" the filters then return `Pass`
    /// comparison is "inclusive" i.e. `dt` == `dt_filter_after` will return `Pass`
    /// If both filters are `None` then return `Pass`
    /// TODO: finish this docstring
    pub fn dt_pass_filters(
        dt: &DateTimeL, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
    ) -> Result_Filter_DateTime2 {
        debug_eprintln!("{}dt_pass_filters({:?}, {:?}, {:?})", sn(), dt, dt_filter_after, dt_filter_before,);
        if dt_filter_after.is_none() && dt_filter_before.is_none() {
            debug_eprintln!(
                "{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange; (no dt filters)",
                sx(),
            );
            return Result_Filter_DateTime2::OccursInRange;
        }
        if dt_filter_after.is_some() && dt_filter_before.is_some() {
            debug_eprintln!(
                "{}dt_pass_filters comparing datetime dt_filter_after {:?} < {:?} dt < {:?} dt_fiter_before ???",
                so(),
                &dt_filter_after.unwrap(),
                dt,
                &dt_filter_before.unwrap()
            );
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
            debug_eprintln!(
                "{}dt_pass_filters comparing datetime dt_filter_after {:?} < {:?} dt ???",
                so(),
                &dt_filter_after.unwrap(),
                dt
            );
            let da = &dt_filter_after.unwrap();
            if dt < da {
                debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursBeforeRange;", sx());
                return Result_Filter_DateTime2::OccursBeforeRange;
            }
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::OccursInRange;", sx());
            return Result_Filter_DateTime2::OccursInRange;
        } else {
            debug_eprintln!(
                "{}dt_pass_filters comparing datetime dt {:?} < {:?} dt_filter_before ???",
                so(),
                dt,
                &dt_filter_before.unwrap()
            );
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
        syslinep: &SyslineP, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
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

    /// find the first `Sysline`, starting at `fileoffset`, that is at or after datetime filter
    /// `dt_filter_after` and before datetime filter `dt_filter_before`
    pub fn find_sysline_between_datetime_filters(
        &mut self, fileoffset: FileOffset, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        let _fname = "find_sysline_between_datetime_filters";
        debug_eprintln!("{}{}({}, {:?}, {:?})", sn(), _fname, fileoffset, dt_filter_after, dt_filter_before);

        match self.find_sysline_at_datetime_filter(fileoffset, dt_filter_after) {
            ResultS4_SyslineFind::Found((fo, slp)) => {
                debug_eprintln!(
                "{}{}: find_sysline_at_datetime_filter returned ResultS4_SyslineFind::Found(({}, {:?})); call sysline_pass_filters",
                    so(),
                    _fname,
                    fo,
                    slp,
                );
                match Self::sysline_pass_filters(&slp, dt_filter_after, dt_filter_before) {
                    Result_Filter_DateTime2::OccursInRange => {
                        debug_eprintln!("{}{}: sysline_pass_filters(…) returned OccursInRange;", so(), _fname);
                        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Found(({}, {:?}))", sx(), _fname, fo, slp);
                        return ResultS4_SyslineFind::Found((fo, slp));
                    },
                    Result_Filter_DateTime2::OccursBeforeRange => {
                        debug_eprintln!("{}{}: sysline_pass_filters(…) returned OccursBeforeRange;", so(), _fname);
                        eprintln!("ERROR: sysline_pass_filters(Sysline@{:p}, {:?}, {:?}) returned OccursBeforeRange, however the prior call to find_sysline_at_datetime_filter({}, {:?}) returned Found; this is unexpected.",
                                  slp, dt_filter_after, dt_filter_before,
                                  fileoffset, dt_filter_after
                        );
                        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done (not sure what to do here)", sx(), _fname);
                        return ResultS4_SyslineFind::Done; 
                    },
                    Result_Filter_DateTime2::OccursAfterRange => {
                        debug_eprintln!("{}{}: sysline_pass_filters(…) returned OccursAfterRange;", so(), _fname);
                        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done", sx(), _fname);
                        return ResultS4_SyslineFind::Done;
                    },
                };
            },
            ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                debug_eprintln!("{}{}: return ResultS4_SyslineFind::Found_EOF(({}, {:?}))", sx(), _fname, fo, slp);
                return ResultS4_SyslineFind::Found_EOF((fo, slp));
            },
            ResultS4_SyslineFind::Done => {},
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!(
                    "{}{}: find_sysline_at_datetime_filter({}, dt_after: {:?}) returned Err({})",
                    so(),
                    _fname,
                    fileoffset,
                    dt_filter_after,
                    err,
                );
                eprintln!("ERROR: {}", err);
                debug_eprintln!("{}{}: return ResultS4_SyslineFind::Err({})", sx(), _fname, err);
                return ResultS4_SyslineFind::Err(err);
            },
        };

        debug_eprintln!("{}{} return ResultS4_SyslineFind::Done", sx(), _fname);
        ResultS4_SyslineFind::Done
    }
    
    /// return an up-to-date `Summary` instance for this `SyslineReader`
    fn summary(&self) -> Summary {
        let bytes = self.linereader.blockreader.count_bytes();
        let bytes_total = self.linereader.blockreader.filesz as u64;
        let blocks = self.linereader.blockreader.count();
        let blocks_total = self.linereader.blockreader.blockn;
        let blocksz = self.blocksz();
        let lines = self.linereader.count();
        let syslines = self.count();
        return Summary::new(bytes, bytes_total, blocks, blocks_total, blocksz, lines, syslines);
    }
}

/// thread-safe Atomic Reference Counting Pointer to a `SyslineReader`
/// XXX: should the `&` be removed? (completely move the SyslineReader?)
type SyslineReaderP<'syslinereader> = Arc<&'syslinereader SyslineReader<'syslinereader>>;

#[test]
fn test_datetime_from_str_workaround_Issue660() {
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("", ""));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("a", ""));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("", "a"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660(" ", ""));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("", " "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" ", " "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" a", " a"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660(" a", "  a"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("  a", " a"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("  a", "   a"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("a", "   a"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("  a", "a"));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("a ", "a "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("a  ", "a  "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" a ", " a "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" a  ", " a  "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("   a  ", "   a  "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660("   a  ", "   a  "));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("   a  ", "   a   "));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("   a   ", "   a  "));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("   a   ", " a  "));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("a   ", " a  "));

    assert!(!SyslineReader::datetime_from_str_workaround_Issue660(" \t", " "));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660(" ", "\t "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \t", "\t "));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("\t ", "\t a\t"));

    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n\t", " \n\t"));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n\t", " \t\n"));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n\ta", " \t\n"));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n\t", " \t\na"));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n", " \n"));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n", "\n "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n", "\r "));
    assert!(SyslineReader::datetime_from_str_workaround_Issue660(" \n", " \n"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("\t a", "\t a\t\n"));
    assert!(!SyslineReader::datetime_from_str_workaround_Issue660("\t\n a\n", "\t\n a\t\n"));
}

/// basic test of `SyslineReader.find_datetime_in_line`
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_find_datetime_in_line_by_block(blocksz: BlockSz) {
    debug_eprintln!("{}_test_find_datetime_in_line_by_block()", sn());

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

    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = match SyslineReader::new(&path, blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({:?}, {}) failed {}", &path, blocksz, err);
            return;
        }
    };

    let mut fo1: FileOffset = 0;
    loop {
        let result = slr.find_sysline(fo1);
        let done = result.is_done() || result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                debug_eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Found|Found_EOF({}, @{:p})", so(), fo1, fo, &*slp);
                debug_eprintln!(
                    "{}test_find_datetime_in_line: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print_slp(&slp);
                fo1 = fo;
            }
            ResultS4_SyslineFind::Done => {
                debug_eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Done", so(), fo1);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Err({})", so(), fo1, err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
        if done {
            break;
        }
    }

    debug_eprintln!("{}_test_find_datetime_in_line_by_block()", sx());
}

#[test]
fn test_find_datetime_in_line_by_block2() {
    _test_find_datetime_in_line_by_block(2);
}

#[test]
fn test_find_datetime_in_line_by_block4() {
    _test_find_datetime_in_line_by_block(4);
}

#[test]
fn test_find_datetime_in_line_by_block8() {
    _test_find_datetime_in_line_by_block(8);
}

#[test]
fn test_find_datetime_in_line_by_block256() {
    _test_find_datetime_in_line_by_block(256);
}

#[cfg(test)]
type _test_find_sysline_at_datetime_filter_Checks<'a> = Vec<(FileOffset, &'a str, &'a str)>;

/// underlying test code for `SyslineReader.find_datetime_in_line`
/// called by other functions `test_find_sysline_at_datetime_filterX`
#[cfg(test)]
fn __test_find_sysline_at_datetime_filter(
    file_content: String, dt_pattern: DateTimePattern, blocksz: BlockSz,
    checks: _test_find_sysline_at_datetime_filter_Checks,
) {
    debug_eprintln!("{}__test_find_sysline_at_datetime_filter(…, {:?}, {}, …)", sn(), dt_pattern, blocksz);

    let ntf1 = create_temp_file(&file_content.as_str());
    let path = String::from(ntf1.path().to_str().unwrap());
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = match SyslineReader::new(&path, blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({:?}, {}) failed {}", &path, blocksz, err);
        }
    };
    for (fo1, dts, sline_expect) in checks.iter() {
        //let dt = match Local.datetime_from_str(dts, dt_pattern.as_str()) {
        // TODO: add `has_tz` to `checks`, remove this
        let tzo = FixedOffset::west(3600 * 8);
        let has_tz = dt_pattern_has_tz(&dt_pattern.as_str());
        debug_eprintln!("{}str_datetime({:?}, {:?}, {:?}, {:?})", so(), str_to_String_noraw(dts), dt_pattern, has_tz, &tzo);
        let dt = match str_datetime(dts, &dt_pattern.as_str(), has_tz, &tzo) {
            Some(val) => val,
            None => {
                panic!("ERROR: datetime_from_str({:?}, {:?}) returned None", dts, dt_pattern);
            }
        };
        let sline_expect_noraw = str_to_String_noraw(sline_expect);
        debug_eprintln!("{}find_sysline_at_datetime_filter({}, {:?})", so(), fo1, dt);
        let result = slr.find_sysline_at_datetime_filter(*fo1, &Some(dt));
        match result {
            ResultS4_SyslineFind::Found(val) | ResultS4_SyslineFind::Found_EOF(val) => {
                let sline = val.1.to_String();
                let sline_noraw = str_to_String_noraw(sline.as_str());
                debug_eprintln!("\nexpected: {:?}", sline_expect_noraw);
                debug_eprintln!("returned: {:?}\n", sline_noraw);
                //print_colored(Color::Yellow, format!("expected: {}\n", sline_expect_noraw).as_bytes());
                //print_colored(Color::Yellow, format!("returned: {}\n", sline_noraw).as_bytes());
                assert_eq!(
                    sline,
                    String::from(*sline_expect),
                    "Expected {:?} == {:?} but it is not!",
                    sline_noraw,
                    sline_expect_noraw
                );
                //debug_eprintln!("{}Check PASSED {:?}", so(), sline_noraw);
                match print_colored_stdout(
                    Color::Green,
                    format!(
                        "Check PASSED SyslineReader().find_sysline_at_datetime_filter({} {:?}) == {:?}\n",
                        fo1, dts, sline_noraw
                    )
                    .as_bytes(),
                ) {
                    Ok(_) => {},
                    Err(_) => {},
                };
            }
            ResultS4_SyslineFind::Done => {
                panic!("During test unexpected result Done");
            }
            ResultS4_SyslineFind::Err(err) => {
                panic!("During test unexpected result Error {}", err);
            }
        }
    }

    debug_eprintln!("{}_test_find_sysline_at_datetime_filter(…)", sx());
}

// TODO: [2022/03/16] create test cases with varying sets of Checks passed-in, current setup is always
//       clean, sequential series of checks from file_offset 0.
// TODO: BUG: [2022/03/15] why are these checks done in random order? The tests pass but run
//       in a confusing manner. Run `cargo test` to see.
/// basic test of `SyslineReader.find_datetime_in_line`
#[cfg(test)]
fn _test_find_sysline_at_datetime_filter(
    blocksz: BlockSz, checks: Option<_test_find_sysline_at_datetime_filter_Checks>,
) {
    stack_offset_set(None);
    debug_eprintln!("{}_test_find_sysline_at_datetime_filter()", sn());
    let dt_fmt1: DateTimePattern = String::from("%Y-%m-%d %H:%M:%S");
    let file_content1 = String::from(
        "\
2020-01-01 00:00:00
2020-01-01 00:00:01a
2020-01-01 00:00:02ab
2020-01-01 00:00:03abc
2020-01-01 00:00:04abcd
2020-01-01 00:00:05abcde
2020-01-01 00:00:06abcdef
2020-01-01 00:00:07abcdefg
2020-01-01 00:00:08abcdefgh
2020-01-01 00:00:09abcdefghi
2020-01-01 00:00:10abcdefghij
2020-01-01 00:00:11abcdefghijk
2020-01-01 00:00:12abcdefghijkl
2020-01-01 00:00:13abcdefghijklm
2020-01-01 00:00:14abcdefghijklmn
2020-01-01 00:00:15abcdefghijklmno
2020-01-01 00:00:16abcdefghijklmnop
2020-01-01 00:00:17abcdefghijklmnopq
2020-01-01 00:00:18abcdefghijklmnopqr
2020-01-01 00:00:19abcdefghijklmnopqrs
2020-01-01 00:00:20abcdefghijklmnopqrst
2020-01-01 00:00:21abcdefghijklmnopqrstu
2020-01-01 00:00:22abcdefghijklmnopqrstuv
2020-01-01 00:00:23abcdefghijklmnopqrstuvw
2020-01-01 00:00:24abcdefghijklmnopqrstuvwx
2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy
2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz
",
    );
    let checks0: _test_find_sysline_at_datetime_filter_Checks = Vec::from([
        (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
        (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        (0, "2020-01-01 00:00:04", "2020-01-01 00:00:04abcd\n"),
        (0, "2020-01-01 00:00:05", "2020-01-01 00:00:05abcde\n"),
        (0, "2020-01-01 00:00:06", "2020-01-01 00:00:06abcdef\n"),
        (0, "2020-01-01 00:00:07", "2020-01-01 00:00:07abcdefg\n"),
        (0, "2020-01-01 00:00:08", "2020-01-01 00:00:08abcdefgh\n"),
        (0, "2020-01-01 00:00:09", "2020-01-01 00:00:09abcdefghi\n"),
        (0, "2020-01-01 00:00:10", "2020-01-01 00:00:10abcdefghij\n"),
        (0, "2020-01-01 00:00:11", "2020-01-01 00:00:11abcdefghijk\n"),
        (0, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
        (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
        (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        (0, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
        (0, "2020-01-01 00:00:16", "2020-01-01 00:00:16abcdefghijklmnop\n"),
        (0, "2020-01-01 00:00:17", "2020-01-01 00:00:17abcdefghijklmnopq\n"),
        (0, "2020-01-01 00:00:18", "2020-01-01 00:00:18abcdefghijklmnopqr\n"),
        (0, "2020-01-01 00:00:19", "2020-01-01 00:00:19abcdefghijklmnopqrs\n"),
        (0, "2020-01-01 00:00:20", "2020-01-01 00:00:20abcdefghijklmnopqrst\n"),
        (0, "2020-01-01 00:00:21", "2020-01-01 00:00:21abcdefghijklmnopqrstu\n"),
        (0, "2020-01-01 00:00:22", "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n"),
        (0, "2020-01-01 00:00:23", "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n"),
        (0, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
        (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
    ]);

    let checksx: _test_find_sysline_at_datetime_filter_Checks = Vec::from([
        (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        (19, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        (40, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
        (62, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        (85, "2020-01-01 00:00:04", "2020-01-01 00:00:04abcd\n"),
        (109, "2020-01-01 00:00:05", "2020-01-01 00:00:05abcde\n"),
        (134, "2020-01-01 00:00:06", "2020-01-01 00:00:06abcdef\n"),
        (162, "2020-01-01 00:00:07", "2020-01-01 00:00:07abcdefg\n"),
        (187, "2020-01-01 00:00:08", "2020-01-01 00:00:08abcdefgh\n"),
        (215, "2020-01-01 00:00:09", "2020-01-01 00:00:09abcdefghi\n"),
        (244, "2020-01-01 00:00:10", "2020-01-01 00:00:10abcdefghij\n"),
        (274, "2020-01-01 00:00:11", "2020-01-01 00:00:11abcdefghijk\n"),
        (305, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
        (337, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
        (370, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        (404, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
        (439, "2020-01-01 00:00:16", "2020-01-01 00:00:16abcdefghijklmnop\n"),
        (475, "2020-01-01 00:00:17", "2020-01-01 00:00:17abcdefghijklmnopq\n"),
        (512, "2020-01-01 00:00:18", "2020-01-01 00:00:18abcdefghijklmnopqr\n"),
        (550, "2020-01-01 00:00:19", "2020-01-01 00:00:19abcdefghijklmnopqrs\n"),
        (589, "2020-01-01 00:00:20", "2020-01-01 00:00:20abcdefghijklmnopqrst\n"),
        (629, "2020-01-01 00:00:21", "2020-01-01 00:00:21abcdefghijklmnopqrstu\n"),
        (670, "2020-01-01 00:00:22", "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n"),
        (712, "2020-01-01 00:00:23", "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n"),
        (755, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
        (799, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        (844, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
    ]);

    let mut checks_ = checks0;
    if checks.is_some() {
        checks_ = checks.unwrap();
    }
    __test_find_sysline_at_datetime_filter(file_content1, dt_fmt1, blocksz, checks_);
    debug_eprintln!("{}_test_find_sysline_at_datetime_filter()", sx());
}

// XXX: are these different BlockSz tests necessary? are not these adequately tested by
//      other lower-level tests?

#[test]
fn test_find_sysline_at_datetime_filter_4() {
    _test_find_sysline_at_datetime_filter(4, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_8() {
    _test_find_sysline_at_datetime_filter(8, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_16() {
    _test_find_sysline_at_datetime_filter(16, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_32() {
    _test_find_sysline_at_datetime_filter(32, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_64() {
    _test_find_sysline_at_datetime_filter(64, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_128() {
    _test_find_sysline_at_datetime_filter(128, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_256() {
    _test_find_sysline_at_datetime_filter(256, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_512() {
    _test_find_sysline_at_datetime_filter(512, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_1024() {
    _test_find_sysline_at_datetime_filter(1024, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_2056() {
    _test_find_sysline_at_datetime_filter(2056, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_() {
    _test_find_sysline_at_datetime_filter(64,Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:00",
            "2020-01-01 00:00:00\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_a() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:01",
            "2020-01-01 00:00:01a\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_b() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:02",
            "2020-01-01 00:00:02ab\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_c() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:03",
            "2020-01-01 00:00:03abc\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_d() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:04",
            "2020-01-01 00:00:04abcd\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_e() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:05",
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_f() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:06",
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_g() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:07",
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_h() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:08",
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_i() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:09",
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_j() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:10",
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_k() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:11",
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_l() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:12",
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_m() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:13",
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_n() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:14",
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_o() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:15",
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_p() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:16",
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_q() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:17",
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_r() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:18",
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_s() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:19",
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_t() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:20",
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_u() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:21",
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_v() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:22",
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_w() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:23",
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_x() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:24",
            "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_y() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:25",
            "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_z() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:26",
            "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_a() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            19,
            "2020-01-01 00:00:01",
            "2020-01-01 00:00:01a\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_b() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            40,
            "2020-01-01 00:00:02",
            "2020-01-01 00:00:02ab\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_c() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            62,
            "2020-01-01 00:00:03",
            "2020-01-01 00:00:03abc\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_d() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            85,
            "2020-01-01 00:00:04",
            "2020-01-01 00:00:04abcd\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_e() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            109,
            "2020-01-01 00:00:05",
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_f() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            134,
            "2020-01-01 00:00:06",
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_g() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            160,
            "2020-01-01 00:00:07",
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_h() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            187,
            "2020-01-01 00:00:08",
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_i() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            215,
            "2020-01-01 00:00:09",
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_j() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            244,
            "2020-01-01 00:00:10",
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_k() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            274,
            "2020-01-01 00:00:11",
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_l() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            305,
            "2020-01-01 00:00:12",
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_m() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            337,
            "2020-01-01 00:00:13",
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_n() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            370,
            "2020-01-01 00:00:14",
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_o() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            404,
            "2020-01-01 00:00:15",
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_p() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            439,
            "2020-01-01 00:00:16",
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_q() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            475,
            "2020-01-01 00:00:17",
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_r() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            512,
            "2020-01-01 00:00:18",
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_s() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            550,
            "2020-01-01 00:00:19",
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_t() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            589,
            "2020-01-01 00:00:20",
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_u() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            629,
            "2020-01-01 00:00:21",
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_v() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            670,
            "2020-01-01 00:00:22",
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_w() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            712,
            "2020-01-01 00:00:23",
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_x() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            755,
            "2020-01-01 00:00:24",
            "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_y() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            799,
            "2020-01-01 00:00:25",
            "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_z() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            844,
            "2020-01-01 00:00:26",
            "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_z_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_y_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_x_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_m_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_za() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_ya() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_xa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_ma() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3____() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__ab() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__az() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__bd() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:04", "2020-01-01 00:00:04abcd\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__ml() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__my() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__mz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__m_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aaa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abc() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aba() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abn() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aby() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aaz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_byo() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zaa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbc() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zba() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbn() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zby() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zaz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybc() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yba() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybn() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yby() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

// TODO: [2022/03/18] create one wrapper test test_find_sysline_at_datetime_checks_ that takes some
//        vec of test-input-output, and does all possible permutations.

/// basic test of `SyslineReader.sysline_pass_filters`
/// TODO: add tests with TZ
#[allow(non_snake_case)]
#[test]
fn test_sysline_pass_filters() {
    debug_eprintln!("{}test_sysline_pass_filters()", sn());

    fn DTL(s: &str) -> DateTimeL {
        //return DateTimeL.datetime_from_str(s, &"%Y%m%dT%H%M%S").unwrap();
        let tzo = FixedOffset::west(3600 * 8);
        str_datetime(s, &"%Y%m%dT%H%M%S", false, &tzo).unwrap()
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
        (Some(DTL(&"20000101T010101")), DTL(&"20000101T010106"), None, Result_Filter_DateTime2::OccursInRange),
        (
            Some(DTL(&"20000101T010102")),
            DTL(&"20000101T010101"),
            None,
            Result_Filter_DateTime2::OccursBeforeRange,
        ),
        (Some(DTL(&"20000101T010101")), DTL(&"20000101T010101"), None, Result_Filter_DateTime2::OccursInRange),
        (None, DTL(&"20000101T010101"), Some(DTL(&"20000101T010106")), Result_Filter_DateTime2::OccursInRange),
        (
            None,
            DTL(&"20000101T010101"),
            Some(DTL(&"20000101T010100")),
            Result_Filter_DateTime2::OccursAfterRange,
        ),
        (None, DTL(&"20000101T010101"), Some(DTL(&"20000101T010101")), Result_Filter_DateTime2::OccursInRange),
    ] {
        let result = SyslineReader::dt_pass_filters(&dt, &da, &db);
        assert_eq!(exp_result, result, "Expected {:?} Got {:?} for ({:?}, {:?}, {:?})", exp_result, result, dt, da, db);
        #[allow(unused_must_use)]
        match print_colored_stdout(
            Color::Green,
            format!("{}({:?}, {:?}, {:?}) returned expected {:?}\n", so(), dt, da, db, result).as_bytes(),
        ) {
            Ok(_) => {},
            Err(_) => {},
        }
    }
    debug_eprintln!("{}test_sysline_pass_filters()", sx());
}

/// basic test of `SyslineReader.dt_after_or_before`
/// TODO: add tests with TZ
#[allow(non_snake_case)]
#[test]
fn test_dt_after_or_before() {
    debug_eprintln!("{}test_dt_after_or_before()", sn());

    fn DTL(s: &str) -> DateTimeL {
        let tzo = FixedOffset::west(3600 * 8);
        str_datetime(s, &"%Y%m%dT%H%M%S", false, &tzo).unwrap()
    }

    for (dt, da, exp_result) in [
        (DTL(&"20000101T010106"), None, Result_Filter_DateTime1::Pass),
        (DTL(&"20000101T010101"), Some(DTL(&"20000101T010103")), Result_Filter_DateTime1::OccursBefore),
        (DTL(&"20000101T010100"), Some(DTL(&"20000101T010100")), Result_Filter_DateTime1::OccursAtOrAfter),
        (DTL(&"20000101T010109"), Some(DTL(&"20000101T010108")), Result_Filter_DateTime1::OccursAtOrAfter),
    ] {
        let result = SyslineReader::dt_after_or_before(&dt, &da);
        assert_eq!(exp_result, result, "Expected {:?} Got {:?} for ({:?}, {:?})", exp_result, result, dt, da);
        #[allow(unused_must_use)]
        match print_colored_stdout(
            Color::Green,
            format!("{}({:?}, {:?}) returned expected {:?}\n", so(), dt, da, result).as_bytes(),
        ) {
            Ok(_) => {},
            Err(_) => {},
        }
    }
    debug_eprintln!("{}test_dt_after_or_before()", sx());
}

/// testing helper
/// if debug then print with color
/// else print efficiently
/// XXX: does not handle multi-byte
/// BUG: if `(*slp).dt_beg` or `(*slp).dt_end` are within multi-byte encoded character
///      then this will panic. e.g. Sysline with underlying "2000-01-01 00:00:00\n".to_String_noraw()
///      will return "2000-01-01 00:00:00␊". Which will panic:
///          panicked at 'byte index 20 is not a char boundary; it is inside '␊' (bytes 19..22) of `2000-01-01 00:00:00␊`'
///      However, this function is only an intermediary development helper. Can this problem have a
///      brute-force workaround. 
#[cfg(any(debug_assertions,test))]
fn print_slp(slp: &SyslineP) {
    if cfg!(debug_assertions) {
        let out = (*slp).to_String_noraw();
        // XXX: presumes single-byte character encoding, does not handle multi-byte encoding
        /*
        debug_eprintln!("{}print_slp: to_String_noraw() {:?} dt_beg {} dt_end {} len {}", so(), out, split_ab, (*slp).dt_end, (*slp).len());
        debug_eprintln!("{}print_slp: out.chars():", so());
        for (c_n, c_) in out.chars().enumerate() {
            debug_eprintln!("{}print_slp:              char {} {:?}", so(), c_n, c_);
        }
        debug_eprintln!("{}print_slp: out.bytes():", so());
        for (b_n, b_) in out.bytes().enumerate() {
            debug_eprintln!("{}print_slp:              byte {} {:?}", so(), b_n, b_);
        }
        */
        let a = &out[..(*slp).dt_beg];
        match print_colored_stdout(Color::Green, &a.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: print_colored a returned error {}", err);
            }
        };
        let b = &out[(*slp).dt_beg..(*slp).dt_end];
        match print_colored_stdout(Color::Yellow, &b.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: print_colored b returned error {}", err);
            }
        };
        let c = &out[(*slp).dt_end..];
        match print_colored_stdout(Color::Green, &c.as_bytes()) {
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
            write_stdout(slice);
        }
    }
}

#[cfg(test)]
type _test_SyslineReader_check<'a> = (&'a str, FileOffset);

#[cfg(test)]
type _test_SyslineReader_checks<'a> = Vec<(&'a str, FileOffset)>;

/// basic test of SyslineReader things
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader(path: &Path, blocksz: BlockSz, fileoffset: FileOffset, checks: &_test_SyslineReader_checks) {
    debug_eprintln!("{}test_SyslineReader({:?}, {})", sn(), &path, blocksz);
    let fpath: FPath = path.to_str().unwrap_or("").to_string();
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = match SyslineReader::new(&fpath, blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({:?}, {}) failed {}", fpath, blocksz, err);
            return;
        }
    };
    debug_eprintln!("{}test_SyslineReader: {:?}", so(), slr);

    let mut fo1: FileOffset = fileoffset;
    let mut check_i: usize = 0;
    loop {
        let result = slr.find_sysline(fo1);
        let done = result.is_done() || result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) => {
                debug_eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Found({}, @{:p})", so(), fo1, fo, &*slp);
                debug_eprintln!(
                    "{}test_SyslineReader: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print_slp(&slp);
                assert!(!slr.is_sysline_last(&slp), "returned Found yet this Sysline is last! Should have returned Found_EOF or is this Sysline not last?");
                fo1 = fo;

                debug_eprintln!("{}test_SyslineReader: check {}", so(), check_i);
                // check slp.String
                let check_String = checks[check_i].0.to_string();
                let actual_String = (*slp).to_String();
                assert_eq!(check_String, actual_String,"\nexpected string value {:?}\nfind_sysline returned {:?}", check_String, actual_String);
                // check fileoffset
                let check_fo = checks[check_i].1;
                assert_eq!(check_fo, fo, "expected fileoffset {}, but find_sysline returned fileoffset {} for check {}", check_fo, fo, check_i);
            }
            ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                debug_eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Found_EOF({}, @{:p})", so(), fo1, fo, &*slp);
                debug_eprintln!(
                    "{}test_SyslineReader: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print_slp(&slp);
                assert!(slr.is_sysline_last(&slp), "returned Found_EOF yet this Sysline is not last!");
                fo1 = fo;

                debug_eprintln!("{}test_SyslineReader: check {}", so(), check_i);
                // check slp.String
                let check_String = checks[check_i].0.to_string();
                let actual_String = (*slp).to_String();
                assert_eq!(check_String, actual_String,"\nexpected string value {:?}\nfind_sysline returned {:?}", check_String, actual_String);
                // check fileoffset
                let check_fo = checks[check_i].1;
                assert_eq!(check_fo, fo, "expected fileoffset {}, but find_sysline returned fileoffset {} for check {}", check_fo, fo, check_i);
            }
            ResultS4_SyslineFind::Done => {
                debug_eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Done", so(), fo1);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Err({})", so(), fo1, err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
        check_i += 1;
        if done {
            break;
        }
    }
    assert_eq!(checks.len(), check_i, "expected {} Sysline checks but only {} Sysline checks were done", checks.len(), check_i);

    debug_eprintln!("{}test_SyslineReader: Found {} Lines, {} Syslines", so(), slr.linereader.count(), slr.syslines.len());
    debug_eprintln!("{}test_SyslineReader({:?}, {})", sx(), &path, blocksz);
}

#[allow(non_upper_case_globals)]
#[cfg(test)]
static test_data_file_A_dt6: &str = &"\
2000-01-01 00:00:00
2000-01-01 00:00:01a
2000-01-01 00:00:02ab
2000-01-01 00:00:03abc
2000-01-01 00:00:04abcd
2000-01-01 00:00:05abcde";

#[allow(non_upper_case_globals)]
#[cfg(test)]
static test_data_file_A_dt6_checks: [_test_SyslineReader_check; 6] = [
    ("2000-01-01 00:00:00\n", 20),
    ("2000-01-01 00:00:01a\n", 41),
    ("2000-01-01 00:00:02ab\n", 63),
    ("2000-01-01 00:00:03abc\n", 86),
    ("2000-01-01 00:00:04abcd\n", 110),
    ("2000-01-01 00:00:05abcde", 134),
];

#[cfg(test)]
lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref test_SyslineReader_A_ntf: NamedTempFile = {
        create_temp_file(test_data_file_A_dt6)
    };
}

#[test]
fn test_SyslineReader_A_dt6_128_0_()
{
    let checks = _test_SyslineReader_checks::from(test_data_file_A_dt6_checks);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 0, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_1_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[1..]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 1, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_2_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[2..]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 40, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_3_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[3..]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 62, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_4_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[4..]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 85, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_5_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[5..]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 86, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_X_beforeend()
{
    let checks = _test_SyslineReader_checks::from([]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 132, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_X_pastend()
{
    let checks = _test_SyslineReader_checks::from([]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 135, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_X9999()
{
    let checks = _test_SyslineReader_checks::from([]);
    test_SyslineReader(test_SyslineReader_A_ntf.path(), 128, 9999, &checks);
}

// LAST WORKING HERE 2022/03/19 21:11:23 getting these tests test_SyslineReader_A_dt6* to work.
// After that, add *at least* one more data set.
//  see test_data_file_dt5
// then extraploate more tests for test_SyslineReader_w_filtering*

#[allow(non_upper_case_globals)]
#[cfg(test)]
static test_data_file_B_dt0: &str = &"
foo
bar
";

#[allow(non_upper_case_globals)]
#[cfg(test)]
static test_data_file_B_dt0_checks: [_test_SyslineReader_check; 0] = [];

#[test]
fn test_SyslineReader_B_dt0_0()
{
    let ntf = create_temp_file(test_data_file_B_dt0);
    let checks = _test_SyslineReader_checks::from(test_data_file_B_dt0_checks);
    test_SyslineReader(ntf.path(), 128, 0, &checks);
}

#[test]
fn test_SyslineReader_B_dt0_3()
{
    let ntf = create_temp_file(test_data_file_B_dt0);
    let checks = _test_SyslineReader_checks::from(test_data_file_B_dt0_checks);
    test_SyslineReader(ntf.path(), 128, 3, &checks);
}

#[allow(non_upper_case_globals)]
#[cfg(test)]
static _test_data_file_C_dt6: &str = &"\
[ERROR] 2000-01-01 00:00:00
[ERROR] 2000-01-01 00:00:01a
[ERROR] 2000-01-01 00:00:02ab
[ERROR] 2000-01-01 00:00:03abc
[ERROR] 2000-01-01 00:00:04abcd
[ERROR] 2000-01-01 00:00:05abcde";

#[allow(non_upper_case_globals)]
#[cfg(test)]
static _test_data_file_C_dt6_checks: [_test_SyslineReader_check; 6] = [
    ("[ERROR] 2000-01-01 00:00:00\n", 28),
    ("[ERROR] 2000-01-01 00:00:01a\n", 57),
    ("[ERROR] 2000-01-01 00:00:02ab\n", 87),
    ("[ERROR] 2000-01-01 00:00:03abc\n", 118),
    ("[ERROR] 2000-01-01 00:00:04abcd\n", 150),
    ("[ERROR] 2000-01-01 00:00:05abcde", 182),
];

#[cfg(test)]
lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref test_SyslineReader_C_ntf: NamedTempFile = {
        create_temp_file(_test_data_file_C_dt6)
    };
}

#[test]
fn test_SyslineReader_C_dt6_0()
{
    let checks = _test_SyslineReader_checks::from(_test_data_file_C_dt6_checks);
    test_SyslineReader(test_SyslineReader_C_ntf.path(), 128, 0, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_1()
{
    let checks = _test_SyslineReader_checks::from(&_test_data_file_C_dt6_checks[1..]);
    test_SyslineReader(test_SyslineReader_C_ntf.path(), 128, 3, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_2a()
{
    let checks = _test_SyslineReader_checks::from(&_test_data_file_C_dt6_checks[1..]);
    test_SyslineReader(test_SyslineReader_C_ntf.path(), 128, 27, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_2b()
{
    let checks = _test_SyslineReader_checks::from(&_test_data_file_C_dt6_checks[2..]);
    test_SyslineReader(test_SyslineReader_C_ntf.path(), 128, 28, &checks);
}

//#[allow(non_upper_case_globals)]
//#[cfg(test)]
//static _test_data_file_D_invalid: [_test_SyslineReader_check; 0] = [];

#[test]
fn test_SyslineReader_D_invalid1()
{
    let data_invalid1: [u8; 1] = [ 0xFF ];
    let date_checks1: _test_SyslineReader_checks = _test_SyslineReader_checks::from([]);
    let ntf1: NamedTempFile = create_temp_file_bytes(&data_invalid1);
    test_SyslineReader(ntf1.path(), 128, 0, &date_checks1);
}

/// basic test of SyslineReader things
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_w_filtering_1(
    path: &FPath, blocksz: BlockSz, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
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
        let s1 = file_to_String_noraw(path);
        #[allow(unused_must_use)]
        match print_colored_stdout(Color::Yellow, s1.as_bytes()) { _ => {}, };
        println!();
    }

    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = match SyslineReader::new(path, blocksz, tzo) {
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
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                debug_eprintln!(
                    "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Found({}, @{:p})",
                    so(),
                    fo1,
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                    fo,
                    &*slp
                );
                debug_eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                print!("FileOffset {:3} {:?} '", fo1, filter_dt_after_opt);
                let snippet = slr
                    .linereader
                    .blockreader
                    ._vec_from(fo1, std::cmp::min(fo1 + 40, filesz));
                match print_colored_stdout(Color::Yellow, buffer_to_String_noraw(snippet.as_slice()).as_bytes())
                     { _ => {}, };
                print!("' ");
                //print_slp(&slp);
                let slices = (*slp).get_slices();
                for slice in slices.iter() {
                    match print_colored_stdout(Color::Green, slice) { _ => {}, };
                }
                println!();
            }
            ResultS4_SyslineFind::Done => {
                debug_eprintln!(
                    "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Done",
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

    debug_eprintln!("{}Found {} Lines, {} Syslines", so(), slr.linereader.count(), slr.syslines.len());
    debug_eprintln!(
        "{}test_SyslineReader_w_filtering_1({:?}, {}, {:?}, {:?})",
        sx(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
}

// TODO: add test cases for `test_SyslineReader_w_filtering_1`

/// print the filtered syslines for a SyslineReader
/// quick debug helper
#[cfg(any(debug_assertions,test))]
fn process_SyslineReader(
    slr: &mut SyslineReader, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    debug_eprintln!("{}process_SyslineReader({:?}, {:?}, {:?})", sn(), slr, filter_dt_after_opt, filter_dt_before_opt,);
    let mut fo1: FileOffset = 0;
    let mut search_more = true;
    debug_eprintln!("{}slr.find_sysline_at_datetime_filter({}, {:?})", so(), fo1, filter_dt_after_opt);
    let result = slr.find_sysline_at_datetime_filter(fo1, &filter_dt_after_opt);
    match result {
        ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
            debug_eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Found|Found_EOF({}, @{:p})",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt,
                fo,
                &*slp
            );
            debug_eprintln!(
                "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                so(),
                fo,
                &(*slp),
                slp.lines.len(),
                (*slp).len(),
                (*slp).to_String_noraw(),
            );
            fo1 = fo;
            print_slp(&slp);
        }
        ResultS4_SyslineFind::Done => {
            debug_eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Done",
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
        debug_eprintln!("{}process_SyslineReader(…)", sx());
        return;
    }
    let mut fo2: FileOffset = fo1;
    loop {
        let result = slr.find_sysline(fo2);
        let eof = result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                if eof {
                    debug_eprintln!("{}slr.find_sysline({}) returned Found_EOF({}, @{:p})", so(), fo2, fo, &*slp);
                } else {
                    debug_eprintln!("{}slr.find_sysline({}) returned Found({}, @{:p})", so(), fo2, fo, &*slp);
                }
                fo2 = fo;
                debug_eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
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
                match SyslineReader::sysline_pass_filters(&slp, filter_dt_after_opt, filter_dt_before_opt) {
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
                            assert!(slr.is_sysline_last(&slp), "returned Found_EOF yet this Sysline is not last!?");
                        } else {
                            assert!(!slr.is_sysline_last(&slp), "returned Found yet this Sysline is last!? Should have returned Found_EOF or this Sysline is really not last.");
                        }
                    }
                }
            }
            ResultS4_SyslineFind::Done => {
                debug_eprintln!("{}slr.find_sysline({}) returned Done", so(), fo2);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!("{}slr.find_sysline({}) returned Err({})", so(), fo2, err);
                eprintln!("ERROR: {}", err);
                break;
            }
        }
    }
    debug_eprintln!("{}process_SyslineReader({:?}, …)", sx(), slr.path());
}

/// quick debug helper
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_SyslineReader_process_file<'a>(
    path: &'a FPath, blocksz: BlockSz, filter_dt_after_opt: &'a DateTimeL_Opt, filter_dt_before_opt: &'a DateTimeL_Opt,
) -> Option<Box<SyslineReader<'a>>> {
    debug_eprintln!(
        "{}process_file({:?}, {}, {:?}, {:?})",
        sn(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
    let tzo8 = FixedOffset::west(3600 * 8);
    let slr = match SyslineReader::new(path, blocksz, tzo8) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({}, {}) failed {}", path, blocksz, err);
            return None;
        }
    };
    debug_eprintln!("{}{:?}", so(), slr);
    debug_eprintln!("{}process_file(…)", sx());
    return Some(Box::new(slr));
}

/// basic test of SyslineReader things
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_w_filtering_2(
    path: &FPath, blocksz: BlockSz, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    debug_eprintln!(
        "{}test_SyslineReader_w_filtering_2({:?}, {}, {:?}, {:?})",
        sn(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
    let slr_opt = _test_SyslineReader_process_file(path, blocksz, filter_dt_after_opt, filter_dt_before_opt);
    if slr_opt.is_some() {
        let slr = &slr_opt.unwrap();
        debug_eprintln!("{}Found {} Lines, {} Syslines", so(), slr.linereader.count(), slr.syslines.len());
    }
    debug_eprintln!("{}test_SyslineReader_w_filtering_2(…)", sx());
}

// TODO: add test cases for test_SyslineReader_w_filtering_2

/// basic test of SyslineReader things
/// process multiple files
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_w_filtering_3(
    paths: &Vec<String>, blocksz: BlockSz, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    debug_eprintln!(
        "{}test_SyslineReader_w_filtering_3({:?}, {}, {:?}, {:?})",
        sn(),
        &paths,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );

    let mut slrs = Vec::<SyslineReader>::with_capacity(paths.len());
    for path in paths.iter() {
        let tzo8 = FixedOffset::west(3600 * 8);
        debug_eprintln!("{}SyslineReader::new({:?}, {}, {:?})", so(), path, blocksz, tzo8);
        let slr = match SyslineReader::new(path, blocksz, tzo8) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("ERROR: SyslineReader::new({:?}, {}) failed {}", path, blocksz, err);
                return;
            }
        };
        debug_eprintln!("{}{:?}", so(), slr);
        slrs.push(slr)
    }
    for slr in slrs.iter_mut() {
        process_SyslineReader(slr, filter_dt_after_opt, filter_dt_before_opt);
        println!();
    }
    debug_eprintln!("{}test_SyslineReader_w_filtering_3(…)", sx());
}

// TODO: add test cases for `test_SyslineReader_w_filtering_3`

/// basic test of SyslineReader things
/// read all file offsets but randomly
/// TODO: this test was hastily designed for human review. Redesign it for automatic review.
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_SyslineReader_rand(path_: &FPath, blocksz: BlockSz) {
    debug_eprintln!("{}test_SyslineReader_rand({:?}, {})", sn(), &path_, blocksz);
    let tzo8 = FixedOffset::west(3600 * 8);
    let mut slr1 = match SyslineReader::new(path_, blocksz, tzo8) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: SyslineReader::new({}, {}, ...) failed {}", path_, blocksz, err);
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
                assert!(false, "slr1.find_sysline({}) returned Err({})", fo1, err);
            }
            _ => {}
        }
    }
    // should print the file as-is and not be affected by random reads
    slr1.print_all(true);
    debug_eprintln!("\n{}{:?}", so(), slr1);
    debug_eprintln!("{}test_SyslineReader_rand(…)", sx());
}

#[test]
fn test_SyslineReader_rand__zero__2() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/zero.log"), 2);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1__2() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1.log"), 2);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1.log"), 4);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1__8() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1.log"), 8);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1_Win__2() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1_Win.log"), 2);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1_Win__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1_Win.log"), 4);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1_Win__8() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1_Win.log"), 8);
}

#[test]
fn test_SyslineReader_rand__test0_nlx2__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx2.log"), 4);
}

#[test]
fn test_SyslineReader_rand__basic_dt1__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/basic-dt1.log"), 4);
}

#[test]
fn test_SyslineReader_rand__dtf5_6c__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/dtf5-6c.log"), 4);
}

#[test]
fn test_SyslineReader_rand__dtf5_6c__8() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/dtf5-6c.log"), 8);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslogWriter
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// XXX: unfinished attempt at `Printer` or `Writer` "class"

type SyslineReaders<'syslogwriter> = Vec<SyslineReader<'syslogwriter>>;

/// Specialized Writer that coordinates writing multiple SyslineReaders
pub struct SyslogWriter<'syslogwriter> {
    syslinereaders: SyslineReaders<'syslogwriter>,
}

impl<'syslogwriter> SyslogWriter<'syslogwriter> {
    pub fn new(syslinereaders: SyslineReaders<'syslogwriter>) -> SyslogWriter<'syslogwriter> {
        assert_gt!(syslinereaders.len(), 0, "Passed zero SyslineReaders");
        SyslogWriter { syslinereaders }
    }

    pub fn push(&mut self, syslinereader: SyslineReader<'syslogwriter>) {
        self.syslinereaders.push(syslinereader);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// multi-threaded
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

use std::thread;

// would using the premade ThreadPool be easier?
// https://docs.rs/threadpool/1.8.1/threadpool/
// or Rayon threading
// https://crates.io/crates/rayon
//
// these might be good to read:
// https://pkolaczk.github.io/multiple-threadpools-rust/
//
// https://doc.rust-lang.org/book/ch16-03-shared-state.html#atomic-reference-counting-with-arct
// https://doc.rust-lang.org/book/ch20-02-multithreaded.html

// -------------------------------------------------------------------------------------------------
// threading try #1
// using rust built-in Thread module
// -------------------------------------------------------------------------------------------------

// REMOVED ... see archived code

// -------------------------------------------------------------------------------------------------
// threading try #2
// using crate rayon
// -------------------------------------------------------------------------------------------------

// REMOVED ... see archived code

// -------------------------------------------------------------------------------------------------

#[test]
fn test_str_datetime() {
    let hour = 3600;
    let tzo8 = FixedOffset::west(3600 * 8);
    let tzo5 = FixedOffset::east(3600 * 5);

    // good without timezone
    let dts1 = "2000-01-01 00:00:01";
    let p1 = "%Y-%m-%d %H:%M:%S";
    let dt1 = str_datetime(&dts1, &p1, false, &tzo8).unwrap();
    let answer1 = Local.ymd(2000, 01, 01).and_hms(0, 0, 1);
    assert_eq!(dt1, answer1);

    // good without timezone
    let dts1 = "2000-01-01 00:00:01";
    let p1 = "%Y-%m-%d %H:%M:%S";
    let dt1 = str_datetime(&dts1, &p1, false, &tzo5).unwrap();
    let answer1 = tzo5.ymd(2000, 01, 01).and_hms(0, 0, 1);
    assert_eq!(dt1, answer1);

    // good with timezone
    let dts2 = "2000-01-01 00:00:02 -0100";
    let p2 = "%Y-%m-%d %H:%M:%S %z";
    let dt2 = str_datetime(&dts2, &p2, true, &tzo8).unwrap();
    let answer2 = FixedOffset::west(1 * hour).ymd(2000, 01, 01).and_hms(0, 0, 2);
    assert_eq!(dt2, answer2);

    // bad with timezone
    let dts3 = "2000-01-01 00:00:03 BADD";
    let p3 = "%Y-%m-%d %H:%M:%S %z";
    let dt3 = str_datetime(&dts3, &p3, true, &tzo8);
    assert_eq!(dt3, None);

    // bad without timezone
    let dts4 = "2000-01-01 00:00:XX";
    let p4 = "%Y-%m-%d %H:%M:%S";
    let dt4 = str_datetime(&dts4, &p4, false, &tzo8);
    assert_eq!(dt4, None);
}

/// given the vector of `DateTimeL`, return the vector index and value of the soonest
/// (minimum) value within a `Some`
/// If the vector is empty then return `None`
#[cfg(test)]
fn datetime_soonest2(vec_dt: &Vec<DateTimeL>) -> Option<(usize, DateTimeL)> {
    if vec_dt.is_empty() {
        return None;
    }

    let mut index: usize = 0;
    for (index_, _) in vec_dt.iter().enumerate() {
        if vec_dt[index_] < vec_dt[index] {
            index = index_;
        }
    }

    Some((index, vec_dt[index].clone()))
}

/// test function `datetime_soonest2`
#[test]
fn test_datetime_soonest2() {
    debug_eprintln!("{}test_datetime_soonest2()", sn());
    let vec0 = Vec::<DateTimeL>::with_capacity(0);
    let val = datetime_soonest2(&vec0);
    assert!(val.is_none());
    let tzo = FixedOffset::west(3600 * 8);

    let dt1_a = str_datetime(&"2001-01-01T12:00:00", &"%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
    let vec1: Vec<DateTimeL> = vec![dt1_a.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec1) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None1a");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_a);

    let dt1_b = str_datetime(&"2001-01-01T12:00:00-0100", &"%Y-%m-%dT%H:%M:%S%z", true, &tzo).unwrap();
    let vec1: Vec<DateTimeL> = vec![dt1_b.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec1) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None1b");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_b);

    let dt2_a = str_datetime(&"2002-01-01T11:00:00", &"%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
    let vec2a: Vec<DateTimeL> = vec![dt1_a.clone(), dt2_a.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec2a) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None2a");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_a);

    let vec2b: Vec<DateTimeL> = vec![dt2_a.clone(), dt1_a.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec2b) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None2b");
        }
    };
    assert_eq!(i_, 1);
    assert_eq!(dt_, dt1_a);

    let dt3 = str_datetime(&"2000-01-01T12:00:00", &"%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
    let vec3a: Vec<DateTimeL> = vec![dt1_a.clone(), dt2_a.clone(), dt3.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec3a) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None3a");
        }
    };
    assert_eq!(i_, 2);
    assert_eq!(dt_, dt3);

    let vec3b: Vec<DateTimeL> = vec![dt1_a.clone(), dt3.clone(), dt2_a.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec3b) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None3b");
        }
    };
    assert_eq!(i_, 1);
    assert_eq!(dt_, dt3);

    let vec3c: Vec<DateTimeL> = vec![dt3.clone(), dt1_a.clone(), dt2_a.clone()];
    let (i_, dt_) = match datetime_soonest2(&vec3c) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None3c");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt3);

    debug_eprintln!("{}test_datetime_soonest2()", sx());
}

// -------------------------------------------------------------------------------------------------
// threading try #3
// -------------------------------------------------------------------------------------------------

// REMOVED ... see archived code

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
/// this creates `SyslineReader` and process the `Syslines`
fn exec_4(chan_send_dt: Chan_Send_Datum, thread_init_data: Thread_Init_Data4) -> thread::ThreadId {
    stack_offset_set(Some(2));
    let (path, blocksz, filter_dt_after_opt, filter_dt_before_opt, tz_offset) = thread_init_data;
    debug_eprintln!("{}exec_4({:?})", sn(), path);
    let thread_cur = thread::current();
    let tid = thread_cur.id();
    let tname = thread_cur.name().unwrap_or(&"").clone();
    //let ti = tid.as_u64() as u64;

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
                    Result_Filter_DateTime2::OccursInRange => {
                        let is_last = slr.is_sysline_last(&slp);
                        assert_eq!(eof, is_last, "from find_sysline, ResultS4_SyslineFind.is_eof is {:?} (EOF), yet the returned SyslineP.is_sysline_last is {:?}; they should always agree", eof, is_last);
                        debug_eprintln!("{}{:?}({}): OccursInRange, chan_send_dt.send(({:p}, None, {}));", so(), tid, tname, slp, is_last);
                        match chan_send_dt.send((Some(slp), None, is_last)) {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("ERROR: D chan_send_dt.send(…) failed {}", err);
                            }
                        }
                    }
                    Result_Filter_DateTime2::OccursBeforeRange => {
                        debug_eprintln!("{}{:?}{} ERROR: Sysline out of order: {:?}", so(), tid, tname, (*slp).to_String_noraw());
                        eprintln!("ERROR: Encountered a Sysline that is out of order; will abandon processing of file {:?}", path);
                        break;
                    }
                    Result_Filter_DateTime2::OccursAfterRange => {
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
            match print_colored_stderr(clrerr, &self.bytes.to_string().as_bytes()) {
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
            match print_colored_stderr(clrerr, &self.lines.to_string().as_bytes()) {
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
            match print_colored_stderr(clrerr, &self.syslines.to_string().as_bytes()) {
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
            match print_colored_stderr(clrerr, &("None".as_bytes())) {
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
            match print_colored_stderr(clrerr, &("None".as_bytes())) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                },
                _ => {},
            }
        } else {
            eprint!("{:?}", self.dt_first);
        }
        eprintln!(" }}");
    }
}

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
    let basename = FPath::from(riter.next().unwrap_or(&""));
    basename
}

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
            (fpath.clone().to_owned(), blocksz, *filter_dt_after_opt, *filter_dt_before_opt, tz_offset.clone());
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
        return (fpath.clone(), result);
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
            let mut fp1: FPath = FPath::new();
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
                        match print_colored_stdout(*clr, out.as_bytes()) { _ => {}};
                    } else {
                        if cli_opt_prepend_utc || cli_opt_prepend_local {
                            match (*slp_min).dt {
                                Some(dt) => {
                                    let fmt: chrono::format::DelayedFormat<chrono::format::StrftimeItems<'_>>;
                                    if cli_opt_prepend_utc {
                                        let dt_ = dt.with_timezone(&tz_utc);
                                        fmt = dt_.format(cli_opt_prepend_fmt);
                                    } else { // cli_opt_prepend_local
                                        let dt_ = dt.with_timezone(&tz_local);
                                        fmt = dt_.format(cli_opt_prepend_fmt);
                                    }
                                    write_stdout(&fmt.to_string().as_bytes());
                                    write_stdout(&" ".as_bytes());
                                },
                                _ => {},
                            }
                        }
                        match (*slp_min).print_color(*clr, color_datetime) {
                            Ok(_) => {},
                            Err(err) => {
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
                    return;
                }
            };
            eprintln!();

            let summary_opt = map_path_summary.remove(fpath);
            match summary_opt {
                Some(summary) => {
                    eprintln!("   {:?}", summary);
                },
                None => {
                    eprintln!("   None");
                }
            }
            let summary_print_opt = map_path_sumpr.remove(fpath);
            match summary_print_opt {
                Some(summary_print) => {
                    eprint!("   ");
                    summary_print.print_colored_stderr(&summary_opt);
                },
                None => {
                    eprint!("   ");
                    SummaryPrinted::default().print_colored_stderr(&summary_opt);
                }
            }
        }
        eprintln!("{:?}", sp_total);
    }

    debug_eprintln!("{}run_4: E _count_recv_ok {:?} _count_recv_di {:?}", so(), _count_recv_ok, _count_recv_di);
    debug_eprintln!("{}run_4()", sx());
}
