// src/readers/filepreprocssor.rs

//! A collection of functions to search for potentially parseable files,
//! and prepare data needed to create a [`SyslogProcessor`] instance.
//!
//! [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor

use crate::common::{FPath, FileType};

use crate::readers::blockreader::SUBPATH_SEP;

use crate::readers::helpers::{filename_count_extensions, fpath_to_path, path_to_fpath, remove_extension};

use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate mime_guess;
#[doc(hidden)]
pub use mime_guess::MimeGuess;

extern crate si_trace_print;
#[allow(unused_imports)]
use si_trace_print::{dpfn, dpfo, dpfx, dpfñ, dpn, dpo, dpx, dpñ};

extern crate tar;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FilePreProcessor
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// TODO: [2022/06/06] a `struct FilePreProcessed` should be implemented
//     it should hold the `ProcessPathResult`, `MimeGuess`, and other stuff collected
//     during preprocessing here, and then give that to `SyslogProcessor` that gives it
//     to `BlockReader`
//     See Issue #15

/// Initial path processing return type.
#[derive(Debug, Eq, PartialEq)]
pub enum ProcessPathResult {
    /// File can be processed by `s4`
    FileValid(FPath, MimeGuess, FileType),
    // TODO: [2022/06] not currently checked until too late
    /// Filesystem permissions do not allow reading the file
    FileErrNoPermissions(FPath, MimeGuess),
    /// File is a known or unknown type and is not supported
    FileErrNotSupported(FPath, MimeGuess),
    /// Path exists and is not a file
    FileErrNotAFile(FPath),
    /// Path does not exist
    FileErrNotExist(FPath),
}

pub type ProcessPathResults = Vec<ProcessPathResult>;

lazy_static! {
    /// files without file extensions known to be parseable
    static ref PARSEABLE_FILENAMES_FILE: Vec<&'static OsStr> = {
        #[allow(clippy::vec_init_then_push)]
        let v: Vec::<&'static OsStr> = vec![
            OsStr::new("messages"),
            OsStr::new("MESSAGES"),
            OsStr::new("syslog"),
            OsStr::new("SYSLOG"),
            OsStr::new("faillog"),
            OsStr::new("lastlog"),
            OsStr::new("kernlog"),
        ];
        v
    };
    /// files without file extensions known to be not parseable
    static ref UNPARSEABLE_FILENAMES_FILE: Vec<&'static OsStr> = {
        #[allow(clippy::vec_init_then_push)]
        let v: Vec::<&'static OsStr> = vec![
            OsStr::new("wtmp"),
            OsStr::new("btmp"),
            OsStr::new("utmp"),
        ];
        v
    };
}

/// Map a single [`MimeGuess`] as a [`str`] into a `FileType`.
///
/// [`MimeGuess`]: https://docs.rs/mime_guess/2.0.4/mime_guess/struct.MimeGuess.html
pub fn mimeguess_to_filetype_str(mimeguess_str: &str) -> FileType {
    // see https://docs.rs/mime/latest/mime/
    // see https://docs.rs/mime/latest/src/mime/lib.rs.html
    // see https://github.com/abonander/mime_guess/blob/f6d36d8531bef9ad86f3ee274f65a1a31ea4d9b4/src/mime_types.rs
    dpfñ!("({:?})", mimeguess_str);
    let lower: String = mimeguess_str.to_lowercase();

    // ::mime::PLAIN.as_str();
    const PLAIN: &str = "plain";
    // ::mime::TEXT.as_str();
    const TEXT: &str = "text";
    // ::mime::TEXT_PLAIN.to_string().as_str();
    const TEXT_PLAIN: &str = "text/plain";
    // ::mime::TEXT_PLAIN_UTF_8.to_string().as_str();
    const TEXT_PLAIN_UTF8: &str = "text/plain; charset=utf-8";
    // ::mime::TEXT_STAR.to_string().as_str();
    const TEXT_STAR: &str = "text/*";
    // ::mime::UTF_8.as_str();
    const UTF8_: &str = "utf-8";

    // see https://www.rfc-editor.org/rfc/rfc6713.html#section-3
    const APP_GZIP: &str = "application/gzip";
    // see https://superuser.com/a/901963/167043
    const APP_XGZIP: &str = "application/x-gzip";

    const APP_X_XZ: &str = "application/x-xz";

    const APP_TAR: &str = "application/x-tar";
    const APP_GTAR: &str = "application/x-gtar";

    match lower.as_str() {
        PLAIN | TEXT | TEXT_PLAIN | TEXT_PLAIN_UTF8 | TEXT_STAR | UTF8_ => FileType::File,
        APP_GZIP | APP_XGZIP => FileType::Gz,
        APP_X_XZ => FileType::Xz,
        APP_TAR | APP_GTAR => FileType::Tar,
        _ => FileType::Unknown,
    }
}

/// Map a [`MimeGuess`] to a `FileType`.
///
/// [`MimeGuess`]: https://docs.rs/mime_guess/2.0.4/mime_guess/struct.MimeGuess.html
pub fn mimeguess_to_filetype(mimeguess: &MimeGuess) -> FileType {
    dpfn!("mimeguess_to_filetype({:?})", mimeguess);
    for mimeguess_ in mimeguess.iter() {
        dpo!("mimeguess_to_filetype: check {:?}", mimeguess_);
        match mimeguess_to_filetype_str(mimeguess_.as_ref()) {
            FileType::Unknown | FileType::Unset => {}
            val => {
                dpfx!("mimeguess_to_filetype: return {:?}", val);
                return val;
            }
        }
    }

    dpfx!("mimeguess_to_filetype: return {:?}", FileType::Unknown);

    FileType::Unknown
}

/// Helper function to compensates `mimeguess_to_filetype` for some files
/// not handled by `MimeGuess::from`, like file names without extensions
/// in the name, e.g. `messages` or `syslog`, or files
/// with appended extensions, e.g. `samba.log.old`.
///
/// _supplementary_ for `fn mimeguess_to_filetype`.
/// Does not replace that function!
/// e.g. calling `path_to_filetype("file.txt")` will return `FileUnknown`.
pub(crate) fn path_to_filetype(path: &Path) -> FileType {
    dpfn!("({:?})", path);

    if PARSEABLE_FILENAMES_FILE.contains(
        &path
            .file_name()
            .unwrap_or_default(),
    ) {
        dpfx!("return FILE; PARSEABLE_FILENAMES_FILE.contains({:?})", &path.file_name());
        return FileType::File;
    }

    if UNPARSEABLE_FILENAMES_FILE.contains(
        &path
            .file_name()
            .unwrap_or_default(),
    ) {
        dpfx!("return FILE; PARSEABLE_FILENAMES_FILE.contains({:?})", &path.file_name());
        return FileType::Unparseable;
    }

    // many logs have no extension in the name
    if path.extension().is_none() {
        dpfx!("return FILE; no path.extension()");
        return FileType::File;
    }

    let file_name: &OsStr = path
        .file_name()
        .unwrap_or_default();
    let file_name_s: &str = file_name
        .to_str()
        .unwrap_or_default();
    dpo!("file_name {:?}", file_name_s);
    // XXX: `file_prefix` WIP https://github.com/rust-lang/rust/issues/86319
    //let file_prefix: &OsStr = &path.file_prefix().unwrap_or_default();
    let file_prefix: &OsStr = path
        .file_stem()
        .unwrap_or_default();
    let file_prefix_s: &str = file_prefix
        .to_str()
        .unwrap_or_default();
    dpo!("file_prefix {:?}", file_prefix_s);

    let file_suffix: &OsStr = path
        .extension()
        .unwrap_or_default();
    let file_suffix_s: &str = file_suffix
        .to_str()
        .unwrap_or_default();
    dpo!("file_suffix {:?}", file_suffix_s);

    // File

    // file name `log` often on cheap embedded systems
    if file_prefix_s == "log" {
        dpfx!("return File; log");
        return FileType::File;
    }
    // for example, `log.host` as emitted by samba daemon
    if file_name_s.starts_with("log.") {
        dpfx!("return File; log.");
        return FileType::File;
    }
    // for example, `log_media`
    if file_prefix_s.starts_with("log_") {
        dpfx!("return File; log_");
        return FileType::File;
    }
    // for example, `media_log`
    if file_name_s.ends_with("_log") {
        dpfx!("return File; _log");
        return FileType::File;
    }
    // for example, `media.log.old`
    if file_name_s.ends_with(".log.old") {
        dpfx!("return File; .log.old");
        return FileType::File;
    }

    // FileGz

    // for example, `media.gz.old`
    if file_name_s.ends_with(".gz.old") {
        dpfx!("return FileGz; .gz.old");
        return FileType::Gz;
    }
    // for example, `media.gzip`
    if file_suffix_s.ends_with("gzip") {
        dpfx!("return FileGz; .gzip");
        return FileType::Gz;
    }
    // for example, `media.gz`
    // XXX: this should be handled in `path_to_filetype_mimeguess`
    if file_suffix_s.ends_with("gz") {
        dpfx!("return FileGz; .gz");
        return FileType::Gz;
    }

    // FileXz

    // for example, `media.gz.old`
    if file_name_s.ends_with(".xz.old") {
        dpfx!("return FileXz; .xz.old");
        return FileType::Xz;
    }
    // for example, `media.gzip`
    if file_suffix_s.ends_with("gzip") {
        dpfx!("return FileXz; .xzip");
        return FileType::Xz;
    }
    // for example, `media.gz`
    // XXX: this should be handled in `path_to_filetype_mimeguess`
    if file_suffix_s.ends_with("xz") {
        dpfx!("return FileXz; .xz");
        return FileType::Xz;
    }

    // FileTar

    // for example, `var-log.tar.old`
    if file_name_s.ends_with(".tar.old") {
        dpfx!("return FileTar; .tar.old");
        return FileType::Tar;
    }
    // XXX: this should be handled in `path_to_filetype_mimeguess`
    if file_name_s.ends_with(".tar") {
        dpfx!("return FileTar; .tar");
        return FileType::Tar;
    }

    // other file patterns

    // for example, `syslog.2`
    if file_prefix_s == "syslog" {
        dpfx!("return File; syslog");
        return FileType::File;
    }

    // for example, `dmesg.2`
    if file_prefix_s == "dmesg" {
        dpfx!("return File; dmesg");
        return FileType::File;
    }

    dpfx!("return FileUnknown");

    FileType::Unknown
}

/// Wrapper function for `path_to_filetype`
#[doc(hidden)]
#[cfg(any(debug_assertions, test))]
pub fn fpath_to_filetype(path: &FPath) -> FileType {
    path_to_filetype(fpath_to_path(path))
}

/// A simple `enum` to answer a simple question.
pub enum FileParseable {
    YES,
    NotSupported,
    NotParseable,
}

/// Is the `FileType` processing implemented by `s4lib`?
///
/// There are plans for future support of differing files.
pub fn parseable_filetype(filetype: &FileType) -> bool {
    match filetype {
        &FileType::File | &FileType::Gz | &FileType::Xz | &FileType::Tar => true,
        _ => false,
    }
}

#[doc(hidden)]
#[allow(dead_code)]
#[cfg(any(debug_assertions, test))]
pub(crate) fn mimeguess_parseable(mimeguess: &MimeGuess) -> bool {
    parseable_filetype(&mimeguess_to_filetype(mimeguess))
}

/// Wrapper function to call `mimeguess_to_filetype` and if necessary
/// `path_to_filetype`
#[doc(hidden)]
#[allow(dead_code)]
#[cfg(any(debug_assertions, test))]
pub(crate) fn mimguess_path_to_filetype(
    mimeguess: &MimeGuess,
    path: &Path,
) -> FileType {
    let mut filetype: FileType = mimeguess_to_filetype(mimeguess);
    if !parseable_filetype(&filetype) {
        filetype = path_to_filetype(path);
    }

    filetype
}

/// Wrapper function to call `mimeguess_to_filetype` and if necessary
/// `path_to_filetype`
#[doc(hidden)]
#[allow(dead_code)]
#[cfg(any(debug_assertions, test))]
pub(crate) fn mimeguess_fpath_to_filetype(
    mimeguess: &MimeGuess,
    path: &FPath,
) -> FileType {
    let mut filetype: FileType = mimeguess_to_filetype(mimeguess);
    if !parseable_filetype(&filetype) {
        let path_: &Path = fpath_to_path(path);
        filetype = path_to_filetype(path_);
    }

    filetype
}

/// Make an effort to determine a file's `FileType`.
/// Wrapper function to call `mimeguess_to_filetype` and if necessary
/// `path_to_filetype`.
pub fn path_to_filetype_mimeguess(path: &Path) -> (FileType, MimeGuess) {
    dpfn!("({:?})", path);
    let mut mimeguess: MimeGuess = MimeGuess::from_path(path);
    dpo!("mimeguess {:?}", mimeguess);

    // Sometimes syslog files get automatically renamed by appending `.old` to the filename,
    // or a number, e.g. `file.log.old`, `kern.log.1`. If so, try MimeGuess without the extra
    // extensions.
    if mimeguess.is_empty() {
        let mut path_: &Path = path.clone();
        let mut fpath: FPath;
        while filename_count_extensions(path_) > 1 && mimeguess.is_empty() {
            dpfo!("no mimeguess found, and file name is {:?} (multiple extensions), try again with removed file extension", path_.file_name().unwrap_or_default());
            match remove_extension(path_) {
                None => {}
                Some(fpath_rm1ext) => {
                    fpath = fpath_rm1ext;
                    path_ = fpath_to_path(&fpath);
                    mimeguess = MimeGuess::from_path(path_);
                    dpfo!("mimeguess {:?}", mimeguess);
                }
            }
        }
    }

    let mut filetype: FileType = mimeguess_to_filetype(&mimeguess);
    dpfo!("filetype {:?}", filetype);
    if !parseable_filetype(&filetype) {
        dpfo!("parseable_filetype({:?}) failed", filetype);
        filetype = path_to_filetype(path);
        dpfo!("path_to_filetype({:?}) returned {:?}", path, filetype);
        // Sometimes syslog files get automatically renamed by appending `.old` to the filename,
        // or a number, e.g. `file.log.old`, `kern.log.1`. If so, try supplement check without extra
        // extension.
        if !parseable_filetype(&filetype) && filename_count_extensions(path) > 1 {
            dpfo!(
                "file name is {:?} (multiple extensions), try again with removed file extension",
                path.file_name()
                    .unwrap_or_default()
            );
            match remove_extension(path) {
                None => {}
                Some(path_) => {
                    let std_path_: &Path = fpath_to_path(&path_);
                    filetype = path_to_filetype(std_path_);
                }
            }
        }
    }
    dpfx!("return ({:?}, {:?})", filetype, mimeguess);

    (filetype, mimeguess)
}

/// Wrapper function to call `mimeguess_to_filetype` and if necessary
/// `path_to_filetype`
#[doc(hidden)]
#[allow(dead_code)]
#[cfg(any(debug_assertions, test))]
pub(crate) fn fpath_to_filetype_mimeguess(path: &FPath) -> (FileType, MimeGuess) {
    let path_: &Path = fpath_to_path(path);

    path_to_filetype_mimeguess(path_)
}

/// Return a `ProcessPathResult` for each parseable file within
/// the `.tar` file at `path`.
pub fn process_path_tar(path: &FPath) -> Vec<ProcessPathResult> {
    dpfn!("({:?})", path);

    let file: File = File::open(path).unwrap();
    let mut archive: tar::Archive<File> = tar::Archive::<File>::new(file);
    let entry_iter: tar::Entries<File> = match archive.entries() {
        Ok(val) => val,
        Err(_err) => {
            dpfx!("Err {:?}", _err);
            //return Result::Err(err);
            return vec![];
        }
    };
    let mut results = Vec::<ProcessPathResult>::new();
    for entry_res in entry_iter {
        let entry: tar::Entry<File> = match entry_res {
            Ok(val) => val,
            Err(_err) => {
                dpo!("entry Err {:?}", _err);
                continue;
            }
        };
        let header: &tar::Header = entry.header();
        let etype: tar::EntryType = header.entry_type();
        dpo!("entry.header().entry_type() {:?}", etype);
        // TODO: handle tar types `symlink` and `long_link`, currently they are ignored
        if !etype.is_file() {
            continue;
        }
        let subpath: Cow<Path> = match entry.path() {
            Ok(val) => val,
            Err(_err) => {
                dpo!("entry.path() Err {:?}", _err);
                continue;
            }
        };
        // first get the `FileType` of the subpath
        let subfpath: FPath = subpath
            .to_string_lossy()
            .to_string();
        let filetype_subpath: FileType;
        let mimeguess: MimeGuess;
        (filetype_subpath, mimeguess) = path_to_filetype_mimeguess(&subpath);
        // the `FileType` within the tar might be a regular file. It needs to be
        // transformed to corresponding tar `FileType`, so later `BlockReader` understands what to do.
        let filetype: FileType = match filetype_subpath.to_tar() {
            FileType::Unknown => {
                dpo!("{:?}.to_tar() is FileUnknown", filetype_subpath);
                continue;
            }
            val => val,
        };
        if !parseable_filetype(&filetype) {
            dpo!("push FileErrNotSupported({:?}, {:?})", filetype, mimeguess);
            results.push(ProcessPathResult::FileErrNotSupported(subfpath, mimeguess));
            continue;
        }
        // path to a file within a .tar file looks like:
        //
        //     "path/file.tar//subpath/subfile"
        //
        // where `path/file.tar` are on the host filesystem, and `subpath/subfile` are within
        // the `.tar` file
        let mut fullpath: FPath = String::with_capacity(path.len() + 2 + subfpath.len());
        fullpath.push_str(path.as_str());
        fullpath.push(SUBPATH_SEP);
        fullpath.push_str(subfpath.as_str());
        dpo!("push FileValid({:?}, {:?}, {:?})", fullpath, mimeguess, filetype);
        results.push(ProcessPathResult::FileValid(fullpath, mimeguess, filetype));
    }

    dpfx!("process_path_tar({:?})", path);

    results
}

/// Return all parseable files in the Path.
///
/// Given a directory, recurses the directory.<br/>
/// Given a plain file path, returns that path.<br/>
/// For each recursed file, checks if file is parseable (correct file type,
/// appropriate permissions).<br/>
/// For archive files, `.tar`, enumerates files within the archive.<br/>
/// For compressed files, `.gz` `.xz`, presumes they hold only one file
/// (Relates to Issue #11, Issue #8).
///
/// This behavior assumes a user-passed file path should attempt to be parsed.
pub fn process_path(path: &FPath) -> Vec<ProcessPathResult> {
    dpfn!("({:?})", path);

    // if passed a path directly to a plain file (or a symlink to a plain file)
    // then assume the user wants to force an attempt to process such a file
    // i.e. do not call `parseable_filetype`
    let std_path: &Path = Path::new(path);
    if std_path.is_file() {
        let filetype: FileType;
        let mimeguess: MimeGuess;
        (filetype, mimeguess) = path_to_filetype_mimeguess(std_path);
        if !filetype.is_archived() {
            let paths: Vec<ProcessPathResult> =
                vec![ProcessPathResult::FileValid(path.clone(), mimeguess, filetype)];
            dpfx!("({:?}) {:?}", path, paths);
            return paths;
        }
        // is_archived
        dpfx!("process_path_tar({:?})", path);
        return process_path_tar(path);
    }

    if ! std_path.exists() {
        return vec![ProcessPathResult::FileErrNotExist(path.clone())];
    }

    // getting here means `path` likely refers to a directory

    let mut paths: Vec<ProcessPathResult> = Vec::<ProcessPathResult>::new();

    dpo!("WalkDir({:?})…", path);
    for entry in walkdir::WalkDir::new(path.as_str())
        .follow_links(true)
        .contents_first(true)
        .sort_by_file_name()
        .same_file_system(true)
    {
        // XXX: what is type `T` in `Result<T, E>` returned by `WalkDir`?
        let path_entry = match entry {
            Ok(val) => {
                dpo!("Ok({:?})", val);
                val
            }
            Err(err) => {
                dpo!("Err({:?})", err);
                continue;
            }
        };

        dpo!("analayzing {:?}", path_entry);
        let std_path_entry: &Path = path_entry.path();
        let fpath_entry: FPath = path_to_fpath(std_path_entry);
        if !path_entry
            .file_type()
            .is_file()
        {
            if path_entry
                .file_type()
                .is_dir()
            {
                continue;
            }
            dpo!("Path not a file {:?}", path_entry);
            paths.push(ProcessPathResult::FileErrNotAFile(fpath_entry));
            continue;
        }
        let filetype: FileType;
        let mimeguess: MimeGuess;
        (filetype, mimeguess) = path_to_filetype_mimeguess(std_path_entry);
        match parseable_filetype(&filetype) {
            true => {
                dpo!("paths.push(FileValid(({:?}, {:?}, {:?})))", fpath_entry, mimeguess, filetype);
                paths.push(ProcessPathResult::FileValid(fpath_entry, mimeguess, filetype));
            }
            false => {
                dpo!("Path not supported {:?}", std_path_entry);
                paths.push(ProcessPathResult::FileErrNotSupported(fpath_entry, mimeguess));
            }
        }
    }
    dpfx!("return {:?}", paths);

    paths
}
