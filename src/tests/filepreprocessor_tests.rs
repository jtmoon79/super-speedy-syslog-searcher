// src/tests/filepreprocessor_tests.rs

//! tests for `filepreprocessor.rs` functions

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::tests::common::{
    FILETYPE_EVTX,
    FILETYPE_EVTX_GZ,
    FILETYPE_EVTX_LZ4,
    FILETYPE_EVTX_XZ,
    FILETYPE_JOURNAL,
    FILETYPE_JOURNAL_GZ,
    FILETYPE_JOURNAL_LZ4,
    FILETYPE_JOURNAL_XZ,
    FILETYPE_UTF8,
    FILETYPE_UTF8_GZ,
    FILETYPE_UTF8_XZ,
    NTF_GZ_EMPTY,
    NTF_GZ_EMPTY_FPATH,
    NTF_LOG_EMPTY,
    NTF_LOG_EMPTY_FILETYPE,
    NTF_LOG_EMPTY_FPATH,
    NTF_TAR_1BYTE,
    NTF_TAR_1BYTE_FILEA_FILETYPE,
    NTF_TAR_1BYTE_FILEA_FPATH,
    NTF_TAR_8BYTE_FILEA_FILETYPE,
    NTF_TAR_8BYTE_FILEA_FPATH,
    NTF_TAR_8BYTE_FPATH,
    NTF_TAR_AB_FILEA_FILETYPE,
    NTF_TAR_AB_FILEA_FPATH,
    NTF_TAR_AB_FILEB_FILETYPE,
    NTF_TAR_AB_FILEB_FPATH,
    NTF_TAR_AB_FPATH,
    NTF_TAR_ABCDEFGHI,
    NTF_TAR_ABCDEFGHI_FILEA_FILETYPE,
    NTF_TAR_ABCDEFGHI_FILEA_FPATH,
    NTF_TAR_ABCDEFGHI_FILEB_FILETYPE,
    NTF_TAR_ABCDEFGHI_FILEB_FPATH,
    NTF_TAR_ABCDEFGHI_FILEC_FILETYPE,
    NTF_TAR_ABCDEFGHI_FILEC_FPATH,
    NTF_TAR_ABCDEFGHI_FILED_FILETYPE,
    NTF_TAR_ABCDEFGHI_FILED_FPATH,
    NTF_TAR_ABCDEFGHI_FILEE_FPATH,
    NTF_TAR_ABCDEFGHI_FILEF_FPATH,
    NTF_TAR_ABCDEFGHI_FILEG_FPATH,
    NTF_TAR_ABCDEFGHI_FILEH_FPATH,
    NTF_TAR_ABCDEFGHI_FILEI_FILETYPE,
    NTF_TAR_ABCDEFGHI_FILEI_FPATH,
    NTF_TAR_ABCDEFGHI_FPATH,
    NTF_TGZ_8BYTE,
    NTF_TGZ_8BYTE_FPATH,
};
use crate::common::{
    FileType,
    FileTypeArchive,
    FileTypeFixedStruct,
    FileTypeTextEncoding,
    FPath,
};
use crate::readers::filepreprocessor::{
    copy_process_path_result_canonicalize_path,
    fpath_to_filetype,
    FileTypeArchiveMultiple,
    PathToFiletypeResult,
    process_path,
    process_path_tar,
    ProcessPathResult,
};
use crate::readers::helpers::{fpath_to_path, path_to_fpath};
use crate::debug::helpers::{
    create_files_and_tmpdir,
    create_temp_dir,
    ntf_fpath,
    NamedTempFile,
};

use std::fs::copy;

#[allow(unused_imports)]
use ::filepath::FilePath; // provide `path` function on `File`

use ::si_trace_print::{defn, defo, defx, defñ};
use ::test_case::test_case;


// FileType consts
const FTTN8: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Text {
        archival_type: FileTypeArchive::Normal,
        encoding_type: FileTypeTextEncoding::Utf8Ascii,
    }
);
const FTTB8: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Text {
        archival_type: FileTypeArchive::Bz2,
        encoding_type: FileTypeTextEncoding::Utf8Ascii,
    }
);
const FTTGZ8: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Text {
        archival_type: FileTypeArchive::Gz,
        encoding_type: FileTypeTextEncoding::Utf8Ascii,
    }
);
const FTTXZ8: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Text {
        archival_type: FileTypeArchive::Xz,
        encoding_type: FileTypeTextEncoding::Utf8Ascii,
    }
);
const FTEVTXN: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Evtx {
        archival_type: FileTypeArchive::Normal,
    }
);
const FTEVTXB: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Evtx {
        archival_type: FileTypeArchive::Bz2,
    }
);
const FTEVTXG: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Evtx {
        archival_type: FileTypeArchive::Gz,
    }
);
const FTEVTXL4: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Evtx {
        archival_type: FileTypeArchive::Lz4,
    }
);
const FTEVTXX: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Evtx {
        archival_type: FileTypeArchive::Xz,
    }
);
const FTJOURNALN: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Journal {
        archival_type: FileTypeArchive::Normal,
    }
);
const FTJOURNALB: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Journal {
        archival_type: FileTypeArchive::Bz2,
    }
);
const FTJOURNALG: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Journal {
        archival_type: FileTypeArchive::Gz,
    }
);
const FTJOURNALL4: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Journal {
        archival_type: FileTypeArchive::Lz4,
    }
);
const FTJOURNALX: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Journal {
        archival_type: FileTypeArchive::Xz,
    }
);
const FTUNPARSABLE: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::Unparsable,
);
// PathToFiletypeResult::Archive consts
const AMTARN: PathToFiletypeResult = PathToFiletypeResult::Archive(
    FileTypeArchiveMultiple::Tar,
    FileTypeArchive::Normal,
);
const AMTARB: PathToFiletypeResult = PathToFiletypeResult::Archive(
    FileTypeArchiveMultiple::Tar,
    FileTypeArchive::Bz2,
);
const AMTARG: PathToFiletypeResult = PathToFiletypeResult::Archive(
    FileTypeArchiveMultiple::Tar,
    FileTypeArchive::Gz,
);
const AMTARX: PathToFiletypeResult = PathToFiletypeResult::Archive(
    FileTypeArchiveMultiple::Tar,
    FileTypeArchive::Xz,
);
// FileType::FixedStruct consts
const FTACCTN: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Normal,
        fixedstruct_type: FileTypeFixedStruct::Acct,
    }
);
const FTACCTV3N: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Normal,
        fixedstruct_type: FileTypeFixedStruct::AcctV3,
    }
);
const FTACCTV3B: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Bz2,
        fixedstruct_type: FileTypeFixedStruct::AcctV3,
    }
);
const FTACCTV3G: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Gz,
        fixedstruct_type: FileTypeFixedStruct::AcctV3,
    }
);
const FTACCTV3X: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Xz,
        fixedstruct_type: FileTypeFixedStruct::AcctV3,
    }
);
const FTLASTLOGN: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Normal,
        fixedstruct_type: FileTypeFixedStruct::Lastlog,
    }
);
const FTLASTLOGG: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Gz,
        fixedstruct_type: FileTypeFixedStruct::Lastlog,
    }
);
const FTLASTLOG_X: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Xz,
        fixedstruct_type: FileTypeFixedStruct::Lastlog,
    }
);
const FTLASTLOGXN: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Normal,
        fixedstruct_type: FileTypeFixedStruct::Lastlogx,
    }
);
const FTUTMPN: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Normal,
        fixedstruct_type: FileTypeFixedStruct::Utmp,
    }
);
const FTUTMP_X: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Xz,
        fixedstruct_type: FileTypeFixedStruct::Utmp,
    }
);
const FTUTMPG: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Gz,
        fixedstruct_type: FileTypeFixedStruct::Utmp,
    }
);
const FTUTMPL4: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Lz4,
        fixedstruct_type: FileTypeFixedStruct::Utmp,
    }
);
const FTUTMPXN: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Normal,
        fixedstruct_type: FileTypeFixedStruct::Utmpx,
    }
);
const FTUTMPXG: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Gz,
        fixedstruct_type: FileTypeFixedStruct::Utmpx,
    }
);
const FTUTMPXX: PathToFiletypeResult = PathToFiletypeResult::Filetype(
    FileType::FixedStruct {
        archival_type: FileTypeArchive::Xz,
        fixedstruct_type: FileTypeFixedStruct::Utmpx,
    }
);

// TEXT
#[test_case("log", FTTN8, true)]
#[test_case("LOG", FTTN8, true; "LOG ALLCAPS")]
#[test_case("log.log", FTTN8, true)]
#[test_case("log_media", FTTN8, true)]
#[test_case("log_media", FTTN8, false)]
#[test_case("log_log", FTTN8, true; "log us log true")]
#[test_case("log_log", FTTN8, false; "log us log false")]
#[test_case("media_log", FTTN8, true)]
#[test_case("media_log", FTTN8, false)]
#[test_case("MY_LOG", FTTN8, true)]
#[test_case("media.log.old", FTTN8, true)]
#[test_case("messages", FTTN8, true)]
#[test_case("MESSAGES", FTTN8, true; "MESSAGES ALLCAPS")]
#[test_case("pagefile.sys", FTTN8, true)]
#[test_case("syslog", FTTN8, true)]
#[test_case("syslog~", FTTN8, true; "syslog_tilde")]
#[test_case("syslog-", FTTN8, true; "syslog_dash")]
#[test_case("syslog.3", FTTN8, true)]
#[test_case("syslog.3.20240101", FTTN8, true)]
#[test_case("somefile", FTTN8, true)]
#[test_case("SOMEFILE", FTTN8, true; "SOMEFILE ALLCAPS")]
#[test_case("output.txt", FTTN8, true)]
#[test_case("cloud-init.log.out", FTTN8, true)]
#[test_case("cloud-init.out.log", FTTN8, true)]
#[test_case("cloud-init-output.log", FTTN8, true)]
#[test_case("droplet-agent.update.log", FTTN8, true)]
#[test_case("kern.log", FTTN8, true)]
#[test_case("KERN.LOG", FTTN8, true; "KERN.LOG ALLCAPS")]
#[test_case("kern.log.1", FTTN8, true)]
#[test_case("kern.log.2", FTTN8, true)]
#[test_case("kern.log.2~", FTTN8, true; "kern.log.2_tilde")]
#[test_case("rhsm.log-20230422", FTTN8, true)]
#[test_case("aptitude.4", FTTN8, true)]
#[test_case("aptitude.~", FTTN8, true; "aptitude_tilde")]
#[test_case("systemsetup-server-info.log.208", FTTN8, true)]
#[test_case("a.log", FTTN8, true)]
#[test_case("log.a", FTTN8, true)]
#[test_case("LOG.B", FTTN8, true)]
#[test_case("log.1", FTTN8, true)]
#[test_case("log.2", FTTN8, true)]
#[test_case("HOSTNAME.log", FTTN8, true)]
#[test_case("log.HOSTNAME", FTTN8, true)]
#[test_case("log.nmbd", FTTN8, true)]
#[test_case("LOG.NMDB", FTTN8, true; "LOG.NMDB")]
#[test_case("log.nmbd.1", FTTN8, true)]
#[test_case("log.nmbd.old", FTTN8, true)]
#[test_case("null", FTTN8, false)]
#[test_case("null", FTTN8, true)]
#[test_case("nul", FTTN8, false)]
#[test_case("nul", FTTN8, true)]
#[test_case("soap_agent", FTTN8, true)]
#[test_case("soap_agent.old", FTTN8, true)]
#[test_case("soap_agent.old.old", FTTN8, true)]
#[test_case("2023.10.26.asl", FTTN8, true)]
#[test_case("-", FTTN8, true; "dash")]
#[test_case("-", FTUNPARSABLE, false; "dash false")]
#[test_case("$", FTTN8, true; "dollar")]
#[test_case("$", FTTN8, false; "dollar false")]
#[test_case("$$", FTTN8, true; "dollar dollar")]
#[test_case("_", FTTN8, true; "underscore")]
#[test_case("_", FTTN8, false; "underscore false")]
#[test_case("__", FTTN8, true; "underscore underscore")]
#[test_case("__", FTTN8, false; "underscore underscore false")]
#[test_case("telemetry", FTTN8, true)]
#[test_case("initial-status", FTTN8, true)]
#[test_case("smart_extend_log", FTTN8, true)]
#[test_case(".disk_daily_info_send_udc_time", FTTN8, true)]
#[test_case("messages-DropletAgent", FTTN8, true)]
#[test_case("CC_AA_DD_EE_FF_00-ns", FTTN8, true)]
#[test_case("CC_AA_DD_EE_FF_00-ns.old", FTTN8, true)]
#[test_case("CC_AA_DD_EE_FF_00-ns.old.1", FTTN8, true)]
#[test_case("history", FTTN8, true)]
#[test_case("fe80::984c:ffff:eeee:eeee.log", FTTN8, true)]
#[test_case("[fe80::984c:ffff:eeee:eeef].log", FTTN8, true)]
#[test_case("錄音.log", FTTN8, true)]
#[test_case("opname.log", FTTN8, true)]
#[test_case("บันทึก.log", FTTN8, true)]
#[test_case("innspilling.log", FTTN8, true)]
#[test_case("Запису.log", FTTN8, true)]
#[test_case("تسجيل.log", FTTN8, true)]
#[test_case("grabación.log", FTTN8, true)]
#[test_case("錄音.檔", FTTN8, true)]
#[test_case("錄音", FTTN8, true)]
#[test_case("錄音.log", FTTN8, true; "CC dot log")]
#[test_case("錄音log", FTTN8, true; "CC log")]
#[test_case("log錄音", FTTN8, true)]
#[test_case("บันทึก", FTTN8, true)]
#[test_case("innspilling", FTTN8, true)]
#[test_case("Запису", FTTN8, true)]
#[test_case("تسجيل", FTTN8, true)]
#[test_case("grabación", FTTN8, true)]
#[test_case("192.168.1.100.log", FTTN8, true)]
#[test_case("log.192.168.1.100", FTTN8, true)]
#[test_case("setup.log.full", FTTN8, true)]
#[test_case("setup.log.full.1", FTTN8, true)]
#[test_case("setup.log.full.old", FTTN8, true)]
#[test_case("setup.log.full.old.1", FTTN8, true)]
#[test_case("setup.log.full.old.2", FTTN8, true)]
#[test_case("SIH.20230422.034724.362.1.etl", FTTN8, true)]
// TEXT bz2
#[test_case("syslog.bz2", FTTB8, true)]
#[test_case("syslog.1.bz2", FTTB8, true)]
#[test_case("unattended-upgrades-dpkg.log.3.bz2", FTTB8, true)]
#[test_case("data.bz2", FTTB8, true)]
#[test_case("data.log.bz2", FTTB8, true)]
#[test_case("DATA.BZ2", FTTB8, true; "DATA.BZ2 ALLCAPS")]
#[test_case("data.bz2.old", FTTB8, true)]
#[test_case("log.bz2.1", FTTB8, true)]
#[test_case("log.bz2.2", FTTB8, true)]
#[test_case("log.bz2.99", FTTB8, true)]
#[test_case("log.nmbd.old.bz2", FTTB8, true)]
#[test_case("192.168.1.100.log.bz2", FTTB8, true)]
#[test_case("192.168.1.100.log.bz2.1", FTTB8, true)]
#[test_case("192.168.1.100.log.bz2.old.1", FTTB8, true)]
// TEXT gz
#[test_case("syslog.gz", FTTGZ8, true)]
#[test_case("syslog.9.gz", FTTGZ8, true)]
#[test_case("SYSLOG.9.GZ", FTTGZ8, true; "SYSLOG.9.GZ")]
#[test_case("unattended-upgrades-dpkg.log.3.gz", FTTGZ8, true)]
#[test_case("data.gz", FTTGZ8, true)]
#[test_case("data.log.gz", FTTGZ8, true)]
#[test_case("DATA.GZ", FTTGZ8, true; "DATA.GZ ALLCAPS")]
#[test_case("data.gz.old", FTTGZ8, true)]
#[test_case("data.gzip", FTTGZ8, true)]
#[test_case("log.gz.1", FTTGZ8, true)]
#[test_case("log.gz.2", FTTGZ8, true)]
#[test_case("log.gz.99", FTTGZ8, true)]
#[test_case("log.nmbd.old.gz", FTTGZ8, true)]
#[test_case("192.168.1.100.log.gz", FTTGZ8, true)]
#[test_case("192.168.1.100.log.gz.1", FTTGZ8, true)]
#[test_case("192.168.1.100.log.gz.old.1", FTTGZ8, true)]
// oddities
#[test_case("_.gz", FTTGZ8, true)]
// TEXT xz
#[test_case("eipp.log.xz", FTTXZ8, true)]
#[test_case("eipp.xz", FTTXZ8, true)]
// oddities
#[test_case("-.xz", FTTXZ8, true; "dash xz")]
#[test_case("--.xz", FTTXZ8, true; "dash dash xz")]
#[test_case("~.xz", FTTXZ8, true; "tilde xz")]
#[test_case("~.xz~", FTTXZ8, true; "tilde xz tilde")]
#[test_case("_.xz", FTTXZ8, true)]
// TEXT tar
#[test_case("my.logs.tar", AMTARN, true)]
// oddities
#[test_case("-.logs.tar", AMTARN, true; "dash dot logs dot tar")]
#[test_case("-.tar", AMTARN, true; "dash dot tar")]
//
// TAR
//
#[test_case("data.tar", AMTARN, true)]
#[test_case("data.xz.tar", AMTARN, true)]
#[test_case("DATA.TAR", AMTARN, true; "DATA.TAR ALLCAPS")]
#[test_case("data.tar.old", AMTARN, true)]
#[test_case("logs.tar", AMTARN, true)]
#[test_case("LOGS.TAR", AMTARN, true; "LOGS.TAR")]
#[test_case("log.1.tar", AMTARN, true)]
#[test_case("utmp.tar", AMTARN, true)]
#[test_case("LOG.1.TAR", AMTARN, true; "LOG.1.TAR ALLCAPS")]
#[test_case("tar.tar", AMTARN, true)]
#[test_case("tgz.tar", AMTARN, true)]
#[test_case("_.tar", AMTARN, true)]
// bz2
#[test_case("data.tar.bz2", AMTARB, true)]
// gz
#[test_case("data.tar.gz", AMTARG, true)]
// xz
#[test_case("data.tar.xz", AMTARX, true)]
//
// FIXEDSTRUCT
//
// FixedStruct Utmp
#[test_case("wtmp", FTUTMPN, true; "wtmp")]
#[test_case("WTMP", FTUTMPN, true; "WTMP ALLCAPS")]
#[test_case("btmp", FTUTMPN, true; "btmp")]
#[test_case("utmp", FTUTMPN, true; "utmp")]
#[test_case("UTMP", FTUTMPN, true; "UTMP ALLCAPS")]
#[test_case("UTMP.1", FTUTMPN, true; "UTMP.1 ALLCAPS")]

#[test_case("UTMP.gz", FTUTMPG, true; "UTMP ALLCAPS GZ")]
#[test_case("UTMP.tar", AMTARN, true; "UTMP ALLCAPS TAR")]
#[test_case("UTMP.xz", FTUTMP_X, true; "UTMP ALLCAPS XZ")]
#[test_case("UTMP.1.GZ", FTUTMPG, true; "UTMP.1.GZ")]
#[test_case("UTMP.1.XZ", FTUTMP_X, true; "UTMP.1.XZ")]
#[test_case("UTMP.2.lz4", FTUTMPL4, true)]
// FixedStruct Utmpx
#[test_case("btmpx", FTUTMPXN, true)]
#[test_case("utmpx", FTUTMPXN, true)]
#[test_case("wtmpx", FTUTMPXN, true)]
#[test_case("wtmpx~",FTUTMPXN, true; "wtmpx_tilde")]
#[test_case("btmp.1", FTUTMPN, true)]
#[test_case("utmp.2", FTUTMPN, true)]
#[test_case("wtmp.1", FTUTMPN, true)]
#[test_case("WTMP.1", FTUTMPN, true; "WTMP.1 ALLCAPS")]
#[test_case("host.wtmp", FTUTMPN, true)]
#[test_case("192.168.1.1.btmp", FTUTMPN, true)]
#[test_case("file.utmp", FTUTMPN, true)]
#[test_case("btmpx", FTUTMPXN, true; "btmpx")]
#[test_case("utmpx", FTUTMPXN, true; "utmpx")]
#[test_case("utmpx.bak", FTUTMPXN, true; "utmpx.bak")]
#[test_case("utmpx.2.bak", FTUTMPXN, true; "utmpx.2.bak")]
#[test_case("wtmpx", FTUTMPXN, true; "wtmpx")]
#[test_case("wtmpx.1", FTUTMPXN, true; "wtmpx.1")]
#[test_case("btmpx.xz", FTUTMPXX, true; "btmpx dot xz")]
#[test_case("btmpx.gz", FTUTMPXG, true; "btmpx dot gz")]
// FixedStruct Lastlog
#[test_case("lastlog", FTLASTLOGN, true)]
#[test_case("lastlogx", FTLASTLOGXN, true)]
#[test_case("lastlog.1", FTLASTLOGN, true)]
#[test_case("lastlog.bak", FTLASTLOGN, true)]
#[test_case("lastlog.2.bak", FTLASTLOGN, true)]
#[test_case("lastlog.gz", FTLASTLOGG, true)]
#[test_case("lastlog.XZ", FTLASTLOG_X, true)]
#[test_case("lastlog.1.XZ", FTLASTLOG_X, true)]
// FixedStruct Acct
#[test_case("acct", FTACCTN, true)]
#[test_case("acct.2", FTACCTN, true)]
#[test_case("acct-20220101", FTTN8, true)]
#[test_case("pacct", FTACCTV3N, true)]
#[test_case("pacct.1", FTACCTV3N, true)]
#[test_case("pacct.20220101", FTACCTV3N, true)]
#[test_case("pacct.bz2", FTACCTV3B, true)]
#[test_case("pacct.gz", FTACCTV3G, true)]
#[test_case("pacct.20220101.gz", FTACCTV3G, true)]
#[test_case("pacct.xz", FTACCTV3X, true)]
// on FreeBSD 13, there is a log file `utx.log` that is a variable-length utmpx-ish format file
#[test_case("utx.log", FTTN8, true)]
#[test_case("utx.log-", FTTN8, true; "utx.log dash")]
#[test_case("utx.active", FTTN8, true)]
// File `utx.lastlogin` exists on FreeBSD 13.   
#[test_case("utx.lastlogin", FTTN8, true)]
//
// EVTX
//
#[test_case("file.evtx", FTEVTXN, true)]
#[test_case("FILE.EVTX", FTEVTXN, true; "FILE.EVTX ALLCAPS")]
#[test_case("file.evtx.1", FTEVTXN, true)]
#[test_case("file.xz.evtx.1", FTEVTXN, true)]
#[test_case("xz.evtx", FTEVTXN, true)]
#[test_case("tar.evtx", FTEVTXN, true)]
#[test_case("mp3.evtx", FTEVTXN, true)]
#[test_case("_.evtx", FTEVTXN, true)]
// gz
#[test_case("log.evtx.bz2", FTEVTXB, true)]
// gz
#[test_case("log.evtx.gz", FTEVTXG, true)]
// lz4
#[test_case("log.evtx.lz4", FTEVTXL4, true)]
// xz
#[test_case("log.evtx.xz", FTEVTXX, true)]
#[test_case("tar.evtx.xz", FTEVTXX, true)]
//
// JOURNAL
//
#[test_case("a.journal", FTJOURNALN, true)]
#[test_case("A.JOURNAL", FTJOURNALN, true; "A.JOURNAL ALLCAPS")]
#[test_case("a.journal~", FTJOURNALN, true; "a.journal tilde")]
#[test_case("a.journal~.1", FTJOURNALN, true; "a.journal tilde 1")]
#[test_case("a.journal~.old", FTJOURNALN, true; "a.journal tilde old")]
#[test_case("A.JOURNAL~", FTJOURNALN, true; "A.JOURNAL ALLCAPS tilde")]
#[test_case("user-1000.journal", FTJOURNALN, true)]
#[test_case("user-1000@2feff012228b405bb557ccd80a0ba755-000000005100032b-0006129e5481135e.journal", FTJOURNALN, true)]
#[test_case("system@a8b80590f2654a95aed5c11b3c9e3c48-0000000000000001-0005f6f737b6b0e0.journal", FTJOURNALN, true)]
// bz2
#[test_case("user-1000.journal.BZ2", FTJOURNALB, true)]
// gz
#[test_case("user-1000.journal.gz", FTJOURNALG, true)]
#[test_case("journal.journal.gz", FTJOURNALG, true)]
#[test_case("journal.gz", FTJOURNALG, true)]
#[test_case("journal.gz.xz", FTJOURNALG, true)] // Issue #14
// lz4
#[test_case("user-1000.journal.lz4", FTJOURNALL4, true)]
// xz
#[test_case("user-1000.journal.xz", FTJOURNALX, true)]
#[test_case("journal.journal.xz", FTJOURNALX, true)]
#[test_case("journal.xz", FTJOURNALX, true)]
#[test_case("-.journal.xz", FTJOURNALX, true; "dash dot journal dot xz")]
#[test_case("--.journal.xz", FTJOURNALX, true; "dash dash dot journal dot xz")]
#[test_case(".journal.xz", FTJOURNALX, true; "dot journal dot xz")]
#[test_case("system@a8b80590f2654a95aed5c11b3c9e3c48-0000000000000001-0005f6f737b6b0e0.journal.xz", FTJOURNALX, true)]
#[test_case("journal.xz.gz", FTJOURNALX, true)] // Issue #14
//
// Unparseable
//
#[test_case("data.tgz", FTTN8, true)]
#[test_case("data.tgz", FTUNPARSABLE, false)]
#[test_case("data.tgz.old", FTUNPARSABLE, false)]
#[test_case("data.tgz.old", FTTN8, true)]
#[test_case("data.tgz.old.1", FTUNPARSABLE, false)]
#[test_case("data.tgz.old.1", FTTN8, true)]
#[test_case("lib.dll", FTUNPARSABLE, false)]
#[test_case("lib.dll", FTTN8, true)]
#[test_case("log.bz", FTUNPARSABLE, false)]
#[test_case("log.bz", FTTN8, true)]
#[test_case("logs.tgz", FTTN8, true)]
#[test_case("logs.tgz", FTUNPARSABLE, false)]
#[test_case("log.tgz.99", FTUNPARSABLE, false)]
#[test_case("log.tgz.99", FTTN8, true)]
#[test_case("logs.2.zip", FTUNPARSABLE, false)]
#[test_case("logs.2.zip", FTTN8, true)]
#[test_case("logs.tgz.99", FTUNPARSABLE, false)]
#[test_case("logs.tgz.99", FTTN8, true)]
#[test_case("LOGS.TGZ.99", FTUNPARSABLE, false; "LOGS.TGZ.99 ALLCAPS unparsable")]
#[test_case("LOGS.TGZ.99", FTTN8, true; "LOGS.TGZ.99 ALLCAPS filetype_utf8")]
#[test_case("logs.xz.zip", FTUNPARSABLE, false)]
#[test_case("logs.xz.zip", FTTN8, true)]
#[test_case("logs.zip", FTUNPARSABLE, false)]
#[test_case("logs.zip", FTTN8, true)]
#[test_case("logs.zip.2", FTUNPARSABLE, false)]
#[test_case("logs.zip.2", FTTN8, true)]
#[test_case("media.avi", FTUNPARSABLE, false)]
#[test_case("media.avi", FTTN8, true)]
#[test_case("media.mp3", FTUNPARSABLE, false)]
#[test_case("media.mp3", FTTN8, true)]
#[test_case("mp3", FTTN8, false)]
#[test_case("mp3", FTTN8, true)]
#[test_case("media.mp4", FTUNPARSABLE, false)]
#[test_case("media.mp4", FTTN8, true)]
#[test_case("pic.jpg", FTUNPARSABLE, false)]
#[test_case("pic.jpg", FTTN8, true)]
#[test_case("pic.png", FTUNPARSABLE, false)]
#[test_case("pic.png", FTTN8, true)]
#[test_case("prog.exe", FTUNPARSABLE, false)]
#[test_case("prog.exe", FTTN8, true)]
// oddities
#[test_case("bz2", FTTN8, true)]
#[test_case("bz2", FTTN8, false)]
#[test_case("gz", FTTN8, true)]
#[test_case("gz", FTTN8, false)]
#[test_case("lz4", FTTN8, true)]
#[test_case("lz4", FTTN8, false)]
#[test_case("xz", FTTN8, true)]
#[test_case("xz", FTTN8, false)]
#[test_case("-.tgz.99", FTUNPARSABLE, false; "dash tgz 99 Unparsable")]
#[test_case("-.tgz.99", FTTN8, true; "dash tgz 99 FILETYPE_UTF8")]
#[test_case("-", FTTN8, true; "dash1 FILETYPE_UTF8")]
#[test_case("-", FTUNPARSABLE, false; "dash1 Unparsable")]
#[test_case("--", FTTN8, true; "dash2 FILETYPE_UTF8")]
#[test_case("--", FTUNPARSABLE, false; "dash2 Unparsable")]
#[test_case("?", FTTN8, true; "question1 FILETYPE_UTF8")]
#[test_case("?", FTUNPARSABLE, false; "question1 Unparsable")]
#[test_case("~", FTTN8, true; "tilde1 FILETYPE_UTF8")]
#[test_case("~", FTUNPARSABLE, false; "tilde1 Unparsable")]
#[test_case("~~", FTTN8, true; "tilde2 FILETYPE_UTF8")]
#[test_case("~~", FTUNPARSABLE, false; "tilde2 Unparsable")]
// XXX: case `.` is unusual
//      path.file_name() resolves '/var/log/.' to 'log' which is `FileType::Text``
//      path.file_name() '.' to '' which is `FileType::Unparsable`
#[test_case(".", FTUNPARSABLE, false; "dot1 Unparsable")]
#[test_case(".", FTTN8, true; "dot1 FILETYPE_UTF8")]
#[test_case("..", FTUNPARSABLE, false; "dot2 Unparsable")]
#[test_case("..", FTTN8, true; "dot2 FILETYPE_UTF8")]
#[test_case("...", FTUNPARSABLE, false; "dot3 Unparsable")]
#[test_case("...", FTTN8, true; "dot3 FILETYPE_UTF8")]
#[test_case("....", FTUNPARSABLE, false; "dot4 Unparsable")]
#[test_case("....", FTTN8, true; "dot4 FILETYPE_UTF8")]
#[test_case(".....", FTUNPARSABLE, false; "dot5 Unparsable")]
#[test_case(".....", FTTN8, true; "dot5 FILETYPE_UTF8")]
fn test_fpath_to_filetype(
    fname: &str,
    expect_result: PathToFiletypeResult,
    unparseable_are_text: bool,
) {
    let fpath: FPath = FPath::from(fname);
    defo!("fpath_to_filetype(fpath={:?}, unparseable_are_text={:?})", fpath, unparseable_are_text);
    let result: PathToFiletypeResult = fpath_to_filetype(&fpath, unparseable_are_text);
    defo!("fpath_to_filetype returned {:?}", result);
    let (filetype_expect, filetype_result) = match result {
        PathToFiletypeResult::Filetype(ftr) => {
            match expect_result {
                PathToFiletypeResult::Filetype(fte) => {
                    (fte, ftr)
                },
                PathToFiletypeResult::Archive(_ftam, _fta) => {
                    panic!("Expected PathToFiletypeResult::Archive, got PathToFiletypeResult::FileType");
                },
            }
        },
        PathToFiletypeResult::Archive(_ftam, _fta) => {
            match expect_result {
                PathToFiletypeResult::Filetype(ft) => {
                    panic!("Expected FileType::{:?}, got PathToFiletypeResult::Archive", ft);
                },
                PathToFiletypeResult::Archive(_ftam, _fta) => {
                    defx!();
                    return;
                },
            }
        }
    };
    defo!("filetype {:?}", filetype_result);
    assert_eq!(
        filetype_expect, filetype_result,
        "\npath {:?}\nexpected FileType::{:?}\nactual FileType::{:?}\n",
        fpath, filetype_expect, filetype_result
    );

    // test again with leading path `/var/log`

    // handle special case of `"."`
    if fname == "." {
        return;
    }
    let fpath_full = FPath::from("/var/log/") + fname;
    defo!("fpath_to_filetype(fpath_full={:?}, unparseable_are_text={:?})", fpath_full, unparseable_are_text);
    let result = fpath_to_filetype(&fpath_full, unparseable_are_text);
    defo!("fpath_to_filetype returned {:?}", result);
    let (filetype_expect, filetype_result) = match result {
        PathToFiletypeResult::Filetype(ft) => {
            match expect_result {
                PathToFiletypeResult::Filetype(fte) => {
                    (fte, ft)
                },
                PathToFiletypeResult::Archive(_ftam, _fta) => {
                    panic!("Expected PathToFiletypeResult::Archive, got PathToFiletypeResult::FileType");
                },
            }
        },
        PathToFiletypeResult::Archive(_ftam, _fta) => {
            match expect_result {
                PathToFiletypeResult::Filetype(ft) => {
                    panic!("Expected FileType::{:?}, got PathToFiletypeResult::Archive", ft);
                },
                PathToFiletypeResult::Archive(_ftam, _fta) => {
                    defx!();
                    return;
                },
            }
        }
    };
    defo!("filetype {:?}", filetype_result);
    assert_eq!(
        filetype_expect, filetype_result,
        "\npath {:?}\nexpected FileType::{:?}\nactual FileType::{:?}\n",
        fpath_full, filetype_expect, filetype_result
    );
}

fn test_process_path_fpath(
    path: &FPath,
    checks: &Vec<ProcessPathResult>,
    unparseable_are_text: bool,
) {
    defn!("({:?}, …, unparseable_are_text={:?})", path, unparseable_are_text);
    let results = process_path(path, unparseable_are_text);
    for (i, result) in results.iter().enumerate() {
        defo!("result[{}] = {:?}", i, result);
    }
    // XXX: create a copy of `results`, canonicalize every `fpath` within each `ProcessPathResult`
    //      on some Windows systems, `process_path` will return an MS-DOS shortened form of a path
    //      e.g. `"C:\\Users\\RUNNER~1\\AppData\\Local\\Temp\\.tmp6TC2W5\\file1"`
    //           !=
    //           `"C:\\Users\\runneradmin\\AppData\\Local\\Temp\\.tmp6TC2W5\\file1"`
    //      So try harder to make sure the comparison succeeds.
    let mut results_can: Vec<ProcessPathResult> = vec![];
    for result in results.into_iter() {
        let result_can = copy_process_path_result_canonicalize_path(result);
        results_can.push(result_can);
    }
    for (i, result_can) in results_can.iter().enumerate() {
        defo!("result_can[{}] = {:?}", i, result_can);
    }
    // create copy of `checks` for the same reason as `results_can` above
    let mut checks_can: Vec<ProcessPathResult> = vec![];
    for check in checks.iter() {
        let check_can = copy_process_path_result_canonicalize_path(check.clone());
        checks_can.push(check_can);
    }
    for (i, check_can) in checks_can.iter().enumerate() {
        defo!("check_can[{}] = {:?}", i, check_can);
    }
    // check that each `check` is in the `results`
    for (i, check) in checks_can.iter().enumerate() {
        defo!("check[{}] = {:?}", i, check);
        let mut results_can_s: String = String::from("[\n");
        for (i, result_can) in results_can.iter().enumerate() {
            results_can_s.push_str(&format!("    [{}] {:?}\n", i, result_can));
        };
        results_can_s.push_str("  ]");
        assert!(
            results_can.contains(check),
            "\nprocess_path({:?})\n  the check {:?}\n  is not contained in the results:\n  {}\n",
            path,
            check,
            results_can_s,
        );
    }
    // check that each `result` is in the `checks`
    for (i, result) in results_can.iter().enumerate() {
        defo!("result[{}] = {:?}", i, result);
        assert!(
            checks_can.contains(result),
            "\nprocess_path({:?})\n  the result {:?}\n  is not contained in the checks:\n       {:?}\n",
            path,
            result,
            checks_can,
        );
    }
    defx!();
}

fn test_process_path_ntf(
    ntf: &NamedTempFile,
    checks: &Vec<ProcessPathResult>,
    unparseable_are_text: bool,
) {
    let path = ntf_fpath(ntf);
    test_process_path_fpath(&path, checks, unparseable_are_text);
}

// test individual files

#[test]
fn test_process_path_files_log() {
    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_LOG_EMPTY_FPATH.clone(),
            NTF_LOG_EMPTY_FILETYPE,
        ),
    ];
    test_process_path_ntf(&NTF_LOG_EMPTY, &checks, true);
    test_process_path_ntf(&NTF_LOG_EMPTY, &checks, false);
}

#[test]
fn test_process_path_files_gz() {
    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_GZ_EMPTY_FPATH.clone(),
            FILETYPE_UTF8_GZ,
        ),
    ];
    test_process_path_ntf(&NTF_GZ_EMPTY, &checks, true);
    test_process_path_ntf(&NTF_GZ_EMPTY, &checks, false);
}

#[test]
fn test_process_path_files_tar() {
    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TAR_1BYTE_FILEA_FPATH.clone(),
            NTF_TAR_1BYTE_FILEA_FILETYPE,
        ),
    ];
    test_process_path_ntf(&NTF_TAR_1BYTE, &checks, true);
    test_process_path_ntf(&NTF_TAR_1BYTE, &checks, false);
}

#[test]
fn test_process_path_files_tgz_true() {
    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TGZ_8BYTE_FPATH.clone(),
            FILETYPE_UTF8,
        ),
    ];
    test_process_path_ntf(&NTF_TGZ_8BYTE, &checks, true);
}

#[test]
fn test_process_path_file_nonparseable() {
    // TODO: pass in `file.dll`
    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TGZ_8BYTE_FPATH.clone(),
            FILETYPE_UTF8,
        ),
    ];
    test_process_path_ntf(&NTF_TGZ_8BYTE, &checks, true);
}

#[test]
fn test_process_path_files_not_exist_file() {
    let path: FPath = FPath::from("/THIS/FILE/DOES/NOT/EXIST!");
    let checks: Vec<ProcessPathResult> = vec![ProcessPathResult::FileErrNotExist(path.clone())];
    test_process_path_fpath(&path, &checks, true);
    test_process_path_fpath(&path, &checks, false);
}

#[test]
fn test_process_path_files_not_exist_dir() {
    let path: FPath = FPath::from("/THIS/DIRECTORY/DOES/NOT/EXIST/");
    let checks: Vec<ProcessPathResult> = vec![ProcessPathResult::FileErrNotExist(path.clone())];
    test_process_path_fpath(&path, &checks, true);
    test_process_path_fpath(&path, &checks, false);
}

#[test]
fn test_process_path_files_devnull() {
    let fpath: FPath = FPath::from("/dev/null");
    // do not test if path does not exist; avoids failures on unusual platforms
    if !fpath_to_path(&fpath).exists() {
        defo!("Path '{:?}' does not exist, pass test", fpath);
        return;
    }
    let checks: Vec<ProcessPathResult> = vec![ProcessPathResult::FileErrNotAFile(fpath.clone())];
    test_process_path_fpath(&fpath, &checks, true);
    test_process_path_fpath(&fpath, &checks, false);
}

#[test]
fn test_process_path_files_devzero() {
    let fpath: FPath = FPath::from("/dev/zero");
    // do not test if path does not exist; avoids failures on unusual platforms
    if !fpath_to_path(&fpath).exists() {
        defo!("Path '{:?}' does not exist, pass test", fpath);
        return;
    }
    let checks: Vec<ProcessPathResult> = vec![ProcessPathResult::FileErrNotAFile(fpath.clone())];
    test_process_path_fpath(&fpath, &checks, true);
    test_process_path_fpath(&fpath, &checks, false);
}

#[test]
fn test_process_path_files_tgz_false() {
    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TGZ_8BYTE_FPATH.clone(),
            FILETYPE_UTF8,
        ),
    ];
    test_process_path_ntf(&NTF_TGZ_8BYTE, &checks, false);
}

#[test]
fn test_process_path_files_dll_false() {

    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TGZ_8BYTE_FPATH.clone(),
            FILETYPE_UTF8,
        ),
    ];
    test_process_path_ntf(&NTF_TGZ_8BYTE, &checks, false);
}

// test directories of files

#[test]
fn test_process_path_dirs_file1() {
    let filenames = &[FPath::from("file1")];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> =
        vec![ProcessPathResult::FileValid(fpaths.get(0).unwrap().clone(), FILETYPE_UTF8)];

    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, true);
    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, false);
}

#[test]
fn test_process_path_dirs_file1_txt1_evtx1_journal1() {
    let filenames = &[
        FPath::from("file1"),
        FPath::from("file2.txt"),
        FPath::from("file3.evtx"),
        FPath::from("file4.journal"),
        FPath::from("file5.dll"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(fpaths.get(0).unwrap().clone(), FILETYPE_UTF8),
        ProcessPathResult::FileValid(fpaths.get(1).unwrap().clone(), FILETYPE_UTF8),
        ProcessPathResult::FileValid(fpaths.get(2).unwrap().clone(), FILETYPE_EVTX),
        ProcessPathResult::FileValid(fpaths.get(3).unwrap().clone(), FILETYPE_JOURNAL),
        ProcessPathResult::FileErrNotSupported(fpaths.get(4).unwrap().clone(), None),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, true);
    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, false);
}

#[test]
fn test_process_path_dirs_gz1_tar1_txt1_journal1() {
    let filenames = &[
        FPath::from("file1.gz"),
        FPath::from("file2.tar"),
        FPath::from("file3.txt"),
        FPath::from("file4.journal"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(fpaths.get(0).unwrap().clone(), FILETYPE_UTF8_GZ),
        // no .tar file in results
        ProcessPathResult::FileValid(fpaths.get(2).unwrap().clone(), FILETYPE_UTF8),
        ProcessPathResult::FileValid(fpaths.get(3).unwrap().clone(), FILETYPE_JOURNAL),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, true);
    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, false);
}

#[test]
fn test_process_path_dirs_dirA_fileA1() {
    let filenames = &[FPath::from(
        "dirA/fileA1.txt",
    )];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> =
        vec![ProcessPathResult::FileValid(fpaths.get(0).unwrap().clone(), FILETYPE_UTF8)];

    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, true);
    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, false);
}

#[test]
fn test_process_path_dirs_dirABC_files3() {
    let filenames = &[
        FPath::from("file1.txt"),
        FPath::from("dirA/fileA1.txt"),
        FPath::from("dirA/fileA2.gz"),
        FPath::from("dirB/"),
        FPath::from("dirC/"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(fpaths.get(0).unwrap().clone(), FILETYPE_UTF8),
        ProcessPathResult::FileValid(fpaths.get(1).unwrap().clone(), FILETYPE_UTF8),
        ProcessPathResult::FileValid(fpaths.get(2).unwrap().clone(), FILETYPE_UTF8_GZ),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, true);
    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, false);
}

#[test]
fn test_process_path_dirs_dirABC_files6() {
    let filenames = &[
        FPath::from("dirA1/dirA2/fileA12.tar"),
        FPath::from("dirB/fileB1.gz"),
        FPath::from("dirB/fileB2.xz"),
        FPath::from("dirB/fileB3.xz.tar"),
        FPath::from("dirB/fileB4.tar.xz"),
        FPath::from("dirC/fileC1.tgz"),
        FPath::from("dirC/fileC2.journal"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        // fileA12.tar will not be in results
        ProcessPathResult::FileValid(fpaths.get(1).unwrap().clone(), FILETYPE_UTF8_GZ),
        ProcessPathResult::FileValid(fpaths.get(2).unwrap().clone(), FILETYPE_UTF8_XZ),
        // fileB3.xz.tar will not be in results
        // fileB4.tar.xz will not be in results
        ProcessPathResult::FileErrNotSupported(fpaths.get(5).unwrap().clone(), None),
        ProcessPathResult::FileValid(fpaths.get(6).unwrap().clone(), FILETYPE_JOURNAL),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, true);
    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, false);
}

#[test]
fn test_process_path_dirs_dirAB_files4() {
    let filenames = &[
        FPath::from("dirA1/system@f2e8a336aa58640aa39cac58b6ffc7e7-0000000000294e62-0d05dc1215b8e84c.journal"),
        FPath::from("dirB/picture.bmp"),
        FPath::from("dirB/picture.png"),
        FPath::from("dirB/this.crazy.file.name.has.many.extensions.chars.within.the.name"),
        FPath::from("dirB/fileB.evtx"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            fpaths.get(0).unwrap().clone(), FILETYPE_JOURNAL
        ),
        ProcessPathResult::FileErrNotSupported(
            fpaths.get(1).unwrap().clone(),
            None,
        ),
        ProcessPathResult::FileErrNotSupported(
            fpaths.get(2).unwrap().clone(),
            None,
        ),
        ProcessPathResult::FileValid(
            fpaths.get(3).unwrap().clone(), FILETYPE_UTF8
        ),
        ProcessPathResult::FileValid(
            fpaths.get(4).unwrap().clone(), FILETYPE_EVTX
        ),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, true);
}

#[test]
fn test_process_path_dirs_dirAB_files4_false() {
    let filenames = &[
        FPath::from("dirA1/system@f2e8a336aa58640aa39cac58b6ffc7e7-0000000000294e62-0d05dc1215b8e84c.journal"),
        FPath::from("dirB/picture.bmp"),
        FPath::from("dirB/picture.png"),
        FPath::from("dirB/this.crazy.file.name.has.many.extensions.chars.within.the.name"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            fpaths.get(0).unwrap().clone(), FILETYPE_JOURNAL
        ),
        ProcessPathResult::FileErrNotSupported(
            fpaths.get(1).unwrap().clone(),
            None,
        ),
        ProcessPathResult::FileErrNotSupported(
            fpaths.get(2).unwrap().clone(),
            None,
        ),
        ProcessPathResult::FileValid(
            fpaths.get(3).unwrap().clone(), FILETYPE_UTF8
        ),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, false);
}

#[test]
fn test_process_path_evtx_gz_lz4_xz() {
    let filenames = &[
        FPath::from("dirA/user.evtx"),
        FPath::from("dirA/user.evtx.gz"),
        FPath::from("dirA/user.evtx.lz4"),
        FPath::from("dirA/user.evtx.xz"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            fpaths.get(0).unwrap().clone(), FILETYPE_EVTX,
        ),
        ProcessPathResult::FileValid(
            fpaths.get(1).unwrap().clone(), FILETYPE_EVTX_GZ,
        ),
        ProcessPathResult::FileValid(
            fpaths.get(2).unwrap().clone(), FILETYPE_EVTX_LZ4,
        ),
        ProcessPathResult::FileValid(
            fpaths.get(3).unwrap().clone(), FILETYPE_EVTX_XZ,
        ),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, false);
}

#[test]

fn test_process_path_journal_gz_lz4_xz() {
    let filenames = &[
        FPath::from("dirA/user.journal"),
        FPath::from("dirA/user.journal.gz"),
        FPath::from("dirA/user.journal.lz4"),
        FPath::from("dirA/user.journal.xz"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            fpaths.get(0).unwrap().clone(), FILETYPE_JOURNAL,
        ),
        ProcessPathResult::FileValid(
            fpaths.get(1).unwrap().clone(), FILETYPE_JOURNAL_GZ,
        ),
        ProcessPathResult::FileValid(
            fpaths.get(2).unwrap().clone(), FILETYPE_JOURNAL_LZ4,
        ),
        ProcessPathResult::FileValid(
            fpaths.get(3).unwrap().clone(), FILETYPE_JOURNAL_XZ,
        ),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), &checks, false);
}

fn test_process_path_tar(
    path: &FPath,
    checks: &Vec<ProcessPathResult>,
    unparseable_are_text: bool,
) {
    defn!("test_process_path_tar({:?}, …, {:?})", path, unparseable_are_text);
    for check in checks.iter() {
        defo!("check {:?}", check);
    }
    let results = process_path_tar(
        path,
        unparseable_are_text,
        FileTypeArchive::Normal,
    );
    for result in results.iter() {
        defo!("result {:?}", result);
    }
    // basic comparison
    defo!("There are {} results and {} checks", results.len(), checks.len());
    assert_eq!(results.len(), checks.len(), "results and checks have different lengths!");
    // are all `checks` in `results`?
    for check in checks.iter() {
        assert!(
            results.contains(check),
            "\nprocess_path({:?})\n  the check {:?}\n  is not contained in the results:\n       {:?}\n",
            path, check, results,
        );
        defo!("found check {:?}", check);
    }
    // are all `results` in `checks`?
    for result in results.iter() {
        assert!(
            checks.contains(result),
            "\nprocess_path({:?})\n  the result {:?}\n  is not contained in the checks:\n       {:?}\n",
            path, result, &checks,
        );
        defo!("found result {:?}", result);
    }
    defx!();
}

#[test]
fn test_process_path_tar_tar1_file_a() {
    let check: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TAR_8BYTE_FILEA_FPATH.clone(),
            NTF_TAR_8BYTE_FILEA_FILETYPE,
        ),
    ];
    defñ!();
    test_process_path_tar(&NTF_TAR_8BYTE_FPATH, &check, true);
    test_process_path_tar(&NTF_TAR_8BYTE_FPATH, &check, false);
}

#[test]
fn test_process_path_tar_tar1_file_ab() {
    let check: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TAR_AB_FILEA_FPATH.clone(),
            NTF_TAR_AB_FILEA_FILETYPE,
        ),
        ProcessPathResult::FileValid(
            NTF_TAR_AB_FILEB_FPATH.clone(),
            NTF_TAR_AB_FILEB_FILETYPE,
        ),
    ];
    defñ!();
    test_process_path_tar(&NTF_TAR_AB_FPATH, &check, true);
    test_process_path_tar(&NTF_TAR_AB_FPATH, &check, false);
}

#[test]
fn test_process_path_tar_tar1_file_abcdefghi() {
    let check: Vec<ProcessPathResult> = vec![
        // fileABCDEFGHI.tar|fileA.evtx
        ProcessPathResult::FileValid(
            NTF_TAR_ABCDEFGHI_FILEA_FPATH.clone(),
            NTF_TAR_ABCDEFGHI_FILEA_FILETYPE,
        ),
        // fileABCDEFGHI.tar|fileB.journal
        ProcessPathResult::FileValid(
            NTF_TAR_ABCDEFGHI_FILEB_FPATH.clone(),
            NTF_TAR_ABCDEFGHI_FILEB_FILETYPE,
        ),
        // fileABCDEFGHI.tar|fileC.utmp
        ProcessPathResult::FileValid(
            NTF_TAR_ABCDEFGHI_FILEC_FPATH.clone(),
            NTF_TAR_ABCDEFGHI_FILEC_FILETYPE,
        ),
        // fileABCDEFGHI.tar|fileD.txt
        ProcessPathResult::FileValid(
            NTF_TAR_ABCDEFGHI_FILED_FPATH.clone(),
            NTF_TAR_ABCDEFGHI_FILED_FILETYPE,
        ),
        // fileABCDEFGHI.tar|fileE.evtx.gz
        ProcessPathResult::FileErrNotSupported(
            NTF_TAR_ABCDEFGHI_FILEE_FPATH.clone(),
            None,
        ),
        // fileABCDEFGHI.tar|fileF.journal.xz
        ProcessPathResult::FileErrNotSupported(
            NTF_TAR_ABCDEFGHI_FILEF_FPATH.clone(),
            None,
        ),
        // fileABCDEFGHI.tar|fileG.txt.xz
        ProcessPathResult::FileErrNotSupported(
            NTF_TAR_ABCDEFGHI_FILEG_FPATH.clone(),
            None,
        ),
        // fileABCDEFGHI.tar|fileH.txt.tar
        ProcessPathResult::FileErrNotSupported(
            NTF_TAR_ABCDEFGHI_FILEH_FPATH.clone(),
            None,
        ),
        // fileABCDEFGHI.tar|fileI
        ProcessPathResult::FileValid(
            NTF_TAR_ABCDEFGHI_FILEI_FPATH.clone(),
            NTF_TAR_ABCDEFGHI_FILEI_FILETYPE,
        ),
        // TODO: add known ignored file; `fileJ.dll`
    ];
    defñ!();
    test_process_path_tar(&NTF_TAR_ABCDEFGHI_FPATH, &check, true);
    test_process_path_tar(&NTF_TAR_ABCDEFGHI_FPATH, &check, false);
}


#[test]
fn test_process_path_tar_dir_tar1_file_abcdefghi() {
    defn!();

    // need to copy the tar file to a temporary directory
    // and then replace the `NTF_TAR_ABCDEFGHI_FILE*_FPATH` variables
    // with the new temporary directory
    // the tar file `NTF_TAR_ABCDEFGHI_FPATH` will be at `/tmp/ntf-abcdefghi.tar` and the preceding
    // path needs to include to the temporary directory,
    // e.g. `/tmp/tmp.dir.1Baer3/ntf-abcdefghi.tar`
    // and each "subpath" needs to go from `/tmp/ntf-abcdefghi.tar|fileA` to
    // path `/tmp/tmp.dir.1Baer3/ntf-abcdefghi.tar|fileA`

    defo!("NTF_TAR_ABCDEFGHI {:?}", (&*NTF_TAR_ABCDEFGHI).path());
    let replace1 = (&*NTF_TAR_ABCDEFGHI).path().parent().unwrap();
    let replace1_fpath = path_to_fpath(replace1);
    defo!("replace1          {:?}", replace1_fpath);
    let tmpdir = create_temp_dir();
    defo!("tmpdir            {:?}", tmpdir.path());
    let tmpdir_fpath = &path_to_fpath(tmpdir.path());
    let dest = tmpdir.path().join((&*NTF_TAR_ABCDEFGHI).path().file_name().unwrap_or_default());
    defo!("dest              {:?}", dest);
    match copy((&*NTF_TAR_ABCDEFGHI).path(), dest) {
        Ok(_) => {},
        Err(e) => {
            panic!("Failed to copy {:?} to {:?}: {}", &*NTF_TAR_ABCDEFGHI.path(), tmpdir.path(), e);
        },
    }

    let check: Vec<ProcessPathResult> = vec![
        // fileABCDEFGHI.tar|fileA.evtx
        ProcessPathResult::FileValid(
            NTF_TAR_ABCDEFGHI_FILEA_FPATH.clone().replacen(&replace1_fpath, tmpdir_fpath, 1),
            NTF_TAR_ABCDEFGHI_FILEA_FILETYPE,
        ),
        // fileABCDEFGHI.tar|fileB.journal
        ProcessPathResult::FileValid(
            NTF_TAR_ABCDEFGHI_FILEB_FPATH.clone().replacen(&replace1_fpath, tmpdir_fpath, 1),
            NTF_TAR_ABCDEFGHI_FILEB_FILETYPE,
        ),
        // fileABCDEFGHI.tar|fileC.utmp
        ProcessPathResult::FileValid(
            NTF_TAR_ABCDEFGHI_FILEC_FPATH.clone().replacen(&replace1_fpath, tmpdir_fpath, 1),
            NTF_TAR_ABCDEFGHI_FILEC_FILETYPE,
        ),
        // fileABCDEFGHI.tar|fileD.txt
        ProcessPathResult::FileValid(
            NTF_TAR_ABCDEFGHI_FILED_FPATH.clone().replacen(&replace1_fpath, tmpdir_fpath, 1),
            NTF_TAR_ABCDEFGHI_FILED_FILETYPE,
        ),
        // fileABCDEFGHI.tar|fileE.evtx.gz
        ProcessPathResult::FileErrNotSupported(
            NTF_TAR_ABCDEFGHI_FILEE_FPATH.clone().replacen(&replace1_fpath, tmpdir_fpath, 1),
            None,
        ),
        // fileABCDEFGHI.tar|fileF.journal.xz
        ProcessPathResult::FileErrNotSupported(
            NTF_TAR_ABCDEFGHI_FILEF_FPATH.clone().replacen(&replace1_fpath, tmpdir_fpath, 1),
            None,
        ),
        // fileABCDEFGHI.tar|fileG.txt.xz
        ProcessPathResult::FileErrNotSupported(
            NTF_TAR_ABCDEFGHI_FILEG_FPATH.clone().replacen(&replace1_fpath, tmpdir_fpath, 1),
            None,
        ),
        // fileABCDEFGHI.tar|fileH.txt.tar
        ProcessPathResult::FileErrNotSupported(
            NTF_TAR_ABCDEFGHI_FILEH_FPATH.clone().replacen(&replace1_fpath, tmpdir_fpath, 1),
            None,
        ),
        // fileABCDEFGHI.tar|fileI
        ProcessPathResult::FileValid(
            NTF_TAR_ABCDEFGHI_FILEI_FPATH.clone().replacen(&replace1_fpath, tmpdir_fpath, 1),
            NTF_TAR_ABCDEFGHI_FILEI_FILETYPE,
        ),
    ];

    test_process_path_fpath(tmpdir_fpath, &check, true);
}
