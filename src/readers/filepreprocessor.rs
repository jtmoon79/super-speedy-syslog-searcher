// src/readers/filepreprocssor.rs

//! A collection of functions to search for potentially parseable files,
//! and prepare data needed to create a [`SyslogProcessor`] instance or other
//! "Reader" instance for file processing.
//!
//! [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor

use crate::common::{FPath, FileType};
use crate::readers::blockreader::SUBPATH_SEP;
use crate::readers::helpers::{
    filename_count_extensions,
    fpath_to_path,
    path_clone,
    path_to_fpath,
    remove_extension,
};

use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

#[doc(hidden)]
pub use ::mime_guess::MimeGuess;
#[allow(unused_imports)]
use ::si_trace_print::{defn, defo, defx, defñ, den, deo, dex, deñ};
use ::tar;

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
    // TODO: [2022/06] `FileErrNoPermissions` not currently checked until too late
    /// Filesystem permissions do not allow reading the file
    FileErrNoPermissions(FPath, MimeGuess),
    /// File is a known or unknown type and is not supported
    FileErrNotSupported(FPath, MimeGuess),
    /// Path exists and is not a file
    FileErrNotAFile(FPath),
    /// Path does not exist
    FileErrNotExist(FPath),
    /// Error loading necessary shared libraries
    FileErrLoadingLibrary(FPath, &'static str, FileType),
}

pub type ProcessPathResults = Vec<ProcessPathResult>;

/// files without file extensions known to be parseable
const PARSEABLE_FILENAMES_FILE: [&str; 3] = [
    "messages",
    "syslog",
    "kernlog",
];

/// [utmpx format] file names.
///
/// [utmpx format]: https://en.wikipedia.org/w/index.php?title=Utmp&oldid=1143772537#Location
const UTMP_FILENAMES_FILE: [&str; 9] = [
    // Linux, HP-UX
    "btmp",
    "utmp",
    "wtmp",
    // Solaris
    "btmpx",
    "utmpx",
    "wtmpx",
    // FreeBSD
    "utx.active",
    "utx.lastlogin",
    // `utx.log` is an ambiguous name, could be a plain log file or utmpx file
    "utx.log",
];

/// [evtx format] file name extensions.
///
/// [evtx format]: https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc
const EVTX_FILENAMES_EXT: [&str; 1] = [
    "evtx",
];

/// journal format [file name extensions].
///
/// [file name extensions]: https://github.com/systemd/systemd/blob/v249/src/libsystemd/sd-journal/journal-file.c#L3774
const JOURNAL_FILENAMES_EXT: [&str; 1] = [
    "journal",
];

/// Odd strings that are appended to normal files by various programs and
/// services.
///
/// For example, `~` is appended to some .journal files in OpenSUSE Tumbleweed,
/// e.g. `system.journal~`.
const EXT_JUNK_APPEND_STR: [&str; 2] = [
    "~",
    "-",
];

/// Map a single [`MimeGuess`] as a [`str`] into a `FileType`.
///
/// Mimetype values are in [`mime_types.rs`].
///
/// [`MimeGuess`]: https://docs.rs/mime_guess/2.0.4/mime_guess/struct.MimeGuess.html
/// [`mime_types.rs`]: https://docs.rs/crate/mime_guess/2.0.4/source/src/mime_types.rs
pub fn mimeguess_to_filetype_str(mimeguess_str: &str) -> FileType {
    // see https://docs.rs/mime/latest/mime/
    // see https://docs.rs/mime/latest/src/mime/lib.rs.html
    // see https://github.com/abonander/mime_guess/blob/f6d36d8531bef9ad86f3ee274f65a1a31ea4d9b4/src/mime_types.rs
    defñ!("({:?})", mimeguess_str);
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
    // known unparseable log file types
    const APP_TARGZ: &str = "application/x-compressed";
    const APP_ETL: &str = "application/etl";
    const APP_ZIP: &str = "application/zip";
    const APP_BZ: &str = "application/x-bzip";
    const APP_BZ2: &str = "application/x-bzip2";

    match lower.as_str() {
        PLAIN | TEXT | TEXT_PLAIN | TEXT_PLAIN_UTF8 | TEXT_STAR | UTF8_ => FileType::File,
        APP_GZIP | APP_XGZIP => FileType::Gz,
        APP_X_XZ => FileType::Xz,
        APP_TAR | APP_GTAR => FileType::Tar,
        // XXX: `.targz` is an odd case because currently it has it's own
        //      `FileType` but is still not supported.
        //      This was due to overplanning.
        //      See Issue #14
        APP_TARGZ
        // Support for `.bz` and `.bz2` is Issue #40
        | APP_BZ
        | APP_BZ2
        // Support for `.etl` is Issue #99
        | APP_ETL
        // Support for `.zip` is Issue #39
        | APP_ZIP => FileType::Unparseable,
        _ => FileType::Unknown,
    }
}

/// Given multiple [`MimeGuess`] try to map any to a parseable `FileType`.
/// Attempt to preserve known unparseable files.
///
/// [`MimeGuess`]: https://docs.rs/mime_guess/2.0.4/mime_guess/struct.MimeGuess.html
pub fn mimeguess_to_filetype(mimeguess: &MimeGuess) -> FileType {
    defn!("mimeguess_to_filetype({:?})", mimeguess);
    let mut filetype_un: FileType = FileType::Unknown;
    for mimeguess_ in mimeguess.iter() {
        deo!("mimeguess_to_filetype: check {:?}", mimeguess_);
        match mimeguess_to_filetype_str(mimeguess_.as_ref()) {
            FileType::Unset => {}
            FileType::Unparseable => {
                filetype_un = FileType::Unparseable;
            }
            val => {
                defx!("mimeguess_to_filetype: return {:?}", val);
                return val;
            }
        }
    }

    defx!("mimeguess_to_filetype: return {:?}", filetype_un);

    filetype_un
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
    defn!("({:?})", path);

    let file_name: &OsStr = path
        .file_name()
        .unwrap_or_default();
    deo!("file_name {:?}", file_name);
    let file_name_string: String = file_name
        .to_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let file_name_s: &str = file_name_string.as_str();
    deo!("file_name_s {:?}", file_name_s);

    if PARSEABLE_FILENAMES_FILE.contains(&file_name_s) {
        defx!("return File; PARSEABLE_FILENAMES_FILE.contains({:?})", file_name_s);
        return FileType::File;
    }

    if UTMP_FILENAMES_FILE.contains(&file_name_s) {
        defx!("return Utmpx; UTMP_FILENAMES_FILE.contains({:?})", file_name_s);
        return FileType::Utmpx;
    }

    // XXX: `file_prefix` WIP https://github.com/rust-lang/rust/issues/86319
    //let file_prefix: &OsStr = &path.file_prefix().unwrap_or_default();
    let file_prefix = path
        .file_stem()
        .unwrap_or_default();
    let file_prefix_string: String = file_prefix
        .to_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let file_prefix_s: &str = file_prefix_string.as_str();
    deo!("file_prefix {:?}", file_prefix_s);

    let file_suffix: &OsStr = path
        .extension()
        .unwrap_or_default();
    let file_suffix_string: String = file_suffix
        .to_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let file_suffix_s = file_suffix_string.as_str();
    deo!("file_suffix {:?}", file_suffix_s);

    if EVTX_FILENAMES_EXT.contains(&file_suffix_s) {
        defx!("return Evtx; EVTX_FILENAMES_EXT.contains({:?})", file_suffix_s);
        return FileType::Evtx;
    }

    if JOURNAL_FILENAMES_EXT.contains(&file_suffix_s) {
        defx!("return Journal; JOURNAL_FILENAMES_EXT.contains({:?})", file_suffix_s);
        return FileType::Journal;
    }

    if UTMP_FILENAMES_FILE.contains(&file_suffix_s) {
        defx!("return Utmpx; UTMP_FILENAMES_FILE.contains({:?})", file_suffix_s);
        return FileType::Utmpx;
    }

    // FileTgz (returns `Unparseable`)
    // Known to be unparseable. Someday it should be supported. Issue #14

    // for example `data.tgz`
    // XXX: this should be handled in `path_to_filetype_mimeguess`
    if file_suffix_s == "tgz" {
        defx!("return Unparseable; .tgz");
        return FileType::Unparseable;
    }

    // for example `data.tgz.old`
    if file_prefix_s.ends_with(".tgz") {
        defx!("return Unparseable; data.tgz");
        return FileType::Unparseable;
    }

    // FileGz

    // for example, `media.gz.old`
    if file_name_s.ends_with(".gz.old") {
        defx!("return Gz; .gz.old");
        return FileType::Gz;
    }
    // for example, `media.gzip`
    if file_suffix_s == "gzip" {
        defx!("return Gz; .gzip");
        return FileType::Gz;
    }
    // for example, `media.gz`
    // XXX: this should be handled in `path_to_filetype_mimeguess`
    if file_suffix_s == "gz" {
        defx!("return Gz; .gz");
        return FileType::Gz;
    }

    // FileXz

    // for example, `media.xz.old`
    if file_name_s.ends_with(".xz.old") {
        defx!("return Xz; .xz.old");
        return FileType::Xz;
    }
    // for example, `media.xzip`
    if file_suffix_s == "xzip" {
        defx!("return Xz; .xzip");
        return FileType::Xz;
    }
    // for example, `media.xz`
    // XXX: this should be handled in `path_to_filetype_mimeguess`
    if file_suffix_s == "xz" {
        defx!("return Xz; .xz");
        return FileType::Xz;
    }

    // FileTar

    // for example, `var-log.tar.old`
    if file_name_s.ends_with(".tar.old") {
        defx!("return Tar; .tar.old");
        return FileType::Tar;
    }
    // XXX: this should be handled in `path_to_filetype_mimeguess`
    if file_suffix_s == "tar" {
        defx!("return Tar; .tar");
        return FileType::Tar;
    }

    // File

    // for example, `log_media`
    if file_prefix_s.starts_with("log_") {
        defx!("return File; log_");
        return FileType::File;
    }
    // for example, `media_log`
    if file_name_s.ends_with("_log") {
        defx!("return File; _log");
        return FileType::File;
    }
    // for example, `media.log.old`
    if file_name_s.ends_with(".log.old") {
        defx!("return File; .log.old");
        return FileType::File;
    }

    // other misc. file patterns

    // for example, `syslog.2`
    if file_prefix_s == "syslog" {
        defx!("return File; syslog.");
        return FileType::File;
    }

    // for example, `syslog`
    if file_name_s == "syslog" {
        defx!("return File; syslog");
        return FileType::File;
    }

    if file_name_s == "kernellog" {
        defx!("return File; kernellog");
        return FileType::File;
    }

    if file_name_s == "kernelog" {
        defx!("return File; kernelog");
        return FileType::File;
    }

    // for example, `messages.2`
    if file_prefix_s == "messages" {
        defx!("return File; messages.");
        return FileType::File;
    }

    // for example, `dmesg.2`
    if file_prefix_s == "dmesg" {
        defx!("return File; dmesg.");
        return FileType::File;
    }

    // for example, `dmesg`
    if file_name_s == "dmesg" {
        defx!("return File; dmesg");
        return FileType::File;
    }

    // for example, `log.host` as emitted by samba daemon
    if file_name_s.starts_with("log.") {
        defx!("return File; log..");
        return FileType::File;
    }

    // for example, `log.log.host`
    if file_prefix_s == "log" {
        defx!("return File; log.");
        return FileType::File;
    }

    // on cheap embedded systems it may be just named `log`
    if file_name_s == "log" {
        defx!("return File; log");
        return FileType::File;
    }

    // some logs have no extension in the name
    if path.extension().is_none() {
        defx!("return File; no path.extension()");
        return FileType::File;
    }

    defx!("return Unknown");

    FileType::Unknown
}

/// Wrapper function for `path_to_filetype`
#[doc(hidden)]
#[cfg(test)]
pub fn fpath_to_filetype(path: &FPath) -> FileType {
    path_to_filetype(fpath_to_path(path))
}

/// Is the `FileType` processing implemented by `s4lib`?
///
/// There are plans for future support of differing files.
pub fn processable_filetype(filetype: &FileType) -> bool {
    defñ!("({:?})", filetype);
    match filetype {
        &FileType::Unknown | &FileType::Unset => false,
        // `FileType::Unparseable` is not parseable but
        // but is explicitly recognized as such.
        _ => true,
    }
}

/// Make an effort to determine a file's `FileType`.
/// Wrapper function to call `mimeguess_to_filetype` and if necessary
/// `path_to_filetype`.
/// Users should prefer this function and not those other functions.
pub fn path_to_filetype_mimeguess(path: &Path) -> (FileType, MimeGuess) {
    defn!("({:?})", path);

    // first, try to determine the mimetype
    let mut mimeguess: MimeGuess = MimeGuess::from_path(path);
    deo!("mimeguess {:?}", mimeguess);

    const RM_LIMIT: i32 = 3;

    if mimeguess.is_empty() {
        // Sometimes syslog files get automatically renamed by appending `.old`
        // to the filename, or a number, e.g. `file.log.old`, `kern.log.1`.
        // If so, try MimeGuess without the extra extensions.
        // However, limit attempts to `RM_LIMIT` as some files could have names
        // like `this.is.a.long.name.of.a.file.with.dots`.
        let mut fpath: FPath;
        let mut path_: &Path = path_clone(path);
        let mut ext_rm = 0;
        while mimeguess.is_empty() && filename_count_extensions(path_) != 0 && ext_rm < RM_LIMIT {
            // some files have junk appended to the filename, e.g. `system.journal~`
            // so remove the junk characters and try again
            for junk_end in EXT_JUNK_APPEND_STR.iter() {
                if path_
                    .extension()
                    .unwrap_or(path_.file_name().unwrap_or_default())
                    .to_str()
                    .unwrap_or_default()
                    .ends_with(junk_end)
                {
                    defo!("no mimeguess found, try again with removed {:?}", junk_end);
                    let fpath2 = path_to_fpath(path_).trim_end_matches(junk_end).to_string();
                    let path2 = fpath_to_path(&fpath2);
                    mimeguess = MimeGuess::from_path(path2);
                    defo!("mimeguess {:?}", mimeguess);
                    if !mimeguess.is_empty() {
                        break;
                    }
                }
            }
            if !mimeguess.is_empty() {
                break;
            }
            match remove_extension(path_) {
                None => {}
                Some(fpath_rm1ext) => {
                    defo!("no mimeguess found, try again with removed file extension {:?}", fpath_rm1ext);
                    fpath = fpath_rm1ext;
                    path_ = fpath_to_path(&fpath);
                    mimeguess = MimeGuess::from_path(path_);
                    defo!("mimeguess {:?}", mimeguess);
                }
            }
            ext_rm += 1;
        }
    }

    // second, use the mimetype to determine the file type
    let mut filetype: FileType = mimeguess_to_filetype(&mimeguess);

    match filetype {
        FileType::Unknown | FileType::Unset => {
            // The filetype still could not be determined so try removing
            // extensions from the name. Sometimes syslog files get
            // automatically renamed by appending signifiers like `.old`.
            // Files can have several signifiers like `file.log.old.2`
            // or characters appended like `file.log~`.
            // However, limit attempts to `RM_LIMIT` as some files could
            // have names like `this.is.a.long.name.of.a.file.with.dots`.
            defo!("filetype '{:?}' is not parseable", filetype);
            let mut fpath: FPath;
            let mut path_: &Path = path_clone(path);
            filetype = path_to_filetype(path_);
            defo!("filetype {:?}", filetype);
            let mut ext_rm = 0;
            while !processable_filetype(&filetype) && filename_count_extensions(path_) != 0 && ext_rm < RM_LIMIT {
                // some files have junk appended to the filename, e.g. `system.journal~`
                // so remove the junk characters and try again
                for junk_end in EXT_JUNK_APPEND_STR.iter() {
                    if path_
                        .extension()
                        .unwrap_or(path_.file_name().unwrap_or_default())
                        .to_str()
                        .unwrap_or_default()
                        .ends_with(junk_end)
                    {
                        defo!("no filetype found, try again with removed {:?}", junk_end);
                        let fpath2 = path_to_fpath(path_).trim_end_matches(junk_end).to_string();
                        let path2 = fpath_to_path(&fpath2);
                        filetype = path_to_filetype(path2);
                        defo!("filetype {:?}", filetype);
                        if processable_filetype(&filetype) {
                            break;
                        }
                    }
                }
                if processable_filetype(&filetype) {
                    break;
                }
                match remove_extension(path_) {
                    None => {}
                    Some(fpath_rm1ext) => {
                        defo!("no filetype found, try again with removed file extension {:?}", fpath_rm1ext);
                        fpath = fpath_rm1ext;
                        path_ = fpath_to_path(&fpath);
                        filetype = path_to_filetype(path_);
                        defo!("filetype {:?}", filetype);
                    }
                }
                ext_rm += 1;
            }
        }
        _ => {}
    }

    defx!("return ({:?}, {:?})", filetype, mimeguess);

    (filetype, mimeguess)
}

/// Wrapper function to call `mimeguess_to_filetype` and if necessary
/// `path_to_filetype`
#[doc(hidden)]
#[cfg(test)]
pub(crate) fn fpath_to_filetype_mimeguess(path: &FPath) -> (FileType, MimeGuess) {
    let path_: &Path = fpath_to_path(path);

    path_to_filetype_mimeguess(path_)
}

/// Return a `ProcessPathResult` for each parseable file within
/// the `.tar` file at `path`.
pub fn process_path_tar(path: &FPath) -> Vec<ProcessPathResult> {
    defn!("({:?})", path);

    let file: File = File::open(path).unwrap();
    let mut archive: tar::Archive<File> = tar::Archive::<File>::new(file);
    let entry_iter: tar::Entries<File> = match archive.entries() {
        Ok(val) => val,
        Err(_err) => {
            defx!("Err {:?}", _err);
            //return Result::Err(err);
            return vec![];
        }
    };
    let mut results = Vec::<ProcessPathResult>::new();
    for entry_res in entry_iter {
        let entry: tar::Entry<File> = match entry_res {
            Ok(val) => val,
            Err(_err) => {
                deo!("entry Err {:?}", _err);
                continue;
            }
        };
        let header: &tar::Header = entry.header();
        let etype: tar::EntryType = header.entry_type();
        deo!("entry.header().entry_type() {:?}", etype);
        // TODO: handle tar types `symlink` and `long_link`, currently they are ignored
        if !etype.is_file() {
            continue;
        }
        let subpath: Cow<Path> = match entry.path() {
            Ok(val) => val,
            Err(_err) => {
                deo!("entry.path() Err {:?}", _err);
                continue;
            }
        };
        // first get the `FileType` of the subpath
        let subfpath: FPath = subpath
            .to_string_lossy()
            .to_string();
        let _filetype_subpath: FileType;
        let mimeguess: MimeGuess;
        (_filetype_subpath, mimeguess) = path_to_filetype_mimeguess(&subpath);
        // path to a file within a .tar file looks like:
        //
        //     "path/file.tar|subpath/subfile"
        //
        // where `path/file.tar` are on the host filesystem, and `subpath/subfile` are within
        // the `.tar` file
        let mut fullpath: FPath =
            String::with_capacity(path.len() + SUBPATH_SEP.len_utf8() + subfpath.len() + 1);
        fullpath.push_str(path.as_str());
        fullpath.push(SUBPATH_SEP);
        fullpath.push_str(subfpath.as_str());
        // XXX: force filetype to be `Tar` (ignore `_filetype_subpath`). Later an attempt
        //      will be made to parse it.
        //      Chained reads are not supported. See Issue #14
        deo!("push FileValid({:?}, {:?}, {:?})", fullpath, mimeguess, FileType::Tar);
        results.push(ProcessPathResult::FileValid(fullpath, mimeguess, FileType::Tar));
    }

    defx!("process_path_tar({:?})", path);

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
    defn!("({:?})", path);

    let std_path: &Path = Path::new(path);
    if !std_path.exists() {
        defx!("return FileErrNotExist({:?})", path);
        return vec![ProcessPathResult::FileErrNotExist(path.clone())];
    }

    // if passed a path directly to a plain file (or a symlink to a plain file)
    // and `force` then assume the user wants to force an attempt to process
    // such a file (even if it's known to be unparseable, e.g. `picture.png`)
    // so skip call to `parseable_filetype` and treat is as `FileValid`
    if std_path.is_file() {
        let filetype: FileType;
        let mimeguess: MimeGuess;
        (filetype, mimeguess) = path_to_filetype_mimeguess(std_path);
        if filetype.is_archived() && filetype.is_supported() {
            // is_archived
            defñ!("return process_path_tar({:?})", path);
            return process_path_tar(path);
        }
        let paths: Vec<ProcessPathResult> =
            vec![ProcessPathResult::FileValid(path.clone(), mimeguess, filetype)];
        defx!("({:?}) {:?}", path, paths);
        return paths;
    }

    // getting here means `path` likely refers to a directory

    let mut paths: Vec<ProcessPathResult> = Vec::<ProcessPathResult>::new();

    deo!("WalkDir({:?})…", path);
    for entry in walkdir::WalkDir::new(path.as_str())
        .follow_links(true)
        .contents_first(true)
        .sort_by_file_name()
        .same_file_system(false)
    {
        // XXX: what is type `T` in `Result<T, E>` returned by `WalkDir`?
        let path_entry = match entry {
            Ok(val) => {
                deo!("Ok({:?})", val);
                val
            }
            Err(_err) => {
                deo!("Err({:?})", _err);
                continue;
            }
        };

        deo!("analayzing {:?}", path_entry);
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
            deo!("Path not a file {:?}", path_entry);
            paths.push(ProcessPathResult::FileErrNotAFile(fpath_entry));
            continue;
        }
        let filetype: FileType;
        let mimeguess: MimeGuess;
        (filetype, mimeguess) = path_to_filetype_mimeguess(std_path_entry);
        if filetype.is_archived() && filetype.is_supported() {
            // is_archived
            defo!("process_path_tar({:?})", std_path_entry);
            for result in process_path_tar(&fpath_entry).into_iter() {
                paths.push(result);
            }
            continue;
        }
        match filetype {
            FileType::TarGz | FileType::Unparseable => {
                deo!("Path not supported {:?}", std_path_entry);
                paths.push(ProcessPathResult::FileErrNotSupported(fpath_entry, mimeguess));
            }
            FileType::Unset => {
                eprintln!("ERROR: filetype {:?} for {:?}", filetype, std_path_entry);
            }
            _ => {
                deo!("paths.push(FileValid(({:?}, {:?}, {:?})))", fpath_entry, mimeguess, filetype);
                paths.push(ProcessPathResult::FileValid(fpath_entry, mimeguess, filetype));
            }
        }
    }
    defx!("return {:?}", paths);

    paths
}
