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

if [[ ! "${DIROUT+x}" ]]; then
    echo "Must set DIROUT environment variable" >&2
    exit 1
fi

if [[ ! "${S4_ALLOC_TRACKER_LINK-}" ]]; then
    commit_hash=$(git rev-parse HEAD)
    export S4_ALLOC_TRACKER_LINK="https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/${commit_hash}/"
fi

export DIROUT=${DIROUT-.}

if [[ ! "${VIRTUAL_ENV+x}" ]]; then
    echo "ERROR: not running within a Python virtual environment" >&2
    exit 1
fi

sudo --validate -p "update the cached sudo credentials (enter sudo password): "

mkdir -vp "${DIROUT}"

./tools/build-all-profiles.sh

(
    set -x
    ./target/release/s4 --venv
)

(
    set -x
    export RUST_MIN_STACK=50000000  # 50 MB
    ./tools/s4-alloc_trackers.sh
)

(
    set -x
    ./tools/cargo-bloat.sh -n 100
)

(
    set -x
    ./tools/osv-scanner.sh --format=markdown --output-file="${DIROUT}/osv-scanner.md"
    ./tools/mdtohtml.sh "${DIROUT}/osv-scanner.md"
) || true

(
    set -x
    ./tools/valgrind-callgrind.sh > "${DIROUT}/callgrind.txt"
)
rm -v "${DIROUT}/callgrind.out" "${DIROUT}/callgrind.dot" || true
./tools/clean-file.sh "${DIROUT}/callgrind.txt"

(
    set -x
    ./tools/valgrind-massif.sh > "${DIROUT}/massif.txt"
)
rm -v "${DIROUT}/massif.out" || true
./tools/clean-file.sh "${DIROUT}/massif.txt"

(
    set -x
    ./tools/heaptrack.sh ./tools/compare-log-mergers/*.log
)

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
    export PROGRAM=./target/mimalloc/s4
    ./tools/compare-grep-sort.sh &> "${DIROUT}/compare-grep-sort.txt"
)
./tools/clean-file.sh "${DIROUT}/compare-grep-sort.txt"

(
    export PROGRAMS_S4_LISTING=${TMPD-/tmp}/programs-s4-listing.tsv
    echo '
./target/release/s4
./target/jemalloc/s4
./target/mimalloc/s4
./target/rpmalloc/s4
./target/tcmalloc/s4
' > "${PROGRAMS_S4_LISTING}"
    set -x
    ./tools/compare-log-mergers/compare-log-mergers.sh --skip-tl &> "${DIROUT}/compare-log-mergers.txt"
)
./tools/clean-file.sh "${DIROUT}/compare-log-mergers.txt"

(
    set -x
    ./tools/flamegraphs.sh
)

(
    set -x
    ./tools/performance-plots.sh
)
