#!/usr/bin/env bash
#
# build all important profiles in parallel

set -euo pipefail

cd "$(dirname -- "${0}")/.."

if command -v renice &>/dev/null && [[ "${RENICE-}" = "1" ]]; then
    # the `rustc` processes are long-running aggressive users of CPU+RAM resources
    # so renice them to ease CPU pressure
    renice -n 20 -p $$
fi

# run
function r() {
    (set -euxo pipefail; "${@}")
}

# global tracker of PIDs for next `wait`
declare -ag PIDs=()

# run + save PID for `wait`
function rw() {
    r "${@}" &
    PIDs+=($!)
}

if [[ "${DIROUT-}" ]]; then
    mkdir -p "${DIROUT}"
fi

VERSION=$(
    set -euo pipefail
    grep -m1 -Eo '^version\s*=\s*"[0-9]+\.[0-9]+\.[0-9]+"' Cargo.toml \
    | cut -d'"' -f2- \
    | tr -d '"'
)
readonly VERSION

function archive () {
    if [[ ! "${DIROUT-}" ]]; then
        return 0
    fi
    local binary_path="$1"
    local target="$2"
    ./tools/package-binary.sh "$binary_path" "$target" "$VERSION"
}

DEFAULT_TARGET=$(default-target)
readonly DEFAULT_TARGET

# debug, release

rw cargo build --quiet --locked --profile dev
rw cargo build --quiet --locked --profile release
rw cargo test --quiet --locked

wait ${PIDs[@]}

r ./target/debug/s4 --version
r ./target/release/s4 --version

archive "./target/release/s4" "${DEFAULT_TARGET}-release"

# jemalloc, rpmalloc, mimalloc, tcmalloc

PIDs=()

rw cargo build --quiet --locked --profile jemalloc --features jemalloc
rw cargo build --quiet --locked --profile rpmalloc --features rpmalloc
rw cargo build --quiet --locked --profile mimalloc --features mimalloc
rw cargo build --quiet --locked --profile tcmalloc --features tcmalloc

wait ${PIDs[@]}

r ./target/jemalloc/s4 --version
r ./target/rpmalloc/s4 --version
r ./target/mimalloc/s4 --version
r ./target/tcmalloc/s4 --version

archive "./target/jemalloc/s4" "${DEFAULT_TARGET}-jemalloc"
archive "./target/rpmalloc/s4" "${DEFAULT_TARGET}-rpmalloc"
archive "./target/mimalloc/s4" "${DEFAULT_TARGET}-mimalloc"
#archive "./target/tcmalloc/s4" "${DEFAULT_TARGET}-tcmalloc"

# alloc_tracker, flamegraph

PIDs=()

rw cargo build --quiet --locked --profile alloc_tracker --features alloc_tracker
rw env RUSTFLAGS=-g cargo build --quiet --locked --profile flamegraph
rw cargo publish --quiet --locked --dry-run --allow-dirty

wait ${PIDs[@]}

r ./target/alloc_tracker/s4 --version
r ./target/flamegraph/s4 --version

# x86_64 x86_64-v2, x86_64-v3, x86_64-v4

PIDs=()

rw cargo build-x86_64-v2-musl --quiet --locked --profile x86_64-v2-musl
rw cargo build-x86_64-v3-musl --quiet --locked --profile x86_64-v3-musl
rw cargo build-x86_64-v4-musl --quiet --locked --profile x86_64-v4-musl

wait ${PIDs[@]}

r ./target/x86_64-unknown-linux-musl/x86_64-v2-musl/s4 --version
r ./target/x86_64-unknown-linux-musl/x86_64-v3-musl/s4 --version
r ./target/x86_64-unknown-linux-musl/x86_64-v4-musl/s4 --version

archive "./target/x86_64-unknown-linux-musl/x86_64-v2-musl/s4" "x86_64-v2-musl"
archive "./target/x86_64-unknown-linux-musl/x86_64-v3-musl/s4" "x86_64-v3-musl"
archive "./target/x86_64-unknown-linux-musl/x86_64-v4-musl/s4" "x86_64-v4-musl"

# x86_64-v2, x86_64-v3, x86_64-v4 x86_64-unknown-linux-gnu

PIDs=()

rw cargo build-x86_64-gnu-v2 --quiet --locked --profile x86_64-gnu-v2
rw cargo build-x86_64-gnu-v3 --quiet --locked --profile x86_64-gnu-v3
rw cargo build-x86_64-gnu-v4 --quiet --locked --profile x86_64-gnu-v4

wait ${PIDs[@]}

r ./target/x86_64-unknown-linux-gnu/x86_64-gnu-v2/s4 --version
r ./target/x86_64-unknown-linux-gnu/x86_64-gnu-v3/s4 --version
r ./target/x86_64-unknown-linux-gnu/x86_64-gnu-v4/s4 --version

archive "./target/x86_64-unknown-linux-gnu/x86_64-gnu-v2/s4" "x86_64-gnu-v2"
archive "./target/x86_64-unknown-linux-gnu/x86_64-gnu-v3/s4" "x86_64-gnu-v3"
archive "./target/x86_64-unknown-linux-gnu/x86_64-gnu-v4/s4" "x86_64-gnu-v4"

# clippy, docs, valgrind, check

PIDs=()

rw cargo clippy --locked --quiet --no-deps --all-targets --all-features
rw cargo doc --locked --quiet --release --no-deps
rw cargo check --locked
rw env RUSTFLAGS=-g cargo build --quiet --locked --profile valgrind

wait ${PIDs[@]}

r ./target/valgrind/s4 --version

# release_Opt0, release_O0_cgu1, release_O0_cgu512_pa

PIDs=()

rw cargo build --quiet --locked --profile release_Opt0
rw cargo build --quiet --locked --profile release_O0_cgu1
rw cargo build --quiet --locked --profile release_O0_cgu512_pa

wait ${PIDs[@]}

r ./target/release_Opt0/s4 --version
r ./target/release_O0_cgu1/s4 --version
r ./target/release_O0_cgu512_pa/s4 --version

# release_O1_cgu1, release_O1_cgu256_pa, release_O1_cgu512_pa

PIDs=()

rw cargo build --quiet --locked --profile release_O1_cgu1
rw cargo build --quiet --locked --profile release_O1_cgu256_pa
rw cargo build --quiet --locked --profile release_O1_cgu512_pa

wait ${PIDs[@]}

r ./target/release_O1_cgu1/s4 --version
r ./target/release_O1_cgu256_pa/s4 --version
r ./target/release_O1_cgu512_pa/s4 --version

# release_O2_cgu1, release_O2_cgu256_pa, release_O2_cgu512_pa

PIDs=()

rw cargo build --quiet --locked --profile release_O2_cgu1
rw cargo build --quiet --locked --profile release_O2_cgu256_pa
rw cargo build --quiet --locked --profile release_O2_cgu512_pa

wait ${PIDs[@]}

r ./target/release_O2_cgu1/s4 --version
r ./target/release_O2_cgu256_pa/s4 --version
r ./target/release_O2_cgu512_pa/s4 --version

# release_O3_cgu1_pa, release_O3_cgu256_pa, release_O3_cgu512_pa

PIDs=()

rw cargo build --quiet --locked --profile release_O3_cgu1_pa
rw cargo build --quiet --locked --profile release_O3_cgu256_pa
rw cargo build --quiet --locked --profile release_O3_cgu512_pa

wait ${PIDs[@]}

r ./target/release_O3_cgu1_pa/s4 --version
r ./target/release_O3_cgu256_pa/s4 --version
r ./target/release_O3_cgu512_pa/s4 --version
