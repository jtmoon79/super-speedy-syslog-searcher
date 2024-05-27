// filedecompressor_tests.rs

use std::time::SystemTime;

use crate::common::{
    FPath,
    FileSz,
    FileType,
    FileTypeArchive,
};
use crate::readers::filedecompressor::decompress_to_ntf;
use crate::readers::helpers::fpath_to_path;
use crate::tests::common::{
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
    match filetype {
        FileType::Unparsable => {
            panic!();
        }
        FileType::Evtx { archival_type }
        | FileType::FixedStruct { archival_type, .. }
        | FileType::Journal { archival_type }
        | FileType::Text { archival_type, .. }
        => {
            // XXX: catch error for newly added FileTypeArchive variants not yet handled
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
    assert_eq!(
        systemtime, value.1,
        "systemtime differs;\nexpected: {:?}\ngot     : {:?}",
        systemtime, value.1
    );
    assert_eq!(
        filesz, value.2,
        "filesz differs; expected: {}, got: {}",
        filesz, value.2
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
    match filetype {
        FileType::Unparsable => {
            panic!();
        }
        FileType::Evtx { archival_type }
        | FileType::FixedStruct { archival_type, .. }
        | FileType::Journal { archival_type }
        | FileType::Text { archival_type, .. }
        => {
            // XXX: catch error for newly added FileTypeArchive variants not yet handled
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
