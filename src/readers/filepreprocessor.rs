// src/readers/filepreprocssor.rs

//! A collection of functions to search for potentially parseable files,
//! and prepare data needed to create a [`SyslogProcessor`] instance or other
//! "Reader" instance for file processing.
//!
//! [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor

use std::borrow::Cow;
use std::collections::{
    HashMap,
    HashSet,
};
use std::ffi::OsStr;
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};
#[cfg(test)]
use std::str::FromStr; // for `String::from_str`
use std::sync::LazyLock;

use ::chardet::detect;
use ::const_format::concatcp;
use ::jwalk;
#[allow(unused_imports)]
use ::si_trace_print::{
    defn,
    defo,
    defx,
    defñ,
    def1n,
    def1o,
    def1x,
    den,
    deo,
    dex,
    deñ,
};
use ::tar;
use ::zip;

use crate::common::{
    AulDetection,
    AulStorageType,
    BOM_UTF16BE,
    BOM_UTF16LE,
    BOM_UTF32BE,
    BOM_UTF32LE,
    BOM_UTF8,
    FPath,
    FileSz,
    FileType,
    FileTypeArchive,
    FileTypeFixedStruct,
    FileTypeTextEncoding,
    OdlSubType,
    SUBPATH_SEP,
    SUBPATH_SEP_DISPLAY,
    SUBPATH_SEP_DISPLAY_ALT,
};
use crate::debug_panic;
use crate::debug::printers::de_err;
#[cfg(any(debug_assertions, test))]
use crate::readers::helpers::fpath_to_path;
use crate::readers::helpers::{
    path_to_fpath,
    zip_error,
};

#[inline(always)]
fn is_texty_ascii_byte(b: u8) -> bool {
    b == b'\n' || b == b'\r' || b == b'\t' || (0x20..=0x7E).contains(&b)
}

const SCORE_NO_MATCH: u8 = 0;
const SCORE_DIRECT_MIN: u8 = 114; // 45% mapped into 0..=255.
const SCORE_BOM_ONLY: u8 = 240;
const SCORE_UTF8_BOM: u8 = 250;
const SCORE_VALID_UTF8: u8 = 220;
const SCORE_UTF8_FAST_PATH_MIN: u8 = 1;

/// unparseable files by name.
///
/// miscellaneous one-name files I found on various systems; admittedly a
/// haphazard list.
pub const NAME_UNPARSEABLE: [&str; 109] = [
    ".DS_Store",
    "asset",
    "AUTHORS.txt",
    "AUTHORS",
    "BookmarkMergedSurfaceOrdering",
    "BrowsingTopicsSiteData-journal",
    "BrowsingTopicsSiteData",
    "cache.db",
    "changelog.md",
    "CHANGELOG.md",
    "Cookies-journal",
    "Cookies",
    "customSettings",
    "db-journal",
    "dep_analysis_cache_v2-shm",
    "dep_analysis_cache_v2-wal",
    "dep_analysis_cache_v2",
    "Device Bound Sessions-journal",
    "Device Bound Sessions",
    "DIPS-journal",
    "DIPS-wal",
    "Dockerfile",
    "downloadCache_",
    "downloadCache-journal",
    "downloadCache",
    "EdgeEDropSQLite.db-journal",
    "EdgeEDropSQLite.db-shm",
    "EdgeEDropSQLite.db-wal",
    "EdgeEDropSQLite.db",
    "EdgeHubAppUsageSQLite.db",
    "EdgeJourneys.db-journal",
    "EdgeJourneys.db-shm",
    "EdgeJourneys.db-shm",
    "EdgeJourneys.db-wal",
    "EdgeJourneys.db-wal",
    "EdgeJourneys.db",
    "EdgeJourneys.db",
    "edgeSettings",
    "EncryptedBookmarks2",
    "index-of.js",
    "index.d.cts",
    "index.d.mts",
    "index.d.ts.map",
    "index.d.ts",
    "index.js",
    "keys.js",
    "language-configuration.json",
    "Last Browser-journal",
    "Last Browser",
    "Last Version-journal",
    "Last Version",
    "license.md",
    "LICENSE.md",
    "license",
    "LICENSE",
    "Login Data For Account-journal",
    "Login Data For Account",
    "Login Data-journal",
    "Login Data",
    "MANIFEST-000001",
    "manifest.json",
    "manifest",
    "mirror_metadata_update_db-shm",
    "mirror_metadata_update_db-wal",
    "mirror_metadata_update_db",
    "Network Action Predictor-journal",
    "Network Action Predictor",
    "Network Persistent State",
    "NetworkDataMigrated-journal",
    "NetworkDataMigrated",
    "README",
    "README.md",
    "scalar.js",
    "Scalar.js",
    "SCT Auditing Pending Reports",
    "Secure Preferences",
    "ServerCertificate-journal",
    "ServerCertificate",
    "session.db-shm",
    "session.db-wal",
    "session.db",
    "settings.dat",
    "settings.db-shm",
    "settings.db-wal",
    "settings.db",
    "Settings.index",
    "settings.json",
    "settings.schema",
    "SettingsCache.txt",
    "SettingsDatabase.db-shm",
    "SettingsDatabase.db-wal",
    "SettingsDatabase.db",
    "settingsglobals.txt",
    "settingssynonyms.txt",
    "SetupGridENV",
    "TemporaryItems",
    "Thumbs.db:encryptable",
    "Thumbs.db",
    "TransportSecurity",
    "Trust Tokens-journal",
    "Trust Tokens",
    "uriCache_",
    "uriCache",
    "UserClasses.dat",
    "Vpn Tokens-journal",
    "Vpn Tokens",
    "Web Data-journal",
    "Web Data",
    "WHEEL",
];

/// unparseable files by extension
pub const NAME_EXT_UNPARSEABLE: [&str; 134] = [
    "_baseband",
    "_powermanagement",
    "_storedata",
    "3gp",
    "7z",
    "a",
    "aac",
    "aux",
    "avi",
    "awk",
    "bat",
    "bin",
    "bmp",
    "bytecode",
    "bz",
    "c",
    "cab",
    "cat",
    "chm",
    "class",
    "cmd",
    "conf",
    "config",
    "cpp",
    "css",
    "db-journal",
    "db-shm",
    "db-wal",
    "diagpkg",
    "dll",
    "docx",
    "dxcache-shm",
    "dxcache-wal",
    "dxcache",
    "dynlib",
    "ear",
    "etag",
    "exe",
    "flac",
    "flv",
    "gif",
    "gitignore",
    "go",
    "h",
    "h264",
    "hlsl",
    "hpp",
    "htm",
    "html",
    "hyb",
    "ico",
    "identifier",
    "imagemanifest",
    "ini",
    "installstate",
    "jar",
    "java",
    "jpeg",
    "jpg",
    "keystore",
    "keystream",
    "lib",
    "library-ms",
    "lnk",
    "lsv",
    "lua",
    "m4b",
    "m4p",
    "m4r",
    "m4v",
    "manifest",
    "md5",
    "metadata-v2",
    "metadata",
    "mkv",
    "mov",
    "mp3",
    "mp4",
    "msi",
    "mui",
    "nupkg",
    "o",
    "ogg",
    "opus",
    "otc-shm",
    "otc-wal",
    "pdf",
    "pl",
    "png",
    "properties",
    "proto",
    "ps1",
    "psd1",
    "py",
    "pyc",
    "raw",
    "rb",
    "regtrans-ms",
    "schema",
    "sh",
    "sig",
    "so",
    "sock",
    "sql-shm",
    "sql-wal",
    "sql",
    "sqlite-shm",
    "sqlite-wal",
    "sqlite",
    "sqlite3",
    "svg",
    "sys",
    "tag",
    "tgz",
    "thumb",
    "thumbs",
    "tif",
    "tiff",
    "toml",
    "ttf",
    "vim",
    "vmoptions",
    "vsix",
    "wallet",
    "war",
    "wasm",
    "wav",
    "webm",
    "webp",
    "wma",
    "wmv",
    "xhtml",
    "yaml",
    "yml",
];

// TODO: nice rust feature would be compile-time join over const array
/// user-facing listing of the unparseable file extensions.
pub const NAME_EXT_UNPARSEABLE_STR: &str = concatcp!(
    NAME_EXT_UNPARSEABLE[0], ", ",
    NAME_EXT_UNPARSEABLE[1], ", ",
    NAME_EXT_UNPARSEABLE[2], ", ",
    NAME_EXT_UNPARSEABLE[3], ", ",
    NAME_EXT_UNPARSEABLE[4], ", ",
    NAME_EXT_UNPARSEABLE[5], ", ",
    NAME_EXT_UNPARSEABLE[6], ", ",
    NAME_EXT_UNPARSEABLE[7], ", ",
    NAME_EXT_UNPARSEABLE[8], ", ",
    NAME_EXT_UNPARSEABLE[9], ",\n",
    NAME_EXT_UNPARSEABLE[10], ", ",
    NAME_EXT_UNPARSEABLE[11], ", ",
    NAME_EXT_UNPARSEABLE[12], ", ",
    NAME_EXT_UNPARSEABLE[13], ", ",
    NAME_EXT_UNPARSEABLE[14], ", ",
    NAME_EXT_UNPARSEABLE[15], ", ",
    NAME_EXT_UNPARSEABLE[16], ", ",
    NAME_EXT_UNPARSEABLE[17], ", ",
    NAME_EXT_UNPARSEABLE[18], ", ",
    NAME_EXT_UNPARSEABLE[19], ",\n",
    NAME_EXT_UNPARSEABLE[20], ", ",
    NAME_EXT_UNPARSEABLE[21], ", ",
    NAME_EXT_UNPARSEABLE[22], ", ",
    NAME_EXT_UNPARSEABLE[23], ", ",
    NAME_EXT_UNPARSEABLE[24], ", ",
    NAME_EXT_UNPARSEABLE[25], ", ",
    NAME_EXT_UNPARSEABLE[26], ", ",
    NAME_EXT_UNPARSEABLE[27], ", ",
    NAME_EXT_UNPARSEABLE[28], ", ",
    NAME_EXT_UNPARSEABLE[29], ",\n",
    NAME_EXT_UNPARSEABLE[30], ", ",
    NAME_EXT_UNPARSEABLE[31], ", ",
    NAME_EXT_UNPARSEABLE[32], ", ",
    NAME_EXT_UNPARSEABLE[33], ", ",
    NAME_EXT_UNPARSEABLE[34], ", ",
    NAME_EXT_UNPARSEABLE[35], ", ",
    NAME_EXT_UNPARSEABLE[36], ", ",
    NAME_EXT_UNPARSEABLE[37], ", ",
    NAME_EXT_UNPARSEABLE[38], ", ",
    NAME_EXT_UNPARSEABLE[39], ",\n",
    NAME_EXT_UNPARSEABLE[40], ", ",
    NAME_EXT_UNPARSEABLE[41], ", ",
    NAME_EXT_UNPARSEABLE[42], ", ",
    NAME_EXT_UNPARSEABLE[43], ", ",
    NAME_EXT_UNPARSEABLE[44], ", ",
    NAME_EXT_UNPARSEABLE[45], ", ",
    NAME_EXT_UNPARSEABLE[46], ", ",
    NAME_EXT_UNPARSEABLE[47], ", ",
    NAME_EXT_UNPARSEABLE[48], ", ",
    NAME_EXT_UNPARSEABLE[49], ",\n",
    NAME_EXT_UNPARSEABLE[50], ", ",
    NAME_EXT_UNPARSEABLE[51], ", ",
    NAME_EXT_UNPARSEABLE[52], ", ",
    NAME_EXT_UNPARSEABLE[53], ", ",
    NAME_EXT_UNPARSEABLE[54], ", ",
    NAME_EXT_UNPARSEABLE[55], ", ",
    NAME_EXT_UNPARSEABLE[56], ", ",
    NAME_EXT_UNPARSEABLE[57], ", ",
    NAME_EXT_UNPARSEABLE[58], ", ",
    NAME_EXT_UNPARSEABLE[59], ",\n",
    NAME_EXT_UNPARSEABLE[60], ", ",
    NAME_EXT_UNPARSEABLE[61], ", ",
    NAME_EXT_UNPARSEABLE[62], ", ",
    NAME_EXT_UNPARSEABLE[63], ", ",
    NAME_EXT_UNPARSEABLE[64], ", ",
    NAME_EXT_UNPARSEABLE[65], ", ",
    NAME_EXT_UNPARSEABLE[66], ", ",
    NAME_EXT_UNPARSEABLE[67], ", ",
    NAME_EXT_UNPARSEABLE[68], ", ",
    NAME_EXT_UNPARSEABLE[69], ",\n",
    NAME_EXT_UNPARSEABLE[70], ", ",
    NAME_EXT_UNPARSEABLE[71], ", ",
    NAME_EXT_UNPARSEABLE[72], ", ",
    NAME_EXT_UNPARSEABLE[73], ", ",
    NAME_EXT_UNPARSEABLE[74], ", ",
    NAME_EXT_UNPARSEABLE[75], ", ",
    NAME_EXT_UNPARSEABLE[76], ", ",
    NAME_EXT_UNPARSEABLE[77], ", ",
    NAME_EXT_UNPARSEABLE[78], ", ",
    NAME_EXT_UNPARSEABLE[79], ",\n",
    NAME_EXT_UNPARSEABLE[80], ", ",
    NAME_EXT_UNPARSEABLE[81], ", ",
    NAME_EXT_UNPARSEABLE[82], ", ",
    NAME_EXT_UNPARSEABLE[83], ", ",
    NAME_EXT_UNPARSEABLE[84], ", ",
    NAME_EXT_UNPARSEABLE[85], ", ",
    NAME_EXT_UNPARSEABLE[86], ", ",
    NAME_EXT_UNPARSEABLE[87], ", ",
    NAME_EXT_UNPARSEABLE[88], ", ",
    NAME_EXT_UNPARSEABLE[89], ",\n",
    NAME_EXT_UNPARSEABLE[90], ", ",
    NAME_EXT_UNPARSEABLE[91], ", ",
    NAME_EXT_UNPARSEABLE[92], ", ",
    NAME_EXT_UNPARSEABLE[93], ", ",
    NAME_EXT_UNPARSEABLE[94], ", ",
    NAME_EXT_UNPARSEABLE[95], ", ",
    NAME_EXT_UNPARSEABLE[96], ", ",
    NAME_EXT_UNPARSEABLE[97], ", ",
    NAME_EXT_UNPARSEABLE[98], ", ",
    NAME_EXT_UNPARSEABLE[99], ",\n",
    NAME_EXT_UNPARSEABLE[100], ", ",
    NAME_EXT_UNPARSEABLE[101], ", ",
    NAME_EXT_UNPARSEABLE[102], ", ",
    NAME_EXT_UNPARSEABLE[103], ", ",
    NAME_EXT_UNPARSEABLE[104], ", ",
    NAME_EXT_UNPARSEABLE[105], ", ",
    NAME_EXT_UNPARSEABLE[106], ", ",
    NAME_EXT_UNPARSEABLE[107], ", ",
    NAME_EXT_UNPARSEABLE[108], ", ",
    NAME_EXT_UNPARSEABLE[109], ",\n",
    NAME_EXT_UNPARSEABLE[110], ", ",
    NAME_EXT_UNPARSEABLE[111], ", ",
    NAME_EXT_UNPARSEABLE[112], ", ",
    NAME_EXT_UNPARSEABLE[113], ", ",
    NAME_EXT_UNPARSEABLE[114], ", ",
    NAME_EXT_UNPARSEABLE[115], ", ",
    NAME_EXT_UNPARSEABLE[116], ", ",
    NAME_EXT_UNPARSEABLE[117], ", ",
    NAME_EXT_UNPARSEABLE[118], ", ",
    NAME_EXT_UNPARSEABLE[119], ",\n",
    NAME_EXT_UNPARSEABLE[120], ", ",
    NAME_EXT_UNPARSEABLE[121], ", ",
    NAME_EXT_UNPARSEABLE[122], ", ",
    NAME_EXT_UNPARSEABLE[123], ", ",
    NAME_EXT_UNPARSEABLE[124], ", ",
    NAME_EXT_UNPARSEABLE[125], ", ",
    NAME_EXT_UNPARSEABLE[126], ", ",
    NAME_EXT_UNPARSEABLE[127], ", ",
    NAME_EXT_UNPARSEABLE[128], ", ",
    NAME_EXT_UNPARSEABLE[129], ",\n",
    NAME_EXT_UNPARSEABLE[130], ", ",
    NAME_EXT_UNPARSEABLE[131], ", ",
    NAME_EXT_UNPARSEABLE[132], ", ",
    NAME_EXT_UNPARSEABLE[133],
);

#[inline(always)]
fn ratio_to_score(
    numerator: usize,
    denominator: usize,
) -> u8 {
    if denominator == 0 {
        return SCORE_NO_MATCH;
    }

    std::cmp::min((numerator * 255) / denominator, 255) as u8
}

/// Aid in detecting a UTF-8 encoded file with a BOM
fn score_heuristic_utf8_with_bom(bytes: &[u8]) -> u8 {
    if !bytes.starts_with(&BOM_UTF8) {
        return SCORE_NO_MATCH;
    }

    match std::str::from_utf8(&bytes[BOM_UTF8.len()..]) {
        Ok(_) => SCORE_UTF8_BOM,
        Err(_) => SCORE_NO_MATCH,
    }
}

/// Aid in detecting a UTF-8 encoded file without a BOM
fn score_heuristic_utf8_without_bom(bytes: &[u8]) -> u8 {
    // NUL-heavy UTF-16/UTF-32 ASCII text is technically valid UTF-8 bytes.
    if bytes.contains(&0) {
        return SCORE_NO_MATCH;
    }

    match std::str::from_utf8(bytes) {
        Ok(_) => SCORE_VALID_UTF8,
        Err(_) => SCORE_NO_MATCH,
    }
}

/// Aid in detecting a UTF-16 encoded file.
/// Score UTF-16 shape; `& !1` keeps the sample aligned to full code units.
fn score_heuristic_utf16(
    bytes: &[u8],
    bom: bool,
    little_endian: bool,
) -> u8 {
    let payload = if bom {
        let bom_bytes = if little_endian {
            &BOM_UTF16LE[..]
        } else {
            &BOM_UTF16BE[..]
        };
        if !bytes.starts_with(bom_bytes) {
            return SCORE_NO_MATCH;
        }
        &bytes[bom_bytes.len()..]
    } else {
        bytes
    };

    let sample_len = std::cmp::min(payload.len(), 256) & !1;
    if sample_len < 16 {
        return if bom { SCORE_BOM_ONLY } else { SCORE_NO_MATCH };
    }

    let mut pairs: usize = 0;
    let mut expected_zero: usize = 0;
    let mut opposite_zero: usize = 0;
    let mut expected_texty: usize = 0;
    let mut i = 0;
    while i + 1 < sample_len {
        let b0 = payload[i];
        let b1 = payload[i + 1];
        pairs += 1;

        if little_endian {
            if b1 == 0 {
                expected_zero += 1;
            }
            if b0 == 0 {
                opposite_zero += 1;
            }
            if is_texty_ascii_byte(b0) {
                expected_texty += 1;
            }
        } else {
            if b0 == 0 {
                expected_zero += 1;
            }
            if b1 == 0 {
                opposite_zero += 1;
            }
            if is_texty_ascii_byte(b1) {
                expected_texty += 1;
            }
        }

        i += 2;
    }

    let score = std::cmp::min(
        ratio_to_score(expected_zero, pairs),
        ratio_to_score(expected_texty, pairs),
    );

    if expected_zero * 100 < pairs * 45
        || expected_texty * 100 < pairs * 45
        || opposite_zero * 100 > pairs * 20
    {
        return if bom { SCORE_BOM_ONLY } else { SCORE_NO_MATCH };
    }

    if bom {
        std::cmp::max(score, SCORE_BOM_ONLY)
    } else {
        score
    }
}

/// Aid in detecting a UTF-16BE encoded file with a BOM
fn score_heuristic_utf16be_with_bom(bytes: &[u8]) -> u8 {
    score_heuristic_utf16(bytes, true, false)
}

/// Aid in detecting a UTF-16BE encoded file without a BOM
fn score_heuristic_utf16be_without_bom(bytes: &[u8]) -> u8 {
    score_heuristic_utf16(bytes, false, false)
}

/// Aid in detecting a UTF-16LE encoded file with a BOM
fn score_heuristic_utf16le_with_bom(bytes: &[u8]) -> u8 {
    score_heuristic_utf16(bytes, true, true)
}

/// Aid in detecting a UTF-16LE encoded file without a BOM
fn score_heuristic_utf16le_without_bom(bytes: &[u8]) -> u8 {
    score_heuristic_utf16(bytes, false, true)
}

/// Aid in detecting a UTF-32 encoded file.
/// Score UTF-32 shape; `& !3` keeps the sample aligned to full code units.
fn score_heuristic_utf32(
    bytes: &[u8],
    bom: bool,
    little_endian: bool,
) -> u8 {
    let payload = if bom {
        let bom_bytes = if little_endian {
            &BOM_UTF32LE[..]
        } else {
            &BOM_UTF32BE[..]
        };
        if !bytes.starts_with(bom_bytes) {
            return SCORE_NO_MATCH;
        }
        &bytes[bom_bytes.len()..]
    } else {
        bytes
    };

    let sample_len = std::cmp::min(payload.len(), 256) & !3;
    if sample_len < 32 {
        return if bom { SCORE_BOM_ONLY } else { SCORE_NO_MATCH };
    }

    let mut quads: usize = 0;
    let mut shape: usize = 0;
    let mut i = 0;
    while i + 3 < sample_len {
        let b0 = payload[i];
        let b1 = payload[i + 1];
        let b2 = payload[i + 2];
        let b3 = payload[i + 3];
        quads += 1;

        if little_endian {
            if is_texty_ascii_byte(b0) && b1 == 0 && b2 == 0 && b3 == 0 {
                shape += 1;
            }
        } else if b0 == 0 && b1 == 0 && b2 == 0 && is_texty_ascii_byte(b3) {
            shape += 1;
        }

        i += 4;
    }

    let score = ratio_to_score(shape, quads);
    if shape * 100 < quads * 45 {
        return if bom { SCORE_BOM_ONLY } else { SCORE_NO_MATCH };
    }

    if bom {
        std::cmp::max(score, SCORE_BOM_ONLY)
    } else {
        score
    }
}

/// Aid in detecting a UTF-32BE encoded file with a BOM
fn score_heuristic_utf32be_with_bom(bytes: &[u8]) -> u8 {
    score_heuristic_utf32(bytes, true, false)
}

/// Aid in detecting a UTF-32BE encoded file without a BOM
fn score_heuristic_utf32be_without_bom(bytes: &[u8]) -> u8 {
    score_heuristic_utf32(bytes, false, false)
}

/// Aid in detecting a UTF-32LE encoded file with a BOM
fn score_heuristic_utf32le_with_bom(bytes: &[u8]) -> u8 {
    score_heuristic_utf32(bytes, true, true)
}

/// Aid in detecting a UTF-32LE encoded file without a BOM
fn score_heuristic_utf32le_without_bom(bytes: &[u8]) -> u8 {
    score_heuristic_utf32(bytes, false, true)
}

/// Detect text encoding from raw bytes.
///
/// Returns `None` for empty input.
/// Defaults non-empty unknown input to UTF-8/ASCII.
///
/// Fast-path detection detection for UTF-8/ASCII as it is by far
/// the most common text encoding for log files.
pub fn detect_filetype_text_encoding(bytes: &[u8]) -> Option<FileTypeTextEncoding> {
    defn!("(bytes.len() {})", bytes.len());
    if bytes.is_empty() {
        defx!("empty bytes");
        return None;
    }

    // fast-path for UTF-8 detections

    let utf8_bom_score = score_heuristic_utf8_with_bom(bytes);
    if utf8_bom_score >= SCORE_UTF8_FAST_PATH_MIN {
        defx!("heuristic UTF-8 BOM score {}", utf8_bom_score);
        return Some(FileTypeTextEncoding::Utf8BOM);
    }

    let utf8_score = score_heuristic_utf8_without_bom(bytes);
    if utf8_score >= SCORE_UTF8_FAST_PATH_MIN {
        defx!("heuristic UTF-8 score {}", utf8_score);
        return Some(FileTypeTextEncoding::Utf8Ascii);
    }

    let candidates = [
        (
            FileTypeTextEncoding::Utf32leBOM,
            score_heuristic_utf32le_with_bom(bytes),
            80_u8,
        ),
        (
            FileTypeTextEncoding::Utf32beBOM,
            score_heuristic_utf32be_with_bom(bytes),
            79_u8,
        ),
        (
            FileTypeTextEncoding::Utf16leBOM,
            score_heuristic_utf16le_with_bom(bytes),
            70_u8,
        ),
        (
            FileTypeTextEncoding::Utf16beBOM,
            score_heuristic_utf16be_with_bom(bytes),
            69_u8,
        ),
        (
            FileTypeTextEncoding::Utf32le,
            score_heuristic_utf32le_without_bom(bytes),
            60_u8,
        ),
        (
            FileTypeTextEncoding::Utf32be,
            score_heuristic_utf32be_without_bom(bytes),
            59_u8,
        ),
        (
            FileTypeTextEncoding::Utf16le,
            score_heuristic_utf16le_without_bom(bytes),
            50_u8,
        ),
        (
            FileTypeTextEncoding::Utf16be,
            score_heuristic_utf16be_without_bom(bytes),
            49_u8,
        ),
    ];

    let (best_encoding, best_score, _best_priority) = candidates
        .iter()
        .copied()
        .max_by_key(|(_encoding, score, priority)| (*score, *priority))
        .unwrap();
    if best_score >= SCORE_DIRECT_MIN {
        defx!(
            "heuristic {:?} score {}",
            best_encoding,
            best_score,
        );
        return Some(best_encoding);
    }

    // Statistical detection using `chardet`.
    let (charset, _confidence, _language) = detect(bytes);
    let charset_norm = charset
        .to_ascii_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>();

    defx!(
        "chardet.detect: charset {:?}, confidence {}, language {:?}",
        charset,
        _confidence,
        _language,
    );

    match charset_norm.as_str() {
        "UTF16BE" | "UCS2BE" => Some(FileTypeTextEncoding::Utf16be),
        "UTF16" | "UTF16LE" | "UCS2" | "UCS2LE" => Some(FileTypeTextEncoding::Utf16le),
        "UTF32BE" | "UCS4BE" => Some(FileTypeTextEncoding::Utf32be),
        "UTF32" | "UTF32LE" | "UCS4" | "UCS4LE" => Some(FileTypeTextEncoding::Utf32le),
        "UTF8" | "ASCII" => Some(FileTypeTextEncoding::Utf8Ascii),
        _ => Some(FileTypeTextEncoding::Utf8Ascii),
    }
}

// ----------------
// FilePreProcessor

/// Initial path processing return type.
#[derive(Clone, Debug, Eq)]
pub enum ProcessPathResult {
    /// File can be processed by `s4`
    FileValid(FPath, FileType),
    /// File can be processed by `s4` specifically for Apple Unified Logs
    FileValidAppleUnifiedLogs {
        diagnostics: FPath,
        uuidtext: FPath,
        storage_type: AulStorageType,
        detection: AulDetection,
    },
    FileErrEmpty(FPath, FileType),
    FileErrTooSmall(FPath, FileType, FileSz),
    // TODO: [2022/06] `FileErrNoPermissions` not currently checked until too late
    /// Filesystem permissions do not allow reading the file
    FileErrNoPermissions(FPath),
    /// File is a known or unknown type and is not supported
    /// The second optional field is descriptive error
    FileErrNotSupported(FPath, Option<String>),
    /// Path exists and is not a file
    FileErrNotAFile(FPath),
    /// Path does not exist
    FileErrNotExist(FPath),
    /// Error loading necessary shared libraries
    FileErrLoadingLibrary(FPath, &'static str, FileType),
    /// All other errors as described in the second parameter message
    FileErr(FPath, String),
}

impl PartialEq for ProcessPathResult {
    /// implemented for comparisons in tests
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ProcessPathResult::FileValidAppleUnifiedLogs {
                    diagnostics: a,
                    uuidtext: b,
                    storage_type: e,
                    detection: g,
                },
                ProcessPathResult::FileValidAppleUnifiedLogs {
                    diagnostics: c,
                    uuidtext: d,
                    storage_type: f,
                    detection: h,
                },
            ) => a == c && b == d && e == f && g == h,
            (
                ProcessPathResult::FileValid(fpath1, f1),
                ProcessPathResult::FileValid(fpath2, f2)
            ) => {
                fpath1 == fpath2 && f1 == f2
            }
            (
                ProcessPathResult::FileErrNoPermissions(fpath1),
                ProcessPathResult::FileErrNoPermissions(fpath2)
            ) => {
                fpath1 == fpath2
            }
            (
                ProcessPathResult::FileErrNotSupported(fpath1, _),
                ProcessPathResult::FileErrNotSupported(fpath2, _)
            ) => {
                // ignore the optional message field
                fpath1 == fpath2
            }
            (
                ProcessPathResult::FileErrNotAFile(fpath1),
                ProcessPathResult::FileErrNotAFile(fpath2)
            ) => {
                fpath1 == fpath2
            }
            (
                ProcessPathResult::FileErrNotExist(fpath1),
                ProcessPathResult::FileErrNotExist(fpath2)
            ) => {
                fpath1 == fpath2
            }
            (
                ProcessPathResult::FileErrLoadingLibrary(fpath1, lib1, f1),
                ProcessPathResult::FileErrLoadingLibrary(fpath2, lib2, f2)
            ) => {
                fpath1 == fpath2 && lib1 == lib2 && f1 == f2
            }
            (
                ProcessPathResult::FileErr(fpath1, msg1),
                ProcessPathResult::FileErr(fpath2, msg2)
            ) => {
                fpath1 == fpath2 && msg1 == msg2
            }
            _ => false
        }
    }
}

/// a multi-file storage format
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileTypeArchiveMultiple {
    /// a `.tar` archive file
    Tar,
    /// a `.zip` archive file
    Zip,
}

#[cfg(test)]
/// Helper to `copy_process_path_result_canonicalize_path`
fn canonicalize_fpath(fpath: &FPath) -> FPath {
    let path: &Path = fpath_to_path(fpath);
    match path.canonicalize() {
        Ok(pathbuf) => {
            let s = FPath::from_str(pathbuf.to_str().unwrap());
            s.unwrap()
        }
        Err(_) => {
            // best effort: return the value passed-in
            fpath.clone()
        }
    }
}

#[cfg(test)]
/// Test helper to canonicalize the path contained by `ProcessPathResult`
///
/// Some Windows hosts return the MS-DOS shortened form of a path.
/// Later, that will fail comparisons to the canonical full form of the same
/// path.
///
/// e.g. `"C:\\Users\\RUNNER~1\\AppData\\Local\\Temp\\.tmp6TC2W5\\file1"`
///      !=
///      `"C:\\Users\\runneradmin\\AppData\\Local\\Temp\\.tmp6TC2W5\\file1"`
///
/// This function should resolve the first string to the second string.
pub(crate) fn copy_process_path_result_canonicalize_path(ppr: ProcessPathResult) -> ProcessPathResult {
    match ppr {
        ProcessPathResult::FileValid(fpath, f) => {
            let fpath_c = canonicalize_fpath(&fpath);

            ProcessPathResult::FileValid(fpath_c, f)
        }
        ProcessPathResult::FileErrNoPermissions(fpath) => {
            let fpath_c = canonicalize_fpath(&fpath);

            ProcessPathResult::FileErrNoPermissions(fpath_c)
        }
        ProcessPathResult::FileErrNotSupported(fpath, message) => {
            let fpath_c = canonicalize_fpath(&fpath);

            ProcessPathResult::FileErrNotSupported(fpath_c, message)
        }
        ProcessPathResult::FileErrNotAFile(fpath) => {
            let fpath_c = canonicalize_fpath(&fpath);

            ProcessPathResult::FileErrNotAFile(fpath_c)
        }
        ProcessPathResult::FileErrNotExist(fpath) => {
            let fpath_c = canonicalize_fpath(&fpath);

            ProcessPathResult::FileErrNotExist(fpath_c)
        }
        ret => ret,
    }
}

/// A list of `ProcessPathResult` items.
pub type ProcessPathResults = Vec<ProcessPathResult>;

/// returned by `pathbuf_to_filetype`, if `filetype` is set then
/// it is the `FileType` of the file at `pathbuf`
/// else an multi-file `archive`` type was encountered and must be processed
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathToFiletypeResult {
    Filetype(FileType),
    Archive(FileTypeArchiveMultiple, FileTypeArchive),
    AppleUnifiedLogs(AulStorageType, AulDetection),
}

/// Determine the `FileType` of a file based on the `pathbuf` file name.
///
/// This function does the bulk of tedious work around determining a `FileType` based on a
/// file name. It is recursive to handle multiple suffixes/extensions such as
/// `archive.tar.gz` or `syslog.1.gz`.
fn pathbuf_to_filetype_impl(
    pathbuf: &PathBuf,
    unparseable_are_text: bool,
    filetype_archive: Option<FileTypeArchive>,
) -> PathToFiletypeResult {
    defn!("({:?}, {:?}, {:?})", pathbuf, unparseable_are_text, filetype_archive);

    let fta: FileTypeArchive = match filetype_archive {
        Some(val) => val,
        None => FileTypeArchive::Normal,
    };

    let ret_fallback: PathToFiletypeResult = if unparseable_are_text {
        // unparseable is considered UTF-8/ASCII text
        PathToFiletypeResult::Filetype(
            FileType::Text {
                archival_type: fta,
                encoding_type: FileTypeTextEncoding::Utf8Ascii
            })
    } else {
        // unparseable is considered unknown or binary data
        PathToFiletypeResult::Filetype(FileType::Unparsable)
    };

    // TODO: check for subdirectory `__MACOSX` and other special directories

    // trim common trailing junk characters from suffix
    const JUNK_CHARS: &[char] = &['~', '-', ',', '?', ';'];
    let mut pathbuf_clean: &PathBuf = pathbuf;
    let mut file_name: &OsStr = pathbuf_clean
        .file_name()
        .unwrap_or_default();
    defo!("file_name     {:?}", file_name);
    let fname = file_name.to_str().unwrap_or_default();

    // check for known skippable file names
    if NAME_UNPARSEABLE.contains(&fname) {
        defx!("{fname:?} in NAME_UNPARSEABLE, return {ret_fallback:?}");
        return ret_fallback;
    }

    let mut pathbuf_ref: PathBuf;
    if fname.ends_with(JUNK_CHARS) {
        defo!("trailing junk characters {:?}", fname);
        let fname2 = fname.trim_end_matches(JUNK_CHARS);
        defo!("fname2        {:?}", fname2);
        if fname2.is_empty() {
            // the file name is all junk characters so return a fallback
            defx!("fname.ends_with(JUNK_CHARS) {fname:?}, return {ret_fallback:?}");
            return ret_fallback;
        }
        pathbuf_ref = pathbuf.with_file_name(fname2);
        pathbuf_clean = &pathbuf_ref;
        defo!("pathbuf_clean {:?}", pathbuf_clean);
        file_name = pathbuf_clean
            .file_name()
            .unwrap_or_default();
        defo!("file_name     {:?}", file_name);
    }

    // check for special case where symbolic directory, `.` or `..`, name was passed
    // this probably should never get here but handle it anyway
    let file_name_s = file_name
        .to_str()
        .unwrap_or_default();
    if !file_name_s.is_empty()
        && file_name_s
            .chars()
            .all(|c| c == '.')
    {
        de_err!("file_name_s {:?} is all '.'", file_name_s);
        defx!("file_name_s {:?} is all '.', return {ret_fallback:?}", file_name_s);
        return ret_fallback;
    }

    // trim leading junk characters from file_name
    const JUNK_CHARS_LEAD: &[char] = &['~', '-', ',', '?', ';', '.'];
    let fname_ = file_name.to_str().unwrap_or_default();
    if fname_.starts_with(JUNK_CHARS_LEAD) {
        defo!("leading junk characters {:?}", fname_);
        let fname2 = fname_.trim_start_matches(JUNK_CHARS_LEAD);
        defo!("fname2        {:?}", fname2);
        if fname2.is_empty() {
            // the file name is all junk characters so return a fallback
            defx!("fname.starts_with(JUNK_CHARS) {fname_:?}, return {ret_fallback:?}");
            return ret_fallback;
        }
        if fname2 != pathbuf_clean.extension().unwrap_or_default() {
            pathbuf_ref = pathbuf.with_file_name(fname2);
            pathbuf_clean = &pathbuf_ref;
            defo!("pathbuf_clean {:?}", pathbuf_clean);
            file_name = pathbuf_clean
                .file_name()
                .unwrap_or_default();
            defo!("file_name     {:?}", file_name);
        }
    }

    if file_name.is_empty() {
        defx!("file_name is empty, return {ret_fallback:?}");
        return ret_fallback;
    }

    // check again for known skippable file names
    if NAME_UNPARSEABLE.contains(&file_name.to_str().unwrap_or_default()) {
        defx!("{file_name:?} in NAME_UNPARSEABLE, return {ret_fallback:?}");
        return ret_fallback;
    }

    let file_suffix: String = pathbuf_clean
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    defo!("file_suffix   {:?}", file_suffix);

    if file_suffix.parse::<i32>().is_ok() {
        defo!("file_suffix   {:?} is a number; remove it", file_suffix);
        // suffix/extension is a number so remove it and call again
        let ret = pathbuf_to_filetype_impl(
            &pathbuf.clone().with_extension(""),
            unparseable_are_text,
            Some(fta),
        );
        defx!("return {:?}", ret);
        return ret;
    }

    // TODO: skip Apple Double shadow metadata files, e.g. `._photo.jpg`
    //       see https://cloudstreet-dev.github.io/macOS-and-Unix/part2-filesystem/metadata-files.html

    defo!("match file_suffix {:?}", file_suffix.as_str());
    match file_suffix.as_str() {
        "asl" => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Asl {
                    archival_type: fta,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "bz2" => {
            defo!("file_suffix {:?} is a bz2", file_suffix);
            let ret = pathbuf_to_filetype_impl(
                &pathbuf.clone().with_extension(""),
                unparseable_are_text,
                Some(FileTypeArchive::Bz2),
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "etl" => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Etl {
                    archival_type: fta,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "evtx" => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Evtx {
                    archival_type: fta,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "journal" => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Journal {
                    archival_type: fta,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        // `.loggz` is a OneDrive convention, though it typically wraps UTF-16 log files
        "loggz" => {
            defo!("file_suffix {:?} is a gzip but a OneDrive convention", file_suffix);
            let ret = PathToFiletypeResult::Filetype(
                FileType::Text {
                    archival_type: FileTypeArchive::Gz,
                    encoding_type: FileTypeTextEncoding::Utf16le,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        // `.gz` is very common
        // `.gzip` I have seen but I don't recall where
        "gz" | "gzip" => {
            defo!("file_suffix {:?} is a gzip", file_suffix);
            let ret = pathbuf_to_filetype_impl(
                &pathbuf.clone().with_extension(""),
                unparseable_are_text,
                Some(FileTypeArchive::Gz),
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "lz4" => {
            defo!("file_suffix {:?} is lzma4", file_suffix);
            let ret = pathbuf_to_filetype_impl(
                &pathbuf.clone().with_extension(""),
                unparseable_are_text,
                Some(FileTypeArchive::Lz4),
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        odl_sub_type @ "aodl"
        | odl_sub_type @ "odl"
        | odl_sub_type @ "odlsent"
        | odl_sub_type @ "odlgz" => {
            let odl_sub_type_enum: OdlSubType = match odl_sub_type {
                "aodl" => OdlSubType::Aodl,
                "odl" => OdlSubType::Odl,
                "odlsent" => OdlSubType::Odlsent,
                "odlgz" => OdlSubType::Odlgz,
                _ => {
                    debug_panic!("unexpected odl_sub_type {:?}", odl_sub_type);
                    OdlSubType::Odl
                }
            };
            let ret = PathToFiletypeResult::Filetype(
                FileType::Odl {
                    archival_type: fta,
                    odl_sub_type: odl_sub_type_enum,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "tar" => {
            defo!("file_suffix {:?} is a tar filetype_archive.is_none()", file_suffix);
            let ret = PathToFiletypeResult::Archive(
                FileTypeArchiveMultiple::Tar,
                fta,
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "xz" | "xzip" => {
            defo!("file_suffix {:?} is xz", file_suffix);
            let ret = pathbuf_to_filetype_impl(
                &pathbuf.clone().with_extension(""),
                unparseable_are_text,
                Some(FileTypeArchive::Xz),
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "zip" => {
            return PathToFiletypeResult::Archive(FileTypeArchiveMultiple::Zip, fta);
        }
        "jsonl"
        | "log"
        | "text"
        | "txt"
        // these may have broken parsing as text (i.e. datetime string may be N lines into the larger XML element)
        // but definitely not other formats
        | "json"
        | "xml"
        => {
            // This `FileTypeTextEncoding` value may be updated later during
            // function `blockzero_analysis`.
            let encoding_type = FileTypeTextEncoding::Utf8Ascii;
            let ret = PathToFiletypeResult::Filetype(
                FileType::Text {
                    archival_type: fta,
                    encoding_type,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "btmp"
        | "utmp"
        | "wtmp"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::Utmp,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "btmpx"
        | "utmpx"
        | "wtmpx"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::Utmpx,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "lastlog"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::Lastlog,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "lastlogx"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::Lastlogx,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "acct"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::Acct,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "pacct"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::AcctV3,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        // known unparseable file extensions/suffixes
        // covers some common extensions. not at all exhaustive but still helpful
        suffix if NAME_EXT_UNPARSEABLE.contains(&suffix)
        => {
            defx!("matched file_suffix {file_suffix:?}; return fallback {ret_fallback:?}");
            return ret_fallback;
        }
        _ => {
            defo!("file_suffix {:?} not matched", file_suffix);
        }
    }

    let file_name_sl: String = file_name
        .to_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let file_name_s: &str = file_name_sl.as_str();

    defo!("match file_name_s {:?}", file_name_s);
    match file_name_s {
        // OneDrive mapping files
        "general.keystore"
        | "ObfuscationStringMap.txt"
        => {
            defx!("file_name_s {file_name_s:?} (OneDrive mapping), return {ret_fallback:?}");
            return ret_fallback;
        }
        _ => {}
    }

    defo!("match file_suffix.is_empty() {:?}", file_suffix.is_empty());

    if !file_suffix.is_empty() {
        defo!("file_suffix {:?} not empty; remove it and call again", file_suffix);
        // remove the file suffix/extension and call again
        let ret = pathbuf_to_filetype_impl(
            &pathbuf.clone().with_extension(""),
            unparseable_are_text,
            Some(fta),
        );
        defx!("return {:?}", ret);
        return ret;
    }

    // getting here means there is no suffix/extension in the file name

    defo!("match file_name_s.is_empty() {:?}", file_name_s.is_empty());

    if file_name_s.is_empty() {
        defx!("file_name_s is empty, return {ret_fallback:?}");
        return ret_fallback;
    }

    defo!("match file_name_s {:?}", file_name_s);

    match file_name_s {
        "dmesg"
        | "history"
        | "kernellog"
        | "kernelog"
        | "kernlog"
        | "log"
        | "messages"
        | "syslog"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Text {
                    archival_type: fta,
                    encoding_type: FileTypeTextEncoding::Utf8Ascii
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "journal"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Journal {
                    archival_type: fta,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "acct"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::Acct,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "pacct"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::AcctV3,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "lastlog"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::Lastlog,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "lastlogx"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::Lastlogx,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "btmp"
        | "utmp"
        | "wtmp"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::Utmp,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "btmpx"
        | "utmpx"
        | "wtmpx"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::Utmpx,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        _ => {
            defo!("file_name_s   {:?} not matched", file_name_s);
        }
    }

    // for example, `log_media`
    if file_name_s.starts_with("log_") {
        let ret = PathToFiletypeResult::Filetype(
            FileType::Text {
                archival_type: fta,
                encoding_type: FileTypeTextEncoding::Utf8Ascii
            }
        );
        defx!("starts_with('log_') {:?}; return {:?}", file_name_s, ret);
        return ret;
    }
    // for example, `media_log`
    if file_name_s.ends_with("_log") {
        let ret = PathToFiletypeResult::Filetype(
            FileType::Text {
                archival_type: fta,
                encoding_type: FileTypeTextEncoding::Utf8Ascii
            }
        );
        defx!("ends_with('_log') {:?}; return {:?}", file_name_s, ret);
        return ret;
    }

    // not sure what the file is so try to process it as a plain text file

    let ret = PathToFiletypeResult::Filetype(
        FileType::Text {
            archival_type: fta,
            encoding_type: FileTypeTextEncoding::Utf8Ascii
        }
    );
    defx!("not sure what is {:?}; return {:?}", file_name_s, ret);

    ret
}

/// Wrapper function for function `pathbuf_to_filetype_impl`
pub fn pathbuf_to_filetype(
    pathbuf: &PathBuf,
    unparseable_are_text: bool,
) -> PathToFiletypeResult {
    if let Some(ProcessPathResult::FileValidAppleUnifiedLogs { storage_type, detection, .. }) =
        detect_aul_path(pathbuf)
    {
        return PathToFiletypeResult::AppleUnifiedLogs(storage_type, detection);
    }

    pathbuf_to_filetype_impl(pathbuf, unparseable_are_text, None)
}

/// Wrapper function for [`pathbuf_to_filetype`]
pub fn path_to_filetype(path: &Path, unparseable_are_text: bool) -> PathToFiletypeResult {
    pathbuf_to_filetype(&PathBuf::from(path), unparseable_are_text)
}

/// Wrapper function for [`path_to_filetype`]
#[cfg(test)]
pub fn fpath_to_filetype(path: &FPath, unparseable_are_text: bool) -> PathToFiletypeResult {
    path_to_filetype(fpath_to_path(path), unparseable_are_text)
}

/// Helper to `process_path_tar`
fn error_to_string(error: &std::io::Error, path: &FPath) -> String {
    format!("{}: {} for file {:?}", error.kind(), error, path)
}

/// Check if an archive member's name matches the given selector.
/// Returns `true` if the archive member's name matches the selector, `false` otherwise.
///
/// e.g. given an archive member name "dir1/file2.txt" and a selector "dir1",
/// this function will return `true`.
fn archive_member_matches(name: &str, selector: Option<&str>) -> bool {
    defñ!("name = {name:?}, selector = {selector:?}");
    match selector {
        None | Some("") => true,
        Some(selector) => name == selector || name
            .strip_prefix(selector.trim_end_matches('/'))
            .is_some_and(|suffix| suffix.starts_with('/')),
    }
}

/// Returned updated `results` of archive members based on the given `selector`.
fn archive_selection(
    mut results: ProcessPathResults,
    path: &FPath,
    selector: Option<&str>,
) -> ProcessPathResults {
    defn!("path: {path:?}, selector: {selector:?}, results {results:?}");
    let Some(selector) = selector.filter(|selector| !selector.is_empty()) else {
        defx!("empty or None selector");
        return results;
    };
    fn result_path(result: &ProcessPathResult) -> &str {
        match result {
            ProcessPathResult::FileValid(path, _)
            | ProcessPathResult::FileValidAppleUnifiedLogs { diagnostics: path, .. }
            | ProcessPathResult::FileErrEmpty(path, _)
            | ProcessPathResult::FileErrTooSmall(path, _, _)
            | ProcessPathResult::FileErrNoPermissions(path)
            | ProcessPathResult::FileErrNotSupported(path, _)
            | ProcessPathResult::FileErrNotAFile(path)
            | ProcessPathResult::FileErrNotExist(path)
            | ProcessPathResult::FileErrLoadingLibrary(path, _, _)
            | ProcessPathResult::FileErr(path, _) => path,
        }
    }
    let exact_path = format!("{path}{SUBPATH_SEP}{selector}");
    if results.iter().any(|result| result_path(result) == exact_path) {
        // Keep archive-level errors even when an exact member wins over descendants.
        results.retain(|result| {
            let path = result_path(result);
            path == exact_path || !path.contains(SUBPATH_SEP)
        });
    }
    if results.is_empty() {
        results.push(ProcessPathResult::FileErrNotExist(
            format!("{path}{SUBPATH_SEP_DISPLAY}{selector}"),
        ));
    }
    defx!("results={results:?}");

    results
}

/// Return a `ProcessPathResult` for each parseable file within
/// the `.tar` file at `path`.
pub fn process_path_tar(
    path: &FPath,
    unparseable_are_text: bool,
    _filetypearchive: FileTypeArchive,
) -> ProcessPathResults {
    process_path_tar_impl(path, unparseable_are_text, _filetypearchive, None)
}

fn process_path_tar_impl(
    path: &FPath,
    unparseable_are_text: bool,
    _filetypearchive: FileTypeArchive,
    selector: Option<&str>,
) -> ProcessPathResults {
    defn!("({:?}, {:?}, {:?})", path, unparseable_are_text, _filetypearchive);

    // TODO [2024/04/29] handle `filetypearchive`; extract the file first and then process it

    // debug runtime checks
    #[cfg(all(debug_assertions, not(test)))]
    {
        defo!("debug_assertions");
        let std_path = fpath_to_path(path);
        if !std_path.exists() {
            panic!("path does not exist: {:?}", std_path);
        }
        if !std_path.is_file() {
            panic!("path is not a file: {:?}", std_path);
        }
    }

    let file: File = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            return vec![ProcessPathResult::FileErrNoPermissions(path.clone())];
        }
        Err(error) => {
            return vec![ProcessPathResult::FileErr(path.clone(), error_to_string(&error, path))];
        }
    };
    defo!("tar::Archive::<File>::new({:?})", path);
    let mut archive: tar::Archive<File> = tar::Archive::<File>::new(file);
    let entry_iter: tar::Entries<File> = match archive.entries() {
        Ok(val) => val,
        Err(err) => {
            defx!("return FileErr; {:?}", err);
            let err_string = error_to_string(&err, path);
            return vec![ProcessPathResult::FileErr(path.clone(), err_string)];
        }
    };
    let mut results = Vec::<ProcessPathResult>::new();
    for (_i, entry_res) in entry_iter.enumerate() {
        let entry: tar::Entry<File> = match entry_res {
            Ok(val) => val,
            Err(err) => {
                defo!("entry Err {:?}; skip", err);
                let err_string = error_to_string(&err, path);
                results.push(ProcessPathResult::FileErr(path.clone(), err_string));
                continue;
            }
        };
        defo!("entry {:2}, size {:4}, {:?}", _i, entry.size(), entry.path().unwrap_or_default());
        let header: &tar::Header = entry.header();
        let etype: tar::EntryType = header.entry_type();
        // TODO: handle tar types `symlink` and `long_link`, currently they are ignored
        if !etype.is_file() {
            defo!("entry.header().entry_type() {:?}; skip", etype);
            continue;
        }
        defo!("entry.header().entry_type() {:?}", etype);
        let subpath: Cow<Path> = match entry.path() {
            Ok(val) => val,
            Err(err) => {
                defo!("entry.path() Err {:?}; skip", err);
                let err_string = error_to_string(&err, path);
                results.push(ProcessPathResult::FileErr(path.clone(), err_string));
                continue;
            }
        };
        let name = subpath.to_string_lossy();
        if !archive_member_matches(&name, selector) {
            continue;
        }
        let unparseable_are_text = unparseable_are_text || selector == Some(name.as_ref());
        if entry.size() == 0 {
            defo!("entry.size() is 0; skip");
            let subfpath: FPath = path.clone()
                + &String::from(SUBPATH_SEP)
                + subpath
                    .to_string_lossy()
                    .as_ref();
            results.push(ProcessPathResult::FileErrEmpty(subfpath, FileType::Unparsable));
            continue;
        }
        // first get the `FileType` of the subpath
        let subfpath: FPath = subpath
            .to_string_lossy()
            .to_string();
        let pathtofileresult = path_to_filetype(&subpath, unparseable_are_text);
        defo!("pathtofileresult {:?}", pathtofileresult);
        // path to a file within a .tar file looks like:
        //
        //     "path/file.tar|subpath/subfile"
        //
        // where `path/file.tar` are on the host filesystem, and `subpath/subfile` are within
        // the `.tar` file
        let mut fullpath: FPath = String::with_capacity(
            path.len()
            + SUBPATH_SEP.len_utf8()
            + subfpath.len()
            + 1
        );
        fullpath.push_str(path.as_str());
        fullpath.push(SUBPATH_SEP);
        fullpath.push_str(subfpath.as_str());
        defo!("fullpath {:?}", fullpath);
        let result: ProcessPathResult;
        match pathtofileresult {
            PathToFiletypeResult::Filetype(filetype) => {
                match filetype {
                    // Asl
                    FileType::Asl { .. } => {
                        result = ProcessPathResult::FileValid(
                            fullpath,
                            FileType::Asl { archival_type: FileTypeArchive::Tar }
                        );
                    }
                    // Etl
                    FileType::Etl { archival_type: at @ FileTypeArchive::Bz2, .. }
                    | FileType::Etl { archival_type: at @ FileTypeArchive::Gz, .. }
                    | FileType::Etl { archival_type: at @ FileTypeArchive::Lz4, .. }
                    | FileType::Etl{ archival_type: at @ FileTypeArchive::Xz, .. }
                    | FileType::Etl{ archival_type: at @ FileTypeArchive::Tar, .. }
                    | FileType::Etl{ archival_type: at @ FileTypeArchive::Zip, .. }
                    => {
                        result = ProcessPathResult::FileErrNotSupported(
                            fullpath,
                            Some(format!("cannot extract {} type from a tar archived file", at))
                        );
                    }
                    FileType::Etl { archival_type: FileTypeArchive::Normal, .. }
                    => {
                        result = ProcessPathResult::FileValid(
                            fullpath, FileType::Etl { archival_type: FileTypeArchive::Tar }
                        );
                    }
                    // Evtx
                    FileType::Evtx { archival_type: at @ FileTypeArchive::Bz2, .. }
                    | FileType::Evtx { archival_type: at @ FileTypeArchive::Gz, .. }
                    | FileType::Evtx { archival_type: at @ FileTypeArchive::Lz4, .. }
                    | FileType::Evtx{ archival_type: at @ FileTypeArchive::Xz, .. }
                    | FileType::Evtx{ archival_type: at @ FileTypeArchive::Tar, .. }
                    | FileType::Evtx{ archival_type: at @ FileTypeArchive::Zip, .. }
                    => {
                        result = ProcessPathResult::FileErrNotSupported(
                            fullpath,
                            Some(format!("cannot extract {} type from a tar archived file", at))
                        );
                    }
                    FileType::Evtx { archival_type: FileTypeArchive::Normal, .. }
                    => {
                        result = ProcessPathResult::FileValid(
                            fullpath, FileType::Evtx { archival_type: FileTypeArchive::Tar }
                        );
                    }
                    // FixedStruct
                    FileType::FixedStruct{ archival_type: at @ FileTypeArchive::Bz2, .. }
                    | FileType::FixedStruct{ archival_type: at @ FileTypeArchive::Gz, .. }
                    | FileType::FixedStruct{ archival_type: at @ FileTypeArchive::Lz4, .. }
                    | FileType::FixedStruct{ archival_type: at @ FileTypeArchive::Xz, .. }
                    | FileType::FixedStruct{ archival_type: at @ FileTypeArchive::Tar, .. }
                    | FileType::FixedStruct{ archival_type: at @ FileTypeArchive::Zip, .. }
                    => {
                        result = ProcessPathResult::FileErrNotSupported(
                            fullpath,
                            Some(format!("cannot extract {} type from a tar archived file", at))
                        );
                    }
                    FileType::FixedStruct { archival_type: FileTypeArchive::Normal, fixedstruct_type: ft }
                    => {
                        result = ProcessPathResult::FileValid(
                            fullpath, FileType::FixedStruct {
                                archival_type: FileTypeArchive::Tar, fixedstruct_type: ft
                            }
                        );
                    }
                    // Odl
                    FileType::Odl { archival_type: at @ FileTypeArchive::Bz2, .. }
                    | FileType::Odl { archival_type: at @ FileTypeArchive::Gz, .. }
                    | FileType::Odl { archival_type: at @ FileTypeArchive::Lz4, .. }
                    | FileType::Odl{ archival_type: at @ FileTypeArchive::Xz, .. }
                    | FileType::Odl{ archival_type: at @ FileTypeArchive::Tar, .. }
                    | FileType::Odl{ archival_type: at @ FileTypeArchive::Zip, .. }
                    => {
                        result = ProcessPathResult::FileErrNotSupported(
                            fullpath,
                            Some(format!("cannot extract {} type from a tar archived file", at))
                        );
                    }
                    FileType::Odl { archival_type: FileTypeArchive::Normal, odl_sub_type }
                    => {
                        result = ProcessPathResult::FileValid(
                            fullpath, FileType::Odl { archival_type: FileTypeArchive::Tar, odl_sub_type }
                        );
                    }
                    // Journal
                    FileType::Journal { archival_type: at @ FileTypeArchive::Bz2 }
                    | FileType::Journal { archival_type: at @ FileTypeArchive::Gz }
                    | FileType::Journal { archival_type: at @ FileTypeArchive::Lz4 }
                    | FileType::Journal { archival_type: at @ FileTypeArchive::Xz }
                    | FileType::Journal { archival_type: at @ FileTypeArchive::Tar }
                    | FileType::Journal { archival_type: at @ FileTypeArchive::Zip }
                    => {
                        result = ProcessPathResult::FileErrNotSupported(
                            fullpath,
                            Some(format!("cannot extract {} type from a tar archived file", at))
                        );
                    }
                    FileType::Journal { archival_type: FileTypeArchive::Normal }
                    => {
                        result = ProcessPathResult::FileValid(
                            fullpath, FileType::Journal {
                                archival_type: FileTypeArchive::Tar,
                            }
                        );
                    }
                    // Text
                    FileType::Text { archival_type: at @ FileTypeArchive::Bz2, .. }
                    | FileType::Text { archival_type: at @ FileTypeArchive::Gz, .. }
                    | FileType::Text { archival_type: at @ FileTypeArchive::Lz4, .. }
                    | FileType::Text { archival_type: at @ FileTypeArchive::Xz, .. }
                    | FileType::Text { archival_type: at @ FileTypeArchive::Tar, .. }
                    | FileType::Text { archival_type: at @ FileTypeArchive::Zip, .. }
                    => {
                        result = ProcessPathResult::FileErrNotSupported(
                            fullpath,
                            Some(format!("cannot extract {} type from a tar archived file", at))
                        );
                    }
                    FileType::Text { archival_type: FileTypeArchive::Normal, encoding_type: et }
                    => {
                        result = ProcessPathResult::FileValid(
                            fullpath, FileType::Text {
                                archival_type: FileTypeArchive::Tar, encoding_type: et
                            }
                        );
                    }
                    FileType::Aul { .. }
                    | FileType::Unparsable => {
                        result = ProcessPathResult::FileErrNotSupported(fullpath, None);
                    }
                }
            }
            PathToFiletypeResult::Archive(..)
            | PathToFiletypeResult::AppleUnifiedLogs(..) => {
                // Issue #14 cannot support nested archives
                result = ProcessPathResult::FileErrNotSupported(
                    fullpath,
                    Some(String::from("nested archives are not supported")),
                );
            }
        }
        defo!("push {:?}", result);
        results.push(result);
    }
    #[cfg(any(debug_assertions, test))]
    {
        for (i, result) in results.iter().enumerate() {
            defo!("result {} {:?}", i, result);
        }
    }
    let results_: ProcessPathResults = archive_selection(results, path, selector);
    defx!("process_path_tar({path:?}) {} results: {results_:?}", results_.len());

    results_
}

/// Return a result for each regular member of a ZIP archive.
///
/// XXX: Technically ZIP specification supports duplicate named entries.
///      The `zip` crate will only return the last entry in that case.
pub fn process_path_zip(
    path: &FPath,
    unparseable_are_text: bool,
    filetypearchive: FileTypeArchive,
) -> ProcessPathResults {
    process_path_zip_impl(path, unparseable_are_text, filetypearchive, None)
}

fn process_path_zip_impl(
    path: &FPath,
    unparseable_are_text: bool,
    filetypearchive: FileTypeArchive,
    selector: Option<&str>,
) -> ProcessPathResults {
    defn!("path={:?}", path);

    if filetypearchive != FileTypeArchive::Normal {
        debug_panic!("unsupported filetypearchive={:?}", filetypearchive);
        defx!("unsupported filetypearchive={:?}", filetypearchive);
        return vec![ProcessPathResult::FileErrNotSupported(
            path.clone(), Some(String::from("externally compressed ZIP archives are not supported")),
        )];
    }
    let file: File = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            defx!("permission denied for path={:?}", path);
            return vec![ProcessPathResult::FileErrNoPermissions(path.clone())];
        }
        Err(error) => {
            defx!("error opening path={:?}: {:?}", path, error);
            return vec![ProcessPathResult::FileErr(path.clone(), error_to_string(&error, path))];
        }
    };
    let archive: zip::ZipArchive<File> = match zip::ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(error) => {
            let zip_err = zip_error(error, path);
            let s = format!("error creating ZIP archive for path={:?}: {}", path, zip_err);
            defx!("{s}");
            return vec![ProcessPathResult::FileErr(path.clone(), s)];
        }
    };
    let mut results: ProcessPathResults = ProcessPathResults::new();
    for index in 0..archive.len() {
        defo!("processing ZIP entry {index}");
        let entry = match archive.by_index_data(index) {
            Ok(entry) => entry,
            Err(error) => {
                let zip_err = zip_error(error, path);
                let s = format!("error reading ZIP entry at index {index}: {zip_err}");
                defo!("skip entry; {s}");
                results.push(ProcessPathResult::FileErr(path.clone(), s));
                continue;
            }
        };
        if !entry.is_file() {
            defo!("skip entry {index}; not a file");
            continue;
        }
        let name = match entry.name() {
            Ok(name) => name,
            Err(error) => {
                let zip_err = zip_error(error, path);
                let s = format!("error getting ZIP entry name: {zip_err}");
                defo!("skip entry; {s}");
                results.push(ProcessPathResult::FileErr(path.clone(), s));
                continue;
            }
        };
        defo!("ZIP entry[{index}] name: {name:?}");
        if !archive_member_matches(name.as_ref(), selector) {
            defo!("skip entry; archive member does not match selector: {name:?}");
            continue;
        }
        let unparseable_are_text = unparseable_are_text || selector == Some(name.as_ref());
        if name.contains(SUBPATH_SEP) || entry.name_raw().contains(&0) {
            let s: String = format!("ZIP member {name:?} contains the archive separator {SUBPATH_SEP:?}");
            results.push(ProcessPathResult::FileErr(path.clone(), s));
            continue;
        }
        let fullpath = format!("{path}{SUBPATH_SEP}{name}");
        if entry.encrypted() {
            let s: String = format!("skip entry; ZIP entry {index} is encrypted: {path:?}:{name:?}");
            defo!("{s}");
            results.push(ProcessPathResult::FileErrNotSupported(
                if selector.is_some() { fullpath } else { path.clone() }, Some(s),
            ));
            continue;
        }
        // zip's SUPPORTED_COMPRESSION_METHODS omits its LZMA reader.
        if !matches!(entry.compression(),
            zip::CompressionMethod::Stored
                | zip::CompressionMethod::Deflated
                | zip::CompressionMethod::Deflate64
                | zip::CompressionMethod::Bzip2
                | zip::CompressionMethod::Lzma
                | zip::CompressionMethod::Ppmd
        ) {
            defo!("skip entry {index}; unsupported ZIP compression method: {:?}", entry.compression());
            results.push(ProcessPathResult::FileErrNotSupported(
                fullpath, Some(format!("unsupported ZIP compression method for entry {index}: {}", entry.compression())),
            ));
            continue;
        }
        let result = match path_to_filetype(Path::new(name.as_ref()), unparseable_are_text) {
            PathToFiletypeResult::Archive(..)
            | PathToFiletypeResult::AppleUnifiedLogs(..)
            => ProcessPathResult::FileErrNotSupported(
                fullpath, Some(format!("nested archives are not supported, found at entry {index}")),
            ),
            PathToFiletypeResult::Filetype(FileType::Unparsable) => {
                ProcessPathResult::FileErrNotSupported(fullpath, None)
            }
            PathToFiletypeResult::Filetype(filetype) if filetype.to_filetypearchive() != FileTypeArchive::Normal => {
                ProcessPathResult::FileErrNotSupported(
                    fullpath, Some(format!("compressed files within ZIP archives are not supported, found at entry {index}")),
                )
            }
            PathToFiletypeResult::Filetype(mut filetype) => {
                match &mut filetype {
                    FileType::Asl { archival_type }
                    | FileType::Etl { archival_type }
                    | FileType::Evtx { archival_type }
                    | FileType::FixedStruct { archival_type, .. }
                    | FileType::Journal { archival_type }
                    | FileType::Odl { archival_type, .. }
                    | FileType::Aul { archival_type, .. }
                    | FileType::Text { archival_type, .. }
                    => *archival_type = FileTypeArchive::Zip,
                    FileType::Unparsable => unreachable!(),
                }
                if entry.size() == 0 {
                    ProcessPathResult::FileErrEmpty(fullpath, filetype)
                } else {
                    ProcessPathResult::FileValid(fullpath, filetype)
                }
            }
        };
        defo!("result for entry {index}: {:?}", result);
        results.push(result);
    }
    if results.is_empty() && selector.is_none_or(|selector| selector.is_empty()) {
        defo!("no valid files found; returning FileErrEmpty for path {:?}", path);
        results.push(ProcessPathResult::FileErrEmpty(path.clone(), FileType::Unparsable));
    }
    let results_: ProcessPathResults = archive_selection(results, path, selector);
    defx!("for {path:?}: {} results: {results_:?}", results_.len());

    results_
}

fn process_archive_selector(
    path: &FPath,
    archive_path: &str,
    selector: &str,
) -> ProcessPathResults {
    defn!("path={path:?}, archive_path={archive_path:?}, selector={selector:?}");
    let archive_path: FPath = FPath::from(archive_path);
    let std_path: PathBuf = match Path::new(&archive_path).canonicalize() {
        Ok(path) => path,
        Err(error) => {
            defx!("error={error:?}");
            return vec![match error.kind() {
                std::io::ErrorKind::NotFound => ProcessPathResult::FileErrNotExist(path.clone()),
                std::io::ErrorKind::PermissionDenied => ProcessPathResult::FileErrNoPermissions(path.clone()),
                _ => ProcessPathResult::FileErr(path.clone(), error_to_string(&error, path)),
            }];
        }
    };
    if !std_path.is_file() {
        defx!("not a file: {:?}", std_path);
        return vec![ProcessPathResult::FileErrNotAFile(archive_path)];
    }

    match path_to_filetype(&std_path, false) {
        PathToFiletypeResult::Archive(FileTypeArchiveMultiple::Zip, FileTypeArchive::Normal) => {
            defx!("processing zip archive with selector: {:?}", selector);
            process_path_zip_impl(&archive_path, false, FileTypeArchive::Normal, Some(selector))
        }
        PathToFiletypeResult::Archive(FileTypeArchiveMultiple::Tar, FileTypeArchive::Normal) => {
            defx!("processing tar archive with selector: {:?}", selector);
            process_path_tar_impl(&archive_path, false, FileTypeArchive::Normal, Some(selector))
        }
        _ft => {
            defx!("unsupported filetype: {:?}", _ft);

            vec![
                ProcessPathResult::FileErrNotSupported(
                    path.clone(),
                    Some(String::from("member selection requires a ZIP or TAR archive")),
            )]
        }
    }
}

pub(crate) const AUL_TRACE_DIRS: [&str; 4] = ["Persist", "Special", "Signpost", "HighVolume"];
pub(crate) const AUL_LOGDATA_FILENAME: &str = "logdata.LiveData.tracev3";
pub(crate) const AUL_TIMESYNC_DIR: &str = "timesync";
pub(crate) const AUL_TIMESYNC_SUFFIX: &str = ".timesync";
pub(crate) static EXT_LOGARCHIVE: LazyLock<&OsStr> = LazyLock::new(|| &OsStr::new("logarchive"));
pub(crate) static EXT_TRACEV3: LazyLock<&OsStr> = LazyLock::new(|| &OsStr::new("tracev3"));
const ENV_AUL_PATHS: &str = "S4_AUL_PATHS";

/// Check if a given path is an Apple Unified Log trace file.
pub(crate) fn is_aul_trace(path: &Path) -> bool {
    let ret = path.extension() == Some(EXT_TRACEV3.as_ref())
        && !path.file_name().unwrap_or_default().to_string_lossy().starts_with("._");
    defñ!("path is {} AUL trace {path:?}", if ret { "" } else { "not" });

    ret
}

/// Check for Apple Unified Log traces in either the flattened or in-place layout.
fn has_aul_traces(root: &Path) -> bool {
    if root.join(AUL_LOGDATA_FILENAME).is_file() {
        defñ!("found file {AUL_LOGDATA_FILENAME:?}");
        return true;
    }

    let ret: bool = AUL_TRACE_DIRS.iter().any(|name| {
        std::fs::read_dir(root.join(name)).is_ok_and(|entries| {
            entries.filter_map(Result::ok).any(|entry|
                if entry.path().is_file() && is_aul_trace(&entry.path()) {
                    defñ!("found AUL trace: {:?}", entry.path());
                    true
                } else {
                    false
                }
            )
        })
    });

    if !ret {
        defñ!("no AUL traces found in {root:?}");
    }

    ret
}

fn has_aul_timesync(root: &Path) -> bool {
    defñ!("checking {root:?}");
    std::fs::read_dir(root.join(AUL_TIMESYNC_DIR)).is_ok_and(|entries| {
        entries.filter_map(Result::ok).any(|entry| {
            let Ok(file_name) = entry.file_name().into_string() else {
                return false;
            };
            let valid_name = file_name.strip_suffix(AUL_TIMESYNC_SUFFIX)
                .is_some_and(|stem| {
                    stem.len() == 16
                        && stem.bytes().all(|byte| byte.is_ascii_hexdigit())
                });
            valid_name
                && entry.file_type().is_ok_and(|file_type| file_type.is_file())
                && entry.metadata().is_ok_and(|metadata| metadata.len() > 0)
        })
    })
}

fn aul_detection(root: &Path) -> AulDetection {
    if has_aul_timesync(root) {
        defñ!("{root:?} ByTraceTimesync");

        AulDetection::ByTraceTimesync
    } else {
        defñ!("{root:?} ByTrace");

        AulDetection::ByTrace
    }
}

/// Create one logical *Apple Unified Logs* input from its trace and string roots.
fn create_apple_unified_logs(
    diagnostics: PathBuf,
    uuidtext: PathBuf,
    storage_type: AulStorageType,
    detection: AulDetection,
) -> ProcessPathResult {
    defñ!("AppleUnifiedLogs{{{diagnostics:?}, {uuidtext:?}, {storage_type:?}, {detection:?}}}");
    ProcessPathResult::FileValidAppleUnifiedLogs {
        diagnostics: path_to_fpath(&diagnostics),
        uuidtext: path_to_fpath(&uuidtext),
        storage_type,
        detection,
    }
}

/// Detect a flattened `.logarchive` by its directory suffix and trace files.
///
/// LogArchive roots store traces and their companion data in one tree, so both
/// logical parser roots are the archive directory.
fn detect_aul_logarchive_path(path: &Path) -> Option<ProcessPathResult> {
    defn!("logarchive path={path:?}");
    if path.extension() != Some(EXT_LOGARCHIVE.as_ref()) || !has_aul_traces(path) {
        defx!("not an Apple Unified Logs LogArchive");
        return None;
    }

    let capture = create_apple_unified_logs(
        path.to_owned(),
        path.to_owned(),
        AulStorageType::LogArchive,
        aul_detection(path),
    );
    defx!("detected LogArchive: {capture:?}");

    Some(capture)
}

/// Detect an in-place capture from one of its sibling data directories.
///
/// The only assumed layout is sibling `diagnostics` and `uuidtext` names.
/// Their absolute location is intentionally unrestricted. Trace evidence must
/// be present in `diagnostics`; `uuidtext` may be missing and is still passed to
/// the reader so it can report the companion-data failure with file context.
fn detect_aul_in_place_path(path: &Path) -> Option<ProcessPathResult> {
    defn!("in-place path={path:?}");
    let name: &str = match path.file_name() {
        Some(name) => name.to_str()?,
        None => {
            defx!("return None; file_name is None");
            return None
        },
    };
    if !matches!(name, "diagnostics" | "uuidtext") {
        defx!("return None; does not match 'diagnostics' or 'uuidtext'");
        return None;
    }
    let parent: &Path = match path.parent() {
        Some(parent) => parent,
        None => {
            defx!("return None; parent is None");
            return None;
        }
    };
    let diagnostics: PathBuf = parent.join("diagnostics");
    let uuidtext: PathBuf = parent.join("uuidtext");

    let capture = has_aul_traces(&diagnostics).then(|| create_apple_unified_logs(
        diagnostics,
        uuidtext,
        AulStorageType::InPlace,
        aul_detection(parent.join("diagnostics").as_path()),
    ));
    defx!("in-place capture={capture:?}");

    capture
}

/// Detect either *Apple Unified Logs* storage layout and retain its storage type.
fn detect_aul_path(path: &Path) -> Option<ProcessPathResult> {
    fn detect(path: &Path) -> Option<ProcessPathResult> {
        if path.extension() == Some(EXT_LOGARCHIVE.as_ref()) {
            return detect_aul_logarchive_path(path);
        }
        if path.file_name().is_some_and(|name| matches!(name.to_str(), Some("diagnostics" | "uuidtext"))) {
            return detect_aul_in_place_path(path);
        }
        None
    }

    if !path.is_dir() {
        return None;
    }

    detect(path).or_else(|| path.is_symlink().then(|| path.canonicalize().ok()).flatten()
        .and_then(|canonical_path| detect(&canonical_path)))
}

/// Allow user-passed *Apple Unified Logs* paths via the environment variable `S4_AUL_PATHS`
fn configured_aul_paths() -> Vec<PathBuf> {
    std::env::var_os(ENV_AUL_PATHS).map_or_else(Vec::new, |value| {
        std::env::split_paths(&value)
            .filter(|path| !path.as_os_str().is_empty())
            .filter_map(|path| path.canonicalize().ok())
            .collect()
    })
}

fn is_configured_aul_path(path: &Path, configured_paths: &[PathBuf]) -> bool {
    path.canonicalize().is_ok_and(|path| configured_paths.contains(&path))
}

/// Detect if the given path is a configured *Apple Unified Logs* path.
fn detect_aul_configured_path(
    path: &Path,
    configured_paths: &[PathBuf],
) -> Option<ProcessPathResult> {
    defn!("({path:?}, configured_paths)");
    if configured_paths.is_empty()
        || !path.is_dir()
        || !is_configured_aul_path(path, configured_paths)
    {
        defx!("return None");
        return None;
    }

    let diagnostics = path.join("diagnostics");
    let uuidtext = path.join("uuidtext");
    if diagnostics.is_dir() || uuidtext.is_dir() {
        let detection = if has_aul_traces(&diagnostics) {
            aul_detection(&diagnostics)
        } else {
            AulDetection::ByEnvironment
        };
        defx!("return Some(AppleUnifiedLogs{{{diagnostics:?}, {{{uuidtext:?}}}, AulStorageType::InPlace, {detection:?}}})");
        return Some(create_apple_unified_logs(
            diagnostics,
            uuidtext,
            AulStorageType::InPlace,
            detection,
        ));
    }

    let detection = if has_aul_traces(path) {
        aul_detection(path)
    } else {
        AulDetection::ByEnvironment
    };
    defx!("return Some(AppleUnifiedLogs{{{path:?}, {{{path:?}}}, AulStorageType::LogArchive, {detection:?}}})");

    Some(create_apple_unified_logs(
        path.to_owned(),
        path.to_owned(),
        AulStorageType::LogArchive,
        detection,
    ))
}

fn detect_aul_path_with_configured(
    path: &Path,
    configured_paths: &[PathBuf],
) -> Option<ProcessPathResult> {
    detect_aul_path(path).or_else(|| detect_aul_configured_path(path, configured_paths))
}

/// Find parseable files, archive members, and Apple Unified Log captures.
///
/// Recognized Apple Unified Log captures are not traversed. A directly passed
/// file or an exact `archive|member` selection is tried as text if its format
/// is unknown; other archive members follow directory-discovery exclusions.
pub fn process_path(
    path: &FPath,
    _unparseable_are_text: bool,
) -> ProcessPathResults {
    defn!("({:?}, {:?})", path, _unparseable_are_text);

    let configured_aul_paths = configured_aul_paths();
    let mut std_path: PathBuf = PathBuf::from(path);

    defo!("std_path {:?}", std_path);

    std_path = match std_path.canonicalize() {
        Ok(val) => val,
        Err(err) => {
            // Windows rejects the display separator before checking path existence.
            if matches!(
                err.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidFilename
            ) {
                if let Some((archive_path, selector)) = path.split_once(
                    [SUBPATH_SEP_DISPLAY, SUBPATH_SEP_DISPLAY_ALT]
                ) {
                     return process_archive_selector(path, archive_path, selector);
                 }
            }
            match err.kind() {
                std::io::ErrorKind::NotFound => {
                    defx!("return FileErrNotExist({:?})", path);
                    return vec![ProcessPathResult::FileErrNotExist(path.clone())];
                }
                std::io::ErrorKind::PermissionDenied => {
                    defx!("return FileErrNoPermissions({:?})", path);
                    return vec![ProcessPathResult::FileErrNoPermissions(path.clone())];
                }
                _ => {
                    let err_string = error_to_string(&err, path);
                    defx!("return FileErr({:?}, {:?})", path, err_string);
                    return vec![ProcessPathResult::FileErr(path.clone(), err_string)];
                }
            }
        }
    };
    defo!("std_path {:?}", std_path);

    if std_path.is_file() {
        if std_path.extension() == Some(EXT_TRACEV3.as_ref()) {
            defx!("return FileErr for found TraceV3 {path:?} file");
            return vec![ProcessPathResult::FileErr(
                path.clone(), "Apple Unified Logs require a capture directory, not an individual tracev3 file".into(),
            )];
        }
        // if passed a path to a file (or a symlink to a plain file)
        // then assume the user wants to force an attempt to process
        // such a file (even if it's known to be unparsable, e.g. `picture.png`)
        // so pass `unparseable_are_text` as `true` so any unknown or unparseable
        // are treated as `FileType::Text`
        let result: PathToFiletypeResult = pathbuf_to_filetype(&std_path, true);
        match result {
            PathToFiletypeResult::Filetype(filetype) => {
                debug_assert!(!filetype.is_archived());
                let paths: ProcessPathResults = vec![ProcessPathResult::FileValid(path.clone(), filetype)];
                defx!("({:?}) {:?}", path, paths);
                return paths;
            }
            PathToFiletypeResult::Archive(archive, fta) => {
                match archive {
                    FileTypeArchiveMultiple::Zip => {
                        defo!("std_path is a .zip file {:?}", std_path);
                        let results = process_path_zip(path, false, fta);
                        defx!("return process_path_zip({:?}) returned {} results", path, results.len());
                        return results;
                    }
                    FileTypeArchiveMultiple::Tar => {
                        // getting here means `std_path` is a `.tar` file
                        defo!("std_path is a .tar file {:?}", std_path);
                        let results = process_path_tar(path, false, fta);
                        defx!("return process_path_tar({:?}) returned {} results", path, results.len());
                        return results;
                    }
                }
            }
            PathToFiletypeResult::AppleUnifiedLogs(..) => {
                debug_panic!("unexpected AppleUnifiedLogs path");
                return vec![ProcessPathResult::FileErr(path.clone(), "unexpected AppleUnifiedLogs path".into())];
            }
        }
    }

    // Direct Apple Unified Logs roots are processed as one logical input.
    if let Some(capture) = detect_aul_path_with_configured(Path::new(path), &configured_aul_paths) {
        defx!("direct Apple Unified Logs path detected");
        return vec![capture];
    }

    // getting here means `path` likely refers to a directory

    let mut paths: ProcessPathResults = ProcessPathResults::new();

    defo!("jwalk::rayon::current_num_threads = {}", jwalk::rayon::current_num_threads());
    defo!("jwalk::rayon::max_num_threads = {}", jwalk::rayon::max_num_threads());

    defo!("WalkDir({:?})…", path);
    for entry in jwalk::WalkDirGeneric::<((), Option<ProcessPathResult>)>::new(path.as_str())
        .follow_links(true)
        .sort(true)
        .process_read_dir(move |_, _, _, children| {
            def1n!("pre-processing {} entries", children.len());
            // do not let jwalk descend into Apple Unified Logs directories
            for entry in children.iter_mut().filter_map(|entry| entry.as_mut().ok())
            {
                def1o!("pre-process {:?}", entry.path());
                if ! entry.file_type().is_dir() {
                    def1x!("not a directory");
                    continue;
                }
                let path: PathBuf = entry.path();
                if let Some(capture) = detect_aul_path_with_configured(&path, &configured_aul_paths) {
                    def1o!("Apple Unified Logs path detected at {path:?}");
                    entry.read_children = None;
                    entry.client_state = Some(capture);
                }
            }
            def1x!();
        })
    {
        // XXX: what is type `T` in `Result<T, E>` returned by `WalkDir`?
        let mut path_entry = match entry {
            Ok(val) => {
                defo!("Ok({:?})", val);
                val
            }
            Err(_err) => {
                defo!("Err({:?})", _err);
                continue;
            }
        };

        if let Some(capture) = path_entry.client_state.take() {
            paths.push(capture);
            continue;
        }
        defo!("analayzing {:?}", path_entry);
        let std_path_entry: &Path = &path_entry.path();
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
            defo!("Path not a file {:?}", path_entry);
            paths.push(ProcessPathResult::FileErrNotAFile(fpath_entry));
            continue;
        }
        if std_path_entry.extension() == Some(EXT_TRACEV3.as_ref()) {
            defo!("Skip TraceV3 file {:?}", path_entry);
            continue;
        }
        let result: PathToFiletypeResult = path_to_filetype(std_path_entry, false);
        defo!("path_to_filetype({:?}) returned {:?}", std_path_entry, result);
        let filetype: FileType = match result {
            PathToFiletypeResult::Filetype(filetype) => filetype,
            PathToFiletypeResult::AppleUnifiedLogs(..) => {
                debug_panic!("Apple Unified Logs should not be classified as a single file");
                continue;
            },
            PathToFiletypeResult::Archive(archive, fta) => {
                let results: ProcessPathResults = match archive {
                    FileTypeArchiveMultiple::Zip => {
                        process_path_zip(&fpath_entry, false, fta)
                    }
                    FileTypeArchiveMultiple::Tar => {
                        // getting here means `std_path` is a `.tar` file
                        defo!("std_path_entry is a .tar file {:?}", std_path_entry);

                        process_path_tar(&path_to_fpath(std_path_entry), false, fta)
                    }
                };
                for result in results.into_iter() {
                    paths.push(result);
                }
                continue;
            }
        };
        match filetype {
            FileType::Asl{ archival_type: FileTypeArchive::Normal }
            | FileType::Asl{ archival_type: FileTypeArchive::Bz2 }
            | FileType::Asl{ archival_type: FileTypeArchive::Gz }
            | FileType::Asl{ archival_type: FileTypeArchive::Lz4 }
            | FileType::Asl{ archival_type: FileTypeArchive::Tar }
            | FileType::Asl{ archival_type: FileTypeArchive::Xz }
            | FileType::Asl{ archival_type: FileTypeArchive::Zip }
            | FileType::Etl{ archival_type: FileTypeArchive::Normal }
            | FileType::Etl{ archival_type: FileTypeArchive::Bz2 }
            | FileType::Etl{ archival_type: FileTypeArchive::Gz }
            | FileType::Etl{ archival_type: FileTypeArchive::Lz4 }
            | FileType::Etl{ archival_type: FileTypeArchive::Tar }
            | FileType::Etl{ archival_type: FileTypeArchive::Xz }
            | FileType::Etl{ archival_type: FileTypeArchive::Zip }
            | FileType::Evtx{ archival_type: FileTypeArchive::Normal }
            | FileType::Evtx{ archival_type: FileTypeArchive::Bz2 }
            | FileType::Evtx{ archival_type: FileTypeArchive::Gz }
            | FileType::Evtx{ archival_type: FileTypeArchive::Lz4 }
            | FileType::Evtx{ archival_type: FileTypeArchive::Tar }
            | FileType::Evtx{ archival_type: FileTypeArchive::Xz }
            | FileType::Evtx{ archival_type: FileTypeArchive::Zip }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, fixedstruct_type: _ }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Bz2, fixedstruct_type: _ }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, fixedstruct_type: _ }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, fixedstruct_type: _ }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, fixedstruct_type: _ }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, fixedstruct_type: _ }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Zip, fixedstruct_type: _ }
            | FileType::Journal{ archival_type: FileTypeArchive::Normal }
            | FileType::Journal{ archival_type: FileTypeArchive::Bz2 }
            | FileType::Journal{ archival_type: FileTypeArchive::Gz }
            | FileType::Journal{ archival_type: FileTypeArchive::Lz4 }
            | FileType::Journal{ archival_type: FileTypeArchive::Tar }
            | FileType::Journal{ archival_type: FileTypeArchive::Xz }
            | FileType::Journal{ archival_type: FileTypeArchive::Zip }
            | FileType::Odl{ .. }
            | FileType::Aul{ .. }
            | FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: _ }
            | FileType::Text{ archival_type: FileTypeArchive::Bz2, encoding_type: _ }
            | FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: _ }
            | FileType::Text{ archival_type: FileTypeArchive::Lz4, encoding_type: _ }
            | FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: _ }
            | FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: _ }
            | FileType::Text{ archival_type: FileTypeArchive::Zip, encoding_type: _ }
            => {
                defo!("paths.push(FileValid(({:?}, {:?})))", fpath_entry, filetype);
                paths.push(ProcessPathResult::FileValid(fpath_entry, filetype));
            }
            FileType::Unparsable
            => {
                defo!("Path not a log file {:?}", std_path_entry);
                paths.push(ProcessPathResult::FileErrNotSupported(
                    fpath_entry,
                    None,
                ));
            }
        }
    }

    let mut aul_captures = HashMap::<(String, String, AulStorageType), AulDetection>::new();
    for result in &paths {
        if let ProcessPathResult::FileValidAppleUnifiedLogs {
            diagnostics,
            uuidtext,
            storage_type,
            detection,
        } = result
        {
            aul_captures.entry((diagnostics.clone(), uuidtext.clone(), *storage_type))
                .and_modify(|best| *best = (*best).max(*detection))
                .or_insert(*detection);
        }
    }
    let mut aul_captures_retained = HashSet::new();
    paths.retain(|result| {
        let ProcessPathResult::FileValidAppleUnifiedLogs {
            diagnostics,
            uuidtext,
            storage_type,
            detection,
        } = result else {
            return true;
        };

        let key = (diagnostics.clone(), uuidtext.clone(), *storage_type);
        *detection == aul_captures[&key] && aul_captures_retained.insert(key)
    });
    defx!("return {:?}", paths);

    paths
}
