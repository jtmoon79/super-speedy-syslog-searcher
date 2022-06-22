#!/usr/bin/env bash
#
# valgrind-callgrind.sh
#
# Run valgrind with Call Grind.
# https://valgrind.org/docs/manual/cl-manual.html
# This script runs `valgrind --tool=callgrind`
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
if ! callgrind=$(which callgrind_annotate); then
    echo "callgrind not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install valgrind" >&2
    exit 1
fi

declare -r bin=./target/release/s4

(set -x; uname -a)
(set -x; git log -n1 --format='%h %D')
(set -x; "${bin}" --version)
(set -x; $valgrind --version) | head -n1
(set -x; $callgrind --version) || true  # --version causes process return code 255

echo

declare -a files=(
    $(ls -1 ./logs/other/tests/gen-{100-10-......,100-10-BRAAAP,100-10-FOOBAR,100-10-______,100-10-skullcrossbones,100-4-happyface,1000-3-foobar,200-1-jajaja,400-4-shamrock}.log)
)

OUT=./callgrind.out
rm -f "${OUT}"

#export RUST_BACKTRACE=1
set -x

$valgrind --tool=callgrind \
    --collect-bus=yes \
    --collect-systime=yes \
    `#--separate-threads=yes` \
    --callgrind-out-file="${OUT}" \
    -- \
    "${bin}" \
        -z 0xFFFF \
        -a 20000101T000000 -b 20000101T080000 \
        "${files[@]}" \
    >/dev/null

exec \
    $callgrind \
    --tree=both \
    --show-percs=yes \
    $(find ./src -xdev -type d -exec echo -n '--include={} ' \;) \
    "${OUT}"
