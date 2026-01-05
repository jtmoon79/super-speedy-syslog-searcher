// src/python/pyrunner.rs

//! Runs a Python process instance. It communicates with the Python process
//! over threaded `PipeStreamReader`s connected to stdout, stderr, and stdin.
//! It uses `std::process::Child` to start and manage the Python process.

use std::cmp::{
    max,
    min,
};
use std::collections::{
    HashSet,
    VecDeque,
};
use std::env;
use std::io::{
    Error,
    ErrorKind,
    Read,
    Result,
    Write,
    stderr,
    stdout,
};
use std::path::PathBuf;
use std::process::{
    Child,
    Command,
    Stdio,
};
use std::sync::RwLock;
use std::thread;
use std::time::{
    Duration,
    Instant,
};

use ::crossbeam_channel::{
    Sender,
    Receiver,
    RecvError,
    RecvTimeoutError,
    Select,
};
use ::lazy_static::lazy_static;
use ::memchr::memmem::Finder as memchr_Finder;
use ::once_cell::sync::OnceCell;
use ::pathsearch::find_executable_in_path;
use ::shell_escape::escape;
#[allow(unused_imports)]
use ::si_trace_print::{
    defñ,
    defn,
    defo,
    defx,
    def1ñ,
    def1n,
    def1o,
    def1x,
    def2ñ,
    def2n,
    def2o,
    def2x,
    e,
    ef1n,
    ef1o,
    ef1x,
    ef1ñ,
    ef2n,
    ef2o,
    ef2x,
    ef2ñ,
};

use crate::{
    de_err,
    de_wrn,
    debug_assert_none,
    debug_panic,
};
use crate::common::{
    Bytes,
    Count,
    FPath,
    threadid_to_u64,
    summary_stat,
};
#[cfg(any(debug_assertions, test))]
use crate::debug::printers::buffer_to_String_noraw;
use crate::readers::helpers::path_to_fpath;
use crate::python::venv::venv_path;

/// Python process exit result
pub type ExitStatus = std::process::ExitStatus;

/// Size of pipe read/write buffers in bytes
pub type PipeSz = usize;

/// Delimiter byte used to separate chunks of data read from the Python process
pub type ChunkDelimiter = u8;

/// Names of possible Python executables that could be found in path
pub const PYTHON_NAMES: [&str; 13] = [
    "python3",
    "python",
    "python3.exe",
    "python.exe",
    "python37",
    "python38",
    "python39",
    "python310",
    "python311",
    "python312",
    "python313",
    "pypy3",
    "pypy",
];
/// Possible subdirectories within a Python installation where the Python
/// interpreter executable may be found
pub const PYTHON_SUBDIRS: [&str; 3] = [
    "bin",
    "Scripts",
    "",
];

pub const PROMPT_DEFAULT: &str = "$ ";

pub const CHANNEL_CAPACITY: usize = 16;

/// Environment variable that refers to the exact path to a Python interpreter
/// executable
pub const PYTHON_ENV: &str = "S4_PYTHON";

/// default timeout for Pipe `recv_timeout` when reading from the child Python processes
pub const RECV_TIMEOUT: Duration = Duration::from_millis(5);

/// cached Python path found in environment variable `S4_PYTHON`.
/// set in `find_python_executable`
#[allow(non_upper_case_globals)]
pub static PythonPathEnv: OnceCell<Option<FPath>> = OnceCell::new();
/// cached Python path found in path, set in `find_python_executable`
#[allow(non_upper_case_globals)]
pub static PythonPathPath: OnceCell<Option<FPath>> = OnceCell::new();
/// cached Python path in s4 venv, set in `find_python_executable`
#[allow(non_upper_case_globals)]
pub static PythonPathVenv: OnceCell<Option<FPath>> = OnceCell::new();

lazy_static! {
    /// Summary statistic.
    /// Record which Python interpreters ran.
    /// only intended for summary printing
    pub static ref PythonPathsRan: RwLock<HashSet<FPath>> = {
        defñ!("init PythonPathsRan");

        RwLock::new(HashSet::<FPath>::new())
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PythonToUse {
    /// only use Python referred to by environment variable `S4_PYTHON`
    Env,
    /// only use Python found in the `PATH`
    Path,
    /// use Python referred to by environment variable `S4_PYTHON` if set
    /// but if not set then use Python found in the `PATH`
    EnvPath,
    /// only use Python in predetermined s4 .venv
    Venv,
    /// use Python referred to by environment variable `S4_PYTHON` if set
    /// but if not set then use Python in predetermined s4 .venv
    EnvVenv,
    /// use a passed value
    Value,
}

/// find a Python executable.
/// `python_to_use` instructs how to find the Python executable.
/// Does not check if the found Python executable is valid.
/// Caches found paths for reliability among threads.
/// Returns `None` when passed `PythonToUse::Value`.
pub fn find_python_executable(python_to_use: PythonToUse) -> &'static Option<FPath> {
    defn!("{:?}", python_to_use);

    match python_to_use {
        PythonToUse::Env => {
            let ret: &Option<FPath> = PythonPathEnv.get_or_init(||
                // check process environment variable
                match env::var(PYTHON_ENV) {
                    Ok(val) => {
                        defo!("env::var found {}={:?}", PYTHON_ENV, val);
                        if ! val.is_empty() {
                            Some(val)
                        } else {
                            None
                        }
                    }
                    Err (_err) => {
                        defo!("env::var did not find {:?}; {:?}", PYTHON_ENV, _err);
                        None
                    }
                }
            );
            defx!("{:?}, return {:?}", python_to_use, ret);

            ret
        }
        PythonToUse::Path => {
            let ret: &Option<FPath> = PythonPathPath.get_or_init(||{
                let mut python_path: Option<PathBuf> = None;
                // check PATH for python executable
                for name in PYTHON_NAMES.iter() {
                    defo!("find_executable_in_path({:?})", name);
                    if let Some(p) = find_executable_in_path(name) {
                        defo!("find_executable_in_path returned {:?}", p);
                        python_path = Some(p);
                        break;
                    };
                }
                if let Some(p) = python_path {
                    Some(path_to_fpath(p.as_path()))
                } else {
                    None
                }
            });
            defx!("{:?}, return {:?}", python_to_use, ret);

            ret
        }
        PythonToUse::EnvPath => {
            // try Env then try Path
            let p = find_python_executable(PythonToUse::Env);
            if p.is_some() {
                defx!("{:?}, return {:?}", python_to_use, p);
                return p;
            }
            let p = find_python_executable(PythonToUse::Path);
            defx!("{:?}, return {:?}", python_to_use, p);

            p
        }
        PythonToUse::Venv => {
            let ret: &Option<FPath> = PythonPathVenv.get_or_init(||{
                // get the venv path
                let venv: PathBuf = venv_path();
                defo!("venv={:?}", venv);
                // look for common subdirectories of Python virtual environments where the
                // Python executable may be found
                // XXX: we could try to do this by platform
                //      i.e. on Windows only look in "Scripts", etc.
                //      but this is fine
                for dir in PYTHON_SUBDIRS.iter() {
                    let mut venv_dir = venv.clone();
                    if ! dir.is_empty() {
                        venv_dir.push(dir);
                    }
                    for name in PYTHON_NAMES.iter() {
                        let mut venv_name = venv_dir.clone();
                        venv_name.push(name);
                        defo!("venv_name.exists?={:?}", venv_name);
                        if venv_name.exists() {
                            let fp = path_to_fpath(venv_name.as_path());
                            defo!("found venv python executable: {:?}", fp);
                            return Some(fp);
                        }
                    }
                }
                None
            });
            defx!("{:?}, return {:?}", python_to_use, ret);

            ret
        }
        PythonToUse::EnvVenv => {
            // try Env then try Venv
            let p = find_python_executable(PythonToUse::Env);
            if p.is_some() {
                defx!("{:?}, return {:?}", python_to_use, p);
                return p;
            }
            let p = find_python_executable(PythonToUse::Venv);
            defx!("{:?}, return {:?}", python_to_use, p);

            p
        }
        PythonToUse::Value => {
            debug_panic!("PythonToUse::Value should not be used in find_python_executable");

            &None
        }
    }
}

#[derive(Debug)]
enum PipedChunk {
    /// a chunk of bytes read from the child process
    Chunk(Bytes),
    /// process not sending but still running
    Continue,
    /// process exited or no more data to read
    /// contains number of reads performed and remaining bytes
    Done(u64, Bytes),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ProcessStatus {
    #[default]
    Running,
    Exited,
}

/// Reads data from the pipe and returns chunks of data to the caller.
/// Churks are delimited by the passed `chunk_delimiter_opt` if set.
/// If `chunk_delimiter_opt` is `None` then each read immediately returns any data read.
///
/// Inspired by gist [ArtemGr/db40ae04b431a95f2b78](https://gist.github.com/ArtemGr/db40ae04b431a95f2b78).
struct PipeStreamReader {
    chunk_receiver: Receiver<core::result::Result<PipedChunk, Error>>,
    exit_sender: Sender<ProcessStatus>,
}

impl PipeStreamReader {
    /// Starts a thread reading bytes from a child process pipe.
    ///
    /// `pipe_sz` is the size of the Pipe chunk buffer in bytes.
    ///
    /// `recv_timeout` is the timeout duration for calls to `recv_timeout()`.
    ///
    /// `chunk_delimiter_opt` is an optional byte delimiter used to separate chunks of data.
    /// If `None` then each read immediately returns any data read.
    ///
    /// `stream_child_proc` is the `Read` stream of the child process to read from.
    ///
    /// `name` and `pid` are used for debugging messages.
    /// `name` is the name of the pipe.
    /// `pid` is the process ID of the child process.
    fn new(
        name: String,
        pid: u32,
        pipe_sz: PipeSz,
        recv_timeout: Duration,
        chunk_delimiter_opt: Option<ChunkDelimiter>,
        mut stream_child_proc: Box<dyn Read + Send>
    ) -> PipeStreamReader
    {
        def1n!("PipeStreamReader new(pipe_sz={}, name={:?}, chunk_delimiter_opt={:?})",
               pipe_sz, name, chunk_delimiter_opt);
        def1o!("PipeStreamReader {:?} create bounded({}) channel", name, CHANNEL_CAPACITY);
        let (tx_exit, rx_exit) =
            ::crossbeam_channel::bounded(CHANNEL_CAPACITY);

        PipeStreamReader {
            chunk_receiver: {
                let thread_name: String = format!("{}_PipeStreamReader", name);
                let _thread_name2: String = thread_name.clone();
                // parent thread ID
                let _tidn_p: u64 = threadid_to_u64(thread::current().id());
                // debug message prepend
                let _d_p = format!(
                    "PipeStreamReader {:?} PID {:?} PTID {:?}",
                    name, pid, _tidn_p
                );
                def1o!("{_d_p} create bounded({}) channel", CHANNEL_CAPACITY);
                let (tx_parent, rx_parent) =
                    ::crossbeam_channel::bounded(CHANNEL_CAPACITY);

                let thread_pipe = thread::Builder::new().name(thread_name.clone());

                def1o!("{_d_p} spawn thread {:?}", thread_name);
                let result = thread_pipe.spawn(move ||
                {
                    // debug message prepend
                    let _d_p = format!(
                        "PipeStreamReader {:?} PID {:?} PTID {:?} TID {:?}",
                        name, pid, _tidn_p, threadid_to_u64(thread::current().id()));
                    def2n!("{_d_p} start, pipe_sz {}", pipe_sz);
                    let mut _recv_bytes: usize = 0;
                    let mut reads: usize = 0;
                    let mut _sends: usize = 0;
                    let mut delim_found: bool = false;
                    let mut buf = Bytes::with_capacity(pipe_sz);
                    let buf_chunk1_sz: usize = match chunk_delimiter_opt {
                        Some(_delim) => pipe_sz,
                        None => 0, // `buf_chunk1` not used
                    };
                    let mut buf_chunk1: Bytes = Bytes::with_capacity(buf_chunk1_sz);
                    //let mut buf_chunk2: Bytes = Bytes::with_capacity(pipe_sz);
                    loop {
                        reads += 1;
                        buf.clear();
                        buf.resize(pipe_sz, 0);

                        def2o!("{_d_p} stream_child_proc.read(buf capacity {}, len {})…", buf.capacity(), buf.len());
                        /*
                        From the docs regarding read():

                            This function does not provide any guarantees about whether it blocks
                            waiting for data, but if an object needs to block for a read and cannot,
                            it will typically signal this via an Err return value.

                        See https://doc.rust-lang.org/1.83.0/std/io/trait.Read.html#tymethod.read
                        */
                        match stream_child_proc.read(&mut buf) {
                            Ok(0) => {
                                def2o!("{_d_p} read zero bytes of {} total", _recv_bytes);
                                /*
                                From the docs regarding read() returning Ok(0):

                                    This reader has reached its "end of file" and will likely no
                                    longer be able to produce bytes. Note that this does not mean
                                    that the reader will always no longer be able to produce bytes.
                                    As an example, on Linux, this method will call the recv syscall
                                    for a [TcpStream], where returning zero indicates the connection
                                    was shut down correctly. While for [File], it is possible to
                                    reach the end of file and get zero as result, but if more data
                                    is appended to the file, future calls to read will return more
                                    data.

                                See https://doc.rust-lang.org/1.83.0/std/io/trait.Read.html#tymethod.read
                                */
                                if delim_found {
                                    delim_found = false;
                                }
                                // XXX: if the child python process has exited then this may become
                                //      a busy loop until the parent thread notices the python
                                //      process has exited and then can send a
                                //      `ProcessStatus::Exited`.
                                //      Using `recv_timeout(5ms)` softens this busy loop.
                                //      It's ugly but it works.
                                def2o!("{_d_p} rx_exit.recv_timeout({:?}) (len {})…", recv_timeout, rx_exit.len());
                                let rx_result = rx_exit.recv_timeout(recv_timeout);
                                match rx_result {
                                    Ok(ProcessStatus::Exited) => {
                                        def2o!("{_d_p} rx_exit ProcessStatus::Exited; send Done({}, buf_chunk1 {} bytes) and break",
                                               reads, buf_chunk1.len());
                                        _sends += 1;
                                        def2o!("{_d_p} tx_parent.send(Ok(PipedChunk::Done({}, buf_chunk1 {} bytes))) (channel len {})…",
                                               reads, buf_chunk1.len(), tx_parent.len());
                                        match tx_parent.send(Ok(PipedChunk::Done(reads as u64, buf_chunk1))) {
                                            Ok(_) => {}
                                            Err(_err) => {
                                                def2o!("{_d_p} tx send error: {:?}", _err);
                                            }
                                        }
                                        break;
                                    }
                                    Ok(ProcessStatus::Running) => {
                                        def2o!("{_d_p} rx_exit ProcessStatus::Running; continue reading");
                                    }
                                    Err(RecvTimeoutError::Timeout) => {
                                        def2o!("{_d_p} RecvTimeoutError::Timeout; continue reading");
                                    }
                                    Err(RecvTimeoutError::Disconnected) => {
                                        def2o!("{_d_p} RecvTimeoutError::Disconnected; break");
                                        break;
                                    }
                                }
                                // send Continue if no more messages to process by parent thread
                                if tx_parent.is_empty() {
                                    def2o!("{_d_p} tx_parent.send(Ok(PipedChunk::Continue))…");
                                    match tx_parent.send(Ok(PipedChunk::Continue)) {
                                        Ok(_) => {
                                            _sends += 1;
                                        }
                                        Err(_err) => {
                                            def2o!("{_d_p} tx send error: {:?}", _err);
                                            de_err!("{_d_p} tx send error: {:?}", _err);
                                        }
                                    };
                                }
                            }
                            Ok(len) => {
                                _recv_bytes += len;
                                def2o!("{_d_p} (read #{}) read {} bytes of {} total in this pipe", reads, len, _recv_bytes);
                                // is there a chunk delimiter in the buffer?

                                match chunk_delimiter_opt {
                                    Some(chunk_delimiter) => {
                                        // look for delimiter
                                        // TODO: [2025/12] add benchmark to compare different methods
                                        //       of finding a delimiter.
                                        //       This `find_memchr` creates a new `Finder<'_>`
                                        //       which may not be worth the trouble.
                                        let needle = &[chunk_delimiter];
                                        let finder = memchr_Finder::new(needle);
                                        let mut at: usize = 0;
                                        let mut _loop: usize = 0;
                                        while at < len {
                                            _loop += 1;
                                            def2o!("{_d_p} (read #{reads} loop {_loop}) searching for delimiter in buf[{at}..{len}] '{}'", 
                                                buffer_to_String_noraw(&buf[at..len]));
                                            match finder.find(&buf[at..len]) {
                                                Some(pos) => {
                                                    // delimiter found at pos
                                                    def2o!("{_d_p} (read #{reads} loop {_loop}) found delimiter at pos {} (absolute pos {}) among {} returned bytes; buf len {}, buf capacity {}",
                                                        pos, at + pos, len, buf.len(), buf.capacity());
                                                    debug_assert!(at + pos < buf.len(), "at {} + pos {} >= buf.len {}", at, pos, buf.len());
                                                    // send chunks, keep the remainder
                                                    def2o!("{_d_p} (read #{reads} loop {_loop}) buf_chunk1.extend_from_slice(buf[{}..{}])", at, at + pos + 1);
                                                    buf_chunk1.extend_from_slice(&buf[at..at + pos + 1]);
                                                    def2o!("{_d_p} (read #{reads} loop {_loop}) buf_chunk1: '{}'", buffer_to_String_noraw(&buf_chunk1));
                                                    let blen = buf_chunk1.len();
                                                    let mut chunk_send: Bytes = Vec::<u8>::with_capacity(blen);
                                                    def2o!("{_d_p} (read #{reads} loop {_loop}) chunk_send.extend_from_slice(&buf_chunk1 len {}) (chunk_send capacity {})",
                                                        buf_chunk1.len(), chunk_send.capacity());
                                                    chunk_send.extend_from_slice(&buf_chunk1);
                                                    def2o!("{_d_p} (read #{reads} loop {_loop}) chunk_send: '{}' (channel len {})",
                                                           buffer_to_String_noraw(&chunk_send), tx_parent.len());
                                                    let data_send = PipedChunk::Chunk(chunk_send);
                                                    _sends += 1;
                                                    match tx_parent.send(Ok(data_send)) {
                                                        Ok(_) => {
                                                            def2o!("{_d_p} (read #{reads} loop {_loop}) sent chunk_send {} bytes, send #{_sends}", blen);
                                                        }
                                                        Err(_err) => {
                                                            def2o!("{_d_p} (read #{reads} loop {_loop}) send error: {:?}", _err);
                                                            break;
                                                        }
                                                    }
                                                    def2o!("{_d_p} (read #{reads} loop {_loop}) buf_chunk1.clear()");
                                                    buf_chunk1.clear();
                                                    // def2o!("{_d_p} (read #{reads} loop {_loop}) buf_chunk1.extend_from_slice(&buf[{}..{}]) (buf len {}, buf capacity {})",
                                                    //     at + pos + 1, len, buf.len(), buf.capacity());
                                                    // buf_chunk1.extend_from_slice(&buf[at + pos + 1..len]);
                                                    // def2o!("{_d_p} (read #{reads} loop {_loop}) buf_chunk1: len {}, capacity {}; contents: '{}'",
                                                    //     buf_chunk1.len(), buf_chunk1.capacity(), buffer_to_String_noraw(&buf_chunk1));
                                                    delim_found = true;
                                                    at += pos + 1;
                                                    def2o!("{_d_p} (read #{reads} loop {_loop}) {} bytes remaining in buf", len - at);
                                                }
                                                None => {
                                                    // delimiter not found, save buffer and then read child process again
                                                    def2o!("{_d_p} (read #{reads} loop {_loop}) no delimiter; buf_chunk1.extend_from_slice(&buf[{}..{}]) '{}'",
                                                        at, len, buffer_to_String_noraw(&buf[at..len]));
                                                    buf_chunk1.extend_from_slice(&buf[at..len]);
                                                    def2o!("{_d_p} (read #{reads} loop {_loop}) buf_chunk1: len {}, capacity {}; contents: '{}'",
                                                        buf_chunk1.len(), buf_chunk1.capacity(), buffer_to_String_noraw(&buf_chunk1));
                                                    delim_found = false;
                                                    at += len + 1;
                                                }
                                            }
                                        }
                                    }
                                    None => {
                                        // no delimiter configured, send entire buffer as a chunk
                                        let slice_ = &buf[..len];
                                        let blen = slice_.len();
                                        let mut chunk_send: Bytes = Vec::<u8>::with_capacity(blen);
                                        chunk_send.extend_from_slice(slice_);
                                        let data_send = PipedChunk::Chunk(chunk_send);
                                        delim_found = false;
                                        def2o!("{_d_p} (read #{reads}) read {} bytes of {} total; no delimiter configured, send Chunk {} bytes",
                                            len, _recv_bytes, blen);
                                        _sends += 1;
                                        match tx_parent.send(Ok(data_send)) {
                                            Ok(_) => {
                                                def2o!("{_d_p} (read #{reads}) sent chunk_send {} bytes, send #{_sends}", blen);
                                            }
                                            Err(_err) => {
                                                def2o!("{_d_p} (read #{reads}) send error: {:?}", _err);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(error) => {
                                if error.kind() == ErrorKind::Interrupted {
                                    def2o!("{_d_p} (read #{reads}) read interrupted; retry");
                                    continue;
                                }
                                def2o!("{_d_p} (read #{reads}) read error {}; send Error", error);
                                delim_found = false;
                                _sends += 1;
                                match tx_parent.send(Err(error)) {
                                    Ok(_) => {}
                                    Err(_err) => {
                                        def2o!("{_d_p} (read #{reads}) send error: {:?}", _err);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    def2x!(
                        "{_d_p} exit, received {} bytes, child process reads {}, parent thread sends {}",
                        _recv_bytes, reads, _sends
                    );
                });
                match result {
                    Ok(_handle) => {
                        def1o!("{_d_p} spawned thread {:?}", _thread_name2);
                    }
                    Err(_err) => {
                        def1o!("{_d_p} thread spawn error: {}", _err);
                    }
                }
                def1x!("{_d_p} return Receiver");

                rx_parent
            },
            exit_sender: tx_exit,
        }
    }
}

/// `PyRunner` is a struct that represents a Python process instance. It hides
/// the complications of starting and communicating with a Python process
/// over pipes. It uses `std::process::Child` to start and manage the Python
/// process. This handles the complexity of asynchronous inter-process
/// communication which is non-trivial to implement decently.
///
/// _XXX:_ ideally, this would use `PyO3` to communicate with a Python interpreter
/// instance. However, [`PyO3::Python::attach`] only creates one
/// Python process per Rust process.
/// And `PyO3` does not provide a way to create Python subprocesses. So all
/// Rust process threads that would use `PyO3` are bottlenecked by this
/// one Python process which is of course, in effect, a single-threaded process.
/// See [PyO3 Issue #576].
/// So instead each `PyRunner` instance creates a new Python process using
/// [`std::process::Child`] and communicates over stdout, stderr, and stdin pipes.
///
/// _XXX:_ I also tried using crate `subprocess` to manage the Python process. However,
/// it was not able to handle irregular asynchronous communication.
///
/// [`PyO3::Python::attach`]: https://docs.rs/pyo3/0.27.1/pyo3/marker/struct.Python.html#method.attach
/// [PyO3 Issue #576]: https://github.com/PyO3/pyo3/issues/576
pub struct PyRunner {
    /// handle to the Python process
    //pub process: subprocess::Popen,
    pub process: Child,
    pipe_stdout: PipeStreamReader,
    pipe_stderr: PipeStreamReader,
    /// arguments of the process
    argv: Vec<String>,
    /// path to Python exectuable
    pub python_path: FPath,
    /// save the `ExitStatus`
    exit_status: Option<ExitStatus>,
    pipe_stdout_eof: bool,
    pipe_stderr_eof: bool,
    /// protect against sending repeat exit messages to child pipe threads.
    /// only used in `pipes_exit_sender()`
    pipe_sent_exit: bool,
    /// pipe buffer size in bytes for stdout `PipeStreamReader`
    pub pipe_sz_stdout: PipeSz,
    /// pipe buffer size in bytes for stderr `PipeStreamReader`
    pub pipe_sz_stderr: PipeSz,
    /// `Instant` Python process was started.
    time_beg: Instant,
    /// `Instant` the Python process was first known to be exited.
    time_end: Option<Instant>,
    /// process ID of the Python process
    pid_: u32,
    /// this thread ID. For help during debugging.
    _tidn: u64,
    /// debug message prepend. For help during debugging.
    _d_p: String,
    /// all stderr is stored in case the process exits with an error
    /// this is because stderr may be `read` and only later calls to
    /// `poll` or `wait` may find the process has exited.
    ///
    /// oldest stderr data nearest the front
    stderr_all: Option<VecDeque<u8>>,
    /// Summary statistic.
    /// Maximum number of messages seen in the pipe_stdout channel.
    pub(crate) pipe_channel_max_stdout: Count,
    /// Summary statistic.
    /// Maximum number of messages seen in the pipe_stderr channel.
    pub(crate) pipe_channel_max_stderr: Count,
    /// Summary statistic.
    /// count of reads performed by pipeline thread reading Python process stdout
    pub(crate) count_proc_reads_stdout: Count,
    /// Summary statistic.
    /// count of reads performed by pipeline thread reading Python process stderr
    pub(crate) count_proc_reads_stderr: Count,
    /// Summary statistic.
    /// count of recv of pipeline thread reading Python process stdout
    pub(crate) count_pipe_recv_stdout: Count,
    /// Summary statistic.
    /// count of recv of pipeline thread reading Python process stderr
    pub(crate) count_pipe_recv_stderr: Count,
    /// Summary statistic.
    /// count of writes to Python process stdin
    pub(crate) count_proc_writes: Count,
    /// Summary statistic.
    /// count of polls of Python process
    pub(crate) count_proc_polls: Count,
    /// Duration of process waiting
    pub(crate) duration_proc_wait: Duration,
    /// first seen error
    pub(crate) error: Option<Error>,
}

impl std::fmt::Debug for PyRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PyRunner")
            .field("process", &self.process)
            .field("argv", &self.argv)
            .field("python_path", &self.python_path)
            .field("exit_status", &self.exit_status)
            .field("pipe_stdout_eof", &self.pipe_stdout_eof)
            .field("pipe_stderr_eof", &self.pipe_stderr_eof)
            .field("pipe_sent_exit", &self.pipe_sent_exit)
            .field("pipe_sz_stdout", &self.pipe_sz_stdout)
            .field("pipe_sz_stderr", &self.pipe_sz_stderr)
            .field("time_beg", &self.time_beg)
            .field("time_end", &self.time_end)
            .field("pid_", &self.pid_)
            .field("_tidn", &self._tidn)
            .finish()
    }
}

impl PyRunner {
    /// Create a new `PyRunner` instance.
    ///
    /// `python_to_use` indicates which Python executable to use.
    /// If `PythonToUse::Value` is used then `python_path` must be `Some(FPath)`.
    /// Otherwise `python_path` must be `None`.
    ///
    /// `argv` is the list of arguments to pass to the Python executable.
    pub fn new(
        python_to_use: PythonToUse,
        pipe_sz: PipeSz,
        recv_timeout: Duration,
        chunk_delimiter_stdout: Option<ChunkDelimiter>,
        chunk_delimiter_stderr: Option<ChunkDelimiter>,
        python_path: Option<FPath>,
        argv: Vec<&str>
    ) -> Result<Self> {
        def1n!("python_to_use {:?}, python_path {:?}, pipe_sz {:?}, chunk_delimiter_stdout {:?}, chunk_delimiter_stderr {:?}, argv {:?}",
            python_to_use, python_path, pipe_sz, chunk_delimiter_stdout, chunk_delimiter_stderr, argv);

        let python_path_: &FPath;
        // get the Python exectuble
        if python_to_use == PythonToUse::Value {
            match &python_path {
                Some(val) => python_path_ = val,
                None => {
                    let s = format!("PyRunner::new: python_path must be Some(FPath) when python_to_use is Value");
                    def1x!("Error InvalidInput {}", s);
                    return Result::Err(
                        Error::new(ErrorKind::InvalidInput, s)
                    );
                }
            }
        } else {
            debug_assert_none!(python_path, "python_path must be None unless python_to_use is Value");
            python_path_ = match find_python_executable(python_to_use) {
                Some(s) => s,
                None => {
                    let s = format!(
                        "failed to find a Python interpreter; create the Python virtual environment with command --venv, or you may specify the Python interpreter path using environment variable {}; failed",
                        PYTHON_ENV
                    );
                    def1x!("{}", s);
                    return Result::Err(
                        Error::new(ErrorKind::NotFound, s)
                    )
                }
            };
        }
        def1o!("Using Python executable: {:?}", python_path_);

        // construct argv_
        let mut argv_: Vec<&str> = Vec::with_capacity(argv.len() + 1);
        argv_.push(python_path_.as_str());
        for arg in argv.iter() {
            argv_.push(arg);
        }

        summary_stat!(
            // save this python path
            match PythonPathsRan.write() {
                Ok(mut set) => {
                    if ! set.contains(python_path_) {
                        set.insert(python_path_.clone());
                    }
                }
                Err(err) => {
                    def1x!("Failed to acquire write lock on PythonPathsRan: {}", err);
                    return Result::Err(
                        Error::new(
                            ErrorKind::Other,
                            format!("Failed to acquire write lock on PythonPathsRan: {}", err),
                        )
                    );
                }
            }
        );

        def1o!("Command::new({:?}).args({:?}).spawn()", python_path_, Vec::from_iter(argv_.iter().skip(1)));
        let time_beg: Instant = Instant::now();
        let result = Command::new(python_path_.as_str())
            .args(argv_.iter().skip(1))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let mut process: Child = match result {
            Ok(p) => p,
            Err(err) => {
                def1x!("Failed to start Python process: {}", err);
                return Result::Err(
                    Error::new(
                        err.kind(),
                        format!("Python process failed to start: Python path {:?}; {}",
                            python_path_, err),
                    )
                );
            }
        };

        // TODO: [2025/11] is there a more rustic one-liner to create Vec<String> from Vec<&str>?
        let mut argv: Vec<String> = Vec::with_capacity(argv_.len());
        for a in argv_.into_iter() {
            argv.push(String::from(a));
        }

        let pid: u32 = process.id();
        def1o!("Python process PID {}", pid);

        let _d_p = format!("Python process {}", pid);

        let process_stdout = match process.stdout.take() {
            Some(s) => s,
            None => {
                let s = format!("{_d_p} stdout was None");
                def1x!("{}", s);
                return Result::Err(
                    Error::other(s)
                );
            }
        };
        let process_stderr = match process.stderr.take() {
            Some(s) => s,
            None => {
                let s = format!("{_d_p} stderr was None");
                def1x!("{}", s);
                return Result::Err(
                    Error::other(s)
                );
            }
        };

        // create PipeStreamReaders for stdout, stderr
        def1o!("{_d_p} PipeStreamReader::new() stdout");
        let pipe_sz_stdout: usize = pipe_sz;
        let pipe_stdout = PipeStreamReader::new(
            String::from("stdout"),
            pid,
            pipe_sz_stdout,
            recv_timeout,
            chunk_delimiter_stdout,
            Box::new(process_stdout)
        );
        def1o!("{_d_p} PipeStreamReader::new() stderr");
        // stderr pipe capped at 5096 bytes
        let pipe_sz_stderr: usize = min(pipe_sz, 5096);
        let pipe_stderr = PipeStreamReader::new(
            String::from("stderr"),
            pid,
            pipe_sz_stderr,
            recv_timeout,
            chunk_delimiter_stderr,
            Box::new(process_stderr)
        );

        let _tidn: u64 = threadid_to_u64(thread::current().id());
        defx!("{_d_p} PyRunner created for Python process PID {}, TID {}", pid, _tidn);

        Result::Ok(Self {
            process,
            pipe_stdout,
            pipe_stderr,
            argv,
            python_path: python_path_.clone(),
            exit_status: None,
            pipe_stdout_eof: false,
            pipe_stderr_eof: false,
            pipe_sent_exit: false,
            pipe_sz_stdout,
            pipe_sz_stderr,
            pid_: pid,
            _tidn,
            _d_p,
            stderr_all: None,
            time_beg,
            time_end: None,
            pipe_channel_max_stdout: 0,
            pipe_channel_max_stderr: 0,
            count_proc_reads_stdout: 0,
            count_proc_reads_stderr: 0,
            count_pipe_recv_stdout: 0,
            count_pipe_recv_stderr: 0,
            count_proc_writes: 0,
            count_proc_polls: 0,
            duration_proc_wait: Duration::default(),
            error: None,
        })
    }

    #[allow(dead_code)]
    pub fn pid(&self) -> u32 {
        self.pid_
    }

    /// Returns the process exit status.
    /// If the process has not exited yet, returns `None`.
    pub fn exit_status(&self) -> Option<ExitStatus> {
        self.exit_status
    }

    /// Returns `true` if the process exited successfully.
    pub fn exit_okay(&self) -> bool {
        self.exit_status == Some(ExitStatus::default())
    }

    /// convert a `RecvError` into an `Error`
    fn new_error_from_recverror(&self, recverror: &RecvError) -> Error {
        Error::new(
            ErrorKind::Other,
            format!("Python process {} RecvError: {}",
                self.pid_, recverror),
        )
    }

    /// Returns all stderr data accumulated so far.
    /// oldest stderr data nearest the front
    pub fn stderr_all(&mut self) -> Option<&[u8]> {
        match &mut self.stderr_all {
            Some(v) => {
                v.make_contiguous();
                Some(v.as_slices().0)
            },
            None => None,
        }
    }

    /// Poll the Python process to see if it has exited.
    /// If the process has exited, returns `Some(ExitStatus)`.
    /// If the process is still running, returns `None`.
    pub fn poll(&mut self) -> Option<ExitStatus> {
        let _d_p: &String = &self._d_p;
        def1n!("{_d_p} poll()");

        summary_stat!(self.count_proc_polls += 1);

        match self.process.try_wait() {
            Ok(Some(exit_status)) => {
                if self.time_end.is_none() {
                    self.time_end = Some(Instant::now());
                    debug_assert_none!(self.exit_status, "exit_status should not be set yet");
                }
                // XXX: not sure if the subprocess returned ExitStatus would
                //      change on later polls, so only set it once
                let mut _was_exited = true;
                if self.exit_status.is_none() {
                    self.exit_status = Some(exit_status);
                    _was_exited = false;
                }
                if exit_status.success() {
                    def1x!("{_d_p} exited successfully{}", if _was_exited { " was" } else { "" });
                } else if let Some(_code) = exit_status.code() {
                    def1x!("{_d_p} exited with code {}{}", _code, if _was_exited { " was" } else { "" });
                } else {
                    def1x!("{_d_p} exited with status {:?}", exit_status);
                }
                self.pipes_exit_sender(ProcessStatus::Exited);

                Some(exit_status)
            },
            Ok(None) => {
                // Process is still alive
                def1x!("{_d_p} is still running");

                None
            },
            Err(err) => {
                def1x!("{_d_p} poll error: {}", err);
                self.error = Some(err);
                self.pipes_exit_sender(ProcessStatus::Exited);

                None
            }
        }
    }

    /// Accumulate stderr data up to a maximum number of bytes.
    // TODO: [2025/12] isn't there a more rustic way to do this?
    //       or a crate that does this?
    fn stderr_all_add(&mut self, stderr_data: &Bytes) {
        const MAX_STDERR_ALL_BYTES: usize = 1024;
        match self.stderr_all.as_mut() {
            Some(se_prior) => {
                // store as much as possible of prior + new stderr data
                if se_prior.len() + stderr_data.len() <= MAX_STDERR_ALL_BYTES {
                    se_prior.extend(stderr_data.iter());
                } else {
                    // need to drop oldest prior data from the front
                    let mut to_drop: usize = se_prior.len() + stderr_data.len() - MAX_STDERR_ALL_BYTES;
                    while to_drop > 0 {
                        se_prior.pop_front();
                        to_drop -= 1;
                    }
                    // signify the front data has been cut off
                    for b_ in "…".bytes() {
                        se_prior.push_front(b_);
                    }
                    // separate prior data from new data
                    se_prior.push_back(b'\n');
                    se_prior.push_back(b'\n');
                    // append the new data
                    se_prior.extend(stderr_data.iter());
                }
            }
            None => {
                let mut v = VecDeque::<u8>::with_capacity(
                    if stderr_data.len() > MAX_STDERR_ALL_BYTES {
                        MAX_STDERR_ALL_BYTES
                    } else {
                        stderr_data.len()
                    }
                );
                v.extend(stderr_data.iter());
                self.stderr_all = Some(v);
            }
        }
    }

    /// Send exit message to both stdout and stderr pipe threads.
    /// May be called multiple times but only the first call has effect.
    fn pipes_exit_sender(&mut self, pe: ProcessStatus) {
        if self.pipe_sent_exit {
            return;
        }
        def2ñ!("{} pipes_exit_sender({:?}) (channels len {}, {})",
               self._d_p, pe, self.pipe_stdout.exit_sender.len(), self.pipe_stderr.exit_sender.len());
        self.pipe_stdout.exit_sender.send(pe).unwrap_or(());
        self.pipe_stderr.exit_sender.send(pe).unwrap_or(());
        self.pipe_sent_exit = true;
    }

    /// Write to `input_data` then read from the Python process stdout and
    /// stderr.
    /// Returns (`exited`, `stdout`, `stderr`).
    ///
    /// The stderr_all field accumulates all stderr data read so far. This is to help
    /// when some error occurs in the Python process but the process has not yet exited
    /// and then later calls to `read` find the process has exited. The earlier writes to
    /// stderr are preserved in stderr_all because they often have the crucial error
    /// information e.g. a Python stack trace.
    pub fn write_read(&mut self, input_data: Option<&[u8]>) -> (bool, Option<Bytes>, Option<Bytes>) {
        let _len = input_data.unwrap_or(&[]).len();
        def1n!("{} input_data: {} bytes", self._d_p, _len);

        if let Some(_exit_status) = self.poll() {
            def1o!("{} already exited before read", self._d_p);
        }

        // write string, read from stdout and stderr after poll as there may still be data to read
        // even if the process has exited

        // write to stdin
        if !self.exited() {
            if let Some(input_data_) = input_data {
                if !input_data_.is_empty() {
                    match self.process.stdin.as_mut() {
                        Some(stdin) => {
                            def1o!(
                                "{} writing {} bytes to stdin (\"{}\")",
                                self._d_p,
                                input_data_.len(),
                                buffer_to_String_noraw(&input_data_[..input_data_.len().min(10)]).to_string()
                            );
                            match stdin.write(input_data_) {
                                Ok(_len) => {
                                    summary_stat!(self.count_proc_writes += 1);
                                    def1o!(
                                        "{} wrote {} bytes to stdin, expected {} bytes",
                                        self._d_p, _len, input_data_.len()
                                    );
                                }
                                Err(_err) => {
                                    de_err!("Error writing to Python process {} stdin: {:?}", self.pid_, _err);
                                    self.pipes_exit_sender(ProcessStatus::Exited);
                                }
                            }
                        }
                        None => {
                            de_err!("{} stdin is None", self._d_p);
                        }
                    }
                } else {
                    def1o!("{} no stdin data to write", self._d_p);
                }
            }
        } else {
            def1o!("{} has exited; skip writing to stdin", self._d_p);
        }

        let _d_p: &String = &self._d_p;
        // use select to block until either channel signals data is available
        let mut sel = Select::new();
        let mut _sel_counts: usize = 0;
        let sel_out: usize = if !self.pipe_stdout_eof {
            let id = sel.recv(&self.pipe_stdout.chunk_receiver) + 1; // avoid zero id
            def1o!("{_d_p} select recv(&pipe_stdout.chunk_receiver)");
            _sel_counts += 1;
            id
        } else { 0 };
        let sel_err: usize = if !self.pipe_stderr_eof {
            let id = sel.recv(&self.pipe_stderr.chunk_receiver) + 1; // avoid zero id
            def1o!("{_d_p} select recv(&pipe_stderr.chunk_receiver)");
            _sel_counts += 1;
            id
        } else { 0 };

        if sel_out == 0 && sel_err == 0 {
            def1o!("{_d_p} both stdout and stderr EOF; return");
            return (self.exited_exhausted(), None, None);
        }

        def1o!("{_d_p} wait on {} selects…", _sel_counts);
        let d1: Instant = Instant::now();
        let sel_oper = sel.select();
        summary_stat!(self.duration_proc_wait += d1.elapsed());
        let sel_index: usize = sel_oper.index() + 1; // avoid zero index
        def1o!("{_d_p} selected {}", sel_index);

        // sanity check `*_stream_eof` is not inconsistent with channel readiness
        if cfg!(any(debug_assertions,test)) {
            if sel_index == sel_out && self.pipe_stdout_eof {
                de_wrn!("pipe_stdout_eof should not be false if sel_out is ready");
            }
            if sel_index == sel_err && self.pipe_stderr_eof {
                de_wrn!("pipe_stderr_eof should not be false if sel_err is ready");
            }
        }

        let mut stdout_data: Option<Bytes> = None;
        let mut stderr_data: Option<Bytes> = None;

        // avoid borrow-checker conflicts
        let _d_p = ();

        match sel_index {
            // TODO: combine these matches since they are nearly identical?
            //       though stdout might be treated differently from stderr?
            //       maybe the messages passed back to main thread should distinguish
            //       between stdout and stderr ?
            i if i == sel_out && sel_out != 0 => {
                // read stdout
                summary_stat!(self.pipe_channel_max_stdout =
                    max(
                        self.pipe_channel_max_stdout,
                        self.pipe_stdout.chunk_receiver.len() as Count
                    )
                );
                def1o!("{} recv(&pipe_stdout.chunk_receiver)…", self._d_p);
                summary_stat!(self.count_pipe_recv_stdout += 1);
                match sel_oper.recv(&self.pipe_stdout.chunk_receiver) {
                    Ok(remote_result) => {
                        match remote_result {
                            Ok(piped_line) => {
                                match piped_line {
                                    PipedChunk::Chunk(chunk) => {
                                        let len_ = chunk.len();
                                        def1o!("{} received {} bytes from stdout", self._d_p, len_);
                                        stdout_data = Some(Vec::with_capacity(len_ + 1));
                                        let data = stdout_data.as_mut().unwrap();
                                        data.extend_from_slice(chunk.as_slice());
                                    }
                                    PipedChunk::Continue => {
                                        def1o!("{} stdout Continue", self._d_p);
                                    }
                                    PipedChunk::Done(reads, remaining_bytes) => {
                                        summary_stat!(self.count_proc_reads_stdout = reads);
                                        def1o!("{} stdout Done({} reads, {} remaining bytes)",
                                               self._d_p, reads, remaining_bytes.len());
                                        if !remaining_bytes.is_empty() {
                                            stdout_data = Some(Vec::with_capacity(remaining_bytes.len() + 1));
                                            let data = stdout_data.as_mut().unwrap();
                                            data.extend_from_slice(remaining_bytes.as_slice());
                                        }
                                        self.pipe_stdout_eof = true;
                                        self.pipes_exit_sender(ProcessStatus::Exited);
                                    }
                                }
                            }
                            Err(error) => {
                                de_err!("Error reading from Python process {} stdout: {:?}", self.pid_, error);
                                self.error = Some(error);
                                self.pipe_stdout_eof = true;
                                self.pipes_exit_sender(ProcessStatus::Exited);
                            }
                        }
                    }
                    Err(recverror) => {
                        def1o!("{} stdout channel RecvError {}; set pipe_stdout_eof=true", self._d_p, recverror);
                        self.error = Some(self.new_error_from_recverror(&recverror));
                        self.pipe_stdout_eof = true;
                        self.pipes_exit_sender(ProcessStatus::Exited);
                    }
                }
            }
            i if i == sel_err && sel_err != 0 => {
                // read stderr
                summary_stat!(self.pipe_channel_max_stderr =
                    max(
                        self.pipe_channel_max_stderr,
                        self.pipe_stderr.chunk_receiver.len() as Count
                    )
                );
                def1o!("{} recv(&pipe_stderr.chunk_receiver)…", self._d_p);
                summary_stat!(self.count_pipe_recv_stderr += 1);
                match sel_oper.recv(&self.pipe_stderr.chunk_receiver) {
                    Ok(remote_result) => {
                        match remote_result {
                            Ok(piped_line) => {
                                match piped_line {
                                    PipedChunk::Chunk(chunk) => {
                                        let len_ = chunk.len();
                                        def1o!("{} received {} bytes from stderr", self._d_p, len_);
                                        let mut data: Bytes = Bytes::with_capacity(len_);
                                        data.extend_from_slice(chunk.as_slice());
                                        self.stderr_all_add(&data);
                                        stderr_data = Some(data);
                                    }
                                    PipedChunk::Continue => {
                                        def1o!("{} stderr Continue", self._d_p);
                                    }
                                    PipedChunk::Done(reads, remaining_bytes) => {
                                        summary_stat!(self.count_proc_reads_stderr = reads);
                                        def1o!("{} stderr Done({} reads, {} remaining bytes)",
                                               self._d_p, reads, remaining_bytes.len());
                                        if !remaining_bytes.is_empty() {
                                            let mut data: Bytes = Bytes::with_capacity(remaining_bytes.len());
                                            data.extend_from_slice(remaining_bytes.as_slice());
                                            self.stderr_all_add(&data);
                                            stderr_data = Some(data);
                                        }
                                        self.pipe_stderr_eof = true;
                                        self.pipes_exit_sender(ProcessStatus::Exited);
                                    }
                                }
                            }
                            Err(error) => {
                                de_err!("Error reading from Python process {} stderr: {:?}", self.pid_, error);
                                self.error = Some(error);
                                self.pipe_stderr_eof = true;
                                self.pipes_exit_sender(ProcessStatus::Exited);
                            }
                        }
                    }
                    Err(_err) => {
                        def1o!("{} stderr channel RecvError {}; set pipe_stderr_eof=true", self._d_p, _err);
                        self.error = Some(self.new_error_from_recverror(&_err));
                        self.pipe_stderr_eof = true;
                        self.pipes_exit_sender(ProcessStatus::Exited);
                    }
                }
            }
            _i => {
                def1o!("{} selected unknown index {}", self._d_p, _i);
            }
        }

        def1x!("{} return ({}, stdout bytes {:?} (eof? {}), stderr bytes {:?} (eof? {}))",
                self._d_p,
                self.exited_exhausted(),
                stdout_data.as_ref().unwrap_or(&vec![]).len(),
                self.pipe_stdout_eof,
                stderr_data.as_ref().unwrap_or(&vec![]).len(),
                self.pipe_stderr_eof
        );

        (self.exited_exhausted(), stdout_data, stderr_data)
    }

    /// Has a `subprocess::poll` or `subprocess::wait` already returned an `ExitStatus`?
    pub fn exited(&self) -> bool {
        self.exit_status.is_some()
    }

    /// Has a `subprocess::poll` or `subprocess::wait` already returned an `ExitStatus`
    /// *and* have both stdout and stderr streams reached EOF?
    pub fn exited_exhausted(&self) -> bool {
        self.exit_status.is_some() && self.pipe_stdout_eof && self.pipe_stderr_eof
    }

    /// Wait for the Python process to exit.
    /// If the process has already exited then return the saved `ExitStatus`.
    pub fn wait(&mut self) -> Result<ExitStatus> {
        let _d_p: &String = &self._d_p;
        if self.exited() {
            def1ñ!("{_d_p} exited; return {:?}",
                   self.exit_status.unwrap());
            return Ok(self.exit_status.unwrap());
        }
        def1n!("{_d_p} wait()");
        let d1: Instant = Instant::now();
        // XXX: should `wait` be passed a timeout?
        let rc = self.process.wait();
        summary_stat!(self.duration_proc_wait += d1.elapsed());
        match rc {
            Ok(exit_status) => {
                debug_assert_none!(self.time_end, "time_end should not be set since exited() was false");
                self.time_end = Some(Instant::now());
                // prefer the first saved `ExitStatus`
                // however setting `self.exit_status` again is never expected to happen
                if self.exit_status.is_none() {
                    self.exit_status = Some(exit_status);
                } else {
                    debug_panic!("Python process {} exit_status is already set! {:?}",
                                 self.pid_, self.exit_status)
                }
                def1x!("{_d_p} wait returned {:?}", exit_status);
                return Ok(self.exit_status.unwrap());
            }
            Err(error) => {
                de_err!("{_d_p} wait returned {:?}", error);
                def1x!("{_d_p} error wait returned {:?}", error);
                return Result::Err(
                    Error::new(
                        error.kind(),
                        format!("Python process {} wait() failed: {}", self.pid_, error),
                    )
                );
            }
        }
    }

    /// Total run duration of the process; imprecise as the end time is merely the first `Instant`
    /// a `subprocess::poll` or `subprocess::wait` returned an `ExitStatus`.
    /// Precise enough for most needs.
    ///
    /// Returns `None` if the process is not yet known to have exited.
    pub fn duration(&self) -> Option<Duration> {
        self.time_end?;
        match self.time_end {
            Some(time_end) => Some(time_end - self.time_beg),
            None => {
                debug_panic!("Python process {} is exited but time_end is None", self.pid_);

                None
            }
        }
    }

    /// Read from the `PyRunner`, print the output, and wait for it to finish.
    /// Do not call `read` or `wait` before calling this function.
    /// Helper for simple Python commands that do not require interaction.
    ///
    /// This is not expected to be run as part of normal operation of
    /// `s4`. This is for special operations such as `s4 --venv`. It prints
    /// to stdout. In normal operation of `s4`, only the main
    /// thread should print to stdout.
    pub fn run(&mut self, print_argv: bool, print_stdout: bool, print_stderr: bool) -> Result<(Bytes, Bytes)> {
        let _d_p: &String = &self._d_p;
        def1n!("{_d_p}, print_argv={}", print_argv);

        if self.exited() {
            debug_panic!("{_d_p} already exited!");
            return Result::Ok((Bytes::with_capacity(0), Bytes::with_capacity(0)));
        }

        if print_argv {
            // print the command executed

            // get the prompt, prefer to use PS4 env var
            let prompt = match env::var("PS4") {
                Ok(s) => {
                    if s.is_empty() {
                        String::from(PROMPT_DEFAULT)
                    } else {
                        s
                    }
                },
                Err(_) => String::from(PROMPT_DEFAULT),
            };
            // print the command, escaping each argument
            let mut lock = stdout().lock();
            // TODO: [2025/11/17] handle write returning an error?
            let _ = lock.write(prompt.as_bytes());
            for arg in self.argv.iter() {
                let es = escape(arg.into());
                let _ = lock.write(es.as_bytes());
                let _ = lock.write(b" ");
            }
            let _ = lock.write(b"\n");
            let _ = lock.flush();
        }

        let rc = self.process.wait();
        def1o!("{_d_p} wait() returned {:?}", rc);
        match rc {
            Ok(exit_status) => {
                debug_assert_none!(self.time_end, "time_end should not be set since exited() was false");
                self.time_end = Some(Instant::now());
                // prefer the first saved `ExitStatus`
                // however setting `self.exit_status` again is never expected to happen
                if self.exit_status.is_none() {
                    self.exit_status = Some(exit_status);
                } else {
                    debug_panic!("{_d_p} exit_status is already set! {:?}",
                                 self.exit_status)
                }
            }
            Err(error) => {
                de_err!("{_d_p} wait returned {:?}", error);
                def1x!("{_d_p} error wait returned {:?}", error);
                return Result::Err(
                    Error::new(
                        error.kind(),
                        format!("Python process {} wait() failed: {}", self.pid_, error),
                    )
                );
            }
        }

        let mut stdout_data: Bytes = Bytes::with_capacity(2056);
        let mut stderr_data: Bytes = Bytes::with_capacity(1024);

        // remove _d_p reference to avoid borrow checker conflict that would occur in the loop
        let _d_p = ();

        // print remaining stdout and stderr
        loop {
            let (
                _exited,
                out_data,
                err_data,
            ) = self.write_read(None);
            def1o!("{} exited? {:?}", self._d_p, _exited);
            // print stdout to stdout
            if let Some(data) = out_data {
                stdout_data.extend_from_slice(data.as_slice());
                if ! data.is_empty() && print_stdout {
                    let mut lock = stdout().lock();
                    let _ = lock.write(data.as_slice());
                    let _ = lock.flush();
                }
            }
            // print stderr to stderr
            if let Some(data) = err_data {
                stderr_data.extend_from_slice(data.as_slice());
                if ! data.is_empty() && print_stderr {
                    let mut lock = stderr().lock();
                    let _ = lock.write(data.as_slice());
                    let _ = lock.flush();
                }
            }
            if _exited {
                break;
            }
        }

        let _d_p: &String = &self._d_p;

        match self.exit_status {
            Some(status) => {
                if ! status.success() {
                    let s = format!("Python process {} exited with non-zero status {:?}", self.pid_, status);
                    def1x!("{_d_p} {}", s);
                    return Result::Err(
                        Error::other(s)
                    )
                }
            }
            None => {
                debug_panic!("{_d_p} exit_status is None after wait()");
            }
        }

        if print_argv {
            let mut lock = stdout().lock();
            let _ = lock.write(b"\n");
            let _ = lock.flush();
        }

        def1x!("{_d_p} duration {:?}, return Ok(stdout {} bytes, stderr {} bytes)",
            self.duration(), stdout_data.len(), stderr_data.len());

        Result::Ok((stdout_data, stderr_data))
    }

    /// Create a `PyRunner`, run it, return Ok or Err.
    ///
    /// This calls `PyRunner::new()` and then `PyRunner::run()`.
    /// See `run()` regarding its intended use.
    pub fn run_once(
        python_to_use: PythonToUse,
        pipe_sz: PipeSz,
        recv_timeout: Duration,
        chunk_delimiter: ChunkDelimiter,
        python_path: Option<FPath>,
        argv: Vec<&str>,
        print_argv: bool
    ) -> Result<(PyRunner, Bytes, Bytes)> {
        def1ñ!("({:?}, {:?}, {:?})", python_to_use, python_path, argv);
        let mut pyrunner = match PyRunner::new(
            python_to_use,
            pipe_sz,
            recv_timeout,
            Some(chunk_delimiter),
            None,
            python_path,
            argv,
        ) {
            Ok(pyrunner) => pyrunner,
            Err(err) => {
                def1x!("PyRunner::new failed {:?}", err);
                return Result::Err(err);
            }
        };

        match pyrunner.run(print_argv, true, true) {
            Ok((stdout_data, stderr_data)) => {
                def1x!("PyRunner::run Ok");
                return Result::Ok((pyrunner, stdout_data, stderr_data));
            }
            Err(err) => {
                def1x!("PyRunner::run Error {:?}", err);
                return Result::Err(err);
            }
        }
    }
}
