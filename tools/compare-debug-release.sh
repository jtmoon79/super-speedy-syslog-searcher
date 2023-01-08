#!/usr/bin/env bash
#
# compare-debug-release.sh
#
# run the local debug build and release build, compare the outputs
# ignore stderr any program return codes
# a simple, useful end-to-end test of differences and assertions.
# debug builds have more asserts and self-checks
#

set -euo pipefail

cd "$(dirname "${0}")/.."

(set -x; diff --version) | head -n1

PROGRAMR=${PROGRAMR-./target/release/s4}
(set -x; "${PROGRAMR}" --version)
PROGRAMD=${PROGRAMD-./target/debug/s4}
(set -x; "${PROGRAMD}" --version 2>/dev/null)

do_keep=false
if [[ "${1-}" = "--keep" ]]; then
    do_keep=true
    shift
fi

# output of the release run
tmpr=$(mktemp -t "compare-s4_release_XXXXX")
# output of the debug run
tmpd=$(mktemp -t "compare-s4_debug_XXXXX")

function exit_() {
    if ! ${do_keep}; then
        rm -f "${tmpr}" "${tmpd}"
    fi
}

declare -a logs=()
while read log; do
    logs[${#logs[@]}]=${log}
done <<< $(find ./logs -xdev -type f -size -2M | sort)

echo >&2

# run the release build
(
    set -x
    "${PROGRAMR}" \
        --color=never \
        --prepend-filename \
        '--tz-offset=+08:00' \
        "${logs[@]}" 2>/dev/null
) > "${tmpr}" || true

echo >&2

# run the debug build (might take a few minutes)
(
    set -x
    "${PROGRAMD}" \
        --color=never \
        --prepend-filename \
        '--tz-offset=+08:00' \
        "${logs[@]}" 2>/dev/null
) > "${tmpd}" || true

#
# compare the program outputs
#

# current s4 line count byte count
echo
s4r_lc=$(wc -l < "${tmpr}")
s4r_bc=$(wc -c < "${tmpr}")
echo "super_speedy_syslog_searcher release output '${tmpr}'"
echo "  Line Count ${s4r_lc}"
echo "  Byte Count ${s4r_bc}"
# expected s4 line count byte count
s4d_lc=$(wc -l < "${tmpd}")
s4d_bc=$(wc -c < "${tmpd}")
echo "super_speedy_syslog_searcher debug output '${tmpd}'"
echo "  Line Count ${s4d_lc}"
echo "  Byte Count ${s4d_bc}"

declare -i ret=0
if ! diff --text --brief "${tmpr}" "${tmpd}"; then
    ret=1
    echo "Files are not the same. (ಠ_ಠ)"
    echo
    echo "Difference Preview:"
    ((set -x; diff --text -y --width=${COLUMNS-120} --suppress-common-lines "${tmpr}" "${tmpd}") || true) | head -n 20
    echo
else
    echo
    echo "Files are the same. (ʘ‿ʘ)"
    echo
fi

exit ${ret}
