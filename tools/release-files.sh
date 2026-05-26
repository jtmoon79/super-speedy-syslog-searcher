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

export DIROUT=${DIROUT-.}

if [[ ! "${VIRTUAL_ENV+x}" ]]; then
    echo "ERROR: not running within a Python virtual environment" >&2
    exit 1
fi

sudo --validate -p "update the cached sudo credentials (enter sudo password): "

mkdir -vp "${DIROUT}"

# prebuilds in parallel
(
    declare -a PIDs=()

    set -x
    cargo build --locked &
    PIDs+=($!)
    cargo build --locked --profile release &
    PIDs+=($!)
    cargo build --locked --profile jemalloc --features jemalloc &
    PIDs+=($!)

    wait ${PIDs[@]}

    ./target/debug/s4 --version
    ./target/release/s4 --version
    ./target/jemalloc/s4 --version
)

# prebuilds in parallel
(
    declare -a PIDs=()

    set -x
    cargo build --locked --profile mimalloc --features mimalloc &
    PIDs+=($!)
    cargo build  --locked --profile alloc_tracker --features alloc_tracker &
    PIDs+=($!)

    wait ${PIDs[@]}

    ./target/mimalloc/s4 --version
    ./target/alloc_tracker/s4 --version
)

# prebuilds in parallel
(
    declare -a PIDs=()

    set -x
    cargo build --locked --profile rpmalloc --features rpmalloc &
    PIDs+=($!)
    cargo build --locked --profile tcmalloc --features tcmalloc &
    PIDs+=($!)

    wait ${PIDs[@]}

    ./target/rpmalloc/s4 --version
    ./target/tcmalloc/s4 --version
)

# prebuilds in parallel
(
    declare -a PIDs=()

    set -x
    RUSTFLAGS=-g cargo build --locked --profile flamegraph &
    PIDs+=($!)
    RUSTFLAGS=-g cargo build --locked --profile valgrind &
    PIDs+=($!)

    wait ${PIDs[@]}

    ./target/flamegraph/s4 --version
    ./target/valgrind/s4 --version
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
    ./tools/performance-plots.sh
)

(
    set -x
    ./tools/osv-scanner.sh --format=markdown --output-file="${DIROUT}/osv-scanner.md"
) || true

(
    set -x
    ./tools/valgrind-callgrind.sh > "${DIROUT}/callgrind.txt"
)
rm -v "${DIROUT}/callgrind.out" "${DIROUT}/callgrind.dot" || true
./scripts/clean-file.sh "${DIROUT}/callgrind.txt"

(
    set -x
    ./tools/valgrind-massif.sh > "${DIROUT}/massif.txt"
)
rm -v "${DIROUT}/massif.out" || true
./scripts/clean-file.sh "${DIROUT}/massif.txt"

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
./scripts/clean-file.sh "${DIROUT}/compare-grep-sort.txt"

(
    set -x
    ./tools/compare-log-mergers/compare-log-mergers.sh --skip-tl &> "${DIROUT}/compare-log-mergers.txt"
)
./scripts/clean-file.sh "${DIROUT}/compare-log-mergers.txt"

(
    set -x
    ./tools/flamegraphs.sh
)
