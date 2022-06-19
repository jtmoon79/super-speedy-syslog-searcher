#!/usr/bin/env bash
#
# valgrind-dhat.sh
#
# Run valgrind with *Dynamic Heap Analysis Tool*.
# https://valgrind.org/docs/manual/dh-manual.html
# This script runs `valgrind --tool=dhat` without Rust crate `dhat`.
# The differences between the modes is described at
# https://docs.rs/dhat/latest/dhat/
#

set -euo pipefail

cd "$(dirname "${0}")/.."

# use full path to Unix tools
if ! valgrind=$(which valgrind); then
    echo "valgrind not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install valgrind g++" >&2
    exit 1
fi

declare -r bin=./target/release/s4

(set -x; uname -a)
(set -x; git log -n1 --format='%h %D')
(set -x; "${bin}" --version)
(set -x; $valgrind --version) | head -n1

echo

declare -a files=(
    $(ls -1 ./logs/other/tests/gen-{100-10-......,100-10-BRAAAP,100-10-FOOBAR,100-10-______,100-10-skullcrossbones,100-4-happyface,1000-3-foobar,200-1-jajaja,400-4-shamrock}.log)
)

(
export RUST_BACKTRACE=1
set -x
    valgrind --tool=dhat \
    "${bin}" \
    -z 0xFFFF \
    -a 20000101T000000 -b 20000101T080000 \
    "${files[@]}" \
    >/dev/null
)
