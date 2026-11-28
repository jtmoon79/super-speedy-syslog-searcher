//! Decode and merge one captured Apple Unified Log set.

use std::collections::{
    BTreeMap,
    HashMap,
    HashSet,
};
use std::io::{
    self,
    Read,
    Error,
    ErrorKind,
};
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};

use ::macos_unifiedlogs::cache::MemoryStringCache;
use ::macos_unifiedlogs::iterator::UnifiedLogIterator;
use ::macos_unifiedlogs::timesync::TimesyncBoot;
use ::macos_unifiedlogs::traits::StringCache;
use ::macos_unifiedlogs::unified_log::{
    LogData,
    UnifiedLogCatalogData,
    UnifiedLogData,
};
use ::smallvec::SmallVec;
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
    AulDetection,
    Count,
    FPath,
    PathId,
    summary_stat,
};
use crate::data::aul::Aul;
use crate::data::common::DtBegEndPair;
use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    Result_Filter_DateTime2,
    dt_pass_filters,
};
use crate::readers::aulparser::{
    AulParser,
    Clock,
    calibrated,
    datetime,
    trace_batches,
    trace_bounds,
};
use crate::readers::filehandlemanager::{
    FILE_HANDLE_MANAGER,
    FileHandleRole,
    OpenOptionsManaged,
};
use crate::readers::helpers::path_to_fpath;

type Order = (DateTimeL, u64);
type ProcRef = (u64, u32, u32);
type OversizeKey = (Arc<str>, ProcRef);
type CacheType = MemoryStringCache<Arc<std::sync::RwLock<HashMap<String, Arc<macos_unifiedlogs::uuidtext::UUIDText>>>>, Arc<std::sync::RwLock<HashMap<String, Arc<macos_unifiedlogs::dsc::SharedCacheStrings>>>>>;
type ScanType<'a> = Scan<'a, MemoryStringCache<Arc<std::sync::RwLock<HashMap<String, Arc<macos_unifiedlogs::uuidtext::UUIDText>>>>, Arc<std::sync::RwLock<HashMap<String, Arc<macos_unifiedlogs::dsc::SharedCacheStrings>>>>>>;
type QueueMapType = BTreeMap<Order, Pending>;
type HeldOrders = SmallVec<[Order; 1]>;
type OversizeMapType = HashMap<OversizeKey, HeldOrders>;
type OversizeIndexMapType = HashMap<OversizeKey, usize>;

#[derive(Clone, Debug, Default)]
pub struct SummaryAulReader {
    pub files_opened: Count,
    pub records_processed: Count,
    pub records_accepted: Count,
    pub records_failed: Count,
    pub records_incomplete: Count,
    pub records_skipped_no_calibration: Count,
    pub out_of_order: Count,
    pub records_oversize_hold_max: Count,
    pub records_oversize_total: Count,
    pub records_oversize_hi: Count,
    pub records_oversize_unfound: Count,
    pub aulreader_warnings: Vec<(FPath, String)>,
    pub files_processed: Vec<FPath>,
    pub datetime_first_processed: DateTimeLOpt,
    pub datetime_last_processed: DateTimeLOpt,
    pub datetime_first_accepted: DateTimeLOpt,
    pub datetime_last_accepted: DateTimeLOpt,
}

pub struct AulReader {
    parser: AulParser,
    stats: SummaryAulReader,
}

impl AulReader {
    pub fn new(pathid: PathId, diagnostics: PathBuf, uuidtext: PathBuf) -> Self {
        def1ñ!("pathid={pathid:?}, diagnostics={diagnostics:?}, uuidtext={uuidtext:?}");
        let roots = [&diagnostics, &uuidtext].map(|root| root.canonicalize().unwrap_or_else(|_| root.clone()));
        Self::with_opener_and_detection(diagnostics, uuidtext, None, move |path| {
            let source = path.canonicalize()?;
            if !roots.iter().any(|root| source.starts_with(root)) || !source.is_file() {
                return Err(Error::new(ErrorKind::InvalidInput, "not a regular file inside the capture"));
            }
            let file = FILE_HANDLE_MANAGER.request_open_managed(
                pathid, FileHandleRole::PrimaryRead, &source, OpenOptionsManaged::read_only(),
            )?;
            Ok(Box::new(file))
        })
    }

    pub fn new_with_detection(
        pathid: PathId,
        diagnostics: PathBuf,
        uuidtext: PathBuf,
        detection: AulDetection,
    ) -> Self {
        def1ñ!("pathid={pathid:?}, diagnostics={diagnostics:?}, uuidtext={uuidtext:?}, detection={detection:?}");
        let roots = [&diagnostics, &uuidtext].map(|root| root.canonicalize().unwrap_or_else(|_| root.clone()));
        Self::with_opener_and_detection(diagnostics, uuidtext, Some(detection), move |path| {
            let source = path.canonicalize()?;
            if !roots.iter().any(|root| source.starts_with(root)) || !source.is_file() {
                return Err(Error::new(ErrorKind::InvalidInput, "not a regular file inside the capture"));
            }
            let file = FILE_HANDLE_MANAGER.request_open_managed(
                pathid, FileHandleRole::PrimaryRead, &source, OpenOptionsManaged::read_only(),
            )?;
            Ok(Box::new(file))
        })
    }

    pub fn with_opener(
        diagnostics: PathBuf,
        uuidtext: PathBuf,
        opener: impl Fn(&Path) -> io::Result<Box<dyn Read>> + 'static,
    ) -> Self {
        Self::with_opener_and_detection(diagnostics, uuidtext, None, opener)
    }

    fn with_opener_and_detection(
        diagnostics: PathBuf,
        uuidtext: PathBuf,
        detection: Option<AulDetection>,
        opener: impl Fn(&Path) -> io::Result<Box<dyn Read>> + 'static,
    ) -> Self {
        def1ñ!("diagnostics={diagnostics:?}, uuidtext={uuidtext:?}, detection={detection:?}");
        let mut stats = SummaryAulReader::default();
        if detection == Some(AulDetection::ByEnvironment) {
            summary_stat!(
                stats.aulreader_warnings.push((
                    path_to_fpath(&diagnostics),
                    "Apple Unified Logs layout was explicitly configured with S4_AUL_PATHS; standard trace layout was not detected".into(),
            )));
        } else if detection == Some(AulDetection::ByTrace) {
            summary_stat!(
                stats.aulreader_warnings.push((
                    path_to_fpath(&diagnostics),
                    "standard timesync companion layout was not detected".into(),
            )));
        }

        Self {
            parser: AulParser::new(diagnostics, uuidtext, Box::new(opener)),
            stats,
        }
    }

    pub fn into_stats(self) -> SummaryAulReader {
        let mut stats: SummaryAulReader = self.stats;
        let (parser_stats, warnings) = self.parser.into_stats();
        stats.files_opened = parser_stats.files_opened;
        stats.files_processed = parser_stats.files_processed;
        stats.aulreader_warnings.extend(warnings);

        stats
    }

    pub fn run(
        &mut self,
        after: &DateTimeLOpt,
        before: &DateTimeLOpt,
        cancel: &AtomicBool,
        mut send: impl FnMut(Aul) -> bool,
    ) -> io::Result<()> {
        def1n!("after={after:?}, before={before:?}, cancel={cancel:?}");
        let parser: &AulParser = &self.parser;
        let clock: HashMap<String, TimesyncBoot>  = parser.clocks(cancel);
        let paths = parser.trace_paths();
        let mut files = Vec::with_capacity(paths.len());
        for path in paths {
            if cancel.load(Ordering::Relaxed) {
                def1x!("return Ok; cancelled while iterating trace paths");
                return Ok(());
            }
            let bounds = parser.read(&path).ok().and_then(|bytes| {
                parser.scoped(&path, || trace_bounds(&bytes, &clock, cancel))
            });
            files.push((bounds, path));
        }
        files.sort_unstable();
        let cache: CacheType = MemoryStringCache::default();
        let queue: QueueMapType = BTreeMap::new();
        let held: OversizeMapType = HashMap::new();
        let payloads: UnifiedLogData = UnifiedLogData::default();
        let payload_indices: OversizeIndexMapType = HashMap::new();
        let mut scan: ScanType = Scan {
            parser,
            stats: &mut self.stats,
            clock: &clock,
            cache: &cache,
            after,
            before,
            usable: false,
            order: 0,
            admissions: 0,
            unresolved: 0,
            queue,
            held,
            payloads,
            payload_indices,
        };
        let mut files = files.into_iter().peekable();
        while let Some((_, path)) = files.next() {
            if cancel.load(Ordering::Relaxed) {
                def1x!("return Ok; cancelled while iterating trace paths");
                return Ok(());
            }
            scan.trace(&path, cancel);
            if let Some((Some((unread_min, _)), _)) = files.peek()
                && !scan.drain(Some(*unread_min), cancel, &mut send)
            {
                def1x!("return Ok; cancelled while draining scan");
                return Ok(());
            }
        }
        if cancel.load(Ordering::Relaxed) {
            def1x!("return Ok; cancelled before finishing holds");
            return Ok(());
        }
        scan.finish_holds();
        if !scan.drain(None, cancel, &mut send) {
            def1x!("return Ok; cancelled while draining final scan");
            return Ok(());
        }
        if !scan.usable {
            def1x!("return Err:no usable scan found");
            return Err(Error::new(ErrorKind::InvalidData, format!(
                "{}: no readable Apple Unified Log records with usable clock calibration",
                parser.diagnostics.display(),
            )));
        }

        Ok(())
    }
}

struct Pending {
    data: PendingData,
    held_at: Option<Count>,
}

enum PendingData {
    Event(Aul),
    Held(Box<Replay>),
}

struct Replay {
    path: Arc<Path>,
    context: Arc<UnifiedLogData>,
    record: UnifiedLogCatalogData,
}

fn serialized_timestamp_range(json: &[u8], timestamp: &str) -> Option<(usize, usize)> {
    let marker = b"\"timestamp\":\"";
    let beg = memchr::memmem::rfind(json, marker)?.checked_add(marker.len())?;
    let end = beg.checked_add(timestamp.len())?;
    (json.get(beg..end) == Some(timestamp.as_bytes())).then_some((beg, end))
}

struct Scan<'a, C> {
    parser: &'a AulParser,
    stats: &'a mut SummaryAulReader,
    clock: &'a Clock,
    cache: &'a C,
    after: &'a DateTimeLOpt,
    before: &'a DateTimeLOpt,
    usable: bool,
    order: u64,
    admissions: Count,
    unresolved: Count,
    queue: QueueMapType,
    held: OversizeMapType,
    payloads: UnifiedLogData,
    payload_indices: OversizeIndexMapType,
}

impl<C: StringCache> Scan<'_, C> {
    fn trace(&mut self, path: &Path, cancel: &AtomicBool) {
        def1n!("{path:?}, {cancel:?}");
        let Ok(bytes) = self.parser.read(path) else {
            defx!("failed parse {path:?}");
            return;
        };
        let evidence: String = path.to_string_lossy().into_owned();
        let path: Arc<Path> = Arc::from(path);
        let mut headers = Vec::new();
        for (fresh_header, batch) in trace_batches(&bytes) {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            let Some(batch) = batch else {
                self.parser.warn(&path, "truncated or malformed tracev3 frame");
                break;
            };
            if fresh_header {
                headers.clear();
            }
            let mut data: Vec<u8> = batch.to_vec();
            let Some(padded) = data.len().checked_next_multiple_of(8) else {
                self.parser.warn(&path, "tracev3 catalog batch is too large");
                continue;
            };
            data.resize(padded, 0);
            let mut iterator: UnifiedLogIterator = UnifiedLogIterator {
                data, header: headers, evidence: evidence.clone()
            };
            let Some(mut chunk) = self.parser.scoped(&path, || iterator.next()) else {
                self.parser.warn(&path, "malformed tracev3 catalog batch");
                headers = iterator.header;
                continue;
            };
            headers = iterator.header;
            let Some(header) = chunk.header.first() else {
                self.parser.warn(&path, "missing or malformed tracev3 header");
                continue;
            };
            let boot: Arc<str> = Arc::from(header.boot_uuid.as_str());
            for payload in std::mem::take(&mut chunk.oversize) {
                let key: OversizeKey = (boot.clone(), (payload.first_proc_id, payload.second_proc_id, payload.data_ref_index));
                if let Some(orders) = self.held.remove(&key) {
                    for order in orders {
                        summary_stat!(self.unresolved = self.unresolved.saturating_sub(1));
                        let Some(pending) = self.queue.remove(&order) else {
                            self.failed(&path, "missing deferred oversize reconstruction");
                            continue;
                        };
                        let record = match pending.data {
                            PendingData::Held(record) => record,
                            data => {
                                self.queue.insert(order, Pending { data, held_at: pending.held_at });
                                self.failed(&path, "inconsistent deferred oversize reconstruction");
                                continue;
                            }
                        };
                        let input = UnifiedLogData { oversize: vec![payload.clone()], ..Default::default() };
                        if let Some(event) = self.replay(*record, order.0, input) {
                            self.queue.insert(order, Pending { data: PendingData::Event(event), held_at: pending.held_at });
                        }
                    }
                }
                self.payload_indices.entry(key).or_insert_with(|| {
                    let index: usize = self.payloads.oversize.len();
                    self.payloads.oversize.push(payload);

                    index
                });
            }
            for catalog in std::mem::take(&mut chunk.catalog_data) {
                self.catalog(&path, &boot, &chunk, catalog);
            }
        }
        def1x!();
    }

    fn accepted_time(&mut self, path: &Path, boot: &str, tick: Option<u64>, base: u64, known: bool) -> Option<DateTimeL> {
        def1n!("path={path:?}, boot={boot}, tick={tick:?}, base={base}, known={known}");
        summary_stat!(self.stats.records_processed += 1);
        let Some(tick) = tick else {
            self.failed(path, "firehose continuous timestamp overflow");
            defx!("return None");
            return None;
        };
        if !calibrated(self.clock.get(boot), base) {
            if known {
                self.parser.warn(path, "event has no usable timesync calibration");
                summary_stat!(self.stats.records_skipped_no_calibration += 1);
            } else {
                self.failed(path, "unrecognized firehose record type");
            }
            def1x!("return None");
            return None;
        }
        let Some(dt) = datetime(TimesyncBoot::get_timestamp(self.clock, boot, tick, base)) else {
            self.failed(path, "calibrated timestamp out of range");
            def1x!("return None");
            return None;
        };
        self.usable |= known;
        summary_stat!({
            self.stats.datetime_first_processed = Some(self.stats.datetime_first_processed.map_or(dt, |first| first.min(dt)));
            self.stats.datetime_last_processed = Some(self.stats.datetime_last_processed.map_or(dt, |last| last.max(dt)));
        });
        if !matches!(dt_pass_filters(&dt, self.after, self.before), Result_Filter_DateTime2::InRange) {
            def1x!("return None; {dt:?} not within datetime filters (after={:?}, before={:?})", self.after, self.before);
            return None;
        }
        if !known {
            self.failed(path, "unrecognized firehose record type");
            def1x!("return None");
            return None;
        }
        def1x!("return Some({dt:?})");

        Some(dt)
    }

    fn next_order(&mut self, dt: DateTimeL) -> Option<Order> {
        self.order = self.order.checked_add(1)?;
        Some((dt, self.order))
    }

    fn catalog(
        &mut self,
        path: &Arc<Path>,
        boot: &Arc<str>,
        chunk: &UnifiedLogData,
        mut catalog: UnifiedLogCatalogData,
    ) {
        def1n!("path={path:?}, boot={boot:?}, chunk, catalog ({}, {}, {}, {}) elements",
            catalog.firehose.len(),
            catalog.simpledump.len(),
            catalog.statedump.len(),
            catalog.oversize.len()
        );
        let tag_capacity = catalog.firehose.iter()
            .map(|preamble| preamble.public_data.len())
            .sum::<usize>() + catalog.simpledump.len() + catalog.statedump.len();
        let mut tags = Vec::with_capacity(tag_capacity);
        let mut payloads = Vec::new();
        let mut matched = HashSet::new();
        let mut replay_context = None;
        for preamble in &mut catalog.firehose {
            let base = preamble.base_continous_time;
            let mut template = None;
            let records = std::mem::take(&mut preamble.public_data);
            for mut record in records {
                let kind: u8 = record.log_activity_type;
                let known: bool = matches!(kind, 0x2 | 0x3 | 0x4 | 0x6 | 0x7);
                let delta: u64 = u64::from(record.continous_time_delta) | (u64::from(record.continous_time_delta_upper) << 32);
                let Some(dt) = self.accepted_time(path, boot, base.checked_add(delta), base, known) else { continue };
                if kind == 0x6 {
                    // 0.7.0 reads the non-activity slot when reconstructing signpost oversize data.
                    record.firehose_non_activity.data_ref_value = record.firehose_signpost.data_ref_value;
                }
                let Some(order) = self.next_order(dt) else {
                    self.failed(path, "event ordering counter overflow");
                    continue;
                };
                let reference: u32 = record.firehose_non_activity.data_ref_value;
                if matches!(kind, 0x4 | 0x6) && reference != 0 {
                    let proc_ref: (u64, u32, u32) = (preamble.first_number_proc_id, preamble.second_number_proc_id, reference);
                    let key: (Arc<str>, (u64, u32, u32)) = (boot.clone(), proc_ref);
                    if let Some(payload) = self.payload_indices.get(&key)
                        .and_then(|index| self.payloads.oversize.get(*index))
                    {
                        if matched.insert(proc_ref) {
                            payloads.push(payload.clone());
                        }
                    } else {
                        let context = replay_context.get_or_insert_with(|| {
                            Arc::new(UnifiedLogData {
                                header: chunk.header.clone(),
                                catalog_data: vec![UnifiedLogCatalogData {
                                    catalog: catalog.catalog.clone(), ..Default::default()
                                }],
                                evidence: chunk.evidence.clone(),
                                ..Default::default()
                            })
                        });
                        let mut replay = template.get_or_insert_with(|| {
                            let mut template = preamble.clone();
                            template.public_data = Vec::new();
                            template
                        }).clone();
                        replay.public_data.push(record);
                        let record = Box::new(Replay {
                            path: path.clone(), context: context.clone(),
                            record: UnifiedLogCatalogData { firehose: vec![replay], ..Default::default() },
                        });
                        self.held.entry(key).or_default().push(order);
                        self.queue.insert(order, Pending { data: PendingData::Held(record), held_at: None });
                        summary_stat!(tags.push((order, None)));
                        summary_stat!({
                            self.unresolved += 1;
                            self.stats.records_oversize_total += 1;
                            self.stats.records_oversize_hi = self.stats.records_oversize_hi.max(self.unresolved);
                        });
                        continue;
                    }
                }
                tags.push((order, Some(kind)));
                preamble.public_data.push(record);
            }
        }
        catalog.simpledump.retain(|record| {
            self.accepted_time(path, boot, Some(record.continous_time), 1, true).is_some_and(|dt| {
                let Some(order) = self.next_order(dt) else {
                    self.failed(path, "event ordering counter overflow");
                    return false;
                };
                tags.push((order, Some(0)));
                true
            })
        });
        catalog.statedump.retain(|record| {
            self.accepted_time(path, boot, Some(record.continuous_time), 1, true).is_some_and(|dt| {
                let Some(order) = self.next_order(dt) else {
                    self.failed(path, "event ordering counter overflow");
                    return false;
                };
                tags.push((order, Some(0)));
                true
            })
        });
        let expected = tags.iter().filter(|(_, kind)| kind.is_some()).count();
        let mut decoded = (expected != 0).then(|| {
            let chunk = UnifiedLogData {
                header: chunk.header.clone(), catalog_data: vec![catalog],
                oversize: payloads, evidence: chunk.evidence.clone(),
            };
            self.build(path, &chunk).into_iter()
        }).filter(|decoded| decoded.len() == expected);
        // Upstream emits items in catalog order. Count only successful queue admissions in that order.
        for (order, kind) in tags {
            let Some(kind) = kind else {
                let Some(pending) = self.queue.get_mut(&order) else {
                    self.failed(path, "missing deferred oversize queue entry");
                    continue;
                };
                if !matches!(&pending.data, PendingData::Held(_)) {
                    self.failed(path, "inconsistent deferred oversize queue entry");
                    continue;
                }
                summary_stat!(self.admissions += 1);
                pending.held_at = Some(self.admissions);
                continue;
            };
            let Some(record) = decoded.as_mut().and_then(Iterator::next) else {
                self.failed(path, "unexpected upstream reconstruction result count");
                continue;
            };
            if let Some(event) = self.event(path, order.0, record, kind, false) {
                self.queue.insert(order, Pending { data: PendingData::Event(event), held_at: None });
                summary_stat!(self.admissions += 1);
            }
        }

        def1x!();
    }

    fn build(&self, path: &Path, chunk: &UnifiedLogData) -> Vec<LogData> {
        self.parser.scoped(path, || LogData::build_log(chunk, self.parser, self.cache, self.clock, false).0)
    }

    fn failed(&mut self, path: &Path, reason: &str) {
        self.parser.warn(path, reason);
        summary_stat!(self.stats.records_failed += 1);
    }

    fn event(&mut self, path: &Path, dt: DateTimeL, record: LogData, kind: u8, unmatched: bool) -> Option<Aul> {
        defn!("path={path:?}, dt={dt:?}, record, kind={kind}, unmatched={unmatched}");
        if datetime(record.time) != Some(dt) {
            self.failed(path, "reconstructed event timestamp does not match its queue order");
            return None;
        }
        let json = match serde_json::to_vec(&record) {
            Ok(json) => json,
            Err(_) => {
                self.failed(path, "unable to serialize reconstructed event");
                return None;
            }
        };
        summary_stat!({
            // These are upstream format-lookup status values, not matches on the rendered log message.
            let raw = &record.raw_message;
            let failed_format = matches!(raw.as_str(), "Error: Invalid UUID index" | "Error: Invalid shared string offset" | "Unknown shared string message")
                || raw.starts_with("Failed to get string message from ")
                || raw.starts_with("Error: Invalid offset ") && raw.contains(" UUID ")
                || matches!(kind, 0x2 | 0x3 | 0x4 | 0x6) && record.library_uuid.is_empty() && record.process_uuid.is_empty()
                    && record.library.is_empty() && record.process.is_empty();
            // Upstream trace arguments leave `item` at its default even for successfully parsed numbers.
            let unknown_argument = kind != 0x3 && record.message_entries.iter().any(|entry| entry.item == Default::default());
            if unmatched || failed_format || unknown_argument {
                self.stats.records_incomplete += 1;
                self.parser.warn(path, "event reconstruction is incomplete");
            }
        });
        // JSON escapes quotes inside values, so this marker identifies the timestamp field.
        let Some((beg, end)) = serialized_timestamp_range(&json, &record.timestamp) else {
            self.failed(path, "serialized event timestamp field is invalid");
            return None;
        };
        defx!("Some(Aul::new({dt:?}, DtBegEndPair {{ {beg}, {end} }}, json {} bytes))", json.len());

        Some(Aul::new(dt, DtBegEndPair { beg, end }, json))
    }

    fn replay(&mut self, mut record: Replay, dt: DateTimeL, mut chunk: UnifiedLogData) -> Option<Aul> {
        defñ!("record, dt={dt:?}, record, chunk");
        let unmatched = chunk.oversize.is_empty();
        let Some(kind) = record.record.firehose.first()
            .and_then(|firehose| firehose.public_data.first())
            .map(|record| record.log_activity_type)
        else {
            self.failed(&record.path, "deferred event is missing its firehose record");
            return None;
        };
        chunk.header = record.context.header.clone();
        chunk.evidence = record.context.evidence.clone();
        let Some(context) = record.context.catalog_data.first() else {
            self.failed(&record.path, "deferred event is missing its catalog context");
            return None;
        };
        record.record.catalog = context.catalog.clone();
        chunk.catalog_data = vec![record.record];
        let mut decoded = self.build(&record.path, &chunk);
        if decoded.len() == 1 {
            let Some(decoded) = decoded.pop() else {
                self.failed(&record.path, "unable to reconstruct deferred event");
                return None;
            };
            self.event(&record.path, dt, decoded, kind, unmatched)
        } else {
            self.failed(&record.path, "unable to reconstruct deferred event");
            None
        }
    }

    fn finish_holds(&mut self) {
        summary_stat!({
            self.stats.records_oversize_unfound += self.unresolved;
            self.unresolved = 0;
            for order in self.held.values().flatten() {
                let Some(pending) = self.queue.get(order) else {
                    self.parser.warn(
                        &self.parser.diagnostics,
                        "unmatched oversize record is missing its queue entry",
                    );
                    continue;
                };
                if let PendingData::Held(record) = &pending.data {
                    self.parser.warn(&record.path, "unmatched oversize record reference");
                } else {
                    self.parser.warn(
                        &self.parser.diagnostics,
                        "unmatched oversize queue entry has inconsistent state",
                    );
                }
            }
        });
        self.held.clear();
        self.payload_indices.clear();
        self.payloads.oversize.clear();
    }

    fn drain(&mut self, cutoff: DateTimeLOpt, cancel: &AtomicBool, send: &mut impl FnMut(Aul) -> bool) -> bool {
        defn!("cutoff={cutoff:?}, cancel={cancel:?}, send");
        while let Some(((dt, _), pending)) = self.queue.first_key_value() {
            if cancel.load(Ordering::Relaxed) {
                defx!("return false; cancelled");
                return false;
            }
            if cutoff.is_some_and(|unread_min| *dt >= unread_min || matches!(pending.data, PendingData::Held(_))) {
                break;
            }
            let Some(((dt, _), pending)) = self.queue.pop_first() else {
                self.parser.warn(
                    &self.parser.diagnostics,
                    "ordered event queue entry disappeared while draining",
                );
                break;
            };
            let event = match pending.data {
                PendingData::Event(event) => event,
                // Render unmatched placeholders at their queue position instead of retaining a second batch.
                PendingData::Held(record) => {
                    let Some(event) = self.replay(*record, dt, UnifiedLogData::default()) else { continue };
                    event
                }
            };
            if !send(event) {
                defx!("return false; send returned false");
                return false;
            }
            summary_stat!({
                if self.stats.datetime_last_accepted.is_some_and(|last| dt < last) {
                    self.stats.out_of_order += 1;
                }
                self.stats.datetime_first_accepted.get_or_insert(dt);
                self.stats.datetime_last_accepted = Some(dt);
                self.stats.records_accepted += 1;
                if let Some(held_at) = pending.held_at {
                    let held = self.admissions.saturating_sub(held_at);
                    self.stats.records_oversize_hold_max = self.stats.records_oversize_hold_max.max(held);
                }
            });
        }
        let ret = !cancel.load(Ordering::Relaxed);
        defx!("return {ret}; drain complete");

        ret
    }
}
