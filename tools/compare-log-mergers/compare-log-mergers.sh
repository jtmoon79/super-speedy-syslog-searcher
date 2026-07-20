#!/usr/bin/env bash
#
# hardcoded performance comparison of GNU `grep | sort`, `s4` (all allocator
# variations), `logmerger`, `lnav`, `logdissect` and `toolong` (tl).
#
# For each log merging tool, this script runs a search for log lines
# between two datetimes in three 5,000-line log files.
# Gathers program runtime performance data using `hyperfine` and GNU `time`.
# Then does some scraping of those results and outputs that as a markdown table.
# The markdown output is typically for display (bragging) in the top-level
# README.md file.
#
# pass `--skip-tl` to skip processing toolong which takes over the console
# window and stalls non-interactive consoles
#
# Set PROGRAMS_S4_LISTING to a TSV file listing other s4 programs to compare.
# Each non-empty line must have 2 tab-separated fields:
#   <path-to-s4> <extra-info>
# Hint: use `download-released-s4.sh` to quickly get old s4 binaries.
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
readonly skip_tl

PROJ_DIR=$(readlink -f "$(dirname -- "${0}")/../..")
readonly PROJ_DIR
cd "${PROJ_DIR}"
readonly REQUIREMENTS_FILE=./tools/compare-log-mergers/requirements.txt

(
    export PAGER=cat
    set -x
    # pipe to `cat` to make very sure a pager is not used
    git log -n1 --oneline -1 | cat -
)

# use full path to Unix tools
TIME=$(which time)
(set -x; "${TIME}" --version) | head -n1
readonly TIME

# do a little work to find Python interpreter in the PATH
PYTHON=${PYTHON-$(
    if which -a python &>/dev/null; then
        echo -n 'python'
    else
        echo -n 'python3'
    fi
)}
readonly PYTHON
(set -x; "${PYTHON}" --version) | head -n1

# check for hyperfine
HYPERFINE=$(which hyperfine) || {
    echo "ERROR: hyperfine not found in PATH" >&2
    echo "install:" >&2
    echo "    cargo install --locked hyperfine" >&2
    exit 1
}
readonly HYPERFINE
(set -x; "${HYPERFINE}" --version)

# check for jq
if ! which jq &>/dev/null; then
    echo "ERROR: jq not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install jq" >&2
    exit 1
fi
JQ=$(which jq)
(set -x; "${JQ}" --version)
readonly JQ

# check for lnav
if [[ "${PROGRAM_LNAV-}" = '' ]] && ! which lnav &>/dev/null; then
    echo "ERROR: lnav not found in PATH" >&2
    echo '       and $PROGRAM_LNAV not set' >&2
    echo "install:" >&2
    echo "    sudo apt install lnav" >&2
    exit 1
fi
readonly PROGRAM_LNAV=${PROGRAM_LNAV-lnav}

declare -ir HRUNS=30

if [[ ! "${PROGRAMS_S4_LISTING-}" ]]; then
    echo "ERROR: Environment variable PROGRAMS_S4_LISTING must be set to a TSV file" >&2
    exit 1
fi
if [[ ! -f "${PROGRAMS_S4_LISTING}" ]]; then
    echo "ERROR: PROGRAMS_S4_LISTING must point to a TSV file: ${PROGRAMS_S4_LISTING}" >&2
    exit 1
fi

# make sure Python packages are installed to expected versions
(
    set -x
    "${PYTHON}" -m pip install \
        --upgrade \
        --no-python-version-warning --disable-pip-version-check \
        --quiet \
        -r "${REQUIREMENTS_FILE}"
)

# precompile all python packages
PYSITE_PKG_PATH=$("${PYTHON}" -c "import sysconfig; print(sysconfig.get_path('purelib'))")
readonly PYSITE_PKG_PATH
(
    set -x
    "${PYTHON}" -m compileall -o 0 -o 1 -o 2 -q "${PYSITE_PKG_PATH}"
)

declare -ar files=(
    './tools/compare-log-mergers/gen-5000-1-facesA.log'
    './tools/compare-log-mergers/gen-5000-1-facesB.log'
    './tools/compare-log-mergers/gen-5000-1-facesC.log'
    './tools/compare-log-mergers/gen-5000-1-facesD.log'
    './tools/compare-log-mergers/gen-5000-1-facesE.log'
    './tools/compare-log-mergers/gen-5000-1-facesF.log'
    './tools/compare-log-mergers/gen-5000-1-facesG.log'
    './tools/compare-log-mergers/gen-5000-1-facesH.log'
    './tools/compare-log-mergers/gen-5000-1-facesI.log'
    './tools/compare-log-mergers/gen-5000-1-facesJ.log'
)

TEMPD=$(mktemp -p "${TMP-/tmp}" -d "s4-tmp-compare-log-mergers_XXXXX")
readonly TEMPD
tmpOut=$(mktemp -p "${TEMPD}" -t "tmp_XXX.out")
readonly tmpOut
mdfinal="${TEMPD}/compare-log-mergers.md"
readonly mdfinal
json_files=$(mktemp -p "${TEMPD}" -t "json_files_XXX.txt")
readonly json_files
tm_files=$(mktemp -p "${TEMPD}" -t "tm_files_XXX.txt")
readonly tm_files

function json_file_new() {
    local path
    path=$(mktemp -p "${TEMPD}" -t "json_XXXXX.json")
    printf '%s\n' "${path}" | tee -a "${json_files}"
}

function tm_file_new() {
    local path
    path=$(mktemp -p "${TEMPD}" -t "tm_XXXXX.txt")
    printf '%s\n' "${path}" | tee -a "${tm_files}"
}

function exit_() {
    rm -rf -- "${TEMPD}"
}
trap exit_ EXIT

# datetime range for s4, lnav
readonly after_dt="2000-01-01T00:20:00"
readonly befor_dt="2000-01-01T00:50:00"
# datetime range for logdissect
readonly after_dt_ld="200001010020"
readonly befor_dt_ld="200001010050"
# datetime range for GNU grep + sort
# grep regex equivalent of $after_dt $befor_dt
readonly regex_dt='^2000-01-01T00\:([234][[:digit:]]\:[[:digit:]]{2}|50\:00)'

function files_caching() {
    # force reading of files from disk to allow any possible caching.
    # crude but possibly better than nothing
    cat "${files[@]}" > "${tmpOut}"
}

function echo_line() {
    python -Bc "print('─' * ${COLUMNS:-100})"
    echo
}

function echo_warn() {
    # yellow bold with default background for warnings
    echo -e "\033[1;33mWARNING: ${*}\033[0m"
}

function echo_title() {
    # black bold with orange background for titles
    echo -e "\033[1;4;38;5;0;48;5;214m${*}\033[0m"
    echo
}

function echo_command() {
    # green bold with default background for commands
    echo -e "\033[1;32m${PS4}${*}\033[0m"
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
readonly TIME_FORMAT='%M|%P|%E'

# print the s4 version string from the --version string
#
# handle newer 0.8.80 format
#    $ s4 --version
#    s4 (Super Speedy Syslog Searcher)
#    Version: 0.9.81
#    MSRV: 1.88.0
#    Allocator: system
#
# handle older formats format <0.8.80
#    $ s4 --version
#    s4 (Super Speedy Syslog Searcher), Version 0.7.79, Allocator system
#
function s4_version() {
    declare -r prog=$1
    declare out=
    out=$("${prog}" --version)
    # check if new version
    if echo -n "${out}" | grep -qE '^Version: ' &> /dev/null; then
        # new version format
        echo -n "${out}" | grep -Ee '^Version: ' | sed -E 's/^Version: ([0-9.]+)$/\1/'
    else
        # old version format
        echo -n "${out}" | sed -E 's/.+, Version (.+), .+/\1/'
    fi
}

function s4_profile() {
    declare -r prog=$1
    declare out=
    out=$("${prog}" --version)
    # check if new version
    if echo -n "${out}" | grep -qE '^Profile: ' &> /dev/null; then
        # new version format
        echo -n "${out}" | grep -Ee '^Profile: ' | sed -E 's/^Profile: (.+)$/\1/'
    else
        # old version format
        echo_warn "no Profile found for s4 program ${prog}" >&2
    fi
}

# print the s4 allocator from the --version string
function s4_allocator() {
    declare -r prog=$1
    declare out=
    out=$("${prog}" --version)
    # check if new version
    if echo -n "${out}" | grep -qE '^Allocator: ' &> /dev/null; then
        # new version format
        echo -n "${out}" | grep -Ee '^Allocator: ' | sed -E 's/^Allocator: (\w+)$/\1/'
    else
        # old version format
        echo -n "${out}" | sed -E 's/.+, Allocator (\w+)/\1/'
    fi
}

# print the s4 platform from the --version string
function s4_platform() {
    declare -r prog=$1
    declare out=
    out=$("${prog}" --version)
    # check if new version
    if echo -n "${out}" | grep -qE '^Platform: ' &> /dev/null; then
        # new version format
        echo -n "${out}" | grep -Ee '^Platform: ' | sed -E 's/^Platform: (.+)$/\1/'
    else
        # try to grep the platform from the file name
        echo -n "${prog}" | sed -E 's/.+\/s4_(.+)_v[0-9.]+/\1/'
    fi
}

# print the s4 `Optimization Level` from the --version string
function s4_opt_lvl() {
    declare -r prog=$1
    declare out=
    out=$("${prog}" --version)
    # check if version
    if echo -n "${out}" | grep -qE '^Optimization Level: ' &> /dev/null; then
        echo -n "${out}" | grep -Ee '^Optimization Level: ' | sed -E 's/^Optimization Level: (.+)$/\1/'
    else
        echo_warn "no Optimization Level found for s4 program ${prog}" >&2
    fi
}

# Super Speedy Syslog Searcher (required s4 programs from TSV)

echo "Processing s4 programs from TSV file: ${PROGRAMS_S4_LISTING}" >&2
echo >&2

declare -i line_no=0
while IFS=$'\t' read -r s4_prog_path s4_prog_extra; do
    line_no=$((line_no + 1)) || true
    if [[ -z "${s4_prog_path-}" ]]; then
        continue
    fi
    if [[ "${s4_prog_path}" = \#* ]]; then
        continue
    fi

    s4_full_path=$(readlink -f "${PROJ_DIR}/${s4_prog_path}") || {
        echo_warn "failed to resolve full path for s4 program '${PROJ_DIR}/${s4_prog_path}'" >&2
        continue
    }
    echo_line
    echo_title "Super Speedy Syslog Searcher (S4) ${s4_full_path} ${s4_prog_extra}"

    if ! [[ -f "${s4_full_path}" ]]; then
        echo_warn "s4 program does not exist: ${s4_full_path}" >&2
        continue
    fi
    if ! [[ -x "${s4_full_path}" ]]; then
        echo_warn "s4 program is not executable: ${s4_full_path}" >&2
        continue
    fi
    if ! (set -x; "${s4_full_path}" --version); then
        echo_warn "failed to run s4 program with --version: ${s4_full_path}" >&2
        continue
    fi

    echo >&2
    json_file=$(json_file_new)
    tm_file=$(tm_file_new)
    (
        files_caching
        if [[ -n "${s4_prog_extra-}" ]]; then
            s4_prog_extra=" ${s4_prog_extra}"
        fi
        echo_command "${s4_full_path} -a='${after_dt}' -b='${befor_dt}' -cn ${files[*]}"
        set -x
        "${HYPERFINE}" --warmup=2 --style=basic --runs=${HRUNS} --export-json "${json_file}" -N -n "s4${s4_prog_extra-}" \
            -- \
            "'${s4_full_path}' -a='${after_dt}' -b='${befor_dt}' -cn ${files[*]} > /dev/null"
    )
    (
        files_caching
        version=$(s4_version "${s4_full_path}")
        profile=$(s4_profile "${s4_full_path}")
        allocator=$(s4_allocator "${s4_full_path}")
        platform=$(s4_platform "${s4_full_path}")
        opt_lvl=$(s4_opt_lvl "${s4_full_path}")
        set -x
        "${TIME}" --format="${TIME_FORMAT}|${version}|${profile}|${allocator}|${platform}|${opt_lvl}" --output="${tm_file}" \
            -- \
            "${s4_full_path}" \
            "-a=${after_dt}" \
            "-b=${befor_dt}" \
            "--color=never" \
            "${files[@]}" > "${tmpOut}"
    )
done <<< $(cat "${PROGRAMS_S4_LISTING}")

echo_line
echo_title "GNU grep + sort"

GREP=$(which grep)
readonly GREP
(set -x; "${GREP}" --version) | head -n1
SORT=$(which sort)
readonly SORT
(set -x; "${SORT}" --version) | head -n1

echo

json_file=$(json_file_new)
tm_file=$(tm_file_new)
(
    files_caching
    # search for datetimes between $after_dt $befor_dt
    # using decently constrained regexp to match meaning
    echo_command "${GREP} -hEe '${regex_dt}' -- ${files[*]} | ${SORT} -t ' ' -k 1 -s"
    set -x
    "${HYPERFINE}" --warmup=2 --style=basic --runs=${HRUNS} --export-json "${json_file}" --shell sh -n "grep+sort" \
        -- \
        ""${GREP}" -hEe '${regex_dt}' -- ${files[*]} | ${SORT} -t ' ' -k 1 -s > /dev/null"
)
(
    files_caching
    version=$("${GREP}" --version | head -n1 | cut -f4 -d' ')
    profile=' '
    allocator=' '
    platform=$(arch)
    opt_lvl=' '
    set -x
    "${TIME}" --format="${TIME_FORMAT}|${version}|${profile}|${allocator}|${platform}|${opt_lvl}" --output="${tm_file}" \
        -- \
        sh -c "'"${GREP}"' -hEe '${regex_dt}' -- ${files[*]} | '${SORT}' -t ' ' -k 1 -s" > "${tmpOut}"
)

# BUG: [20260712] lnav schema input is broken!? Using lnav 0.13.2, get this error:
#
#      $ lnav -N -n -c ';SELECT log_raw_text FROM lnav1 WHERE log_time BETWEEN Datetime("2000-01-01T00:20:00") AND Datetime("2000-01-01T00:50:00")' ./tools/compare-log-mergers/gen-5000-1-facesA.log ./tools/compare-log-mergers/gen-5000-1-facesB.log
#      ✘ error: “title” is not a valid log format
#       reason: no regexes specified
#       --> /home/user/.config/lnav/formats/installed/title.json:3
#      ✘ error: “title” is not a valid log format
#       reason: log message samples must be included in a format definition
#       --> /home/user/.config/lnav/formats/installed/title.json:3
#

if false; then

echo_line
echo_title "lnav"

json_file=$(json_file_new)
tm_file=$(tm_file_new)
(
    files_caching
    echo_command "${PROGRAM_LNAV}' -N -n -c ';SELECT log_raw_text FROM lnav1 WHERE log_time BETWEEN Datetime(\"${after_dt}\") AND Datetime(\"${befor_dt}\")' ${files[*]}"
    set -x
    "${PROGRAM_LNAV}" --version
    "${PROGRAM_LNAV}" -i -W ./tools/compare-log-mergers/lnav1.json
    "${HYPERFINE}" --warmup=2 --style=basic --runs=${HRUNS} --export-json "${json_file}" -N -n "${PROGRAM_LNAV}" \
        -- \
        "'${PROGRAM_LNAV}' -N -n \
-c ';SELECT log_raw_text FROM lnav1 WHERE log_time BETWEEN Datetime(\"${after_dt}\") AND Datetime(\"${befor_dt}\")' \
${files[*]}"
)

(
    files_caching
    version=$("${PROGRAM_LNAV}" --version | head -n1 | cut -d' ' -f2)
    profile=$(arch)
    allocator=' '
    platform=' '
    opt_lvl=' '
    set -x
    "${TIME}" --format="${TIME_FORMAT}|${version}|${profile}|${allocator}|${platform}|${opt_lvl}" --output="${tm_file}" \
        -- \
        "${PROGRAM_LNAV}" -N -n \
            -c ";SELECT log_raw_text FROM lnav1 WHERE log_time BETWEEN Datetime('${after_dt}') AND Datetime('${befor_dt}')" \
            "${files[@]}" > "${tmpOut}"
)
fi

echo_line
echo_title "logmerger"

PROGRAM_LM=${PROGRAM_LM-logmerger}
# XXX: logmerger does not have a `--version` option
echo "${PS4}logmerger --version"
"${PYTHON}" -m pip list | grep -Fe 'logmerger'

# precompile logmerger
(
    set -x
    "${PYTHON}" -m compileall -o 0 -o 1 -o 2 -q "${PYSITE_PKG_PATH}/logmerger"*
)

echo

json_file=$(json_file_new)
tm_file=$(tm_file_new)
(
    files_caching
    export PYTHONOPTIMIZE=2
    echo_command "${PROGRAM_LM} --inline --output=- --start '${after_dt}' --end '${befor_dt}' ${files[*]}"
    set -x
    "${HYPERFINE}" -i --warmup=2 --style=basic --runs=${HRUNS} --export-json "${json_file}" \
        --shell sh -n "${PROGRAM_LM}" \
        -- \
        "'${PROGRAM_LM}' --inline --output=- --start '${after_dt}' --end '${befor_dt}' ${files[*]} > /dev/null"
)
(
    files_caching
    version=$("${PYTHON}" -m pip list | grep -Fe 'logmerger' | awk '{print $2}')
    profile=' '
    allocator=' '
    platform=$("${PYTHON}" --version)
    export PYTHONOPTIMIZE=2
    opt_lvl='2'
    set -x
    "${TIME}" --format="${TIME_FORMAT}|${version}|${profile}|${allocator}|${platform}|${opt_lvl}" --output="${tm_file}" \
        -- \
        "${PROGRAM_LM}" \
        "--inline" \
        "--output=-" \
        "--start" \
        "${after_dt}" \
        "--end" \
        "${befor_dt}" \
        "${files[@]}" \
         > "${tmpOut}"
) || true

echo_line
echo_title "logdissect"

PROGRAM_LD=${PROGRAM_LD-logdissect}
(set -x; "${PROGRAM_LD}" --version)

echo

(
    echo "TODO: figure out how to use logdissect. I'm unable to get it to match on ANY files." >&2
    exit 0
    files_caching
    version=$("${PROGRAM_LD}" --version | head -n1)
    profile=$("${PYTHON}" -c 'import platform; print(platform.machine())')
    allocator=' '
    platform=' '
    opt_lvl=' '
    echo_command "${PROGRAM_LD}" --range "'${after_dt_ld}-${befor_dt_ld}'" "${files[@]}" >&2
    set -x
    "${TIME}" --format="${TIME_FORMAT}|${version}|${profile}|${allocator}|${platform}|${opt_lvl}" --output="${tm}" \
        -- \
        "${PROGRAM_LD}" \
        --range "${after_dt_ld}-${befor_dt_ld}" \
        "${files[@]}" \
        > "${tmpOut}"
)
echo

echo_line
echo_title "TooLong"

# TODO: how to force toolong to not create a TUI window so it doesn't need
#       to be forcefully killed?
PROGRAM_TL=${PROGRAM_TL-tl}
(set -x; "${PROGRAM_TL}" --version)

(
    # precompile toolong
    set -x
    "${PYTHON}" -m compileall -o 0 -o 1 -o 2 -q "${PYSITE_PKG_PATH}/toolong"
)

echo

tm_file=$(tm_file_new)
if ! ${skip_tl}; then
    (
        files_caching
        # tl, version 1.5.0
        version=$("${PROGRAM_TL}" --version | head -n1 | cut -d' ' -f3)
        allocator=' '
        profile=' '
        platform=$("${PYTHON}" --version)
        export PYTHONOPTIMIZE=2
        opt_lvl='2'
        echo_command "${PROGRAM_TL}" --merge --output-merge "${tmpOut}" "${files[@]}"
        # run toolong (tl)
        # there is no way to make toolong automatically exit after processing input
        # the user must manually exit the TUI
        set -x
        "${TIME}" --format="${TIME_FORMAT}|${version}|${profile}|${allocator}|${platform}|${opt_lvl}" --output="${tm_file}" \
            -- \
            "${PROGRAM_TL}" \
            --merge \
            --output-merge "${tmpOut}" \
            "${files[@]}"
    )
else
    echo "Skipping toolong (tl)" >&2
    # set dummy data
    echo '0|0|0:0' > "${tm_file}"
fi
cat "${tm_file}"
echo

erealtime=$(cat "${tm_file}" | cut -d'|' -f3 | cut -d':' -f2)
json_file=$(json_file_new)
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
}' > "${json_file}"

echo_line

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
echo '|Program|Version|Profile|Allocator|Platform|Opt Level|Mean (ms)|Min (ms)|Max (ms)|Max RSS (KB)|CPU %|' > "${tmpOut}"
echo '|:---   |:---   |:---   |:---     |:---    |:---     |---:    |---:    |---:      |---:       |---: |' >> "${tmpOut}"

json_count=$(wc -l < "${json_files}")
tm_count=$(wc -l < "${tm_files}")
echo "Processing ${json_count} JSON and time files to generate markdown table..." >&2

# sanity self-check
if [[ "${json_count}" -ne "${tm_count}" ]]; then
    echo "ERROR: number of JSON files ${json_count} does not match number of time files ${tm_count}" >&2
    exit 1
fi

# markdown table rows
exec 3< "${json_files}"
exec 4< "${tm_files}"
while IFS= read -r json <&3 && IFS= read -r tm <&4; do
    if file_isempty "${json}"; then
        echo "skip empty JSON file ${json}" >&2
        continue
    fi
    if file_isempty "${tm}"; then
        echo "skip empty time file ${tm}" >&2
        continue
    fi
    echo "Processing files: ${json} ${tm}" >&2
    (
        command=$($JQ '.results[0].command' < "${json}" | tr -d '"')
        mean=$(to_milliseconds $($JQ '.results[0].mean' < "${json}"))
        stddev=$(to_milliseconds $($JQ '.results[0].stddev' < "${json}"))
        min=$(to_milliseconds $($JQ '.results[0].min' < "${json}"))
        max=$(to_milliseconds $($JQ '.results[0].max' < "${json}"))
        mss=$(cat "${tm}" | cut -d'|' -f1)
        cpup=$(cat "${tm}" | cut -d'|' -f2)
        elapsed=$(cat "${tm}" | cut -d'|' -f3)
        version=$(cat "${tm}" | cut -d'|' -f4)
        profile=$(cat "${tm}" | cut -d'|' -f5)
        allocator=$(cat "${tm}" | cut -d'|' -f6)
        platform=$(cat "${tm}" | cut -d'|' -f7)
        opt_lvl=$(cat "${tm}" | cut -d'|' -f8)
        echo "|\`${command}\`|${version}|${profile}|${allocator}|${platform}|${opt_lvl}|${mean} ± ${stddev}|${min}|${max}|${mss}|${cpup}|"
    ) >> "${tmpOut}"
done
exec 3<&-
exec 4<&-

cat "${tmpOut}" | column -t -s '|' -o '|' > "${mdfinal}"

mdFinalFinal=${mdfinal}
if [[ "${DIROUT-}" ]]; then
    mkdir -p "${DIROUT}"
    cp -av "${mdfinal}" "${DIROUT}"
    mdFinalFinal="${DIROUT}/$(basename "${mdfinal}")"
fi

echo
cat "${mdFinalFinal}"
echo

(set -x; ./tools/mdtohtml.sh "${mdFinalFinal}")

export PATH="${PATH}:${HOME}/go/bin"  # for glow
if which glow &>/dev/null; then
    declare -i col=160
    if [[ $COLUMNS -lt $col ]]; then
        col=${COLUMNS}
    fi
    glow "${mdFinalFinal}" --width ${col}
    echo
else
    echo "install 'glow' for pretty markdown viewing" >&2
    echo "    go install github.com/charmbracelet/glow/v2@latest" >&2
fi
