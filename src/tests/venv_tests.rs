// src/tests/venv_tests.rs

//! tests for [`src/python/venv.rs`]
//!
//! [`src/python/venv.rs`]: crate::python::venv

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::io::ErrorKind;
use std::path::PathBuf;

#[allow(unused_imports)]
use ::si_trace_print::printers::{
    defn,
    defo,
    defx,
    defñ,
    def2o,
    def2n,
    def2x,
    def2ñ,
};
use ::tempfile::env::temp_dir;

use crate::common::{
    Bytes,
    threadid_to_u64,
};
use crate::debug::printers::buffer_to_String_noraw;
use crate::python::venv::{
    create,
    deploy_pyproject_s4_event_readers,
    extract_compare_version,
    venv_path,
};
use crate::tests::common::touch;

/// setup the Python virtual environment for tests, do this once per parent process (once per nextest run)
pub fn venv_setup() {
    let tid = threadid_to_u64(std::thread::current().id());
    let pid = std::process::id();
    #[cfg(target_family = "unix")]
    let ppid: u32 = std::os::unix::process::parent_id();
    #[cfg(not(target_family = "unix"))]
    let ppid: u32 = 0;
    let ppid_s = ppid.to_string();

    def2n!("TID {tid}, PID {pid}, PPID {ppid_s}…");

    let venv_pmutex: PathBuf = temp_dir().join("tmp-s4-test-python-venv-mutex");

    _ = std::fs::create_dir_all(&venv_pmutex);
    if ! venv_pmutex.exists() {
        panic!("path {:?} does not exist after create_dir_all()", venv_pmutex);
    }

    // XXX: hacky but functional method to coordinate multi-process test runs.
    //      If touch fails then another process controlled by the same parent process
    //      has already created the "pmutex" file.
    //      This is pretty close to a multi-platform inter-process mutex lock.
    //      This is to workaround nextest creating multiple processes for tests
    //      but we only want one creation of the Python venv per nextest run.
    // XXX: I tried using crate `process-sync` and `SharedMutex` but `Send` is not implemented for
    //      `SharedMutex` so it does not supported multi-threaded coordination.
    let venv_pmutex_claim: PathBuf = venv_pmutex.join(format!("venv_setup-claim-{}", ppid_s));
    let venv_pmutex_ready: PathBuf = venv_pmutex.join(format!("venv_setup-ready-{}", ppid_s));
    if touch(&venv_pmutex_claim.as_path()).is_err() {
        def2o!("Claim pmutex file already exists: {:?}", venv_pmutex_claim);
        // XXX: polling sleep
        while !venv_pmutex_ready.exists() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        def2x!("Ready pmutex file exists: {:?}", venv_pmutex_ready);
        return;
    }
    def2o!("touched Claim pmutex file {:?}", venv_pmutex_claim);

    def2o!("creating venv (TID {}, PID {}, PPID {})…", tid, pid, ppid_s);
    let create_result = create();
    assert!(create_result.is_ok(), "venv creation failed in venv_setup()");

    if let Err(err) = touch(&venv_pmutex_ready.as_path()) {
        panic!("failed to touch Ready pmutex file {:?} in venv_setup(); {}", venv_pmutex_ready, err);
    }
    def2o!("touched Ready pmutex file {:?}", venv_pmutex_ready);

    def2x!("venv created (TID {}, PID {}, PPID {}).", tid, pid, ppid_s);
}

#[test]
fn test_venv_path() {
    defn!();
    venv_setup();
    let venv_path_result = venv_path();
    defx!("venv_path() returned {:?}", venv_path_result);
}

#[test]
fn test_deploy_pyproject_s4_event_readers() {
    defn!();
    let temp_dir = deploy_pyproject_s4_event_readers();
    defo!("deploy_result: {:?}", temp_dir);
    assert!(temp_dir.is_ok());
    let td = temp_dir.unwrap();
    defo!("temp_dir path: {:?}", td.path());
    assert!(td.path().exists());
    defx!();
}

#[test]
fn test_extract_compare_version() {
    let data_v = Bytes::from(b"Python 3.12.0 (main, Oct  3 2023, 13:59:11) [MSC v.1934 64 bit (AMD64)] on win32\n");
    defo!("data_v: {:?}", buffer_to_String_noraw(&data_v));
    let version_opt = extract_compare_version(&data_v);
    defo!("version_opt: {:?}", version_opt);
    assert!(version_opt.is_ok());

    let data_v = Bytes::from(b"Python 3.8.10\n");
    defo!("data_v: {:?}", buffer_to_String_noraw(&data_v));
    let version_opt = extract_compare_version(&data_v);
    defo!("version_opt: {:?}", version_opt);
    assert!(version_opt.is_err());
    assert!(version_opt.err().unwrap().kind() == ErrorKind::Unsupported);

    let data_v = Bytes::from(b"");
    defo!("data_v: {:?}", buffer_to_String_noraw(&data_v));
    let version_opt = extract_compare_version(&data_v);
    defo!("version_opt: {:?}", version_opt);
    assert!(version_opt.is_err());
    assert!(version_opt.err().unwrap().kind() == ErrorKind::Other);
}

// XXX: no `test_create` to test `create()` because other tests already call `venv_setup()` which calls `create()`
//      and it would be too complicated to reset the state for testing `create()` again
