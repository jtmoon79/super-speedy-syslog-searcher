// src/tests/pyeventreader_tests.rs

//! tests for [`src/readers/pyeventreader.rs`]
//!
//! [`src/readers/pyeventreader.rs`]: crate::readers::pyeventreader

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

#[allow(unused_imports)]
use ::si_trace_print::printers::{
    defn,
    defo,
    defx,
};
use ::test_case::test_case;

use crate::common::{
    Count,
    FPath,
    FileSz,
    FileType,
    FileTypeArchive,
};
use crate::data::datetime::{
    DateTimeLOpt,
    ymdhmsl,
};
use crate::python::pyrunner::PipeSz;
use crate::readers::pyeventreader::{
    PyEventReader,
    ResultNextPyDataEvent,
};
use crate::tests::common::{
    ASL_1_EVENT_COUNT,
    ASL_1_FILESZ,
    ASL_1_FPATH,
    FO_0,
    path_id_generator,
};
use crate::tests::venv_tests::venv_setup;

const FILETYPE_ASL: FileType = FileType::Asl {
    archival_type: FileTypeArchive::Normal,
};
const PIPE_SZ_ASL: PipeSz = 1024;

#[test_case(
    ASL_1_FPATH.clone(),
    ASL_1_FILESZ,
    PIPE_SZ_ASL,
    FILETYPE_ASL;
    "asl"
)]
fn test_PyEventReader_new_asl(
    path: FPath,
    size_expected: FileSz,
    pipe_sz: PipeSz,
    filetype: FileType,
) {
    venv_setup();

    let path_id = path_id_generator();
    let per = PyEventReader::new(path_id, path.clone(), filetype, FO_0, pipe_sz).unwrap();
    defo!("per: {:?}", per);
    assert_eq!(per.path_id(), path_id);
    assert_eq!(per.filesz(), size_expected, "expected filesz {} for path {:?}", size_expected, &path);
    assert_eq!(per.filetype(), filetype);
    defo!("per.mtime(): {:?}", per.mtime());
    defo!("per.path(): {:?}", per.path());
    defo!("per.pipe_sz_stdout(): {:?}", per.pipe_sz_stdout());
    defo!("per.pipe_sz_stderr(): {:?}", per.pipe_sz_stderr());
    assert_eq!(per.pipe_sz_stdout(), pipe_sz);
    assert_eq!(per.pipe_sz_stderr(), pipe_sz);
}

#[test]
fn test_PyEventReader_ts_data_to_datetime_ok() {
    defn!();
    venv_setup();

    let per = PyEventReader::new(path_id_generator(), ASL_1_FPATH.clone(), FILETYPE_ASL, FO_0, 1).unwrap();

    let ts_data = b"1590429555554"; // 2020-05-25T17:59:15.554+00:00
    let dt_ts = per
        .ts_data_to_datetime(ts_data)
        .unwrap();
    defo!("dt_ts: {:?}", dt_ts);
    let dt_utc = ymdhmsl(&FO_0, 2020, 5, 25, 17, 59, 15, 554);
    defo!("dt_utc: {:?}", dt_utc);
    assert_eq!(dt_utc, dt_ts);

    defx!();
}

#[test]
fn test_PyEventReader_ts_data_to_datetime_none() {
    defn!();
    venv_setup();

    let per = PyEventReader::new(path_id_generator(), ASL_1_FPATH.clone(), FILETYPE_ASL, FO_0, 1).unwrap();

    let ts_data = b"-";
    let dt_ts = per.ts_data_to_datetime(ts_data);
    assert!(dt_ts.is_none());

    defx!();
}

#[test_case(
    ASL_1_FPATH.clone(),
    8,
    FILETYPE_ASL,
    &DateTimeLOpt::None,
    &DateTimeLOpt::None,
    ASL_1_EVENT_COUNT;
    "asl1 pipesz 8 events all"
)]
#[test_case(
    ASL_1_FPATH.clone(),
    2056,
    FILETYPE_ASL,
    &DateTimeLOpt::None,
    &DateTimeLOpt::None,
    ASL_1_EVENT_COUNT;
    "asl1 pipesz 2056 events all"
)]
#[test_case(
    ASL_1_FPATH.clone(),
    64,
    FILETYPE_ASL,
    // 2030-01-01 12:00:00.000+00:00
    &DateTimeLOpt::Some(ymdhmsl(&FO_0, 2030, 1, 1, 12, 0, 0, 0)),
    &DateTimeLOpt::None,
    0;
    "asl1 pipesz 64 events 0 after 2030-01-01T12:00:00.000"
)]
fn test_PyEventReader_next(
    path: FPath,
    pipe_sz: PipeSz,
    file_type: FileType,
    dt_filter_after: &DateTimeLOpt,
    dt_filter_before: &DateTimeLOpt,
    events_expected: Count,
) {
    defn!(
        "test_PyEventReader_next: path={:?}, pipe_sz={:?}, file_type={:?}, dt_filter_after={:?}, dt_filter_before={:?}, events_expected={}",
        path,
        pipe_sz,
        file_type,
        dt_filter_after,
        dt_filter_before,
        events_expected
    );

    venv_setup();

    let mut per = PyEventReader::new(path_id_generator(), path, file_type, FO_0, pipe_sz).unwrap();

    let mut count: Count = 0;
    loop {
        let pde_result = per.next(dt_filter_after, dt_filter_before);
        match pde_result {
            ResultNextPyDataEvent::Found(pde) => {
                count += 1;
                defo!("pde: {:?}, count is {}", pde, count);
            }
            ResultNextPyDataEvent::Done => {
                defo!("Done");
                break;
            }
            ResultNextPyDataEvent::Err(err) => {
                defo!("Err PyDataEvent: {}", err);
                break;
            }
            ResultNextPyDataEvent::ErrIgnore(err) => {
                defo!("ErrIgnore reading PyDataEvent: {}", err);
                break;
            }
        }
    }
    defo!("total PyDataEvents read: {}", count);
    assert_eq!(count, events_expected, "expected {} PyDataEvents", events_expected);

    defx!();
}
