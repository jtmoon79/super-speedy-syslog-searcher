#!/usr/bin/env bash
#
# Run `heaptrack` and convert to a memory allocation flamegraph.
#
# Arguments are passed to `s4`
#
# install:
#   apt install heaptrack
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

declare -r PROGRAM=${PROGRAM:-"./target/valgrind/s4"}
declare -r BIN_TARGET=$(basename -- "${PROGRAM}")
declare -r DIROUT=${DIROUT-"."}
declare -r OUT_DATA=${DIROUT}/heaptrack.${BIN_TARGET}.data
# heaptrack automatically creates a .zst compressed version of the data file
declare -r OUT_ZST_DATA=${OUT_DATA}.zst
declare -r OUT_TXT_DATA=${DIROUT}/heaptrack.txt
declare -r OUT_TXT_HISTOGRAM=${DIROUT}/heaptrack.histogram.txt
declare -r OUT_FLAMEGRAPH_DATA=${DIROUT}/heaptrack.flamegraph.txt
declare -r OUT_SVG=${DIROUT}/heaptrack.svg

rm -f -- \
    "${OUT_DATA}" \
    "${OUT_ZST_DATA}" \
    "${OUT_TXT_DATA}" \

export RUST_BACKTRACE=1

(
    set -x

    "${PROGRAM}" --version

    heaptrack --output "${OUT_DATA}" "${PROGRAM}" -p --color=never "${@}" 1>/dev/null
    heaptrack --analyze -F "${OUT_FLAMEGRAPH_DATA}" "${OUT_ZST_DATA}"

    #heaptrack_print "${OUT_ZST_DATA}" --print-flamegraph "${OUT_FLAMEGRAPH_DATA}"
    heaptrack_print "${OUT_ZST_DATA}" --print-histogram "${OUT_TXT_HISTOGRAM}"
    #heaptrack_print "${OUT_ZST_DATA}" --print-massif "${OUT_TXT_MASSIF}"

    flamegraph.pl \
        --countname "allocations" \
        --title "allocations (${BIN_TARGET} -p --color=never ${*})" \
        "${OUT_FLAMEGRAPH_DATA}" > "${OUT_SVG}"

    # the title is now a long string so make the font smaller
    sed -i -Ee 's/<text id="title" /<text id="title" style="font-size:xx-small" /' --  "${OUT_SVG}"
)
