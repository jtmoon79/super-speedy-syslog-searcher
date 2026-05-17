#!/usr/bin/env bash
#
# compare-two-s4.sh
#
# compare run time and output of two s4 builds. ignore stderr.
# a simple, useful end-to-end test of differences and assertions.
# useful for comparing debug and release builds which should not have different outputs.
# (debug builds have more asserts and self-checks)
#
# usage: compare-two-s4.sh [s4-args-for-both-builds]
#
# environment variables:
#   PROGRAM_A: path to the first s4 build to compare; default ./target/release/s4
#   PROGRAM_B: path to the second s4 build to compare; default ./target/debug/s4
#   LOGS_LISTING: path to the file listing logs to process, one per line; default ./tools/compare-debug-release_logs.txt
#   RUST_MIN_STACK: if set, will be exported to the s4 runs
#

set -euo pipefail

cd "$(dirname "${0}")/.."

if which colordiff &>/dev/null; then
    DIFF=colordiff
else
    DIFF=diff
fi
if [[ "${DIFF+x}" ]]; then
    DIFF=$DIFF
fi
readonly DIFF

readonly C_ERR=$'\e[31m'  # red
readonly C_OK=$'\e[32m'   # green
readonly C_OFF=$'\e[0m'  # default

# logs to process listed one per line
declare -r LOGS_LISTING=${LOGS_LISTING:-./tools/compare-debug-release_logs.txt}

if [[ ! -f "${LOGS_LISTING}" ]]; then
    echo "ERROR Logs listing file '${LOGS_LISTING}' does not exist." >&2
    echo "      realpath '$(realpath -m "${LOGS_LISTING}")'" >&2
    exit 1
fi

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
done <<< "$(cat "${LOGS_LISTING}")"
echo >&2
echo "Comparing results of processing ${#logs[@]} files from \"${LOGS_LISTING}\"" >&2
echo >&2

(set -x; "${DIFF}" --version) | head -n1
echo >&2

PROGRAM_A=${PROGRAM_A-./target/release/s4}
(set -x; "${PROGRAM_A}" --version 2>/dev/null)
readonly PROGRAM_A

PROGRAM_B=${PROGRAM_B-./target/debug/s4}
(set -x; "${PROGRAM_B}" --version 2>/dev/null)
echo >&2
readonly PROGRAM_B

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
    --dt-before='20260210T221032+0000'
    --summary
    "${@}"
)

# output of the A run
tmpA=$(mktemp -t "s4-tmp.compare-two-s4_A_XXXXX")
# output of the B run
tmpB=$(mktemp -t "s4-tmp.compare-two-s4_B_XXXXX")

declare -i rcA=0
declare -i rcB=0

if [[ "${RUST_MIN_STACK+x}" ]]; then
    export RUST_MIN_STACK=$RUST_MIN_STACK
fi

declare -a logs_eq=()
declare -a logs_ne=()

for log in "${logs[@]}"; do
    declare okay=true
    echo "${log}" >&2
    set +e
    # run the A build
    (set -x; "${PROGRAM_A}" "${S4_ARGS[@]}" "${log}" 2>/dev/null > "${tmpA}")
    rcA=$?
    # run the B build
    (set -x; "${PROGRAM_B}" "${S4_ARGS[@]}" "${log}" 2>/dev/null > "${tmpB}")
    rcB=$?
    set -e

    if [[ ${rcA} -ne ${rcB} ]]; then
        echo "${C_ERR}Return codes differ A=${rcA} B=${rcB} for log '${log}'${C_OFF}" >&2
        ret=1
        okay=false
    fi
    if ! "${DIFF}" --text --brief "${tmpA}" "${tmpB}"; then
        echo "${C_ERR}Files are not the same for log '${log}'.${C_OFF}" >&2
        ret=1
        okay=false
        logs_ne+=("${log}")
        #"${DIFF}" --text -y --width=${COLUMNS-120} "${tmpA}" "${tmpB}"
    fi
    if $okay; then
        echo "${C_OK}Files are the same for log '${log}'.${C_OFF}" >&2
        logs_eq+=("${log}")
    fi
    echo >&2
done

echo >&2

set +e

# run the A build
time (
    set -x
    "${PROGRAM_A}" "${S4_ARGS[@]}" "${logs[@]}" 2>/dev/null > "${tmpA}"
)
rcA=$?

echo >&2

# run the B build
time (
    set -x
    "${PROGRAM_B}" "${S4_ARGS[@]}" "${logs[@]}" 2>/dev/null > "${tmpB}"
)
rcB=$?

set -e

#
# compare the program A and program B outputs
#

# s4 A line count, byte count; only informative
echo
s4r_lc=$(wc -l < "${tmpA}")
s4r_bc=$(wc -c < "${tmpA}")
echo "super_speedy_syslog_searcher output A '${tmpA}'"
echo "  Line Count ${s4r_lc}"
echo "  Byte Count ${s4r_bc}"
echo "  Return Code ${rcA}"

# s4 B line count, byte count; only informative
s4d_lc=$(wc -l < "${tmpB}")
s4d_bc=$(wc -c < "${tmpB}")
echo "super_speedy_syslog_searcher output B '${tmpB}'"
echo "  Line Count ${s4d_lc}"
echo "  Byte Count ${s4d_bc}"
echo "  Return Code ${rcB}"

if ! "${DIFF}" --text --brief "${tmpA}" "${tmpB}"; then
    ret=1
    echo "${C_ERR}Files are not the same. (ಠ_ಠ)${C_OFF}"
    echo
    echo "Difference Preview:"
    (
        (
            set -x;
            "${DIFF}" --text -y --width=${COLUMNS-120} --suppress-common-lines "${tmpA}" "${tmpB}"
        ) || true
    ) | head -n 20
    echo
else
    echo
    echo "${C_OK}Files are the same. (ʘ‿ʘ)${C_OFF}"
    rm -f "${tmpA}" "${tmpB}"
    echo
fi

if [[ ${#logs_eq[@]} -ne 0 ]]; then
    echo "${C_OK}Files match for ${#logs_eq[@]} logs.${C_OFF}"
    for log_ in "${logs_eq[@]}"; do
        echo "${log_}"
    done
    echo
fi

if [[ ${#logs_ne[@]} -ne 0 ]]; then
    echo "${C_ERR}Files differ for ${#logs_ne[@]} logs.${C_OFF}"
    for log_ in "${logs_ne[@]}"; do
        echo "${log_}"
    done
    echo
fi

echo "${PROGRAM_A}" "${S4_ARGS[@]}"
echo "${PROGRAM_B}" "${S4_ARGS[@]}"

exit ${ret}
