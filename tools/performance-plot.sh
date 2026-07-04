#!/usr/bin/env bash
#
# gnuplot `s4` performance when processing increasing number of log files,
# specifically Max RSS and mean time.
#

if [[ "${1-}" = "-h" || "${1-}" = "--help" || "${1-}" = "-?" ]]; then
    echo "\
Usage: ${0} [s4-args-for-all-runs]

user must set environment variables:
  FILE         - path to a log file to be used for testing (default: ./tools/compare-log-mergers/gen-5000-1-facesA.log)
  FILE_NUM     - maximum number of files to test (default: 100)

user may set environment variables:
  S4_PROGRAM   - path to the \`s4\` binary to test (default: ./target/release/s4)
  DIROUT       - output directory for markdown and SVG files (default: current directory)
  PYTHON       - python3 interpreter (default: python3)

requires:
  hyperfine    - measures runtime and memory usage
  jq           - parses hyperfine JSON output
  gnuplot      - creates ASCII and SVG graphs
  python3      - used for some math and string formatting
  xmllint      - prettify the SVG files

usage:
  FILE=path/to/some.log FILE_NUM=N ./tools/performance-plot.sh [<s4-args>]

example:
  FILE=./tools/compare-log-mergers/gen-5000-1-facesA.log FILE_NUM=200 ./tools/performance-plot.sh --color=never

outputs:
  performance-plot-data__<log-file-name>__<FILE_NUM>.csv
  performance-plot-data__<log-file-name>__<FILE_NUM>.md
  performance-plot-rss__<log-file-name>__<FILE_NUM>.svg
  performance-plot-time__<log-file-name>__<FILE_NUM>.svg
" >&2
    exit 0
fi

set -euo pipefail

cd "$(dirname "${0}")/.."

declare -r DIROUT=${DIROUT-"."}

# check for hyperfine
HYPERFINE=$(which hyperfine) || {
    echo "ERROR: hyperfine not found in PATH" >&2
    echo "install:" >&2
    echo "    cargo install --locked hyperfine" >&2
    exit 1
}
readonly HYPERFINE
(set -x; "$HYPERFINE" --version)

# check for xmllint
XMLLINT=$(which xmllint) || {
    echo "ERROR: xmllint not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install libxml2-utils" >&2
    exit 1
}
readonly XMLLINT
(set -x; "$XMLLINT" --version | head -n1)

# check for jq
JQ=$(which jq) || {
    echo "ERROR: jq not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install jq" >&2
    exit 1
}
readonly JQ
(set -x; "${JQ}" --version)

# check for python
PYTHON=${PYTHON-"python3"}
if ! which "${PYTHON}" &>/dev/null; then
    echo "ERROR: python3 not found in PATH" >&2
    exit 1
fi
readonly PYTHON
(set -x; "${PYTHON}" --version)

# check for gnuplot
GNUPLOT=$(which gnuplot) || {
    echo "ERROR: gnuplot not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install gnuplot" >&2
    exit 1
}
readonly GNUPLOT
(set -x; "$GNUPLOT" --version)

declare -r FILE=${FILE-'./tools/compare-log-mergers/gen-5000-1-facesA.log'}
declare -r FILE_NAME=$(basename -- "${FILE}")
# no comma
declare -r FILE_NAME_NOC=$(echo -n "${FILE_NAME}" | tr -s ',' '_')

# hyperfine runs
declare -ir HYPERFINE_RUNS=${HYPERFINE_RUNS-5}

# echo color escapes
declare -r CLR_INFO="\033[1;32m"  # green
declare -r CLR_RESET="\033[0m"

# the upcoming `git checkout` may remove some of the above log files
# so copy them to the temporary directory
TDIR_LOGS=/tmp/s4-performance-plot
mkdir -vp "${TDIR_LOGS}"

# print a line as wide as the terminal
function echo_line() {
    python -Bc "import sys; print('─' * ${COLUMNS:-100}, file=sys.stderr)"
    echo >&2
}

# print file size in bytes
function file_size() {
    stat --printf='%s' "${1}"
}

# return 0 if file is empty or does not exist, 1 otherwise
function file_isempty() {
    if [[ ! -f "${1}" ]]; then
        return 1
    fi
    [[ $(file_size "${1}") -eq 0 ]]
}

# print number to 3 decimal places; '0.0034125904' -> '0.003'
# reads from stdin
function to_3f() {
    local data=
    read data
    "${PYTHON}" -c "print('%.3f' % (${data}))"
}

# from seconds to milliseconds; '0.0034125904' -> '3'
# reads from stdin
function to_milliseconds() {
    local data=
    read data
    "${PYTHON}" -c "print('%d' % int(${data} * 1000))"
}

# print $2 string $1 times
function repeat() {
    declare -i start=1
    declare -i end=${1:-80}
    declare str=${2}
    for i in $(seq $start $end); do
        echo -n "${str}"
    done
}

# print CPU model name
function print_cpu_model () {
    grep -m1 -Fe 'model name' /proc/cpuinfo | cut -f2 -d':' | sed -Ee 's/^[[:space:]]+//'
}

# print CPU core count
function print_cpu_cores() {
    grep -c -Fe 'processor' /proc/cpuinfo
}

# print RAM total size in megabytes
function print_ram_total_mb() {
    grep -m1 -Fe 'MemTotal' /proc/meminfo \
    | cut -f2 -d':' \
    | sed -Ee 's/^[[:space:]]+//' \
    | cut -f1 -d' ' \
    | awk '{print int($1 / 1024)}'
}

# print max integer value of the arguments which are numeric
function max() {
    "${PYTHON}" -c "
import sys
data = [int(x) for x in sys.argv[1:]]
print(max(data))" \
"${@}"
}

function min() {
    "${PYTHON}" -c "
import sys
data = [int(x) for x in sys.argv[1:]]
print(min(data))" \
"${@}"
}

function avg() {
    "${PYTHON}" -c "
import sys
data = [int(x) for x in sys.argv[1:]]
print(int(sum(data) / len(data)))" \
"${@}"
}

# print current time in milliseconds
function print_time_now_ms() {
    echo -n "${EPOCHREALTIME//./}" | cut -b1-13
}

# escape XML special characters
function xml_escape() {
    echo -n "${@}" | sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g' -e "s/'/\&apos;/g" -e 's/"/\&quot;/g'
}

# escape regex special characters
function regex_escape() {
    echo -n "${@}" | "$PYTHON" -c 'import re, sys; print(re.escape(sys.stdin.read().rstrip()))'
}

# check if FILE exists
if [[ ! -f "${FILE}" ]]; then
    echo "FILE not found or not a file '${FILE}'" >&2
    exit 1
fi

# check if file name has spaces
if [[ "${FILE}" =~ [[:space:]] ]]; then
    echo "FILE name has spaces which is not supported: '${FILE}'" >&2
    exit 1
fi

declare -ir FILE_SZ=$(file_size "${FILE}")
declare -ir FILE_SZ_KB=$((FILE_SZ / 1024 + 1))

# get file size of compressed files, e.g. .gz, .xz, etc.
declare -i FILE_SZ_UNCOMPRESSED=0
if [[ "${FILE}" == *.bz2 ]]; then
    FILE_SZ_UNCOMPRESSED=$((set -x; bzip2 -k -d -c "${FILE}") | wc -c)
elif [[ "${FILE}" == *.gz ]]; then
    FILE_SZ_UNCOMPRESSED=$((set -x; gzip -k -d -c "${FILE}") | wc -c)
elif [[ "${FILE}" == *.lz4 ]]; then
    FILE_SZ_UNCOMPRESSED=$((set -x; lz4 -k -d -c "${FILE}") | wc -c)
elif [[ "${FILE}" == *.xz ]]; then
    FILE_SZ_UNCOMPRESSED=$((set -x; xz -k -d -c "${FILE}") | wc -c)
elif [[ "${FILE}" == *.zst ]]; then
    FILE_SZ_UNCOMPRESSED=$((set -x; zstd -k -d -c "${FILE}") | wc -c)
fi
declare -i FILE_SZ_UNCOMPRESSED_KB=0
declare -i FILE_SZ_UNCOMPRESSED_BLOCKS=0
if [[ ${FILE_SZ_UNCOMPRESSED} -ne 0 ]]; then
    FILE_SZ_UNCOMPRESSED_KB=$((FILE_SZ_UNCOMPRESSED / 1024 + 1))
    FILE_SZ_UNCOMPRESSED_BLOCKS=$((FILE_SZ_UNCOMPRESSED / 65536 + 1))
fi

declare -r S4_PROGRAM=${S4_PROGRAM-"./target/release/s4"}
# very presumptive that the profile name will be the 3rd path component
# e.g. ./target/release/s4 -> release
#      ./target/debug/s4   -> debug
BUILD_PROFILE=$(echo "${S4_PROGRAM}" | cut -f3 -d'/')
if [[ -z "${BUILD_PROFILE}" ]]; then
    BUILD_PROFILE="unknown"
fi

# example --version output
#
# $ ./target/release/s4 --version
# s4 (Super Speedy Syslog Searcher)
# Version: 0.8.80
# MSRV: 1.85.0
# Allocator: system
# Platform: x86_64-unknown-linux-gnu
# Rust Build Flags: 
# Optimization Level: 3
# License: MIT
# Repository: https://github.com/jtmoon79/super-speedy-syslog-searcher
# Author: James Thomas Moon
#

# sanity check S4_PROGRAM
(set -x; "${S4_PROGRAM}" --version)

# parse version info, OS info, CPU model
version_out=$("${S4_PROGRAM}" --version 2>&1)
Version=$(echo "${version_out}" | grep -m1 -Ee '^Version:' | cut -f2 -d' ' | tr -d '\n')
Allocator=$(echo "${version_out}" | grep -m1 -Ee '^Allocator:' | cut -f2 -d' ' | tr -d '\n')
Platform=$(echo "${version_out}" | grep -m1 -Ee '^Platform:' | cut -f2 -d' ' | tr -d '\n')
OptimizationLevel=$(echo "${version_out}" | grep -m1 -Ee '^Optimization Level:' | cut -f3 -d' ' | tr -d '\n')
Msrv=$(echo "${version_out}" | grep -m1 -Ee '^MSRV:' | cut -f2 -d' ' | tr -d '\n')
CpuModel=$(print_cpu_model)
CpuCores=$(print_cpu_cores)
RamTotalMB=$(print_ram_total_mb)
source /etc/os-release
OsName="${NAME} ${VERSION_ID}"
GitTagLast=$(git describe --tags --abbrev=0 || echo "unknown")
# XXX: must match `BLOCKSZ_DEF` defined in `blockreader.rs`
declare -i S4_BLOCKSZ=${S4_BLOCKSZ-65535}
declare -ir S4_BLOCKSZ_KB=$((S4_BLOCKSZ / 1024))
declare -ir FILE_SZ_BLOCKS=$((FILE_SZ / 65536 + 1))

declare -ir FILE_NUM=${FILE_NUM-100}

declare -r MD_FINAL="${DIROUT}/performance-plot__${FILE_NAME}__${FILE_NUM}__data.md"
declare -r CSV_FINAL="${DIROUT}/performance-plot__${FILE_NAME}__${FILE_NUM}__data.csv"

if [[ -f "${MD_FINAL}" ]]; then
    echo "Final output already exists, skipping. '${MD_FINAL}'" >&2
    exit 0
fi
if [[ -f "${CSV_FINAL}" ]]; then
    echo "Final output already exists, skipping. '${CSV_FINAL}'" >&2
    exit 0
fi

tmpD=$(mktemp -d -t "tmp-s4-performance-plot_XXXXX")

function exit_() {
    rm -rf "${tmpD}"
}

trap exit_ EXIT

mkdir -p "${DIROUT}"

#
# start the markdown draft file
#

declare -r MD_DRAFT="${tmpD}/performance-plot-draft.md"
declare -r CSV_DRAFT="${tmpD}/performance-plot_${FILE_NAME}_${FILE_NUM}.csv"

# markdown table header
echo "\
|Files       |Profile|Mean (ms)|Min (ms)|Max (ms)|Diff (ms)|Max RSS (KB)|Max RSS (KB) diff|CPU %|
|:---        |:---   |---:     |---:    |---:    |---:     |---:        |---:             |---: |" > "${MD_DRAFT}"
# CSV header
echo "#File,Files,Profile,Mean (ms),Min (ms),Max (ms),Diff (ms),Max RSS (KB),Max RSS (KB) diff,CPU %" > "${CSV_DRAFT}"

#
# run the tests for each file count
#

first=true
declare -a time_values=()
declare -a time_diff_values=()
declare -a mss_values=()
declare -a fnum_values=()
declare -a mss_diff_values=()
declare s4_command=$(printf "%q" "${S4_PROGRAM}")
# must pass command as a single shell-escaped string to `hyperfine`
for arg in "${@}"; do
    arg_escaped=$(printf "%q" "$arg")
    s4_command+=" ${arg_escaped}"
done

declare -i count=0
declare -ir COUNT_RUNS_FILE=$(seq 1 ${FILE_NUM} | wc -l)
for fnum in $(seq 1 ${FILE_NUM}); do
    count+=1
    echo_line

    echo -e "${CLR_INFO}Testing '${S4_PROGRAM}' with ${fnum} files; run ${count} of ${COUNT_RUNS_FILE}${CLR_RESET}" >&2
    echo >&2

    json="${tmpD}/${fnum}.json"

    declare -a current_files=()
    for ((i=0; i < fnum; i++)); do
        # XXX: presuming there are no spaces in the file name
        current_files+=("${FILE}")
    done
    # here is the hyperfine run
    declare -i proc_time_beg=$(print_time_now_ms)
    (
        set -x
        ${HYPERFINE} \
            --warmup=0 \
            --style=color \
            --time-unit=millisecond \
            --runs=${HYPERFINE_RUNS} \
            --export-json "${json}" \
            -N \
            --command-name "s4 ${fnum} files" \
            -- \
                "${s4_command} ${current_files[*]}"
    )
    declare -i proc_time_end=$(print_time_now_ms)
    declare -i proc_time_diff=$((proc_time_end - proc_time_beg))
    echo >&2

    # example hyperfine JSON output:
    #
    # {
    #   "results": [
    #     {
    #       "command": "s4_0.7.77",
    #       "mean": 0.34721515733333336,
    #       "stddev": 0.0022061126997868254,
    #       "median": 0.34669891150000004,
    #       "user": 0.29701803333333326,
    #       "system": 0.44120176666666666,
    #       "min": 0.34394707700000005,
    #       "max": 0.35471939,
    #       "times": [
    #         0.35471939,
    #         ...,
    #         0.346103957
    #       ],
    #       "memory_usage_byte": [
    #         138768384,
    #         ...,
    #         138899456
    #       ],
    #       "exit_codes": [
    #         0,
    #         ...,
    #         0
    #       ]
    #     }
    #   ]
    # }
    cat "${json}" | "${JQ}" .

    # memory_usage_byte is in bytes, explained at https://github.com/sharkdp/hyperfine/discussions/846

    time_last=${mean-0}
    mean=$($JQ '.results[0].mean' < "${json}" | to_milliseconds)
    stddev=$($JQ '.results[0].stddev' < "${json}" | to_milliseconds)
    min=$($JQ '.results[0].min' < "${json}" | to_milliseconds)
    max=$($JQ '.results[0].max' < "${json}" | to_milliseconds)
    mss_last=${mss_KB-0}
    # convert to KiB
    mss_KB=$($JQ '.results[0].memory_usage_byte | max / 1024' < "${json}")

    if ${first}; then
        mss_diff='-'
        time_diff='-'
    else
        declare -i mss_diff=$((mss_KB - mss_last))
        mss_diff_values+=("${mss_diff}")
        if [[ "${mss_diff}" -gt 0 ]]; then
            mss_diff="+${mss_diff}"
        fi

        declare -i time_diff=$((mean - time_last))
        time_diff_values+=("${time_diff}")
        if [[ "${time_diff}" -gt 0 ]]; then
            time_diff="+${time_diff}"
        fi
    fi
    cpup=$($JQ '.results[0].user + .results[0].system' < "${json}" | to_3f)
    # markdown table row
    echo "|${fnum}|${BUILD_PROFILE}|${mean} ± ${stddev}|${min}|${max}|${time_diff}|${mss_KB}|${mss_diff}|${cpup}|" >> "${MD_DRAFT}"
    # CSV row
    echo "${FILE_NAME_NOC},${fnum},${BUILD_PROFILE},${mean} ± ${stddev},${min},${max},${time_diff},${mss_KB},${mss_diff},${cpup}" >> "${CSV_DRAFT}"

    fnum_values+=("${fnum}")
    mss_values+=("${mss_KB}")
    time_values+=("${mean}")

    echo >&2
    echo -e "${CLR_INFO}For ${HYPERFINE_RUNS} runs of ${fnum} files: time ${proc_time_diff} ms, Max RSS ${mss_KB} KB (current datetime $(date))${CLR_RESET}" >&2

    first=false
done

echo_line

#
# create the final markdown file of results
#

# prettify the markdown table with aligned columns
cat "${MD_DRAFT}" | column -t -s '|' -o '|' > "${MD_FINAL}"
# save the CSV data
cp -av "${CSV_DRAFT}" "${CSV_FINAL}"

export PATH="${PATH}:${HOME}/go/bin"  # for glow
if which glow &>/dev/null; then
    glow --width=${COLUMNS} --preserve-new-lines "${MD_FINAL}"
else
    cat "${MD_FINAL}"
fi

echo >&2

#
# gnuplot an ASCII graph for file count vs max RSS
#

declare -r gnuplot_vertical_line_x0='set arrow from 0, graph 0 to 0, graph 1 nohead'

mss_max=$(max "${mss_values[@]}")
mss_min=$(min "${mss_values[@]}")
mss_diff_max=$(max "${mss_diff_values[@]}")
mss_diff_min=$(min "${mss_diff_values[@]}")
mss_diff_avg=$(avg "${mss_diff_values[@]}")

declare -i FILE_SZ_MULTIPLE_DENOMINATOR=${FILE_SZ_KB}
if [[ ${FILE_SZ_UNCOMPRESSED} -gt 0 ]]; then
    FILE_SZ_MULTIPLE_DENOMINATOR=${FILE_SZ_UNCOMPRESSED_KB}
fi
mss_diff_multiple_max=$("${PYTHON}" -c "print('%.1f' % (${mss_diff_max} / ${FILE_SZ_MULTIPLE_DENOMINATOR}))")
mss_diff_multiple_min=$("${PYTHON}" -c "print('%.1f' % (${mss_diff_min} / ${FILE_SZ_MULTIPLE_DENOMINATOR}))")
mss_diff_multiple_avg=$("${PYTHON}" -c "print('%.1f' % (${mss_diff_avg} / ${FILE_SZ_MULTIPLE_DENOMINATOR}))")

mss_diff_blocksz_multiple_max=$("${PYTHON}" -c "print('%.1f' % ((${mss_diff_max} * 1024) / ${S4_BLOCKSZ}))")
mss_diff_blocksz_multiple_min=$("${PYTHON}" -c "print('%.1f' % ((${mss_diff_min} * 1024) / ${S4_BLOCKSZ}))")
mss_diff_blocksz_multiple_avg=$("${PYTHON}" -c "print('%.1f' % ((${mss_diff_avg} * 1024) / ${S4_BLOCKSZ}))")

time_diff_max=$(max "${time_diff_values[@]}")
time_diff_min=$(min "${time_diff_values[@]}")
time_diff_avg=$(avg "${time_diff_values[@]}")

# sanity check
if [[ ${#mss_values[@]} -ne ${#fnum_values[@]} ]]; then
    echo "Mismatched mss_values fnum_values; ${#mss_values[@]} ${#fnum_values[@]}" >&2
    exit 1
fi

DataRss=$(for i in "${!mss_values[@]}"; do echo "${mss_values[$i]} ${fnum_values[$i]}"; done)
DataRssDiffs=$(for i in "${!mss_diff_values[@]}"; do echo "${mss_diff_values[$i]} ${fnum_values[$i+1]}"; done)

declare -i x_range_max=$(max "${mss_values[@]}" "${mss_diff_values[@]}")
x_range_max+=20000

declare -i x_range_min=$(min "${mss_values[@]}" "${mss_diff_values[@]}")
if [[ ${x_range_min} -lt 0 ]]; then
    let x_range_min-=1000
else
    x_range_min=0
fi

# draw a vertical line at x=0 if the x_range_min is less than 0
declare line_at_x0=
if [[ ${x_range_min} -lt 0 ]]; then
    line_at_x0=${gnuplot_vertical_line_x0}
fi

declare -i x_range_max_minus_min=$((x_range_max - x_range_min))

declare -i xtics_step=0
if [[ ${x_range_max_minus_min} -lt 100 ]]; then
    xtics_step=1
elif [[ ${x_range_max_minus_min} -lt 1000 ]]; then
    xtics_step=10
elif [[ ${x_range_max_minus_min} -lt 10000 ]]; then
    xtics_step=1000
elif [[ ${x_range_max_minus_min} -lt 100000 ]]; then
    xtics_step=10000
elif [[ ${x_range_max_minus_min} -lt 500000 ]]; then
    xtics_step=15000
elif [[ ${x_range_max_minus_min} -lt 1000000 ]]; then
    xtics_step=100000
else
    xtics_step=200000
fi

declare -i ytics_step=0
if [[ $FILE_NUM -le 50 ]]; then
    ytics_step=1
elif [[ $FILE_NUM -le 100 ]]; then
    ytics_step=2
elif [[ $FILE_NUM -le 200 ]]; then
    ytics_step=4
else
    ytics_step=10
fi

function gnuplot_svg_title_replace() {
    local file="${1}"
    shift
    # replace the non-descriptive '<title>Gnuplot</title>' with something interesting
    sed -i -e "s|$(regex_escape "<title>Gnuplot</title>")|$(regex_escape "<title>$(xml_escape "${@}")</title>")|" -- "${file}"
}

function xml_format() {
    local file="${1}"
    local tmp_file="${tmpD}/$(basename "${file}").tmp"
    "$XMLLINT" --format "${file}" --output "${tmp_file}"
    mv -f "${tmp_file}" "${file}"
}

#
# gnuplot create SVG for file count vs max RSS
#

declare -r OUT_SVG_RSS="${DIROUT}/performance-plot__${FILE_NAME}__${FILE_NUM}__rss.svg"

echo >&2

(
    echo "Max RSS diff (KB)|${mss_diff_max}"
    echo "Min RSS diff (KB)|${mss_diff_min}"
    echo "Avg RSS diff (KB)|${mss_diff_avg}"
    echo "File Size (KB) |${FILE_SZ_KB}"
    echo "Block Size (Bytes) |${S4_BLOCKSZ}"
    echo "File Size (Blocks) |${FILE_SZ_BLOCKS}"
    if [[ ${FILE_SZ_UNCOMPRESSED} -gt 0 ]]; then
        FILE_SZ_UNCOMPRESSED_KB=$((FILE_SZ_UNCOMPRESSED / 1024))
        echo "Uncompressed File Size (KB) |${FILE_SZ_UNCOMPRESSED_KB}"
        echo "Uncompressed File Size (Blocks) |${FILE_SZ_UNCOMPRESSED_BLOCKS}"
    fi
    echo "RSS diff multiple (avg)|${mss_diff_multiple_avg}"
    echo "RSS diff multiple (max)|${mss_diff_multiple_max}"
    echo "RSS diff multiple (min)|${mss_diff_multiple_min}"
) | column -t -s '|' -o ':' --table-columns='Info,Data' --table-right='Data' --table-noheadings

declare -i SVG_HEIGHT=1280
declare -i SVG_WIDTH=1280
if [[ $FILE_NUM -le 20 ]]; then
    SVG_HEIGHT=520
    SVG_WIDTH=768
elif [[ $FILE_NUM -ge 200 ]]; then
    SVG_HEIGHT=1536
    SVG_WIDTH=1280
fi

declare -i FONT_SIZE_OUTER=12
declare -i FONT_SIZE_TICS=8
declare -i FONT_SIZE_LABELS=8
declare -i FONT_SIZE_POINTS=8
if [[ $FILE_NUM -ge 50 ]]; then
    FONT_SIZE_TICS=8
    FONT_SIZE_LABELS=6
    FONT_SIZE_POINTS=6
fi
if [[ $FILE_NUM -gt 100 ]]; then
    FONT_SIZE_OUTER=12
    FONT_SIZE_TICS=6
    FONT_SIZE_LABELS=5
    FONT_SIZE_POINTS=5
fi

FONT_NAME_OUTER="Arial"
FONT_NAME_TICS="Monospace"
FONT_NAME_POINTS="Monospace"

FILE_SZ_MESG="File Size: ${FILE_SZ_KB} KB (${FILE_SZ} bytes) (${FILE_SZ_BLOCKS} blocks)"
if [[ ${FILE_SZ_UNCOMPRESSED} -gt 0 ]]; then
    FILE_SZ_MESG+=", Uncompressed Size: ${FILE_SZ_UNCOMPRESSED_KB} KB (${FILE_SZ_UNCOMPRESSED} bytes) (${FILE_SZ_UNCOMPRESSED_BLOCKS} blocks)"
fi

COLOR_1="dark-magenta"
COLOR_2="blue"
COLOR_3="green"

GNUPLOT_SVG=$(cat <<EOF
set terminal svg size ${SVG_WIDTH}, ${SVG_HEIGHT} fname "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}"
set encoding utf8
set color
set key off
set output "${OUT_SVG_RSS}"
set title "Command: ${s4_command} ${FILE_NAME} …\n\nBuild profile: ${BUILD_PROFILE}, Version: ${Version} (git tag ${GitTagLast}), MSRV: ${Msrv}\nAllocator: ${Allocator}, Platform: ${Platform}, Optimization Level: ${OptimizationLevel}\nRun on: ${OsName}, CPU: ${CpuModel} (${CpuCores} cores), RAM: ${RamTotalMB} MB\n\nHyperfine runs per data point: ${HYPERFINE_RUNS}\n\nFile: ${FILE}\nBlock Size: ${S4_BLOCKSZ_KB} KB (${S4_BLOCKSZ} Bytes)\n${FILE_SZ_MESG}\n\nMax max RSS difference per 1 File: ${mss_diff_max} KB (×${mss_diff_multiple_max} file size) (×${mss_diff_blocksz_multiple_max} Blocks)\nAvg max RSS difference per 1 File: ${mss_diff_avg} KB (×${mss_diff_multiple_avg} file size) (×${mss_diff_blocksz_multiple_avg} Blocks)\nMin max RSS difference per 1 File: ${mss_diff_min} KB (×${mss_diff_multiple_min} file size) (×${mss_diff_blocksz_multiple_min} Blocks)\n\n" \
    font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" \
    noenhanced
set format "%.0f"
set xlabel left "Max Resident Set Size (KB)" textcolor rgbcolor "${COLOR_1}" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" enhanced
set ylabel "File count ${FILE_NUM}\n" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" noenhanced
set xtics ${xtics_step} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set ytics ${ytics_step} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set grid xtics
set grid ytics
set xrange [${x_range_min}:${x_range_max}]
set yrange [0:$((${FILE_NUM} + 1))]
${line_at_x0}
\$DataRss << EOD
$DataRss
EOD
\$DataRssDiffs << EOD
$DataRssDiffs
EOD
plot \$DataRss with lines linecolor rgbcolor "${COLOR_1}" title "Max RSS (KB)", \
     \$DataRss every 1 using 1:2:(sprintf("%d", \$1)) with labels point pointtype 7 pointsize 0.5 offset char 5,-0.5 font "${FONT_NAME_POINTS},${FONT_SIZE_LABELS}" title "Max RSS (KB)", \
     \$DataRssDiffs with lines linecolor rgbcolor "${COLOR_2}" title "Max RSS Diff (KB) from processing N files to processing N+1 files", \
     \$DataRssDiffs every 1 using 1:2:(sprintf("%d (diff)", \$1)) with labels point pointtype 7 pointsize 0.5 offset char 5,-0.5 font "${FONT_NAME_POINTS},${FONT_SIZE_LABELS}" title "Max RSS Diff (KB) from processing N files to processing N+1 files"
EOF
)
# TODO: add labels to each point see https://stackoverflow.com/a/63194918/471376 ?
#       cannot get this to work after many varied attempts

(
    set -x
    echo "$GNUPLOT_SVG" | "$GNUPLOT"
)

gnuplot_svg_title_replace "${OUT_SVG_RSS}" "Max RSS (KB) per N file for '${FILE_NAME}'"
xml_format "${OUT_SVG_RSS}"

echo >&2

#
# gnuplot create SVG for file count vs time
#

declare -r OUT_SVG_TIME="${DIROUT}/performance-plot__${FILE_NAME}__${FILE_NUM}__time.svg"

DataTime=$(for i in "${!time_values[@]}"; do echo "${time_values[$i]} ${fnum_values[$i]}"; done)
DataTimeDiffs=$(for i in "${!time_diff_values[@]}"; do echo "${time_diff_values[$i]} ${fnum_values[$i+1]}"; done)

declare -i time_max_x=0
time_max_x=$(max "${time_values[@]}" "${time_diff_values[@]}")
declare -i x_range_max=$((time_max_x + 1))

declare -i time_min_x=0
time_min_x=$(min "${time_values[@]}" "${time_diff_values[@]}")
declare -i x_range_min=0
if [[ ${time_min_x} -lt 0 ]]; then
    x_range_min=$((time_min_x - 1))
fi

declare -i x_range_max_minus_min=$((x_range_max - x_range_min))

if [[ ${x_range_max_minus_min} -lt 100 ]]; then
    xtics_step=1
elif [[ ${x_range_max_minus_min} -lt 1000 ]]; then
    xtics_step=10
elif [[ ${x_range_max_minus_min} -lt 10000 ]]; then
    xtics_step=1000
else
    xtics_step=100000
fi

# draw a vertical line at x=0 if the x_range_min is less than 0
declare line_at_x0=
if [[ ${x_range_min} -lt 0 ]]; then
    line_at_x0=${gnuplot_vertical_line_x0}
fi

GNUPLOT_SVG=$(cat <<EOF
set terminal svg size ${SVG_WIDTH}, ${SVG_HEIGHT} fname "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}"
set encoding utf8
set color
set key off
set title "Command: ${s4_command} ${FILE_NAME} …\nBuild profile: ${BUILD_PROFILE}, Version: ${Version} (git tag ${GitTagLast}), MSRV: ${Msrv}\nAllocator: ${Allocator}, Platform: ${Platform}, Optimization Level: ${OptimizationLevel}\nRun on: ${OsName}, CPU: ${CpuModel} (${CpuCores} cores), RAM: ${RamTotalMB} MB\n\nHyperfine runs per data point: ${HYPERFINE_RUNS}\n\nTime Difference per 1 File Max ${time_diff_max} ms\nTime Difference per 1 File Avg ${time_diff_avg} ms\nTime Difference per 1 File Min ${time_diff_min} ms" \
    font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" \
    noenhanced
set output "${OUT_SVG_TIME}"
set format "%.0f"
set xlabel "Time (ms)" textcolor rgbcolor "${COLOR_1}" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" enhanced
set ylabel "File count ${FILE_NUM}\n" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" noenhanced
set xtics ${xtics_step} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set ytics ${ytics_step} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set grid xtics
set grid ytics
set xrange [${x_range_min}:${x_range_max}]
set yrange [0:$((${FILE_NUM} + 1))]
${line_at_x0}
\$DataTime << EOD
$DataTime
EOD
\$DataTimeDiffs << EOD
$DataTimeDiffs
EOD
plot \$DataTime with lines linecolor rgbcolor "${COLOR_1}" title "Time (ms) mean among ${HYPERFINE_RUNS} runs", \
     \$DataTime using 1:2:(sprintf("%d ms", \$1)) with labels point pointtype 7 pointsize 0.5 offset char 3,-0.5 font "${FONT_NAME_POINTS},${FONT_SIZE_POINTS}" title "File Count, Time (ms) mean", \
     \$DataTimeDiffs with lines linecolor rgbcolor "${COLOR_2}" title "Time (ms) Diff from processing N files to processing N+1 files", \
     \$DataTimeDiffs using 1:2:(sprintf("%d ms (diff)", \$1)) with labels point pointtype 7 pointsize 0.5 offset char 3,-0.5 font "${FONT_NAME_POINTS},${FONT_SIZE_POINTS}" title "File Count, Time (ms) Diff from processing N files to processing N+1 files"
EOF
)

(
    set -x
    echo "$GNUPLOT_SVG" | "$GNUPLOT"
)

gnuplot_svg_title_replace "${OUT_SVG_TIME}" "Time (ms) mean per N file for '${FILE_NAME}'"
xml_format "${OUT_SVG_TIME}"

#
# run the tests for each Block Size
#

echo_line

declare -ir FILE_RUNS_BLOCKSZ=${FILE_RUNS_BLOCKSZ-40}
declare -ir BLOCKSZ_ALIGN=${BLOCKSZ_ALIGN-1024}
declare -ir BLOCKSZ_MIN=$(((${BLOCKSZ_MIN-1024} / ${BLOCKSZ_ALIGN} + 1) * ${BLOCKSZ_ALIGN}))
declare -ir BLOCKSZ_MAX=$(((${BLOCKSZ_MAX-1048575} / ${BLOCKSZ_ALIGN}) * ${BLOCKSZ_ALIGN}))

echo -e "${CLR_INFO}Block sizes from ${BLOCKSZ_MIN} to ${BLOCKSZ_MAX} in increments of ${BLOCKSZ_ALIGN}${CLR_RESET}" >&2
echo >&2

declare -a current_files=()
for ((i=0; i < ${FILE_RUNS_BLOCKSZ}; i++)); do
    # XXX: presuming there are no spaces in the file name
    current_files+=("${FILE}")
done

declare -r MD_DRAFT_BSZ="${tmpD}/performance-plot-draft-blocksz.md"
declare -r CSV_DRAFT_BSZ="${tmpD}/performance-plot-draft-blocksz.csv"

declare -r MD_FINAL_BSZ="${DIROUT}/performance-plot__${FILE_NAME}__blocksz_${BLOCKSZ_ALIGN}.md"
declare -r CSV_FINAL_BSZ="${DIROUT}/performance-plot__${FILE_NAME}__blocksz_${BLOCKSZ_ALIGN}.csv"

# markdown table header
echo "\
|Block Size  |Profile|Mean (ms)|Min (ms)|Max (ms)|Diff (ms)|Max RSS (KB)|Max RSS (KB) diff|CPU %|
|:---        |:---   |---:     |---:    |---:    |---:     |---:        |---:             |---: |" > "${MD_DRAFT_BSZ}"
# CSV header
echo "#File,Block Size,Profile,Mean (ms),Min (ms),Max (ms),Diff (ms),Max RSS (KB),Max RSS (KB) diff,CPU %" > "${CSV_DRAFT_BSZ}"

first=true
declare -a time_values=()
declare -a time_diff_values=()
declare -a mss_values=()
declare -a blocksz_values=()
declare -a mss_diff_values=()

declare -i count=0
declare -ir COUNT_RUNS_BSZ=$(seq ${BLOCKSZ_MIN} ${BLOCKSZ_ALIGN} ${BLOCKSZ_MAX} | wc -l)
for blocksz in $(seq ${BLOCKSZ_MIN} ${BLOCKSZ_ALIGN} ${BLOCKSZ_MAX}); do
    count+=1
    echo -e "${CLR_INFO}Testing '${S4_PROGRAM}' with block size ${blocksz}; run ${count} of ${COUNT_RUNS_BSZ}. Step ${BLOCKSZ_ALIGN} up to ${BLOCKSZ_MAX}${CLR_RESET}" >&2
    echo >&2

    json="${tmpD}/blocksz_${blocksz}.json"

    # here is the hyperfine run
    declare -i proc_time_beg=$(print_time_now_ms)
    (
        export S4_BLOCKSZ=${blocksz}
        set -x
        ${HYPERFINE} \
            --warmup=0 \
            --style=color \
            --time-unit=millisecond \
            --runs=${HYPERFINE_RUNS} \
            --export-json "${json}" \
            -N \
            --command-name "S4_BLOCKSZ=${blocksz} ${s4_command} ${current_files[0]} ..." \
            -- \
                "${s4_command} ${current_files[*]}"
    )
    declare -i proc_time_end=$(print_time_now_ms)
    declare -i proc_time_diff=$((proc_time_end - proc_time_beg))
    echo >&2

    cat "${json}" | "${JQ}" .

    # memory_usage_byte is in bytes, explained at https://github.com/sharkdp/hyperfine/discussions/846

    time_last=${mean-0}
    mean=$($JQ '.results[0].mean' < "${json}" | to_milliseconds)
    stddev=$($JQ '.results[0].stddev' < "${json}" | to_milliseconds)
    min=$($JQ '.results[0].min' < "${json}" | to_milliseconds)
    max=$($JQ '.results[0].max' < "${json}" | to_milliseconds)
    mss_last=${mss_KB-0}
    # convert to KiB
    mss_KB=$($JQ '.results[0].memory_usage_byte | max / 1024' < "${json}")

    if ${first}; then
        unset mss_diff time_diff
        mss_diff='-'
        time_diff='-'
    else
        declare -i mss_diff=$((mss_KB - mss_last))
        mss_diff_values+=("${mss_diff}")
        if [[ "${mss_diff}" -gt 0 ]]; then
            mss_diff="+${mss_diff}"
        fi

        declare -i time_diff=$((mean - time_last))
        time_diff_values+=("${time_diff}")
        if [[ "${time_diff}" -gt 0 ]]; then
            time_diff="+${time_diff}"
        fi
    fi
    cpup=$($JQ '.results[0].user + .results[0].system' < "${json}" | to_3f)
    # markdown table row
    echo "|${blocksz}|${BUILD_PROFILE}|${mean} ± ${stddev}|${min}|${max}|${time_diff}|${mss_KB}|${mss_diff}|${cpup}|" >> "${MD_DRAFT_BSZ}"
    # CSV row
    echo "${FILE_NAME_NOC},${blocksz},${BUILD_PROFILE},${mean} ± ${stddev},${min},${max},${time_diff},${mss_KB},${mss_diff},${cpup}" >> "${CSV_DRAFT_BSZ}"

    blocksz_values+=("${blocksz}")
    mss_values+=("${mss_KB}")
    time_values+=("${mean}")

    echo >&2
    echo -e "${CLR_INFO}For ${HYPERFINE_RUNS} runs with block size ${blocksz}: time ${proc_time_diff} ms, Max RSS ${mss_KB} KB (current datetime $(date))${CLR_RESET}" >&2

    first=false

    echo_line
done

# prettify the markdown table with aligned columns
cat "${MD_DRAFT_BSZ}" | column -t -s '|' -o '|' > "${MD_FINAL_BSZ}"
# save the CSV data
cp -av "${CSV_DRAFT_BSZ}" "${CSV_FINAL_BSZ}"

if which glow &>/dev/null; then
    glow --width=${COLUMNS} --preserve-new-lines "${MD_FINAL_BSZ}"
else
    cat "${MD_FINAL_BSZ}"
fi

echo >&2

echo_line

#
# gnuplot create SVG for block size vs time
#

declare -r OUT_SVG_BLOCKSZ="${DIROUT}/performance-plot__${FILE_NAME}__blocksz_${BLOCKSZ_ALIGN}.svg"

DataTime=$(for i in "${!time_values[@]}"; do echo "${time_values[$i]} ${blocksz_values[$i]}"; done)
DataTimeDiffs=$(for i in "${!time_diff_values[@]}"; do echo "${time_diff_values[$i]} ${blocksz_values[$i+1]}"; done)

declare -i time_max_x=0
time_max_x=$(max "${time_values[@]}" "${time_diff_values[@]}")
declare -i x_range_max=$((time_max_x + 1))

declare -i time_min_x=0
time_min_x=$(min "${time_values[@]}" "${time_diff_values[@]}")
declare -i x_range_min=0
if [[ ${time_min_x} -lt 0 ]]; then
    x_range_min=$((time_min_x - 1))
fi

x_range_max_minus_min=$((x_range_max - x_range_min))

if [[ ${x_range_max_minus_min} -le 10 ]]; then
    xtics_step=1
elif [[ ${x_range_max_minus_min} -le 50 ]]; then
    xtics_step=5
elif [[ ${x_range_max_minus_min} -le 100 ]]; then
    xtics_step=10
elif [[ ${x_range_max_minus_min} -le 500 ]]; then
    xtics_step=50
else
    xtics_step=100
fi

ytics_step=${BLOCKSZ_ALIGN}

declare -i SVG_HEIGHT=1280
declare -i SVG_WIDTH=1280
if [[ ${#blocksz_values[@]} -lt 10 ]]; then
    SVG_HEIGHT=520
    SVG_WIDTH=768
elif [[ ${#blocksz_values[@]} -ge 20 ]]; then
    SVG_HEIGHT=1536
    SVG_WIDTH=1280
elif [[ ${#blocksz_values[@]} -ge 50 ]]; then
    SVG_HEIGHT=2048
    SVG_WIDTH=1440
fi

# draw a vertical line at x=0 if the x_range_min is less than 0
declare line_at_x0=
if [[ ${x_range_min} -lt 0 ]]; then
    line_at_x0=${gnuplot_vertical_line_x0}
fi

GNUPLOT_SVG=$(cat <<EOF
set terminal svg size ${SVG_WIDTH}, ${SVG_HEIGHT} fname "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}"
set encoding utf8
set color
set key off
set title "Command: S4_BLOCKSZ=(${BLOCKSZ_MIN}..${BLOCKSZ_MAX} step by ${BLOCKSZ_ALIGN}) ${s4_command} ${FILE_NAME} …\nBuild profile: ${BUILD_PROFILE}, Version: ${Version} (git tag ${GitTagLast}), MSRV: ${Msrv}\nAllocator: ${Allocator}, Platform: ${Platform}, Optimization Level: ${OptimizationLevel}\nRun on: ${OsName}, CPU: ${CpuModel}, Cores: ${CpuCores}, RAM: ${RamSize} MB\n\nHyperfine runs per data point: ${HYPERFINE_RUNS}\n\nTime Difference per 1 File Max ${time_diff_max} ms\nTime Difference per 1 File Avg ${time_diff_avg} ms\nTime Difference per 1 File Min ${time_diff_min} ms" \
    font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" \
    noenhanced
set output "${OUT_SVG_BLOCKSZ}"
set format "%.0f"
set xlabel "Time (ms)" textcolor rgbcolor "${COLOR_1}" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" enhanced
set ylabel "Block size ${BLOCKSZ_ALIGN} step\n" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" noenhanced
set xtics ${xtics_step} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set ytics ${ytics_step} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set grid xtics
set grid ytics
set xrange [${x_range_min}:${x_range_max}]
set yrange [${BLOCKSZ_MIN}:${BLOCKSZ_MAX}]
${line_at_x0}
\$DataTime << EOD
$DataTime
EOD
\$DataTimeDiffs << EOD
$DataTimeDiffs
EOD
plot \$DataTime with lines linecolor rgbcolor "${COLOR_1}" title "Time (ms) mean among ${HYPERFINE_RUNS} runs", \
     \$DataTime using 1:2:(sprintf("%d ms", \$1)) with labels point pointtype 7 pointsize 0.5 offset char 3,-0.5 font "${FONT_NAME_POINTS},${FONT_SIZE_POINTS}" title "Block Size, Time (ms) mean", \
     \$DataTimeDiffs with lines linecolor rgbcolor "${COLOR_2}" title "Time (ms) Diff from processing N files to processing N+1 files", \
     \$DataTimeDiffs using 1:2:(sprintf("%d ms (diff)", \$1)) with labels point pointtype 7 pointsize 0.5 offset char 3,-0.5 font "${FONT_NAME_POINTS},${FONT_SIZE_POINTS}" title "File Count, Time (ms) Diff from processing N files to processing N+1 files"
EOF
)

(
    set -x
    echo "$GNUPLOT_SVG" | "$GNUPLOT"
)

gnuplot_svg_title_replace "${OUT_SVG_BLOCKSZ}" "Time (ms) mean per ${BLOCKSZ_ALIGN} BlockSz step for ${FILE_RUNS_BLOCKSZ} x '${FILE_NAME}'"
xml_format "${OUT_SVG_BLOCKSZ}"

echo_line

echo >&2

echo -e "SVG RSS output written to: ${CLR_INFO}${OUT_SVG_RSS}${CLR_RESET}" >&2
echo -e "SVG TIME output written to: ${CLR_INFO}${OUT_SVG_TIME}${CLR_RESET}" >&2
echo -e "Markdown written to: ${CLR_INFO}${MD_FINAL}${CLR_RESET}" >&2
echo -e "CSV written to: ${CLR_INFO}${CSV_FINAL}${CLR_RESET}" >&2
echo >&2
echo -e "SVG BLOCKSZ output written to: ${CLR_INFO}${OUT_SVG_BLOCKSZ}${CLR_RESET}" >&2
echo -e "Markdown written to: ${CLR_INFO}${MD_FINAL_BSZ}${CLR_RESET}" >&2
echo -e "CSV written to: ${CLR_INFO}${CSV_FINAL_BSZ}${CLR_RESET}" >&2
