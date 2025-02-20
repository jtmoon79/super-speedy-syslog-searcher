#!/usr/bin/env bash
#
# Run some of the tools to create release files.
#
# may require apt packages:
#    graphviz gnuplot linux-perf valgrind g++
# and
#    python -m pip install -r ./tools/requirements.txt
#

set -euo pipefail

cd "$(dirname -- "${0}")/.."

export DIROUT=${DIROUT-.}

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
sed -i -e "s|$(realpath .)|.|g" "${DIROUT}/callgrind.txt"
sed -i -e "s|${HOME}|/home|g" "${DIROUT}/callgrind.txt"
sed -i -e "s|$(hostname)|host|g" "${DIROUT}/callgrind.txt"

(
    set -x
    ./tools/valgrind-massif.sh > "${DIROUT}/massif.txt"
)
rm -v "${DIROUT}/massif.out" || true
sed -i -e "s|$(realpath .)|.|g" "${DIROUT}/massif.txt"
sed -i -e "s|$(hostname)|host|g" "${DIROUT}/massif.txt"

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
    cargo build --release --features mimalloc
    ./tools/compare-grep-sort.sh &> "${DIROUT}/compare-grep-sort.txt"
    ./tools/compare-log-mergers/compare-log-mergers.sh &> "${DIROUT}/compare-log-mergers.txt"
)

sed -i -e "s|$(hostname)|host|g" "${DIROUT}/compare-grep-sort.txt"
sed -i -e "s|$(hostname)|host|g" "${DIROUT}/compare-log-mergers.txt"
