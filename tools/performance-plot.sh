#!/usr/bin/env bash
#
# gnuplot `s4` performance when processing increasing number of log files,
# specifically Max RSS, mean time, and disk I/O (syscr/rchar).
#

set -euo pipefail

readonly FILE_DEFAULT='./tools/compare-log-mergers/gen-5000-1-facesA.log'
declare -irg FILE_NUM_DEFAULT=100
readonly S4_PROGRAM_DEFAULT='./target/release/s4'
readonly PYTHON_DEFAULT='python3'

declare -irg FILE_RUNS_BLOCKSZ_DEFAULT=10
declare -irg BLOCKSZ_MIN_DEFAULT=4096
declare -irg BLOCKSZ_MAX_DEFAULT=131072
declare -irg BLOCKSZ_ALIGN_DEFAULT=4096

if [[ "${1-}" = "-h" || "${1-}" = "--help" || "${1-}" = "-?" ]]; then
    echo "\
Usage: ${0} [s4 args]

user may set environment variables:

  # RSS, time, and disk I/O testing

  FILE         - path to a log file to be used for testing
                 default: ${FILE_DEFAULT}
  FILE_NUM     - maximum number of files to test
                 default: ${FILE_NUM_DEFAULT}
  S4_PROGRAM   - path to the \`s4\` binary to test
                 default: ${S4_PROGRAM_DEFAULT}
  DIROUT       - output directory for markdown and SVG files
                 default: current directory
  PYTHON       - python3 interpreter
                 default: ${PYTHON_DEFAULT}

  Disk I/O is measured once per file-count via /proc/<pid>/io
  (syscr, rchar, syscw, write_bytes) before hyperfine runs.
  rchar counts bytes requested via read syscalls (includes page cache).
  Page cache is dropped when permitted (drop_caches); otherwise a
  warning is printed and measurement continues.
  Outputs include __diskio.md/csv and __diskio.svg (syscr + rchar).

  # Block Size testing

  FILE_RUNS_BLOCKSZ - number of files passed per s4 run
                      default: ${FILE_RUNS_BLOCKSZ_DEFAULT}
  BLOCKSZ_MIN   -     starting block size in bytes
                      default: ${BLOCKSZ_MIN_DEFAULT}
  BLOCKSZ_MAX   -     ending block size in bytes
                      default: ${BLOCKSZ_MAX_DEFAULT}
  BLOCKSZ_ALIGN -     step blocksz in bytes
                      default: ${BLOCKSZ_ALIGN_DEFAULT}

usage:
  FILE=path/to/some.log FILE_NUM=N ./tools/performance-plot.sh [s4 args]

example:
  DIROUT=/tmp/perf FILE=./tools/compare-log-mergers/gen-5000-1-facesA.log FILE_NUM=200 BLOCKSZ_ALIGN=128 S4_PROGRAM=./target/mimalloc/s4 ./tools/performance-plot.sh -cn

requires programs:
  hyperfine - measures runtime and memory usage
  jq        - parses hyperfine JSON output
  gnuplot   - creates ASCII and SVG graphs
  python3   - used for some math and string formatting
  xmllint   - prettify the SVG files
" >&2
    exit 0
fi

SCRIPTD=$(realpath "$(dirname -- "${0}")")

cd "$(dirname "${0}")/.."

readonly DIROUT=${DIROUT-"."}

declare -ir TIME_START=${SECONDS}

# check for hyperfine
HYPERFINE=$(which hyperfine) || {
    echo "ERROR: hyperfine not found in PATH" >&2
    echo "install:" >&2
    echo "    cargo install --locked hyperfine" >&2
    exit 1
}
readonly HYPERFINE
(set -x; "$HYPERFINE" --version)

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
PYTHON=${PYTHON-"${PYTHON_DEFAULT}"}
if ! which "${PYTHON}" &>/dev/null; then
    echo "ERROR: ${PYTHON} not found in PATH" >&2
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

readonly FILE=${FILE-"${FILE_DEFAULT}"}
readonly FILE_NAME=$(basename -- "${FILE}")
# no comma
readonly FILE_NAME_NOC=$(echo -n "${FILE_NAME}" | tr -s ',' '_')

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

# number of hyperfine runs
declare -ir HYPERFINE_RUNS=${HYPERFINE_RUNS-5}

# echo color escapes
readonly CLR_INFO="\033[1;32m"  # green
readonly CLR_RESET="\033[0m"

# pre-cache sudo password
sudo --validate -p "update the cached sudo credentials (enter sudo password): "

# the upcoming `git checkout` may remove some of the above log files
# so copy them to the temporary directory
TDIR_LOGS=/tmp/s4-performance-plot
mkdir -vp "${TDIR_LOGS}"

# print a line as wide as the terminal
function echo_line() {
    "${PYTHON}" -Bc "import sys; print('─' * ${COLUMNS:-100}, file=sys.stderr)"
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
    echo -n "${@}" | \
        sed \
            -e 's/&/\&amp;/g' \
            -e 's/</\&lt;/g' \
            -e 's/>/\&gt;/g' \
            -e "s/'/\&apos;/g" \
            -e 's/"/\&quot;/g'
}

# escape regex special characters
function regex_escape() {
    echo -n "${@}" | "$PYTHON" -c 'import re, sys; print(re.escape(sys.stdin.read().rstrip()))'
}

# print $1 seconds as HH:MM:SS
function seconds_to_hms() {
    declare -ir seconds=${1}
    declare -ir h=$((seconds / 3600))
    declare -ir m=$(((seconds % 3600) / 60))
    declare -ir s=$((seconds % 60))
    printf "%02d:%02d:%02d" "${h}" "${m}" "${s}"
}

function gnuplot_svg_title_replace() {
    local file="${1}"
    shift
    # replace the non-descriptive '<title>Gnuplot</title>' with something interesting
    sed -i -e "s|$(regex_escape "<title>Gnuplot</title>")|$(regex_escape "<title>$(xml_escape "${@}")</title>")|" -- "${file}"
}

function xml_format() {
    "${SCRIPTD}/xmllint.sh" "${@}"
}

# read one key from a /proc/<pid>/io style file: "key: value"
#
# example:
#
#     $ cat /proc/self/io
#     rchar: 4092
#     wchar: 0
#     syscr: 9
#     syscw: 0
#     read_bytes: 0
#     write_bytes: 0
#     cancelled_write_bytes: 0
#
# usage: io_field <io_file> <key>
function io_field() {
    declare -r iofile=${1}
    declare -r key=${2}
    local val
    val=$(set -euo pipefail; grep -m1 -Ee "^${key}:" "${iofile}" | cut -f2 -d':' | tr -d '[:space:]')
    if [[ -z "${val}" || ! "${val}" =~ ^[0-9]+$ ]]; then
        echo "ERROR: missing or non-integer '${key}' in '${iofile}'" >&2
        if [[ -f "${iofile}" ]]; then
            cat "${iofile}" >&2 || true
        fi
        return 1
    fi
    echo -n "${val}"
}

# Attempt to drop page cache before the I/O measurement run.
# Sets global CACHES_DROPPED to true or false (shell builtins).
# Note: plotted read bytes use rchar (syscall bytes, includes cache).
function drop_caches_try() {
    declare -r drop_path=/proc/sys/vm/drop_caches
    if [[ -w "${drop_path}" ]]; then
        echo "${PS4}sync" >&2
        sync
        echo "${PS4}echo 3 > ${drop_path}" >&2
        echo 3 > "${drop_path}"
        CACHES_DROPPED=true
        return 0
    fi
    if sudo -n true 2>/dev/null; then
        sudo -n sh -c 'set -x; sync; echo 3 > /proc/sys/vm/drop_caches'
        CACHES_DROPPED=true
        return 0
    fi
    if ! ${CACHES_DROP_WARNED}; then
        echo "WARNING: cannot write ${drop_path}; disk I/O may be dominated by page cache hits" >&2
        CACHES_DROP_WARNED=true
    fi
    CACHES_DROPPED=false
    return 0
}

# Atomically copy /proc/<pid>/io into $2 if readable.
# Important: never `cat ... > iofile` on failure — the shell truncates the
# destination before cat runs, which would wipe a good earlier snapshot.
# usage: copy_proc_io <pid> <io_file>
function copy_proc_io() {
    declare -ir pid=${1}
    declare -r iofile=${2}
    declare -r tmp="${iofile}.tmp.$$"
    declare -r src="/proc/${pid}/io"
    if [[ ! -r "${src}" ]]; then
        return 1
    fi
    if cat "${src}" > "${tmp}" 2>/dev/null; then
        mv -f "${tmp}" "${iofile}"
        return 0
    fi
    rm -f "${tmp}"
    return 1
}

# Run a shell-escaped command in the background and copy /proc/<pid>/io into
# $1 until exit.
# Copies cumulative kernel counters (not rate estimates).
# Last successful copy is the lifetime total.
# usage: cmd_io <io_file> <command_string>
function cmd_io() {
    declare -r iofile=${1}
    declare -r cmd=${2}
    declare -i pid
    declare -i status=0
    : > "${iofile}"
    echo "${PS4-}${cmd}" >&2
    # `exec` so pid is s4, not a wrapper shell
    # shellcheck disable=SC2086
    eval "exec ${cmd}" 1>/dev/null &
    pid=$!
    # poll until the process exits
    while kill -0 "${pid}" 2>/dev/null; do
        copy_proc_io "${pid}" "${iofile}" || true
        sleep 0.010
    done
    # /proc may remain until wait reaps; try one last copy
    copy_proc_io "${pid}" "${iofile}" || true
    status=0
    wait "${pid}" || status=$?
    if [[ ! -s "${iofile}" ]]; then
        echo "ERROR: empty I/O snapshot '${iofile}' for command: ${cmd}" >&2
        return 1
    fi
    return ${status}
}

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

readonly S4_PROGRAM=${S4_PROGRAM-"${S4_PROGRAM_DEFAULT}"}
# very presumptive that the profile name will be the 3rd path component
# e.g. ./target/release/s4 -> release
#      ./target/debug/s4   -> debug
BUILD_PROFILE=$(echo "${S4_PROGRAM}" | cut -f3 -d'/')
if [[ -z "${BUILD_PROFILE}" ]]; then
    BUILD_PROFILE="unknown"
fi
readonly BUILD_PROFILE

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
# XXX: `S4_BLOCKSZ` must match `BLOCKSZ_DEF` defined in `blockreader.rs`
declare -i S4_BLOCKSZ=${S4_BLOCKSZ-65535}
declare -ir S4_BLOCKSZ_KB=$((S4_BLOCKSZ / 1024))
declare -ir FILE_SZ_BLOCKS=$((FILE_SZ / 65536 + 1))

declare -ir FILE_NUM=${FILE_NUM-"${FILE_NUM_DEFAULT}"}
if [[ ${FILE_NUM} -lt 1 ]]; then
    echo "FILE_NUM must be greater than 0, got ${FILE_NUM}" >&2
    exit 1
fi

readonly MD_FINAL="${DIROUT}/performance-plot__${FILE_NAME}__${FILE_NUM}__rss_time_data.md"
readonly CSV_FINAL="${DIROUT}/performance-plot__${FILE_NAME}__${FILE_NUM}__rss_time_data.csv"
readonly MD_FINAL_DISKIO="${DIROUT}/performance-plot__${FILE_NAME}__${FILE_NUM}__diskio.md"
readonly CSV_FINAL_DISKIO="${DIROUT}/performance-plot__${FILE_NAME}__${FILE_NUM}__diskio.csv"

if [[ -f "${MD_FINAL}" ]]; then
    echo "Final output already exists, skipping. '${MD_FINAL}'" >&2
    exit 0
fi
if [[ -f "${CSV_FINAL}" ]]; then
    echo "Final output already exists, skipping. '${CSV_FINAL}'" >&2
    exit 0
fi

readonly tmpD=$(mktemp -d -t "tmp-s4-performance-plot_XXXXX")
readonly IO_TXT="${tmpD}/io.txt"

function exit_() {
    rm -rf "${tmpD}"
}

trap exit_ EXIT

mkdir -p "${DIROUT}"

# whether page cache was dropped on the most recent drop_caches_try call
CACHES_DROPPED=false
CACHES_DROP_WARNED=false
# sticky flag if any fnum measurement dropped caches successfully
CACHES_DROPPED_ANY=false

# -----------------------------------------------------------------------------

#
# start the markdown draft file
#

readonly MD_DRAFT="${tmpD}/performance-plot-draft-time-rss.md"
readonly CSV_DRAFT="${tmpD}/performance-plot_${FILE_NAME}_${FILE_NUM}.csv"
readonly MD_DRAFT_DISKIO="${tmpD}/performance-plot-draft-diskio.md"
readonly CSV_DRAFT_DISKIO="${tmpD}/performance-plot-draft-diskio.csv"

# markdown table header
echo "\
|Files       |Profile|Mean (ms)|Min (ms)|Max (ms)|Diff (ms)|Max RSS (KB)|Max RSS (KB) diff|CPU %|
|:---        |:---   |---:     |---:    |---:    |---:     |---:        |---:             |---: |" > "${MD_DRAFT}"
# CSV header
echo "#File,Files,Profile,Mean (ms),Min (ms),Max (ms),Diff (ms),Max RSS (KB),Max RSS (KB) diff,CPU %" > "${CSV_DRAFT}"

# disk I/O markdown table header (kernel /proc/<pid>/io counters)
echo "\
|Files       |Profile|syscr (read calls)|rchar (read bytes)|syscw (write calls)|write_bytes|
|:---        |:---   |---:              |---:              |---:               |---:       |" > "${MD_DRAFT_DISKIO}"
echo "#File,Files,Profile,syscr,rchar,syscw,write_bytes" > "${CSV_DRAFT_DISKIO}"

#
# run the tests for each file count
#

first=true
declare -a time_values=()
declare -a time_diff_values=()
declare -a mss_values=()
declare -a fnum_values=()
declare -a mss_diff_values=()
declare -a diskio_syscr_values=()
declare -a diskio_rchar_values=()

# must pass command as a single shell-escaped string to `hyperfine`
declare s4_command=$(printf "%q" "${S4_PROGRAM}")
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

    #
    # run s4 to capture disk I/O from /proc/<pid>/io
    #
    echo -e "${CLR_INFO}Measuring disk I/O for ${fnum} files via /proc/<pid>/io${CLR_RESET}" >&2
    drop_caches_try
    if ${CACHES_DROPPED}; then
        CACHES_DROPPED_ANY=true
    fi
    cmd_io "${IO_TXT}" "${s4_command} ${current_files[*]}"
    declare -i diskio_syscr
    declare -i diskio_rchar
    declare -i diskio_syscw
    declare -i diskio_write_bytes
    diskio_syscr=$(io_field "${IO_TXT}" syscr)
    diskio_rchar=$(io_field "${IO_TXT}" rchar)
    diskio_syscw=$(io_field "${IO_TXT}" syscw)
    diskio_write_bytes=$(io_field "${IO_TXT}" write_bytes)
    echo -e "${CLR_INFO}disk I/O: syscr=${diskio_syscr} rchar=${diskio_rchar} syscw=${diskio_syscw} write_bytes=${diskio_write_bytes} (caches_dropped=${CACHES_DROPPED})${CLR_RESET}" >&2
    echo >&2

    # disk I/O markdown / CSV rows
    echo "|${fnum}|${BUILD_PROFILE}|${diskio_syscr}|${diskio_rchar}|${diskio_syscw}|${diskio_write_bytes}|" >> "${MD_DRAFT_DISKIO}"
    echo "${FILE_NAME_NOC},${fnum},${BUILD_PROFILE},${diskio_syscr},${diskio_rchar},${diskio_syscw},${diskio_write_bytes}" >> "${CSV_DRAFT_DISKIO}"

    diskio_syscr_values+=("${diskio_syscr}")
    diskio_rchar_values+=("${diskio_rchar}")

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

    declare -i time_since_start=$((${SECONDS} - TIME_START))
    time_since_start_hms=$(seconds_to_hms "${time_since_start}")
    echo >&2
    echo -e "${CLR_INFO}For ${HYPERFINE_RUNS} runs of ${fnum} files: time ${proc_time_diff} ms, Max RSS ${mss_KB} KB, syscr ${diskio_syscr}, rchar ${diskio_rchar}, syscw ${diskio_syscw}, write_bytes ${diskio_write_bytes} (script running for ${time_since_start_hms})${CLR_RESET}" >&2

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

# disk I/O finals
cat "${MD_DRAFT_DISKIO}" | column -t -s '|' -o '|' > "${MD_FINAL_DISKIO}"
cp -av "${CSV_DRAFT_DISKIO}" "${CSV_FINAL_DISKIO}"

export PATH="${PATH}:${HOME}/go/bin"  # for glow
if which glow &>/dev/null; then
    glow --width=${COLUMNS} --preserve-new-lines "${MD_FINAL}"
    echo >&2
    glow --width=${COLUMNS} --preserve-new-lines "${MD_FINAL_DISKIO}"
else
    cat "${MD_FINAL}"
    echo >&2
    cat "${MD_FINAL_DISKIO}"
fi

echo >&2

#
# gnuplot an ASCII graph for file count (Y) vs max RSS (X)
#

readonly gnuplot_vertical_line_x0='set arrow from 0, graph 0 to 0, graph 1 nohead'

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

#
# gnuplot create SVG for file count (Y) vs max RSS (X)
#

readonly OUT_SVG_RSS="${DIROUT}/performance-plot__${FILE_NAME}__${FILE_NUM}__rss.svg"

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
#       cannot get this to work after many attempts

(
    set -x
    echo "$GNUPLOT_SVG" | "$GNUPLOT"
)

gnuplot_svg_title_replace "${OUT_SVG_RSS}" "Max RSS (KB) per N file for '${FILE_NAME}'"
xml_format "${OUT_SVG_RSS}"

echo >&2

#
# gnuplot create SVG for file count (X) vs process time (Y)
#

readonly OUT_SVG_TIME="${DIROUT}/performance-plot__${FILE_NAME}__${FILE_NUM}__time.svg"

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

# -----------------------------------------------------------------------------

#
# gnuplot create SVG for file count (Y) vs disk read count / read bytes (X)
# two stacked panels in one multiplot SVG
#

readonly OUT_SVG_DISKIO_READ="${DIROUT}/performance-plot__${FILE_NAME}__${FILE_NUM}__diskio.svg"

# sanity check disk I/O series length matches file counts
if [[ ${#diskio_syscr_values[@]} -ne ${#fnum_values[@]} ]]; then
    echo "Mismatched diskio_syscr_values fnum_values; ${#diskio_syscr_values[@]} ${#fnum_values[@]}" >&2
    exit 1
fi
if [[ ${#diskio_rchar_values[@]} -ne ${#fnum_values[@]} ]]; then
    echo "Mismatched diskio_rchar_values fnum_values; ${#diskio_rchar_values[@]} ${#fnum_values[@]}" >&2
    exit 1
fi

DataDiskReadCount=$(for i in "${!diskio_syscr_values[@]}"; do echo "${diskio_syscr_values[$i]} ${fnum_values[$i]}"; done)
DataDiskReadBytes=$(for i in "${!diskio_rchar_values[@]}"; do echo "${diskio_rchar_values[$i]} ${fnum_values[$i]}"; done)

# per-file rates for each run: total / file count (integer division)
declare -a diskio_syscr_per_file_values=()
declare -a diskio_rchar_per_file_values=()
for i in "${!diskio_syscr_values[@]}"; do
    declare -i fnum_i=${fnum_values[$i]}
    if [[ ${fnum_i} -lt 1 ]]; then
        echo "ERROR: invalid file count '${fnum_i}' at index ${i}" >&2
        exit 1
    fi
    diskio_syscr_per_file_values+=("$((${diskio_syscr_values[$i]} / fnum_i))")
    diskio_rchar_per_file_values+=("$((${diskio_rchar_values[$i]} / fnum_i))")
done

diskio_syscr_per_file_max=$(max "${diskio_syscr_per_file_values[@]}")
diskio_syscr_per_file_min=$(min "${diskio_syscr_per_file_values[@]}")
diskio_syscr_per_file_avg=$(avg "${diskio_syscr_per_file_values[@]}")
diskio_rchar_per_file_max=$(max "${diskio_rchar_per_file_values[@]}")
diskio_rchar_per_file_min=$(min "${diskio_rchar_per_file_values[@]}")
diskio_rchar_per_file_avg=$(avg "${diskio_rchar_per_file_values[@]}")

# ratios vs file size and S4 block size — same style as RSS SVG
# for compressed inputs, prefer uncompressed size as the logical file size
declare -i DISKIO_FILE_SZ_BYTES=${FILE_SZ}
declare -i DISKIO_FILE_SZ_BLOCKS=${FILE_SZ_BLOCKS}
if [[ ${FILE_SZ_UNCOMPRESSED} -gt 0 ]]; then
    DISKIO_FILE_SZ_BYTES=${FILE_SZ_UNCOMPRESSED}
    DISKIO_FILE_SZ_BLOCKS=${FILE_SZ_UNCOMPRESSED_BLOCKS}
fi

# rchar is bytes: × file size, × Blocks (bytes / BLOCKSZ) — mirrors RSS KB ratios
diskio_rchar_per_file_multiple_max=$("${PYTHON}" -c "print('%.1f' % (${diskio_rchar_per_file_max} / max(${DISKIO_FILE_SZ_BYTES}, 1)))")
diskio_rchar_per_file_multiple_min=$("${PYTHON}" -c "print('%.1f' % (${diskio_rchar_per_file_min} / max(${DISKIO_FILE_SZ_BYTES}, 1)))")
diskio_rchar_per_file_multiple_avg=$("${PYTHON}" -c "print('%.1f' % (${diskio_rchar_per_file_avg} / max(${DISKIO_FILE_SZ_BYTES}, 1)))")
diskio_rchar_per_file_blocksz_multiple_max=$("${PYTHON}" -c "print('%.1f' % (${diskio_rchar_per_file_max} / max(${S4_BLOCKSZ}, 1)))")
diskio_rchar_per_file_blocksz_multiple_min=$("${PYTHON}" -c "print('%.1f' % (${diskio_rchar_per_file_min} / max(${S4_BLOCKSZ}, 1)))")
diskio_rchar_per_file_blocksz_multiple_avg=$("${PYTHON}" -c "print('%.1f' % (${diskio_rchar_per_file_avg} / max(${S4_BLOCKSZ}, 1)))")

# syscr is a call count: × file-size-in-blocks (calls per logical file block)
diskio_syscr_per_file_multiple_max=$("${PYTHON}" -c "print('%.1f' % (${diskio_syscr_per_file_max} / max(${DISKIO_FILE_SZ_BLOCKS}, 1)))")
diskio_syscr_per_file_multiple_min=$("${PYTHON}" -c "print('%.1f' % (${diskio_syscr_per_file_min} / max(${DISKIO_FILE_SZ_BLOCKS}, 1)))")
diskio_syscr_per_file_multiple_avg=$("${PYTHON}" -c "print('%.1f' % (${diskio_syscr_per_file_avg} / max(${DISKIO_FILE_SZ_BLOCKS}, 1)))")

declare -i diskio_syscr_max
diskio_syscr_max=$(max "${diskio_syscr_values[@]}")
declare -i diskio_rchar_max
diskio_rchar_max=$(max "${diskio_rchar_values[@]}")

declare -i x_range_max_syscr=$((diskio_syscr_max + (diskio_syscr_max / 10) + 1))
declare -i x_range_max_rchar=$((diskio_rchar_max + (diskio_rchar_max / 10) + 1))

declare -i xtics_step_syscr=0
if [[ ${x_range_max_syscr} -lt 100 ]]; then
    xtics_step_syscr=1
elif [[ ${x_range_max_syscr} -lt 1000 ]]; then
    xtics_step_syscr=10
elif [[ ${x_range_max_syscr} -lt 10000 ]]; then
    xtics_step_syscr=100
elif [[ ${x_range_max_syscr} -lt 100000 ]]; then
    xtics_step_syscr=1000
else
    xtics_step_syscr=10000
fi

declare -i xtics_step_rchar=0
if [[ ${x_range_max_rchar} -lt 1000 ]]; then
    xtics_step_rchar=100
elif [[ ${x_range_max_rchar} -lt 100000 ]]; then
    xtics_step_rchar=10000
elif [[ ${x_range_max_rchar} -lt 1000000 ]]; then
    xtics_step_rchar=100000
elif [[ ${x_range_max_rchar} -lt 10000000 ]]; then
    xtics_step_rchar=1000000
else
    xtics_step_rchar=10000000
fi

declare -i ytics_step_diskio=0
if [[ $FILE_NUM -le 50 ]]; then
    ytics_step_diskio=1
elif [[ $FILE_NUM -le 100 ]]; then
    ytics_step_diskio=2
elif [[ $FILE_NUM -le 200 ]]; then
    ytics_step_diskio=4
else
    ytics_step_diskio=10
fi

declare -i SVG_HEIGHT_DISKIO=$((SVG_HEIGHT + SVG_HEIGHT / 2))
declare -i SVG_WIDTH_DISKIO=${SVG_WIDTH}

if ${CACHES_DROPPED_ANY}; then
    CACHES_DROP_MESG="Page cache: dropped when permitted (at least one fnum)"
else
    CACHES_DROP_MESG="Page cache: not dropped (I/O may include cache hits)"
fi

GNUPLOT_SVG=$(cat <<EOF
set terminal svg size ${SVG_WIDTH_DISKIO}, ${SVG_HEIGHT_DISKIO} fname "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}"
set encoding utf8
set color
set key off
set output "${OUT_SVG_DISKIO_READ}"
set multiplot layout 2,1 title "Command: ${s4_command} ${FILE_NAME} …\nBuild profile: ${BUILD_PROFILE}, Version: ${Version} (git tag ${GitTagLast}), MSRV: ${Msrv}\nAllocator: ${Allocator}, Platform: ${Platform}, Optimization Level: ${OptimizationLevel}\nRun on: ${OsName}, CPU: ${CpuModel} (${CpuCores} cores), RAM: ${RamTotalMB} MB\n\nDisk I/O from /proc/<pid>/io (one run per file count)\n${CACHES_DROP_MESG}\n\nFile: ${FILE}\nBlock Size: ${S4_BLOCKSZ_KB} KB (${S4_BLOCKSZ} Bytes)\n${FILE_SZ_MESG}\n\nMax Read Count (syscr) per 1 File: ${diskio_syscr_per_file_max} (×${diskio_syscr_per_file_multiple_max} file blocks)\nAvg Read Count (syscr) per 1 File: ${diskio_syscr_per_file_avg} (×${diskio_syscr_per_file_multiple_avg} file blocks)\nMin Read Count (syscr) per 1 File: ${diskio_syscr_per_file_min} (×${diskio_syscr_per_file_multiple_min} file blocks)\nMax Read Bytes (rchar) per 1 File: ${diskio_rchar_per_file_max} (×${diskio_rchar_per_file_multiple_max} file size) (×${diskio_rchar_per_file_blocksz_multiple_max} Blocks)\nAvg Read Bytes (rchar) per 1 File: ${diskio_rchar_per_file_avg} (×${diskio_rchar_per_file_multiple_avg} file size) (×${diskio_rchar_per_file_blocksz_multiple_avg} Blocks)\nMin Read Bytes (rchar) per 1 File: ${diskio_rchar_per_file_min} (×${diskio_rchar_per_file_multiple_min} file size) (×${diskio_rchar_per_file_blocksz_multiple_min} Blocks)\n" \
    font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" \
    noenhanced

set format "%.0f"
set ylabel "File count ${FILE_NUM}\n" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" noenhanced
set ytics ${ytics_step_diskio} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set grid xtics
set grid ytics
set yrange [0:$((${FILE_NUM} + 1))]

# panel 1: syscr (read syscall count)
set xlabel "Disk Read Count (syscr)" textcolor rgbcolor "${COLOR_1}" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" enhanced
set xtics ${xtics_step_syscr} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set xrange [0:${x_range_max_syscr}]
\$DataDiskReadCount << EOD
$DataDiskReadCount
EOD
plot \$DataDiskReadCount with lines linecolor rgbcolor "${COLOR_1}" title "syscr", \
     \$DataDiskReadCount every 1 using 1:2:(sprintf("%d", \$1)) with labels point pointtype 7 pointsize 0.5 offset char 5,-0.5 font "${FONT_NAME_POINTS},${FONT_SIZE_LABELS}" title "syscr"

# panel 2: rchar (bytes requested via read syscalls, includes page cache)
set xlabel "Disk Read Bytes (rchar)" textcolor rgbcolor "${COLOR_2}" font "${FONT_NAME_OUTER},${FONT_SIZE_OUTER}" enhanced
set xtics ${xtics_step_rchar} font "${FONT_NAME_TICS},${FONT_SIZE_TICS}" noenhanced
set xrange [0:${x_range_max_rchar}]
\$DataDiskReadBytes << EOD
$DataDiskReadBytes
EOD
plot \$DataDiskReadBytes with lines linecolor rgbcolor "${COLOR_2}" title "rchar", \
     \$DataDiskReadBytes every 1 using 1:2:(sprintf("%d", \$1)) with labels point pointtype 7 pointsize 0.5 offset char 5,-0.5 font "${FONT_NAME_POINTS},${FONT_SIZE_LABELS}" title "rchar"

unset multiplot
EOF
)

(
    set -x
    echo "$GNUPLOT_SVG" | "$GNUPLOT"
)

gnuplot_svg_title_replace "${OUT_SVG_DISKIO_READ}" "Disk reads (syscr, rchar) per N file for '${FILE_NAME}'"
xml_format "${OUT_SVG_DISKIO_READ}"

echo >&2

# -----------------------------------------------------------------------------

#
# run the tests for each Block Size
#
# Process wall clock run time on X
# Block Size on Y
#

echo_line

declare -ir FILE_RUNS_BLOCKSZ=${FILE_RUNS_BLOCKSZ-${FILE_RUNS_BLOCKSZ_DEFAULT}}
declare -ir BLOCKSZ_ALIGN=${BLOCKSZ_ALIGN-${BLOCKSZ_ALIGN_DEFAULT}}
declare -ir BLOCKSZ_MIN=$(((${BLOCKSZ_MIN-${BLOCKSZ_MIN_DEFAULT}} / ${BLOCKSZ_ALIGN} + 1) * ${BLOCKSZ_ALIGN}))
declare -ir BLOCKSZ_MAX=$(((${BLOCKSZ_MAX-${BLOCKSZ_MAX_DEFAULT}} / ${BLOCKSZ_ALIGN}) * ${BLOCKSZ_ALIGN}))

declare -a current_files=()
for ((i=0; i < ${FILE_RUNS_BLOCKSZ}; i++)); do
    # XXX: presuming there are no spaces in the file name
    current_files+=("${FILE}")
done

readonly MD_DRAFT_BSZ="${tmpD}/performance-plot-draft-blocksz.md"
readonly CSV_DRAFT_BSZ="${tmpD}/performance-plot-draft-blocksz.csv"

readonly MD_FINAL_BSZ="${DIROUT}/performance-plot__${FILE_NAME}__blocksz_${BLOCKSZ_ALIGN}.md"
readonly CSV_FINAL_BSZ="${DIROUT}/performance-plot__${FILE_NAME}__blocksz_${BLOCKSZ_ALIGN}.csv"

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

# this can take a long time: let the user know what block sizes will be tested
echo -e "${CLR_INFO}Testing block sizes from ${BLOCKSZ_MIN} to ${BLOCKSZ_MAX} in increments of ${BLOCKSZ_ALIGN}; pass ${FILE_RUNS_BLOCKSZ} files per run${CLR_RESET}" >&2
for blocksz in $(seq ${BLOCKSZ_MIN} ${BLOCKSZ_ALIGN} ${BLOCKSZ_MAX}); do
    echo -e "${CLR_INFO}${blocksz}${CLR_RESET}" >&2
done
echo >&2

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

    declare -i time_since_start=$((${SECONDS} - TIME_START))
    time_since_start_hms=$(seconds_to_hms "${time_since_start}")

    echo >&2
    echo -e "${CLR_INFO}For ${HYPERFINE_RUNS} runs with block size ${blocksz}: time ${proc_time_diff} ms, Max RSS ${mss_KB} KB (script running for ${time_since_start_hms})${CLR_RESET}" >&2

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

readonly OUT_SVG_BLOCKSZ="${DIROUT}/performance-plot__${FILE_NAME}__blocksz_${BLOCKSZ_ALIGN}.svg"

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
set title "Command: S4_BLOCKSZ=(${BLOCKSZ_MIN}..${BLOCKSZ_MAX} step by ${BLOCKSZ_ALIGN}) ${s4_command} ${FILE_NAME} …\nBuild profile: ${BUILD_PROFILE}, Version: ${Version} (git tag ${GitTagLast}), MSRV: ${Msrv}\nAllocator: ${Allocator}, Platform: ${Platform}, Optimization Level: ${OptimizationLevel}\nRun on: ${OsName}, CPU: ${CpuModel}, Cores: ${CpuCores}, RAM: ${RamTotalMB} MB\n\nHyperfine runs per data point: ${HYPERFINE_RUNS}\n\nTime Difference per 1 File Max ${time_diff_max} ms\nTime Difference per 1 File Avg ${time_diff_avg} ms\nTime Difference per 1 File Min ${time_diff_min} ms" \
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

echo -e "
SVG RSS output written to: ${CLR_INFO}${OUT_SVG_RSS}${CLR_RESET}
SVG TIME output written to: ${CLR_INFO}${OUT_SVG_TIME}${CLR_RESET}
SVG DISK READS output written to: ${CLR_INFO}${OUT_SVG_DISKIO_READ}${CLR_RESET}
Markdown written to: ${CLR_INFO}${MD_FINAL}${CLR_RESET}
CSV written to: ${CLR_INFO}${CSV_FINAL}${CLR_RESET}

Disk I/O markdown written to: ${CLR_INFO}${MD_FINAL_DISKIO}${CLR_RESET}
Disk I/O CSV written to: ${CLR_INFO}${CSV_FINAL_DISKIO}${CLR_RESET}

SVG BLOCKSZ output written to: ${CLR_INFO}${OUT_SVG_BLOCKSZ}${CLR_RESET}
Markdown written to: ${CLR_INFO}${MD_FINAL_BSZ}${CLR_RESET}
CSV written to: ${CLR_INFO}${CSV_FINAL_BSZ}${CLR_RESET}
" >&2
