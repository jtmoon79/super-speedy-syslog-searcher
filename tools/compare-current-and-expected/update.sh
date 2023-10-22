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
    set -x
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

# update per-file hash listing
echo "Updating files, expecting ${logs_wc} hashes:" >&2
echo "    ${HASHES_STDOUT}" >&2
echo "    ${HASHES_STDERR}" >&2
echo -n > "${HASHES_STDOUT}"
echo -n > "${HASHES_STDERR}"
declare -i hash_count=0
tmp1=$(mktemp -t "tmp.s4.compare-current-and-expected_XXXXX")
tmp2=$(mktemp -t "tmp.s4.compare-current-and-expected_XXXXX")
while read -r log_file; do
    if [[ "${log_file}" = '' ]]; then
        continue
    fi
    # run s4 and save stdout, stderr
    (
        "${PROGRAM}" "${S4_ARGS[@]}" "${log_file}" 1>"${tmp1}" 2>"${tmp2}"
    ) || true
    # store hash stdout
    hash=$(cat "${tmp1}" | md5sum_clean)
    echo "${log_file}|${hash}" >> "${HASHES_STDOUT}"
    # store hash stderr
    hash=$(cat "${tmp2}" | stderr_clean_1 | md5sum_clean)
    echo "${log_file}|${hash}" >> "${HASHES_STDERR}"
    hash_count+=1
    echo -n '.' >&2
done < "${LOGS}"
rm -f "${tmp1}" "${tmp2}"
echo >&2
echo "Updated ${hash_count} hashes in files:" >&2
echo "    '${HASHES_STDOUT}'" >&2
echo "    '${HASHES_STDERR}'" >&2

echo >&2

echo -e "Now run \e[1mcompare-current-and-expected/compare.sh\e[0m." >&2
