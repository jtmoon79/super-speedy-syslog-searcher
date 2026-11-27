//! Capture-local input, calibration and binary framing for Apple Unified Logs.

use std::cell::RefCell;
use std::collections::{
    BTreeSet,
    HashMap,
    HashSet,
};
use std::fmt;
use std::fs;
use std::io::{
    self,
    Read,
};
use std::path::{
    Path,
    PathBuf,
};
use std::rc::Rc;
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};

use ::chrono::DateTime;
use ::compact_str::{
    CompactString,
    format_compact,
};
use ::macos_unifiedlogs::dsc::SharedCacheStrings;
use ::macos_unifiedlogs::timesync::TimesyncBoot;
use ::macos_unifiedlogs::traits::{
    FileProvider,
    SourceFile,
};
use ::macos_unifiedlogs::uuidtext::UUIDText;
#[allow(unused_imports)]
use ::si_trace_print::{
    def1n,
    def1o,
    def1x,
    def1ñ,
    defn,
    defo,
    defx,
    defñ,
    den,
    deo,
    dex,
    deñ,
};

use crate::common::{
    Count,
    FPath,
    summary_stats_enabled,
    summary_stat,
};
use crate::data::datetime::DateTimeL;
use crate::readers::filepreprocessor::{
    AUL_TRACE_DIRS,
    is_aul_trace,
};

const HEADER: u32 = 0x1000;
const CATALOG: u32 = 0x600b;
const CHUNKSET: u32 = 0x600d;

pub(super) type Clock = HashMap<String, TimesyncBoot>;
pub(crate) type Opener = Box<dyn Fn(&Path) -> io::Result<Box<dyn Read>>>;
type MissingType = RefCell<HashSet<PathBuf>>;
type Warnings = Rc<RefCell<BTreeSet<(CompactString, CompactString)>>>;

thread_local! {
    static LOG_SCOPE: RefCell<Option<(CompactString, Warnings)>> = const { RefCell::new(None) };
}

/// Forward warnings/errors from the CLI logger. Returns whether the record was consumed.
pub fn capture_upstream_log(record: &log::Record<'_>) -> bool {
    defn!("record");
    if !summary_stats_enabled() {
        return false;
    }
    if record.level() > log::Level::Warn || !record.target().starts_with("macos_unifiedlogs") {
        defx!("return false; record not captured");
        return false;
    }
    let ret: bool = LOG_SCOPE.with_borrow(|scope| {
        let Some((path, warnings)) = scope else {
            return false;
        };
        // Upstream diagnostics can interpolate raw records; retain their source location instead.
        summary_stat!({
            let reason = record.args().as_str().map_or_else(|| format_compact!(
                "upstream {} at {}:{}",
                record.level(),
                record.module_path().unwrap_or(record.target()),
                record.line().unwrap_or_default(),
            ), CompactString::new);
            warnings.borrow_mut().insert((path.clone(), reason));
        });
        true
    });
    defx!("return {ret}");

    ret
}

#[derive(Default)]
pub(super) struct SummaryAulParser {
    pub(super) files_opened: Count,
    pub(super) files_processed: Vec<FPath>,
    opened: HashSet<PathBuf>,
}

pub(crate) struct AulParser {
    pub(super) diagnostics: PathBuf,
    uuidtext: PathBuf,
    open: Opener,
    stats: RefCell<SummaryAulParser>,
    missing: MissingType,
    warnings: Warnings,
}

impl AulParser {
    pub(crate) fn new(diagnostics: PathBuf, uuidtext: PathBuf, open: Opener) -> Self {
        def1ñ!("diagnostics={diagnostics:?}, uuidtext={uuidtext:?}");
        let stats: RefCell<SummaryAulParser> = RefCell::default();
        let missing: MissingType = RefCell::default();
        let warnings: Warnings = Warnings::default();
        Self {
            diagnostics,
            uuidtext,
            open,
            stats,
            missing,
            warnings,
        }
    }

    pub(super) fn scoped<T>(&self, path: &Path, f: impl FnOnce() -> T) -> T {
        if !summary_stats_enabled() {
            return f();
        }
        struct Restore(Option<(CompactString, Warnings)>);
        impl Drop for Restore {
            fn drop(&mut self) {
                LOG_SCOPE.replace(self.0.take());
            }
        }
        let _restore = Restore(LOG_SCOPE.replace(Some((
            CompactString::new(path.to_string_lossy()),
            self.warnings.clone(),
        ))));

        f()
    }

    pub(super) fn warn(&self, path: &Path, reason: impl AsRef<str>) {
        summary_stat!(
            self.warnings.borrow_mut().insert((
                CompactString::new(path.to_string_lossy()),
                CompactString::new(reason.as_ref()
            ),
        )));
    }

    fn warn_fmt(&self, path: &Path, reason: fmt::Arguments<'_>) {
        summary_stat!(
            self.warnings.borrow_mut().insert((
                CompactString::new(path.to_string_lossy()),
                format_compact!("{reason}"),
            ),
        ));
    }

    pub(super) fn into_stats(self) -> (SummaryAulParser, Vec<(FPath, String)>) {
        let warnings = self.warnings.take().into_iter()
            .map(|(path, reason)| (path.into_string(), reason.into_string()))
            .collect();
        (self.stats.into_inner(), warnings)
    }

    fn open(&self, path: &Path) -> io::Result<Box<dyn Read>> {
        def1ñ!("opening {path:?}");
        let source = (self.open)(path)?;
        summary_stat!({
            let mut stats = self.stats.borrow_mut();
            if stats.opened.insert(path.to_path_buf()) {
                stats.files_opened += 1;
                stats.files_processed.push(path.to_string_lossy().into_owned());
            }
        });

        Ok(source)
    }

    pub(super) fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        def1n!("read {path:?}");
        let mut bytes: Vec<u8> = Vec::new();
        let result = self.open(path).and_then(|mut source| source.read_to_end(&mut bytes));
        if let Err(err) = result {
            self.warn_fmt(path, format_args!("cannot read capture file: {err}"));
            def1x!("Error {err}; {path:?}");
            return Err(err);
        }
        def1x!("read {} bytes from {path:?}", bytes.len());

        Ok(bytes)
    }

    fn list(&self, dir: &Path, include: impl Fn(&Path) -> bool, optional: bool) -> Vec<PathBuf> {
        defn!("listing directory: {dir:?}, optional: {optional}");
        let mut paths: Vec<PathBuf> = Vec::new();
        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            if include(&path) && path.is_file() {
                                paths.push(path);
                            }
                        }
                        Err(err) => self.warn_fmt(dir, format_args!("cannot list capture entry: {err}")),
                    }
                }
            }
            Err(err) if optional && err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => self.warn_fmt(dir, format_args!("cannot list capture directory: {err}")),
        }
        paths.sort_unstable();
        defx!("returning {} paths", paths.len());

        paths
    }

    pub(super) fn trace_paths(&self) -> Vec<PathBuf> {
        defn!("retrieving trace paths");
        let mut paths: Vec<PathBuf> = self.list(&self.diagnostics, is_aul_trace, false);
        for folder in AUL_TRACE_DIRS {
            let folder_path = &self.diagnostics.join(folder);
            defo!("append trace path {folder_path:?}");
            paths.extend(self.list(folder_path, is_aul_trace, true));
        }
        paths.sort_unstable();
        defx!("returning {} trace paths", paths.len());

        paths
    }

    fn timesync_paths(&self) -> Vec<PathBuf> {
        self.list(&self.diagnostics.join("timesync"), |path| {
            path.extension().is_some_and(|ext| ext == "timesync")
                && !path.file_name().unwrap_or_default().as_encoded_bytes().starts_with(b"._")
        }, false)
    }

    pub(super) fn clocks(&self, cancel: &AtomicBool) -> Clock {
        defn!("cancel={cancel:?}");
        let mut clocks = Clock::new();
        for path in self.timesync_paths() {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            let Ok(bytes) = self.read(&path) else { continue };
            let parsed = self.scoped(&path, || TimesyncBoot::parse_timesync_data(&bytes));
            let Ok((_, parsed)) = parsed else {
                self.warn(&path, "malformed timesync data");
                continue;
            };
            for (boot, mut map) in parsed {
                if boot.is_empty() || map.signature == 0
                    || !matches!((map.timebase_numerator, map.timebase_denominator), (1, 1) | (125, 3))
                {
                    self.warn(&path, "unusable timesync boot mapping");
                    continue;
                }
                let anchors = map.timesync.len();
                map.timesync.retain(|anchor| anchor.walltime > 0);
                if anchors != map.timesync.len() {
                    self.warn(&path, "timesync contains invalid wall-clock anchors");
                }
                if map.boot_time <= 0 && map.timesync.is_empty() {
                    self.warn(&path, "timesync boot has no usable clock anchors");
                    continue;
                }
                if let Some(existing) = clocks.get_mut(&boot) {
                    if (existing.timebase_numerator, existing.timebase_denominator)
                        != (map.timebase_numerator, map.timebase_denominator)
                    {
                        self.warn(&path, "conflicting timesync timebase for boot UUID");
                        continue;
                    }
                    if existing.boot_time <= 0 {
                        existing.boot_time = map.boot_time;
                    }
                    existing.timesync.append(&mut map.timesync);
                } else {
                    clocks.insert(boot, map);
                }
            }
        }
        for boot in clocks.values_mut() {
            // Stable ties retain the last captured anchor, as upstream does at a given tick.
            boot.timesync.sort_by_key(|anchor| anchor.kernel_time);
        }
        defx!("returning {} clocks", clocks.len());

        clocks
    }

    fn companion(&self, uuid: &str, dsc: bool) -> io::Result<(PathBuf, Vec<u8>)> {
        defn!("retrieving companion for uuid: {uuid}, dsc: {dsc}");
        if !(30..=32).contains(&uuid.len()) || !uuid.bytes().all(|n| n.is_ascii_hexdigit()) {
            defx!("invalid uuid: {uuid}");
            return Err(io::Error::from(io::ErrorKind::NotFound));
        }
        let uuid: UuidHex = pad_uuid_hex(uuid.as_bytes()).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "hex UUID is too long")
        })?;
        let uuid: &str = std::str::from_utf8(&uuid).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "hex UUID bytes are not valid UTF-8")
        })?;
        let path: PathBuf = if dsc {
            self.uuidtext.join("dsc").join(uuid)
        } else {
            self.uuidtext.join(&uuid[..2]).join(&uuid[2..])
        };
        // Don't repeatedly hit the filesystem for the same missing companion.
        if self.missing.borrow().contains(&path) {
            defx!("companion missing for path {path:?}");
            return Err(io::Error::from(io::ErrorKind::NotFound));
        }
        let bytes: Vec<u8> = self.read(&path).inspect_err(|_| {
            self.missing.borrow_mut().insert(path.clone());
        })?;
        defx!("retrieved companion at path {path:?}");

        Ok((path, bytes))
    }
}

struct CaptureFile<'a>(&'a AulParser, PathBuf, CompactString);

impl<'a> CaptureFile<'a> {
    fn new(parser: &'a AulParser, path: PathBuf) -> Self {
        let source_path = CompactString::new(path.to_string_lossy());
        Self(parser, path, source_path)
    }
}

struct ReadError {
    error: Option<io::Error>,
    kind: io::ErrorKind,
}

impl ReadError {
    fn new(error: io::Error) -> Self {
        let kind = error.kind();
        Self { error: Some(error), kind }
    }
}

impl Read for ReadError {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(self.error.take().unwrap_or_else(|| io::Error::from(self.kind)))
    }
}

impl SourceFile for CaptureFile<'_> {
    fn reader(&mut self) -> impl Read {
        self.0.open(&self.1).unwrap_or_else(|err| Box::new(ReadError::new(err)))
    }

    fn source_path(&self) -> &str {
        &self.2
    }
}

impl FileProvider for AulParser {
    fn tracev3_files(&self) -> impl Iterator<Item = impl SourceFile> {
        self.trace_paths().into_iter().map(|path| CaptureFile::new(self, path))
    }

    fn uuidtext_files(&self) -> impl Iterator<Item = impl SourceFile> {
        def1ñ!();
        let hex_name = |path: &Path, len| path.file_name().and_then(|name| name.to_str())
            .is_some_and(|name| name.len() == len && name.bytes().all(|n| n.is_ascii_hexdigit()));
        fs::read_dir(&self.uuidtext).into_iter().flatten().filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(move |path| path.is_dir() && hex_name(path, 2))
            .flat_map(move |path| self.list(&path, |path| hex_name(path, 30), false))
            .map(|path| CaptureFile::new(self, path))
    }

    fn read_uuidtext(&self, uuid: &str) -> io::Result<UUIDText> {
        def1ñ!();
        let (path, bytes) = self.companion(uuid, false)?;
        let result = self.scoped(&path, || UUIDText::parse_uuidtext(&bytes));
        result.map(|(_, data)| data).map_err(|_| {
            self.warn(&path, "malformed UUIDText companion");
            self.missing.borrow_mut().insert(path);
            io::Error::from(io::ErrorKind::InvalidData)
        })
    }

    fn dsc_files(&self) -> impl Iterator<Item = impl SourceFile> {
        def1ñ!();
        self.list(&self.uuidtext.join("dsc"), |path| {
            path.file_name().and_then(|name| name.to_str())
                .is_some_and(|name| (30..=32).contains(&name.len()) && name.bytes().all(|n| n.is_ascii_hexdigit()))
        }, false).into_iter().map(|path| CaptureFile::new(self, path))
    }

    fn read_dsc_uuid(&self, uuid: &str) -> io::Result<SharedCacheStrings> {
        def1ñ!();
        let (path, bytes) = self.companion(uuid, true)?;
        let result = self.scoped(&path, || SharedCacheStrings::parse_dsc(&bytes));
        result.map(|(_, data)| data).map_err(|_| {
            self.warn(&path, "malformed DSC companion");
            self.missing.borrow_mut().insert(path);
            io::Error::from(io::ErrorKind::InvalidData)
        })
    }

    fn timesync_files(&self) -> impl Iterator<Item = impl SourceFile> {
        self.timesync_paths().into_iter().map(|path| CaptureFile::new(self, path))
    }
}

/// Parse only complete outer frames, sharing framing across the metadata and decoding passes.
pub(super) fn frame(bytes: &[u8]) -> Option<(u32, &[u8], usize)> {
    let tag = u32::from_le_bytes(bytes.get(..4)?.try_into().ok()?);
    let size = usize::try_from(u64::from_le_bytes(bytes.get(8..16)?.try_into().ok()?)).ok()?;
    let end = 16usize.checked_add(size)?;
    let padded = end.checked_next_multiple_of(8)?;
    let chunk = bytes.get(..end)?;
    let len = if end == bytes.len() { end } else { padded };
    (len <= bytes.len()).then_some((tag, chunk, len))
}

/// Feed the upstream iterator one catalog at a time; it otherwise copies the entire tail per catalog.
pub(super) fn trace_batches(mut bytes: &[u8]) -> impl Iterator<Item = (bool, Option<&[u8]>)> {
    def1ñ!();
    std::iter::from_fn(move || {
        if bytes.is_empty() {
            return None;
        }
        let fresh_header: bool = frame(bytes).is_some_and(|(tag, _, _)| tag == HEADER);
        let mut end: usize = 0;
        let mut catalog: bool = false;
        while let Some((tag, _, len)) = frame(&bytes[end..]) {
            if end != 0 && (tag == HEADER || (tag == CATALOG && catalog)) {
                break;
            }
            catalog |= tag == CATALOG;
            end += len;
        }
        if end == 0 {
            bytes = &[];
            return Some((false, None));
        }
        let (batch, rest) = bytes.split_at(end);
        bytes = rest;
        Some((fresh_header, Some(batch)))
    })
}

pub(super) fn calibrated(boot: Option<&TimesyncBoot>, base: u64) -> bool {
    boot.is_some_and(|boot| !boot.timesync.is_empty() || (base == 0 && boot.boot_time > 0))
}

fn project(boot: &TimesyncBoot, tick: u64, anchor: u64, wall: i64) -> f64 {
    let scale = if (boot.timebase_numerator, boot.timebase_denominator) == (125, 3) { 125.0 / 3.0 } else { 1.0 };
    (tick as f64).mul_add(scale, -(anchor as f64) * scale) + wall as f64
}

pub(super) fn datetime(ns: f64) -> Option<DateTimeL> {
    (ns.is_finite() && ns >= i64::MIN as f64 && ns < i64::MAX as f64)
        .then(|| DateTime::from_timestamp_nanos(ns as i64).fixed_offset())
}

fn envelope(boot: &TimesyncBoot, start: u64, end: u64) -> Option<(DateTimeL, DateTimeL)> {
    def1ñ!();
    if start > end {
        return None;
    }
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    let mut add = |from: u64, to: u64, anchor, wall| {
        if from <= to && wall > 0 {
            lo = lo.min(project(boot, from, anchor, wall));
            hi = hi.max(project(boot, to, anchor, wall));
        }
    };
    if let Some(first) = boot.timesync.first() {
        if start < first.kernel_time {
            let to = end.min(first.kernel_time - 1);
            add(start, to, 0, boot.boot_time);
            add(start, to, first.kernel_time, first.walltime);
        }
        let first = boot.timesync.partition_point(|anchor| anchor.kernel_time <= start).saturating_sub(1);
        for (i, anchor) in boot.timesync.iter().enumerate().skip(first) {
            if anchor.kernel_time > end {
                break;
            }
            let mut to = end;
            if let Some(next) = boot.timesync.get(i + 1) {
                if next.kernel_time == anchor.kernel_time {
                    continue;
                }
                to = to.min(next.kernel_time - 1);
            }
            add(start.max(anchor.kernel_time), to, anchor.kernel_time, anchor.walltime);
        }
    } else {
        add(start, end, 0, boot.boot_time);
    }

    Some((datetime(lo.floor())?, datetime(hi.ceil())?))
}

fn field<const N: usize>(bytes: &[u8], offset: usize) -> Option<[u8; N]> {
    bytes.get(offset..offset.checked_add(N)?)?.try_into().ok()
}

type UuidHex = [u8; 32];

fn pad_uuid_hex(uuid: &[u8]) -> Option<UuidHex> {
    let mut padded = [b'0'; 32];
    let start = 32usize.checked_sub(uuid.len())?;
    padded.get_mut(start..)?.copy_from_slice(uuid);

    Some(padded)
}

fn encode_uuid_hex(uuid: u128) -> UuidHex {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = [b'0'; 32];
    for (index, byte) in encoded.iter_mut().enumerate() {
        let shift = 124 - index * 4;
        *byte = HEX[(uuid >> shift) as usize & 0xf];
    }

    encoded
}

fn catalog_bounds(
    bytes: &[u8], boot: &TimesyncBoot,
) -> Option<(usize, (DateTimeL, DateTimeL))> {
    defn!("{} bytes", bytes.len());
    // The crate keeps its catalog parser private; the on-disk offset avoids parsing unrelated tables.
    let offset: usize = usize::from(u16::from_le_bytes(field(bytes, 22)?));
    let count: usize = usize::from(u16::from_le_bytes(field(bytes, 24)?));
    let mut entries: &[u8] = bytes.get(40 + offset..)?;
    let mut bounds: Option<(DateTimeL, DateTimeL)> = None;
    for _ in 0..count {
        let start: u64 = u64::from_le_bytes(field(entries, 0)?);
        let end: u64 = u64::from_le_bytes(field(entries, 8)?);
        let compression: u32 = u32::from_le_bytes(field(entries, 20)?);
        if !matches!(compression, 0x100 | 0x700 | 0x701) {
            defx!("unsupported compression: {compression:#x}");
            return None;
        }
        let indexes: usize = usize::try_from(u32::from_le_bytes(field(entries, 24)?)).ok()?;
        let offsets_pos: usize = 28usize.checked_add(indexes.checked_mul(2)?)?;
        let offsets: usize = usize::try_from(u32::from_le_bytes(field(entries, offsets_pos)?)).ok()?;
        let len: usize = 32usize.checked_add(indexes.checked_add(offsets)?.checked_mul(2)?)?;
        entries = entries.get(len.checked_next_multiple_of(8)?..)?;
        let (lo, hi) = envelope(boot, start, end)?;
        bounds = Some(match bounds {
            Some((a, b)) => (a.min(lo), b.max(hi)),
            None => (lo, hi),
        });
    }
    defx!("return count={count} bounds={bounds:?}");

    Some((count, bounds?))
}

pub(super) fn trace_bounds(
    bytes: &[u8],
    clock: &Clock,
    cancel: &AtomicBool,
) -> Option<(DateTimeL, DateTimeL)> {
    defn!();
    let mut bytes: &[u8] = bytes;
    let mut boot = None;
    let mut bounds: Option<(DateTimeL, DateTimeL)> = None;
    let mut expected: usize = 0;
    while !bytes.is_empty() {
        if cancel.load(Ordering::Relaxed) {
            defx!("canceled; return None");
            return None;
        }
        let (tag, chunk, size) = frame(bytes)?;
        match tag {
            HEADER if expected == 0 => {
                let uuid: u128 = u128::from_be_bytes(field(chunk, 144)?);
                let uuid = encode_uuid_hex(uuid);
                let uuid = std::str::from_utf8(&uuid).ok()?;
                boot = clock.get(uuid);
            }
            CATALOG if expected == 0 => {
                let (count, (lo, hi)) = catalog_bounds(chunk, boot?)?;
                expected = count;
                bounds = Some(match bounds {
                    Some((a, b)) => (a.min(lo), b.max(hi)),
                    None => (lo, hi),
                });
            }
            CHUNKSET if expected != 0 => expected -= 1,
            _ => {
                defx!("unexpected tag: {tag:?}");
                return None;
            }
        }
        bytes = &bytes[size..];
    }
    let ret: Option<(DateTimeL, DateTimeL)> = if expected == 0 { bounds } else { None };
    defx!("return {ret:?}");

    ret
}
