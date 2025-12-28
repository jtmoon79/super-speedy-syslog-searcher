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
};

use crate::common::Bytes;
use crate::debug::printers::buffer_to_String_noraw;
use crate::python::venv::{
    create,
    deploy_pyproject_s4_event_readers,
    extract_compare_version,
    venv_path,
};


#[test]
fn test_venv_path() {
    let venv_path_result = venv_path();
    defñ!("venv_path() returned {:?}", venv_path_result);
}

#[test]
fn test_deploy_pyproject_s4_event_readers() {
    let temp_dir = deploy_pyproject_s4_event_readers();
    defñ!("deploy_result: {:?}", temp_dir);
    assert!(temp_dir.is_ok());
    let td = temp_dir.unwrap();
    defñ!("temp_dir path: {:?}", td.path());
    assert!(td.path().exists());
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
}

#[test]
fn test_create() {
    let create_result = create();
    defñ!("create_result: {:?}", create_result);
    assert!(create_result.is_ok(), "venv creation failed");
}
