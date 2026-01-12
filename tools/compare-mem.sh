#!/usr/bin/env bash
#

set -euo pipefail

cd "$(dirname "${0}")/.."

OUTDIR=${OUTDIR-"."}

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

declare -ar FILE='./tools/compare-log-mergers/gen-5000-1-facesA.log'
declare -ir FILE_SZ=$(stat --printf='%s' "${FILE}")

# the upcoming `git checkout` may remove some of the above log files
# so copy them to the temporary directory
TDIR_LOGS=/tmp/s4-compare-mem
mkdir -vp "${TDIR_LOGS}"

function echo_line() {
    python -Bc "print('─' * ${COLUMNS:-100})"
    echo
}

function file_size() {
    stat --printf='%s' "${1}"
}

function file_isempty() {
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

S4_PROGRAM=${S4_PROGRAM-"./target/release/s4"}
build_profile=${build_profile-"release"}

(set -x; "${S4_PROGRAM}" --version)

# datetime range for s4
declare -r after_dt="2000-01-01T00:20:00"
declare -r befor_dt="2000-01-01T00:50:00"

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
declare -ir fnum_max=100
declare -r s4_command="'${S4_PROGRAM}' -a='${after_dt}' -b='${befor_dt}' --color=never"
# markdown table rows
for fnum in $(seq 1 ${fnum_max}); do
    echo_line

    echo "Testing '${S4_PROGRAM}' with ${fnum} file(s)" >&2
    echo

    json="${tmpD}/${fnum}.json"

    declare -a current_files=()
    for ((i=0; i < fnum; i++)); do
        current_files+=("${FILE}")
    done
    (
        set -x
        ${hyperfine} \
            --warmup=2 \
            --style=color \
            --time-unit=millisecond \
            --runs=${HRUNS} \
            --export-json "${json}" \
            -N \
            --command-name "s4 ${fnum} files" \
            -- \
                "${s4_command} ${current_files[*]}"
    )
    echo

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
    echo "|${fnum}|${build_profile}|${mean} ± ${stddev}|${min}|${max}|${mss}|${mss_diff}|${cpup}|" >> "${mddraft}"

    mss_values+=("${mss}")
    fnum_values+=("${fnum}")

    first=false
done

echo_line

mdfinal="/tmp/compare-mem.md"

cat "${mddraft}" | column -t -s '|' -o '|' > "${mdfinal}"

(set -x; cat "${mdfinal}")

if which glow &>/dev/null; then
    glow --width=${COLUMNS} --preserve-new-lines "${mdfinal}"
else
    echo "install 'glow' for pretty markdown viewing" >&2
    echo "    go install github.com/charmbracelet/glow/v2@latest" >&2
fi

echo >&2

mss_max=$(printf "%s\n" "${mss_values[@]}" | sort -nr | head -n1)
mss_min=$(printf "%s\n" "${mss_values[@]}" | sort -n | head -n1)
mss_diff_max=$(printf "%s\n" "${mss_diff_values[@]}" | sort -nr | head -n1)
mss_diff_min=$(printf "%s\n" "${mss_diff_values[@]}" | sort -n | head -n1)
mss_diff_avg=$(echo -n "${mss_diff_values[@]}" | awk '{sum=0; for(i=1;i<=NF;i++) sum+=$i; print sum/NF}')
FILE_SZ_KB=$((FILE_SZ / 1024))
mss_diff_multiple=$("${PYTHON}" -c "print('%.1f' % (${mss_diff_avg} / ${FILE_SZ_KB}))")

let mss_max_x=$((mss_max + 20000))
let mss_min_x=$((mss_min - 20000))

echo "${PS4}gnuplot terminal output:" >&2
gnuplot <<EOF
set terminal dumb size $COLUMNS, 30
set key off
set xlabel "MSS (KB)"
set ylabel "File count"
set ytics 2
set grid xtics
set xrange [${mss_min_x}:${mss_max_x}]
set title "command: ${s4_command} …\n\nMSS (KB) per additional file of size ${FILE_SZ_KB} KB"
plot '-' with linespoints
$(paste <(printf "%s\n" "${mss_values[@]}") <(printf "%s\n" "${fnum_values[@]}"))
end
EOF

echo >&2

(
    #echo "MSS diffs (KB)|${mss_diff_values[*]}"
    echo "Max MSS diff (KB)|${mss_diff_max}"
    echo "Min MSS diff (KB)|${mss_diff_min}"
    echo "Avg MSS diff (KB)|${mss_diff_avg}"
    echo "File Size (KB) |${FILE_SZ_KB}"
    echo "MSS diff multiple|${mss_diff_multiple}"
) | column -t -s '|' -o ':' --table-columns='Info,Data' --table-right='Data' --table-noheadings

OUT_SVG="${OUTDIR}/compare-mem-mss.svg"
echo >&2
gnuplot <<EOF
set terminal svg size 1152, 864 fname 'Arial,12'
set key off
set output '${OUT_SVG}'
set xlabel "MSS (KB)"
set ylabel "File count"
set ytics 2
set grid xtics
set xrange [${mss_min_x}:${mss_max_x}]
set title "command: ${s4_command} …\n\nMSS (KB) per additional file of size ${FILE_SZ_KB} KB"
plot '-' with linespoints
$(paste <(printf "%s\n" "${mss_values[@]}") <(printf "%s\n" "${fnum_values[@]}"))
end
EOF

echo "SVG output written to: ${OUT_SVG}" >&2
