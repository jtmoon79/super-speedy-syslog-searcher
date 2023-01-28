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
# User can set environment variable $PROGRAM
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

PROGRAM=${PROGRAM-./target/release/s4}

(set -x; uname -a)
(set -x; git log -n1 --format='%h %D') || true
(set -x; "${PROGRAM}" --version)
(set -x; "${valgrind}" --version) | head -n1

echo

# default arguments for s4
declare -a args=(
    -z 0xFFFF
    -a 20000101T000000
    -b 20000101T080000
    ./logs/other/tests/dtf5-3-LF-CR.log
    ./logs/other/tests/dtf5-6a.log.gz.tar
    ./logs/other/tests/dtf5-6a.log.xz
    ./logs/other/tests/dtf7-20-LEVELS.log
    ./logs/other/tests/dtf7-20-LEVELS.log
    ./logs/other/tests/dtf7-20-LEVELS.log.gz
    ./logs/other/tests/dtf7-20-LEVELS.log.old
    ./logs/other/tests/dtf7-20-LEVELS.log.old.gz
    ./logs/other/tests/dtf7-20-LEVELS.log.tar
    ./logs/other/tests/dtf7-20-LEVELS.log.xz
    ./logs/other/tests/dtf9d-12x3-37.log
    ./logs/other/tests/gen-20-1-🌚🌛🌜🌝.log
    ./logs/other/tests/gen-20-1-⚀⚁⚂⚃⚄⚅.log
    ./logs/other/tests/gen-100-10-.......log
    ./logs/other/tests/gen-100-10-BRAAAP.log
    ./logs/other/tests/gen-100-10-FOOBAR.log
    ./logs/other/tests/gen-100-10-______.log
    ./logs/other/tests/gen-100-10-skullcrossbones.log
    ./logs/other/tests/gen-100-10-skullcrossbones.log.gz
    ./logs/other/tests/gen-100-10-skullcrossbones.log.xz
    ./logs/other/tests/gen-100-10-skullcrossbones.tar
    ./logs/other/tests/gen-100-10.tar
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

set -x

exec \
    "${valgrind}" \
        --tool=dhat \
        "${PROGRAM}" \
        "${args[@]}" \
        >/dev/null \
