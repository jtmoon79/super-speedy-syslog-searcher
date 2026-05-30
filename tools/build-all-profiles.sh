#!/usr/bin/env bash
#
# build all important profiles, three at a time in parallel

set -euo pipefail

cd "$(dirname -- "${0}")/.."

# debug, release, jemalloc

set -x

declare -a PIDs=()

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

# mimalloc, tcmalloc, rpmalloc

PIDs=()

cargo build --locked --profile mimalloc --features mimalloc &
PIDs+=($!)
cargo build --locked --profile tcmalloc --features tcmalloc &
PIDs+=($!)
cargo build --locked --profile rpmalloc --features rpmalloc &
PIDs+=($!)

wait ${PIDs[@]}

./target/mimalloc/s4 --version
./target/tcmalloc/s4 --version
./target/rpmalloc/s4 --version

# alloc_tracker, flamegraph, valgrind

PIDs=()

cargo build  --locked --profile alloc_tracker --features alloc_tracker &
PIDs+=($!)
RUSTFLAGS=-g cargo build --locked --profile flamegraph &
PIDs+=($!)
RUSTFLAGS=-g cargo build --locked --profile valgrind &
PIDs+=($!)

wait ${PIDs[@]}

./target/alloc_tracker/s4 --version
./target/flamegraph/s4 --version
./target/valgrind/s4 --version
