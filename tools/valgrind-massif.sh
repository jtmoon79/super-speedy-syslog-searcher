#!/usr/bin/env bash
#
# valgrind-massif.sh
#
# Run valgrind with heap profiling by Massif, `valgrind --tool=massif`.
# https://valgrind.org/docs/manual/ms-manual.html
#
# User can set environment variable $PROGRAM.
# Script arguments are passed to $PROGRAM.
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
if ! ms_print=$(which ms_print); then
    echo "ms_print not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install valgrind" >&2
    exit 1
fi

declare -r PROGRAM=${PROGRAM-./target/release/s4}

(set -x; uname -a)
(set -x; git log -n1 --format='%h %D' 2>/dev/null) || true
(set -x; "${PROGRAM}" --version)
(set -x; "${valgrind}" --version) | head -n1
(set -x; "${ms_print}" --version) || true  # --version causes process return code 255

echo

# default s4 arguments
declare -a args=(
    --color=never
    #./logs/other/tests/gen-1000-3-foobar.log
    #./logs/other/tests/gen-20-1-⚀⚁⚂⚃⚄⚅.log
    ./logs/other/tests/gen-99999-1-Motley_Crue.log
    #./logs/other/tests/gen-99999-1-Motley_Crue.log
    #./logs/other/tests/gen-400-4-shamrock.log
    #./logs/other/tests/gen-100-4-happyface.log
    #./logs/other/tests/gen-200-1-jajaja.log
    #./logs/other/tests/gen-100-10-BRAAAP.log
    #./logs/other/tests/gen-100-10-FOOBAR.log
)

if [[ ${#} -ge 1 ]]; then
    # use user-passed arguments
    args=()
    for arg in "${@}"; do
        args+=("${arg}")
    done
fi

OUT=${DIROUT-.}/massif.out
rm -f "${OUT}"

COLS_=$(($(tput cols) - 10))
LINES_=$(($(tput lines) - 10))

set -x

"${valgrind}" \
    --time-stamp=yes \
    --tool=massif \
    --heap=yes \
    --stacks=yes \
    --massif-out-file="${OUT}" \
    -- \
    "${PROGRAM}" \
        "${args[@]}" \
    >/dev/null

exec \
    "${ms_print}" \
    --x=${COLS_} \
    --y=${LINES_} \
    "${OUT}" \
