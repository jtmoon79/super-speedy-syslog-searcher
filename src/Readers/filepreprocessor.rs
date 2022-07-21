// Readers/filepreprocssor.rs
//
// a collection of functions to search for potentially parseable files,
// and prepare for the creation of `SyslogProcessor`

use crate::common::{
    FPath,
    FileType,
};

use crate::Readers::blockreader::{
    SUBPATH_SEP,
};

use crate::Readers::helpers::{
    path_to_fpath,
    fpath_to_path,
    remove_extension,
    filename_count_extensions,
};

#[cfg(any(debug_assertions,test))]
use crate::printer_debug::stack::{
    sn,
    snx,
    so,
    sx,
};

pub use crate::Readers::linereader::{
    ResultS4_LineFind,
};

use std::borrow::Cow;
use std::fs::File;
use std::ffi::OsStr;
use std::path::Path;

extern crate debug_print;
use debug_print::debug_eprintln;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate mime;

extern crate mime_guess;
pub use mime_guess::MimeGuess;

extern crate tar;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FilePreProcessor
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// TODO: [2022/06/02] AFAICT, this doens't need to be a long-lived object,
// only a series of functions... thinking about it... this series of functions could
// be placed within `syslogprocessor.rs`:
//    pub fn generate_syslogprocessor(path: FPath) -> Vec<(ProcessPathResult, Option<SyslogProcessor>)>
// with helper function:
//    pub fn process_path(path: FPath) -> Vec<ProcessPathResult>
//
// type ProcessPathResult = (Path, Option<SubPath>, FileType);
//
// The algorithm for analyzing a path would be:
//    if directory the recurse directory for more paths.
//    if not file then eprintln and (if --summary the save error) and return.
//    (must be plain file so)
//    if file name implies obvious file type then presume mimeguess to be correct.
//       example, `messages.gz`, is very likely a gzipped text file. Try to gunzip. If gunzip fails then give up on it. (`FILE_ERR_DECOMPRESS_FAILED`)
//       example, `logs.tar`, is very likely multiple tarred text files. Try to untar. If untar fails then give up on it. (`FILE_ERR_UNARCHIVE_FAILED`)
//    else if mime analysis has likely answer then presume that to be correct.
//        example, `messages`, is very likely a text file.
//    else try blockzero analysis (attempt to parse Lines and Syslines).
// Failures to process paths should be:
//    eprintln() at time of opening failure.
//    if --summary then printed with the closing summary.
//
// That algorithm should be correct in 99% of cases.
//
// LAST WORKING HERE 2022/06/06 the `struct FilePreProcessed` should be implemented
//     it should hold the `ProcessPathResult`, `MimeGuess`, and other stuff collected
//     during preprocessing here.

#[derive(Debug, Eq, PartialEq)]
pub enum ProcessPathResult {
    FILE_VALID(FPath, MimeGuess, FileType),
    FILE_ERR_NO_PERMISSIONS(FPath, MimeGuess),
    FILE_ERR_NOT_SUPPORTED(FPath, MimeGuess),
    FILE_ERR_NOT_PARSEABLE(FPath, MimeGuess),
    FILE_ERR_NOT_A_FILE(FPath, MimeGuess),
}

pub type ProcessPathResults = Vec<ProcessPathResult>;

lazy_static! {
    static ref PARSEABLE_FILENAMES_FILE: Vec<&'static OsStr> = {
        #[allow(clippy::vec_init_then_push)]
        let mut v = Vec::<&'static OsStr>::with_capacity(7);
        v.push(OsStr::new("messages"));
        v.push(OsStr::new("MESSAGES"));
        v.push(OsStr::new("syslog"));
        v.push(OsStr::new("SYSLOG"));
        v.push(OsStr::new("faillog"));
        v.push(OsStr::new("lastlog"));
        v.push(OsStr::new("kernlog"));
        v
    };
}

/// map `MimeGuess` into a `FileType`
/// (i.e. call `find_line`)
pub fn mimeguess_to_filetype_str(mimeguess_str: &str) -> FileType {
    // see https://docs.rs/mime/latest/mime/
    // see https://docs.rs/mime/latest/src/mime/lib.rs.html
    // see https://github.com/abonander/mime_guess/blob/f6d36d8531bef9ad86f3ee274f65a1a31ea4d9b4/src/mime_types.rs
    debug_eprintln!("{}mimeguess_to_filetype_str: mimeguess {:?}", snx(), mimeguess_str);
    let lower = mimeguess_str.to_lowercase();
    //
    const plain: &str = "plain"; //mime::PLAIN.as_str();
    const text: &str = "text"; //mime::TEXT.as_str();
    const text_plain: &str = "text/plain"; //mime::TEXT_PLAIN.to_string().as_str();
    const text_plain_utf8: &str = "text/plain; charset=utf-8"; //mime::TEXT_PLAIN_UTF_8.to_string().as_str();
    const text_star: &str = "text/*"; //mime::TEXT_STAR.to_string().as_str();
    const utf8_: &str = "utf-8"; //mime::UTF_8.as_str();
    //
    const app_gzip: &str = "application/gzip";
    //
    const app_x_xz: &str = "application/x-xz";
    //
    const app_tar: &str = "application/x-tar";

    // LAST WORKING HERE 2022/07/10 00:24:49 beginning of implementation of handling tar files
    match lower.as_str() {
        plain
        | text
        | text_plain
        | text_plain_utf8
        | text_star
        | utf8_ => FileType::FILE,
        app_gzip => FileType::FILE_GZ,
        app_tar => FileType::FILE_TAR,
        app_x_xz => FileType::FILE_XZ,
        _ => FileType::FILE_UNKNOWN,
    }
}

/// can a `LineReader` parse this file/MIME type?
/// (i.e. call `self.find_line()`)
pub fn mimeguess_to_filetype(mimeguess: &MimeGuess) -> FileType {
    debug_eprintln!("{}mimeguess_to_filetype({:?})", sn(), mimeguess);
    for mimeguess_ in mimeguess.iter() {
        debug_eprintln!("{}mimeguess_to_filetype: check {:?}", so(), mimeguess_);
        match mimeguess_to_filetype_str(mimeguess_.as_ref()) {
            FileType::FILE_UNKNOWN | FileType::FILE_UNSET_ => {},
            val => {
                debug_eprintln!("{}mimeguess_to_filetype: return {:?}", sx(), val);
                return val;
            }
        }
    }

    debug_eprintln!("{}mimeguess_to_filetype: return {:?}", sx(), FileType::FILE_UNKNOWN);

    FileType::FILE_UNKNOWN
}

/// compensates `mimeguess_to_filetype` for some files not handled by `MimeGuess::from`,
/// like file names without extensions in the name, e.g. `messages` or `syslog`
pub fn path_to_filetype(path: &Path) -> FileType {
    debug_eprintln!("{}path_to_filetype({:?})", sn(), path);

    if PARSEABLE_FILENAMES_FILE.contains(&path.file_name().unwrap_or_default()) {
        debug_eprintln!("{}path_to_filetype: return FILE; PARSEABLE_FILENAMES_FILE.contains({:?})", sx(), &path.file_name());
        return FileType::FILE;
    }
    // many logs have no extension in the name
    if path.extension().is_none() {
        debug_eprintln!("{}path_to_filetype: return FILE; no path.extension()", sx());
        return FileType::FILE;
    }
    // XXX: `file_prefix` WIP https://github.com/rust-lang/rust/issues/86319
    //let file_prefix: &OsStr = &path.file_prefix().unwrap_or_default();
    let file_prefix: &OsStr = path.file_stem().unwrap_or_default();
    let file_prefix_s: &str = file_prefix.to_str().unwrap_or_default();
    debug_eprintln!("{}path_to_filetype: file_prefix {:?}", so(), file_prefix_s);

    let file_suffix: &OsStr = path.extension().unwrap_or_default();
    let file_suffix_s: &str = file_suffix.to_str().unwrap_or_default();
    debug_eprintln!("{}path_to_filetype: file_suffix {:?}", so(), file_suffix_s);

    // FILE

    // file name `log` often on cheap embedded systems
    if file_prefix_s == "log" {
        debug_eprintln!("{}path_to_filetype: return FILE; log", sx());
        return FileType::FILE;
    }
    // for example, `log.host` as emitted by samba daemon
    if file_prefix_s.starts_with("log.") {
        debug_eprintln!("{}path_to_filetype: return FILE; log.", sx());
        return FileType::FILE;
    }
    // for example, `log_media`
    if file_prefix_s.starts_with("log_") {
        debug_eprintln!("{}path_to_filetype: return FILE; log_", sx());
        return FileType::FILE;
    }
    // for example, `media_log`
    if file_prefix_s.ends_with("_log") {
        debug_eprintln!("{}path_to_filetype: return FILE; _log", sx());
        return FileType::FILE;
    }
    // for example, `media.log.old`
    if file_suffix_s.ends_with(".log.old") {
        debug_eprintln!("{}path_to_filetype: return FILE; .log.old", sx());
        return FileType::FILE;
    }

    // FILE_GZ

    // for example, `media.gz.old`
    // TODO: this should be handled in `guess_filetype_from_path`, can be removed
    if file_suffix_s.ends_with(".gz.old") {
        debug_eprintln!("{}path_to_filetype: return FILE_GZ; .gz.old", sx());
        return FileType::FILE_GZ;
    }
    // for example, `media.gzip`
    if file_suffix_s.ends_with(".gzip") {
        debug_eprintln!("{}path_to_filetype: return FILE_GZ; .gzip", sx());
        return FileType::FILE_GZ;
    }

    // FILE_TAR

    // for example, `var-log.tar.old`
    // TODO: this should be handled in `guess_filetype_from_path`, can be removed
    if file_suffix_s.ends_with(".tar.old") {
        debug_eprintln!("{}path_to_filetype: return FILE_TAR; .tar.old", sx());
        return FileType::FILE_TAR;
    }

    debug_eprintln!("{}path_to_filetype: return FILE_UNKNOWN", sx());

    FileType::FILE_UNKNOWN
}

/// wrapper for `path_to_filetype`
#[cfg(any(debug_assertions,test))]
pub fn fpath_to_filetype(path: &FPath) -> FileType {
    path_to_filetype(fpath_to_path(path))
}

pub enum FileParseable {
    YES,
    NO_NOT_SUPPORTED,
    NO_NOT_PARSEABLE,
}

/// is `FileType` supported?
pub fn parseable_filetype(filetype: &FileType) -> FileParseable {
    match filetype {
        // `YES` is effectively the list of currently supported file types
        &FileType::FILE
        | &FileType::FILE_GZ
        | &FileType::FILE_XZ
        | &FileType::FILE_TAR
        => FileParseable::YES,
        // `NOT_SUPPORTED` is the list of "Someday this program should support this file type"
        | &FileType::FILE_TAR_GZ
        => FileParseable::NO_NOT_SUPPORTED,
        // etc.
        _ => FileParseable::NO_NOT_PARSEABLE,
    }
}

/// reduce `parseable_filetype` to a boolean
pub fn parseable_filetype_ok(filetype: &FileType) -> bool {
    matches!(parseable_filetype(filetype), FileParseable::YES)
}

/// reduce `mimeguess_to_filetype()` to a boolean
#[cfg(any(debug_assertions,test))]
pub fn mimeguess_to_filetype_ok(mimeguess: &MimeGuess) -> bool {
    matches!(parseable_filetype(&mimeguess_to_filetype(mimeguess)), FileParseable::YES)
}

/// wrapper to call `mimeguess_to_filetype` and if necessary `path_to_filetype`
/// TODO: change name to `path_mimeguess_to_filetype`
#[cfg(any(debug_assertions,test))]
pub fn guess_filetype_from_mimeguess_path(mimeguess: &MimeGuess, path: &Path) -> FileType {
    let mut filetype: FileType = mimeguess_to_filetype(mimeguess);
    if ! parseable_filetype_ok(&filetype) {
        filetype = path_to_filetype(path);
    }

    filetype
}

/// wrapper to call `mimeguess_to_filetype` and if necessary `path_to_filetype`
/// TODO: change name to `fpath_mimegyess_to_filetype_`
#[cfg(any(debug_assertions,test))]
pub fn guess_filetype_from_mimeguess_fpath(mimeguess: &MimeGuess, path: &FPath) -> FileType {
    let mut filetype: FileType = mimeguess_to_filetype(mimeguess);
    if ! parseable_filetype_ok(&filetype) {
        let path_: &Path = fpath_to_path(path);
        filetype = path_to_filetype(path_);
    }

    filetype
}

/// wrapper to call `mimeguess_to_filetype` and if necessary `path_to_filetype`
/// TODO: change name to `path_to_filetype_mimeguess`
pub fn guess_filetype_from_path(path: &Path) -> (FileType, MimeGuess) {
    debug_eprintln!("{}guess_filetype_from_path({:?})", sn(), path);
    let mut mimeguess: MimeGuess = MimeGuess::from_path(path);
    debug_eprintln!("{}guess_filetype_from_path: mimeguess {:?}", so(), mimeguess);
    // Sometimes syslog files get automatically renamed by appending `.old` to the filename,
    // or a number, e.g. `file.log.old`, `kern.log.1`. If so, try MimeGuess without the extra extension.
    if mimeguess.is_empty() && filename_count_extensions(path) > 1 {
        debug_eprintln!("{}guess_filetype_from_path: no mimeguess found, and file name is {:?} (multiple extensions), try again with removed file extension", so(), path.file_name().unwrap_or_default());
        match remove_extension(path) {
            None => {},
            Some(path_) => {
                mimeguess = MimeGuess::from_path(path_);
                debug_eprintln!("{}guess_filetype_from_path: mimeguess #2 {:?}", so(), mimeguess);
            }
        }
    }
    let mut filetype: FileType = mimeguess_to_filetype(&mimeguess);
    debug_eprintln!("{}guess_filetype_from_path: filetype {:?}", so(), filetype);
    if ! parseable_filetype_ok(&filetype) {
        debug_eprintln!("{}guess_filetype_from_path: parseable_filetype_ok({:?}) failed", so(), filetype);
        filetype = path_to_filetype(path);
        // Sometimes syslog files get automatically renamed by appending `.old` to the filename,
        // or a number, e.g. `file.log.old`, `kern.log.1`. If so, try supplement check without extra extension.
        if ! parseable_filetype_ok(&filetype) && filename_count_extensions(path) > 1 {
            debug_eprintln!("{}guess_filetype_from_path: file name is {:?} (multiple extensions), try again with removed file extension", so(), path.file_name().unwrap_or_default());
            match remove_extension(path) {
                None => {},
                Some(path_) => {
                    let std_path_ = fpath_to_path(&path_);
                    filetype = path_to_filetype(std_path_);
                }
            }
        }
    }
    debug_eprintln!("{}guess_filetype_from_path: return ({:?}, {:?})", sx(), filetype, mimeguess);

    (filetype, mimeguess)
}

/// wrapper to call `mimeguess_to_filetype` and if necessary `path_to_filetype`
/// TODO: change name to `fpath_to_filetype_mimeguess`
#[cfg(any(debug_assertions,test))]
pub fn guess_filetype_from_fpath(path: &FPath) -> (FileType, MimeGuess) {
    let path_: &Path = fpath_to_path(path);

    guess_filetype_from_path(path_)
}

/// Return a `ProcessPathResult` for each parseable file within .tar file at `path`
fn process_path_tar(path: &FPath) -> Vec<ProcessPathResult> {
    debug_eprintln!("{}process_path_tar({:?})", sn(), path);

    let file: File = File::open(path).unwrap();
    let mut archive: tar::Archive<File> = tar::Archive::<File>::new(file);
    let entry_iter: tar::Entries<File> = match archive.entries() {
        Ok(val) => val,
        Err(err) => {
            debug_eprintln!("{}process_path_tar: Err {:?}", sx(), err);
            //return Result::Err(err);
            return vec![];
        }
    };
    let mut results = Vec::<ProcessPathResult>::new();
    for entry_res in entry_iter {
        let entry: tar::Entry<File> = match entry_res {
            Ok(val) => val,
            Err(err) => {
                debug_eprintln!("{}process_path_tar: entry Err {:?}", so(), err);
                continue;
            }
        };
        let header = entry.header();
        let etype = header.entry_type();
        debug_eprintln!("{}process_path_tar: entry.header().entry_type() {:?}", so(), etype);
        // TODO: handle tar types `symlink` and `long_link`, currently they are ignored
        if !etype.is_file() {
            continue;
        }
        let subpath: Cow<Path> = match entry.path() {
            Ok(val) => val,
            Err(err) => {
                debug_eprintln!("{}process_path_tar: entry.path() Err {:?}", so(), err);
                continue;
            }
        };
        // first get the `FileType` of the subpath
        let subfpath: FPath = subpath.to_string_lossy().to_string();
        let (filetype_subpath, mimeguess) = guess_filetype_from_path(&subpath);
        // the `FileType` within the tar might be a regular file. It needs to be
        // transformed to corresponding tar `FileType`, so later `BlockReader` understands what to do.
        let filetype: FileType = match filetype_subpath.to_tar() {
            FileType::FILE_UNKNOWN => {
                debug_eprintln!("{}process_path_tar: {:?}.to_tar() is FILE_UNKNOWN", so(), filetype_subpath);
                continue;
            }
            val => val
        };
        if !parseable_filetype_ok(&filetype) {
            debug_eprintln!("{}process_path_tar: push FILE_ERR_NOT_PARSEABLE({:?}, {:?})", so(), filetype, mimeguess);
            results.push(
                ProcessPathResult::FILE_ERR_NOT_PARSEABLE(subfpath, mimeguess)
            );
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
        debug_eprintln!("{}process_path_tar: push FILE_VALID({:?}, {:?}, {:?})", so(), fullpath, mimeguess, filetype);
        results.push(
            ProcessPathResult::FILE_VALID(fullpath, mimeguess, filetype)
        );
    }

    debug_eprintln!("{}process_path_tar({:?})", sx(), path);

    results
}

/// Return all parseable files in the Path.
///
/// Given a directory, recurses the directory.
/// For each recursed file, checks if file is parseable (correct file type,
/// appropriate permissions).
///
/// Given a plain file path, returns that path. This behavior assumes a user-passed
/// file path should attempt to be parsed.
pub fn process_path(path: &FPath) -> Vec<ProcessPathResult> {
    debug_eprintln!("{}process_path({:?})", sn(), path);

    // if passed a path directly to a plain file (or a symlink to a plain file)
    // then assume the user wants to force an attempt to process such a file
    // i.e. do not call `parseable_filetype`
    let std_path: &std::path::Path = std::path::Path::new(path);
    if std_path.is_file() {
        let (filetype, mimeguess) = guess_filetype_from_path(std_path);
        if !filetype.is_archived() {
            let paths: Vec<ProcessPathResult> = vec![
                ProcessPathResult::FILE_VALID(path.clone(), mimeguess, filetype),
            ];
            debug_eprintln!("{}process_path({:?}) {:?}", sx(), path, paths);
            return paths;
        }
        // is_archived
        return process_path_tar(path);
    }

    // getting here means `path` likely refers to a directory

    let mut paths: Vec<ProcessPathResult> = Vec::<ProcessPathResult>::new();

    debug_eprintln!("{}process_path: WalkDir({:?})…", so(), path);
    for entry in walkdir::WalkDir::new(path.as_str())
        .follow_links(true)
        .contents_first(true)
        .sort_by_file_name()
        .same_file_system(true)
    {
        // XXX: what is type `T` in `Result<T, E>` returned by `WalkDir`?
        let path_entry = match entry {
            Ok(val) => {
                debug_eprintln!("{}Ok({:?})", so(), val);
                val
            },
            Err(err) => {
                debug_eprintln!("{}Err({:?})", so(), err);
                continue;
            }
        };

        debug_eprintln!("{}process_path: analayzing {:?}", so(), path_entry);
        let std_path_entry: &std::path::Path = path_entry.path();
        let fpath_entry: FPath = path_to_fpath(std_path_entry);
        if ! path_entry.file_type().is_file() {
            if path_entry.file_type().is_dir() {
                continue;
            }
            debug_eprintln!("{}process_path: Path not a file {:?}", so(), path_entry);
            paths.push(ProcessPathResult::FILE_ERR_NOT_A_FILE(fpath_entry, MimeGuess::from_ext("")));
            continue;
        }
        let (filetype, mimeguess) = guess_filetype_from_path(std_path_entry);
        match parseable_filetype(&filetype) {
            FileParseable::YES => {
                debug_eprintln!("{}process_path: paths.push(FILE_VALID(({:?}, {:?}, {:?})))", so(), fpath_entry, mimeguess, filetype);
                paths.push(ProcessPathResult::FILE_VALID(fpath_entry, mimeguess, filetype));
            },
            FileParseable::NO_NOT_PARSEABLE => {
                debug_eprintln!("{}process_path: Path not parseable {:?}", so(), std_path_entry);
                paths.push(ProcessPathResult::FILE_ERR_NOT_PARSEABLE(fpath_entry, mimeguess));
            }
            FileParseable::NO_NOT_SUPPORTED => {
                debug_eprintln!("{}process_path: Path not supported {:?}", so(), std_path_entry);
                paths.push(ProcessPathResult::FILE_ERR_NOT_SUPPORTED(fpath_entry, mimeguess));
            }
        }
    }
    debug_eprintln!("{}process_path({:?}) {:?}", sx(), path, paths);

    paths
}
