#!/usr/bin/env bash
#
# llvm-cov-run-command.sh
#
# run `cargo llvm-cov run` multiple times and combine the results
#

set -euo pipefail

cd "$(dirname "${0}")/.."

LCOV_OUTPUT_PATHD=$(mktemp -d)
LCOV_OUTPUT_SUMMARY=${LCOV_OUTPUT_SUMMARY-${LCOV_OUTPUT_PATHD}/summary.lcov}
HTML_OUTPUT_DIR=${LCOV_OUTPUT_SUMMARY%.*}_html

if ! cargo llvm-cov --version; then
    echo "Is cargo-llvm-cov installed?" >&2
    echo "    cargo install --locked cargo-llvm-cov" >&2
    exit 1
fi

if ! (lcov --version && genhtml --version); then
    echo "Is lcov installed?" >&2
    echo "    sudo apt install lcov" >&2
    exit 1
fi

function exit_() {
    rm -rf "${LCOV_OUTPUT_PATHD}"
}
trap exit_ EXIT

declare -a S4_TEST_FILES=(
    ./logs/CentOS9/x86_64/wtmp
    ./logs/other/tests/dtf2-2.log
    ./logs/other/tests/dtf3-2a.log
    ./logs/other/tests/dtf5-6a.log.gz
    ./logs/other/tests/dtf7-20-LEVELS.log.xz
    ./logs/other/tests/gen-2-1.tar
    ./logs/other/tests/gen-20-1-faces.log
    ./logs/other/tests/gen-20-1-⚀⚁⚂⚃⚄⚅.log
    ./logs/other/tests/gen-20-2-2-faces.log
    ./logs/programs/AWS/elasticloadbalancer.log
    ./logs/programs/evtx/Microsoft-Windows-Kernel-PnP%4Configuration.evtx
    ./logs/programs/journal/RHE_91_system.journal
    ./logs/programs/journal/RHE_91_system.journal.gz
    ./logs/programs/pacman/pacman.log
    ./logs/programs/strace/strace-unix,us_ls.out
    ./logs/programs/utmp/host-entry6.wtmp
    ./logs/programs/utmp/host-entry6.wtmp.bz2
    ./logs/programs/utmp/host-entry6.wtmp.gz
    ./logs/programs/utmp/host-entry6.wtmp.lz4
    ./logs/programs/utmp/host-entry6.wtmp.tar
    ./logs/programs/utmp/host-entry6.wtmp.xz
    ./logs/standards/ctime.log
    ./logs/standards/ISO8601-YY-MM-DD.log
    ./logs/standards/ISO8601-YYYYDDMMTHHMM.log
    ./logs/standards/ISO8601-YYYYDDMMTHHMMSS.log
    ./logs/standards/ISO8601-YYYYMM.log
    ./logs/standards/ISO8601-YYYYMMDD.log
    ./logs/standards/ISO8601-YYYY-DD-MMTHH-MM-SS.log
    ./logs/standards/ISO8601-YYYY-DDD.log
    ./logs/standards/ISO8601-YYYY-MM-DD.log
    ./logs/standards/RFC-2822.log
    ./logs/standards/RFC-3164.log
    ./logs/standards/RFC-5424-2dot-0400.log
    ./logs/standards/RFC-5424-2dotZ.log
    ./logs/standards/RFC-5424-3dotZ.log
    ./logs/standards/RFC-5424-6dot-0700.log
    ./logs/standards/Unix-ms.log
    ./logs/standards/W3C-DTF.log
)

declare -i i=0


(
    set -x
    ./tools/cargo-llvm-cov-run.sh --output-path="${LCOV_OUTPUT_PATHD}/${i}.lcov" || true
)
i+=1
(
    set -x
    cargo llvm-cov test --locked --tests --lcov --output-path "${LCOV_OUTPUT_PATHD}/${i}.lcov"
)
i+=1
(
    set -x
    cargo llvm-cov run --locked --bin s4 --lcov --output-path="${LCOV_OUTPUT_PATHD}/${i}.lcov" -- &>/dev/null || true
)
i+=1
for args in \
    "--version" \
    "--help" \
    "--color=never" \
    "-s --color=never -l -n -w" \
    "-s --color=never -l -p -w" \
    "-s --color=never -u -n -w" \
    "-s --color=never -u -p -w" \
    "-s --color=always -l -n -w" \
    "-s --color=always -l -p -w" \
    "-s --color=always -u -n -w" \
    "-s --color=always -u -p -w" \
    "-s --color=always --light-theme -u -p -w" \
; do
    (
        set -x
        cargo llvm-cov run --locked --bin s4 --lcov --output-path="${LCOV_OUTPUT_PATHD}/${i}.lcov" -- ${args} "${S4_TEST_FILES[@]}" &>/dev/null || true
    )
    i+=1
done

set -x
lcov \
    $(find "${LCOV_OUTPUT_PATHD}" -maxdepth 1 -type f -name "*.lcov" -printf "--add-tracefile=%p ") \
    --output-file "${LCOV_OUTPUT_SUMMARY}"

genhtml -o "${HTML_OUTPUT_DIR}" "${LCOV_OUTPUT_SUMMARY}"
