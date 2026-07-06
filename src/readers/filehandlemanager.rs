// src/readers/filehandlemanager.rs
//
// This file was initially created by Co-pilot + GPT-5.5 and heavily revised by human @jtmoon79

//! Managed file handle cache for limiting simultaneously open files
//! via a global singleton [`FILE_HANDLE_MANAGER`].
//!
//! The [`FileHandleManager`] struct and related structs work two ways:
//! 1. A maximum number of outstanding open files is configured, `open_max`, and any
//!    open/read requests beyond `open_max` forcefully close the least recently used
//!    file handle. Later, that file may re-opened if requested
//!    and the new handle to the same seek position as the prior closed handle.
//! 2. Upon first open error of "_too many open files_" the manager will reduce
//!    `open_max` to the current number of
//!    open files and retry the open. If the retry succeeds then
//!    the `FileHandleManager` will resume the requested operation.
//!
//! Point 2. allows the `FileHandleManager` to adapt to the actual system limits of
//! open files, which may be lower than the configured maximum of Point 1.
//!
//! In theory, only approach 2. could have been implemented but we don't have
//! certainty about the error code that signifies "_too many open files_" on all
//! platforms.
//! Relatedly, consequences of hitting the system limit may have other
//! side effects. So implementing 1. allows the manager to avoid hitting the
//! system limit in the first place.
//! That's not perfect efficiency but it's good enough in practice.
//!
//! Additionally, unmanaged handles can be tracked by the `FILE_HANDLE_MANAGER` to
//! avoid exceeding the system limit of open files. But as the name implies,
//! unmanaged handles cannot be forcefully closed to make space for new open file
//! requests.
//!
//! The user may adjust the maximum number of simultaneously open files, `open_max`,
//! via environment variable `S4_FILE_HANDLE_OPEN_MAX`.
//!
//! See [Issue #270](https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/270).

use std::collections::HashMap;
use std::hash::Hash;
use std::io::{
    Error,
    ErrorKind,
    Read,
    Result,
    Seek,
    SeekFrom,
    Write,
};
use std::num::NonZeroUsize;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
use std::sync::{
    Arc,
    Mutex,
    Weak,
};

use ::lazy_static::lazy_static;
use ::lru::LruCache;
use ::si_trace_print::{
    defñ,
    def1n,
    def1o,
    def1x,
    def1ñ,
};

use crate::common::{
    debug_panic,
    Count,
    File,
    FileMetadata,
    FileOpenOptions,
    PathId,
    summary_stat,
};
use crate::debug::printers::e_wrn;

/// Environment variable used to override [`FILE_HANDLE_OPEN_MAX_DEFAULT`].
pub const ENV_FILE_HANDLE_OPEN_MAX: &str = "S4_FILE_HANDLE_OPEN_MAX";

/// Number of file descriptors reserved for file handles used before
/// any use of `FILE_HANDLE_MANAGER` occurs.
/// Typically these are file handles created automatically by the Operating System
/// when loading external libraries during program startup.
const FILE_HANDLE_OPEN_MAX_RLIMIT_RESERVE: usize = 8;

/// Default maximum number of simultaneously managed open file handles.
///
/// On Linux, on a low-resource Debian 12 system the `ulimit -n` limit was found to be 1024.
///
/// On Windows the lowest possible limit is 512; see <https://superuser.com/a/1356327/167043>.
pub const FILE_HANDLE_OPEN_MAX_DEFAULT: OpenMaxCountType = {
    #[cfg(windows)]
    {
        unsafe { OpenMaxCountType::new_unchecked(480) }
    }
    #[cfg(not(windows))]
    {
        unsafe { OpenMaxCountType::new_unchecked(980) }
    }
};

/// The role for a handle associated with a [`PathId`].
/// The `FileHandleManager` does not enforce behaviors for different
/// `FileHandleRole` values.
/// They identify distinct handle slots for the same [`PathId`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FileHandleRole {
    /// Primary read slot for a path. Use this for handles that are owned for
    /// long lifetimes.
    ///
    /// This role identifies the main reader-owned read handle for a
    /// [`PathId`].
    PrimaryRead,
    /// Secondary read slot for a path.
    ///
    /// This role identifies an additional readable handle for the same
    /// [`PathId`] when caller code needs an independent stream position from
    /// [`FileHandleRole::PrimaryRead`].
    SecondaryRead,
    /// Secondary write slot for a path.
    ///
    /// This role identifies an additional writable handle for the same
    /// [`PathId`] when caller code needs a managed output stream distinct from
    /// the read slots.
    SecondaryWrite,
    /// Unmanaged OS handle slot for a path.
    ///
    /// This role identifies a file handle that exists but is owned outside the
    /// [`FileHandleManager`].
    /// The manager will reserve capacity for the unmanaged handle but will not
    /// manage its lifecycle.
    Unmanaged,
}

/// Cache key for a managed file handle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FileHandleKey {
    path_id: PathId,
    role: FileHandleRole,
}

impl FileHandleKey {
    /// Create a new [`FileHandleKey`].
    pub const fn new(
        path_id: PathId,
        role: FileHandleRole,
    ) -> Self {
        Self { path_id, role }
    }
}

/// Re-openable file open options used by [`FileHandleManager`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenOptionsManaged {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    #[cfg(test)]
    force_open_error: Option<(i32, u32)>,
}

impl OpenOptionsManaged {
    /// Open an existing file read-only.
    pub const fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            #[cfg(test)]
            force_open_error: None,
        }
    }

    /// Open an existing file for writing without truncating.
    pub const fn write_existing() -> Self {
        Self {
            read: false,
            write: true,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            #[cfg(test)]
            force_open_error: None,
        }
    }

    #[cfg(test)]
    pub(crate) const fn read_only_force_open_error(
        raw_os_error: i32,
        count: u32,
    ) -> Self {
        Self {
            read: true,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            force_open_error: Some((raw_os_error, count)),
        }
    }

    /// Wrap the call to `std::fs::File::open` with `OpenOptionsManaged` settings.
    fn open(
        &mut self,
        path: &Path,
    ) -> Result<File> {
        def1n!("({:?}, {:?})", self, path);
        #[cfg(test)]
        if let Some((raw_os_error, count)) = &mut self.force_open_error {
            if *count != 0 {
                *count -= 1;
                let err = Error::from_raw_os_error(*raw_os_error);
                def1x!("return forced Err({:?})", err);
                return Err(err);
            }
        }
        let mut open_options = FileOpenOptions::new();
        let result = open_options
            .read(self.read)
            .write(self.write)
            .append(self.append)
            .truncate(self.truncate)
            .create(self.create)
            .create_new(self.create_new)
            .open(path);
        match result {
            Ok(file) => {
                def1x!("return Ok(file)");
                Ok(file)
            }
            Err(err) => {
                def1x!("return Err({:?})", err);
                Err(err)
            }
        }
    }
}

/// Return true if the `err` is a "_too many open files_" error.
pub(crate) fn is_error_too_many_open_files(err: &Error) -> bool {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            err.raw_os_error()
                .is_some_and(
                    |raw_os_error|
                    matches!(
                        ::nix::errno::Errno::from_raw(raw_os_error),
                        ::nix::errno::Errno::EMFILE | ::nix::errno::Errno::ENFILE
                    )
                )
        } else if #[cfg(windows)] {
            const ERROR_TOO_MANY_OPEN_FILES: i32 = 4;
            err.raw_os_error()
                .is_some_and(|raw_os_error| raw_os_error == ERROR_TOO_MANY_OPEN_FILES)
        } else {
            false
        }
    }
}

/// Summary statistics for [`FileHandleManager`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SummaryFileHandleManager {
    pub open_max_default: u32,
    pub open_max_adjusted: u32,
    pub managed_open_count_hi: u32,
    pub unmanaged_count_hi: u32,
    pub count_hi: u32,
    pub request_open_calls: Count,
    pub request_read_calls: Count,
    pub request_unmanaged_open_calls: Count,
    pub read_calls: Count,
    pub write_calls: Count,
    pub seek_calls: Count,
    pub metadata_calls: Count,
    pub physical_open_calls: Count,
    pub physical_open_error_calls: Count,
    pub physical_reopen_calls: Count,
    pub evict_succeed: Count,
    pub evict_fails: Count,
}

#[derive(Debug)]
struct FileEntryManaged {
    path: PathBuf,
    open_options: OpenOptionsManaged,
    seek_pos: u64,
    opened_once: bool,
    active_handles: usize,
    file: Option<File>,
}

impl FileEntryManaged {
    fn new(
        path: &Path,
        open_options: OpenOptionsManaged,
    ) -> Self {
        def1ñ!("({:?}, {:?})", path, open_options);
        Self {
            path: path.to_path_buf(),
            open_options,
            seek_pos: 0,
            opened_once: false,
            active_handles: 0,
            file: None,
        }
    }
}

pub type OpenMaxCountType = NonZeroUsize;
type OpenCountType = u32;

#[derive(Debug)]
struct FileHandleManagerState {
    /// Collection of managed file handles.
    handles_managed: HashMap<FileHandleKey, FileEntryManaged>,
    /// Collection of unmanaged file handles.
    handles_unmanaged: HashMap<FileHandleKey, usize>,
    /// Least-recently-used cache of managed file handles.
    lru: LruCache<FileHandleKey, ()>,
    /// Current number of open managed file handles.
    open_count: OpenCountType,
    /// Configured maximum number of simultaneously open file handles, managed and unmanaged.
    open_max: OpenMaxCountType,
    /// A summary of activity and statistics for the file handle manager.
    /// Only in-use when user-passed `--summary`.
    summary: SummaryFileHandleManager,
}

impl FileHandleManagerState {
    fn new(open_max: OpenMaxCountType) -> Self {
        def1ñ!("open_max={}", open_max);
        let lru_capacity = open_max;
        let mut summary = SummaryFileHandleManager::default();
        summary_stat!(summary.open_max_default = open_max.get() as u32);
        summary_stat!(summary.open_max_adjusted = summary.open_max_default);
        Self {
            handles_managed: HashMap::new(),
            handles_unmanaged: HashMap::new(),
            lru: LruCache::new(lru_capacity),
            open_count: 0,
            open_max,
            summary,
        }
    }

    /// Return the number of unmanaged handles currently open.
    fn unmanaged_count(&self) -> OpenCountType {
        self.handles_unmanaged
            .values()
            .sum::<usize>() as OpenCountType
    }

    /// Return the total number of open handles, managed and unmanaged.
    fn total_open_count(&self) -> OpenCountType {
        self.open_count + self.unmanaged_count()
    }

    /// Update the high-water marks for simultaneously open file handles.
    fn update_open_count_hi(&mut self) {
        let unmanaged_count = self.unmanaged_count() as u32;
        summary_stat!(
            self.summary.managed_open_count_hi = std::cmp::max(
                self.summary.managed_open_count_hi,
                self.open_count as u32,
            )
        );
        summary_stat!(
            self.summary.unmanaged_count_hi = std::cmp::max(
                self.summary.unmanaged_count_hi,
                unmanaged_count,
            )
        );
        summary_stat!(
            self.summary.count_hi = std::cmp::max(
                self.summary.count_hi,
                self.open_count as u32 + unmanaged_count,
            )
        );
    }

    /// Make room for one more managed file handle, evicting a different managed file handle if necessary.
    fn make_room_for_one(&mut self, key: FileHandleKey) -> Result<()> {
        def1n!("open_count={} unmanaged_count={} open_max={}", self.open_count, self.unmanaged_count(), self.open_max);
        while self.total_open_count() >= (self.open_max.get() as OpenCountType) {
            if ! self.evict_one() {
                def1x!("return Err(no managed file handle available)");
                let uc = self.unmanaged_count();
                let mc = self.open_count;
                let om = self.open_max.get();
                return Err(
                    Error::new(
                        ErrorKind::WouldBlock,
                        format!("no managed file handle available for eviction: currently {uc} unmanaged, {mc} managed, open_max {om}; key {key:?}")
                    )
                );
            }
        }
        def1x!("return Ok(())");

        Ok(())
    }

    /// Mark a managed file handle as recently used.
    fn touch(
        &mut self,
        key: FileHandleKey,
    ) {
        def1ñ!("key={:?}", key);
        self.lru.put(key, ());
    }

    /// Retain a managed file handle, incrementing its active handle count.
    fn retain_handle(
        &mut self,
        key: FileHandleKey,
    ) -> Result<()> {
        def1n!("key={:?}", key);
        let Some(entry) = self.handles_managed.get_mut(&key) else {
            def1x!("return Err(unregistered key {:?})", key);
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("managed file handle {:?} was not registered", key),
            ));
        };
        entry.active_handles += 1;
        def1x!("active_handles={}, return Ok(())", entry.active_handles);

        Ok(())
    }

    /// Release a managed file handle, decrementing its active handle count.
    fn release_handle(
        &mut self,
        key: FileHandleKey,
    ) {
        def1n!("key={:?}", key);
        let mut remove_lru_key = false;
        let Some(entry) = self.handles_managed.get_mut(&key) else {
            def1x!("missing entry for {:?}", key);
            return;
        };
        if entry.active_handles == 0 {
            def1x!("active_handles already 0 for {:?}", key);
            return;
        }
        entry.active_handles -= 1;
        if entry.active_handles != 0 {
            def1x!("active_handles={}, return", entry.active_handles);
            return;
        }
        if let Some(mut file) = entry.file.take() {
            match file.stream_position() {
                Ok(seek_pos) => {
                    def1o!("record seek_pos {} for {:?}", seek_pos, key);
                    entry.seek_pos = seek_pos;
                }
                Err(_err) => {
                    def1o!("stream_position failed for {:?}: {:?}", key, _err);
                }
            }
            self.open_count = self
                .open_count
                .saturating_sub(1);
            remove_lru_key = true;
        }
        if remove_lru_key {
            self.lru.pop(&key);
        }
        def1x!("return");
    }

    /// Retain an unmanaged file handle, incrementing its active handle count.
    fn retain_unmanaged_handle(
        &mut self,
        key: FileHandleKey,
    ) -> Result<()> {
        def1n!("key={:?}", key);
        if key.role != FileHandleRole::Unmanaged {
            def1x!("return Err(invalid unmanaged key {:?})", key);
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("unmanaged file handle {:?} must use FileHandleRole::Unmanaged", key),
            ));
        }
        self.make_room_for_one(key)?;
        *self.handles_unmanaged.entry(key).or_insert(0) += 1;
        self.update_open_count_hi();
        def1x!("handles_unmanaged={}, return Ok(())", self.handles_unmanaged.get(&key).copied().unwrap_or_default());

        Ok(())
    }

    /// Release an unmanaged file handle, decrementing its active handle count.
    fn release_unmanaged_handle(
        &mut self,
        key: FileHandleKey,
    ) {
        def1n!("key={:?}", key);
        let Some(count) = self.handles_unmanaged.get_mut(&key) else {
            def1x!("missing unmanaged entry for {:?}", key);
            return;
        };
        *count = count.saturating_sub(1);
        if *count == 0 {
            self.handles_unmanaged.remove(&key);
        }
        def1x!("return");
    }

    /// Evict one currently-open managed file handle, if possible.
    fn evict_one(&mut self) -> bool {
        def1n!("open_count={} open_max={}", self.open_count, self.open_max);
        debug_assert_eq!(self.open_count as usize, self.lru.len(), "open_count {} != lru.len() {}", self.open_count, self.lru.len());
        while let Some((key, ())) = self.lru.pop_lru() {
            def1o!("consider evict key {:?}", key);
            let Some(entry) = self.handles_managed.get_mut(&key) else {
                def1o!("missing entry for {:?}", key);
                debug_panic!("key {:?} from self.lru was not in self.handles_managed()", key);
                continue;
            };
            if let Some(mut file) = entry.file.take() {
                match file.stream_position() {
                    Ok(seek_pos) => {
                        def1o!("record seek_pos {} for {:?}", seek_pos, key);
                        entry.seek_pos = seek_pos;
                    }
                    Err(_err) => {
                        def1o!("stream_position failed for {:?}: {:?}", key, _err);
                    }
                }
                self.open_count = self
                    .open_count
                    .saturating_sub(1);
                summary_stat!(self.summary.evict_succeed += 1);
                def1x!("evicted {:?}, return true", key);
                return true;
            }
        }
        summary_stat!(self.summary.evict_fails += 1);
        def1x!("return false");

        false
    }

    /// Ensure a managed file handle is open, opening it if necessary.
    fn ensure_open(
        &mut self,
        key: FileHandleKey,
    ) -> Result<()> {
        def1n!("({:?})", key);
        if self
            .handles_managed
            .get(&key)
            .is_some_and(|entry| entry.file.is_some())
        {
            self.touch(key);
            def1x!("already open, return Ok(())");
            return Ok(());
        }

        self.make_room_for_one(key)?;

        match self.open_entry_once(key) {
            Ok(()) => {
                def1x!("return Ok(())");
                Ok(())
            }
            Err(err) if is_error_too_many_open_files(&err) =>
                self.open_entry_once_adjust_open_max(key, err),
            Err(err) => {
                def1x!("return Err(open failed: {:?})", err);
                Err(err)
            }
        }
    }

    /// Open a managed file handle once, without adjusting the maximum open count.
    /// This wraps the `std::fs::File::open` call and handles any errors that may occur.
    fn open_entry_once(
        &mut self,
        key: FileHandleKey,
    ) -> Result<()> {
        def1n!("({:?})", key);
        let entry = match self.handles_managed.get_mut(&key) {
            Some(entry) => entry,
            None => {
                def1x!("return Err(unregistered key {:?})", key);
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("managed file handle {:?} was not registered", key),
                ));
            }
        };
        let is_reopen = entry.opened_once;
        def1o!("open {:?} at seek_pos {}", entry.path, entry.seek_pos);
        let mut file = match entry
            .open_options
            .open(&entry.path)
        {
            Ok(file) => file,
            Err(err) => {
                def1x!("return Err(open failed for {:?}: {:?})", entry.path, err);
                summary_stat!(self.summary.physical_open_error_calls += 1);
                return Err(err);
            }
        };
        if entry.seek_pos != 0 {
            file.seek(SeekFrom::Start(entry.seek_pos))?;
        }
        entry.opened_once = true;
        entry.file = Some(file);
        self.open_count += 1;
        summary_stat!(self.summary.physical_open_calls += 1);
        if is_reopen {
            summary_stat!(self.summary.physical_reopen_calls += 1);
        }
        self.update_open_count_hi();
        self.touch(key);
        def1x!("return Ok(())");

        Ok(())
    }

    /// Open a managed file handle once, adjusting the maximum open count if necessary.
    ///
    /// If an `std::fs::File::open` call fails then we should:
    /// 1. decrease the `open_max` to the current number of open files
    /// 2. evict (close) a managed file handle if possible
    /// 3. retry the open
    ///
    /// In case the default value of `open_max` is too high for the system
    /// then this allows the manager to adapt to the actual kernel's system limits of open files.
    fn open_entry_once_adjust_open_max(
        &mut self,
        key: FileHandleKey,
        err: Error,
    ) -> Result<()> {
        def1n!("({:?}, {:?})", key, err);
        // recalculate the number of open files, managed and unmanaged,
        // to adjust the maximum open count.
        // This typically runs after a OS-level "too many open files" open error occurs.
        let managed_open_count = self
            .handles_managed
            .values()
            .filter(|entry| entry.file.is_some())
            .count();
        let open_count = managed_open_count + self.unmanaged_count() as usize;
        let Some(open_max) = OpenMaxCountType::new(open_count) else {
            def1x!("return Err({:?})", err);
            return Err(err);
        };
        self.open_count = managed_open_count as OpenCountType;
        self.open_max = open_max;
        summary_stat!(self.summary.open_max_adjusted = open_max.get() as u32);
        if ! self.evict_one() {
            def1x!("return Err({:?})", err);
            return Err(err);
        }
        let result = self.open_entry_once(key);
        def1x!("return {:?}", result.is_ok());

        result
    }

    /// Run a closure with a mutable reference to the managed file handle `handles_managed`.
    fn with_file_mut<T>(
        &mut self,
        key: FileHandleKey,
        f: impl FnOnce(&mut File) -> Result<T>,
    ) -> Result<T> {
        def1n!("({:?})", key);
        self.ensure_open(key)?;
        let result = {
            let entry = self
                .handles_managed
                .get_mut(&key)
                .unwrap();
            let file = entry.file.as_mut().unwrap();
            let result = f(file);
            match file.stream_position() {
                Ok(seek_pos) => {
                    entry.seek_pos = seek_pos;
                    def1o!("seek_pos={}", seek_pos);
                }
                Err(_err) => {
                    def1o!("stream_position failed: {:?}", _err);
                }
            }

            result
        };
        self.touch(key);
        def1x!("return result");

        result
    }
}

/// Singleton manager for all managed file handles.
#[derive(Debug)]
pub struct FileHandleManager {
    state: Arc<Mutex<FileHandleManagerState>>,
}

impl FileHandleManager {
    pub fn new() -> Self {
        def1ñ!();

        Self::new_open_max(file_handle_open_max())
    }

    pub fn new_open_max(open_max: OpenMaxCountType) -> Self {
        def1ñ!("open_max={}", open_max);

        Self {
            state: Arc::new(Mutex::new(FileHandleManagerState::new(open_max))),
        }
    }

    /// Create a new managed file handle.
    fn managed_handle(
        &self,
        state: &mut FileHandleManagerState,
        key: FileHandleKey,
    ) -> Result<FileHandleManaged> {
        def1n!("key={:?}", key);
        state.retain_handle(key)?;
        def1x!("return Ok(FileHandleManaged)");

        Ok(FileHandleManaged {
            key,
            state: Arc::downgrade(&self.state),
        })
    }

    /// Create a new unmanaged file handle.
    fn unmanaged_handle(
        &self,
        state: &mut FileHandleManagerState,
        key: FileHandleKey,
    ) -> Result<FileHandleUnmanaged> {
        def1n!("key={:?}", key);
        state.retain_unmanaged_handle(key)?;
        def1x!("return Ok(FileHandleUnmanaged)");

        Ok(FileHandleUnmanaged {
            key,
            state: Arc::downgrade(&self.state),
        })
    }

    /// Register and open a managed file handle.
    pub fn request_open(
        &self,
        path_id: PathId,
        role: FileHandleRole,
        path: &Path,
        open_options: OpenOptionsManaged,
    ) -> Result<FileHandleManaged> {
        def1n!("path_id={} role={:?} path={:?}", path_id, role, path);
        if role == FileHandleRole::Unmanaged {
            def1x!("return Err(FileHandleRole::Unmanaged cannot be managed)");
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "FileHandleRole::Unmanaged cannot be used for managed file handles",
            ));
        }
        let key = FileHandleKey::new(path_id, role);
        let mut state = self.state.lock().unwrap();
        summary_stat!(
            state
                .summary
                .request_open_calls += 1
        );
        let mut closed_existing = false;
        if let Some(entry) = state.handles_managed.get_mut(&key) {
            if entry.file.take().is_some() {
                closed_existing = true;
            }
            entry.path = path.to_path_buf();
            entry.open_options = open_options;
            entry.seek_pos = 0;
        } else {
            state
                .handles_managed
                .insert(key, FileEntryManaged::new(path, open_options));
        }
        if closed_existing {
            state.open_count = state
                .open_count
                .saturating_sub(1);
        }
        state.ensure_open(key)?;
        def1x!("return Ok(FileHandleManaged)");

        self.managed_handle(&mut state, key)
    }

    /// Return a previously registered managed file handle.
    pub fn request_read(
        &self,
        path_id: PathId,
        role: FileHandleRole,
    ) -> Result<FileHandleManaged> {
        def1n!("path_id={} role={:?}", path_id, role);
        if role == FileHandleRole::Unmanaged {
            def1x!("return Err(FileHandleRole::Unmanaged cannot be managed)");
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "FileHandleRole::Unmanaged cannot be used for managed file handles",
            ));
        }
        let key = FileHandleKey::new(path_id, role);
        let mut state = self.state.lock().unwrap();
        summary_stat!(
            state
                .summary
                .request_read_calls += 1
        );
        if !state
            .handles_managed
            .contains_key(&key)
        {
            def1x!("return Err(unregistered key {:?})", key);
            return Err(
                Error::new(
                    ErrorKind::NotFound,
                    format!("managed file handle {:?} was not registered", key)
                )
            );
        }
        state.ensure_open(key)?;
        def1x!("return Ok(FileHandleManaged)");

        self.managed_handle(&mut state, key)
    }

    /// Reserve capacity for a file handle owned outside this manager.
    pub fn request_unmanaged_open(
        &self,
        path_id: PathId,
        role: FileHandleRole,
        _path: &Path,
    ) -> Result<FileHandleUnmanaged> {
        def1n!("path_id={} role={:?} path={:?}", path_id, role, _path);
        if role != FileHandleRole::Unmanaged {
            def1x!("return Err(unmanaged handle requires FileHandleRole::Unmanaged)");
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "unmanaged file handles must use FileHandleRole::Unmanaged",
            ));
        }
        let key = FileHandleKey::new(path_id, role);
        let mut state = self.state.lock().unwrap();
        summary_stat!(
            state
                .summary
                .request_unmanaged_open_calls += 1
        );
        def1x!("return Ok(FileHandleUnmanaged)");

        self.unmanaged_handle(&mut state, key)
    }

    /// Return a copy of the current manager summary.
    pub fn summary(&self) -> SummaryFileHandleManager {
        def1ñ!();
        self.state
            .lock()
            .unwrap()
            .summary
    }

    /// Evict one currently-open managed file handle, if possible.
    pub(crate) fn evict_one(&self) -> bool {
        def1ñ!();
        self.state
            .lock()
            .expect("file handle manager lock poisoned during evict_one()")
            .evict_one()
    }

    /// Return the configured maximum number of simultaneously open files.
    pub fn open_max(&self) -> OpenMaxCountType {
        self.state
            .lock()
            .expect("file handle manager lock poisoned during open_max()")
            .open_max
    }

    /// Return the number of managed open handles.
    #[allow(unused)]
    pub(crate) fn open_count(&self) -> u32 {
        self.state
            .lock()
            .expect("file handle manager lock poisoned during open_count()")
            .open_count
    }

    /// Return the number of managed and unmanaged open handles.
    #[allow(unused)]
    pub(crate) fn total_open_count(&self) -> u32 {
        self.state
            .lock()
            .expect("file handle manager lock poisoned during total_open_count()")
            .total_open_count()
    }

    /// Return the number of managed active handles for a given `path_id` and `role`.
    #[allow(unused)]
    pub(crate) fn handles_managed_active_helper(
        &self,
        path_id: PathId,
        role: FileHandleRole,
    ) -> usize {
        def1ñ!("path_id={} role={:?}", path_id, role);
        self.state
            .lock()
            .expect("file handle manager lock poisoned during handles_managed_active_helper()")
            .handles_managed
            .get(&FileHandleKey::new(path_id, role))
            .map_or(0, |entry| entry.active_handles)
    }

    /// Return the number of unmanaged handles for a given `path_id` and `role`.
    #[allow(unused)]
    pub(crate) fn handles_unmanaged_helper(
        &self,
        path_id: PathId,
        role: FileHandleRole,
    ) -> usize {
        def1ñ!("path_id={} role={:?}", path_id, role);
        self.state
            .lock()
            .expect("file handle manager lock poisoned during handles_unmanaged_helper()")
            .handles_unmanaged
            .get(&FileHandleKey::new(path_id, role))
            .copied()
            .unwrap_or_default()
    }

    /// Testing helper.
    #[cfg(test)]
    pub(crate) fn with_file_mut_helper<T>(
        &self,
        handle: &FileHandleManaged,
        update_summary: impl FnOnce(&mut SummaryFileHandleManager),
        f: impl FnOnce(&mut File) -> Result<T>,
    ) -> Result<T> {
        def1n!("({:?})", handle.key);
        let mut state = self
            .state
            .lock()
            .expect("file handle manager lock poisoned during with_file_mut_helper()");
        update_summary(&mut state.summary);
        let result = state.with_file_mut(handle.key, f);
        def1x!("return {:?}", result.is_ok());

        result
    }
}

/// Managed file handle wrapper.
#[derive(Debug)]
pub struct FileHandleManaged {
    key: FileHandleKey,
    state: Weak<Mutex<FileHandleManagerState>>,
}

/// Reservation for a file handle owned outside [`FileHandleManager`].
#[derive(Debug)]
pub struct FileHandleUnmanaged {
    key: FileHandleKey,
    state: Weak<Mutex<FileHandleManagerState>>,
}

impl Drop for FileHandleUnmanaged {
    fn drop(&mut self) {
        def1n!("({:?})", self.key);
        if let Some(state) = self.state.upgrade() {
            match state.lock() {
                Ok(mut state) => state.release_unmanaged_handle(self.key),
                Err(_err) => {
                    def1o!("file handle manager lock poisoned during unmanaged drop(): {:?}", _err);
                }
            }
        }
        def1x!("return");
    }
}

impl FileHandleManaged {
    /// Run a closure with a mutable reference to the manager state.
    fn with_state_mut<T>(
        &self,
        f: impl FnOnce(&mut FileHandleManagerState) -> Result<T>,
    ) -> Result<T> {
        def1n!("({:?})", self.key);
        let state = self
            .state
            .upgrade()
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "file handle manager is no longer available"))?;
        let mut state = state
            .lock()
            .map_err(|_| Error::new(ErrorKind::Other, "file handle manager lock poisoned"))?;
        let result = f(&mut state);
        def1x!("return {:?}", result.is_ok());

        result
    }

    /// Return file metadata for this handle.
    pub fn metadata(&self) -> Result<FileMetadata> {
        def1n!("({:?})", self.key);
        let result = self.with_state_mut(|state| {
            summary_stat!(state.summary.metadata_calls += 1);
            state.with_file_mut(self.key, |file| file.metadata())
        });
        def1x!("return result");

        result
    }
}

// Implement traits `Clone`, `Drop`, `Read`, `Write`, and `Seek` for `FileHandleManaged`.
// This way a `FileHandleManaged` can be a drop-in replacement for `std::fs::File` in most cases
// while still being managed by the `FileHandleManager`.

impl Clone for FileHandleManaged {
    fn clone(&self) -> Self {
        def1n!("({:?})", self.key);
        if let Some(state) = self.state.upgrade() {
            match state.lock() {
                Ok(mut state) => {
                    if let Err(_err) = state.retain_handle(self.key) {
                        def1o!("retain_handle({:?}) failed: {:?}", self.key, _err);
                    }
                }
                Err(_err) => {
                    def1o!("file handle manager lock poisoned: {:?}", _err);
                }
            }
        }
        def1x!("return clone");

        Self {
            key: self.key,
            state: self.state.clone(),
        }
    }
}

impl Drop for FileHandleManaged {
    fn drop(&mut self) {
        def1n!("({:?})", self.key);
        if let Some(state) = self.state.upgrade() {
            match state.lock() {
                Ok(mut state) => state.release_handle(self.key),
                Err(_err) => {
                    def1o!("file handle manager lock poisoned during drop(): {:?}", _err);
                }
            }
        }
        def1x!("return");
    }
}

impl Read for FileHandleManaged {
    fn read(
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize> {
        let mut handle: &FileHandleManaged = self;

        <&FileHandleManaged as Read>::read(&mut handle, buf)
    }
}

impl Read for &FileHandleManaged {
    fn read(
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize> {
        def1n!("({:?}, buf len {})", self.key, buf.len());
        let result = self.with_state_mut(|state| {
            summary_stat!(state.summary.read_calls += 1);
            state.with_file_mut(self.key, |file| file.read(buf))
        });
        def1x!("return {:?}", result);

        result
    }
}

impl Write for FileHandleManaged {
    fn write(
        &mut self,
        buf: &[u8],
    ) -> Result<usize> {
        def1n!("({:?}, buf len {})", self.key, buf.len());
        let result = self.with_state_mut(|state| {
            summary_stat!(state.summary.write_calls += 1);
            state.with_file_mut(self.key, |file| file.write(buf))
        });
        def1x!("return {:?}", result);

        result
    }

    fn flush(&mut self) -> Result<()> {
        def1n!("({:?})", self.key);
        let result = self.with_state_mut(|state| state.with_file_mut(self.key, |file| file.flush()));
        def1x!("return {:?}", result);

        result
    }
}
impl Seek for FileHandleManaged {
    fn seek(
        &mut self,
        pos: SeekFrom,
    ) -> Result<u64> {
        let mut handle: &FileHandleManaged = self;

        <&FileHandleManaged as Seek>::seek(&mut handle, pos)
    }
}

impl Seek for &FileHandleManaged {
    fn seek(
        &mut self,
        pos: SeekFrom,
    ) -> Result<u64> {
        def1n!("({:?}, {:?})", self.key, pos);
        let result = self.with_state_mut(|state| {
            summary_stat!(state.summary.seek_calls += 1);
            state.with_file_mut(self.key, |file| file.seek(pos))
        });
        def1x!("return {:?}", result);

        result
    }
}

lazy_static! {
    /// Global file handle manager singleton.
    pub static ref FILE_HANDLE_MANAGER: FileHandleManager = {
        defñ!("lazy_static! FILE_HANDLE_MANAGER::new()");

        FileHandleManager::new()
    };
}

/// state for `file_handle_open_max`
static FILE_HANDLE_OPEN_MAX_WRN: AtomicBool = AtomicBool::new(false);

/// wrapper function to get the env. var. `S4_FILE_HANDLE_OPEN_MAX`
/// and return a valid value.
/// On Unix, adjusts to `getrlimit`.
fn file_handle_open_max() -> OpenMaxCountType {
    let mut warning_invalid_value: Option<String> = None;
    let mut warning_env_error: Option<String> = None;
    let open_max: OpenMaxCountType = match std::env::var(ENV_FILE_HANDLE_OPEN_MAX) {
        Ok(value) => {
            let value_trimmed = value.trim();
            if value_trimmed.is_empty() {
                FILE_HANDLE_OPEN_MAX_DEFAULT
            } else {
                match value_trimmed.parse::<usize>() {
                    Ok(value) if value > 0 => OpenMaxCountType::new(value).unwrap(),
                    _ => {
                        warning_invalid_value = Some(value);

                        FILE_HANDLE_OPEN_MAX_DEFAULT
                    }
                }
            }
        }
        Err(std::env::VarError::NotPresent) => FILE_HANDLE_OPEN_MAX_DEFAULT,
        Err(err) => {
            warning_env_error = Some(err.to_string());

            FILE_HANDLE_OPEN_MAX_DEFAULT
        }
    };

    #[cfg(unix)]
    let open_max: OpenMaxCountType = {
        match ::nix::sys::resource::getrlimit(::nix::sys::resource::Resource::RLIMIT_NOFILE) {
            Ok((rlimit_soft, _rlimit_hard)) => {
                let rlimit_soft = usize::try_from(rlimit_soft).unwrap_or(usize::MAX);
                let adjusted = rlimit_soft
                    .saturating_sub(FILE_HANDLE_OPEN_MAX_RLIMIT_RESERVE)
                    .max(1);

                OpenMaxCountType::new(std::cmp::min(open_max.get(), adjusted)).unwrap()
            }
            Err(_err) => open_max,
        }
    };

    if let Some(value) = warning_invalid_value {
        // only warn once
        let wrn = FILE_HANDLE_OPEN_MAX_WRN.load(Ordering::Relaxed);
        FILE_HANDLE_OPEN_MAX_WRN.store(true, Ordering::Relaxed);
        if !wrn {
            e_wrn!(
                "environment variable {} value {:?} is not a decimal number greater than 0; using default {}",
                ENV_FILE_HANDLE_OPEN_MAX,
                value,
                open_max,
            );
        }
    }
    if let Some(err) = warning_env_error {
        e_wrn!(
            "environment variable {} could not be read: {}; using default {}",
            ENV_FILE_HANDLE_OPEN_MAX,
            err,
            open_max,
        );
    }
    defñ!("return {}", open_max);

    open_max
}
