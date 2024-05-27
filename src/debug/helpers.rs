// src/debug/helpers.rs

//! Miscellaneous helper functions for debug printing and testing.

use crate::common::FPath;
use crate::readers::helpers::path_to_fpath;

use std::fs::{create_dir, File, set_permissions};
use std::path::PathBuf;
use std::io::ErrorKind;
#[allow(unused_imports)] // XXX: clippy wrongly marks `Write` as unused
use std::io::Write; // for `NamedTempFile.write_all`

use ::filepath::FilePath; // provide `path` function on `File`
use ::lazy_static::lazy_static;
use ::si_trace_print::{defn, defo, defx, defñ};
#[doc(hidden)]
pub use ::tempfile::tempdir;
#[doc(hidden)]
pub use ::tempfile::NamedTempFile;
#[doc(hidden)]
pub use ::tempfile::TempDir;


// -------------------------------
// temporary file helper functions

/// NamedTempFile instances default to this file name prefix.
///
/// Used by helper script `tools/rust-test.sh` to delete temporary files
/// remaining after testing.
/// See <https://github.com/Stebalien/tempfile/issues/183>.
pub const STR_TEMPFILE_PREFIX: &str = "tmp-s4-test-";

lazy_static! {
    pub static ref STRING_TEMPFILE_PREFIX: String = String::from(STR_TEMPFILE_PREFIX);
    // there is no `String::default` so create this just once
    static ref STRING_TEMPFILE_SUFFIX: String = String::from("");
}

/// Small helper function for copying `NamedTempFile` path to a `FPath`.
pub fn ntf_fpath(ntf: &NamedTempFile) -> FPath {
    FPath::from(ntf.path().to_str().unwrap())
}

/// Testing helper function to write a `str` to a temporary file.
///
/// Wrapper for [`create_temp_file_data`].
pub fn create_temp_file(data: &str) -> NamedTempFile {
    create_temp_file_data(data.as_bytes())
}

/// Testing helper function to write a `[u8]` to a temporary file.
///
/// BUG: `NamedTempFile` created within `lazy_static` will fail to remove itself
///      <https://github.com/Stebalien/tempfile/issues/183>.
pub fn create_temp_file_data(data: &[u8]) -> NamedTempFile {
    let mut ntf = match tempfile::Builder::new()
        // use known prefix for easier cleanup
        .prefix::<str>(&STRING_TEMPFILE_PREFIX)
        .tempfile()
    {
        Ok(val) => val,
        Err(err) => {
            panic!("NamedTempFile::new() return Err {}", err);
        }
    };
    match ntf.write_all(data) {
        Ok(_) => {}
        Err(err) => {
            panic!("NamedTempFile::write_all() return Err {}", err);
        }
    }

    ntf
}

/// Testing helper function to write a `[u8]` to a specially-named
/// temporary file.
///
/// `rand_len` is the string length of a random character sequence
pub fn create_temp_file_with_name_rlen(
    data: &[u8],
    prefix: Option<&String>,
    suffix: Option<&String>,
    rand_len: usize,
) -> NamedTempFile {
    let mut ntf = match tempfile::Builder::new()
        .prefix::<str>(
            prefix
                .unwrap_or(&STRING_TEMPFILE_PREFIX)
                .as_ref(),
        )
        .suffix::<str>(
            suffix
                .unwrap_or(&STRING_TEMPFILE_SUFFIX)
                .as_ref(),
        )
        .rand_bytes(rand_len)
        .tempfile()
    {
        Ok(val) => val,
        Err(err) => {
            panic!("tempfile::Builder::new().tempfile() return Err {}", err);
        }
    };
    match ntf.write_all(data) {
        Ok(_) => {}
        Err(err) => {
            panic!("NamedTempFile::write_all() return Err {}", err);
        }
    }

    ntf
}

/// Testing helper function to write a `str` to a specially-named
/// temporary file.
pub fn create_temp_file_with_name(
    data: &str,
    prefix: Option<&String>,
    suffix: Option<&String>,
) -> NamedTempFile {
    // XXX: tempfile::NUM_RAND_CHARS is not pub
    create_temp_file_with_name_rlen(data.as_bytes(), prefix, suffix, 5)
}

/// Testing helper function to write a `str` to a temporary file with a specific
/// suffix
pub fn create_temp_file_with_suffix(
    data: &str,
    suffix: &String,
) -> NamedTempFile {
    create_temp_file_with_name_rlen(data.as_bytes(), None, Some(suffix), 5)
}

/// Testing helper function to write a `str` to a exactly-named temporary file.
pub fn create_temp_file_with_name_exact(
    data: &str,
    name: &String,
) -> NamedTempFile {
    create_temp_file_with_name_rlen(data.as_bytes(), Some(name), None, 0)
}

/// Testing helper function to write a `[u8]` to a temporary file.
pub fn create_temp_file_bytes(data: &[u8]) -> NamedTempFile {
    create_temp_file_with_name_rlen(data, None, None, 5)
}

/// Testing helper function to write a `[u8]` to a temporary file.
pub fn create_temp_file_bytes_with_suffix(
    data: &[u8],
    suffix: &String,
) -> NamedTempFile {
    create_temp_file_with_name_rlen(data, None, Some(suffix), 5)
}

/// Create a temporary directory
pub fn create_temp_dir() -> TempDir {
    defñ!();
    tempfile::tempdir().unwrap()
}

pub fn create_dir_in_tmpdir(
    pathb: &PathBuf,
    tempdir: &TempDir,
) {
    defn!("({:?}, {:?})", pathb, tempdir);
    let mut pathb_tmp: PathBuf = tempdir.path().to_path_buf();
    for c in pathb.components() {
        pathb_tmp = pathb_tmp.join(PathBuf::from(c.as_os_str()));
        defñ!("create_dir({:?})", pathb_tmp);
        match create_dir(&pathb_tmp) {
            Ok(_) => {}
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {}
            Err(err) => panic!("Error {:?}", err),
        }
    }
    defx!();
}

/// Testing helper function to write a `[u8]` to a file in a temporary directory.
/// Will create leading directories in paths, e.g. `name` value `foo/bar` creates
/// directory `foo` and file `bar`.
pub fn create_file_bytes_name_in_tmpdir(
    data: &[u8],
    name: &FPath,
    tempdir: &TempDir,
) -> Option<File> {
    defn!("({:?}, {:?}, …)", name, tempdir);
    let pathb_name: PathBuf = PathBuf::from(name);

    // create directories with the passed `name` if `name` ends with "/"
    if name.ends_with('/') {
        create_dir_in_tmpdir(&pathb_name, tempdir);
        return None;
    }

    // create directories with the passed `name` but do not use last component (that is a filename)
    let mut pathb_tmp: PathBuf = PathBuf::new();
    for c in pathb_name.components().take(pathb_name.components().count() - 1) {
        pathb_tmp = pathb_tmp.join(PathBuf::from(c.as_os_str()));
    }
    create_dir_in_tmpdir(&pathb_tmp, tempdir);

    // create file with the passed `name`
    #[allow(for_loops_over_fallibles)]
    // XXX: manual inspection shows this code is safe; no risk of "fallible"
    //      changes during the for-loop
    for c in pathb_name.components().nth(pathb_name.components().count() - 1) {
        pathb_tmp = pathb_tmp.join(PathBuf::from(c.as_os_str()));
    }
    let path_file = tempdir.path().join(pathb_tmp);
    defo!("File::create({:?})", path_file);
    let mut file_ = match File::create(path_file) {
        Ok(f) => f,
        Err(err) => panic!("Error {}", err),
    };
    file_.write_all(data).unwrap();
    defx!();

    Some(file_)
}

/// Testing helper to create files within the passed `TempDir`
pub fn create_files_in_tmpdir(
    tmpdir: &TempDir,
    filenames: &[FPath],
) -> Vec<FPath> {
    defn!("({:?}, {:?})", tmpdir, filenames);
    let mut files = Vec::<FPath>::new();

    for fpath in filenames.iter() {
        let file = match create_file_bytes_name_in_tmpdir(&[], fpath, tmpdir) {
            Some(f) => f,
            None => continue,
        };
        let path_ = &file.path().unwrap();
        let fpath: FPath = path_to_fpath(path_);
        files.push(fpath)
    }
    defx!();

    files
}

/// Testing helper to create a directory within the passed `TempDir`
pub fn create_dirs_in_tmpdir(
    tmpdir: &TempDir,
    dirnames: &[FPath],
) -> Vec<FPath> {
    defn!("({:?}, {:?})", tmpdir, dirnames);
    let mut fpaths = Vec::<FPath>::new();
    let path = tmpdir.path();

    for fpath in dirnames.iter() {
        let path_ = path.join(fpath);
        defo!("create_dir({:?})", path_);
        if let Err(err) = create_dir(path_.as_path()) {
            panic!("Error {:?}", err);
        }
        fpaths.push(path_to_fpath(path_.as_path()));
    }
    defx!();

    fpaths
}

/// Testing helper to create a `TempDir` and files
pub fn create_files_and_tmpdir(filenames: &[FPath]) -> (TempDir, Vec<FPath>) {
    defn!("({:?})", filenames);
    let tmpdir = create_temp_dir();
    let mut files = Vec::<FPath>::new();

    for fpath in filenames.iter() {
        let file = match create_file_bytes_name_in_tmpdir(&[], fpath, &tmpdir) {
            Some(f) => f,
            None => continue,
        };
        let path_ = &file.path().unwrap();
        let fpath: FPath = path_to_fpath(path_);
        files.push(fpath)
    }
    defx!();

    (tmpdir, files)
}

/// Create a temp file with no read or write permissions.
///
/// Currently only for Unix. It appears there is no cross-platform method to
/// _remove_ file permission allowances, i.e. remove the "read" permission.
/// See <https://doc.rust-lang.org/nightly/std/fs/struct.Permissions.html>
#[cfg(target_family = "unix")]
pub fn create_temp_file_no_permissions(suffix: &str) -> NamedTempFile {
    #[allow(unused_imports)]
    use std::os::unix::fs::PermissionsExt;

    let mut prefix: String = String::from(STR_TEMPFILE_PREFIX);
    prefix.push_str("no_permissions-");
    let ntf = create_temp_file_with_name(
        "sample data",
        Some(&prefix),
        Some(&String::from(suffix)),
    );
    let path = ntf.path();
    let perm = std::fs::Permissions::from_mode(0o000);
    match set_permissions(path, perm) {
        Ok(_) => {}
        Err(err) => {
            panic!("failed to set_permissions {:?} {}", path, err);
        }
    }

    ntf
}

/// non-Unix platform stub.
#[cfg(not(target_family = "unix"))]
pub fn create_temp_file_no_permissions() -> NamedTempFile {
    unimplemented!("create_temp_file_no_permissions should only run on a Unix");
}
