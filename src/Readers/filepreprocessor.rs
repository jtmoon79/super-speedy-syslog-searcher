// Readers/filepreprocssor.rs
//
// a collection of functions to search for potentially parseable files,
// and prepare for the creation of `SyslogProcessor`

use crate::common::{
    FPath,
    FileType,
};

use crate::Readers::helpers::{
    path_to_fpath,
    fpath_to_path,
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

use std::ffi::OsStr;

extern crate debug_print;
use debug_print::debug_eprintln;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate mime_guess;
use mime_guess::MimeGuess;

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
    FILE_VALID((FileType, FPath)),
    FILE_ERR_NO_PERMISSIONS(FPath),
    FILE_ERR_NOT_PARSEABLE(FPath),
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
    // see https://docs.rs/mime/latest/src/mime/lib.rs.html#572-575
    debug_eprintln!("{}mimeguess_to_filetype_str: mimeguess {:?}", snx(), mimeguess_str);
    match mimeguess_str {
        "plain"
        | "text"
        | "text/plain"
        | "text/*"
        | "utf-8" => FileType::FILE,
        "application/gzip" => FileType::FILE_GZ,
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
pub fn path_to_filetype(path: &std::path::Path) -> FileType {
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
    if file_suffix_s.ends_with(".gz.old") {
        debug_eprintln!("{}path_to_filetype: return FILE_GZ; .gz.old", sx());
        return FileType::FILE_GZ;
    }
    // for example, `media.gzip`
    if file_suffix_s.ends_with(".gzip") {
        debug_eprintln!("{}path_to_filetype: return FILE_GZ; .gzip", sx());
        return FileType::FILE_GZ;
    }

    debug_eprintln!("{}path_to_filetype: return FILE_UNKNOWN", sx());

    FileType::FILE_UNKNOWN
}

/// wrapper for `path_to_filetype`
#[cfg(any(debug_assertions,test))]
pub fn fpath_to_filetype(path: &FPath) -> FileType {
    path_to_filetype(fpath_to_path(path))
}

/// is `FileType` supported?
pub fn parseable_filetype(filetype: &FileType) -> bool {
    matches!(
        filetype,
        // effectively the list of currently supported file types
        &FileType::FILE
        | &FileType::FILE_GZ
    )
}

/// reduce `mimeguess_to_filetype()` to a boolean
#[cfg(any(debug_assertions,test))]
pub fn mimeguess_to_filetype_ok(mimeguess: &MimeGuess) -> bool {
    parseable_filetype(&mimeguess_to_filetype(mimeguess))
}

/// wrapper to call `mimeguess_to_filetype` and if necessary `path_to_filetype`
#[cfg(any(debug_assertions,test))]
pub fn guess_filetype_from_mimeguess_path(mimeguess: &MimeGuess, path: &std::path::Path) -> FileType {
    let mut filetype: FileType = mimeguess_to_filetype(mimeguess);
    if ! parseable_filetype(&filetype) {
        filetype = path_to_filetype(path);
    }

    filetype
}

/// wrapper to call `mimeguess_to_filetype` and if necessary `path_to_filetype`
#[cfg(any(debug_assertions,test))]
pub fn guess_filetype_from_mimeguess_fpath(mimeguess: &MimeGuess, path: &FPath) -> FileType {
    let mut filetype: FileType = mimeguess_to_filetype(mimeguess);
    if ! parseable_filetype(&filetype) {
        let path_: &std::path::Path = fpath_to_path(path);
        filetype = path_to_filetype(path_);
    }

    filetype
}

/// wrapper to call `mimeguess_to_filetype` and if necessary `path_to_filetype`
pub fn guess_filetype_from_path(path: &std::path::Path) -> FileType {
    let mimeguess: MimeGuess = MimeGuess::from_path(path);
    let mut filetype: FileType = mimeguess_to_filetype(&mimeguess);
    if ! parseable_filetype(&filetype) {
        filetype = path_to_filetype(path);
    }

    filetype
}

/// wrapper to call `mimeguess_to_filetype` and if necessary `path_to_filetype`
#[cfg(any(debug_assertions,test))]
pub fn guess_filetype_from_fpath(path: &FPath) -> FileType {
    let path_: &std::path::Path = fpath_to_path(path);

    guess_filetype_from_path(path_)
}

/*
pub(crate) fn mimesniff_analysis(&mut self) -> Result<bool> {
    let bo_zero: FileOffset = 0;
    debug_eprintln!("{}linereader.mimesniff_analysis: self.blockreader.read_block({:?})", sn(), bo_zero);
    let bptr: BlockP = match self.syslinereader.linereader.blockreader.read_block(bo_zero) {
        ResultS3_ReadBlock::Found(val) => val,
        ResultS3_ReadBlock::Done => {
            debug_eprintln!("{}linereader.mimesniff_analysis: read_block({}) returned Done for {:?}, return Error(UnexpectedEof)", sx(), bo_zero, self.path());
            assert_eq!(self.filesz(), 0, "readblock(0) returned Done for file with size {}", self.filesz());
            return Ok(false);
        },
        ResultS3_ReadBlock::Err(err) => {
            debug_eprintln!("{}linereader.mimesniff_analysis: read_block({}) returned Err {:?}", sx(), bo_zero, err);
            return Result::Err(err);
        },
    };

    let sniff: String = String::from((*bptr).as_slice().sniff_mime_type().unwrap_or(""));
    debug_eprintln!("{}linereader.mimesniff_analysis: sniff_mime_type {:?}", so(), sniff);
    // TODO: this function should be moved to filepreprocssor.rs and modified
    //let is_parseable: bool = SyslogProcessor::mimeguess_to_filetype_str(sniff.as_ref());
    let is_parseable = false;

    debug_eprintln!("{}linereader.mimesniff_analysis: return Ok({:?})", sx(), is_parseable);
    Ok(is_parseable)
}
*/

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
    let std_path = std::path::Path::new(path);
    if std_path.is_file() {
        let filetype: FileType = guess_filetype_from_path(&std_path);
        let paths: Vec<ProcessPathResult> = vec![
            ProcessPathResult::FILE_VALID((filetype, path.clone())),
        ];
        debug_eprintln!("{}process_path({:?}) {:?}", sx(), path, paths);
        return paths;
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
        let path_ = match entry {
            Ok(val) => {
                debug_eprintln!("{}Ok({:?})", so(), val);
                val
            },
            Err(err) => {
                debug_eprintln!("{}Err({:?})", so(), err);
                continue;
            }
        };
        if ! path_.file_type().is_file() {
            debug_eprintln!("{}process_path: Path not a file {:?}", so(), path_);
            //paths.push(ProcessPathResult::FILE_ERR_NOT_PARSEABLE(fpath));
            continue;
        }

        let std_path_: &std::path::Path = path_.path();
        let filetype: FileType = guess_filetype_from_path(&std_path);
        if ! parseable_filetype(&filetype) {
            debug_eprintln!("{}process_path: Path not parseable {:?}", so(), std_path);
            //paths.push(ProcessPathResult::FILE_ERR_NOT_PARSEABLE(fpath));
            continue;
        }
        let fpath: FPath = path_to_fpath(std_path_);
        debug_eprintln!("{}process_path: paths.push(FILE_VALID(({:?}, {:?})))", so(), filetype, fpath);
        paths.push(ProcessPathResult::FILE_VALID((filetype, fpath)));
    }
    debug_eprintln!("{}process_path({:?}) {:?}", sx(), path, paths);

    paths
}


