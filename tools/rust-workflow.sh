#!/usr/bin/env bash
#
# Locally run the significant commands done in the
# `.github/workflows/rust.yml` file. Developers may save some time if they
# run this before pushing to github.com.
#
# Developers must manually update this script when `rust.yml` changes.
#
# requires:
#     cargo install --locked mlc cargo-nextest cargo-msrv
#     rustup component add clippy
#

set -euo pipefail

readonly HERE_DIR="$(dirname -- "${0}")"
cd "${HERE_DIR}/.."

declare -r S4R=./target/release/s4
declare -r S4D=./target/debug/s4
declare -ar S4_TEST_FILES=(
    ./logs/programs/Event_Trace_Log/waasmedic.20251005_113019_195.etl
    ./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/Common/FileCoAuth-2025-12-21.1214.4056.1.odl
    ./logs/other/tests/dtf2-2.log
    ./logs/other/tests/dtf3-2a.log
    ./logs/other/tests/dtf5-6a.log.gz
    ./logs/other/tests/dtf7-20-LEVELS.log.xz
    ./logs/other/tests/gen-2-1.tar
    ./logs/other/tests/gen-20-1-faces.log
    ./logs/other/tests/gen-20-1-⚀⚁⚂⚃⚄⚅.log
    ./logs/other/tests/gen-20-2-2-faces.log
)

set -x

# primitive argument parsing
do_clean=false
do_build=true
for arg in "${@}"; do
    if [[ "${arg}" == "--clean" ]]; then
        do_clean=true
    elif [[ "${arg}" == "--skip-build" ]]; then
        do_build=false
    elif [[ "${arg}" == "--help" || "${arg}" == "-h" || "${arg}" == "-?" ]]; then
        echo "Usage: ${0} [--clean] [--skip-build]"
        echo "    --clean: skip 'cargo clean'"
        echo "    --skip-build: skip 'cargo build' and related commands"
        exit 0
    else
        echo "Unknown argument: ${arg}" >&2
        echo "Usage: ${0} [--clean] [--skip-build]" >&2
        exit 1
    fi
done

if ${do_clean}; then
    cargo clean
fi
if ${do_build}; then
    cargo msrv verify  # cargo install cargo-msrv
    cargo build
    cargo build --profile release
    cargo build --profile mimalloc --features mimalloc
    cargo build --profile jemalloc --features jemalloc
    cargo build --profile rpmalloc --features rpmalloc
    cargo build --profile tcmalloc --features tcmalloc
    cargo build --profile alloc_tracker --features alloc_tracker
    cargo build --profile alloc_tracker --features alloc_tracker
    cargo build --profile flamegraph
    cargo build --profile valgrind
fi
./tools/log-files-time-update.sh
cargo test
cargo check --all-targets
cargo check --all-targets --release
cargo clippy --no-deps --all-targets
cargo bench --no-run --features bench_jetscii,bench_memchr,bench_stringzilla
cargo doc --locked --release --frozen --no-deps
cargo publish --dry-run --allow-dirty
"${S4R}" --venv
"${S4R}" --color=never "${S4_TEST_FILES[@]}" 2>/dev/null
"${S4D}" --color=never "${S4_TEST_FILES[@]}" 2>/dev/null
./tools/compare-current-and-expected/compare.sh
./tools/compare-debug-release.sh
./tools/compare-grep-sort.sh
./tools/compare-cat.sh
./tools/cargo-llvm-cov-combine-multiple.sh
env S4_BUILD_REGEX=1 ./tools/cross-builds.sh
mlc ./README.md
