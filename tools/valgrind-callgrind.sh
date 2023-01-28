#!/usr/bin/env bash
#
# valgrind-callgrind.sh
#
# Run valgrind with Call Grind.
# https://valgrind.org/docs/manual/cl-manual.html
# This script runs `valgrind --tool=callgrind`
#
# Article with specific tips for valgrind and rust
# https://nnethercote.github.io/2022/01/05/rust-and-valgrind.html
#
# User may set environment variable $PROGRAM.
# Passed arguments are passed to $PROGRAM and override default arguments.
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

declare -r PROGRAM=${PROGRAM-./target/release/s4}

(set -x; uname -a)
(set -x; git log -n1 --format='%h %D')
(set -x; "${PROGRAM}" --version)
(set -x; "${valgrind}" --version) | head -n1
(set -x; "${callgrind}" --version) || true  # --version causes process return code 255

echo

declare -a args=(
    -z 0xFFFF
    -a 20000101T000000
    -b 20000101T080000
    ./logs/other/tests/gen-100-10-......
    ./logs/other/tests/gen-100-10-BRAAAP.log
    ./logs/other/tests/gen-100-10-FOOBAR.log
    ./logs/other/tests/gen-100-10-______.log
    ./logs/other/tests/gen-100-10-skullcrossbones.log
    ./logs/other/tests/gen-100-4-happyface.log
    ./logs/other/tests/gen-1000-3-foobar.log
    ./logs/other/tests/gen-200-1-jajaja.log
    ./logs/other/tests/gen-400-4-shamrock.log
)

if [[ ${#} -ge 1 ]]; then
    # use user-passed arguments
    args=()
    for arg in "${@}"; do
        args+=("${arg}")
    done
fi

OUT=./callgrind.out
rm -f "${OUT}"

set -x

"${valgrind}" \
    --tool=callgrind \
    --collect-bus=yes \
    --collect-systime=yes \
    `#--separate-threads=yes` \
    --callgrind-out-file="${OUT}" \
    -- \
    "${PROGRAM}" \
        "${args[@]}" \
    >/dev/null

exec \
    "${callgrind}" \
    --tree=both \
    --show-percs=yes \
    $(find ./src -xdev -type d -exec echo -n '--include={} ' \;) \
    "${OUT}"
