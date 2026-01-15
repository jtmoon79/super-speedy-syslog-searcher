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
# usage:
#   FILE=path/to/log FNUM_MAX=N ./tools/compare-mem.sh [<s4-args>]
# example:
#   FILE=./tools/compare-log-mergers/gen-5000-1-facesA.log FNUM_MAX=200 ./tools/compare-mem.sh --color=never
# outputs:
#   compare-mem-rss__<log-file-name>__<FNUM_MAX>.md
#   compare-mem-rss__<log-file-name>__<FNUM_MAX>.svg
#

set -euo pipefail

cd "$(dirname "${0}")/.."

DIROUT=${DIROUT-"."}

# check for hyperfine
hyperfine=$(which hyperfine) || {
    echo "ERROR: hyperfine not found in PATH" >&2
    echo "install:" >&2
    echo "    cargo install --locked hyperfine" >&2
    exit 1
}
(set -x; hyperfine --version)

# check for jq
if ! which jq &>/dev/null; then
    echo "ERROR: jq not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install jq" >&2
    exit 1
fi
JQ=$(which jq)

PYTHON=${PYTHON-"python3"}
if ! which "${PYTHON}" &>/dev/null; then
    echo "ERROR: python3 not found in PATH" >&2
    exit 1
fi

readonly HRUNS=5

declare -r FILE=${FILE-'./tools/compare-log-mergers/gen-5000-1-facesA.log'}
declare -r FILE_NAME=$(basename -- "${FILE}")
declare -ir FILE_SZ=$(stat --printf='%s' "${FILE}")

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


S4_PROGRAM=${S4_PROGRAM-"./target/release/s4"}
# very presumptive that the profile name will be the 3rd path component
# e.g. ./target/release/s4 -> release
#      ./target/debug/s4   -> debug
BUILD_PROFILE=$(echo "${S4_PROGRAM}" | cut -f3 -d'/')

(set -x; "${S4_PROGRAM}" --version)

tmpD=$(mktemp -d -t "compare-mem_XXXXX")

function exit_() {
    rm -rf "${tmpD}"
}

mddraft="${tmpD}/compare-mem-draft.md"

trap exit_ EXIT

# markdown table header
echo "\
|Files       |Profile|Mean (ms)|Min (ms)|Max (ms)|Max RSS (KB)|Max RSS (KB) diff|CPU %|
|:---        |:---   |---:     |---:    |---:    |---:        |---:             |---: |" > "${mddraft}"

first=true

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
    (
        set -x
        ${hyperfine} \
            --warmup=0 \
            --style=color \
            --time-unit=millisecond \
            --runs=${HRUNS} \
            --export-json "${json}" \
            -N \
            --command-name "s4 ${fnum} files" \
            -- \
                "${s4_command} ${current_files[*]}"
    )
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

    mean=$($JQ '.results[0].mean' < "${json}" | to_milliseconds)
    stddev=$($JQ '.results[0].stddev' < "${json}" | to_milliseconds)
    min=$($JQ '.results[0].min' < "${json}" | to_milliseconds)
    max=$($JQ '.results[0].max' < "${json}" | to_milliseconds)
    mss_last=${mss-0}
    mss=$($JQ '.results[0].memory_usage_byte | max / 1024' < "${json}")
    if ${first}; then
        mss_diff="-"
    else
        mss_diff=$((mss - mss_last))
        mss_diff_values+=("${mss_diff}")
        if [[ "${mss_diff}" -gt 0 ]]; then
            mss_diff="+${mss_diff}"
        fi
    fi
    cpup=$($JQ '.results[0].user + .results[0].system' < "${json}" | to_3f)
    echo "|${fnum}|${BUILD_PROFILE}|${mean} ± ${stddev}|${min}|${max}|${mss}|${mss_diff}|${cpup}|" >> "${mddraft}"

    mss_values+=("${mss}")
    fnum_values+=("${fnum}")

    first=false
done

echo_line

mdfinal="${DIROUT}/compare-mem-rss__${FILE_NAME}__${FNUM_MAX}.md"

cat "${mddraft}" | column -t -s '|' -o '|' > "${mdfinal}"

if which glow &>/dev/null; then
    glow --width=${COLUMNS} --preserve-new-lines "${mdfinal}"
else
    cat "${mdfinal}"
fi

echo >&2

mss_max=$(printf "%s\n" "${mss_values[@]}" | sort -nr | head -n1)
mss_min=$(printf "%s\n" "${mss_values[@]}" | sort -n | head -n1)
mss_diff_max=$(printf "%s\n" "${mss_diff_values[@]}" | sort -nr | head -n1)
mss_diff_min=$(printf "%s\n" "${mss_diff_values[@]}" | sort -n | head -n1)
mss_diff_avg=$(echo -n "${mss_diff_values[@]}" | awk '{sum=0; for(i=1;i<=NF;i++) sum+=$i; print sum/NF}')
FILE_SZ_KB=$((FILE_SZ / 1024))
mss_diff_multiple_avg=$("${PYTHON}" -c "print('%.1f' % (${mss_diff_avg} / ${FILE_SZ_KB}))")
mss_diff_multiple_max=$("${PYTHON}" -c "print('%.1f' % (${mss_diff_max} / ${FILE_SZ_KB}))")
mss_diff_multiple_min=$("${PYTHON}" -c "print('%.1f' % (${mss_diff_min} / ${FILE_SZ_KB}))")

let mss_max_x=$((mss_max + 10000))
let mss_min_x=$((mss_min - 20000))

Data=$(for i in "${!mss_values[@]}"; do echo "${mss_values[$i]} ${fnum_values[$i]}"; done)

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
set xlabel "Max Resident Set Size (KB)"
set ylabel "File count"
set xtics ${xtics_step}
set ytics ${ytics_step}
set grid xtics
set grid ytics
set xrange [0:${mss_max_x}]
set yrange [0:$((${FNUM_MAX} + 1))]
set title "command: ${s4_command} ${FILE_NAME}…\n\nMax Resident Set Size (KB) per additional file of size ${FILE_SZ_KB} KB"
\$Data << EOD
$Data
EOD
plot \$Data with linespoints
EOF
)

(
    set -x
    echo "$GNUPLOT_TERMINAL" | gnuplot
)

echo >&2

(
    echo "Max RSS diff (KB)|${mss_diff_max}"
    echo "Min RSS diff (KB)|${mss_diff_min}"
    echo "Avg RSS diff (KB)|${mss_diff_avg}"
    echo "File Size (KB) |${FILE_SZ_KB}"
    echo "RSS diff multiple (avg)|${mss_diff_multiple_avg}"
    echo "RSS diff multiple (max)|${mss_diff_multiple_max}"
    echo "RSS diff multiple (min)|${mss_diff_multiple_min}"
) | column -t -s '|' -o ':' --table-columns='Info,Data' --table-right='Data' --table-noheadings

OUT_SVG="${DIROUT}/compare-mem-rss__${FILE_NAME}__${FNUM_MAX}.svg"

SVG_WIDTH=1280
SVG_HEIGHT=1080
FONT_SIZE_TEXT=12
FONT_SIZE_LABELS=8
if [[ $FNUM_MAX -lt 20 ]]; then
    SVG_WIDTH=768
    SVG_HEIGHT=480
    FONT_SIZE_LABELS=10
fi

GNUPLOT_SVG=$(cat <<EOF
set terminal svg size ${SVG_WIDTH}, ${SVG_HEIGHT} fname 'Arial,${FONT_SIZE_LABELS}'
set color
set key off
set output '${OUT_SVG}'
set xlabel "Max Resident Set Size (KB)" font 'Arial,${FONT_SIZE_TEXT}'
set ylabel "File count (${FILE_NAME})\n" font 'Arial,${FONT_SIZE_TEXT}'
set xtics ${xtics_step} font 'Arial,${FONT_SIZE_TEXT}'
set ytics ${ytics_step} font 'Arial,${FONT_SIZE_TEXT}'
set grid xtics
set grid ytics
set xrange [0:${mss_max_x}]
set yrange [0:$((${FNUM_MAX} + 1))]
set title "command: ${s4_command} ${FILE_NAME}…\n\nFile Name ${FILE_NAME}\nFile Size ${FILE_SZ_KB} KB\nMax max RSS diff ${mss_diff_max} KB (×${mss_diff_multiple_max} file size)\nAvg max RSS diff ${mss_diff_avg} KB (×${mss_diff_multiple_avg} file size)\nMin max RSS diff ${mss_diff_min} KB (×${mss_diff_multiple_min} file size)\n\n" \
    font 'Arial,${FONT_SIZE_TEXT}'
\$Data << EOD
$Data
EOD
plot \$Data with linespoints, \
     \$Data using 1:2:(sprintf("%d", \$1)) with labels point pt 7 offset char 3,-1 title "File Count, Max RSS"
EOF
)

(
    set -x
    echo "$GNUPLOT_SVG" | gnuplot
)

echo >&2
echo "SVG output written to: ${OUT_SVG}" >&2
