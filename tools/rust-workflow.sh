#!/usr/bin/env bash
#
# Locally run the significant commands done in the
# `.github/workflows/rust.yml` file. Developers may save some time if they
# run this before pushing to github.com.
#
# Developers must manually update this script when `rust.yml` changes.
#

set -euo pipefail

cd "$(dirname -- "${0}")/.."

declare -r S4R=./target/release/s4
declare -r S4D=./target/debug/s4
declare -ar S4_TEST_FILES=(
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

cargo clean
cargo msrv verify  # cargo install cargo-msrv
cargo build
cargo build --release
./tools/log-files-time-update.sh
cargo nextest run --all-targets
cargo check --all-targets
cargo check --all-targets --release
cargo clippy --no-deps --all-targets --all-features
cargo bench --no-run
cargo build --profile flamegraph
cargo build --profile valgrind
for TARGET in (
    aarch64-unknown-linux-gnu
    i686-pc-windows-gnu
    i686-pc-windows-msvc
    i686-unknown-linux-gnu
    x86_64-pc-windows-gnu
    x86_64-pc-windows-msvc
    x86_64-unknown-linux-gnu
    aarch64-unknown-linux-musl
    arm-unknown-linux-gnueabi
    arm-unknown-linux-gnueabihf
    armv7-unknown-linux-gnueabihf
    powerpc-unknown-linux-gnu
    powerpc64-unknown-linux-gnu
    riscv64gc-unknown-linux-gnu
    x86_64-unknown-freebsd
    x86_64-unknown-illumos
    x86_64-unknown-linux-musl
    x86_64-unknown-netbsd
    aarch64-linux-android
    i686-linux-android
    x86_64-pc-solaris
    x86_64-sun-solaris
    x86_64-linux-android
    x86_64-unknown-redox
    mips64-unknown-linux-gnuabi64
); do
    cross check --lib --bins --target $TARGET
done
cargo doc --locked --release --frozen --no-deps
cargo publish --dry-run --allow-dirty
"${S4R}" --color=never "${S4_TEST_FILES[@]}" 2>/dev/null
"${S4D}" --color=never "${S4_TEST_FILES[@]}" 2>/dev/null
./tools/compare-current-and-expected/compare.sh
./tools/compare-debug-release.sh
./tools/compare-grep-sort.sh
