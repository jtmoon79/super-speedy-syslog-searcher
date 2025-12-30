// src/tests/venv_tests.rs

//! tests for [`src/python/venv.rs`]
//!
//! [`src/python/venv.rs`]: crate::python::venv

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::io::ErrorKind;

#[allow(unused_imports)]
use ::si_trace_print::printers::{
    defn,
    defo,
    defx,
    defñ,
    def2n,
    def2x,
    def2ñ,
};

use crate::common::Bytes;
use crate::debug::printers::buffer_to_String_noraw;
use crate::python::venv::{
    create,
    deploy_pyproject_s4_event_readers,
    extract_compare_version,
    venv_path,
};

type VENV_LOCK_TYPE<'a> = std::sync::Mutex<()>;
/// Tests should call `venv_setup()` to ensure the virtual environment is created before using it.
static VENV_TESTS_LOCK: VENV_LOCK_TYPE<'static> = VENV_LOCK_TYPE::new(());
/// Indicates if the virtual environment has been created.
/// Protected by `VENV_TESTS_LOCK`.
static VENV_ENV_CREATED: std::sync::OnceLock<()> = std::sync::OnceLock::new();

/// setup the Python virtual environment for tests, do this once
pub fn venv_setup() {
    let _lock = VENV_TESTS_LOCK.lock().unwrap();
    if VENV_ENV_CREATED.get().is_none() {
        def2n!("creating venv…");
        let create_result = create();
        assert!(create_result.is_ok(), "venv creation failed in venv_setup()");
        VENV_ENV_CREATED.set(()).unwrap();
        def2x!("venv created.");
    } else {
        def2ñ!("venv already created.");
    }
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
