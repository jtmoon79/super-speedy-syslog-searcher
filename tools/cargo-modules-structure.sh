#!/usr/bin/env bash
#
# run `cargo modules` with preferred options
#
# to install:
#     cargo install cargo-modules

set -eu

if [[ ! "${DIROUT+x}" ]]; then
    echo "Must set DIROUT environment variable" >&2
    exit 1
fi

cd "$(dirname -- "${0}")/.."

export NO_COLOR=1

set -x

cargo modules --version
cargo modules structure --package super_speedy_syslog_searcher --lib > "${DIROUT}/cargo-modules-structure_super_speedy_syslog_searcher_lib.txt"
cargo modules structure --package super_speedy_syslog_searcher --bin s4 > "${DIROUT}/cargo-modules-structure_super_speedy_syslog_searcher_bin.txt"
cargo modules structure --package super-speedy-syslog-searcher_ere --lib > "${DIROUT}/cargo-modules-structure_super-speedy-syslog-searcher_ere_lib.txt"
cargo modules structure --package super-speedy-syslog-searcher_ere-core --lib > "${DIROUT}/cargo-modules-structure_super-speedy-syslog-searcher_ere-core_lib.txt"
cargo modules structure --package super-speedy-syslog-searcher_ere-macros --lib > "${DIROUT}/cargo-modules-structure_super-speedy-syslog-searcher_ere-macros_lib.txt"
cargo modules structure --package super-speedy-syslog-searcher_ere_automator_procmacro --lib > "${DIROUT}/cargo-modules-structure_super-speedy-syslog-searcher_ere_automator_procmacro_lib.txt"
cargo modules structure --package super-speedy-syslog-searcher_ere_datetimes_impl --lib > "${DIROUT}/cargo-modules-structure_super-speedy-syslog-searcher_ere_datetimes_impl_lib.txt"
