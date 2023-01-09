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

do_keep=false
if [[ "${1-}" = "--keep" ]]; then
    do_keep=true
    shift
fi

# output of the release run
tmpr=$(mktemp -t "tmp.s4.compare-debug-release_release_XXXXX")
# output of the debug run
tmpd=$(mktemp -t "tmp.s4.compare-debug-release_debug_XXXXX")
# output of the debug run
logs=$(mktemp -t "tmp.s4.compare-debug-release_logs_XXXXX")

function exit_() {
    if ! ${do_keep}; then
        rm -f -- "${tmpr}" "${tmpd}" "${logs}"
    fi
}
trap exit_ EXIT

path=./logs

(find "${path}" -xdev -type f -size -2M | sort) > "${logs}"

#
# print some info for the script user, verify the s4 programs can run
#

echo >&2
cat "${logs}" >&2
echo >&2
echo "$(wc -l < "${logs}") files under \"${path}\"" >&2
echo >&2

(set -x; diff --version) | head -n1
PROGRAMR=${PROGRAMR-./target/release/s4}
(set -x; "${PROGRAMR}" --version)
PROGRAMD=${PROGRAMD-./target/debug/s4}
(set -x; "${PROGRAMD}" --version 2>/dev/null)
echo >&2

#
# run s4 release and debug builds
#

# arguments for both release and debug
declare -ar S4_ARGS=(
    --color=never
    --tz-offset=+08:00
    --prepend-filename
    '-'
    "${@}"
)

# run the release build
time (
    set -x
    "${PROGRAMR}" "${S4_ARGS[@]}" 2>/dev/null > "${tmpr}" < "${logs}"
) || true

echo >&2

# run the debug build (might take a few minutes)
time (
    set -x
    "${PROGRAMD}" "${S4_ARGS[@]}" 2>/dev/null > "${tmpd}" < "${logs}"
) || true

#
# compare the program outputs debug and release outputs
#

# s4 release line count, byte count; only informative
echo
s4r_lc=$(wc -l < "${tmpr}")
s4r_bc=$(wc -c < "${tmpr}")
echo "super_speedy_syslog_searcher release output '${tmpr}'"
echo "  Line Count ${s4r_lc}"
echo "  Byte Count ${s4r_bc}"

# s4 debug line count, byte count; only informative
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