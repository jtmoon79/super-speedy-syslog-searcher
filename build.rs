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

// XXX: duplicates `subprojects/ere/ere_datetimes_impl/build.rs`
//      which is very unfortunate.
//      because without this `build.rs then different `S4_BUILD_REGEX` values will
//      not trigger a rebuild of `ere_datetimes_impl/build.rs`

/// env. var. set by docs.rs build environment; see https://docs.rs/about/builds
const ENV_DOCS_RS: &str = "DOCS_RS";

const ENV_BUILD_EPRINT: &str = "S4_BUILD_PRINT";
const ENV_BUILD_REGEX: &str = "S4_BUILD_REGEX";
const ENV_BUILD_REGEX_NO_REBUILD: &str = "S4_BUILD_REGEX_NO_REBUILD";
const REGEX_ALL: &str = "ALL";
const CONFIG_REGEX: &str = "regex";
/// This must match `datetime.rs` value `DATETIME_PARSE_DATAS_LEN_MAX`
pub const DATETIME_PARSE_DATAS_LEN: usize = 181;

pub const PATH_FILE_TIMESTAMP: &str = "timestamp.txt";
/// set this env. var. to override the timestamp value; allows for idempotent builds
pub const ENV_BUILD_TIMESTAMP: &str = "S4_BUILD_TIMESTAMP";
pub const PATH_FILE_RUSTC_VERSION: &str = "rustc_version.txt";
pub const PATH_FILE_GIT_COMMIT: &str = "git_commit.txt";

// TODO: rebuild if `src/python/s4_event_readers` changes
//       see https://doc.rust-lang.org/1.88.0/cargo/reference/build-scripts.html#rerun-if-changed

fn is_env_var_truthy(env_var: &str) -> bool {
    match std::env::var(env_var) {
        Ok(val) => {
            let val_lower = val.to_lowercase();
            val_lower == "1" || val_lower == "true" || val_lower == "yes" || val_lower == "on"
        }
        Err(_) => false,
    }
}

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
    if !build_regex_val.is_empty() {
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

fn out_path() -> PathBuf {
    // used passed `outdir`, fallback to env var `OUT_DIR`, fallback to current directory
    let outdir: String = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let mut out_path_ = PathBuf::new();
    out_path_.push(outdir);

    out_path_
}

/// ripped from <https://www.dgendill.com/posts/programming/2025-10-20-embedding-buildtime-into-rust-binary.html>
fn write_timestamp_file() {
    let mut out_path = out_path();
    out_path.push(PATH_FILE_TIMESTAMP);
    let mut fhandle = fs::File::create(&out_path)
        .unwrap_or_else(|e| panic!("write_timestamp_file failed to create file {out_path:?}: {e:?}"));
    match std::env::var(ENV_BUILD_TIMESTAMP) {
        Ok(val) => {
            info!("Environment variable {ENV_BUILD_TIMESTAMP}={val:?}; write to file {out_path:?}");
            write!(fhandle, "{val:?}").ok();
            return;
        }
        Err(_) => {}
    }
    let local_now = chrono::Local::now();
    let now_s: String = local_now.format("%Y-%m-%dT%H:%M:%S").to_string();
    write!(fhandle, r#""{now_s}""#).ok();
    info!("Wrote timestamp {now_s:?} to file {out_path:?}");
}

fn write_rustc_version_file() {
    let mut out_path = out_path();
    out_path.push(PATH_FILE_RUSTC_VERSION);
    let mut fhandle = fs::File::create(&out_path)
        .unwrap_or_else(|e| panic!("write_rustc_version_file failed to create file {out_path:?}: {e:?}"));
    let rustc_version_str: String = rustc_version_runtime::version().to_string();
    write!(fhandle, r#""{rustc_version_str}""#).ok();
    info!("Wrote rustc version {rustc_version_str:?} to file {out_path:?}");
}

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
        _ => not_available.clone(),
    };

    if git_commit_str.is_empty() {
        write!(fhandle, r#""{not_available}""#).ok();
        info!("Git commit hash not available; wrote sentinel value to file {out_path:?}");
    } else {
        write!(fhandle, r#""{git_commit_str}""#).ok();
        info!("Wrote git commit hash {git_commit_str:?} to file {out_path:?}");
    }
}

fn write_features_file() {
    let list_features_rs = out_path().join("list_features.rs");
    let features: String = list_features::list_enabled_as_string("LIST_FEATURES");
    std::fs::write(&list_features_rs, features).unwrap();
    // writes a Rust file that looks like:
    //      pub const LIST_FEATURES: &[&str] = &[
    //          "alloc_tracker",
    //      ];

    // scrape that Rust file so it can be included as a `const &str` in `s4.rs`
    let file = File::open(&list_features_rs).unwrap();
    let reader = BufReader::new(file);
    let values: Vec<String> = reader.lines().map(|line| line.unwrap()).collect();
    let mut features_str = String::from("\"");
    for line in values[1..values.len() - 1].iter() {
        let line = line.trim().trim_matches(',').trim_matches('"').to_string();
        if !line.is_empty() {
            features_str.push_str(&line);
            features_str.push_str(", ");
        }
    }
    while features_str.ends_with(',') || features_str.ends_with(' ') {
        features_str.pop();
    }
    features_str.push('"');

    // write the scraped values to a text file
    let list_features_txt = out_path().join("list_features.txt");
    std::fs::write(&list_features_txt, &features_str).unwrap();
    info!("Wrote enabled features {features_str} to file {list_features_txt:?}");
}

fn dotenv_load() {
    match dotenvy::dotenv() {
        Ok(path) => {
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
        Err(_) => {}
    }
}

fn main() {
    info!("main() build.rs for super_speedy_syslog_searcher");
    dotenv_load();
    write_timestamp_file();
    write_rustc_version_file();
    write_git_commit_file();
    write_features_file();
    parse_regex_values();
}
