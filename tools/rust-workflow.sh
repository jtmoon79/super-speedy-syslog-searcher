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

declare -r S4R=${S4R:-./target/release/s4}
declare -r S4D=${S4D:-./target/debug/s4}
declare -ar S4_TEST_FILES=(
    ./logs/other/tests/dtf2-2.log
    ./logs/other/tests/dtf3-2a.log
    ./logs/other/tests/dtf5-6a.log.gz
    ./logs/other/tests/dtf5-6b.log
    ./logs/other/tests/dtf5-6b.UTF-16BE_BOM.log
    ./logs/other/tests/dtf5-6b.UTF-16BE.log
    ./logs/other/tests/dtf5-6b.UTF-16LE_BOM.log
    ./logs/other/tests/dtf5-6b.UTF-16LE.log
    ./logs/other/tests/dtf5-6b.UTF-32BE_BOM.log
    ./logs/other/tests/dtf5-6b.UTF-32BE.log
    ./logs/other/tests/dtf5-6b.UTF-32LE_BOM.log
    ./logs/other/tests/dtf5-6b.UTF-32LE.log
    ./logs/other/tests/dtf5-6b.UTF-8_BOM.log
    ./logs/other/tests/dtf5-6b.UTF-8.log
    ./logs/other/tests/dtf7-20-LEVELS.log.xz
    ./logs/other/tests/gen-2-1.tar
    ./logs/other/tests/gen-20-1-⚀⚁⚂⚃⚄⚅.log
    ./logs/other/tests/gen-20-1-faces.log
    ./logs/other/tests/gen-20-2-2-faces.log
    ./logs/programs/Event_Trace_Log/waasmedic.20251005_113019_195.etl
    ./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/Common/FileCoAuth-2025-12-21.1214.4056.1.odl
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

export S4_BUILD_PRINT=1
export S4_BUILD_REGEX_PRINT=1

if ${do_clean}; then
    cargo clean
fi
if ${do_build}; then
    S4_BUILD_REGEX=1 cargo msrv verify  # cargo install cargo-msrv
    ./tools/build-all-profiles.sh
fi
./tools/log-files-time-update.sh
cargo test
(
    cd subprojects/ere/ere
    cargo test --features unstable-attr-regex,tests-compile-fail
)
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
