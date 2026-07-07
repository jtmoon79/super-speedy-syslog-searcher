// build.rs

use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use ::chrono;
use ::dotenvy;
use ::list_features;

// XXX: partially duplicates `subprojects/ere/ere_datetimes_impl/build.rs` regarding
//      handling of `S4_BUILD_REGEX`, `S4_BUILD_PRINT`, `S4_BUILD_REGEX_NO_REBUILD`
//      settings. This is unfortunate.
//      Without this duplicate `build.rs` code the compiler would reject some
//      `cargo::rustc-cfg` expressions generated.
// TODO: [2026/06/26] confirm this!

// TODO: rebuild if `src/python/s4_event_readers/**` files change
//       see https://doc.rust-lang.org/1.88.0/cargo/reference/build-scripts.html#rerun-if-changed

/// env. var. set by docs.rs build environment; see https://docs.rs/about/builds
const ENV_DOCS_RS: &str = "DOCS_RS";

const ENV_BUILD_EPRINT: &str = "S4_BUILD_PRINT";
const ENV_BUILD_REGEX: &str = "S4_BUILD_REGEX";
const ENV_BUILD_REGEX_NO_REBUILD: &str = "S4_BUILD_REGEX_NO_REBUILD";
const REGEX_ALL: &str = "ALL";
const REGEX_TEST: &str = "TEST";
const CONFIG_REGEX: &str = "regex";
/// This must match `datetime.rs` value `DATETIME_PARSE_DATAS_LEN_MAX`
pub const DATETIME_PARSE_DATAS_LEN: usize = 188;

pub const PATH_FILE_TIMESTAMP: &str = "timestamp.txt";
/// set this env. var. to override the timestamp value; allows for idempotent builds
pub const ENV_BUILD_TIMESTAMP: &str = "S4_BUILD_TIMESTAMP";
pub const PATH_FILE_RUSTC_VERSION: &str = "rustc_version.txt";
pub const PATH_FILE_OPT_LEVEL: &str = "opt_level.txt";
pub const PATH_FILE_GIT_COMMIT: &str = "git_commit.txt";
pub const PATH_FILE_PROFILE_NAME: &str = "profile_name.txt";

fn is_env_var_truthy(env_var: &str) -> bool {
    match std::env::var(env_var) {
        Ok(val) => {
            let val_lower = val.to_lowercase();
            val_lower == "1" || val_lower == "true" || val_lower == "yes" || val_lower == "on"
        }
        Err(_) => false,
    }
}

/// check if `ENV_BUILD_EPRINT` is truthy
fn info_enabled() -> bool {
    is_env_var_truthy(ENV_BUILD_EPRINT)
}

/// `info` if `info_enabled()` is true
macro_rules! info {
    ($($arg:tt)*) => {
        if info_enabled() {
            ::build_print::custom_println!("./build.rs:", cyan, $($arg)*);
        }
    };
}

macro_rules! warn {
    ($($arg:tt)*) => {
        if info_enabled() {
            ::build_print::custom_println!("./build.rs:", yellow, $($arg)*);
        }
    };
}

/// wrapper to call `println!` and also `build_print::custom_println!` if `info_enabled()` is true
macro_rules! build_println {
    ($($arg:tt)*) => {
        println!($($arg)*);
        if info_enabled() {
            ::build_print::custom_println!("./build.rs:", green, $($arg)*);
        }
    };
}

/// allow environment variable `S4_BUILD_REGEX` to specify which regexes to compile
/// can specify
/// - single values, e.g. `S4_BUILD_REGEX=1`
/// - multiple values, e.g. `S4_BUILD_REGEX=1,2`
/// - a range of values, e.g. `S4_BUILD_REGEX=1-3`
///
/// These may be combined, e.g. `S4_BUILD_REGEX=1,3-5`.
///
/// If no `S4_BUILD_REGEX` environment variable is specified then all regexes will be
/// compiled.
fn parse_regex_values() {
    // override for building at docs.rs
    // otherwise docs.rs build will fail due to a resource-constrained environment
    if std::env::var(ENV_DOCS_RS).is_ok() {
        info!("docs.rs build detected from {ENV_DOCS_RS}; building only regex #1");
        build_println!("cargo::rustc-cfg={CONFIG_REGEX}=\"1\"");
        build_println!("cargo::rustc-check-cfg=cfg({CONFIG_REGEX}, values(\"1\"))");
        return;
    }

    // process environment variable
    let mut build_regex_val: String = std::env::var(ENV_BUILD_REGEX).unwrap_or_else(|_| String::with_capacity(0));
    if build_regex_val.is_empty() {
        // process file if environment variable is not set or empty
        let project_dir: String = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let file_path: PathBuf = PathBuf::from(project_dir).join(ENV_BUILD_REGEX);
        if file_path.exists() {
            let file_path_: PathBuf = file_path.clone();
            let file_contents: String = std::fs::read_to_string(file_path).expect("Failed to read file");
            let file_contents_trimmed: String = file_contents
                .trim()
                .to_string();
            if !file_contents_trimmed.is_empty() {
                // use the file contents as the build regex value
                build_regex_val = file_contents_trimmed;
                info!("Using file {file_path_:?} contents for {ENV_BUILD_REGEX}: {build_regex_val:?}");
            }
        }
    }

    let mut regex_values: Vec<String> = Vec::new();
    if build_regex_val == REGEX_TEST {
        build_println!("cargo::rustc-cfg={CONFIG_REGEX}=\"{REGEX_TEST}\"");
        regex_values.push(REGEX_TEST.to_string());
    } else if !build_regex_val.is_empty() {
        for val in build_regex_val.split(',') {
            if val.contains('-') {
                // range
                let (a_s, b_s) = val
                    .split_once('-')
                    .expect("Invalid range format");
                let mut a_n: usize = a_s
                    .parse::<usize>()
                    .unwrap_or_else(|_| panic!("Invalid number in range: {a_s:?} from {val:?}"));
                let mut b_n: usize = b_s
                    .parse::<usize>()
                    .unwrap_or_else(|_| panic!("Invalid number in range: {b_s:?} from {val:?}"));
                if a_n > b_n {
                    std::mem::swap(&mut a_n, &mut b_n);
                }
                for n in a_n..=b_n {
                    build_println!("cargo::rustc-cfg={CONFIG_REGEX}=\"{n}\"");
                    regex_values.push(n.to_string());
                }
            } else {
                // single value
                build_println!("cargo::rustc-cfg={CONFIG_REGEX}=\"{val}\"");
                regex_values.push(val.to_string());
            }
        }
    } else {
        build_println!("cargo::rustc-cfg={CONFIG_REGEX}=\"{REGEX_ALL}\"");
        regex_values.push(REGEX_ALL.to_string());
    }
    info!("regex values specified: {regex_values:?}");

    // rerun if environment variable changes
    match std::env::var(ENV_BUILD_REGEX_NO_REBUILD) {
        Ok(val) => {
            if val.is_empty() {
                build_println!(r#"cargo::rerun-if-env-changed={ENV_BUILD_REGEX}"#);
            } else {
                info!("skip rerun-if-env-changed={ENV_BUILD_REGEX} because {ENV_BUILD_REGEX_NO_REBUILD}");
            }
        }
        Err(_) => {
            build_println!(r#"cargo::rerun-if-env-changed={ENV_BUILD_REGEX}"#);
        }
    }
}

/// helper to get the output directory as a `PathBuf`
fn out_path() -> PathBuf {
    // used passed `outdir`, fallback to env var `OUT_DIR`, fallback to current directory
    let outdir: String = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let mut out_path_ = PathBuf::new();
    out_path_.push(outdir);

    out_path_
}

/// Write current datetime to a file as a string for `include!`.
/// Ripped from <https://www.dgendill.com/posts/programming/2025-10-20-embedding-buildtime-into-rust-binary.html>
fn write_timestamp_file() {
    let mut out_path = out_path();
    out_path.push(PATH_FILE_TIMESTAMP);
    let mut fhandle = fs::File::create(&out_path)
        .unwrap_or_else(|e| panic!("write_timestamp_file failed to create file {out_path:?}: {e:?}"));
    if let Ok(val) = std::env::var(ENV_BUILD_TIMESTAMP) {
        info!("Environment variable {ENV_BUILD_TIMESTAMP}={val:?}; write to file {out_path:?}");
        write!(fhandle, "{val:?}").expect("write failed for timestamp file");
        return;
    }
    let local_now = chrono::Local::now();
    let now_s: String = local_now.format("%Y-%m-%dT%H:%M:%S").to_string();
    write!(fhandle, "{now_s:?}").expect("write failed for timestamp file");
    info!("Wrote timestamp {now_s:?} to file {out_path:?}");
}

/// Write rustc version to a file as a string for `include!`.
fn write_rustc_version_file() {
    let mut out_path = out_path();
    out_path.push(PATH_FILE_RUSTC_VERSION);
    let mut fhandle = fs::File::create(&out_path)
        .unwrap_or_else(|e| panic!("write_rustc_version_file failed to create file {out_path:?}: {e:?}"));
    let rustc_version_str: String = rustc_version_runtime::version().to_string();
    write!(fhandle, r#""{rustc_version_str}""#).expect("write failed for rustc_version file");
    info!("Wrote rustc version {rustc_version_str:?} to file {out_path:?}");
}

fn write_opt_level() {
    let mut out_path = out_path();
    out_path.push(PATH_FILE_OPT_LEVEL);
    let mut fhandle = fs::File::create(&out_path)
        .unwrap_or_else(|e| panic!("write_opt_level failed to create file {out_path:?}: {e:?}"));
    let opt_level: String = env::var("OPT_LEVEL").expect("OPT_LEVEL not set; this should be set by cargo. Something is very wrong");
    write!(fhandle, "{opt_level:?}").expect("write failed for opt_level file");
    info!("Wrote opt level {opt_level:?} to file {out_path:?}");
}

/// Write git commit hash to a file as a string for `include!`.
fn write_git_commit_file() {
    let mut out_path = out_path();
    out_path.push(PATH_FILE_GIT_COMMIT);
    let mut fhandle = fs::File::create(&out_path)
        .unwrap_or_else(|e| panic!("write_git_commit_file failed to create file {out_path:?}: {e:?}"));
    let not_available: String = "Not Available".to_string();

    // Save the full commit hash when available, otherwise a sentinel value.
    let git_commit_str: String = match Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
    {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string(),
        Ok(output) => {
            warn!("command 'git rev-parse HEAD' exited with {:?};\n{}",
                output.status, String::from_utf8_lossy(&output.stderr));

            not_available.clone()
        }
        Err(err) => {
            warn!("command 'git rev-parse HEAD' failed: {err:?}");

            not_available.clone()
        }
    };

    if git_commit_str.is_empty() {
        write!(fhandle, r#""{not_available}""#).expect("write failed for git_commit file");
        info!("Git commit hash not available; wrote sentinel value to file {out_path:?}");
    } else {
        write!(fhandle, r#""{git_commit_str}""#).expect("write failed for git_commit file");
        info!("Wrote git commit hash {git_commit_str:?} to file {out_path:?}");
    }
}

/// Write build `--features` to a file as a string for `include!`.
fn write_features_file() {
    let features: Vec<String> = list_features::list_enabled();
    let features_str = format!("\"{}\"", features.join(", "));

    let list_features_txt = out_path().join("list_features.txt");
    std::fs::write(&list_features_txt, &features_str).unwrap();
    info!("Wrote enabled features {features_str} to file {list_features_txt:?}");
}

fn write_cpu_features_file() {
    let mut out_path = out_path();
    out_path.push("list_cpu_features.txt");
    let mut fhandle = fs::File::create(&out_path)
        .unwrap_or_else(|e| panic!("write_cpu_features_file failed to create file {out_path:?}: {e:?}"));
    let cpu_features: String = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    write!(fhandle, "{cpu_features:?}").expect("write failed for cpu_features file");
    info!("Wrote CPU features {cpu_features:?} to file {out_path:?}");
}

/// Write build profile name to a file as a string for `include!`.
fn write_profile_file() {
    // ripped from https://stackoverflow.com/a/73603419/471376
    fn get_build_profile_name() -> String {
        // The profile name is always the 3rd last part of the path (with 1 based indexing).
        // e.g. /code/core/target/cli/build/my-build-info-9f91ba6f99d7a061/out
        std::env::var("OUT_DIR")
            .unwrap()
            .split(std::path::MAIN_SEPARATOR)
            .nth_back(3)
            .unwrap_or_default()
            .to_string()
    }
    let profile_name = get_build_profile_name();
    let mut out_path = out_path();
    out_path.push(PATH_FILE_PROFILE_NAME);
    let mut fhandle = fs::File::create(&out_path)
        .unwrap_or_else(|e| panic!("write_profile_file failed to create file {out_path:?}: {e:?}"));
    write!(fhandle, r#"r"{profile_name}""#).expect("write failed for profile_name file");
    info!("Wrote build profile \"{profile_name}\" to file {out_path:?}");
}

/// Set the Windows file properties for the exectuable.
fn windows_exe_info_create() {
    const FAMILY_EXPECT: &str = "windows";
    if std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap() != FAMILY_EXPECT {
        info!("Skip windows_exe_info because FAMILY is not {FAMILY_EXPECT:?}");
        return;
    }
    use windows_exe_info::versioninfo::*;
    let version_string: String = env!("CARGO_PKG_VERSION").to_string();
    let major: u16 = env!("CARGO_PKG_VERSION_MAJOR")
        .parse::<u16>()
        .expect("Failed to parse CARGO_PKG_VERSION_MAJOR as u16");
    let minor: u16 = env!("CARGO_PKG_VERSION_MINOR")
        .parse::<u16>()
        .expect("Failed to parse CARGO_PKG_VERSION_MINOR as u16");
    let patch: u16 = env!("CARGO_PKG_VERSION_PATCH")
        .parse::<u16>()
        .expect("Failed to parse CARGO_PKG_VERSION_PATCH as u16");
    let author: String = env!("CARGO_PKG_AUTHORS").to_string();
    let copyright: String = "Copyright (C) 2026 ".to_string() + &author;
    let bin_name: String =
        std::env::var("CARGO_BIN_NAME").unwrap_or("s4".to_string()).to_string()
        + std::env::consts::EXE_SUFFIX;
    VersionInfo {
        file_version: Version(
            0,
            major,
            minor,
            patch
        ),
        product_version: Version(
            0,
            major,
            minor,
            patch
        ),
        file_flag_mask: FileFlagMask::Win16,
        file_flags: FileFlags {
            debug: !cfg!(debug_assertions),
            patched: false,
            prerelease: false,
            privatebuild: false,
            infoinferred: false,
            specialbuild: false,
        },
        file_os: FileOS::Windows32,
        file_type: FileType::App,
        file_info: vec![FileInfo {
            lang: Language::USEnglish,
            charset: CharacterSet::Unicode,
            comment: Some(env!("CARGO_PKG_DESCRIPTION").into()),
            company_name: "".into(),
            file_description: env!("CARGO_PKG_DESCRIPTION").into(),
            file_version: version_string.clone().into(),
            internal_name: env!("CARGO_PKG_NAME").into(),
            legal_copyright: Some(copyright.into()),
            legal_trademarks: Some(env!("CARGO_PKG_LICENSE").into()),
            original_filename: bin_name.clone().into(),
            product_name: env!("CARGO_PKG_NAME").into(),
            product_version: version_string.clone().into(),
            private_build: None,
            special_build: None,
        }],
    }
    .link().expect("windows_exe_info failed");
    info!("Wrote Windows executable version info for {bin_name:?} with version {version_string:?}");
}

/// Check for a `.env` file and load it if found; print loaded values if `info_enabled()`.
fn dotenv_load() {
    if let Ok(path) = dotenvy::dotenv() {
        info!("dotenv found .env {path:?}");
        // print what was loaded
        for item in dotenvy::dotenv_iter().unwrap() {
            match item {
                Ok((key, val)) => {
                    info!("dotenv {}={:?}", key, val);
                },
                Err(e) => {
                    ::build_print::warn!("dotenv_iter failed to parse .env file item: {:?}", e);
                    continue;
                }
            };
        }
    }
}

fn main() {
    info!("main() build.rs for super_speedy_syslog_searcher");
    dotenv_load();
    write_timestamp_file();
    write_rustc_version_file();
    write_opt_level();
    write_git_commit_file();
    write_features_file();
    write_cpu_features_file();
    write_profile_file();
    windows_exe_info_create();
    parse_regex_values();
}
