#!/usr/bin/env bash
#
# Run some of the tools to create release files.
#
# may require apt packages:
#    graphviz gnuplot linux-perf valgrind g++
# and
#    python -m pip install -r ./tools/requirements.txt
#
# It is probably best to run `cargo clean` before running this script.
#

set -euo pipefail

cd "$(dirname -- "${0}")/.."

export DIROUT=${DIROUT-.}

if [[ ! "${VIRTUAL_ENV+x}" ]]; then
    echo "ERROR: not running within a Python virtual environment" >&2
    exit 1
fi

function clean_file () {
    # remove specific names from the passed file path $1
    sed -i \
        -e "s|$(realpath .)|.|g" \
        -e "s|${HOME}|/home|g" \
        -e "s|$(hostname)|host|g" \
        -e "s|${USER}|user|g" \
        -- \
        "${1}"
}

sudo --validate -p "update the cached sudo credentials (enter sudo password): "

(
    set -x
    cargo build --locked --profile release
    cargo build --locked --profile jemalloc --features jemalloc
    cargo build --locked --profile mimalloc --features mimalloc
)

(
    set -x
    RUSTFLAGS=-g cargo build --profile flamegraph
    ./tools/flamegraphs.sh
)

(
    set -x
    RUSTFLAGS=-g cargo build --profile valgrind
    ./tools/valgrind-callgrind.sh > "${DIROUT}/callgrind.txt"
)
rm -v "${DIROUT}/callgrind.out" "${DIROUT}/callgrind.dot" || true

clean_file "${DIROUT}/callgrind.txt"

(
    set -x
    ./tools/valgrind-massif.sh > "${DIROUT}/massif.txt"
)
rm -v "${DIROUT}/massif.out" || true

clean_file "${DIROUT}/massif.txt"

(
    # XXX: cargo does not respect color settings
    #      see https://github.com/rust-lang/cargo/issues/9012
    export CARGO_TERM_COLOR=never
    set -x
    cargo bench --locked --features bench_jetscii,bench_memchr,bench_stringzilla --no-run
    # require gnuplot to be installed
    gnuplot --version
    cargo bench \
        --locked \
        --benches \
        --quiet \
        --color=never \
        --features bench_jetscii,bench_memchr,bench_stringzilla \
            &> "${DIROUT}/cargo-bench.txt"
)

(
    set -x
    # use allocator mimalloc for fastest results
    cargo build --profile mimalloc --features mimalloc
    ./tools/compare-grep-sort.sh &> "${DIROUT}/compare-grep-sort.txt"
)
clean_file "${DIROUT}/compare-grep-sort.txt"

(
    set -x
    ./tools/compare-log-mergers/compare-log-mergers.sh --skip-tl &> "${DIROUT}/compare-log-mergers.txt"
)
clean_file "${DIROUT}/compare-log-mergers.txt"
