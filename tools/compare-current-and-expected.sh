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
# comparison for stdout and stderr are done separately. This is because
# stderr output must be edited after the run as it changes on each run
# (it prints the current datetime and uses local system).

set -euo pipefail

cd "$(dirname "${0}")/.."

current_out=./tools/compare-current-and-expected_current.stdout
expect_out=./tools/compare-current-and-expected_expected.stdout
current_err=./tools/compare-current-and-expected_current.stderr
expect_err=./tools/compare-current-and-expected_expected.stderr

if ! touch "${current_out}"; then
    echo "ERROR unable to write to file '${current_out}'" >&2
    exit 1
fi
if ! touch "${current_err}"; then
    echo "ERROR unable to write to file '${current_err}'" >&2
    exit 1
fi

if [[ ! -e "${expect_out}" ]]; then
    echo "ERROR can not find file '${expect_out}'" >&2
    exit 1
elif [[ ! -r "${expect_out}" ]]; then
    echo "ERROR can not read file '${expect_out}'" >&2
    exit 1
fi

if [[ ! -e "${expect_err}" ]]; then
    echo "ERROR can not find file '${expect_err}'" >&2
    exit 1
elif [[ ! -r "${expect_err}" ]]; then
    echo "ERROR can not read file '${expect_err}'" >&2
    exit 1
fi

#
# get list of files to process
#

logs='./tools/compare-current-and-expected_logs.txt'

if [[ ! -e "${logs}" ]]; then
    echo "ERROR file does not exist '${logs}'" >&2
    exit 1
elif [[ ! -r "${logs}" ]]; then
    echo "ERROR file is not readable '${logs}'" >&2
    exit 1
fi

#
# print some info for the script user, verify the s4 program can run
#

cat "${logs}" >&2
echo >&2
echo "$(wc -l < "${logs}") files in \"${logs}\"" >&2
echo >&2

PROGRAM=${PROGRAM-./target/release/s4}
# verify s4 can run
(set -x; "${PROGRAM}" --version)
echo >&2

#
# run s4 program
#

declare -ar S4_ARGS=(
    --color=never
    --tz-offset=+08:00
    --prepend-filepath
    --prepend-utc
    --summary
    '-'
    "${@}"
)

echo "${PS4}${PROGRAM} ${S4_ARGS[@]} < ${logs}" >&2
(
    set -x
    "${PROGRAM}" "${S4_ARGS[@]}" < "${logs}"
) 1> "${current_out}" 2> "${current_err}" || true

# XXX: the following `sed` command must match `compare-current-and-expected-update.sh`
# - remove the printing of the current time
# - remove the printing of the datetime first and last. It might use
#   the local system timezone
# - remove warnings as they are printed in unpredictable order
sed -i -E \
    -e '/^Datetime Now[ ]*:.*$/d' \
    -e '/^[ ]*datetime first[ ]*.*$/d' \
    -e '/^[ ]*datetime last[ ]*.*$/d' \
    -e '/^Datetime printed first[ ]*:.*$/d' \
    -e '/^Datetime printed last[ ]*:.*$/d' \
    -e '/^WARNING: no syslines found .*$/d' \
    -- "${current_err}"

#
# compare the program outputs
#

# current s4 stdout line count, byte count; only informative
echo
s4out_lc=$(wc -l < "${current_out}")
s4out_bc=$(wc -c < "${current_out}")
echo "current stdout output in file '$(basename "${current_out}")'"
echo "  Line Count ${s4out_lc}"
echo "  Byte Count ${s4out_bc}"

# expected s4 stdout line count, byte count; only informative
echo
exout_lc=$(wc -l < "${expect_out}")
exout_bc=$(wc -c < "${expect_out}")
echo "expect stdout output in file '$(basename ${expect_out})'"
echo "  Line Count ${exout_lc}"
echo "  Byte Count ${exout_bc}"

# current s4 stderr line count, byte count; only informative
echo
s4err_lc=$(wc -l < "${current_err}")
s4err_bc=$(wc -c < "${current_err}")
echo "current stderr output in file '$(basename ${current_err})'"
echo "  Line Count ${s4err_lc}"
echo "  Byte Count ${s4err_bc}"

# expected s4 stderr line count, byte count; only informative
echo
exerr_lc=$(wc -l < "${expect_err}")
exerr_bc=$(wc -c < "${expect_err}")
echo "expect stderr output in file '$(basename ${expect_err})'"
echo "  Line Count ${exerr_lc}"
echo "  Byte Count ${exerr_bc}"

function indent () {
    sed -E -e 's/^/  /'
}

# script return value
declare -i ret=0

# compare stdout
if ! diff --text --brief "${current_out}" "${expect_out}"; then
    ret=1
    echo "Outputs of stdout are not the same. (ಠ_ಠ)"
    echo
    echo "Difference Preview:"
    set +o pipefail
    ((set -x; diff --text -y --width=${COLUMNS-120} --suppress-common-lines "${current_out}" "${expect_out}") || true) | head -n 200 | indent
    echo
    #echo "stderr output:"
    #echo
    #cat "${tmpc_out}" | indent
    echo
    echo -e "Do you need to run \e[1mcompare-current-and-expected-update.sh\e[0m ?"
    echo
else
    echo
    echo "Outputs of stdout are the same. (ʘ‿ʘ)"
    echo
fi

# compare stderr
if ! diff --text --brief "${current_err}" "${expect_err}"; then
    ret=1
    echo "Outputs of stderr is not the same. (ಠ_ಠ)"
    echo
    echo "Difference Preview:"
    set +o pipefail
    ((set -x; diff --text -y --width=${COLUMNS-120} --suppress-common-lines "${current_err}" "${expect_err}") || true) | head -n 200 | indent
    echo
    #echo "stderr output:"
    #echo
    #cat "${tmpc_err}" | indent
    echo
    echo -e "Do you need to run \e[1mcompare-current-and-expected-update.sh\e[0m ?"
    echo
else
    echo
    echo "Outputs of stderr are the same. (ʘ‿ʘ)"
    echo
fi

exit ${ret}
