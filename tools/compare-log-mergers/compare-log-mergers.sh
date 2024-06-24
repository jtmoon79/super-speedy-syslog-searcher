#!/usr/bin/env bash
#
# hardcoded time comparison of GNU grep + sort, `s4`, `logmerger`
#

set -eu

if [[ ${#} -ne 0 ]]; then
    echo "Usage: ${0}" >&2
    exit 1
fi

if ! [[ "${VIRTUAL_ENV-}" ]]; then
    echo "ERROR: must run within a Python virtual environment" >&2
    exit 1
fi

cd "$(dirname "${0}")/../.."

(
    set -x
    git log -n1 --oneline -1
)

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
if which hyperfine &>/dev/null; then
    hyperfine=$(which hyperfine)
    (set -x; hyperfine --version)
fi

if [[ -z "${VIRTUAL_ENV-}" ]]; then
    echo "ERROR: must run within a Python virtual environment" >&2
    exit 1
fi
# make sure packages are installed to expected versions
# XXX: these versions should match that described in the `README.md`
for package in \
    'logmerger==0.9.0' \
    'toolong==1.5.0' \
    'logdissect==3.1.1' \
 ; do
    (
        set -x
        "${PYTHON}" -m pip install \
            --upgrade \
            --no-python-version-warning --disable-pip-version-check \
            --force --quiet \
            "${package}"
    )
done

declare -a files=(
    './tools/compare-log-mergers/gen-5000-1-faces.log'
    './tools/compare-log-mergers/gen-2500-1-faces.log'
    './tools/compare-log-mergers/gen-2000-1-faces.log'
)

tmp1=$(mktemp -t "compare-log-mergers_XXXXX.out")
md1=$(mktemp -t "compare-log_mergers_XXXXX.md")
md2=$(mktemp -t "compare-log_mergers_XXXXX.md")
md3=$(mktemp -t "compare-log_mergers_XXXXX.md")
md4=$(mktemp -t "compare-log_mergers_XXXXX.md")
md5=$(mktemp -t "compare-log_mergers_XXXXX.md")
mdfinal=${DIROUT-.}/compare-log_mergers.md

function exit_() {
    rm -f "${tmp1}" "${md1}" "${md2}" "${md3}" "${md4}" "${md5}"
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

TIME_FORMAT='real %e s, Max RSS %M KB, %P %%CPU, (%x)'

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
    $hyperfine --style=basic --export-markdown "${md1}" --shell sh -n "grep+sort" \
        -- \
        "$grep -hEe '${regex_dt}' -- ${files[*]} | $sort -t ' ' -k 1 -s > /dev/null"
)
(
    files_caching
    set -x
    $time --format="${TIME_FORMAT}" \
        "${@}" \
        -- \
        "$grep" -hEe "${regex_dt}" -- "${files[@]}" \
            | "$sort" -t ' ' -k 1 -s \
            > "${tmp1}"
)
echo
wc -l "${tmp1}"
echo
cat "${md1}"
echo

# Super Speedy Syslog Searcher (S4) (system)

echo_line

PROGRAM_S4=${PROGRAM_S4-./target/release/s4}
(set -x; "${PROGRAM_S4}" --version)

echo

(
    (set -x; cargo clean --quiet; cargo build --quiet --release)
    files_caching
    set -x
    $hyperfine --style=basic --export-markdown "${md2}" -N -n "s4 (system)" \
        -- \
        "'${PROGRAM_S4}' -a='${after_dt}' -b='${befor_dt}' --color=never ${files[*]} > /dev/null"
)
(
    files_caching
    set -x
    $time --format="${TIME_FORMAT}" \
        "${@}" \
        -- \
        "${PROGRAM_S4}" \
        "-a=${after_dt}" \
        "-b=${befor_dt}" \
        "--color=never" \
        "${files[@]}" > "${tmp1}"
)
echo
wc -l "${tmp1}"
echo
cat "${md2}"
echo

# Super Speedy Syslog Searcher (S4) (mimalloc)

echo_line

(
    (set -x; cargo clean --quiet; cargo build --quiet --release --features=mimalloc)
    files_caching
    set -x
    $hyperfine --style=basic --export-markdown "${md3}" -N -n "s4 (mimalloc)" \
        -- \
        "'${PROGRAM_S4}' -a='${after_dt}' -b='${befor_dt}' --color=never ${files[*]} > /dev/null"
)
(
    files_caching
    set -x
    $time --format="${TIME_FORMAT}" \
        "${@}" \
        -- \
        "${PROGRAM_S4}" \
        "-a=${after_dt}" \
        "-b=${befor_dt}" \
        "--color=never" \
        "${files[@]}" > "${tmp1}"
)
echo
wc -l "${tmp1}"
echo
cat "${md3}"
echo

# Super Speedy Syslog Searcher (S4) (jemalloc)

echo_line

(
    (set -x; cargo clean --quiet; cargo build --quiet --release --features=jemalloc)
    files_caching
    set -x
    $hyperfine --style=basic --export-markdown "${md4}" -N -n "s4 (jemalloc))" \
        -- \
        "'${PROGRAM_S4}' -a='${after_dt}' -b='${befor_dt}' --color=never ${files[*]} > /dev/null"
)
(
    files_caching
    set -x
    $time --format="${TIME_FORMAT}" \
        "${@}" \
        -- \
        "${PROGRAM_S4}" \
        "-a=${after_dt}" \
        "-b=${befor_dt}" \
        "--color=never" \
        "${files[@]}" > "${tmp1}"
)
echo
wc -l "${tmp1}"
echo
cat "${md4}"
echo

# logmerger

echo_line

PROGRAM_LM=${PROGRAM_LM-logmerger}
# XXX: logmerger does not have a `--version` option
echo "${PS4}logmerger --version"
"${PYTHON}" -m pip list | grep -Fe 'logmerger'

# TODO: precompile logmerger

echo

(
    files_caching
    set -x
    $hyperfine --style=basic --export-markdown "${md5}" --shell sh -n "${PROGRAM_LM}" \
        -- \
        "'${PROGRAM_LM}' --inline --output=- --start '${after_dt}' --end '${befor_dt}' ${files[*]} > /dev/null"
)
(
    files_caching
    set -x
    $time --format="${TIME_FORMAT}" \
        "${@}" \
        -- \
        "${PROGRAM_LM}" \
        "--inline" \
        "--output=-" \
        "--start" \
        "${after_dt}" \
        "--end" \
        "${befor_dt}" \
        "${files[@]}" \
         > "${tmp1}"
)
echo
wc -l "${tmp1}"
echo
cat "${md5}"
echo

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
    $time --format="${TIME_FORMAT}" \
        "${@}" \
        -- \
        "${PROGRAM_LD}" \
        --range "${after_dt_ld}-${befor_dt_ld}" \
        "${files[@]}" \
        > "${tmp1}"
)
echo
# wc -l "${tmp1}"

# TooLong

# TODO: precompile TooLong

echo_line

# TODO: how to force `tl` to not create a TUI window?
PROGRAM_TL=${PROGRAM_TL-tl}
(set -x; "${PROGRAM_TL}" --version)

echo

(
    files_caching
    set -x
    $time --format="${TIME_FORMAT}" \
        "${@}" \
        -- \
        "${PROGRAM_TL}" \
        --merge \
        --output-merge "${tmp1}" \
        "${files[@]}" \
)
echo
wc -l "${tmp1}"
echo

# create the final markdown file

(
    cat "${md1}"
    cat "${md2}" | tail -n +3
    cat "${md3}" | tail -n +3
    cat "${md4}" | tail -n +3
    cat "${md5}" | tail -n +3
) | column -t -s '|' -o '|' > "${mdfinal}"

(set -x; cat "${mdfinal}")

if which glow &>/dev/null && [[ -r "${mdfinal}" ]]; then
    glow "${mdfinal}"
fi
