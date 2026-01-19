#!/usr/bin/env bash
#
# gnuplot memory usage of `s4` when processing increasing number of log files.
#
# user should set:
#   FILE         - path to a log file to be used for testing
#   FNUM_MAX     - maximum number of files to test (default: 100)
#   S4_PROGRAM   - path to the `s4` binary to test (default: ./target/release/s4)
#   DIROUT       - output directory for markdown and SVG files (default: current directory)
#   PYTHON       - python3 interpreter (default: python3)
# requires:
#   hyperfine
#   jq
#   gnuplot
#   python3
#   xmllint
# usage:
#   FILE=path/to/log FNUM_MAX=N ./tools/compare-mem.sh [<s4-args>]
# example:
#   FILE=./tools/compare-log-mergers/gen-5000-1-facesA.log FNUM_MAX=200 ./tools/compare-mem.sh --color=never
# outputs:
#   compare-mem-data__<log-file-name>__<FNUM_MAX>.md
#   compare-mem-rss__<log-file-name>__<FNUM_MAX>.svg
#   compare-mem-time__<log-file-name>__<FNUM_MAX>.svg
#

set -euo pipefail

cd "$(dirname "${0}")/.."

declare -r DIROUT=${DIROUT-"."}

# check for hyperfine
hyperfine=$(which hyperfine) || {
    echo "ERROR: hyperfine not found in PATH" >&2
    echo "install:" >&2
    echo "    cargo install --locked hyperfine" >&2
    exit 1
}
(set -x; hyperfine --version)

which xmllint &>/dev/null || {
    echo "ERROR: xmllint not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install libxml2-utils" >&2
    exit 1
}
(set -x; xmllint --version | head -n1)

# check for jq
JQ=$(which jq) || {
    echo "ERROR: jq not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install jq" >&2
    exit 1
}
(set -x; "${JQ}" --version)

# check for python
PYTHON=${PYTHON-"python3"}
if ! which "${PYTHON}" &>/dev/null; then
    echo "ERROR: python3 not found in PATH" >&2
    exit 1
fi
(set -x; "${PYTHON}" --version)

# check for gnuplot
gnuplot=$(which gnuplot) || {
    echo "ERROR: gnuplot not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install gnuplot" >&2
    exit 1
}
(set -x; gnuplot --version)

declare -r FILE=${FILE-'./tools/compare-log-mergers/gen-5000-1-facesA.log'}
declare -r FILE_NAME=$(basename -- "${FILE}")

# hyperfine runs
declare -ir HYPERFINE_RUNS=${HYPERFINE_RUNS-5}

# the upcoming `git checkout` may remove some of the above log files
# so copy them to the temporary directory
TDIR_LOGS=/tmp/s4-compare-mem
mkdir -vp "${TDIR_LOGS}"

function echo_line() {
    # print a line as wide as the terminal
    python -Bc "import sys; print('─' * ${COLUMNS:-100}, file=sys.stderr)"
    echo >&2
}

function file_size() {
    # print file size in bytes
    stat --printf='%s' "${1}"
}

function file_isempty() {
    # return 0 if file is empty or does not exist, 1 otherwise
    if [[ ! -f "${1}" ]]; then
        return 1
    fi
    [[ $(file_size "${1}") -eq 0 ]]
}

function to_3f() {
    # print number to 3 decimal places; '0.0034125904' -> '0.003'
    # reads from stdin
    local data=
    read data
    "${PYTHON}" -c "print('%.3f' % (${data}))"
}

function to_milliseconds() {
    # from seconds to milliseconds; '0.0034125904' -> '3'
    # reads from stdin
    local data=
    read data
    "${PYTHON}" -c "print('%d' % int(${data} * 1000))"
}

function repeat() {
    # print $2 string $1 times
    declare -i start=1
    declare -i end=${1:-80}
    declare str=${2}
    for i in $(seq $start $end); do
        echo -n "${str}"
    done
}

function print_cpu_model () {
    # print CPU model name
    grep -m1 -Fe 'model name' /proc/cpuinfo | cut -f2 -d':' | sed -Ee 's/^[[:space:]]+//'
}

function max() {
    # print max of the arguments which are alphanumeric
    echo -n "$@" | tr ' ' '\n' | sort -nr | head -n1
}

function print_time_now_ms() {
    # print current time in milliseconds
    echo -n "${EPOCHREALTIME//./}" | cut -b1-13
}

function xml_escape() {
    # escape XML special characters
    echo -n "${@}" | sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g' -e "s/'/\&apos;/g" -e 's/"/\&quot;/g'
}

function regex_escape() {
    # escape regex special characters
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
if [[ ${FILE_SZ_UNCOMPRESSED} -ne 0 ]]; then
    FILE_SZ_UNCOMPRESSED_KB=$((FILE_SZ_UNCOMPRESSED / 1024 + 1))
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
source /etc/os-release
OsName="${NAME} ${VERSION_ID}"

tmpD=$(mktemp -d -t "s4-compare-mem_XXXXX")

function exit_() {
    rm -rf "${tmpD}"
}

trap exit_ EXIT

#
# start the markdown draft file
#

declare -r MD_DRAFT="${tmpD}/compare-mem-draft.md"

# markdown table header
echo "\
|Files       |Profile|Mean (ms)|Min (ms)|Max (ms)|Diff (ms)|Max RSS (KB)|Max RSS (KB) diff|CPU %|
|:---        |:---   |---:     |---:    |---:    |---:     |---:        |---:             |---: |" > "${MD_DRAFT}"

#
# run the tests for each file count
#

first=true
declare -a time_values=()
declare -a time_diff_values=()
declare -a mss_values=()
declare -a fnum_values=()
declare -a mss_diff_values=()
declare -ir FNUM_MAX=${FNUM_MAX-100}
declare s4_command=$(printf "%q" "${S4_PROGRAM}")
for arg in "${@}"; do
    arg_escaped=$(printf "%q" "$arg")
    s4_command+=" ${arg_escaped}"
done
# markdown table rows
for fnum in $(seq 1 ${FNUM_MAX}); do
    echo_line

    echo "Testing '${S4_PROGRAM}' with ${fnum} file(s)" >&2
    echo >&2

    json="${tmpD}/${fnum}.json"

    declare -a current_files=()
    for ((i=0; i < fnum; i++)); do
        # XXX: presuming there are no spaces in the file name
        current_files+=("${FILE}")
    done
    declare -i proc_time_beg=$(print_time_now_ms)
    (
        set -x
        ${hyperfine} \
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

    # memory_usage_byte is explained at
    # https://github.com/sharkdp/hyperfine/discussions/846

    time_last=${mean-0}
    mean=$($JQ '.results[0].mean' < "${json}" | to_milliseconds)
    stddev=$($JQ '.results[0].stddev' < "${json}" | to_milliseconds)
    min=$($JQ '.results[0].min' < "${json}" | to_milliseconds)
    max=$($JQ '.results[0].max' < "${json}" | to_milliseconds)
    mss_last=${mss-0}
    mss=$($JQ '.results[0].memory_usage_byte | max / 1024' < "${json}")

    if ${first}; then
        mss_diff='-'
        time_diff='-'
    else
        declare -i mss_diff=$((mss - mss_last))
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
    echo "|${fnum}|${BUILD_PROFILE}|${mean} ± ${stddev}|${min}|${max}|${time_diff}|${mss}|${mss_diff}|${cpup}|" >> "${MD_DRAFT}"

    fnum_values+=("${fnum}")
    mss_values+=("${mss}")
    time_values+=("${mean}")

    echo >&2
    echo "For ${HYPERFINE_RUNS} runs of ${fnum} files: time ${proc_time_diff} ms, Max RSS ${mss} KB (current datetime $(date))" >&2

    first=false
done

echo_line

#
# create the final markdown file of results
#
declare -r MD_FINAL="${DIROUT}/compare-mem-data__${FILE_NAME}__${FNUM_MAX}.md"

# prettify the markdown table with aligned columns
cat "${MD_DRAFT}" | column -t -s '|' -o '|' > "${MD_FINAL}"

if which glow &>/dev/null; then
    glow --width=${COLUMNS} --preserve-new-lines "${MD_FINAL}"
else
    cat "${MD_FINAL}"
fi

echo >&2

#
# gnuplot an ASCII graph for file count vs max RSS
#

mss_max=$(printf "%s\n" "${mss_values[@]}" | sort -nr | head -n1)
mss_min=$(printf "%s\n" "${mss_values[@]}" | sort -n | head -n1)
mss_diff_max=$(printf "%s\n" "${mss_diff_values[@]}" | sort -nr | head -n1)
mss_diff_min=$(printf "%s\n" "${mss_diff_values[@]}" | sort -n | head -n1)
mss_diff_avg=$(echo -n "${mss_diff_values[@]}" | awk '{sum=0; for(i=1;i<=NF;i++) sum+=$i; print sum/NF}')

declare -i FILE_SZ_MULTIPLE_DENOMINATOR=${FILE_SZ_KB}
if [[ ${FILE_SZ_UNCOMPRESSED} -gt 0 ]]; then
    FILE_SZ_MULTIPLE_DENOMINATOR=${FILE_SZ_UNCOMPRESSED_KB}
fi
mss_diff_multiple_max=$("${PYTHON}" -c "print('%.1f' % (${mss_diff_max} / ${FILE_SZ_MULTIPLE_DENOMINATOR}))")
mss_diff_multiple_min=$("${PYTHON}" -c "print('%.1f' % (${mss_diff_min} / ${FILE_SZ_MULTIPLE_DENOMINATOR}))")
mss_diff_multiple_avg=$("${PYTHON}" -c "print('%.1f' % (${mss_diff_avg} / ${FILE_SZ_MULTIPLE_DENOMINATOR}))")

time_diff_max=$(printf "%s\n" "${time_diff_values[@]}" | sort -nr | head -n1)
time_diff_min=$(printf "%s\n" "${time_diff_values[@]}" | sort -n | head -n1)
time_diff_avg=$(echo -n "${time_diff_values[@]}" | awk '{sum=0; for(i=1;i<=NF;i++) sum+=$i; print sum/NF}')

let mss_max_x=$((mss_max + 10000)) || true
let mss_min_x=$((mss_min - 20000)) || true

# sanity check
if [[ ${#mss_values[@]} -ne ${#fnum_values[@]} ]]; then
    echo "Mismatched mss_values fnum_values; ${#mss_values[@]} ${#fnum_values[@]}" >&2
    exit 1
fi

DataRss=$(for i in "${!mss_values[@]}"; do echo "${mss_values[$i]} ${fnum_values[$i]}"; done)
DataRssDiffs=$(for i in "${!mss_diff_values[@]}"; do echo "${mss_diff_values[$i]} ${fnum_values[$i+1]}"; done)

declare -i ytics_step=0
declare -i xtics_step=0

if [[ $FNUM_MAX -le 50 ]]; then
    ytics_step=1
elif [[ $FNUM_MAX -le 100 ]]; then
    ytics_step=2
elif [[ $FNUM_MAX -le 200 ]]; then
    ytics_step=4
else
    ytics_step=10
fi

if [[ ${mss_max_x} -lt 100 ]]; then
    xtics_step=1
elif [[ ${mss_max_x} -lt 1000 ]]; then
    xtics_step=10
elif [[ ${mss_max_x} -lt 10000 ]]; then
    xtics_step=1000
elif [[ ${mss_max_x} -lt 100000 ]]; then
    xtics_step=10000
elif [[ ${mss_max_x} -lt 500000 ]]; then
    xtics_step=15000
elif [[ ${mss_max_x} -lt 1000000 ]]; then
    xtics_step=100000
else
    xtics_step=200000
fi

GNUPLOT_TERMINAL=$(cat <<EOF
set terminal dumb size $COLUMNS, $(($COLUMNS / 2))
set color
set key off
set xlabel "Max Resident Set Size (KB)" noenhanced
set ylabel "File count" noenhanced
set xtics ${xtics_step}
set ytics ${ytics_step}
set grid xtics
set grid ytics
set xrange [0:${mss_max_x}]
set yrange [0:$((${FNUM_MAX} + 1))]
set title "command: ${s4_command} ${FILE_NAME}…\n\nMax Resident Set Size (KB) per additional file of size ${FILE_SZ_KB} KB"
\$Data << EOD
$DataRss
EOD
plot \$Data with linespoints
EOF
)

(
    set -x
    echo "$GNUPLOT_TERMINAL" | "${gnuplot}"
)

function gnuplot_svg_title_replace() {
    local file="${1}"
    shift
    # replace the non-descriptive '<title>Gnuplot</title>' with something interesting
    sed -i -e "s|$(regex_escape "<title>Gnuplot</title>")|$(regex_escape "<title>$(xml_escape "${@}")</title>")|" -- "${file}"
}

function xml_format() {
    local file="${1}"
    xmllint --format "${file}" --output "${tmpD}/${file}.tmp"
    mv -f "${tmpD}/${file}.tmp" "${file}"
}

#
# gnuplot create SVG for file count vs max RSS
#

declare -r OUT_SVG_RSS="${DIROUT}/compare-mem-rss__${FILE_NAME}__${FNUM_MAX}.svg"

echo >&2

(
    echo "Max RSS diff (KB)|${mss_diff_max}"
    echo "Min RSS diff (KB)|${mss_diff_min}"
    echo "Avg RSS diff (KB)|${mss_diff_avg}"
    echo "File Size (KB) |${FILE_SZ_KB}"
    if [[ ${FILE_SZ_UNCOMPRESSED} -gt 0 ]]; then
        FILE_SZ_UNCOMPRESSED_KB=$((FILE_SZ_UNCOMPRESSED / 1024))
        echo "Uncompressed File Size (KB) |${FILE_SZ_UNCOMPRESSED_KB}"
    fi
    echo "RSS diff multiple (avg)|${mss_diff_multiple_avg}"
    echo "RSS diff multiple (max)|${mss_diff_multiple_max}"
    echo "RSS diff multiple (min)|${mss_diff_multiple_min}"
) | column -t -s '|' -o ':' --table-columns='Info,Data' --table-right='Data' --table-noheadings

declare -i SVG_WIDTH=1280
declare -i SVG_HEIGHT=1080
if [[ $FNUM_MAX -le 20 ]]; then
    SVG_WIDTH=768
    SVG_HEIGHT=480
elif [[ $FNUM_MAX -ge 201 ]]; then
    SVG_WIDTH=1920
fi

declare -i FONT_SIZE_OUTER=12
declare -i FONT_SIZE_TICS=8
declare -i FONT_SIZE_LABELS=8
if [[ $FNUM_MAX -ge 50 ]]; then
    FONT_SIZE_TICS=8
    FONT_SIZE_LABELS=6
fi

FONT_NAME_OUTER="Arial"
FONT_NAME_TICS="Monospace"
FONT_NAME_POINTS="Monospace"

FILE_SZ_MESG="File Size: ${FILE_SZ_KB} KB (${FILE_SZ} bytes)"
if [[ ${FILE_SZ_UNCOMPRESSED} -gt 0 ]]; then
    FILE_SZ_MESG+=", Uncompressed Size: ${FILE_SZ_UNCOMPRESSED_KB} KB (${FILE_SZ_UNCOMPRESSED} bytes)"
fi

COLOR_1="dark-magenta"
COLOR_2="blue"

GNUPLOT_SVG=$(cat <<EOF
set terminal svg size ${SVG_WIDTH}, ${SVG_HEIGHT} fname "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}"
set color
set key off
set output "${OUT_SVG_RSS}"
set title "Command: ${s4_command} ${FILE_NAME}…\n\nBuild profile: ${BUILD_PROFILE}, Version: ${Version}, Allocator: ${Allocator}, Platform: ${Platform}, Optimization Level: ${OptimizationLevel}, MSRV: ${Msrv}\nRun on: ${OsName}, CPU: ${CpuModel}\n\nFile: ${FILE}\n${FILE_SZ_MESG}\nHyperfine runs per data point: ${HYPERFINE_RUNS}\nMax max RSS difference per 1 File: ${mss_diff_max} KB (×${mss_diff_multiple_max} file size)\nAvg max RSS difference per 1 File: ${mss_diff_avg} KB (×${mss_diff_multiple_avg} file size)\nMin max RSS difference per 1 File: ${mss_diff_min} KB (×${mss_diff_multiple_min} file size)\n\n" \
    font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" \
    noenhanced
set xlabel left "Max Resident Set Size (KB)" textcolor rgbcolor "${COLOR_1}" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" enhanced
set ylabel "File count (${FILE_NAME})\n" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" noenhanced
set xtics ${xtics_step} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set ytics ${ytics_step} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set grid xtics
set grid ytics
set xrange [0:${mss_max_x}]
set yrange [0:$((${FNUM_MAX} + 1))]
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
    echo "$GNUPLOT_SVG" | gnuplot
)

gnuplot_svg_title_replace "${OUT_SVG_RSS}" "Max RSS (KB) per N file for '${FILE_NAME}'"
xml_format "${OUT_SVG_RSS}"

echo >&2
echo "SVG output written to: ${OUT_SVG_RSS}" >&2

#
# gnuplot create SVG for file count vs time
#

declare -r OUT_SVG_TIME="${DIROUT}/compare-time__${FILE_NAME}__${FNUM_MAX}.svg"

DataTime=$(for i in "${!time_values[@]}"; do echo "${time_values[$i]} ${fnum_values[$i]}"; done)
DataTimeDiffs=$(for i in "${!time_diff_values[@]}"; do echo "${time_diff_values[$i]} ${fnum_values[$i+1]}"; done)

declare -i time_max_x=0
time_max_x=$(max "${time_values[@]}")
time_max_x=$((time_max_x + 1))

if [[ ${time_max_x} -lt 100 ]]; then
    xtics_step=1
elif [[ ${time_max_x} -lt 1000 ]]; then
    xtics_step=10
elif [[ ${time_max_x} -lt 10000 ]]; then
    xtics_step=1000
else
    xtics_step=200000
fi

GNUPLOT_SVG=$(cat <<EOF
set terminal svg size ${SVG_WIDTH}, ${SVG_HEIGHT} fname "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}"
set color
set key off
set title "Command: ${s4_command} ${FILE_NAME}…\nBuild profile: ${BUILD_PROFILE}, Version: ${Version}, Allocator: ${Allocator}, Platform: ${Platform}, Optimization Level: ${OptimizationLevel}, MSRV: ${Msrv}\nRun on: ${OsName}, CPU: ${CpuModel}\n\nFile: ${FILE}\n${FILE_SZ_MESG}\nHyperfine runs per data point: ${HYPERFINE_RUNS}\n\nTime Difference per 1 File Max ${time_diff_max} ms\nTime Difference per 1 File Avg ${time_diff_avg} ms\nTime Difference per 1 File Min ${time_diff_min} ms" \
    font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" \
    noenhanced
set output "${OUT_SVG_TIME}"
set xlabel "Time (ms)" textcolor rgbcolor "${COLOR_1}" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" enhanced
set ylabel "File count (${FILE_NAME})\n" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" noenhanced
set xtics ${xtics_step} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set ytics ${ytics_step} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set grid xtics
set grid ytics
set xrange [0:${time_max_x}]
set yrange [0:$((${FNUM_MAX} + 1))]
\$DataTime << EOD
$DataTime
EOD
\$DataTimeDiffs << EOD
$DataTimeDiffs
EOD
plot \$DataTime with lines linecolor rgbcolor "${COLOR_1}" title "Time (ms) mean among ${HYPERFINE_RUNS} runs", \
     \$DataTime using 1:2:(sprintf("%d ms", \$1)) with labels point pointtype 7 pointsize 0.5 offset char 3,-1 font "${FONT_NAME_POINTS},${FONT_SIZE_LABELS}" title "File Count, Time (ms) mean", \
     \$DataTimeDiffs with lines linecolor rgbcolor "${COLOR_2}" title "Time (ms) Diff from processing N files to processing N+1 files", \
     \$DataTimeDiffs using 1:2:(sprintf("%d ms (diff)", \$1)) with labels point pointtype 7 pointsize 0.5 offset char 3,-1 font "${FONT_NAME_POINTS},${FONT_SIZE_LABELS}" title "File Count, Time (ms) Diff from processing N files to processing N+1 files"
EOF
)

(
    set -x
    echo "$GNUPLOT_SVG" | "${gnuplot}"
)

gnuplot_svg_title_replace "${OUT_SVG_TIME}" "Time (ms) mean per N file for '${FILE_NAME}'"
xml_format "${OUT_SVG_TIME}"

echo >&2
echo "SVG output written to: ${OUT_SVG_TIME}" >&2
