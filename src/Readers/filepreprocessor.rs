// Readers/filepreprocssor.rs
//

use crate::common::{
    FPath,
    FileOffset,
    FileProcessingResult,
    FileType,
};

use crate::Readers::blockreader::{
    BlockIndex,
    BlockOffset,
    BlockSz,
    BlockP,
    ResultS3_ReadBlock,
};

use crate::printer::printers::{
    Color,
    ColorSpec,
    WriteColor,
};

use crate::dbgpr::stack::{
    sn,
    snx,
    so,
    sx,
};

use crate::Data::datetime::{
    FixedOffset,
    DateTimeL,
    DateTimeL_Opt,
};

pub use crate::Readers::linereader::{
    ResultS4_LineFind,
};

pub use crate::Readers::syslinereader::{
    ResultS4_SyslineFind,
    Sysline,
    SyslineP,
    SyslineReader,
};

use crate::Readers::summary::{
    Summary,
};

use std::ffi::OsStr;
use std::fmt;
use std::io::{
    Error,
    Result,
    ErrorKind,
};

extern crate debug_print;
use debug_print::{debug_eprint, debug_eprintln};

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate mime_guess;
use mime_guess::MimeGuess;

extern crate mime_sniffer;
use mime_sniffer::MimeTypeSniffer;  // adds extension method `sniff_mime_type` to `[u8]`

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    assert_ge,
    assert_gt,
    debug_assert_le,
    debug_assert_lt,
    debug_assert_ge,
};

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
//     during preprocessing here. Then pass that along to the `SyslogProcessor` which
//     passes it along to `BlockReader`. Then remove duplicate code from `BlockReader::new`.

#[derive(Debug, Eq, PartialEq)]
pub enum ProcessPathResult {
    FILE_VALID(FPath),
    FILE_ERR_NO_PERMISSIONS(FPath),
    FILE_ERR_NOT_PARSEABLE(FPath),
}

/// map `MimeGuess` into a `FileType`
/// (i.e. call `find_line`)
pub fn parseable_mimeguess_str(mimeguess_str: &str) -> FileType {
    // see https://docs.rs/mime/latest/mime/
    // see https://docs.rs/mime/latest/src/mime/lib.rs.html#572-575
    debug_eprintln!("{}LineReader::parseable_mimeguess_str: mimeguess {:?}", snx(), mimeguess_str);
    match mimeguess_str {
        "plain"
        | "text"
        | "text/plain"
        | "text/*"
        | "utf-8" => {FileType::FILE},
        _ => {FileType::FILE_UNKNOWN},
    }
}

/// can a `LineReader` parse this file/MIME type?
/// (i.e. call `self.find_line()`)
pub fn parseable_mimeguess(mimeguess: &MimeGuess) -> FileType {
    for mimeguess_ in mimeguess.iter() {
        match parseable_mimeguess_str(mimeguess_.as_ref()) {
            FileType::FILE_UNKNOWN | FileType::_FILE_UNSET => {},
            val => { return val; }
        }
    }

    FileType::FILE_UNKNOWN
}

/// reduce `parseable_mimeguess()` to boolean
pub fn parseable_mimeguess_ok(mimeguess: &MimeGuess) -> bool {
    ! matches!(
        parseable_mimeguess(&mimeguess), FileType::FILE_UNKNOWN | FileType::_FILE_UNSET
    )
}

lazy_static! {
    static ref PARSEABLE_FILENAMES: Vec<&'static OsStr> = {
        #[allow(clippy::vec_init_then_push)]
        let mut v = Vec::<&'static OsStr>::with_capacity(8);
        v.push(OsStr::new("messages"));
        v.push(OsStr::new("MESSAGES"));
        v.push(OsStr::new("syslog"));
        v.push(OsStr::new("SYSLOG"));
        v.push(OsStr::new("faillog"));
        v.push(OsStr::new("access_log"));
        v.push(OsStr::new("error_log"));
        v.push(OsStr::new("lastlog"));
        v
    };
}

/// compensates `parseable_mimeguess` for some files not handled by `MimeGuess::from`,
/// like file names without extensions in the name, e.g. `messages` or `syslog`
pub fn parseable_filename(path: &std::path::Path) -> bool {
    if PARSEABLE_FILENAMES.contains(&path.file_name().unwrap_or_default()) {
        return true;
    }
    // many logs have no extension in the name
    if path.extension().is_none() {
        return true;
    }
    // XXX: `file_prefix` WIP https://github.com/rust-lang/rust/issues/86319
    //let file_prefix: &OsStr = &path.file_prefix().unwrap_or_default();
    let file_prefix: &OsStr = &path.file_stem().unwrap_or_default();
    let file_prefix_s = file_prefix.to_str().unwrap_or_default();
    // file name `log.host` as emitted by samba daemon
    if file_prefix_s == "log" {
        return true;
    }
    // file name `log_media`
    if file_prefix_s.starts_with("log_") {
        return true;
    }
    // file name `media_log`
    if file_prefix_s.ends_with("_log") {
        return true;
    }

    false
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
    //let is_parseable: bool = SyslogProcessor::parseable_mimeguess_str(sniff.as_ref());
    let is_parseable = false;

    debug_eprintln!("{}linereader.mimesniff_analysis: return Ok({:?})", sx(), is_parseable);
    Ok(is_parseable)
}
*/

/*
pub(crate) fn mimeguess_analysis(&mut self) -> bool {
    let mimeguess_ = self.mimeguess();
    debug_eprintln!("{}linereader.mimeguess_analysis: mimeguess is {:?}", sn(), mimeguess_);
    let mut is_parseable: bool = false;

    if !mimeguess_.is_empty() {
        // TODO: this function should be moved to filepreprocssor.rs and modified
        //is_parseable = SyslogProcessor::parseable_mimeguess(&mimeguess_);
        debug_eprintln!("{}linereader.mimeguess_analysis: parseable_mimeguess {:?}", sx(), is_parseable);
        return is_parseable;
    }
    debug_eprintln!("{}linereader.mimeguess_analysis: {:?}", sx(), is_parseable);

    is_parseable
}
*/

/// Return all parseable files in the Path.
/// Given a directory, recurses the directory.
/// for each recursed file, checks if file is parseable (correct file type, appropriate permissions).
/// Given a plain file path, returns that path. This behavior assumes the user-passed
/// file path should attempt to be parsed.
pub fn process_fpath(path: &FPath) -> Vec<ProcessPathResult> {
    debug_eprintln!("{}process_fpath({:?})", sn(), path);

    // if passed a path directly to a plain file (symlink to a plain file)
    // then assume the user wants to force an attempt to process such a file
    let p_ = std::path::Path::new(path);
    if p_.is_file() {
        let paths: Vec<ProcessPathResult> = vec![
            ProcessPathResult::FILE_VALID(path.clone()),
        ];
        return paths;
    }

    // getting here means `path` likely refers to a directory

    let mut paths: Vec<ProcessPathResult> = Vec::<ProcessPathResult>::new();

    debug_eprintln!("{}process_fpath: WalkDir({:?})...", so(), path);
    for entry in walkdir::WalkDir::new(path.as_str())
        .follow_links(true)
        .contents_first(true)
        .sort_by_file_name()
        .same_file_system(true)
    {
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
        // `PathBuf` to `String`
        // https://stackoverflow.com/q/37388107/471376
        let std_path = path_.path();
        let fpath: FPath = (*(std_path.to_string_lossy())).to_string();
        if ! path_.file_type().is_file() {
            debug_eprintln!("{}process_fpath: Path not a file {:?}", so(), path_);
            //paths.push(ProcessPathResult::FILE_ERR_NOT_PARSEABLE(fpath));
            continue;
        }
        let mimeguess: MimeGuess = MimeGuess::from_path(std_path);
        debug_eprintln!("{}process_fpath: {:?} for {:?}", so(), mimeguess, std_path);
        if ! parseable_mimeguess_ok(&mimeguess) && ! parseable_filename(&std_path) {
            debug_eprintln!("{}process_fpath: Path MIME type not parseable {:?} for {:?}", so(), mimeguess, std_path);
            //paths.push(ProcessPathResult::FILE_ERR_NOT_PARSEABLE(fpath));
            continue;
        }
        debug_eprintln!("{}process_fpath: paths.push(FILE_VALID({:?}))", so(), fpath);
        paths.push(ProcessPathResult::FILE_VALID(fpath));
    }
    debug_eprintln!("{}process_fpath({:?})", sx(), path);

    paths
}
