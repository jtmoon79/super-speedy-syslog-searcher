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

DIFF=diff
if which colordiff &>/dev/null; then
    DIFF=colordiff
fi
readonly DIFF

# logs to process listed one per line
declare -r logs_file=./tools/compare-debug-release_logs.txt

#
# print some info for the script user, verify the s4 programs can run
#

declare -i ret=0
declare -a logs=()
while read log; do
    if [[ "${log}" =~ ^#.*$ || -z "${log}" ]]; then
        # skip comments and empty lines
        continue
    fi
    logs+=("${log}")
done <<< "$(cat "${logs_file}")"
echo >&2
echo "Comparing results of processing ${#logs[@]} files from \"${logs_file}\"" >&2
echo >&2

(set -x; "${DIFF}" --version) | head -n1
echo >&2

PROGRAMR=${PROGRAMR-./target/release/s4}
(set -x; "${PROGRAMR}" --version)
readonly PROGRAMR

PROGRAMD=${PROGRAMD-./target/debug/s4}
(set -x; "${PROGRAMD}" --version 2>/dev/null)
echo >&2
readonly PROGRAMD


#
# run s4 release and debug builds
#

# arguments for both release and debug
declare -ar S4_ARGS=(
    --color=never
    --tz-offset=+08:00
    --prepend-filename
    --prepend-file-align
    --prepend-utc
    --prepend-dt-format='%Y%m%dT%H%M%S.%9f'
    --prepend-separator='┋'
    --separator='⇳\n'
    --journal-output=export
    --dt-after='19990303T000000+0000'
    --dt-before='20240210T221032+0000'
    --summary
    "${@}"
)

# output of the release run
tmpr=$(mktemp -t "tmp.s4.compare-debug-release_release_XXXXX")
# output of the debug run
tmpd=$(mktemp -t "tmp.s4.compare-debug-release_debug_XXXXX")

for log in "${logs[@]}"; do
    echo "${log}" >&2
    # run the release build
    "${PROGRAMR}" "${S4_ARGS[@]}" "${log}" 2>/dev/null > "${tmpr}" || true
    # run the debug build (might take a few minutes)
    "${PROGRAMD}" "${S4_ARGS[@]}" "${log}" 2>/dev/null > "${tmpd}" || true

    if ! "${DIFF}" --text --brief "${tmpr}" "${tmpd}"; then
        echo -e "Files are not the same for log '${log}'.\n"
        ret=1
    fi
done

echo >&2

# run the release build
time (
    set -x
    "${PROGRAMR}" "${S4_ARGS[@]}" "${logs[@]}" 2>/dev/null > "${tmpr}"
) || true

echo >&2

# run the debug build (might take a few minutes)
time (
    set -x
    "${PROGRAMD}" "${S4_ARGS[@]}" "${logs[@]}" 2>/dev/null > "${tmpd}"
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

if ! "${DIFF}" --text --brief "${tmpr}" "${tmpd}"; then
    ret=1
    echo "Files are not the same. (ಠ_ಠ)"
    echo
    echo "Difference Preview:"
    (
        (
            set -x;
            "${DIFF}" --text -y --width=${COLUMNS-120} --suppress-common-lines "${tmpr}" "${tmpd}"
        ) || true
    ) | head -n 20
    echo
else
    echo
    echo "Files are the same. (ʘ‿ʘ)"
    rm -f "${tmpr}" "${tmpd}"
    echo
fi

exit ${ret}
