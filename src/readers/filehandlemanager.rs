// src/readers/filehandlemanager.rs
//
// this file initially created by Co-pilot + GPT-5.5, heavily revised by human @jtmoon79

//! Managed file handle cache for limiting simultaneously open files.

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

/// Default maximum number of simultaneously open managed files.
///
/// On a small-resource Debian 12 system the `ulimit -n` limit is 1024.
/// On Windows the default limit is 512 (see https://superuser.com/a/1356327/167043).
/// Use a smaller number than that.
pub const FILE_HANDLE_OPEN_MAX_DEFAULT: OpenMaxCountType = unsafe { OpenMaxCountType::new_unchecked(480) };

/// The role for a managed handle associated with a [`PathId`].
/// The `FileHandleManager` does not enforce behaviors for different `FileHandleRole` values.
/// Merely they are suggestions and internally are used only to
/// distinguish multiple managed handles for the same [`PathId`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FileHandleRole {
    /// Primary read slot for a path.
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
        }
    }

    fn open(
        self,
        path: &Path,
    ) -> Result<File> {
        def1n!("({:?}, {:?})", self, path);
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

/// Summary statistics for [`FileHandleManager`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SummaryFileHandleManager {
    pub files_opened_hi: u32,
    pub request_open_calls: Count,
    pub request_read_calls: Count,
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
    entries: HashMap<FileHandleKey, FileEntryManaged>,
    lru: LruCache<FileHandleKey, ()>,
    open_count: OpenCountType,
    open_max: OpenMaxCountType,
    summary: SummaryFileHandleManager,
}

impl FileHandleManagerState {
    fn new(open_max: OpenMaxCountType) -> Self {
        def1ñ!("open_max={}", open_max);
        let lru_capacity = open_max;
        Self {
            entries: HashMap::new(),
            lru: LruCache::new(lru_capacity),
            open_count: 0,
            open_max,
            summary: SummaryFileHandleManager::default(),
        }
    }

    fn touch(
        &mut self,
        key: FileHandleKey,
    ) {
        def1ñ!("key={:?}", key);
        self.lru.put(key, ());
    }

    fn retain_handle(
        &mut self,
        key: FileHandleKey,
    ) -> Result<()> {
        def1n!("key={:?}", key);
        let Some(entry) = self.entries.get_mut(&key) else {
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

    fn release_handle(
        &mut self,
        key: FileHandleKey,
    ) {
        def1n!("key={:?}", key);
        let mut remove_lru_key = false;
        let Some(entry) = self.entries.get_mut(&key) else {
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

    fn evict_one(&mut self) -> bool {
        def1n!("open_count={} open_max={}", self.open_count, self.open_max);
        debug_assert_eq!(self.open_count as usize, self.lru.len(), "open_count {} != lru.len() {}", self.open_count, self.lru.len());
        while let Some((key, ())) = self.lru.pop_lru() {
            def1o!("consider evict key {:?}", key);
            let Some(entry) = self.entries.get_mut(&key) else {
                def1o!("missing entry for {:?}", key);
                debug_panic!("key {:?} from self.lru was not in self.entries()", key);
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

    fn ensure_open(
        &mut self,
        key: FileHandleKey,
    ) -> Result<()> {
        def1n!("({:?})", key);
        if self
            .entries
            .get(&key)
            .is_some_and(|entry| entry.file.is_some())
        {
            self.touch(key);
            def1x!("already open, return Ok(())");
            return Ok(());
        }

        while self.open_count >= (self.open_max.get() as OpenCountType) {
            if ! self.evict_one() {
                def1x!("return Err(no managed file handle available)");
                return Err(
                    Error::new(
                        ErrorKind::WouldBlock,
                        "no managed file handle available for eviction"
                    )
                );
            }
        }

        let entry = match self.entries.get_mut(&key) {
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
        summary_stat!(
            self.summary.files_opened_hi = std::cmp::max(self.summary.files_opened_hi, self.open_count as u32,)
        );
        self.touch(key);
        def1x!("return Ok(())");

        Ok(())
    }

    fn with_file_mut<T>(
        &mut self,
        key: FileHandleKey,
        f: impl FnOnce(&mut File) -> Result<T>,
    ) -> Result<T> {
        def1n!("({:?})", key);
        self.ensure_open(key)?;
        let result = {
            let entry = self
                .entries
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

    /// Register and open a managed file handle.
    pub fn request_open(
        &self,
        path_id: PathId,
        role: FileHandleRole,
        path: &Path,
        open_options: OpenOptionsManaged,
    ) -> Result<FileHandleManaged> {
        def1n!("path_id={} role={:?} path={:?}", path_id, role, path);
        let key = FileHandleKey::new(path_id, role);
        let mut state = self.state.lock().unwrap();
        summary_stat!(
            state
                .summary
                .request_open_calls += 1
        );
        let mut closed_existing = false;
        if let Some(entry) = state.entries.get_mut(&key) {
            if entry.file.take().is_some() {
                closed_existing = true;
            }
            entry.path = path.to_path_buf();
            entry.open_options = open_options;
            entry.seek_pos = 0;
        } else {
            state
                .entries
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
        let key = FileHandleKey::new(path_id, role);
        let mut state = self.state.lock().unwrap();
        summary_stat!(
            state
                .summary
                .request_read_calls += 1
        );
        if !state
            .entries
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

    /// Return a copy of the current manager summary.
    pub fn summary(&self) -> SummaryFileHandleManager {
        def1ñ!();
        self.state
            .lock()
            .unwrap()
            .summary
    }

    /// Return the configured maximum number of simultaneously open files.
    pub fn open_max(&self) -> OpenMaxCountType {
        self.state
            .lock()
            .expect("file handle manager lock poisoned during open_max()")
            .open_max
    }

    #[allow(unused)]
    pub(crate) fn open_count(&self) -> u32 {
        self.state
            .lock()
            .expect("file handle manager lock poisoned during open_count()")
            .open_count
    }

    #[allow(unused)]
    pub(crate) fn active_handles_helper(
        &self,
        path_id: PathId,
        role: FileHandleRole,
    ) -> usize {
        def1ñ!("path_id={} role={:?}", path_id, role);
        self.state
            .lock()
            .expect("file handle manager lock poisoned during active_handles_helper()")
            .entries
            .get(&FileHandleKey::new(path_id, role))
            .map_or(0, |entry| entry.active_handles)
    }

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

impl FileHandleManaged {
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
        def1n!("({:?}, buf len {})", self.key, buf.len());
        let result = self.with_state_mut(|state| {
            summary_stat!(state.summary.read_calls += 1);
            state.with_file_mut(self.key, |file| file.read(buf))
        });
        def1x!("return {:?}", result);

        result
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
        def1n!("({:?}, {:?})", self.key, pos);
        let result = self.with_state_mut(|state| {
            summary_stat!(state.summary.seek_calls += 1);
            state.with_file_mut(self.key, |file| file.seek(pos))
        });
        def1x!("return {:?}", result);

        result
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

/// wrapper function to get the env. var. `S4_FILE_HANDLE_OPEN_MAX`
/// and return a valid value.
fn file_handle_open_max() -> OpenMaxCountType {
    match std::env::var(ENV_FILE_HANDLE_OPEN_MAX) {
        Ok(value) => {
            let value_trimmed = value.trim();
            if value_trimmed.is_empty() {
                defñ!("empty env value, return default {}", FILE_HANDLE_OPEN_MAX_DEFAULT);
                return FILE_HANDLE_OPEN_MAX_DEFAULT;
            }
            match value_trimmed.parse::<usize>() {
                Ok(value) if value > 0 => {
                    defñ!("return {}", value);

                    OpenMaxCountType::new(value).unwrap()
                }
                _ => {
                    e_wrn!(
                        "environment variable {} value {:?} is not a decimal number greater than 0; using default {}",
                        ENV_FILE_HANDLE_OPEN_MAX,
                        value,
                        FILE_HANDLE_OPEN_MAX_DEFAULT,
                    );
                    defñ!("invalid env value, return default {}", FILE_HANDLE_OPEN_MAX_DEFAULT);

                    FILE_HANDLE_OPEN_MAX_DEFAULT
                }
            }
        }
        Err(std::env::VarError::NotPresent) => {
            defñ!("env not present, return default {}", FILE_HANDLE_OPEN_MAX_DEFAULT);

            FILE_HANDLE_OPEN_MAX_DEFAULT
        }
        Err(err) => {
            e_wrn!(
                "environment variable {} could not be read: {}; using default {}",
                ENV_FILE_HANDLE_OPEN_MAX,
                err,
                FILE_HANDLE_OPEN_MAX_DEFAULT,
            );
            defñ!("env error, return default {}", FILE_HANDLE_OPEN_MAX_DEFAULT);

            FILE_HANDLE_OPEN_MAX_DEFAULT
        }
    }
}
