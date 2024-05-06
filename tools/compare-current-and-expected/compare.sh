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
echo "${LOGS_COUNT} files in \"${LOGS}\"" >&2
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

declare diff_found=false

DIFF=diff
if which colordiff &>/dev/null; then
    DIFF=colordiff
fi

declare -i width=140
if [[ "${COLUMNS+x}" ]]; then
    width=$(($COLUMNS * 2))
fi

total_diff_stdout=true
# compare total stdout
if ! "${DIFF}" --text --brief "${CURRENT_OUT}" "${EXPECT_OUT}"; then
    ret=1
    echo "Output of stdout are not the same. (ಠ_ಠ)"
    echo
    echo "Difference Preview of stdout:"
    set +o pipefail
    ((set -x; "${DIFF}" --text -y --width=${width} --suppress-common-lines "${CURRENT_OUT}" "${EXPECT_OUT}") || true) | head -n 200 | indent
    echo
    echo
    echo -e "Do you need to run \e[1mcompare-current-and-expected-update.sh\e[0m ?"
    echo
    total_diff_stdout=false
else
    echo
    echo "Output of stdout are the same. (ʘ‿ʘ)"
    echo
fi

# compare total stderr
total_diff_stderr=true
if ! "${DIFF}" --text --brief "${CURRENT_ERR}" "${EXPECT_ERR}"; then
    ret=1
    echo "Output of stderr is not the same. (ಠ_ಠ)"
    echo
    echo "Difference Preview of stderr:"
    set +o pipefail
    ((set +e; set -x;
        "${DIFF}" --text -y --width=${width} --suppress-common-lines "${CURRENT_ERR}" "${EXPECT_ERR}") || true
    ) | head -n 100 | indent
    echo
    total_diff_stderr=false
else
    echo
    echo "Output of stderr are the same. (ʘ‿ʘ)"
    echo
fi

echo "Comparing ${LOGS_COUNT} individual files:"

# compare individual files
tmp1=$(mktemp -t "tmp.s4.compare-current-and-expected_stdout_XXXXX")
tmp2=$(mktemp -t "tmp.s4.compare-current-and-expected_stderr_XXXXX")
declare -i diff_log_stdout=0
declare -a diff_file_stdout=0
declare -i same_log_stdout=0
declare -i diff_log_stderr=0
declare -a diff_file_stderr=0
declare -i same_log_stderr=0
while read -r log_file; do
    if [[ "${log_file}" = '' ]] || [[ "${log_file:0:1}" = '#' ]]; then
        continue
    fi
    log_file_stdout="${HERE}/${log_file}.stdout"
    log_file_stderr="${HERE}/${log_file}.stderr"
    (
        set +e
        set +o pipefail
        "${PROGRAM}" "${S4_ARGS[@]}" "${log_file}" 1>"${tmp1}" 2>"${tmp2}"
    ) || true
    stderr_clean "${tmp2}"

    # compare stdout per file
    if ! "${DIFF}" --text --brief "${log_file_stdout}" "${tmp1}" &>/dev/null; then
        diff_log_stdout+=1
        diff_file_stdout+=("${log_file}")
        ret=1
        echo >&2
        echo "    Different stdout ${log_file_stdout}" >&2
        (
            (set -x +e;
                "${DIFF}" --text -y --width=${width} --suppress-common-lines "${log_file_stdout}" "${tmp1}"
            ) || true
        ) | head -n 20 | indent
        echo >&2
        tmp1=$(mktemp -t "tmp.s4.compare-current-and-expected_XXXXX")
    else
        same_log_stdout+=1
        echo -n '.' >&2
    fi
    # compare stderr per file
    if ! "${DIFF}" --text --brief "${log_file_stderr}" "${tmp2}" &>/dev/null; then
        diff_log_stderr+=1
        diff_file_stderr+=("${log_file}")
        ret=1
        echo >&2
        echo "    Different stderr ${log_file_stderr}" >&2
        (
            (set -x +e;
                "${DIFF}" --text -y --width=${width} --suppress-common-lines "${log_file_stderr}" "${tmp2}"
            ) || true
        ) | head -n 20 | indent
        echo >&2
        tmp2=$(mktemp -t "tmp.s4.compare-current-and-expected_XXXXX")
    else
        same_log_stderr+=1
        echo -n '.' >&2
    fi
done < "${LOGS}"

echo
echo "Outputs of ${same_log_stdout} individual stdout comparisons are the same. (ʘ‿ʘ)"
if [[ ${diff_log_stdout} -gt 0 ]]; then
    echo "Outputs of ${diff_log_stdout} individual stdout comparisons were not the same. (ಠ_ಠ)"
    for log in "${diff_file_stdout[@]}"; do
        echo "    ${log}"
    done
fi
echo "Outputs of ${same_log_stderr} individual stderr comparisons are the same. (ʘ‿ʘ)"
if [[ ${diff_log_stderr} -gt 0 ]]; then
    echo "Outputs of ${diff_log_stderr} individual stderr comparisons were not the same. (ಠ_ಠ)"
    for log in "${diff_file_stderr[@]}"; do
        echo "    ${log}"
    done
fi
if ${total_diff_stdout}; then
    echo "Total stdout outputs are the same. (ʘ‿ʘ)"
else
    echo "Total stdout outputs are not the same. (ಠ_ಠ)"
fi
if ${total_diff_stderr}; then
    echo "Total stderr outputs are the same. (ʘ‿ʘ)"
else
    echo "Total stderr outputs are not the same. (ಠ_ಠ)"
fi
echo

rm -f "${tmp1}" "${tmp2}"

if [[ ${ret} -ne 0 ]]; then
    echo
    echo -e "Do you need to run \e[1m$(dirname "${0}")/update.sh\e[0m ?"
    echo
    echo -e "Remember to run \e[1m$(dirname "${0}"/..)/log-files-time-update.sh\e[0m before that!"
fi

exit ${ret}
