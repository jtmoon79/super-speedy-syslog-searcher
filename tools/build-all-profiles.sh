#!/usr/bin/env bash
#
# build all important profiles, three at a time in parallel

set -euo pipefail

cd "$(dirname -- "${0}")/.."

if which renice &>/dev/null; then
    # the `rustc` processes are long-running aggressive users of CPU+RAM resources
    # so renice them to ease CPU pressure
    renice -n 20 -p $$
fi

export S4_BUILD_PRINT=1
export S4_BUILD_REGEX_PRINT=1

set -x

# debug, release

declare -a PIDs=()

cargo build --locked --profile dev &
PIDs+=($!)
cargo build --locked --profile release &
PIDs+=($!)

wait ${PIDs[@]}

./target/debug/s4 --version
./target/release/s4 --version

# jemalloc, rpmalloc

PIDs=()

cargo build --locked --profile jemalloc --features jemalloc &
PIDs+=($!)
cargo build --locked --profile rpmalloc --features rpmalloc &
PIDs+=($!)

wait ${PIDs[@]}

./target/jemalloc/s4 --version
./target/rpmalloc/s4 --version

# mimalloc, tcmalloc

PIDs=()

cargo build --locked --profile mimalloc --features mimalloc &
PIDs+=($!)
cargo build --locked --profile tcmalloc --features tcmalloc &
PIDs+=($!)

wait ${PIDs[@]}

./target/mimalloc/s4 --version
./target/tcmalloc/s4 --version


# alloc_tracker, flamegraph

PIDs=()

cargo build  --locked --profile alloc_tracker --features alloc_tracker &
PIDs+=($!)
RUSTFLAGS=-g cargo build --locked --profile flamegraph &
PIDs+=($!)

wait ${PIDs[@]}

./target/alloc_tracker/s4 --version
./target/flamegraph/s4 --version

# valgrind, check

PIDs=()

RUSTFLAGS=-g cargo build --locked --profile valgrind &
PIDs+=($!)
cargo check --locked &
PIDs+=($!)

wait ${PIDs[@]}

./target/valgrind/s4 --version

# release_Opt0

declare -a PIDs=()

cargo build --locked --profile release_Opt0 &
PIDs+=($!)

wait ${PIDs[@]}

./target/release_Opt0/s4 --version

# release_O0_cgu1, release_O0_cgu512_pa

declare -a PIDs=()

cargo build --locked --profile release_O0_cgu1 &
PIDs+=($!)
cargo build --locked --profile release_O0_cgu512_pa &
PIDs+=($!)

wait ${PIDs[@]}

./target/release_O0_cgu1/s4 --version
./target/release_O0_cgu512_pa/s4 --version

# release_O1_cgu1, release_O1_cgu256_pa, release_O1_cgu512_pa

declare -a PIDs=()

cargo build --locked --profile release_O1_cgu1 &
PIDs+=($!)
cargo build --locked --profile release_O1_cgu256_pa &
PIDs+=($!)
cargo build --locked --profile release_O1_cgu512_pa &
PIDs+=($!)

wait ${PIDs[@]}

./target/release_O1_cgu1/s4 --version
./target/release_O1_cgu256_pa/s4 --version
./target/release_O1_cgu512_pa/s4 --version

# release_O2_cgu1, release_O2_cgu256_pa, release_O2_cgu512_pa

declare -a PIDs=()

cargo build --locked --profile release_O2_cgu1 &
PIDs+=($!)
cargo build --locked --profile release_O2_cgu256_pa &
PIDs+=($!)
cargo build --locked --profile release_O2_cgu512_pa &
PIDs+=($!)

wait ${PIDs[@]}

./target/release_O2_cgu1/s4 --version
./target/release_O2_cgu256_pa/s4 --version
./target/release_O2_cgu512_pa/s4 --version

# release_O3_cgu1_pa, release_O3_cgu256_pa, release_O3_cgu512_pa

declare -a PIDs=()

cargo build --locked --profile release_O3_cgu1_pa &
PIDs+=($!)
cargo build --locked --profile release_O3_cgu256_pa &
PIDs+=($!)
cargo build --locked --profile release_O3_cgu512_pa &
PIDs+=($!)

wait ${PIDs[@]}

./target/release_O3_cgu1_pa/s4 --version
./target/release_O3_cgu256_pa/s4 --version
./target/release_O3_cgu512_pa/s4 --version
