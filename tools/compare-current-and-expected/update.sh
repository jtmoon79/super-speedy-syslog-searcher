#!/usr/bin/env bash
#
# update.sh
#
# helper to compare-current-and-expected.sh
# code in this script must agree with code in that script
#

set -euo pipefail

if [[ ${#} -ne 0 ]]; then
    echo "ERROR This script takes no arguments"
    exit 1
fi

cd "$(dirname "${0}")/../.."
HERE="./tools/compare-current-and-expected"
source "${HERE}/common.sh"

./tools/log-files-time-update.sh

# verify s4 can run
(set -x; "${PROGRAM}" --version)
echo >&2

touch "${EXPECT_OUT}" "${EXPECT_ERR}" || true

if ! chmod +w -- "${EXPECT_OUT}"; then
    echo "ERROR unable to write to file '${EXPECT_OUT}'" >&2
    exit 1
fi

# check twice for CI environments
if ! touch "${EXPECT_OUT}"; then
    echo "ERROR unable to write to file '${EXPECT_OUT}'" >&2
    exit 1
fi

if ! chmod +w -- "${EXPECT_ERR}"; then
    echo "ERROR unable to write to file '${EXPECT_ERR}'" >&2
    exit 1
fi

# check twice for CI environments
if ! touch "${EXPECT_ERR}"; then
    echo "ERROR unable to write to file '${EXPECT_ERR}'" >&2
    exit 1
fi

#
# print some info for the script user, verify the s4 program can run
#

if [[ ! -e "${LOGS}" ]]; then
    echo "ERROR file does not exist '${LOGS}'" >&2
    exit 1
elif [[ ! -r "${LOGS}" ]]; then
    echo "ERROR file is not readable '${LOGS}'" >&2
    exit 1
fi

cat "${LOGS}" >&2
declare -ir logs_wc=$(wc -l < "${LOGS}")
echo >&2
echo "${logs_wc} files in \"${LOGS}\"" >&2
echo >&2

#
# run s4 program
#

echo "${PS4}${PROGRAM} $(for arg in "${S4_ARGS[@]}"; do echo -n "'${arg}' "; done)- < '${LOGS}'" >&2
(
    set +e
    set +o pipefail
    "${PROGRAM}" "${S4_ARGS[@]}" - < "${LOGS}"
) 1> "${EXPECT_OUT}" 2> "${EXPECT_ERR}" || true

stderr_clean "${EXPECT_ERR}"

if ! chmod -wx -- "${EXPECT_OUT}"; then
    echo "WARNING unable to remove wx from file '${EXPECT_OUT}'" >&2
    # on Linux running on Windows NTFS mount, this is not a fatal error
fi

if ! chmod -wx -- "${EXPECT_ERR}"; then
    echo "WARNING unable to remove wx from file '${EXPECT_ERR}'" >&2
    # on Linux running on Windows NTFS mount, this is not a fatal error
fi

echo >&2
echo "Updated file '${EXPECT_OUT}'" >&2
echo "Updated file '${EXPECT_ERR}'" >&2

# update per-file runs
while read -r log_file; do
    if [[ "${log_file}" = '' ]] || [[ "${log_file:0:1}" = '#' ]]; then
        continue
    fi
    log_file_stdout="${HERE}/${log_file}.stdout"
    log_file_stderr="${HERE}/${log_file}.stderr"
    mkdir -p "$(dirname -- "${log_file_stdout}")"
    # run s4 and save stdout, stderr
    (
        "${PROGRAM}" "${S4_ARGS[@]}" "${log_file}" 1>"${log_file_stdout}" 2>"${log_file_stderr}"
    ) || true
    echo "Updated file '${log_file_stdout}'" >&2
    stderr_clean "${log_file_stderr}"
    echo "Updated file '${log_file_stderr}'" >&2
done < "${LOGS}"

echo >&2

echo -e "Now run \e[1m$(dirname "${0}")/compare.sh\e[0m" >&2
