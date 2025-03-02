#!/usr/bin/env bash
#
# hardcoded time comparison of GNU grep + sort, `s4`, `logmerger`
#
# pass `--skip-tl` to skip processing toolong which takes over the console
# window and stalls non-interactive consoles
#

set -eu

if [[ ${#} -gt 1 ]]; then
    echo "Usage: ${0} [--skip-tl]" >&2
    exit 1
fi

if ! [[ "${VIRTUAL_ENV-}" ]]; then
    echo "ERROR: must run within a Python virtual environment" >&2
    exit 1
fi

skip_tl=false
if [[ ${#} -ge 1 ]]; then
    if [[ "${1-}" = '--skip-tl' ]]; then
        skip_tl=true
        shift
    fi
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

if ! which bc &>/dev/null; then
    echo "ERROR: bc not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install bc" >&2
    exit 1
fi

if ! which jq &>/dev/null; then
    echo "ERROR: jq not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install jq" >&2
    exit 1
fi
JQ=$(which jq)

HRUNS=30

# precompile all python packages
PYSITE_PKG_PATH=$("${PYTHON}" -c "import sysconfig; print(sysconfig.get_path('purelib'))")
(
    set -x
    "${PYTHON}" -m compileall -q "${PYSITE_PKG_PATH}"
)

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
            --quiet \
            "${package}"
    )
done

declare -a files=(
    './tools/compare-log-mergers/gen-5000-1-facesA.log'
    './tools/compare-log-mergers/gen-5000-1-facesB.log'
    './tools/compare-log-mergers/gen-5000-1-facesC.log'
)

tmpA=$(mktemp -t "compare-log-mergers_XXXXX.out")
json1=$(mktemp -t "compare-log_mergers_XXXXX.json")
json2=$(mktemp -t "compare-log_mergers_XXXXX.json")
json3=$(mktemp -t "compare-log_mergers_XXXXX.json")
json4=$(mktemp -t "compare-log_mergers_XXXXX.json")
json5=$(mktemp -t "compare-log_mergers_XXXXX.json")
json6=$(mktemp -t "compare-log_mergers_XXXXX.json")
json7=$(mktemp -t "compare-log_mergers_XXXXX.json")
tm1=$(mktemp -t "compare-log_mergers_XXXXX.txt")
tm2=$(mktemp -t "compare-log_mergers_XXXXX.txt")
tm3=$(mktemp -t "compare-log_mergers_XXXXX.txt")
tm4=$(mktemp -t "compare-log_mergers_XXXXX.txt")
tm5=$(mktemp -t "compare-log_mergers_XXXXX.txt")
tm6=$(mktemp -t "compare-log_mergers_XXXXX.txt")
tm7=$(mktemp -t "compare-log_mergers_XXXXX.txt")
mdfinal=$(mktemp -t "compare-log_mergers_final_XXXXX.md")

function exit_() {
    rm -f \
        "${tmpA}" \
        "${json1}" "${json2}" "${json3}" "${json4}" "${json5}" "${json6}" "${json7}" \
        "${tm1}" "${tm2}" "${tm3}" "${tm4}" "${tm5}" "${tm6}" "${tm7}" \
        "${mdfinal}"
}
trap exit_ EXIT

# datetime range for s4, lnav
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
    cat "${files[@]}" > "${tmpA}"
}

function echo_line() {
    python -Bc "print('─' * ${COLUMNS:-100})"
    echo
}

function file_size() {
    stat --printf='%s' "${1}"
}

function file_isempty() {
    [[ $(file_size "${1}") -eq 0 ]]
}

# %M = Maximum resident set size in KB
# %P = CPU percentage
# %E = Elapsed real time
# see https://www.man7.org/linux/man-pages/man1/time.1.html
# Note: metrics %t %K and other memory metrics always returned 0
TIME_FORMAT='%M|%P|%E'

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
    $hyperfine --style=basic --runs=${HRUNS} --export-json "${json1}" --shell sh -n "grep+sort" \
        -- \
        "$grep -hEe '${regex_dt}' -- ${files[*]} | $sort -t ' ' -k 1 -s > /dev/null"
)
(
    files_caching
    set -x
    $time --format="${TIME_FORMAT}" --output="${tm1}" \
        -- \
        sh -c "'$grep' -hEe '${regex_dt}' -- ${files[*]} | '$sort' -t ' ' -k 1 -s" > "${tmpA}"
)

echo
cat "${tmpA}" | wc -l -
echo

# Super Speedy Syslog Searcher (S4) (system)

echo_line

PROGRAM_S4_SYSTEM=${PROGRAM_S4_SYSTEM-./target/release/s4}
(set -x; "${PROGRAM_S4_SYSTEM}" --version)

echo

(
    files_caching
    set -x
    $hyperfine --style=basic --runs=${HRUNS} --export-json "${json2}" -N -n "s4 (system)" \
        -- \
        "'${PROGRAM_S4_SYSTEM}' -a='${after_dt}' -b='${befor_dt}' --color=never ${files[*]} > /dev/null"
)
(
    files_caching
    set -x
    $time --format="${TIME_FORMAT}" --output="${tm2}" \
        -- \
        "${PROGRAM_S4_SYSTEM}" \
        "-a=${after_dt}" \
        "-b=${befor_dt}" \
        "--color=never" \
        "${files[@]}" > "${tmpA}"
)

echo
cat "${tmpA}" | wc -l -
echo

# Super Speedy Syslog Searcher (S4) (jemalloc)

echo_line

PROGRAM_S4_JEMALLOC=${PROGRAM_S4_JEMALLOC-./target/jemalloc/s4}
(set -x; "${PROGRAM_S4_JEMALLOC}" --version)

(
    files_caching
    set -x
    $hyperfine --style=basic --runs=${HRUNS} --export-json "${json3}" -N -n "s4 (jemalloc)" \
        -- \
        "'${PROGRAM_S4_JEMALLOC}' -a='${after_dt}' -b='${befor_dt}' --color=never ${files[*]} > /dev/null"
)

(
    files_caching
    set -x
    $time --format="${TIME_FORMAT}" --output="${tm3}" \
        -- \
        "${PROGRAM_S4_JEMALLOC}" \
        "-a=${after_dt}" \
        "-b=${befor_dt}" \
        "--color=never" \
        "${files[@]}" > "${tmpA}"
)

echo
cat "${tmpA}" | wc -l -
echo

# Super Speedy Syslog Searcher (S4) (mimalloc)

PROGRAM_S4_MIMALLOC=${PROGRAM_S4_MIMALLOC-./target/mimalloc/s4}
(set -x; "${PROGRAM_S4_MIMALLOC}" --version)

echo_line

(
    files_caching
    set -x
    $hyperfine --style=basic --runs=${HRUNS} --export-json "${json4}" -N -n "s4 (mimalloc)" \
        -- \
        "'${PROGRAM_S4_MIMALLOC}' -a='${after_dt}' -b='${befor_dt}' --color=never ${files[*]} > /dev/null"
)
(
    files_caching
    set -x
    $time --format="${TIME_FORMAT}" --output="${tm4}" \
        -- \
        "${PROGRAM_S4_MIMALLOC}" \
        "-a=${after_dt}" \
        "-b=${befor_dt}" \
        "--color=never" \
        "${files[@]}" > "${tmpA}"
)

echo
cat "${tmpA}" | wc -l -
echo

# lnav

PROGRAM_LNAV=${PROGRAM_LNAV-lnav}
(
    files_caching
    set -x
    lnav --version
    lnav -i -W ./tools/compare-log-mergers/lnav1.json
    $hyperfine --style=basic --runs=${HRUNS} --export-json "${json5}" -N -n "${PROGRAM_LNAV}" \
        -- \
        "'${PROGRAM_LNAV}' -N -n \
-c ';SELECT log_raw_text FROM lnav1 WHERE log_time BETWEEN Datetime(\"${after_dt}\") AND Datetime(\"${befor_dt}\")' \
${files[*]}"
)

(
    files_caching
    set -x
    $time --format="${TIME_FORMAT}" --output="${tm5}" \
        -- \
        "${PROGRAM_LNAV}" -N -n \
            -c ";SELECT log_raw_text FROM lnav1 WHERE log_time BETWEEN Datetime('${after_dt}') AND Datetime('${befor_dt}')" \
            "${files[@]}" > "${tmpA}"
)

echo
cat "${tmpA}" | wc -l -
echo

# logmerger

echo_line

PROGRAM_LM=${PROGRAM_LM-logmerger}
# XXX: logmerger does not have a `--version` option
echo "${PS4}logmerger --version"
"${PYTHON}" -m pip list | grep -Fe 'logmerger'

# precompile logmerger
(
    set -x
    "${PYTHON}" -m compileall "${PYSITE_PKG_PATH}/logmerger"
)

echo

(
    files_caching
    set -x
    $hyperfine --style=basic --runs=${HRUNS} --export-json "${json6}" --shell sh -n "${PROGRAM_LM}" \
        -- \
        "'${PROGRAM_LM}' --inline --output=- --start '${after_dt}' --end '${befor_dt}' ${files[*]} > /dev/null"
)
(
    files_caching
    set -x
    $time --format="${TIME_FORMAT}" --output="${tm6}" \
        -- \
        "${PROGRAM_LM}" \
        "--inline" \
        "--output=-" \
        "--start" \
        "${after_dt}" \
        "--end" \
        "${befor_dt}" \
        "${files[@]}" \
         > "${tmpA}"
)

echo
cat "${tmpA}" | wc -l -
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
    $time --format="${TIME_FORMAT}" --output="${tm}" \
        -- \
        "${PROGRAM_LD}" \
        --range "${after_dt_ld}-${befor_dt_ld}" \
        "${files[@]}" \
        > "${tmpA}"
)
echo
# cat "${tmpA}" | wc -l -

# TooLong

echo_line

# TODO: how to force `tl` to not create a TUI window?
PROGRAM_TL=${PROGRAM_TL-tl}
(set -x; "${PROGRAM_TL}" --version)

(
    # precompile toolong
    set -x
    "${PYTHON}" -m compileall "${PYSITE_PKG_PATH}/toolong"
)

echo

if ! ${skip_tl}; then
    (
        files_caching
        set -x
        $time --format="${TIME_FORMAT}" --output="${tm7}" \
            -- \
            "${PROGRAM_TL}" \
            --merge \
            --output-merge "${tmpA}" \
            "${files[@]}"
    )
    echo
    cat "${tmpA}" | wc -l -
    echo
    cat "${tm7}"
    echo

    erealtime=$(cat "${tm7}" | cut -d'|' -f3 | cut -d':' -f2)
    echo '{
"results": [ {
    "command": "toolong",
    "mean": '"${erealtime}"',
    "stddev": 0.0,
    "min": 0.0,
    "max": 0.0,
    "times": [0.0],
    "exit_codes": [0]
  } ]
}' > "${json7}"
fi

#
# merge separate files into one final markdown file
#
# example json output:
#
# $  hyperfine --show-output --export-json /tmp/out.json -n 'my sleep' --shell sh -- "sleep 0.1"
#
#   {
#     "results": [
#         {
#         "command": "my sleep",
#         "mean": 0.10085313591172414,
#         "stddev": 0.00013263308766873322,
#         "median": 0.10084629836000002,
#         "user": 0.0007963648275862067,
#         "system": 0.00002696551724137931,
#         "min": 0.10058878336000002,
#         "max": 0.10112353836000001,
#         "times": [
#             0.10059074636000001,
#             ...
#         ],
#         "exit_codes": [
#             0,
#             ...
#         ]
#       }
#     ]
#   }
#
# example time output:
#
#   402418|81.2

function to_milliseconds() {
    # from seconds to milliseconds; '0.0034125904' -> '3.0'
    # $1 must be a number
    if [[ 1 -ne ${#} ]]; then
        echo "ERROR: wrong number of arguments" >&2
        exit 1
    fi
    if [[ -z "${1}" ]]; then
        echo "ERROR: empty value" >&2
        exit 1
    fi
    "${PYTHON}" -c "print('%.1f' % (${1} * 1000))"
}

# markdown table header
echo '|Command|Mean (ms)|Min (ms)|Max (ms)|Max RSS (KB)|CPU %|' > "${tmpA}"
echo '|:---|---:|---:|---:|---:|---:|' >> "${tmpA}"

# markdown table rows
for files_ in \
    "${json1}|${tm1}" \
    "${json2}|${tm2}" \
    "${json3}|${tm3}" \
    "${json4}|${tm4}" \
    "${json5}|${tm5}" \
    "${json6}|${tm6}" \
    "${json7}|${tm7}" \
; do
    json=$(echo -n "${files_}" | cut -d'|' -f1)
    tm=$(echo -n "${files_}" | cut -d'|' -f2)
    if file_isempty "${json}"; then
        echo "skip empty JSON file ${json}" >&2
        continue
    fi
    if file_isempty "${tm}"; then
        echo "skip empty file ${tm}" >&2
        continue
    fi
    (
        command=$($JQ '.results[0].command' < "${json}" | tr -d '"')
        mean=$(to_milliseconds $($JQ '.results[0].mean' < "${json}"))
        stddev=$(to_milliseconds $($JQ '.results[0].stddev' < "${json}"))
        min=$(to_milliseconds $($JQ '.results[0].min' < "${json}"))
        max=$(to_milliseconds $($JQ '.results[0].max' < "${json}"))
        mss=$(cat "${tm}" | cut -d'|' -f1)
        cpup=$(cat "${tm}" | cut -d'|' -f2)
        echo "|\`${command}\`|${mean} ± ${stddev}|${min}|${max}|${mss}|${cpup}|"
    ) >> "${tmpA}"
done

cat "${tmpA}" | column -t -s '|' -o '|' > "${mdfinal}"

(set -x; cat "${mdfinal}")

if which glow &>/dev/null; then
    glow "${mdfinal}"
fi
