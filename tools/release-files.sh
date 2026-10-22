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

if [[ ! "${VIRTUAL_ENV+x}" ]]; then
    echo "ERROR: not running within a Python virtual environment" >&2
    exit 1
fi

export DIROUT=${DIROUT-.}

sudo --validate -p "update the cached sudo credentials (enter sudo password): "

mkdir -vp "${DIROUT}"

./tools/build-all-profiles.sh
./tools/cargo-deb.sh

# run compare-log-mergers.sh first so interactive tools (toolong) can be manually bypassed
(
    set -x
    ./tools/compare-log-mergers/compare-log-mergers.sh
)

(
    set -x
    ./tools/stack-sizes.sh
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
    ./tools/cargo-modules-structure.sh > "${DIROUT}/cargo-modules-structure.txt"
)

(
    set -x
    ./tools/osv-scanner.sh --format=markdown --output-file="${DIROUT}/osv-scanner.md"
    ./tools/mdtohtml.sh "${DIROUT}/osv-scanner.md"
) || true

(
    set -x
    ./tools/valgrind-callgrind.sh > "${DIROUT}/callgrind.txt"
    ./tools/clean-file.sh "${DIROUT}/callgrind.txt"
) || true
rm -v "${DIROUT}/callgrind.out" "${DIROUT}/callgrind.dot" || true

(
    set -x
    ./tools/valgrind-massif.sh > "${DIROUT}/massif.txt"
    ./tools/clean-file.sh "${DIROUT}/massif.txt"
) || true
rm -v "${DIROUT}/massif.out" || true

(
    set -x
    ./tools/heaptrack.sh ./tools/compare-log-mergers/*.log
) || true

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
    ./tools/compare-grep-sort.sh &> "${DIROUT}/compare-grep-sort.txt"
)
./tools/clean-file.sh "${DIROUT}/compare-grep-sort.txt"

(
    set -x
    ./tools/flamegraphs.sh
)

(
    set -x
    ./tools/performance-plots.sh
)
