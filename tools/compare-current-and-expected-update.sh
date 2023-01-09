#!/usr/bin/env bash
#
# compare-current-and-expected-update.sh
#
# helper to compare-current-and-expected.sh
# code in this script must agree with code in that script, especially the
# s4 command-line arguments and the `find` search for logs
#

set -euo pipefail

cd "$(dirname "${0}")/.."

PROGRAM=${PROGRAM-./target/release/s4}
(set -x; "${PROGRAM}" --version)

expect1=./tools/compare-current-and-expected_expected.out

if ! chmod +w -- "${expect1}"; then
    echo "ERROR unable to write to file '${expect1}'" >&2
    exit 1
fi

if ! touch "${expect1}"; then
    echo "ERROR unable to write to file '${expect1}'" >&2
    exit 1
fi

#
# print some info for the script user, verify the s4 program can run
#

logs='./tools/compare-current-and-expected_logs.txt'

if [[ ! -e "${logs}" ]]; then
    echo "ERROR file does not exist '${logs}'" >&2
    exit 1
elif [[ ! -r "${logs}" ]]; then
    echo "ERROR file is not readable '${logs}'" >&2
    exit 1
fi

cat "${logs}" >&2
echo >&2
echo "$(wc -l < "${logs}") files under \"${logs}\"" >&2
echo >&2

PROGRAM=${PROGRAM-./target/release/s4}
(set -x; "${PROGRAM}" --version)
echo >&2

# these arguments must agree with `compare-current-and-expected.sh`
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
) > "${expect1}" || true

if ! chmod -wx -- "${expect1}"; then
    echo "ERROR unable to remove wx from file '${expect1}'" >&2
    exit 1
fi

echo >&2
echo "Updated file '${expect1}'" >&2
echo -e "Now run \e[1mcompare-current-and-expected.sh\e[0m." >&2
