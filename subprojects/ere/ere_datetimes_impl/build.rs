// subprojects/ere/ere_datetimes_impl/build.rs

// XXX: duplicates top-level `build.rs`

use std::path::PathBuf;

use ::dotenv::dotenv;

// BUG: suffers from false-positive rebuilds (unnecessary rebuilds)
//      you'll see in the build output:
//          $ cargo build -v
//          ...
//          Dirty ere_datetimes_impl v0.0.1 (super-speedy-syslog-searcher/subprojects/ere/ere_datetimes_impl): stale, unknown reason
//          Compiling ere_datetimes_impl v0.0.1 (super-speedy-syslog-searcher/subprojects/ere/ere_datetimes_impl)
//          ...
//      Would be cool to try `cache-proc-macros`
//      See https://dev-doc.rust-lang.org/stable/unstable-book/compiler-flags/cache-proc-macros.html

const ENV_BUILD_REGEX: &str = "S4_BUILD_REGEX";
const ENV_BUILD_REGEX_NO_REBUILD: &str = "S4_BUILD_REGEX_NO_REBUILD";
const REGEX_ALL: &str = "ALL";
const CONFIG_REGEX: &str = "regex";
/// This must match `datetime.rs` value `DATETIME_PARSE_DATAS_LEN`
pub const DATETIME_PARSE_DATAS_LEN: usize = 176;

/// allow environment variable `S4_BUILD_REGEX` to specify which regexes to compile
/// can specify
/// - single values, e.g. `S4_BUILD_REGEX=1`
/// - multiple values, e.g. `S4_BUILD_REGEX=1,2`
/// - a range of values, e.g. `S4_BUILD_REGEX=1-3`
///
/// These may be combined, e.g. `S4_BUILD_REGEX=1,3-5`.
///
/// If there is no environment variable `S4_BUILD_REGEX` then search for file
/// `S4_BUILD_REGEX` in the project directory. This is useful for cases
/// where environment variables are not forwarded, e.g. with `cross`.
///
/// If no `S4_BUILD_REGEX` value is specified then all regexes will be
/// compiled.
fn parse_regex_values() {
    // process environment variable
    let mut build_regex_val: String = std::env::var(ENV_BUILD_REGEX).unwrap_or_else(|_| String::new());
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
          println!("cargo::rerun-if-env-changed={ENV_BUILD_REGEX}");
        }
      }
      Err(_) => {
        println!("cargo::rerun-if-env-changed={ENV_BUILD_REGEX}");
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

fn main() {
    dotenv().ok();
    parse_regex_values();
}
