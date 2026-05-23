// build.rs

use std::env;
use std::io::Write;
use std::fs;
use std::path::PathBuf;

use ::chrono;
use ::dotenvy;

// XXX: duplicates `subprojects/ere/ere_datetimes_impl/build.rs`
//      which is very unfortunate.
//      because without this `build.rs then different `S4_BUILD_REGEX` values will
//      not trigger a rebuild of `ere_datetimes_impl/build.rs`

const ENV_BUILD_REGEX: &str = "S4_BUILD_REGEX";
const ENV_BUILD_REGEX_NO_REBUILD: &str = "S4_BUILD_REGEX_NO_REBUILD";
const REGEX_ALL: &str = "ALL";
const CONFIG_REGEX: &str = "regex";
/// This must match `datetime.rs` value `DATETIME_PARSE_DATAS_LEN`
pub const DATETIME_PARSE_DATAS_LEN: usize = 176;

pub const PATH_FILE_TIMESTAMP: &str = "timestamp.txt";
pub const PATH_FILE_RUSTC_VERSION: &str = "rustc_version.txt";

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
    // process environment variable
    let mut build_regex_val: String = std::env::var(ENV_BUILD_REGEX).unwrap_or_else(|_| String::with_capacity(0));
    if build_regex_val.is_empty() {
      // process file if environment variable is not set or empty
      let project_dir: String = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
      let file_path: PathBuf = PathBuf::from(project_dir).join(ENV_BUILD_REGEX);
      if file_path.exists() {
        let file_contents: String = std::fs::read_to_string(file_path).expect("Failed to read file");
        let file_contents_trimmed: String = file_contents.trim().to_string();
        if !file_contents_trimmed.is_empty() {
          // use the file contents as the build regex value
          build_regex_val = file_contents_trimmed;
        }
      }
    }

    if !build_regex_val.is_empty() {
      for val in build_regex_val.split(',') {
        if val.contains('-') {
          // range
          let (a_s, b_s) = val.split_once('-').expect("Invalid range format");
          let mut a_n: usize = a_s.parse::<usize>().unwrap_or_else(
            |_| panic!("Invalid number in range: {a_s:?} from {val:?}"));
          let mut b_n: usize = b_s.parse::<usize>().unwrap_or_else(
            |_| panic!("Invalid number in range: {b_s:?} from {val:?}"));
          if a_n > b_n {
            std::mem::swap(&mut a_n, &mut b_n);
          }
          for n in a_n..=b_n {
            println!("cargo::rustc-cfg={CONFIG_REGEX}=\"{n}\"");
          }
        } else {
          // single value
          println!("cargo::rustc-cfg={CONFIG_REGEX}=\"{val}\"");
        }
      }
    } else {
      println!("cargo::rustc-cfg={CONFIG_REGEX}=\"{REGEX_ALL}\"");
    }

    // rerun if environment variable changes
    // HACK: workaround buggy false-positive rebuilds with S4_BUILD_REGEX_NO_REBUILD.
    //       see comment above
    //       However, seems to be flaky itself.
    //       Might be related to check-cfg warnings?
    //       See https://github.com/mozilla/sccache/issues/2619
    match std::env::var(ENV_BUILD_REGEX_NO_REBUILD) {
      Ok(val) => {
        if val.is_empty() {
          println!(r#"cargo::rerun-if-env-changed={ENV_BUILD_REGEX}"#);
        }
      }
      Err(_) => {
        println!(r#"cargo::rerun-if-env-changed={ENV_BUILD_REGEX}"#);
      }
    }

    // allow all possible values; avoids a warning from rust
    let mut valid_values_str: String =
      (1..=DATETIME_PARSE_DATAS_LEN)
      .map(|v| format!("\"{v}\""))
      .collect::<Vec<String>>()
      .join(",");
    valid_values_str.push_str(&format!(",\"{REGEX_ALL}\""));
    println!(
      "cargo::rustc-check-cfg=cfg({CONFIG_REGEX}, values({valid_values_str}))"
    );
}

/// ripped from https://www.dgendill.com/posts/programming/2025-10-20-embedding-buildtime-into-rust-binary.html
fn write_timestamp_file() {
    // used passed `outdir`, fallback to env var `OUT_DIR`, fallback to current directory
    let outdir: String = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let mut out_path = PathBuf::new();
    out_path.push(outdir);
    out_path.push(PATH_FILE_TIMESTAMP);
    let mut fhandle = fs::File::create(&out_path).unwrap();
    let local_now = chrono::Local::now();
    let now_s = local_now.format("%Y-%m-%dT%H:%M:%S");
    write!(fhandle, r#""{now_s}""#).ok();
}

fn write_rustc_version_file() {
    let outdir: String = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let mut out_path = PathBuf::new();
    out_path.push(outdir);
    out_path.push(PATH_FILE_RUSTC_VERSION);
    let mut fhandle = fs::File::create(&out_path).unwrap();
    let rustc_version_str: String = rustc_version_runtime::version().to_string();
    write!(fhandle, r#""{rustc_version_str}""#).ok();
}

fn main() {
    dotenvy::dotenv().ok();
    write_timestamp_file();
    write_rustc_version_file();
    parse_regex_values();
}
