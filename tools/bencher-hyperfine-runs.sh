#!/usr/bin/env bash
#
# wrapper to run hyperfine benchmarks on various log files with different
# Bencher Metrics and Variants.
# For running locally and in CI.
#
# Environment variables:
#   PROGRAM           s4 executable (default: ./target/release/s4)
#   HYPERFINE_INSTALL Hyperfine executable (default: hyperfine on PATH)
#   DIROUT            Output directory
#   HRUNS             Measured runs, at least 2 (default: 5)
#   HWARMUP           Warmup runs, at least 0 (default: 2)
#   BENCHER_REPORT    Upload report path (default: DIROUT/bencher-report.json)
#   BENCHER_TESTBED   Testbed name (default: Linux OS, CPU model, and core count)
#   UPLOAD_REPORT     If set to 1, automatically send the report after running
#                     benchmarks.
#
# Extra command-line arguments are forwarded to hyperfine.
#
# Writes Hyperfine JSON/Markdown pairs, bencher-results.json (BMF metrics), and
# bencher-report.json (the report envelope required by POST /reports).
# Reports time_wall_clock in nanoseconds and memory_peak_resident in bytes.
# Report hash comes from PROGRAM --version; both report times use that commit's
# author date, not benchmark runtime.
# Manually upload with `tools/bencher-report-upload.sh` to configure Bencher
# measure units.
#
# Example:
#
#   PROGRAM=./target/release_Opt0/s4 HRUNS=2 HWARMUP=0 DIROUT=/tmp/benches ./.github/tools/hyperfine-runs.sh
#

set -euo pipefail

# prepare as much as possible before running hyperfine to elicit problems, e.g.
# missing programs, forgotten environment variables, etc. before starting
# the hyperfine runs, as they take tens of minutes

# FILE_NUM Variant is applied to all files.
# BLOCKSZ Variant is only applied to files that are processed with a BlockReader.
readonly FILE_NUM="5,50"
readonly FILE_NUM_ARRAY="[5, 50]"
readonly BLOCKSZ_TEST="4096,8192,65535,131071"
declare -ar FILES_BLOCKS=(
    "./logs/Darwin25/var-log/powermanagement/2026.05.30.asl"
    "./logs/NetBSD9.3/Xorg.0.log.old|${BLOCKSZ_TEST}"
    "./logs/OpenBSD7.4/x86_64/utmp|${BLOCKSZ_TEST}"
    "./logs/other/tests/gen-99999-1-Motley_Crue.log|${BLOCKSZ_TEST}"
    "./logs/other/tests/gen-99999-1-Motley_Crue.log.bz2|${BLOCKSZ_TEST}"
    "./logs/other/tests/gen-99999-1-Motley_Crue.log.gz|${BLOCKSZ_TEST}"
    "./logs/other/tests/gen-99999-1-Motley_Crue.log.lz4|${BLOCKSZ_TEST}"
    "./logs/other/tests/gen-99999-1-Motley_Crue.log.xz|${BLOCKSZ_TEST}"
    "./logs/other/tests/gen-99999-1-Motley_Crue.tar|${BLOCKSZ_TEST}"
    "./logs/other/tests/gen-99999-1-Motley_Crue.zip|${BLOCKSZ_TEST}"
    ./logs/programs/Event_Trace_Log/PITR/PointInTimeRestore.etl
    ./logs/programs/evtx/Microsoft-Windows-Kernel-PnP__Configuration.evtx
    ./logs/programs/journal/RHE_91_system.journal
    ./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/ListSync/Business1/Nucleus-2025-12-23.1618.7692.3.aodl
    ./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/ListSync/Business1/Nucleus-2025-12-21.1217.7692.2.odlgz
    ./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/Personal/SyncEngine-2025-12-21.1217.11144.2.odl
)

readonly SCRIPT_DIR=$(cd "$(dirname -- "${BASH_SOURCE[0]}")"; pwd)
cd "${SCRIPT_DIR}/.."
readonly PROGRAM="${PROGRAM-./target/release/s4}"
readonly HYPERFINE="${HYPERFINE_INSTALL-hyperfine}"
readonly HRUNS="${HRUNS-5}"
readonly HWARMUP="${HWARMUP-1}"
readonly FILTER="${SCRIPT_DIR}/bencher-hyperfine-runs-filter.jq"

if [[ "${HRUNS}" -lt 2 ]]; then
    echo "ERROR: HRUNS must be an integer >= 2" >&2
    exit 1
fi
if [[ "${HWARMUP}" -lt 0 ]]; then
    echo "ERROR: HWARMUP must be an integer >= 0" >&2
    exit 1
fi
if [[ -z "${DIROUT-}" ]]; then
    echo "ERROR: DIROUT must be set" >&2
    exit 1
fi

for tool in "${PROGRAM}" "${HYPERFINE}" jq python3 git lscpu nproc; do
    if ! command -v "${tool}" > /dev/null; then
        echo "ERROR: executable not found: ${tool}" >&2
        exit 1
    fi
done

# print `lscpu` model name cleanly.
# remove extraneous information like (R), (TM), CPU, and extra spaces
# which can break bencher's testbed naming
function print_cpu_modelname() {
        if ! modelname=$(lscpu -p='MODELNAME' &>/dev/null); then
            # lscpu does not support COLUMN 'MODELNAME', i.e. on ARM-based systems.
            # output looks like:
            #    $ lscpu
            #    Architecture:        armv6l
            #    Byte Order:          Little Endian
            #    CPU(s):              1
            #    Vendor ID:           ARM
            #    Model:               7
            #    Model name:          ARM1176
            #    Core(s) per socket:  1
            #    Socket(s):           1
            #    ...
            lscpu \
            | grep 'Model name:' \
            | cut -f2 -d: \
            | sed -E -e 's/ +//g' \
            | sed -E -e 's/\((R|TM|tm)\)//g' -e 's/ CPU( |$)/\1/g' -e 's/ @ / /g' -e 's/ +/ /g' -e 's/^ +| +$//g'
            return 0
        fi
        # lscpu supports COLUMN 'MODELNAME', i.e. on x86-based systems.
        # output looks like:
        #    $  lscpu -p='MODELNAME'
        #    # The following is the parsable format, which can be fed to other
        #    # programs. Each different item in every column has an unique ID
        #    # starting usually from zero.
        #    # Modelname
        #    AMD Ryzen 7 7700X 8-Core Processor
        #    AMD Ryzen 7 7700X 8-Core Processor
        #    AMD Ryzen 7 7700X 8-Core Processor
        #    AMD Ryzen 7 7700X 8-Core Processor
        #    AMD Ryzen 7 7700X 8-Core Processor
        #    AMD Ryzen 7 7700X 8-Core Processor
        #    AMD Ryzen 7 7700X 8-Core Processor
        lscpu -p='MODELNAME' \
        | grep -v '^#' \
        | sort -u \
        | paste -sd ',' - \
        | sed -E -e 's/\((R|TM|tm)\)//g' -e 's/ CPU( |$)/\1/g' -e 's/ @ / /g' -e 's/ +/ /g' -e 's/^ +| +$//g'
}

# verify print_cpu_modelname succeeds
if ! CPU_MODELNAME=$(print_cpu_modelname); then
    echo "ERROR: lscpu to print CPU model name" >&2
    exit 1
fi
echo "CPU Model: ${CPU_MODELNAME}"

if [[ "${UPLOAD_REPORT-}" == "1" ]]; then
    if [[ -z "${BENCHER_RUN_KEY-}" ]]; then
        echo "ERROR: BENCHER_RUN_KEY must be set to upload the report" >&2
        exit 1
    fi
fi

branch="${GITHUB_REF_NAME-}"
if [[ -z "${branch}" ]]; then
    branch=$(git symbolic-ref --short HEAD)
fi
readonly branch
echo "Branch: ${branch}"

BENCHER_TESTBED="${BENCHER_TESTBED-}"
if [[ -z "${BENCHER_TESTBED}" ]]; then
    cpu_model=${CPU_MODELNAME}
    cpu_cores=$(nproc)
    BENCHER_TESTBED=$(
        source /etc/os-release
        printf '%s %s, %s, %s cores' "${NAME}" "${VERSION_ID}" "${cpu_model}" "${cpu_cores}"
    )
    # Bencher testbed names must be at most 64 bytes
    BENCHER_TESTBED=$(
        export LC_ALL=C
        t="${BENCHER_TESTBED:0:64}"
        echo -n "${t%"${t##*[! ]}"}"
    )
fi
readonly BENCHER_TESTBED
# Bencher testbed name rules:
# - non-empty
# - no leading/trailing whitespace
# - at most 64 bytes
if [[ -z "${BENCHER_TESTBED}" \
    || "${BENCHER_TESTBED}" != "${BENCHER_TESTBED#[[:space:]]}" \
    || "${BENCHER_TESTBED}" != "${BENCHER_TESTBED%[[:space:]]}" \
    || $(export LC_ALL=C; echo -n "${#BENCHER_TESTBED}") -gt 64 ]]; then
    echo "ERROR: BENCHER_TESTBED must be 1 to 64 bytes without leading or trailing whitespace: '${BENCHER_TESTBED}'" >&2
    exit 1
fi
echo "Bencher testbed: ${BENCHER_TESTBED}" >&2

# print a horizontal line across the terminal
function echo_line() {
    declare -i i=0
    while (( i < ${COLUMNS:-100} )); do
        printf '─'
        (( i++ )) || true
    done
    printf '\n\n'
}

# example --version output:
#
#     $ ./target/release/s4 --version
#     s4 (Super Speedy Syslog Searcher)
#     Version: 0.11.84
#     MSRV: 1.99.1
#     Profile: release
#     Allocator: system
#     Features:
#     Platform: x86_64-unknown-linux-gnu
#     Target OS: linux
#     Target OS Family: unix
#     Arch: x86_64
#     CPU Target: default
#     CPU Features: fxsr,sse,sse2
#     Compiled Regular Expressions: 188
#     Compiler Version: 1.99.1
#     Rust Build Flags:
#     Optimization Level: 3
#     Git Commit: d174f7c48a7b06ebc0c6d66301d5b6e31a39d0d8
#     Build Date: 2026-10-26T23:05:40
#     License: MIT
#     Repository: https://github.com/jtmoon79/super-speedy-syslog-searcher
#     Author: James Thomas Moon
#
if ! program_version=$(set -x; "${PROGRAM}" --version); then
    echo "ERROR: failed to read PROGRAM --version: ${PROGRAM}" >&2
    exit 1
fi
readonly program_version
echo "${program_version}" >&2
if ! hash=$(echo -n "${program_version}" | grep -oP '(?<=Git Commit: )[0-9a-fA-F]{40,64}'); then
    echo "ERROR: invalid Git Commit in PROGRAM --version: ${PROGRAM}" >&2
    exit 1
fi
readonly hash
if ! git rev-parse --verify "${hash}^{commit}" > /dev/null; then
    echo "ERROR: PROGRAM commit is unavailable in local Git history: ${hash}" >&2
    exit 1
fi
if ! commit_time=$(env TZ=UTC git show -s --format=%ad '--date=format-local:%Y-%m-%dT%H:%M:%SZ' "${hash}" --); then
    echo "ERROR: failed to read author date for PROGRAM commit: ${hash}" >&2
    exit 1
fi
readonly commit_time
echo >&2
echo "Commit Time: ${commit_time}" >&2
echo >&2

declare version
version=$(set -x; "${HYPERFINE}" --version)
if [[ "${version}" != 'hyperfine 2.'* ]]; then
    echo "ERROR: Hyperfine 2.x required, found ${version}" >&2
    exit 1
fi

# allow `failed` files to be skipped in the benchmark runs
# instead of exiting. useful for running benchmarks with older `s4`
# that cannot process some of these input files
declare -A failed=()
declare -a manifest_entries=()
for file_block in "${FILES_BLOCKS[@]}"; do
    IFS='|' read -r file blocks <<< "${file_block}"
    if [[ ! -s "${file}" || ! -r "${file}" || ! -f "${file}" ]]; then
        echo "ERROR: missing, empty, or unreadable benchmark input: ${file}" >&2
        exit 1
    fi
    if ! "${PROGRAM}" "${file}" &>/dev/null; then
        echo "WARNING: failed ${PROGRAM} ${file}" >&2
        echo "         the file will not be part of reported results" >&2
        failed["${file}"]=true
        continue
    fi
    name="${file##*/}"
    manifest_entries+=("$(
        jq -cn --arg name "${name}" --arg blocks "${blocks}" '
            [{name: $name, variant: "FILE_NUM", values: '"${FILE_NUM_ARRAY}"'}]
            + (if $blocks == "" then []
                else [{
                    name: $name,
                    variant: "BLOCKSZ",
                    values: ($blocks | split(",") | map(
                        if test("^[1-9][0-9]*$") then tonumber
                        else error("invalid block size") end))
                }]
                end)
        ')
    ")
done
manifest=$(
    printf '%s\n' "${manifest_entries[@]}" \
    | jq -s 'add |
        if length > 0
            and all(.[]; (.values | unique | length) == (.values | length))
            and (map([.name, .variant]) | unique | length) == length
        then . else error("benchmark names and variants must be unique") end
    '
)
readonly manifest

output_dir=$(realpath "${DIROUT}")
readonly output_dir
mkdir -p -- "${output_dir}"
trap 'exit 130' INT
trap 'exit 143' TERM
echo "Benchmark output directory: ${output_dir}"
echo

declare command
declare result
declare -i index=0
declare -a results=()

function run_hyperfine() {
    local name="${1}"
    local result="${2}"
    local command="${3}"
    shift 3
    local -a args=(
        --export-csv "${result%.json}.csv"
        --export-json "${result}"
        --export-markdown "${result%.json}.md"
        --shell=sh
        --show-output
        --warmup "${HWARMUP}"
        --runs "${HRUNS}"
        "$@"
        --command-name "${name}"
    )
    (
        set -x
        "${HYPERFINE}" "${args[@]}" "${command}"
    )
    results+=("${result}")
}

for file_block in "${FILES_BLOCKS[@]}"; do
    echo_line
    file=$(echo -n "${file_block}" | cut -f1 -d'|')
    blocks=$(echo -n "${file_block}" | cut -s -f2 -d'|')
    if [[ -n "${failed["${file}"]-}" ]]; then
        echo "Skipping failed file: ${file}" >&2
        continue
    fi
    name="${file##*/}"

    # run with Variant FILE_NUM
    index=$((index + 1))
    result="${output_dir}/hyperfine-results-${index}-FILE_NUM.json"
    printf -v command 'yes %q | head -n {FILE_NUM} | %q -cn - 1>/dev/null 2>&1' "${file}" "${PROGRAM}"
    command="yes ${file} | head -n {FILE_NUM} | ${PROGRAM} -cn - 1>/dev/null 2>&1"
    run_hyperfine "${name}" "${result}" "${command}" --parameter-list FILE_NUM "${FILE_NUM}"

    if [[ -z "${blocks}" ]]; then
        continue
    fi

    # run with Variant BLOCKSZ
    index=$((index + 1))
    result="${output_dir}/hyperfine-results-${index}-BLOCKSZ.json"
    # Process the file 5 times to make oddities more obvious.
    printf -v command '%q -cn -- %q %q %q %q %q 1>/dev/null 2>&1' "${PROGRAM}" "${file}" "${file}" "${file}" "${file}" "${file}"
    command="env S4_BLOCKSZ={BLOCKSZ} ${command}"
    run_hyperfine "${name}" "${result}" "${command}" --parameter-list BLOCKSZ "${blocks}"
done

echo_line

if [[ -z "${BENCHER_REPORT-}" ]]; then
    BENCHER_REPORT="${output_dir}/bencher-report.json"
fi

(
    set -x
    jq \
        -es \
        --argjson benchmarks "${manifest}" \
        --argjson runs "${HRUNS}" \
        -f "${FILTER}" \
        "${results[@]}" \
            > "${output_dir}/bencher-results.json"
    jq -n \
        --arg branch "${branch}" \
        --arg hash "${hash}" \
        --arg testbed "${BENCHER_TESTBED}" \
        --arg start_time "${commit_time}" \
        --arg end_time "${commit_time}" \
        --rawfile results "${output_dir}/bencher-results.json" \
        '{
            branch: $branch,
            hash: $hash,
            testbed: $testbed,
            start_time: $start_time,
            end_time: $end_time,
            bmf_version: 1,
            settings: {adapter: "json"},
            results: [$results]
        }' > "${BENCHER_REPORT}"
)
echo
echo "Bencher report: ${BENCHER_REPORT}"
echo

if [[ "${UPLOAD_REPORT-}" == "1" ]]; then
    (set -eux; ./tools/bencher-report-upload.sh "${BENCHER_REPORT}")
fi
