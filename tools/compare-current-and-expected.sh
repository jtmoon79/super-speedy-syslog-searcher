#!/usr/bin/env bash
#
# compare-current-and-expected.sh
#
# compare run output of the current build of super_speedy_syslog_searcher to
# an expected output of a known good build of super_speedy_syslog_searcher
# (presumably git committed). This is a simple end-to-end test.
# The success of this tests depends upon
# - the `./logs` directory being clean
# - the `./logs` files being updated with `log-files-time-update.sh`
# - git commits affecting `./logs` being reflected in file `compare-current-and-expected_expected.out`
#

set -euo pipefail

cd "$(dirname "${0}")/.."

current1=./tools/compare-current-and-expected_current.out
expect1=./tools/compare-current-and-expected_expected.out

if ! touch "${current1}"; then
    echo "ERROR unable to write to file '${current1}'" >&2
    exit 1
fi

if [[ ! -e "${expect1}" ]]; then
    echo "ERROR can not find file '${expect1}'" >&2
    exit 1
elif [[ ! -r "${expect1}" ]]; then
    echo "ERROR can not read file '${expect1}'" >&2
    exit 1
fi

#
# get list of files to process
#

# output of the debug run
logs=$(mktemp -t "tmp.s4.compare-current-and-expected_logs_XXXXX")

function exit_() {
    rm -f -- "${logs}"
}
trap exit_ EXIT

path=./logs

(find "${path}" -xdev -type f -size -2M | sort) > "${logs}"

#
# print some info for the script user, verify the s4 program can run
#

echo >&2
cat "${logs}" >&2
echo >&2
echo "$(wc -l < "${logs}") files under \"${path}\"" >&2
echo >&2

PROGRAM=${PROGRAM-./target/release/s4}
(set -x; "${PROGRAM}" --version)
echo >&2

#
# run s4 program
#

declare -ar S4_ARGS=(
    --color=never
    --tz-offset=+08:00
    --prepend-filename
    '-'
    "${@}"
)

(
    set -x
    "${PROGRAM}" "${S4_ARGS[@]}" 2>/dev/null < "${logs}"
) > "${current1}" || true

#
# compare the program outputs
#

# current s4 line count, byte count; only informative
echo
s4_lc=$(wc -l < "${current1}")
s4_bc=$(wc -c < "${current1}")
echo "current super_speedy_syslog_searcher output in file '${current1}'"
echo "  Line Count ${s4_lc}"
echo "  Byte Count ${s4_bc}"

# expected s4 line count, byte count; only informative
ex_lc=$(wc -l < "${expect1}")
ex_bc=$(wc -c < "${expect1}")
echo "expect super_speedy_syslog_searcher output in file '${expect1}'"
echo "  Line Count ${ex_lc}"
echo "  Byte Count ${ex_bc}"

declare -i ret=0
if ! diff --text --brief "${current1}" "${expect1}"; then
    ret=1
    echo "Files are not the same. (ಠ_ಠ)"
    echo
    echo "Difference Preview:"
    ((set -x; diff --text -y --width=${COLUMNS-120} --suppress-common-lines "${current1}" "${expect1}") || true) | head -n 20
    echo
else
    echo
    echo "Files are the same. (ʘ‿ʘ)"
    echo
fi

exit ${ret}
