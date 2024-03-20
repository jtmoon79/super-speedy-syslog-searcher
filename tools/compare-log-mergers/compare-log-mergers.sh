#!/usr/bin/env bash
#
# hardcoded time comparison of GNU grep + sort, `s4`, `logmerger`

set -eu

if [[ ${#} -ne 0 ]]; then
    echo "Usage: ${0}" >&2
    exit 1
fi

cd "$(dirname "${0}")/../.."

# use full path to Unix tools
time=$(which time)
(set -x; $time --version) | head -n1

PYTHON=${PYTHON-$(
    if which -a python &>/dev/null; then
        echo -n 'python'
    else
        echo -n 'python3'
    fi
)}
(set -x; "${PYTHON}" --version) | head -n1

declare -a files=(
    './tools/compare-log-mergers/gen-5000-1-faces.log'
    './tools/compare-log-mergers/gen-2500-1-faces.log'
    './tools/compare-log-mergers/gen-2000-1-faces.log'
)

tmp1=$(mktemp -t "compare-log-mergers_XXXXX.out")

function exit_() {
    rm -f "${tmp1}"
}

trap exit_ EXIT

# datetime range for s4
declare -r after_dt="2000-01-01T00:20:00"
declare -r befor_dt="2000-01-01T00:50:00"
# datetime range for logdissect
declare -r after_dt_ld="200001010020"
declare -r befor_dt_ld="200001010050"
# datetime range for GNU grep + sort
# grep regex equivalent of $after_dt $befor_dt
declare -r regex_dt='^2000-01-01T00\:([234][[:digit:]]\:[[:digit:]]{2}|50\:00)'

function files_caching() {
    # force reading of files from disk to allow any possible caching.
    # crude but possibly better than nothing
    cat "${files[@]}" > "${tmp1}"
}

function echo_line() {
    echo '----------------------------------------'
}

# GNU grep + sort

echo_line

grep=$(which grep)
(set -x; $grep --version) | head -n1
sort=$(which sort)
(set -x; $sort --version) | head -n1

echo

(
    files_caching
    # search for datetimes between $after_dt $befor_dt
    # using decently constrained regexp to match meaning
    set -x
    $time -p -- \
    bash -c "\
$grep -hEe '${regex_dt}' -- \
${files[*]} \
| $sort -t ' ' -k 1 -s \
> '${tmp1}'"
)
echo
wc -l "${tmp1}"

# Super Speedy Syslog Searcher (S4)

echo_line

PROGRAM_S4=${PROGRAM_S4-./target/release/s4}
(set -x; "${PROGRAM_S4}" --version)

echo

(
    files_caching
    set -x
    $time -p -- \
        "${PROGRAM_S4}" \
        -a "${after_dt}" \
        -b "${befor_dt}" \
        --color=never \
        "${files[@]}" \
        > "${tmp1}"
)
echo
wc -l "${tmp1}"

# logmerger

echo_line

PROGRAM_LM=${PROGRAM_LM-logmerger}
# XXX: logmerger does not have a `--version` option
echo "${PS4}logmerger --version"
"${PYTHON}" -m pip list | grep -Fe 'logmerger'

echo

(
    files_caching
    set -x
    $time -p -- \
        "${PROGRAM_LM}" \
        --inline \
        --output=- \
        --start "${after_dt}" \
        --end "${befor_dt}" \
        "${files[@]}" \
        > "${tmp1}"
)
echo
wc -l "${tmp1}"

# logdissect

echo_line

PROGRAM_LD=${PROGRAM_LD-logdissect}
(set -x; "${PROGRAM_LD}" --version)

echo

(
    echo "TODO: figure out how to use logdissect. I'm unable to get it to match on ANY files."
    exit 0
    files_caching
    set -x
    $time -p -- \
        "${PROGRAM_LD}" \
        --range "${after_dt_ld}-${befor_dt_ld}" \
        "${files[@]}" \
        > "${tmp1}"
)
echo
# wc -l "${tmp1}"

# TooLong

echo_line

# TODO: how to force `tl` to not create a TUI window?
PROGRAM_TL=${PROGRAM_TL-tl}
(set -x; "${PROGRAM_TL}" --version)

echo

(
    files_caching
    set -x
    $time -p -- \
        "${PROGRAM_TL}" \
        --merge \
        --output-merge="${tmp1}" \
        "${files[@]}" \
)
echo
wc -l "${tmp1}"
