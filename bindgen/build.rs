// build.rs
//
// generate bindings for `libsystemd`.
// find the generated file using
//     $ find ./target/ -type f -name 'bindings.rs'
//
// Requires `libsystemd` headers to be installed.
// On Ubuntu, install with:
//     $ sudo apt install libsystemd-dev

use std::env;
use std::path::PathBuf;

extern crate bindgen;
extern crate const_format;
use const_format::concatcp;

const PROJ_SYSTEMD_ROOT: &str = "/tmp/systemd";
const PROJ_SYSTEMD_BUILD: &str = concatcp!(PROJ_SYSTEMD_ROOT, "/build");
const LIB_NAME: &str = "systemd";
const WRAPPER_H: &str = "wrapper.h";

fn main() {
    /*
    Rebuild `libsystemd` in the `systemd` project.

        (set -ex;
         ninja -C build -v clean
         ninja -C build -v
         ./build/journalctl --all --no-tail --file=/tmp/user-1000.journal &> /tmp/out
         head -n400 /tmp/out && tail -n200 /tmp/out
        )

    Run using local build of `libsystemd`

        LD_LIBRARY_PATH=${PROJ_SYSTEMD_BUILD} cargo test test_journal -- --nocapture
     */

    // Tell cargo to look for shared libraries in the specified directory
    let arg_link_search: &str = concatcp!("cargo:rustc-link-search=native=", PROJ_SYSTEMD_BUILD);
    println!("{}", arg_link_search);

    // Tell cargo to tell rustc to link the system shared library.
    // XXX: find the file with command:
    //      $ find / -xdev -not -type d -name '*libsystemd*so*'
    let arg_link_lib: &str = concatcp!("cargo:rustc-link-lib=dylib=", LIB_NAME);
    println!("{}", arg_link_lib);

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    let arg_rerun_if_changed: &str = concatcp!("cargo:rerun-if-changed=", WRAPPER_H);
    println!("{}", arg_rerun_if_changed);

    let builder = bindgen::Builder::default();
    let bindings = builder
        .clang_arg("--verbose")
        // The input header we would like to generate bindings for.
        .header(WRAPPER_H)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
