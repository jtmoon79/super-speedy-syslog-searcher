#!/usr/bin/env bash
#
# compare.sh
#
# compare run output of the current build of super_speedy_syslog_searcher to
# an expected output of a known good build of super_speedy_syslog_searcher
# (presumably git committed). This is a simple end-to-end test.
# The success of this tests depends upon
# - the `./logs` directory being clean
# - the `./logs` files being updated with `log-files-time-update.sh`
# - git commits affecting `./logs` being reflected in files
#   `expected.stdout` `compare-current-and-expected_expected.stderr`
#
# comparison for stdout and stderr are done separately. This is because
# stderr output must be edited after the run as it changes on each run
# (it prints the current datetime and uses local system).

set -euo pipefail

if [[ ${#} -ne 0 ]]; then
    echo "ERROR This script takes no arguments"
    exit 1
fi

cd "$(dirname "${0}")/../.."
HERE="./tools/compare-current-and-expected"
source "${HERE}/common.sh"

if ! truncate -s 0 "${CURRENT_OUT}"; then
    echo "ERROR unable to write to file '${CURRENT_OUT}'" >&2
    exit 1
fi
if ! truncate -s 0 "${CURRENT_ERR}"; then
    echo "ERROR unable to write to file '${CURRENT_ERR}'" >&2
    exit 1
fi

if [[ ! -e "${EXPECT_OUT}" ]]; then
    echo "ERROR can not find file '${EXPECT_OUT}'" >&2
    exit 1
elif [[ ! -r "${EXPECT_OUT}" ]]; then
    echo "ERROR can not read file '${EXPECT_OUT}'" >&2
    exit 1
fi

if [[ ! -e "${EXPECT_ERR}" ]]; then
    echo "ERROR can not find file '${EXPECT_ERR}'" >&2
    exit 1
elif [[ ! -r "${EXPECT_ERR}" ]]; then
    echo "ERROR can not read file '${EXPECT_ERR}'" >&2
    exit 1
fi

#
# get list of files to process
#

logs='./tools/compare-current-and-expected/logs.txt'

if [[ ! -e "${LOGS}" ]]; then
    echo "ERROR file does not exist '${LOGS}'" >&2
    exit 1
elif [[ ! -r "${LOGS}" ]]; then
    echo "ERROR file is not readable '${LOGS}'" >&2
    exit 1
fi

#
# print some info for the script user, verify the s4 program can run
#

cat "${LOGS}" >&2
echo >&2
echo "$(wc -l < "${LOGS}") files in \"${LOGS}\"" >&2
echo >&2

# verify s4 can run
(set -x; "${PROGRAM}" --version)
echo >&2

#
# run s4 program
#

echo "${PS4}${PROGRAM} ${S4_ARGS_QUOTED}- < '${LOGS}'" >&2
(
    set +e
    set +o pipefail
    set -x
    "${PROGRAM}" "${S4_ARGS[@]}" - < "${LOGS}"
) 1> "${CURRENT_OUT}" 2> "${CURRENT_ERR}" || true

stderr_clean "${CURRENT_ERR}"

#
# compare the program outputs
#

# current s4 stdout line count, byte count; only informative
echo
s4out_lc=$(wc -l < "${CURRENT_OUT}")
s4out_bc=$(wc -c < "${CURRENT_OUT}")
echo "current stdout output in file '$(basename "${CURRENT_OUT}")'"
echo "  Line Count ${s4out_lc}"
echo "  Byte Count ${s4out_bc}"

# expected s4 stdout line count, byte count; only informative
echo
exout_lc=$(wc -l < "${EXPECT_OUT}")
exout_bc=$(wc -c < "${EXPECT_OUT}")
echo "expect stdout output in file '$(basename ${EXPECT_OUT})'"
echo "  Line Count ${exout_lc}"
echo "  Byte Count ${exout_bc}"

# current s4 stderr line count, byte count; only informative
echo
s4err_lc=$(wc -l < "${CURRENT_ERR}")
s4err_bc=$(wc -c < "${CURRENT_ERR}")
echo "current stderr output in file '$(basename ${CURRENT_ERR})'"
echo "  Line Count ${s4err_lc}"
echo "  Byte Count ${s4err_bc}"

# expected s4 stderr line count, byte count; only informative
echo
exerr_lc=$(wc -l < "${EXPECT_ERR}")
exerr_bc=$(wc -c < "${EXPECT_ERR}")
echo "expect stderr output in file '$(basename ${EXPECT_ERR})'"
echo "  Line Count ${exerr_lc}"
echo "  Byte Count ${exerr_bc}"

function indent () {
    sed -E -e 's/^/  /'
}

# script return value
declare -i ret=0

function compare_single_file() {
    declare -r log=${1}
}

# compare stdout
if ! diff --text --brief "${CURRENT_OUT}" "${EXPECT_OUT}"; then
    ret=1
    echo "Outputs of stdout are not the same. (ಠ_ಠ)"
    echo
    echo "Difference Preview of stdout:"
    set +o pipefail
    ((set -x; diff --text -y --width=${COLUMNS-120} --suppress-common-lines "${CURRENT_OUT}" "${EXPECT_OUT}") || true) | head -n 200 | indent
    echo
    # be precise about what is missing by checking the hashes listing in
    # per-line format:
    #     log file name|md5sum
    echo "Individual files with differences in stdout:"
    declare -i hash_diff_count=0
    while read -r hash_line; do
        if [[ "${hash_line}" = '' ]]; then
            continue
        fi
        log_file=$(echo -n "${hash_line}" | cut -f1 -d'|')
        expect_hash=$(echo -n "${hash_line}" | cut -f2 -d'|')
        tmpr=$(mktemp -t "tmp.s4.compare-current-and-expected_XXXXX")
        (
            set +e; set +o pipefail;
            "${PROGRAM}" "${S4_ARGS[@]}" "${log_file}" 1>"${tmpr}" 2>/dev/null
        ) || true
        current_hash=$(cat "${tmpr}" | md5sum_clean)
        if [[ "${expect_hash}" != "${current_hash}" ]]; then
            echo "    ${log_file}"
            hash_diff_count+=1
        fi
    done < "${HASHES_STDOUT}"
    if [[ ${hash_diff_count} -gt 0 ]]; then
        echo
        echo "Found ${hash_diff_count} differences in stdout outputs."
        echo "Command:"
        echo "${PROGRAM} ${S4_ARGS_QUOTED}… 2>/dev/null"
    fi
    echo
    echo -e "Do you need to run \e[1mcompare-current-and-expected-update.sh\e[0m ?"
    echo
else
    echo
    echo "Outputs of stdout are the same. (ʘ‿ʘ)"
    echo
fi

# compare stderr
if ! diff --text --brief "${CURRENT_ERR}" "${EXPECT_ERR}"; then
    ret=1
    echo "Outputs of stderr is not the same. (ಠ_ಠ)"
    echo
    echo "Difference Preview of stderr:"
    set +o pipefail
    ((set +e; set -x; diff --text -y --width=${COLUMNS-120} --suppress-common-lines "${CURRENT_ERR}" "${EXPECT_ERR}") || true) | head -n 100 | indent
    echo
    # be precise about what is missing by checking the hashes listing in
    # per-line format:
    #     log file name|md5sum
    echo "Individual files with differences in stderr:"
    tmpr=$(mktemp -t "tmp.s4.compare-current-and-expected_XXXXX")
    declare -i hash_diff_count=0
    while read -r hash_line; do
        if [[ "${hash_line}" = '' ]]; then
            continue
        fi
        log_file=$(echo -n "${hash_line}" | cut -f1 -d'|')
        expect_hash=$(echo -n "${hash_line}" | cut -f2 -d'|')
        (
            "${PROGRAM}" "${S4_ARGS[@]}" "${log_file}" 1>/dev/null 2>"${tmpr}"
        ) || true
        current_hash=$(cat "${tmpr}" | stderr_clean_1 | md5sum_clean)
        if [[ "${expect_hash}" != "${current_hash}" ]]; then
            echo "    ${log_file}"
            hash_diff_count+=1
        fi
    done < "${HASHES_STDERR}"
    rm -f "${tmpr}"
    if [[ ${hash_diff_count} -gt 0 ]]; then
        echo
        echo "Found ${hash_diff_count} differences in stderr outputs."
        echo "Command:"
        echo "${PROGRAM} ${S4_ARGS_QUOTED}… 1>/dev/null"
    fi
    echo
    echo -e "Do you need to run \e[1mcompare-current-and-expected/update.sh\e[0m ?"
    echo
else
    echo
    echo "Outputs of stderr are the same. (ʘ‿ʘ)"
    echo
fi

exit ${ret}
