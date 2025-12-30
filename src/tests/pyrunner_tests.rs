// src/tests/pyrunner_tests.rs

//! tests for [`src/readers/pyrunner_tests.rs`]
//!
//! [`src/readers/pyrunner.rs`]: crate::readers::pyrunner

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::env;
use std::path::PathBuf;
use std::time::Duration;

#[allow(unused_imports)]
use ::si_trace_print::printers::{
    defn,
    defo,
    defx,
};
use ::si_trace_print::stack::stack_offset_set;
use ::tempfile;
use ::test_case::test_case;

use crate::common::{
    Bytes,
    FPath,
};
use crate::debug::printers::buffer_to_String_noraw;
use crate::python::pyrunner::{
    ChunkDelimiter,
    PipeSz,
    PyRunner,
    PythonToUse,
    find_python_executable,
    RECV_TIMEOUT,
};
use crate::tests::venv_tests::venv_setup;

fn swap_bytes(data: &mut Option<Bytes>, old: &str, new: &str) -> Bytes {
    let e_s = data.as_ref().unwrap().clone();
    let mut e_s_s: String = String::from_utf8_lossy(&e_s).to_string();
    e_s_s = e_s_s.replace(old, new);

    Bytes::from(e_s_s.as_bytes())
}

#[test_case(
    1,
    RECV_TIMEOUT,
    vec![
        "-c",
        r"print('Hello', end='\n'); print('World', end='\n'); print('Goodbye!', end='\n')",
    ],
    b"Hello\nWorld\nGoodbye!\n".to_vec(),
    Bytes::with_capacity(0);
    "pipsz 1"
)]
#[test_case(
    2,
    RECV_TIMEOUT,
    vec![
        "-c",
        r"print('Hello', end='\n'); print('World', end='\n'); print('Goodbye!', end='\n')",
    ],
    b"Hello\nWorld\nGoodbye!\n".to_vec(),
    Bytes::with_capacity(0);
    "pipsz 2"
)]
#[test_case(
    20,
    RECV_TIMEOUT,
    vec![
        "-c",
        r"print('Hello', end='\n'); print('World', end='\n'); print('Goodbye!', end='\n')",
    ],
    b"Hello\nWorld\nGoodbye!\n".to_vec(),
    Bytes::with_capacity(0);
    "pipsz 20"
)]
#[test_case(
    21,
    RECV_TIMEOUT,
    vec![
        "-c",
        r"print('Hello', end='\n'); print('World', end='\n'); print('Goodbye!', end='\n')",
    ],
    b"Hello\nWorld\nGoodbye!\n".to_vec(),
    Bytes::with_capacity(0);
    "pipsz 21"
)]
#[test_case(
    22,
    RECV_TIMEOUT,
    vec![
        "-c",
        r"print('Hello', end='\n'); print('World', end='\n'); print('Goodbye!', end='\n')",
    ],
    b"Hello\nWorld\nGoodbye!\n".to_vec(),
    Bytes::with_capacity(0);
    "pipsz 22"
)]
#[test_case(
    23,
    RECV_TIMEOUT,
    vec![
        "-c",
        r"print('Hello', end='\n'); print('World', end='\n'); print('Goodbye!', end='\n')",
    ],
    b"Hello\nWorld\nGoodbye!\n".to_vec(),
    Bytes::with_capacity(0);
    "pipsz 23"
)]
#[test_case(
    1,
    RECV_TIMEOUT,
    vec![
        "-c",
        r#"
import sys

print("Hello", end="\n")
print("STDERR1", end="\n", file=sys.stderr)
print("World", end="\n")
print("STDERR2", end="\n", file=sys.stderr)
print("Goodbye!", end="\n")
print("STDERR3", end="\n", file=sys.stderr)
"#,
    ],
    b"Hello\nWorld\nGoodbye!\n".to_vec(),
    b"STDERR1\nSTDERR2\nSTDERR3\n".to_vec();
    "pipsz 1 with stderr"
)]
#[test_case(
    20,
    RECV_TIMEOUT,
    vec![
        "-c",
        r#"
import sys

print("Hello", end="\n")
print("STDERR1", end="\n", file=sys.stderr)
print("World", end="\n")
print("STDERR2", end="\n", file=sys.stderr)
print("Goodbye!", end="\n")
print("STDERR3", end="\n", file=sys.stderr)
"#,
    ],
    b"Hello\nWorld\nGoodbye!\n".to_vec(),
    b"STDERR1\nSTDERR2\nSTDERR3\n".to_vec();
    "pipsz 20 with stderr"
)]
fn test_PyRunner_new_run_run_once(
    pipe_sz: PipeSz,
    recv_timeout: Duration,
    cmd_args: Vec<&str>,
    expect_stdout: Bytes,
    expect_stderr: Bytes,
) {
    defn!("test_PyRunner_new_run_run_once: pipe_sz={:?}, cmd_args[0]={:?}\ncmd_args[1]={}\n",
        pipe_sz, cmd_args[0], cmd_args[1]);

    venv_setup();

    let python_path = find_python_executable(PythonToUse::Path)
        .as_ref().expect("failed to find python executable in the PATH");

    // try with `new()` and `run()`
    let mut pyr = PyRunner::new(
        PythonToUse::Value,
        pipe_sz,
        recv_timeout,
        Some(b'\n'),
        None,
        Some(python_path.clone()),
        cmd_args.clone(),
    ).unwrap();
    let result = pyr.run(false, false, false);
    assert!(result.is_ok(), "PyRunner run failed: {:?}", result);
    let (stdout_bytes, stderr_bytes) = result.unwrap();

    defo!("PyRunner stdout: {}", buffer_to_String_noraw(&stdout_bytes));
    defo!("PyRunner stderr: {}", buffer_to_String_noraw(&stderr_bytes));

    assert_eq!(stdout_bytes, expect_stdout, "PyRunner stdout mismatch");
    assert_eq!(stderr_bytes, expect_stderr, "PyRunner stderr mismatch");

    assert!(pyr.exited(), "PyRunner did not exit after run()");
    assert!(pyr.exited_exhausted(), "PyRunner not exhausted after run()");

    // try with `run_once()`
    let result = PyRunner::run_once(
        PythonToUse::Value,
        pipe_sz,
        recv_timeout,
        b'\n',
        Some(python_path.clone()),
        cmd_args,
        false,
    );
    assert!(result.is_ok(), "PyRunner run_once failed: {:?}", result);
    let (pyr, stdout_bytes, stderr_bytes) = result.unwrap();

    defo!("PyRunner stdout: {}", buffer_to_String_noraw(&stdout_bytes));
    defo!("PyRunner stderr: {}", buffer_to_String_noraw(&stderr_bytes));

    assert_eq!(stdout_bytes, expect_stdout, "PyRunner stdout mismatch");
    assert_eq!(stderr_bytes, expect_stderr, "PyRunner stderr mismatch");

    assert!(pyr.exited(), "PyRunner did not exit after run_once()");
    assert!(pyr.exited_exhausted(), "PyRunner not exhausted after run_once()");

    defx!();
}

const PYTHON_SRC_LOOP_PRINT_WORLD_GOODBYE: &str = r#"
import time
for i in range([LOOPS]):
    print("Hello", end="\n")
    time.sleep((i % 10) * 0.001)
    print("World", end="\n")
    print(f"Goodbye! {i}", end="\n")
    print("\0", end="")
    time.sleep((i % 10) * 0.001)
"#;
const PYTHON_SRC_LOOPS_KEYWORD: &str = "[LOOPS]";
const PYTHON_SRC_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_OUTPUT: &[u8] = b"Hello\nWorld\nGoodbye! [LOOP]\n\0";
const PYTHON_SRC_LOOP_KEYWORD: &str = "[LOOP]";

#[test_case(
    1, // loops
    1, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_LOOPS_KEYWORD, "1").as_str()
    ],
    Some(PYTHON_SRC_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_OUTPUT.to_vec()), // expect_stdout
    None, // expect_stderr
    b'\0'; // chunk_delimiter stdout
    "loops 1 pipsz 1"
)]
#[test_case(
    2, // loops
    1, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_LOOPS_KEYWORD, "2").as_str()
    ],
    Some(PYTHON_SRC_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_OUTPUT.to_vec()), // expect_stdout
    None, // expect_stderr
    b'\0'; // chunk_delimiter stdout
    "loops 2 pipsz 1"
)]
#[test_case(
    200, // loops
    1, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_LOOPS_KEYWORD, "200").as_str()
    ],
    Some(PYTHON_SRC_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_OUTPUT.to_vec()), // expect_stdout
    None, // expect_stderr
    b'\0'; // chunk_delimiter stdout
    "loops 200 pipsz 1"
)]
#[test_case(
    10, // loops
    2, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_LOOPS_KEYWORD, "10").as_str()
    ],
    Some(PYTHON_SRC_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_OUTPUT.to_vec()), // expect_stdout
    None, // expect_stderr
    b'\0'; // chunk_delimiter stdout
    "loops 10 pipsz 2"
)]
#[test_case(
    10, // loops
    2, // pipe_sz
    Duration::from_millis(50),
    vec![
        "-c",
        PYTHON_SRC_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_LOOPS_KEYWORD, "10").as_str()
    ],
    Some(PYTHON_SRC_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_OUTPUT.to_vec()), // expect_stdout
    None, // expect_stderr
    b'\0'; // chunk_delimiter stdout
    "loops 10 pipsz 2 recv_timeout 50ms"
)]
#[test_case(
    100, // loops
    30, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_LOOPS_KEYWORD, "10").as_str()
    ],
    Some(PYTHON_SRC_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_OUTPUT.to_vec()), // expect_stdout
    None, // expect_stderr
    b'\0'; // chunk_delimiter stdout
    "loops 100 pipsz 30"
)]
#[test_case(
    100, // loops
    90, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_LOOPS_KEYWORD, "100").as_str()
    ],
    Some(PYTHON_SRC_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_OUTPUT.to_vec()), // expect_stdout
    None, // expect_stderr
    b'\0'; // chunk_delimiter stdout
    "loops 100 pipsz 90"
)]
#[test_case(
    100, // loops
    90, // pipe_sz
    Duration::from_millis(100),
    vec![
        "-c",
        PYTHON_SRC_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_LOOPS_KEYWORD, "100").as_str()
    ],
    Some(PYTHON_SRC_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_OUTPUT.to_vec()), // expect_stdout
    None, // expect_stderr
    b'\0'; // chunk_delimiter stdout
    "loops 100 pipsz 90 recv_timeout 100ms"
)]
#[test_case(
    100, // loops
    90, // pipe_sz
    Duration::from_millis(200),
    vec![
        "-c",
        PYTHON_SRC_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_LOOPS_KEYWORD, "100").as_str()
    ],
    Some(PYTHON_SRC_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_OUTPUT.to_vec()), // expect_stdout
    None, // expect_stderr
    b'\0'; // chunk_delimiter stdout
    "loops 100 pipsz 90 recv_timeout 200ms"
)]
#[test_case(
    100, // loops
    5092, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_LOOPS_KEYWORD, "100").as_str()
    ],
    Some(PYTHON_SRC_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_OUTPUT.to_vec()), // expect_stdout
    None, // expect_stderr
    b'\0'; // chunk_delimiter stdout
    "loops 100 pipsz 5092"
)]
fn test_PyRunner_stdout_run_many_times(
    loops: usize,
    pipe_sz: PipeSz,
    recv_timeout: Duration,
    cmd_args: Vec<&str>,
    expect_stdout: Option<Bytes>,
    expect_stderr: Option<Bytes>,
    chunk_delimiter: ChunkDelimiter,
) {
    stack_offset_set(Some(2));
    defn!("test_PyRunner_run_many_times: loops={}, pipe_sz={:?}, chunk_delimiter={:?}\ncmd_args[0]={:?}\ncmd_args[1]={}\n",
        loops, pipe_sz, chunk_delimiter, cmd_args[0], cmd_args[1]);

    venv_setup();

    let python_path = find_python_executable(PythonToUse::Path)
        .as_ref().expect("failed to find python executable in the PATH");

    let result = PyRunner::new(
        PythonToUse::Value,
        pipe_sz,
        recv_timeout,
        Some(chunk_delimiter),
        None,
        Some(python_path.clone()),
        cmd_args,
    );
    assert!(result.is_ok(), "PyRunner new failed: {:?}", result);
    let mut pyr = result.unwrap();

    // clone to re-declare as mutable
    let mut expect_stdout = expect_stdout.clone();

    let mut loop_: usize = 0;
    for i in 0..loops {
        defo!("PyRunner {i} write_read(None)",);
        let (exited, stdout_opt, stderr_opt) = pyr.write_read(None);
        defo!("PyRunner {i} exited? {exited}, stdout bytes {:?}, stderr bytes {:?}",
            stdout_opt.as_ref().map_or(0, |b| b.len()), stderr_opt.as_ref().map_or(0, |b| b.len()));
        match stdout_opt {
            Some(ref stdout) => {
                defo!("PyRunner {i} got stdout: '{}'", buffer_to_String_noraw(stdout));
                assert!(expect_stdout.is_some(), "PyRunner loop {i} got stdout but None expected");
                let e_s = swap_bytes(&mut expect_stdout, PYTHON_SRC_LOOP_KEYWORD, loop_.to_string().as_str());
                assert_eq!(
                    stdout, &e_s,
                    "PyRunner {i} stdout mismatch on loop {loop_}; got {}, expected {}",
                    buffer_to_String_noraw(stdout), buffer_to_String_noraw(&e_s));
                loop_ += 1;
            }
            None => defo!("PyRunner {i} stdout: None"),
        }
        match stderr_opt {
            Some(ref stderr) => {
                defo!("PyRunner {i} got stderr: '{}'", buffer_to_String_noraw(stderr));
                assert!(expect_stderr.is_some(), "PyRunner {i} got stderr but None expected");
                let e_s = expect_stderr.as_ref().unwrap();
                assert_eq!(
                    stderr, e_s,
                    "PyRunner {i} stderr mismatch on loop {loop_}; got {}, expected {}",
                    buffer_to_String_noraw(stderr), buffer_to_String_noraw(&e_s));
            }
            None => defo!("PyRunner {i} stderr: None"),
        }
        print!("."); // progress indicator
    }

    defx!();
}

const PYTHON_SRC_WSTDERR_LOOP_PRINT_WORLD_GOODBYE: &str = r#"
import sys
import time

for i in range([LOOPS]):
    print("Hello")
    time.sleep((i % 10) * 0.001)
    print("World")
    time.sleep((i % 10) * 0.001)
    print(f"Goodbye! {i}")
    time.sleep((i % 10) * 0.001)
    print(f"Pretend Error! {i}", file=sys.stderr)
    time.sleep((i % 10) * 0.001)
    print("\0", end="")
    time.sleep((i % 10) * 0.001)
    print("\0", end="", file=sys.stderr)
    time.sleep((i % 10) * 0.001)
"#;
const PYTHON_SRC_WSTDERR_LOOPS_KEYWORD: &str = "[LOOPS]";
const PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT: &[u8] = b"Hello\nWorld\nGoodbye! [LOOP]\n\0";
const PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDERR: &[u8] = b"Pretend Error! [LOOP]\n\0";
const PYTHON_SRC_WSTDERR_LOOP_KEYWORD: &str = "[LOOP]";

#[test_case(
    1, // loops
    1, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_WSTDERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_WSTDERR_LOOPS_KEYWORD, "1").as_str()
    ],
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDERR.to_vec()), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    Some(b'\0'); // chunk_delimiter_stderr
    "loops 1 pipesz 1"
)]
#[test_case(
    10, // loops
    1, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_WSTDERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_WSTDERR_LOOPS_KEYWORD, "10").as_str()
    ],
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDERR.to_vec()), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    Some(b'\0'); // chunk_delimiter_stderr
    "loops 10 pipesz 1"
)]
#[test_case(
    100, // loops
    1, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_WSTDERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_WSTDERR_LOOPS_KEYWORD, "100").as_str()
    ],
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDERR.to_vec()), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    Some(b'\0'); // chunk_delimiter_stderr
    "loops 100 pipesz 1"
)]
#[test_case(
    1000, // loops
    1, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_WSTDERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_WSTDERR_LOOPS_KEYWORD, "1000").as_str()
    ],
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDERR.to_vec()), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    Some(b'\0'); // chunk_delimiter_stderr
    "loops 1000 pipesz 1"
)]
#[test_case(
    10, // loops
    2, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_WSTDERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_WSTDERR_LOOPS_KEYWORD, "10").as_str()
    ],
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDERR.to_vec()), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    Some(b'\0'); // chunk_delimiter_stderr
    "loops 10 pipesz 2"
)]
#[test_case(
    10, // loops
    30, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_WSTDERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_WSTDERR_LOOPS_KEYWORD, "10").as_str()
    ],
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDERR.to_vec()), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    Some(b'\0'); // chunk_delimiter_stderr
    "loops 10 pipesz 30"
)]
#[test_case(
    10, // loops
    30, // pipe_sz
    Duration::from_millis(200),
    vec![
        "-c",
        PYTHON_SRC_WSTDERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_WSTDERR_LOOPS_KEYWORD, "10").as_str()
    ],
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDERR.to_vec()), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    Some(b'\0'); // chunk_delimiter_stderr
    "loops 10 pipesz 30 recv_timeout 200ms"
)]
#[test_case(
    10, // loops
    30, // pipe_sz
    Duration::from_millis(500),
    vec![
        "-c",
        PYTHON_SRC_WSTDERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_WSTDERR_LOOPS_KEYWORD, "10").as_str()
    ],
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDERR.to_vec()), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    Some(b'\0'); // chunk_delimiter_stderr
    "loops 10 pipesz 30 recv_timeout 500ms"
)]
#[test_case(
    100, // loops
    30, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_WSTDERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_WSTDERR_LOOPS_KEYWORD, "100").as_str()
    ],
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDERR.to_vec()), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    Some(b'\0'); // chunk_delimiter_stderr
    "loops 100 pipesz 30"
)]
#[test_case(
    1000, // loops
    90, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_WSTDERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_WSTDERR_LOOPS_KEYWORD, "1000").as_str()
    ],
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(PYTHON_SRC_WSTDERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDERR.to_vec()), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    Some(b'\0'); // chunk_delimiter_stderr
    "loops 1000 pipesz 90"
)]
fn test_PyRunner_stdout_stderr_run_many_times(
    loops: usize,
    pipe_sz: PipeSz,
    recv_timeout: Duration,
    cmd_args: Vec<&str>,
    expect_stdout: Option<Bytes>,
    expect_stderr: Option<Bytes>,
    chunk_delimiter_stdout: Option<ChunkDelimiter>,
    chunk_delimiter_stderr: Option<ChunkDelimiter>,
) {
    stack_offset_set(Some(2));
    defn!("test_PyRunner_run_many_times: loops={}, pipe_sz={:?}, chunk_delimiter_stdout={:?}, chunk_delimiter_stderr={:?}\ncmd_args[0]={:?}\ncmd_args[1]={}\n",
        loops, pipe_sz, chunk_delimiter_stdout, chunk_delimiter_stderr, cmd_args[0], cmd_args[1]);

    venv_setup();

    let python_path = find_python_executable(PythonToUse::Path)
        .as_ref().expect("failed to find python executable in the PATH");

    let result = PyRunner::new(
        PythonToUse::Value,
        pipe_sz,
        recv_timeout,
        chunk_delimiter_stdout,
        chunk_delimiter_stderr,
        Some(python_path.clone()),
        cmd_args,
    );
    assert!(result.is_ok(), "PyRunner new failed: {:?}", result);
    let mut pyr = result.unwrap();

    // clone to re-declare as mutable
    let mut expect_stdout = expect_stdout.clone();
    let mut expect_stderr = expect_stderr.clone();

    let mut loop_stdout: usize = 0;
    let mut loop_stderr: usize = 0;
    for i in 0..loops {
        defo!("PyRunner {i} write_read(None)",);
        let (exited, stdout_opt, stderr_opt) = pyr.write_read(None);
        defo!("PyRunner {i} exited? {exited}, stdout bytes {:?}, stderr bytes {:?}",
            stdout_opt.as_ref().map_or(0, |b| b.len()), stderr_opt.as_ref().map_or(0, |b| b.len()));
        match stdout_opt {
            Some(ref stdout) => {
                defo!("PyRunner {i} got stdout: '{}'", buffer_to_String_noraw(stdout));
                assert!(expect_stdout.is_some(), "PyRunner loop {i} got stdout but None expected");
                let e_s = swap_bytes(&mut expect_stdout, PYTHON_SRC_LOOP_KEYWORD, loop_stdout.to_string().as_str());
                assert_eq!(
                    stdout, &e_s,
                    "PyRunner {i} stdout mismatch on loop {loop_stdout}; got {}, expected {}",
                    buffer_to_String_noraw(stdout),
                    buffer_to_String_noraw(&e_s)
                );
                loop_stdout += 1;
            }
            None => defo!("PyRunner {i} stdout: None"),
        }
        match stderr_opt {
            Some(ref stderr) => {
                defo!("PyRunner {i} got stderr: '{}'", buffer_to_String_noraw(stderr));
                assert!(expect_stderr.is_some(), "PyRunner {i} got stderr but None expected");
                let e_s = swap_bytes(&mut expect_stderr, PYTHON_SRC_WSTDERR_LOOP_KEYWORD, loop_stderr.to_string().as_str());
                assert_eq!(
                    stderr, &e_s,
                    "PyRunner {i} stderr mismatch on loop {loop_stderr}; got {}, expected {}",
                    buffer_to_String_noraw(stderr),
                    buffer_to_String_noraw(&e_s)
                );
                loop_stderr += 1;
            }
            None => defo!("PyRunner {i} stderr: None"),
        }
        print!("."); // progress indicator
    }

    defx!();
}


const PYTHON_SRC_OUT0_ERR_LOOP_PRINT_WORLD_GOODBYE: &str = r#"
import sys
import time

for i in range([LOOPS]):
    print("Hello")
    time.sleep((i % 10) * 0.001)
    print("World")
    time.sleep((i % 10) * 0.001)
    print(f"Goodbye! {i}")
    time.sleep((i % 10) * 0.001)
    print(f"Pretend Error! {i}", file=sys.stderr)
    time.sleep((i % 10) * 0.001)
    print("\0", end="")
    time.sleep((i % 10) * 0.001)
    time.sleep((i % 10) * 0.001)
"#;
const PYTHON_SRC_OUT0_ERR_LOOPS_KEYWORD: &str = "[LOOPS]";
const PYTHON_SRC_OUT0_ERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT: &[u8] = b"Hello\nWorld\nGoodbye! [LOOP]\n\0";
const PYTHON_SRC_OUT0_ERR_LOOP_KEYWORD: &str = "[LOOP]";

#[test_case(
    1, // loops
    1, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_OUT0_ERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_OUT0_ERR_LOOPS_KEYWORD, "1").as_str()
    ],
    Some(PYTHON_SRC_OUT0_ERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(Vec::with_capacity(0)), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    None; // chunk_delimiter_stderr
    "loops 1 pipesz 1"
)]
#[test_case(
    10, // loops
    1, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_OUT0_ERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_OUT0_ERR_LOOPS_KEYWORD, "10").as_str()
    ],
    Some(PYTHON_SRC_OUT0_ERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(Vec::with_capacity(0)), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    None; // chunk_delimiter_stderr
    "loops 10 pipesz 1"
)]
#[test_case(
    100, // loops
    1, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_OUT0_ERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_OUT0_ERR_LOOPS_KEYWORD, "100").as_str()
    ],
    Some(PYTHON_SRC_OUT0_ERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(Vec::with_capacity(0)), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    None; // chunk_delimiter_stderr
    "loops 100 pipesz 1"
)]
#[test_case(
    1000, // loops
    1, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_OUT0_ERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_OUT0_ERR_LOOPS_KEYWORD, "1000").as_str()
    ],
    Some(PYTHON_SRC_OUT0_ERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(Vec::with_capacity(0)), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    None; // chunk_delimiter_stderr
    "loops 1000 pipesz 1"
)]
#[test_case(
    10, // loops
    30, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_OUT0_ERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_OUT0_ERR_LOOPS_KEYWORD, "10").as_str()
    ],
    Some(PYTHON_SRC_OUT0_ERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(Vec::with_capacity(0)), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    None; // chunk_delimiter_stderr
    "loops 10 pipesz 30"
)]
#[test_case(
    1000, // loops
    90, // pipe_sz
    RECV_TIMEOUT,
    vec![
        "-c",
        PYTHON_SRC_OUT0_ERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_OUT0_ERR_LOOPS_KEYWORD, "1000").as_str()
    ],
    Some(PYTHON_SRC_OUT0_ERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(Vec::with_capacity(0)), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    None; // chunk_delimiter_stderr
    "loops 1000 pipesz 90"
)]
#[test_case(
    1000, // loops
    90, // pipe_sz
    Duration::from_millis(1),
    vec![
        "-c",
        PYTHON_SRC_OUT0_ERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_OUT0_ERR_LOOPS_KEYWORD, "1000").as_str()
    ],
    Some(PYTHON_SRC_OUT0_ERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(Vec::with_capacity(0)), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    None; // chunk_delimiter_stderr
    "loops 1000 pipesz 90 recv_timeout 1ms"
)]
#[test_case(
    1000, // loops
    90, // pipe_sz
    Duration::from_millis(50),
    vec![
        "-c",
        PYTHON_SRC_OUT0_ERR_LOOP_PRINT_WORLD_GOODBYE.replace(PYTHON_SRC_OUT0_ERR_LOOPS_KEYWORD, "1000").as_str()
    ],
    Some(PYTHON_SRC_OUT0_ERR_LOOP_PRINTED_WORLD_GOODBYE_EXPECTED_STDOUT.to_vec()), // expect_stdout
    Some(Vec::with_capacity(0)), // expect_stderr
    Some(b'\0'), // chunk_delimiter_stdout
    None; // chunk_delimiter_stderr
    "loops 1000 pipesz 90 recv_timeout 50ms"
)]
fn test_PyRunner_stdout0_stderr_run_many_times(
    loops: usize,
    pipe_sz: PipeSz,
    recv_timeout: Duration,
    cmd_args: Vec<&str>,
    expect_stdout: Option<Bytes>,
    expect_stderr: Option<Bytes>,
    chunk_delimiter_stdout: Option<ChunkDelimiter>,
    chunk_delimiter_stderr: Option<ChunkDelimiter>,
) {
    stack_offset_set(Some(2));
    defn!("test_PyRunner_run_many_times: loops={}, pipe_sz={:?}, chunk_delimiter_stdout={:?}, chunk_delimiter_stderr={:?}\ncmd_args[0]={:?}\ncmd_args[1]={}\n",
        loops, pipe_sz, chunk_delimiter_stdout, chunk_delimiter_stderr, cmd_args[0], cmd_args[1]);

    venv_setup();

    let python_path = find_python_executable(PythonToUse::Path)
        .as_ref().expect("failed to find python executable in the PATH");

    let result = PyRunner::new(
        PythonToUse::Value,
        pipe_sz,
        recv_timeout,
        chunk_delimiter_stdout,
        chunk_delimiter_stderr,
        Some(python_path.clone()),
        cmd_args,
    );
    assert!(result.is_ok(), "PyRunner new failed: {:?}", result);
    let mut pyr = result.unwrap();

    // clone to re-declare as mutable
    let mut expect_stdout = expect_stdout.clone();

    let mut loop_stdout: usize = 0;
    for i in 0..loops {
        defo!("PyRunner {i} write_read(None)",);
        let (exited, stdout_opt, stderr_opt) = pyr.write_read(None);
        defo!("PyRunner {i} exited? {exited}, stdout bytes {:?}, stderr bytes {:?}",
            stdout_opt.as_ref().map_or(0, |b| b.len()), stderr_opt.as_ref().map_or(0, |b| b.len()));
        match stdout_opt {
            Some(ref stdout) => {
                defo!("PyRunner {i} got stdout: '{}'", buffer_to_String_noraw(stdout));
                assert!(expect_stdout.is_some(), "PyRunner loop {i} got stdout but None expected");
                let e_s = swap_bytes(&mut expect_stdout, PYTHON_SRC_OUT0_ERR_LOOP_KEYWORD, loop_stdout.to_string().as_str());
                assert_eq!(
                    stdout, &e_s,
                    "PyRunner {i} stdout mismatch on loop {loop_stdout}; got {}, expected {}",
                    buffer_to_String_noraw(stdout),
                    buffer_to_String_noraw(e_s.as_slice())
                );
                loop_stdout += 1;
            }
            None => defo!("PyRunner {i} stdout: None"),
        }
        match stderr_opt {
            Some(ref stderr) => {
                defo!("PyRunner {i} got stderr: '{}'", buffer_to_String_noraw(stderr));
                assert!(expect_stderr.is_some(), "PyRunner {i} got stderr but None expected");
                assert!(expect_stderr.as_ref().unwrap().is_empty(), "PyRunner {i} test setting should have empty stderr");
            }
            None => defo!("PyRunner {i} stderr: None"),
        }
        print!("."); // progress indicator
    }

    defx!();
}

#[test_case(
    2, // pipe_sz
    vec![
        "-c",
        r#"
import time
import sys

print("Hello", end="\n")
time.sleep(0.5)
input()
time.sleep(0.5)
print("Exiting early!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!?", end="\n", file=sys.stderr)
time.sleep(0.5)
sys.exit(1)
"#,
    ],
    b"Hello\n".to_vec(), // expect_stdout
    b"Exiting early!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!?\n".to_vec(), // expect_stderr
    Some(b'\n'), // chunk_delimiter_stdout
    None, // chunk_delimiter_stderr
    1; // exit_status
    "pipsz 2"
)]
#[test_case(
    128, // pipe_sz
    vec![
        "-c",
        r#"
import time
import sys

print("Hello", end="\n")
time.sleep(0.5)
input()
time.sleep(0.5)
print("Exiting early!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!?", end="\n", file=sys.stderr)
time.sleep(0.5)
sys.exit(1)
"#,
    ],
    b"Hello\n".to_vec(), // expect_stdout
    b"Exiting early!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!?\n".to_vec(), // expect_stderr
    Some(b'\n'), // chunk_delimiter_stdout
    None, // chunk_delimiter_stderr
    1; // exit_status
    "pipsz 128"
)]
#[test_case(
    8, // pipe_sz
    vec![
        "-c",
        r#"
import time
import sys

print("HELLO FROM STDOUT1", end="\n", file=sys.stdout)
print("HELLO FROM STDERR1", end="\n", file=sys.stderr)
time.sleep(0.5)
input()
time.sleep(0.5)
print("HELLO FROM STDERR2", end="\n", file=sys.stderr)
time.sleep(0.5)
print("HELLO FROM STDOUT2", end="\n", file=sys.stdout)
sys.exit(2)
"#,
    ],
    b"HELLO FROM STDOUT1\nHELLO FROM STDOUT2\n".to_vec(), // expect_stdout
    b"HELLO FROM STDERR1\nHELLO FROM STDERR2\n".to_vec(), // expect_stderr
    Some(b'\n'), // chunk_delimiter_stdout
    None, // chunk_delimiter_stderr
    2; // exit_status
    "pipsz 8"
)]
#[test_case(
    8, // pipe_sz
    vec![
        "-c",
        r#"
import time
import sys

print("HELLO FROM STDOUT1", end="\n", file=sys.stdout)
print("HELLO FROM STDERR1", end="\n", file=sys.stderr)
time.sleep(0.5)
input()
time.sleep(0.5)
print("HELLO FROM STDERR2", end="\n", file=sys.stderr)
time.sleep(0.5)
print("HELLO FROM STDOUT2", end="\n", file=sys.stdout)
sys.exit(2)
"#,
    ],
    b"HELLO FROM STDOUT1\nHELLO FROM STDOUT2\n".to_vec(), // expect_stdout
    b"HELLO FROM STDERR1\nHELLO FROM STDERR2\n".to_vec(), // expect_stderr
    None, // chunk_delimiter_stdout
    Some(b'\n'), // chunk_delimiter_stderr
    2; // exit_status
    "pipsz 8, swap delimiters"
)]
fn test_PyRunner_exit_early(
    pipe_sz: PipeSz,
    cmd_args: Vec<&str>,
    expect_stdout: Bytes,
    expect_stderr: Bytes,
    chunk_delimiter_stdout: Option<ChunkDelimiter>,
    chunk_delimiter_stderr: Option<ChunkDelimiter>,
    exit_status: i32,
) {
    stack_offset_set(Some(2));
    defn!("test_PyRunner_exit_early: pipe_sz={:?}, chunk_delimiter_stdout={:?}, chunk_delimiter_stderr={:?}\ncmd_args[0]={:?}\ncmd_args[1]={}\n",
        pipe_sz, chunk_delimiter_stdout, chunk_delimiter_stderr, cmd_args[0], cmd_args[1]);

    venv_setup();

    let python_path = find_python_executable(PythonToUse::Path)
        .as_ref().expect("failed to find python executable in the PATH");

    let result = PyRunner::new(
        PythonToUse::Value,
        pipe_sz,
        RECV_TIMEOUT,
        chunk_delimiter_stdout,
        chunk_delimiter_stderr,
        Some(python_path.clone()),
        cmd_args,
    );
    assert!(result.is_ok(), "PyRunner new failed: {:?}", result);
    let mut pyr = result.unwrap();

    // clone to re-declare as mutable
    let mut expect_stdout = expect_stdout.clone();
    let mut expect_stderr = expect_stderr.clone();
    defo!("expect_stdout: '{}'", buffer_to_String_noraw(expect_stdout.as_slice()));
    defo!("expect_stderr: '{}'", buffer_to_String_noraw(expect_stderr.as_slice()));

    let input2 = b"\n";
    let mut loop_: usize = 0;
    while ! pyr.exited_exhausted() {
        let input = if loop_ == 0 {
            Some(input2.as_ref())
        } else {
            None
        };
        loop_ += 1;
        defo!("PyRunner write_read({:?})", input);
        let (exited, stdout_opt, stderr_opt) = pyr.write_read(input);
        defo!("PyRunner exited? {exited}, stdout bytes {:?}, stderr bytes {:?}\nstdout: '{}'\nstderr: '{}'",
            stdout_opt.as_ref().map_or(0, |b| b.len()), stderr_opt.as_ref().map_or(0, |b| b.len()),
            stdout_opt.as_ref().map_or("".to_string(), |b| buffer_to_String_noraw(b)),
            stderr_opt.as_ref().map_or("".to_string(), |b| buffer_to_String_noraw(b)));
        if let Some(ref stdout) = stdout_opt {
            for c in stdout {
                if *c == expect_stdout[0] {
                    expect_stdout.remove(0);
                } else {
                    break;
                }
            }
        }
        if let Some(ref stderr) = stderr_opt {
            for c in stderr {
                if *c == expect_stderr[0] {
                    expect_stderr.remove(0);
                } else {
                    break;
                }
            }
        }
        if exited {
            break;
        }
    }
    assert!(pyr.exited(), "PyRunner did not exit");
    assert!(pyr.exited_exhausted(), "PyRunner not exhausted after exit");

    assert_eq!(
        pyr.exit_status().unwrap().code().unwrap(),
        exit_status,
        "PyRunner exit status mismatch after early termination"
    );

    assert!(expect_stdout.is_empty(),
        "PyRunner stdout did not match expected value; remaining '{}'",
        buffer_to_String_noraw(expect_stdout.as_slice()));
    assert!(expect_stderr.is_empty(),
        "PyRunner stderr did not match expected value; remaining '{}'",
        buffer_to_String_noraw(expect_stderr.as_slice()));

    defx!();
}

// XXX: cannot test `find_python_executable(PythonToUse::Env)` because `find_python_executable`
//      uses sets a `OnceCell` upon first call.

#[test]
fn test_find_python_executable_path() {
    defn!();

    venv_setup();

    const PATH_KEY: &str = "PATH";
    // save current env var state
    let path_env_current_opt = env::var_os(PATH_KEY);
    let path_env_current: FPath = match path_env_current_opt {
        Some(val) => FPath::from(val.to_string_lossy()),
        None => {
            panic!("PATH env var not set!? cannot run test_find_python_executable_path");
        },
    };
    // create a temporary directory to hold a contrived python executable
    let tmpdir = tempfile::tempdir().expect("failed to create temp dir");
    let contrived_python_path: PathBuf = tmpdir.path().join("python");
    // create a contrived python executable in the temp dir
    let _python_file = std::fs::File::create(&contrived_python_path)
        .expect(format!("failed to create file {:?}", contrived_python_path).as_str());
    // overwrite the PATH env var with added tmpdir at the front
    let path_new = format!(
        "{}{}{}",
        contrived_python_path.parent().unwrap().to_string_lossy(),
        std::path::MAIN_SEPARATOR,
        path_env_current
    );
    unsafe {
        env::set_var(PATH_KEY, path_new);
    }
    // call the funtions to test
    let result = find_python_executable(PythonToUse::Path);
    // restore previous env var state
    unsafe {
        env::set_var(PATH_KEY, path_env_current);
    }
    // check results of `find_python_executable(Path)`
    assert!(
        &result.is_some(),
        "find_python_executable(Path) failed; failed to find python in PATH including temporary directory {:?}",
        tmpdir.path()
    );

    defx!();
}
