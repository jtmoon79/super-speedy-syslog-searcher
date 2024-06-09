#!/usr/bin/env bash
#
# Run `heaptrack` and convert to a memory allocation flamegraph.
#
# install:
#   apt install heaptrack
#
# https://github.com/KDE/heaptrack
#

set -euo pipefail

cd "$(dirname -- "${0}")/.."

function did_install () {
    echo "Did you:"
    echo "    apt install -y heaptrack"
    echo "    wget 'https://raw.githubusercontent.com/brendangregg/FlameGraph/master/flamegraph.pl'"
}

if ! which heaptrack heaptrack_print &>/dev/null; then
    echo "heaptrack is not in the PATH" >&2
    did_install >&2
    exit 1
fi

if ! which flamegraph.pl &>/dev/null; then
    echo "flamegraph.pl is not in the PATH" >&2
    did_install >&2
    exit 1
fi

echo -e "Requires building with \e[1m-g\e[0m
    RUSTFLAGS=-g cargo build --profile valgrind
" >&2
sleep 1

declare -r BIN=./target/valgrind/s4
declare -r BIN_TARGET=s4
NOTES=$("${BIN}" --version | head -n1)
declare -r OUT=heaptrack.${BIN_TARGET}.data
declare -r OUT_ZST_DATA=${OUT}.zst
declare -r OUT_TXT_DATA=${OUT}.txt
declare -r OUT_TXT_F=${OUT}.flamegraph.txt
declare -r OUT_TXT_TEMP=${OUT}.flamegraph.temporary.txt
declare -r OUT_TXT_H=${OUT}.histogram.txt
declare -r OUT_TXT_MASSIF=${OUT}.massif.txt
declare -r OUT_SVG=${OUT}.svg
declare -r OUT_SVG_F=${OUT}.flamegraph.svg
declare -r OUT_SVG_H=${OUT}.histogram.svg
declare -r OUT_SVG_TEMP=${OUT}.histogram.temporary.svg
declare -r OUT_SVG_MASSIF=${OUT}.massif.svg
declare -r DUMMY=dummy

rm -f -- \
    "${OUT}" \
    "${OUT_ZST_DATA}" \
    "${OUT_TXT_DATA}" \
    "${OUT_TXT_F}" \
    "${OUT_TXT_TEMP}" \
    "${OUT_TXT_H}" \
    "${OUT_TXT_MASSIF}" \
    "${OUT_SVG_F}" \
    "${OUT_SVG_H}" \
    "${OUT_SVG_TEMP}" \
    "${OUT_SVG_MASSIF}" \

export RUST_BACKTRACE=1

#trap "rm -f ${DUMMY}" EXIT

(
    set -x

    # heaptrack appends `.zst` to the output file name
    heaptrack --output "${OUT}" "${BIN}" -p --color=never "${@}" 1>/dev/null
    heaptrack --analyze -F "${OUT_TXT_DATA}" "${OUT_ZST_DATA}"

    heaptrack_print "${OUT_ZST_DATA}" --print-flamegraph "${OUT_TXT_DATA}"
    heaptrack_print "${OUT_ZST_DATA}" --print-histogram "${OUT_TXT_H}"
    heaptrack_print "${OUT_ZST_DATA}" --print-massif "${OUT_TXT_MASSIF}"

    flamegraph.pl \
        --countname "allocations" \
        --title "allocations (${BIN_TARGET} -p --color=never ${*})" \
        "${OUT_TXT_DATA}" > "${OUT_SVG}"

    flamegraph.pl \
        --countname "allocations massif" \
        --title "allocations massif (${BIN_TARGET} -p --color=never ${*})" \
        "${OUT_TXT_MASSIF}" > "${OUT_SVG_MASSIF}"
)
