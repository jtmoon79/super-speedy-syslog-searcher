// subprojects/ere/ere_datetimes_impl/build.rs

// XXX: duplicates top-level `build.rs`

use std::path::PathBuf;

use ::dotenvy::dotenv;

// BUG: suffers from false-positive rebuilds (unnecessary rebuilds)
//      you'll see in the build output:
//          $ cargo build -v
//          ...
//          Dirty ere_datetimes_impl v0.0.1 (super-speedy-syslog-searcher/subprojects/ere/ere_datetimes_impl): stale, unknown reason
//          Compiling ere_datetimes_impl v0.0.1 (super-speedy-syslog-searcher/subprojects/ere/ere_datetimes_impl)
//          ...
//      Would be cool to try `cache-proc-macros`
//      See https://dev-doc.rust-lang.org/stable/unstable-book/compiler-flags/cache-proc-macros.html

/// env. var. set by docs.rs build environment; see https://docs.rs/about/builds
const ENV_DOCS_RS: &str = "DOCS_RS";

const ENV_BUILD_EPRINT: &str = "S4_BUILD_PRINT";
const ENV_BUILD_REGEX: &str = "S4_BUILD_REGEX";
const ENV_BUILD_REGEX_NO_REBUILD: &str = "S4_BUILD_REGEX_NO_REBUILD";
const REGEX_ALL: &str = "ALL";
const CONFIG_REGEX: &str = "regex";
/// This must match `datetime.rs` value `DATETIME_PARSE_DATAS_LEN_MAX`
pub const DATETIME_PARSE_DATAS_LEN: usize = 181;

fn info_enabled() -> bool {
    std::env::var(ENV_BUILD_EPRINT).is_ok_and(|x| !x.is_empty())
}

/// `info` if `info_enabled()` is true
macro_rules! info {
    ($($arg:tt)*) => {
        if info_enabled() {
            ::build_print::custom_println!("./subprojects/ere/ere_datetimes_impl/build.rs:", green, $($arg)*);
        }
    };
}

macro_rules! build_println {
    ($($arg:tt)*) => {
        println!($($arg)*);
        if info_enabled() {
            ::build_print::custom_println!("./subprojects/ere/ere_datetimes_impl/build.rs:", cyan, $($arg)*);
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
/// If there is no environment variable `S4_BUILD_REGEX` then search for file
/// `S4_BUILD_REGEX` in the project directory. This is useful for cases
/// where environment variables are not forwarded, e.g. with `cross`.
///
/// If no `S4_BUILD_REGEX` value is specified then all regexes will be
/// compiled.
fn parse_regex_values() {
    // allow all possible values; avoids a warning from rust
    info!("Allow {} possible values for \"{CONFIG_REGEX}\"\n", DATETIME_PARSE_DATAS_LEN + 1);
    let mut valid_values_str: String = (1..=DATETIME_PARSE_DATAS_LEN)
        .map(|v| format!("\"{v}\""))
        .collect::<Vec<String>>()
        .join(",");
    valid_values_str.push_str(&format!(",\"{REGEX_ALL}\""));
    build_println!("cargo::rustc-check-cfg=cfg({CONFIG_REGEX}, values({valid_values_str}))");

    // override for building at docs.rs
    // otherwise docs.rs build will fail due to a resource-constrained environment
    if std::env::var(ENV_DOCS_RS).is_ok() {
        info!("docs.rs build detected from {ENV_DOCS_RS}; building only regex #1");
        build_println!("cargo::rustc-cfg={CONFIG_REGEX}=\"1\"");
        build_println!("cargo::rustc-check-cfg=cfg({CONFIG_REGEX}, values(\"1\"))");
        return;
    }

    // process environment variable
    let mut build_regex_val: String = std::env::var(ENV_BUILD_REGEX).unwrap_or_else(|_| String::new());
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
                build_println!("cargo::rerun-if-env-changed={ENV_BUILD_REGEX}");
            } else {
                info!("skip rerun-if-env-changed={ENV_BUILD_REGEX} because {ENV_BUILD_REGEX_NO_REBUILD}");
            }
        }
        Err(_) => {
            build_println!("cargo::rerun-if-env-changed={ENV_BUILD_REGEX}");
        }
    }
}

fn main() {
    info!("main() build.rs for ere_datetimes_impl");
    match dotenv() {
        Ok(path) => {
            info!("dotenv loaded environment variables from .env file {path:?}");
        }
        Err(_) => {}
    }
    parse_regex_values();
}
