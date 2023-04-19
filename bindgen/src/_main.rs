// _main.rs
//
// hack attempt to use `libloading` for dynamic loading of `libsystemd`

const LIB_PATH: &str = "/usr/lib/systemd/libsystemd-shared-249.so";

const FUNC_NAME_JOURNAL_OPEN: &str = "sd_journal_open";
const FUNC_NAME_JOURNAL_NEXT: &str = "journal_file_next_entry";

use ::libloading;

fn main() {
    unsafe {
        eprintln!("Loading library {:?}", LIB_PATH);
        let lib = match libloading::Library::new(LIB_PATH) {
            Ok(lib) => lib,
            Err(err) => {
                eprintln!("Error {:?}", err);
                return;
            }
        };

        eprintln!("Loading function {:?}", FUNC_NAME_JOURNAL_OPEN);
        let func_open: libloading::Symbol<unsafe extern fn() -> u32> =
            match lib.get(FUNC_NAME_JOURNAL_OPEN.as_bytes()) {
                Ok(func) => func,
                Err(err) => {
                    eprintln!("Error {:?}", err);
                    return;
                }
            };

        eprintln!("Loading function {:?}", FUNC_NAME_JOURNAL_NEXT);
        let func_next: libloading::Symbol<unsafe extern fn() -> u32> =
            match lib.get(FUNC_NAME_JOURNAL_NEXT.as_bytes()) {
                Ok(func) => func,
                Err(err) => {
                    eprintln!("Error {:?}", err);
                    return;
                }
            };
    }
    eprintln!("Done");
}
