// src/python/venv.rs

//! Create and manage the Python virtual environment for `s4`.

#[allow(deprecated)]
use std::env::home_dir;
use std::fs::create_dir_all;
use std::io::{
    ErrorKind,
    Error,
    Result,
};
use std::path::PathBuf;
use std::vec;

use ::include_dir::{
    include_dir,
    Dir as Include_Dir,
};
use ::regex::bytes::Regex;
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
};
use ::tempfile::{
    TempDir,
    env::temp_dir,
};
use ::version_compare::{
    compare_to,
    Cmp,
};

use crate::{
    debug_panic,
    e_err,
    e_wrn,
};
#[allow(unused_imports)]
use crate::de_err;
use crate::common::{
    Bytes,
    Result3E,
};
use crate::python::pyrunner::{
    ChunkDelimiter,
    PipeSz,
    PyRunner,
    PythonToUse,
    RECV_TIMEOUT,
};

/// Minimum acceptable Python version, checked during venv creation
const PYTHON_VERSION_MIN: &str = "3.9";

/// pipe size for PyRunner instances used in venv creation
const PIPE_SZ: PipeSz = 16384;

/// chunk delimiter for PyRunner instances used during venv creation
const CHUNK_DELIMITER: ChunkDelimiter = b'\n';

/// only for user-facing help messages.
/// XXX: this must match the path used in `venv_path()`
pub const PYTHON_VENV_PATH_DEFAULT: &str = "~/.config/s4/venv";

/// Python project name
const PROJECT_NAME: &str = "s4_event_readers";

/// embedded files of the s4_event_readers Python project.
/// unpacked and installed during venv creation
static PY_PROJECT_DIR: Include_Dir = include_dir!("$CARGO_MANIFEST_DIR/src/python/s4_event_readers");

/// return path to python s4 venv directory.
/// does not check if it exists
pub fn venv_path() -> PathBuf {
    #[allow(deprecated)]
    let mut home: PathBuf = match home_dir() {
        Some(h) => h,
        None => {
            // TODO: what is a better fallback path?
            temp_dir()
        },
    };
    home.push(".config");
    home.push("s4");
    home.push("venv");

    if cfg!(test) {
        // for tests, use a temporary path
        home = temp_dir();
        home.push("tmp-s4-test-python-venv");
    }

    defñ!("return {:?}", home);

    home
}

/// copy the python project into a temporary directory
/// for build and installation
pub fn deploy_pyproject_s4_event_readers() -> Result<TempDir> {
    defn!();

    let tmpdir: TempDir = match TempDir::with_prefix(format!("{}_", PROJECT_NAME)) {
        Ok(td) => td,
        Err(err) => {
            defx!("TempDir::new() error: {}", err);
            return Result::Err(err);
        }
    };

    defo!("PY_PROJECT_DIR.extract({:?})", tmpdir.path());
    match PY_PROJECT_DIR.extract(tmpdir.path()) {
        Ok(_) => {}
        Err(err) => {
            defx!("dir.extract error: {}", err);
            return Result::Err(err);
        }
    }
    defx!("Extracted PY_PROJECT_DIR {:?}", PY_PROJECT_DIR.path());

    Result::Ok(tmpdir)
}

/// extract and compare the version
/// return `Ok` if version is acceptable
/// `data` is the output of `python --version`, e.g. `b'Python 3.9.7\n'`
pub(crate) fn extract_compare_version(data: &Bytes) -> Result<()> {
    def1n!();
    // create regex to extract version
    let version_re: Regex = match Regex::new(r"^Python (\d+)\.(\d+)\.(\d+)") {
        Ok(re) => re,
        Err(err) => {
            def1x!("Regex::new returned Err {:?}", err);
            return Err(
                Error::new(
                    ErrorKind::Other,
                    format!("failed to create python version regex; {}", err),
                )
            );
        }
    };
    // regex capture the data
    let captures = match version_re.captures(data) {
        Some(captures) => captures,
        None => {
            def1x!("version_re.captures returned None");
            return Err(
                Error::new(
                    ErrorKind::Other,
                    format!("failed to capture python version from output {:?}", data),
                )
            );
        }
    };
    // get the captured part as a String
    let version_str: String = match std::str::from_utf8(&captures[0]) {
        Ok(s) => {
            def1o!("Converted version capture to str: {:?}", s);
            // remove "Python " prefix
            let mut s: String = s.to_string();
            s = s.replace("Python ", "");
            def1o!("Extracted version string: {:?}", s);

            s
        }
        Err(err) => {
            def1x!("from_utf8 returned Err {:?}", err);
            return Err(
                Error::new(
                    ErrorKind::Other,
                    format!("failed to convert python version capture to str; {}", err),
                )
            );
        }
    };
    def1o!("Found Python version {}", version_str);
    // compare the version strings with `compare_to()`
    match compare_to(&version_str, PYTHON_VERSION_MIN, Cmp::Ge) {
        Ok(cmp_result) => {
            if cmp_result {
                def1o!("Python version {} is acceptable", version_str);
            } else {
                def1x!("Python version too low; return Unsupported");
                return Err(
                    Error::new(
                        ErrorKind::Unsupported,
                        format!("python version {} is less than the required minimum {}", version_str, PYTHON_VERSION_MIN),
                    )
                );
            }
        }
        Err(err) => {
            def1x!("compare_to returned Err {:?}", err);
            return Err(
                Error::new(
                    ErrorKind::Other,
                    format!("failed to compare python versions {:?}", err),
                )
            );
        }
    }
    def1x!("return Ok");

    Ok(())
}

/// create the Python virtual environment using [`PyRunner`]s
pub fn create() -> Result3E<()> {
    def1n!();

    // run `python --version` to sanity check Python
    // using found Python interpreter
    // TODO: warn if version is less than required minimum
    let mut pyrunner = match PyRunner::new(
        PythonToUse::EnvPath,
        PIPE_SZ,
        RECV_TIMEOUT,
        Some(CHUNK_DELIMITER),
        None,
        None,
        vec![
        "--version",
    ]) {
        Ok(pyrunner) => pyrunner,
        Err(err) => {
            de_err!("Failed to create first Python runner: {}", err);
            def1x!("Python --version; return Err {:?}", err);
            return Result3E::Err(err);
        }
    };
    match pyrunner.run(true, true, true) {
        Ok((stdout, _stderr)) => {
             match extract_compare_version(&stdout) {
                Ok(_) => {},
                Err(err) if err.kind() == ErrorKind::Unsupported => {
                    e_wrn!("{}", err.to_string());
                }
                Err(err) => {
                    e_err!("Failed to compare python version: {}", err);
                    def1x!("pyrunner.run() returned Err {:?}", err);
                    return Result3E::ErrNoReprint(err);
                }
            }
        }
        Err(err) => {
            e_err!("Failed to run python --version: {}", err);
            def1x!("pyrunner.run() returned Err {:?}", err);
            return Result3E::ErrNoReprint(err);
        }
    }
    // remember the python path used
    let python_path = pyrunner.python_path;

    // create the venv directory including parent directories if it does not exist
    // using found Python interpreter
    let venv_path_pb: PathBuf = venv_path();
    def1o!("create_dir_all({:?})", venv_path_pb);
    match create_dir_all(venv_path_pb.as_path()) {
        Result::Ok(_) => {},
        Result::Err(err) => {
            e_err!("Failed to create virtual environment directory {:?}: {}", venv_path_pb, err);
            def1x!("create_dir_all returned {:?}", err);
            return Result3E::ErrNoReprint(err);
        }
    }

    // one more sanity check
    if ! venv_path_pb.is_dir() {
        let err_msg = format!("Python virtual environment path {:?} is not a directory", venv_path_pb);
        e_err!("{}", err_msg);
        def1x!("{}", err_msg);
        return Result3E::ErrNoReprint(Error::new(ErrorKind::NotADirectory, err_msg));
    }

    // create the venv using found Python interpreter
    let venv_path_s: &str = match venv_path_pb.as_os_str().to_str() {
        Some(s) => s,
        None => {
            def1x!("failed convert path to os_str to str {:?}, return Unsupported", venv_path_pb);
            return Result3E::Err(
                Error::new(
                    ErrorKind::Unsupported,
                    format!("failed to convert path to os_str to str; {:?}", venv_path_pb),
                )
            );
        }
    };
    match PyRunner::run_once(
        PythonToUse::Value,
        PIPE_SZ,
        RECV_TIMEOUT,
        CHUNK_DELIMITER,
        Some(python_path),
        vec![
            "-m",
            "venv",
            "--clear",
            "--copies",
            "--prompt",
            "s4",
            venv_path_s,
        ],
        true,
    ) {
        Ok(_) => {},
        Result::Err(err) => {
            e_err!("Failed to create Python virtual environment; venv command failed: {}", err);
            def1x!("pyrunner.run() returned {:?}", err);
            return Result3E::ErrNoReprint(err);
        }
    }

    // ensure pip is installed in the venv
    match PyRunner::run_once(
        PythonToUse::Venv,
        PIPE_SZ,
        RECV_TIMEOUT,
        CHUNK_DELIMITER,
        None,
        vec![
            "-m",
            "ensurepip",
        ],
        true,
    ) {
        Ok(_) => {},
        Err(err) => {
            e_err!("Failed to ensurepip: {}", err);
            def1x!("PyRunner::new failed {:?}", err);
            return Result3E::ErrNoReprint(err);
        }
    };

    // prevent pip from version checks
    match PyRunner::run_once(
        PythonToUse::Venv,
        PIPE_SZ,
        RECV_TIMEOUT,
        CHUNK_DELIMITER,
        None,
        vec![
            "-m",
            "pip",
            "config",
            "set",
            "--site",
            "global.disable-pip-version-check",
            "true",
        ],
        true,
    ) {
        Ok(_) => {},
        Err(err) => {
            e_err!("Failed to disable pip version check: {}", err);
            def1x!("PyRunner::new failed {:?}", err);
            return Result3E::ErrNoReprint(err);
        }
    };
    match PyRunner::run_once(
        PythonToUse::Venv,
        PIPE_SZ,
        RECV_TIMEOUT,
        CHUNK_DELIMITER,
        None,
        vec![
            "-m",
            "pip",
            "config",
            "set",
            "--site",
            "global.disable-python-version-warning",
            "true",
        ],
        true,
    ) {
        Ok(_) => {},
        Err(err) => {
            e_err!("Failed to disable python version warning: {}", err);
            def1x!("PyRunner::new failed {:?}", err);
            return Result3E::ErrNoReprint(err);
        }
    };

    // expand the project into a temporary directory
    let mut project_tmp_path: TempDir = match deploy_pyproject_s4_event_readers() {
        Ok(p) => p,
        Err(err) => {
            e_err!("Failed to deploy python project: {}", err);
            def1x!("deploy_pyproject_s4_event_readers failed {:?}", err);
            return Result3E::ErrNoReprint(err);
        }
    };
    if cfg!(debug_assertions) {
        project_tmp_path.disable_cleanup(true);
        def1o!("Temporary project remains at {:?}", project_tmp_path.path());
    }
    let project_tmp_path_s: &str = match project_tmp_path.path().as_os_str().to_str() {
        Some(s) => s,
        None => {
            let err_msg = format!(
                "failed to convert path to os_str to str; {:?}", project_tmp_path
            );
            e_err!("{}", err_msg);
            def1x!("{}", err_msg);
            return Result3E::Err(Error::new(ErrorKind::Other, err_msg));
        }
    };
    eprintln!(
        "inflated project {} to temporary path {:?}\n", PROJECT_NAME, project_tmp_path.path()
    );

    // install wheel
    // this is purely to workaround using an older etl-parser package
    // that uses legacy setup.py installation. without wheel
    // the later project installation warns of a deprecated install method.
    match PyRunner::run_once(
        PythonToUse::Venv,
        PIPE_SZ,
        RECV_TIMEOUT,
        CHUNK_DELIMITER,
        None,
        vec![
            "-m",
            "pip",
            "install",
            "wheel",
        ],
        true,
    ) {
        Ok(_) => {},
        Err(err) => {
            e_err!("Failed to ensurepip: {}", err);
            def1x!("PyRunner::new failed {:?}", err);
            return Result3E::ErrNoReprint(err);
        }
    };

    // install required python packages
    match PyRunner::run_once(
        PythonToUse::Venv,
        PIPE_SZ,
        RECV_TIMEOUT,
        CHUNK_DELIMITER,
        None,
        vec![
            "-m",
            "pip",
            "install",
            project_tmp_path_s,
        ],
        true,
    ) {
        Ok(_) => {},
        Err(err) => {
            e_err!("Failed to install python packages: {}", err);
            def1x!("PyRunner::new failed {:?}", err);
            return Result3E::ErrNoReprint(err);
        }
    };

    // get site-packages path
    let site_path: Bytes = match PyRunner::run_once(
        PythonToUse::Venv,
        PIPE_SZ,
        RECV_TIMEOUT,
        CHUNK_DELIMITER,
        None,
        vec![
            "-c",
            "import sysconfig; print(sysconfig.get_path(\"purelib\"))",
        ],
        true,
    ) {
        Ok((_, stdout, _)) => stdout,
        Err(err) => {
            e_wrn!("Failed to get site-packages path: {}", err);
            debug_panic!("PyRunner::run_once failed {:?}", err);

            Bytes::with_capacity(0)
        }
    };

    let mut argv = vec![
        "-m",
        "compileall",
        "-o2",
    ];
    // add site-packages path if it was obtained
    if ! site_path.is_empty() {
        let site_path_s: &str = match std::str::from_utf8(&site_path) {
            Ok(s) => s.trim(),
            Err(err) => {
                let err_msg = format!(
                    "failed to convert site-packages path to str; path {:?}, error {}",
                    site_path, err
                );
                e_err!("{}", err_msg);
                debug_panic!("from_utf8 failed {:?}", err);
                return Result3E::ErrNoReprint(Error::new(ErrorKind::Other, err_msg));
            }
        };
        argv.push(site_path_s);
    }

    // precompile installed site-packages
    match PyRunner::run_once(
        PythonToUse::Venv,
        PIPE_SZ,
        RECV_TIMEOUT,
        CHUNK_DELIMITER,
        None,
        argv,
        true,
    ) {
        Ok(_) => {},
        Err(err) => {
            e_wrn!("Failed to precompile python site-packages: {}; hopefully this can be ignored.", err);
            debug_panic!("PyRunner::run_once failed {:?}", err);
        }
    };

    if let Err(err) = PyRunner::run_once(
        PythonToUse::Venv,
        PIPE_SZ,
        RECV_TIMEOUT,
        CHUNK_DELIMITER,
        None,
        vec![
            "-OO",
            "-m",
            "s4_event_readers",
        ],
        true,
    ) {
        e_err!("Failed to run s4_event_readers module test: {}", err);
        def1x!("PyRunner::new failed {:?}", err);
        return Result3E::ErrNoReprint(err);
    }

    // touch special flag file to mark the venv is fully created
    let flag_path: PathBuf = venv_path().join("done");
    if let Err(err) = std::fs::write(&flag_path, b"created by s4") {
        e_err!("Failed to create {:?}: {}", flag_path, err);
        def1x!("std::fs::write returned {:?}", err);
        return Result3E::ErrNoReprint(err);
    }

    eprintln!("Python virtual environment created at {}", venv_path_pb.display());
    eprintln!("This environment will be automatically used by s4 for Python-based event readers, i.e. for .etl, .odl files.");

    def1x!("return Ok");

    Result3E::Ok(())
}
