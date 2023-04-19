// src/readers/utmpreader.rs

//! Implements a [`UtmpxReader`],
//! the driver of deriving [`Utmpx`s] from a [utmpx format] file using a
//! [`BlockReader`].
//!
//! Sibling of [`SyslogProcessor`]. But simpler in a number of ways due to
//! predictable format of the utmpx files.
//!
//! The utmpx format is a [POSIX binary file format] used by
//! POSIX-compliant operating systems.
//!
//! This is an _s4lib_ structure used by the binary program _s4_.
//!
//! Implements [Issue #70].
//!
//! [`UtmpxReader`]: self::UtmpxReader
//! [`Utmpx`s]: crate::data::utmpx::Utmpx
//! [`BlockReader`]: crate::readers::blockreader::BlockReader
//! [utmpx format]: https://en.wikipedia.org/wiki/Utmp
//! [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor
//! [open-source binary file format]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/utmpx.h.html
//! [POSIX binary file format]: https://en.wikipedia.org/w/index.php?title=Utmp&oldid=1143772537#utmpx,_wtmpx_and_btmpx
//! [Issue #70]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/70

// TODO: [2023/02/27] create sibling of this for
//       parsing `lastlog` files.
//       https://github.com/shadow-maint/shadow/blob/4.13/src/lastlog.c#L172-L200
//       consider creating GenericEntryReader
//       here is a small Generics example to help get started
//       https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=a01e6139dffe49f26b59188662a6fdb0
//       the `GenericEntryReader` could be used for all fixed-size struct-based
//       log file formats.
//       - Overview of `lastlog` https://unix.stackexchange.com/a/530157/21203
//       - `pam_lastlog` https://www.man7.org/linux/man-pages/man8/pam_lastlog.8.html
//       - OpenBSD `utmp.h` https://github.com/openbsd/src/blob/24e9bd867b8d4b967f896aaa4b182c6616ac610b/include/utmp.h
//         which defines `struct lastlog` and `stuct utmp` (noticeably different than `utmpx`)
//       - `pam_lastlog.c` reading a `lastlog` entry https://github.com/linux-pam/linux-pam/blob/v1.5.2/modules/pam_lastlog/pam_lastlog.c#L264-L369
//       - according to file `/usr/include/lastlog.h`, the `struct lastlog` is defined in
//         `utmp.h`
//       - file `bits/utmp.h` at https://elixir.bootlin.com/glibc/glibc-2.37/source/bits/utmp.h#L57
//         defines `struct utmp` (more similarly to `utmpx`)
//
//       Also see `faillog` format https://github.com/shadow-maint/shadow/blob/4.13/lib/faillog.h

// TODO: ask question on SO about difference in
//       `e_termination` and `e_exit` in `struct exit_status`
//       https://elixir.bootlin.com/glibc/glibc-2.37/source/bits/utmp.h#L48


use crate::de_wrn;
use crate::common::{
    Count,
    FPath,
    FileOffset,
    FileSz,
    FileType,
    filetype_to_logmessagetype,
    ResultS3,
};
use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
    SystemTime,
    Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    dt_after_or_before,
    dt_pass_filters,
};
use crate::data::utmpx::{
    buffer_to_utmpx,
    Utmpx,
    utmpx,
    UTMPX_SZ,
    UTMPX_SZ_FO,
};
use crate::readers::blockreader::{
    BlockIndex,
    BlockOffset,
    BlockReader,
    BlockSz,
    ResultReadDataToBuffer,
};
use crate::readers::summary::Summary;

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fmt;
use std::io::{Error, ErrorKind, Result};

use ::mime_guess::MimeGuess;
use ::more_asserts::debug_assert_le;
#[allow(unused_imports)]
use ::si_trace_print::{
    de,
    defn,
    defo,
    defx,
    defñ,
    def1ñ,
    den,
    deo,
    dex,
    deñ,
    pfo,
    pfn,
    pfx,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// UtmpxReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Map [`FileOffset`] To [`Utmpx`].
///
/// Storage for `Utmpx` found from the underlying `BlockReader`.
/// FileOffset key is the first byte/offset that begins the `Utmpx`.
///
/// [`FileOffset`]: crate::common::FileOffset
/// [`Utmpx`]: crate::data::utmpx::Utmpx
pub type FoToEntry = BTreeMap<FileOffset, Utmpx>;

/// Map [`FileOffset`] To `FileOffset`
///
/// [`FileOffset`]: crate::common::FileOffset
pub type FoToFo = BTreeMap<FileOffset, FileOffset>;

/// [`UtmpxReader.find`*] functions results.
///
/// [`UtmpxReader.find`*]: self::UtmpxReader#method.find_entry
pub type ResultS3UtmpxFind = ResultS3<(FileOffset, Utmpx), Error>;

#[cfg(test)]
pub type SetDroppedEntries = HashSet<FileOffset>;

/// A specialized reader that uses [`BlockReader`] to read [utmpx] entries
/// in a file.
///
/// The `UtmpxReader` converts `[u8]` to [`utmpx`] in [`buffer_to_utmpx`].
///
/// A `UtmpxReader` stores past lookups of data in `self.entries`.
///
/// _XXX: not a rust "Reader"; does not implement trait [`Read`]._
///
/// [`buffer_to_utmpx`]: crate::data::utmpx::buffer_to_utmpx
/// [`utmpx`]: https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html
/// [utmpx]: https://en.wikipedia.org/wiki/Utmp
/// [`BlockReader`]: crate::readers::blockreader::BlockReader
/// [`Read`]: std::io::Read
pub struct UtmpxReader {
    pub(crate) blockreader: BlockReader,
    /// Timezone to use for conversions using function
    /// [`convert_tvsec_utvcsec_datetime`].
    ///
    /// [`convert_tvsec_utvcsec_datetime`]: crate::data::utmpx::convert_tvsec_utvcsec_datetime
    pub(crate) tz_offset: FixedOffset,
    /// Track [`Utmpx`] found among blocks in `blockreader`. Key is
    /// [`FileOffset`] which should match [`Utmpx.fileoffset_beg`].
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    /// [`FileOffset`]: crate::common::FileOffset
    /// [`Utmpx.fileoffset_beg`]: crate::data::utmpx::Utmpx#structfield.fileoffset_beg
    pub(crate) entries: FoToEntry,
    /// "high watermark" of `Utmpx` stored in `self.entries`
    pub(crate) entries_stored_highest: usize,
    /// Internal stats - hits of `self.entries` in `find_entry*` functions.
    pub(super) entries_hits: Count,
    /// Internal stats - misses of `self.entries` in `find_entry*` functions.
    pub(super) entries_miss: Count,
    /// `Count` of `Utmpx`s processed.
    ///
    /// Distinct from `self.entries.len()` as that may have contents removed.
    pub(super) entries_processed: Count,
    /// First (soonest) processed [`DateTimeL`] (not necessarily printed,
    /// not representative of the entire file).
    ///
    /// Intended for `--summary`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    // TODO: [2023/03/22] change behavior to be "first printed" instead of "first processed"
    pub(super) dt_first: DateTimeLOpt,
    /// Last (latest) processed [`DateTimeL`] (not necessarily printed,
    /// not representative of the entire file).
    ///
    /// Intended for `--summary`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    // TODO: [2023/03/22] change behavior to be "last printed" instead of "last processed"
    pub(super) dt_last: DateTimeLOpt,
    /// `Count` of dropped `Utmpx`.
    pub(super) drop_entry_ok: Count,
    /// `Count` of failed drop attempts of `Utmpx`.
    pub(super) drop_entry_errors: Count,
    /// Largest `BlockOffset` of successfully dropped blocks.
    pub(super) blockoffset_drop_last: BlockOffset,
    /// testing-only tracker of successfully dropped `Utmpx`
    #[cfg(test)]
    pub(crate) dropped_entries: SetDroppedEntries,
    /// The last [`Error`], if any, as a `String`. Set by [`set_error`].
    ///
    /// Annoyingly, cannot [Clone or Copy `Error`].
    ///
    /// [`Error`]: std::io::Error
    /// [Clone or Copy `Error`]: https://github.com/rust-lang/rust/issues/24135
    /// [`set_error`]: self::UtmpxReader#method.set_error
    error: Option<String>,
}

impl fmt::Debug for UtmpxReader {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("UtmpxReader")
            .field("Path", &self.path())
            .field("Entries", &self.entries.len())
            .field("tz_offset", &self.tz_offset)
            .field("dt_first", &self.dt_first)
            .field("dt_last", &self.dt_last)
            .field("Error?", &self.error)
            .finish()
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct SummaryUtmpxReader {
    /// `UtmpxReader::utmp_entries`
    pub utmpxreader_utmp_entries: Count,
    pub utmpxreader_utmp_entries_max: Count,
    pub utmpxreader_utmp_entries_hit: Count,
    pub utmpxreader_utmp_entries_miss: Count,
    pub utmpxreader_drop_entry_ok: Count,
    pub utmpxreader_drop_entry_errors: Count,
    /// datetime soonest seen (not necessarily reflective of entire file)
    pub utmpxreader_datetime_first: DateTimeLOpt,
    /// datetime latest seen (not necessarily reflective of entire file)
    pub utmpxreader_datetime_last: DateTimeLOpt,
}

/// Implement the UtmpxReader.
impl UtmpxReader {
    /// Create a new `UtmpxReader`.
    pub fn new(
        path: FPath,
        blocksz: BlockSz,
        tz_offset: FixedOffset,
    ) -> Result<UtmpxReader> {
        defñ!("({:?}, {:?}, {:?})", path, blocksz, tz_offset);
        let blockreader = BlockReader::new(path, FileType::Utmpx, blocksz)?;
        Ok(
            UtmpxReader
            {
                blockreader,
                tz_offset,
                entries: FoToEntry::new(),
                entries_stored_highest: 0,
                entries_hits: 0,
                entries_miss: 0,
                entries_processed: 0,
                dt_first: DateTimeLOpt::None,
                dt_last: DateTimeLOpt::None,
                drop_entry_ok: 0,
                drop_entry_errors: 0,
                blockoffset_drop_last: 0,
                #[cfg(test)]
                dropped_entries: SetDroppedEntries::new(),
                error: None,
            }
        )
    }

    /// See [`BlockReader::blocksz`].
    ///
    /// [`BlockReader::blocksz`]: crate::readers::blockreader::BlockReader#method.blocksz
    #[inline(always)]
    pub const fn blocksz(&self) -> BlockSz {
        self.blockreader.blocksz()
    }

    /// See [`BlockReader::filesz`].
    ///
    /// [`BlockReader::filesz`]: crate::readers::blockreader::BlockReader#method.filesz
    #[inline(always)]
    pub const fn filesz(&self) -> FileSz {
        self.blockreader.filesz()
    }

    /// See [`BlockReader::filetype`].
    ///
    /// [`BlockReader::filetype`]: crate::readers::blockreader::BlockReader#method.filetype
    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        self.blockreader.filetype()
    }

    /// See [`BlockReader::path`].
    ///
    /// [`BlockReader::path`]: crate::readers::blockreader::BlockReader#method.path
    #[inline(always)]
    pub const fn path(&self) -> &FPath {
        self.blockreader.path()
    }

    /// See [`BlockReader::mimeguess`].
    ///
    /// [`BlockReader::mimeguess`]: crate::readers::blockreader::BlockReader#method.mimeguess
    #[inline(always)]
    pub const fn mimeguess(&self) -> MimeGuess {
        self.blockreader.mimeguess()
    }

    /// See [`BlockReader::mtime`].
    ///
    /// [`BlockReader::mtime`]: crate::readers::blockreader::BlockReader#method.mtime
    pub fn mtime(&self) -> SystemTime {
        self.blockreader.mtime()
    }

    /// `Count` of `Utmpx`s processed by this `UtmpxReader`
    /// (i.e. `self.entries_processed`).
    #[inline(always)]
    pub fn count_entries_processed(&self) -> Count {
        self.entries_processed
    }

    /// "high watermark" of `Utmpx` stored in `self.entries`.
    #[inline(always)]
    pub fn entries_stored_highest(&self) -> usize {
        self.entries_stored_highest
    }

    /// See [`BlockReader::block_offset_at_file_offset`].
    ///
    /// [`BlockReader::block_offset_at_file_offset`]: crate::readers::blockreader::BlockReader#method.block_offset_at_file_offset
    #[inline(always)]
    pub const fn block_offset_at_file_offset(
        &self,
        fileoffset: FileOffset,
    ) -> BlockOffset {
        BlockReader::block_offset_at_file_offset(fileoffset, self.blocksz())
    }

    /// See [`BlockReader::file_offset_at_block_offset`].
    ///
    /// [`BlockReader::file_offset_at_block_offset`]: crate::readers::blockreader::BlockReader#method.file_offset_at_block_offset
    #[inline(always)]
    pub const fn file_offset_at_block_offset(
        &self,
        blockoffset: BlockOffset,
    ) -> FileOffset {
        BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz())
    }

    /// See [`BlockReader::file_offset_at_block_offset_index`].
    ///
    /// [`BlockReader::file_offset_at_block_offset_index`]: crate::readers::blockreader::BlockReader#method.file_offset_at_block_offset_index
    #[inline(always)]
    pub const fn file_offset_at_block_offset_index(
        &self,
        blockoffset: BlockOffset,
        blockindex: BlockIndex,
    ) -> FileOffset {
        BlockReader::file_offset_at_block_offset_index(blockoffset, self.blocksz(), blockindex)
    }

    /// See [`BlockReader::block_index_at_file_offset`].
    ///
    /// [`BlockReader::block_index_at_file_offset`]: crate::readers::blockreader::BlockReader#method.block_index_at_file_offset
    #[inline(always)]
    pub const fn block_index_at_file_offset(
        &self,
        fileoffset: FileOffset,
    ) -> BlockIndex {
        BlockReader::block_index_at_file_offset(fileoffset, self.blocksz())
    }

    /// See [`BlockReader::count_blocks`].
    ///
    /// [`BlockReader::count_blocks`]: crate::readers::blockreader::BlockReader#method.count_blocks
    #[inline(always)]
    pub const fn count_blocks(&self) -> Count {
        BlockReader::count_blocks(self.filesz(), self.blocksz()) as Count
    }

    /// See [`BlockReader::blockoffset_last`].
    ///
    /// [`BlockReader::blockoffset_last`]: crate::readers::blockreader::BlockReader#method.blockoffset_last
    pub const fn blockoffset_last(&self) -> BlockOffset {
        self.blockreader
            .blockoffset_last()
    }

    /// See [`BlockReader::fileoffset_last`].
    ///
    /// [`BlockReader::fileoffset_last`]: crate::readers::blockreader::BlockReader#method.fileoffset_last
    pub const fn fileoffset_last(&self) -> FileOffset {
        self.blockreader
            .fileoffset_last()
    }

    /// Is the passed `FileOffset` the last byte of the file?
    pub const fn is_fileoffset_last(
        &self,
        fileoffset: FileOffset,
    ) -> bool {
        self.fileoffset_last() == fileoffset
    }

    /// Is the passed `Utmpx` the last of the file?
    pub const fn is_last(
        &self,
        entry: &Utmpx,
    ) -> bool {
        self.is_fileoffset_last(entry.fileoffset_end())
    }

    #[inline(always)]
    pub const fn utmpsize(&self) -> usize {
        UTMPX_SZ
    }

    /// Return the `FileOffset` that is adjusted to the beginning offset of
    /// a [`utmpx`] entry.
    ///
    /// [`utmpx`]: https://github.com/freebsd/freebsd-src/blob/release/12.4.0/include/utmpx.h#L43-L56
    pub const fn fileoffset_to_utmpoffset (
        &self,
        fileoffset: FileOffset,
    ) -> FileOffset {
        (fileoffset / self.utmpsize() as FileOffset) * self.utmpsize() as FileOffset
    }

    /// Return all currently stored `FileOffset` in `self.entries`.
    ///
    /// Only intended to aid testing.
    #[cfg(test)]
    pub fn get_fileoffsets(&self) -> Vec<FileOffset> {
        self.entries
            .keys()
            .cloned()
            .collect()
    }

    fn set_error(
        &mut self,
        error: &Error,
    ) {
        defñ!("{:?}", error);
        if let Some(ref _err) = self.error {
            de_wrn!("skip overwrite of previous Error {:?} with Error ({:?})", _err, error);
            return;
        }
        self.error = Some(error.to_string());
    }

    /// Store information about a single [`Utmpx`] entry.
    ///
    /// Should only be called by `self.find_entry_impl`.
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    fn insert_entry(
        &mut self,
        entry: Utmpx,
    ) {
        defn!("@{}", entry.fileoffset_begin());
        let fo_beg: FileOffset = entry.fileoffset_begin();
        debug_assert!(
            !self
                .entries
                .contains_key(&fo_beg),
            "self.entries already contains key {}",
            fo_beg
        );
        self.entries
            .insert(fo_beg, entry);
        self.entries_stored_highest = std::cmp::max(self.entries_stored_highest, self.entries.len());
        self.entries_processed += 1;
        self.dt_first_last_update(entry.dt());
        defx!();
    }

    /// Update the two statistic `DateTimeL` of
    /// `self.dt_first` and `self.dt_last`.
    fn dt_first_last_update(
        &mut self,
        datetime: &DateTimeL,
    ) {
        defñ!("({:?})", datetime);
        // TODO: the `dt_first` and `dt_last` are only for `--summary`,
        //       no need to always copy datetimes.
        //       Would be good to only run this when `if self.do_summary {...}`
        match self.dt_first {
            Some(dt_first_) => {
                if &dt_first_ > datetime {
                    self.dt_first = Some(*datetime);
                }
            }
            None => {
                self.dt_first = Some(*datetime);
            }
        }
        match self.dt_last {
            Some(dt_last_) => {
                if &dt_last_ < datetime {
                    self.dt_last = Some(*datetime);
                }
            }
            None => {
                self.dt_last = Some(*datetime);
            }
        }
    }

    /// Forcefully `drop` the [`Utmpx`s] and [`Block`s]
    /// up to and including the passed `fileoffset`.
    ///
    /// [`Utmpx`s]: crate::data::utmpx::Utmpx
    /// [`Block`s]: crate::readers::blockreader::Block
    pub fn drop_entries(
        &mut self,
        fileoffset: FileOffset,
    ) -> bool {
        defn!("({})", fileoffset);

        let bo: BlockOffset;
        match self.entries.get(&fileoffset) {
            Some(entry_) => {
                bo = entry_.blockoffset_end(self.blocksz());
                // drops are done in per-block manner, not per entry.
                // This reduces overhead per processed entry by dropping
                // in batches.
                // So first quickly see if there is a possible block to drop
                if bo <= self.blockoffset_drop_last {
                    defx!("no entries to drop; {} <= {}", bo, self.blockoffset_drop_last);
                    return true;
                }
            }
            None => {
                defx!("failed to find entry at {}", fileoffset);
                return false;
            }
        }


        let bo_max: BlockOffset = BlockReader::block_offset_at_file_offset(
            fileoffset, self.blocksz()
        );
        // collect a "batch" of entry keys to drop
        let mut ret = true;
        let fo_drop: HashSet<FileOffset> = self.entries
            .iter()
            .filter_map(|(fo_, entry)| {
                    if fo_ <= &fileoffset
                        && entry.blockoffset_end(self.blocksz()) <= bo_max
                    {
                        Some(*fo_)
                    } else {
                        None
                    }
                }
            ).collect();
        defo!("fo_drop: {:?}", fo_drop);

        // drop the entries
        for fo in fo_drop.iter() {
            if self.drop_entry(fo) {
                self.drop_entry_ok += 1;
            } else {
                self.drop_entry_errors += 1;
                ret = false;
            }
        }
        defx!("return {}", ret);

        ret
    }

    /// Forcefully `drop` the [`Utmpx`] and [`Block`s] associated with the
    /// passed [`FileOffset`] key.
    ///
    /// The caller must know what they are doing!
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    /// [`Block`s]: crate::readers::blockreader::Block
    /// [`FileOffset`]: crate::common::FileOffset
    fn drop_entry(
        &mut self,
        fileoffset: &FileOffset,
    ) -> bool {
        defn!("({})", fileoffset);
        let entry = match self.entries.remove(fileoffset) {
            Some(entry_) => entry_,
            None => {
                defx!("FileOffset {} not found; return {}", fileoffset, true);
                return true;
            },
        };
        #[cfg(test)]
        {
            self.dropped_entries
                .insert(*fileoffset);
        }
        let mut ret = true;
        let mut bo_beg: BlockOffset = entry.blockoffset_begin(self.blocksz());
        let bo_end: BlockOffset = entry.blockoffset_end(self.blocksz());
        debug_assert_le!(bo_beg, bo_end);
        while bo_beg <= bo_end {
            if !self
                .blockreader
                .drop_block(bo_beg)
            {
                ret = false;
            } else {
                // the largest blockoffset that has been dropped should also
                // imply that all prior blockoffsets have been dropped
                self.blockoffset_drop_last = std::cmp::max(bo_beg, self.blockoffset_drop_last);
            }
            bo_beg += 1;
        }
        defx!("return {}", ret);

        ret
    }

    /// Check the internal storage if this `FileOffset` has a known return
    /// value for `find_entry`.
    #[inline(always)]
    fn check_store(
        &mut self,
        fileoffset: FileOffset,
    ) -> Option<Utmpx> {
        match self.entries.get(&fileoffset) {
            Some(entry) => {
                defñ!("({}): found in store", fileoffset);
                self.entries_hits += 1;

                Some(entry.clone())
            }
            None => {
                defñ!("({}): not found in store", fileoffset);
                self.entries_miss += 1;

                None
            }
        }
    }

    /// Find the [`utmpx`] at `fileoffset` staying within the same [`Block`].
    ///
    /// If a `utmpx` at `fileoffset` extends before or after the `Block` then
    /// [`Done`] is returned.
    ///
    /// The returned `Found(fileoffset, …)` may refer to
    /// a different proceeding `Block`.
    ///
    /// Also see [`find_entry`].
    ///
    /// Wrapper for private `find_entry_impl`.
    ///
    /// This function is _O(1)_.
    ///
    /// [`utmpx`]: https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html
    /// [`Block`]: crate::readers::blockreader::Block
    /// [`Done`]: crate::common::ResultS3#variant.Done
    /// [`find_entry`]: Self::find_entry
    pub fn find_entry_in_block(
        &mut self,
        fileoffset: FileOffset,
    ) -> ResultS3UtmpxFind {
        self.find_entry_impl(fileoffset, true)
    }

    /// Find the [`utmpx`] at the passed [`FileOffset`].
    ///
    /// During the process of finding, this creates and stores the
    /// [`Utmpx`] from underlying [`Block`] data.
    /// A returned [`Found`] includes the `FileOffset` that is one-byte past the
    /// found `utmpx` entry (the value of [`Utmpx.fileoffset_end`]).
    /// Reaching end of file returns `FileOffset` value that is one byte past
    /// the actual end of file.
    /// Otherwise returns [`Done`].
    /// All other [`Result::Err`] errors are propagated.
    ///
    /// Correllary to function [`find_entry_in_block`].
    ///
    /// Caller must ensure that the passed `FileOffset` is "pointing" to the
    /// beginning of a `utmpx` entry.
    ///
    /// Wrapper for private `find_entry_impl`.
    ///
    /// This function is _O(1)_.
    ///
    /// [`utmpx`]: https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html 
    /// [`Block`]: crate::readers::blockreader::Block
    /// [`Found`]: super::utmpxreader::ResultS3UtmpxFind#variant.Found
    /// [`Done`]: super::utmpxreader::ResultS3UtmpxFind#variant.Done
    /// [`Err`]: super::utmpxreader::ResultS3UtmpxFind#variant.Err
    /// [`Result::Err`]: std::result::Result#variant.Err
    /// [`FileOffset`]: crate::common::FileOffset
    /// [`find_entry_in_block`]: BlockReader#method.find_entry_in_block
    /// [`Utmpx.fileoffset_end`]: crate::data::utmpx::Utmpx#method.fileoffset_end
    pub fn find_entry(
        &mut self,
        fileoffset: FileOffset,
    ) -> ResultS3UtmpxFind {
        self.find_entry_impl(fileoffset, false)
    }

    /// Implementation of `find_entry` and `find_entry_in_block` functions.
    ///
    /// This function is _O(1)_.
    fn find_entry_impl(
        &mut self,
        fileoffset: FileOffset,
        oneblock: bool,
    ) -> ResultS3UtmpxFind {
        defn!("({}, {})", fileoffset, oneblock);

        let filesz: FileSz = self.filesz();

        // handle special cases
        if filesz == 0 {
            defx!("({}): return ResultS3UtmpxFind::Done, None; file is empty", fileoffset);
            return ResultS3UtmpxFind::Done;
        } else if fileoffset >= filesz {
            defx!(
                "({0}): return ResultS3UtmpxFind::Done(), None; fileoffset {0} is at end of file {1}!",
                fileoffset,
                filesz
            );
            return ResultS3UtmpxFind::Done;
        }

        let fileoffset = fileoffset - (fileoffset % UTMPX_SZ_FO);

        // check container of `Utmpx`s
        if let Some(utmpx) = self.check_store(fileoffset) {
            defx!("({}): return ResultS3UtmpxFind::Found(({:?}, …)); check_store found it", fileoffset, utmpx.fileoffset_end());
            return ResultS3UtmpxFind::Found((utmpx.fileoffset_end(), utmpx));
        }

        #[cfg(debug_assertions)]
        if fileoffset % UTMPX_SZ_FO != 0 {
            de!("WARNING: fileoffset {} not multiple of {}", fileoffset, UTMPX_SZ_FO);
        }

        defo!("searching for utmpx entry …");

        let mut buffer: [u8; UTMPX_SZ] = [0; UTMPX_SZ];
        let at: usize = match self.blockreader.read_data_to_buffer(
            fileoffset,
            fileoffset + UTMPX_SZ_FO,
            oneblock,
            &mut buffer,
        ) {
            ResultReadDataToBuffer::Found(val) => val,
            ResultReadDataToBuffer::Done => {
                defx!("({}): return ResultS3UtmpxFind::Done, None; read_data_to_buffer returned Done", fileoffset);
                return ResultS3UtmpxFind::Done;
            }
            ResultReadDataToBuffer::Err(err) => {
                self.set_error(&err);
                defx!("({}): return ResultS3UtmpxFind::Err({:?})", fileoffset, err);
                return ResultS3UtmpxFind::Err(err);
            }
        };

        debug_assert_eq!(at, UTMPX_SZ, "buffer at {}, expected {}", at, UTMPX_SZ);
        debug_assert_eq!(buffer.len(), UTMPX_SZ, "buffer len {}, expected {}", buffer.len(), UTMPX_SZ);

        if at != UTMPX_SZ {
            let err = Error::new(
                ErrorKind::Other,
                format!("buffer of len {} given too little data {}", buffer.len(), at),
            );
            self.set_error(&err);
            defx!("return ResultS3UtmpxFind::Err({})", err);
            return ResultS3UtmpxFind::Err(err);
        }

        let utmp_: utmpx = match buffer_to_utmpx(&buffer) {
            Some(utmp_) => utmp_,
            None => {
                let err = Error::new(ErrorKind::Other, "buffer_to_utmpx failed");
                self.set_error(&err);
                defx!("return ResultS3UtmpxFind::Err({})", err);
                return ResultS3UtmpxFind::Err(err);
            }
        };
        let utmpx: Utmpx = Utmpx::new(fileoffset, &self.tz_offset, utmp_);
        defo!("found utmp entry: {:?}", utmpx);
        self.insert_entry(utmpx.clone());
        let fo_end: FileOffset = utmpx.fileoffset_end();

        defx!("({}) return ResultS3UtmpxFind::Found({}, {:?})", fileoffset, fo_end, utmpx);

        ResultS3UtmpxFind::Found((fo_end, utmpx))
    }

    /// Find the nearest [`Utmpx`] at or after the `fileoffset` and
    /// after the optional `dt_filter` filter.
    ///
    /// This does a binary search over the file; _O(log(n))_.
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    pub fn find_entry_at_datetime_filter(
        &mut self,
        fileoffset: FileOffset,
        dt_filter: &DateTimeLOpt,
    ) -> ResultS3UtmpxFind {
        defn!("({}, {:?})", fileoffset, dt_filter);

        let fileoffset = fileoffset - (fileoffset % UTMPX_SZ_FO);

        let dtf = match dt_filter {
            Some(dt_) => dt_,
            None => {
                defx!("return self.find_entry({})", fileoffset);
                return self.find_entry(fileoffset);
            }
        };

        // search "cursor" at beginning
        let mut fo_a: FileOffset = fileoffset;
        let fo_sz: FileOffset = self.filesz() as FileOffset;
        // search "cursor" at end
        let mut fo_b: FileOffset = fo_sz;
        let mut fo_prior;
        // binary search for utmp entry with datetime nearest to `dt_filter`.
        // For each loop, try to narrow the difference of search cursors
        // until they arrive at the same utmp entry offset.
        //
        // XXX: Presumes utmp entries are stored sequentially in datetime order.
        //      Here is the open-source implementation of `last` that walks
        //      the utmp file backwards:
        //      https://github.com/util-linux/util-linux/blob/v2.38.1/login-utils/last.c#L720-L903
        //      However, that implementation does not presume stored entries are
        //      in datetime order.
        loop {
            let result = self.find_entry(fo_a);
            match result {
                ResultS3UtmpxFind::Found((fo_, utmpx)) => {
                    defo!("compare dt_filter {} to utmpx.dt() {}", dtf, utmpx.dt());
                    if utmpx.dt() < dtf {
                        debug_assert_le!(fo_, fo_b);
                        fo_prior = fo_a;
                        // jump forward
                        fo_a = self.fileoffset_to_utmpoffset(fo_a + (fo_b - fo_a) / 2 + 1);
                        if fo_prior > fo_a {
                            fo_a = fo_prior;
                        }
                        defo!("jumped forward: cursor range is now [{}, {}]", fo_a, fo_b);
                        if fo_prior == fo_a {
                            if ! self.is_last(&utmpx) {
                                defx!("!is_last; early return ResultS3UtmpxFind::Found(({}, …)); A1", fo_b);
                                return self.find_entry(fo_b);
                            }
                            defx!("return ResultS3UtmpxFind::Found(({}, …)); A2", fo_);
                            return ResultS3UtmpxFind::Found((fo_, utmpx));
                        }
                    }
                    else {
                        fo_prior = fo_b;
                        // jump backward
                        fo_b = fo_a;
                        fo_a = std::cmp::max(
                            fileoffset,
                            self.fileoffset_to_utmpoffset(fo_a / 2)
                        );
                        if fo_prior < fo_b {
                            fo_b = fo_prior;
                        }
                        defo!("jumped backward: cursor range is now [{}, {}]", fo_a, fo_b);
                        if fo_prior == fo_b {
                            defx!("return ResultS3UtmpxFind::Found(({}, …)); B", fo_);
                            return ResultS3UtmpxFind::Found((fo_, utmpx));
                        }
                    }
                }
                ResultS3UtmpxFind::Done => {
                    defx!("return ResultS3UtmpxFind::Done");
                    return ResultS3UtmpxFind::Done;
                }
                ResultS3UtmpxFind::Err(err) => {
                    self.set_error(&err);
                    defx!("return ResultS3UtmpxFind::Err({})", err);
                    return ResultS3UtmpxFind::Err(err);
                }
            }
        }
    }

    /// Find the nearest [`Utmpx`] at or after the `fileoffset` and
    /// after the optional `dt_filter` filter.
    ///
    /// This does a binary search over the file, _O(log(n))_.
    pub fn find_entry_between_datetime_filters(
        &mut self,
        fileoffset: FileOffset,
        dt_filter_after: &DateTimeLOpt,
        dt_filter_before: &DateTimeLOpt,
    ) -> ResultS3UtmpxFind {
        defn!("({}, {:?}, {:?})", fileoffset, dt_filter_after, dt_filter_before);

        let fileoffset = fileoffset - (fileoffset % UTMPX_SZ_FO);

        match self.find_entry_at_datetime_filter(fileoffset, dt_filter_after) {
            ResultS3UtmpxFind::Found((fo, entry)) => {
                defo!("returned ResultS3UtmpxFind::Found(({}, {:?}))", fo, entry);
                match Self::entry_pass_filters(&entry, dt_filter_after, dt_filter_before) {
                    Result_Filter_DateTime2::InRange => {
                        defo!("entry_pass_filters(…) returned InRange;");
                        defx!("return ResultS3UtmpxFind::Found(({}, {:?}))", fo, entry);
                        return ResultS3UtmpxFind::Found((fo, entry));
                    }
                    Result_Filter_DateTime2::BeforeRange => {
                        defo!("entry_pass_filters(…) returned BeforeRange;");
                        eprintln!("ERROR: entry_pass_filters({:?}, {:?}) returned BeforeRange, however the prior call to find_sysline_at_datetime_filter({}, {:?}) returned Found; this is unexpected.",
                                  dt_filter_after, dt_filter_before,
                                  fileoffset, dt_filter_after
                        );
                        defx!("return ResultS3UtmpxFind::Done (not sure what to do here)");
                        return ResultS3UtmpxFind::Done;
                    }
                    Result_Filter_DateTime2::AfterRange => {
                        defo!("entry_pass_filters(…) returned AfterRange;");
                        defx!("return ResultS3UtmpxFind::Done");
                        return ResultS3UtmpxFind::Done;
                    }
                };
            }
            ResultS3UtmpxFind::Done => {
                defo!("returned ResultS3UtmpxFind::Done");
            }
            ResultS3UtmpxFind::Err(err) => {
                defo!("returned Err({})", err);
                defx!("return ResultS3UtmpxFind::Err({})", err);
                return ResultS3UtmpxFind::Err(err);
            }
        };

        defx!("return ResultS3UtmpxFind::Done");

        ResultS3UtmpxFind::Done
    }

    /// Wrapper function for call to [`datetime::dt_after_or_before`] using the
    /// [`Utmpx::dt`] of the `entry`.
    ///
    /// [`datetime::dt_after_or_before`]: crate::data::datetime::dt_after_or_before
    /// [`Utmpx::dt`]: crate::data::utmpx::Utmpx::dt
    pub fn entry_dt_after_or_before(
        entry: &Utmpx,
        dt_filter: &DateTimeLOpt,
    ) -> Result_Filter_DateTime1 {
        defñ!("({:?})", dt_filter);

        dt_after_or_before(entry.dt(), dt_filter)
    }

    /// Wrapper function for call to [`datetime::dt_pass_filters`] using the
    /// [`Utmpx::dt`] of the `entry`.
    ///
    /// [`datetime::dt_pass_filters`]: crate::data::datetime::dt_pass_filters
    /// [`Utmpx::dt`]: crate::data::utmpx::Utmpx::dt
    pub fn entry_pass_filters(
        entry: &Utmpx,
        dt_filter_after: &DateTimeLOpt,
        dt_filter_before: &DateTimeLOpt,
    ) -> Result_Filter_DateTime2 {
        defn!("({:?}, {:?})", dt_filter_after, dt_filter_before);

        let result: Result_Filter_DateTime2 = dt_pass_filters(entry.
            dt(),
            dt_filter_after,
            dt_filter_before
        );
        defx!("(…) return {:?};", result);

        result
    }

    /// Return an up-to-date [`SummaryUtmpxReader`] instance for this
    /// `UtmpxReader`.
    ///
    /// [`SummaryUtmpxReader`]: SummaryUtmpxReader
    #[allow(non_snake_case)]
    pub fn summary(&self) -> SummaryUtmpxReader {
        let utmpxreader_utmp_entries: Count = self.entries_processed;
        let utmpxreader_utmp_entries_max: Count = self.entries_stored_highest as Count;
        let utmpxreader_utmp_entries_hit: Count = self.entries_hits as Count;
        let utmpxreader_utmp_entries_miss: Count = self.entries_miss as Count;
        let utmpxreader_drop_entry_ok: Count = self.drop_entry_ok;
        let utmpxreader_drop_entry_errors: Count = self.drop_entry_errors;
        let utmpxreader_datetime_first = self.dt_first;
        let utmpxreader_datetime_last = self.dt_last;

        SummaryUtmpxReader {
            utmpxreader_utmp_entries,
            utmpxreader_utmp_entries_max,
            utmpxreader_utmp_entries_hit,
            utmpxreader_utmp_entries_miss,
            utmpxreader_drop_entry_ok,
            utmpxreader_drop_entry_errors,
            utmpxreader_datetime_first,
            utmpxreader_datetime_last,
        }
    }

    /// Return an up-to-date [`Summary`] instance for this `UtmpxReader`.
    ///
    /// [`Summary`]: crate::readers::summary::Summary
    pub fn summary_complete(&self) -> Summary {
        let path = self.path().clone();
        let filetype = self.filetype();
        let logmessagetype = filetype_to_logmessagetype(filetype);
        let summaryblockreader = self.blockreader.summary();
        let summaryutmpreader = self.summary();
        let error: Option<String> = self.error.clone();

        Summary::new(
            path,
            filetype,
            logmessagetype,
            Some(summaryblockreader),
            None,
            None,
            None,
            Some(summaryutmpreader),
            None,
            error,
        )
    }
}
