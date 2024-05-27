// filedecompressor_tests.rs

use std::time::SystemTime;

use crate::common::{
    FPath,
    FileSz,
    FileType,
    FileTypeArchive,
    FileTypeTextEncoding,
};
use crate::readers::filedecompressor::decompress_to_ntf;
use crate::readers::helpers::fpath_to_path;
use crate::tests::common::{
    // evtx
    EVTX_KPNP_FILESZ,
    EVTX_KPNP_FPATH,
    EVTX_KPNP_GZ_FPATH,
    EVTX_KPNP_GZ_MTIME,
    EVTX_KPNP_LZ4_FPATH,
    EVTX_KPNP_LZ4_MTIME,
    EVTX_KPNP_TAR_FPATH,
    EVTX_KPNP_TAR_MTIME,
    EVTX_KPNP_XZ_FPATH,
    EVTX_KPNP_XZ_MTIME,
    // journal
    JOURNAL_FILE_RHE_91_SYSTEM_FILESZ,
    JOURNAL_FILE_RHE_91_SYSTEM_FPATH,
    JOURNAL_FILE_RHE_91_SYSTEM_GZ_FPATH,
    JOURNAL_FILE_RHE_91_SYSTEM_GZ_MTIME,
    JOURNAL_FILE_RHE_91_SYSTEM_LZ4_FPATH,
    JOURNAL_FILE_RHE_91_SYSTEM_LZ4_MTIME,
    JOURNAL_FILE_RHE_91_SYSTEM_TAR_FPATH,
    JOURNAL_FILE_RHE_91_SYSTEM_TAR_MTIME,
    JOURNAL_FILE_RHE_91_SYSTEM_XZ_FPATH,
    JOURNAL_FILE_RHE_91_SYSTEM_XZ_MTIME,
    // text
    NTF_LOG_EMPTY_FPATH,
    NTF_NL_1_PATH,
    NTF_GZ_EMPTY_FPATH,
    NTF_LZ4_8BYTE_FPATH,
    NTF_TAR_1BYTE_FILEA_FPATH,
    NTF_XZ_EMPTY_FPATH,
    // fixedstruct
    LINUX_X86_UTMPX_3ENTRY_FILETYPE,
    NTF_LINUX_X86_UTMPX_3ENTRY_FPATH,
};

use ::test_case::test_case;


// evtx files
#[test_case(
    &EVTX_KPNP_GZ_FPATH,
    FileType::Evtx{ archival_type: FileTypeArchive::Gz },
    Some(*EVTX_KPNP_GZ_MTIME),
    EVTX_KPNP_FILESZ;
    "evtx.gz"
)]
#[test_case(
    &EVTX_KPNP_LZ4_FPATH,
    FileType::Evtx{ archival_type: FileTypeArchive::Lz4 },
    Some(*EVTX_KPNP_LZ4_MTIME),
    EVTX_KPNP_FILESZ;
    "evtx.lz4"
)]
#[test_case(
    &EVTX_KPNP_TAR_FPATH,
    FileType::Evtx{ archival_type: FileTypeArchive::Tar },
    Some(*EVTX_KPNP_TAR_MTIME),
    EVTX_KPNP_FILESZ;
    "evtx.tar"
)]
#[test_case(
    &EVTX_KPNP_XZ_FPATH,
    FileType::Evtx{ archival_type: FileTypeArchive::Xz },
    Some(*EVTX_KPNP_XZ_MTIME),
    EVTX_KPNP_FILESZ;
    "evtx.xz"
)]
// journal files
#[test_case(
    &JOURNAL_FILE_RHE_91_SYSTEM_GZ_FPATH,
    FileType::Journal { archival_type: FileTypeArchive::Gz },
    Some(*JOURNAL_FILE_RHE_91_SYSTEM_GZ_MTIME),
    JOURNAL_FILE_RHE_91_SYSTEM_FILESZ;
    "journal.gz"
)]
#[test_case(
    &JOURNAL_FILE_RHE_91_SYSTEM_LZ4_FPATH,
    FileType::Evtx{ archival_type: FileTypeArchive::Lz4 },
    Some(*JOURNAL_FILE_RHE_91_SYSTEM_LZ4_MTIME),
    JOURNAL_FILE_RHE_91_SYSTEM_FILESZ;
    "journal.lz4"
)]
#[test_case(
    &JOURNAL_FILE_RHE_91_SYSTEM_TAR_FPATH,
    FileType::Evtx{ archival_type: FileTypeArchive::Tar },
    Some(*JOURNAL_FILE_RHE_91_SYSTEM_TAR_MTIME),
    JOURNAL_FILE_RHE_91_SYSTEM_FILESZ;
    "journal.tar"
)]
#[test_case(
    &JOURNAL_FILE_RHE_91_SYSTEM_XZ_FPATH,
    FileType::Evtx{ archival_type: FileTypeArchive::Xz },
    Some(*JOURNAL_FILE_RHE_91_SYSTEM_XZ_MTIME),
    JOURNAL_FILE_RHE_91_SYSTEM_FILESZ;
    "journal.xz"
)]
fn test_decompress_to_ntf_ok_some(
    fpath: &FPath,
    filetype: FileType,
    systemtime: Option<SystemTime>,
    filesz: FileSz,
) {
    // XXX: catch error for newly added FileType or FileTypeArchive variants not yet handled
    match filetype {
        FileType::Unparsable => {
            panic!();
        }
        FileType::Evtx { archival_type }
        | FileType::FixedStruct { archival_type, .. }
        | FileType::Journal { archival_type }
        | FileType::Text { archival_type, .. }
        => {
            match archival_type {
                FileTypeArchive::Normal
                | FileTypeArchive::Gz
                | FileTypeArchive::Lz4
                | FileTypeArchive::Tar
                | FileTypeArchive::Xz
                => {}
            }
        }
    }
    let path = fpath_to_path(fpath);
    let result = decompress_to_ntf(
        path,
        &filetype,
    );
    assert!(result.is_ok());
    let value_opt = result.unwrap();
    assert!(value_opt.is_some());
    let value = value_opt.unwrap();
    // XXX: the `touch` and `SetFile` program on MacOS do not accept timezone offsets.
    //      The datetimestamps in `log-files-time-update.txt` have timezone offsets.
    //      Those timezone offsets cause an error for `touch` and `SetFile`.
    //      After consideration, this hack is the least worst workaround.
    #[cfg(not(target_os="macos"))]
    {
        assert_eq!(
            systemtime, value.1,
            "systemtime differs;\nexpected: {:?}\ngot     : {:?}",
            systemtime, value.1,
        );
    }
    assert_eq!(
        filesz, value.2,
        "filesz differs; expected: {}, got: {}",
        filesz, value.2,
    );
}

// evtx files
#[test_case(
    &EVTX_KPNP_FPATH,
    FileType::Evtx{ archival_type: FileTypeArchive::Normal };
    "evtx"
)]
// journal files
#[test_case(
    &JOURNAL_FILE_RHE_91_SYSTEM_FPATH,
    FileType::Journal { archival_type: FileTypeArchive::Normal };
    "journal"
)]
fn test_decompress_to_ntf_ok_none(
    fpath: &FPath,
    filetype: FileType,
) {
    // XXX: catch error for newly added FileType or FileTypeArchive variants not yet handled
    match filetype {
        FileType::Unparsable => {
            panic!();
        }
        FileType::Evtx { archival_type }
        | FileType::FixedStruct { archival_type, .. }
        | FileType::Journal { archival_type }
        | FileType::Text { archival_type, .. }
        => {
            match archival_type {
                FileTypeArchive::Normal
                | FileTypeArchive::Gz
                | FileTypeArchive::Lz4
                | FileTypeArchive::Tar
                | FileTypeArchive::Xz
                => {}
            }
        }
    }
    let path = fpath_to_path(fpath);
    let result = decompress_to_ntf(
        path,
        &filetype,
    );
    assert!(result.is_ok());
    let value_opt = result.unwrap();
    assert!(value_opt.is_none());
}

const FT_TEXT: FileType = FileType::Text {
    archival_type: FileTypeArchive::Normal,
    encoding_type: FileTypeTextEncoding::Utf8Ascii,
};
const FT_TEXT_GZ: FileType = FileType::Text {
    archival_type: FileTypeArchive::Gz,
    encoding_type: FileTypeTextEncoding::Utf8Ascii,
};
const FT_TEXT_LZ4: FileType = FileType::Text {
    archival_type: FileTypeArchive::Lz4,
    encoding_type: FileTypeTextEncoding::Utf8Ascii,
};
const FT_TEXT_TAR: FileType = FileType::Text {
    archival_type: FileTypeArchive::Tar,
    encoding_type: FileTypeTextEncoding::Utf8Ascii,
};
const FT_TEXT_XZ: FileType = FileType::Text {
    archival_type: FileTypeArchive::Xz,
    encoding_type: FileTypeTextEncoding::Utf8Ascii,
};

// text files
#[test_case(&*NTF_NL_1_PATH, FT_TEXT => panics)]
#[test_case(&*NTF_GZ_EMPTY_FPATH, FT_TEXT_GZ => panics)]
#[test_case(&*NTF_LZ4_8BYTE_FPATH, FT_TEXT_LZ4 => panics)]
#[test_case(&*NTF_TAR_1BYTE_FILEA_FPATH, FT_TEXT_TAR => panics)]
#[test_case(&*NTF_XZ_EMPTY_FPATH, FT_TEXT_XZ => panics)]
// fixedstruct files
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, LINUX_X86_UTMPX_3ENTRY_FILETYPE => panics)]
// unparsable
#[test_case(&*NTF_LOG_EMPTY_FPATH, FileType::Unparsable => panics)]
fn test_decompress_to_ntf_panic(fpath: &FPath, filetype: FileType) {
    // XXX: catch error for newly added FileType or FileTypeArchive variants not yet handled
    match filetype {
        FileType::Unparsable => {}
        FileType::Evtx { archival_type }
        | FileType::FixedStruct { archival_type, .. }
        | FileType::Journal { archival_type }
        | FileType::Text { archival_type, .. }
        => {
            match archival_type {
                FileTypeArchive::Normal
                | FileTypeArchive::Gz
                | FileTypeArchive::Lz4
                | FileTypeArchive::Tar
                | FileTypeArchive::Xz
                => {}
            }
        }
    }
    let path = fpath_to_path(fpath);
    _ = decompress_to_ntf(
        path,
        &filetype,
    );
}
