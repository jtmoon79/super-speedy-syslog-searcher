#!/usr/bin/env bash
#
# wrapper to run hyperfine benchmarks on various log files with different
# Bencher Metrics and Variants.
# For running locally and in CI.
#
# Environment variables:
#   PROGRAM           s4 executable (default: ./target/release/s4)
#   HYPERFINE_INSTALL  Hyperfine executable (default: hyperfine on PATH)
#   DIROUT            Output directory; if unset/empty, use a temporary directory
#                     removed at exit. Supplied directories are never removed.
#   HRUNS             Measured runs, at least 2 (default: 10)
#   HWARMUP           Warmup runs, at least 0 (default: 2)
#   BENCHER_REPORT    Upload report path (default: DIROUT/bencher-report.json)
#   BENCHER_TESTBED   Testbed name (default: Linux OS, CPU model, and core count)
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

# Define files to benchmark, along with differing block sizes
# some files without block sizes because they do not use a BlockReader
declare -r BLOCKSZ_TEST="4096,8192,65535,131071"
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
declare -r HRUNS="${HRUNS-10}"
declare -r HWARMUP="${HWARMUP-2}"
readonly FILTER="${SCRIPT_DIR}/bencher-hyperfine-runs-filter.jq"

if [[ "${HRUNS}" -lt 2 ]]; then
    echo "ERROR: HRUNS must be an integer >= 2" >&2
    exit 1
fi
if [[ "${HWARMUP}" -lt 0 ]]; then
    echo "ERROR: HWARMUP must be an integer >= 0" >&2
    exit 1
fi

for tool in "${PROGRAM}" "${HYPERFINE}" jq python3 git lscpu nproc; do
    if ! command -v "${tool}" > /dev/null; then
        echo "ERROR: executable not found: ${tool}" >&2
        exit 1
    fi
done

function echo_line() {
    # print a horizontal line across the terminal
    python3 -Bc "print('─' * ${COLUMNS:-100}, end='\n\n')"
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

declare -a manifest_entries=()
for file_block in "${FILES_BLOCKS[@]}"; do
    IFS='|' read -r file blocks <<< "${file_block}"
    if [[ ! -s "${file}" || ! -r "${file}" || ! -f "${file}" ]]; then
        echo "ERROR: missing, empty, or unreadable benchmark input: ${file}" >&2
        exit 1
    fi
    name="${file##*/}"
    manifest_entries+=("$(
        jq -en --arg name "${name}" --arg blocks "${blocks}" '
            {name: $name, block_sizes:
                (if $blocks == "" then []
                    else $blocks | split(",") | map(
                    if test("^[1-9][0-9]*$") then tonumber
                    else error("invalid block size") end)
                end)
            }
        ')
    ")
done
manifest=$(
    printf '%s\n' "${manifest_entries[@]}" \
    | jq -es '
        if length > 0
            and (map(.name) | unique | length) == length
            and all(.[]; (.block_sizes | unique | length) == (.block_sizes | length))
        then . else error("benchmark names and block sizes must be unique") end
    '
)
readonly manifest

declare output_dir
if [[ -n "${DIROUT-}" ]]; then
    mkdir -p -- "${DIROUT}"
    output_dir="${DIROUT}"
    output_dir=$(realpath "${output_dir}")
else
    output_dir=$(mktemp -d "${TMPDIR:-${TMP:-/tmp}}/s4-hyperfine-runs.sh.XXXXXX")
    output_dir=$(realpath "${output_dir}")
    trap "rm -rf -- \"${output_dir}\"" EXIT
fi
readonly output_dir
trap 'exit 130' INT
trap 'exit 143' TERM
echo "Benchmark output directory: ${output_dir}"
echo

declare command
declare result
declare -i index=0
declare -a results=()

for file_block in "${FILES_BLOCKS[@]}"; do
    echo_line
    file=$(echo -n "${file_block}" | cut -f1 -d'|')
    blocks=$(echo -n "${file_block}" | cut -s -f2 -d'|')
    name="${file##*/}"
    index=$((index + 1))
    result="${output_dir}/hyperfine-results-${index}.json"
    declare -a args=(
        --export-csv "${result%.json}.csv"
        --export-json "${result}"
        --export-markdown "${result%.json}.md"
        --shell=bash
        --show-output
        --warmup "${HWARMUP}"
        --runs "${HRUNS}"
        --command-name "${name}"
    )
    # process the file 5 times to make oddities more obvious
    printf -v command '%q -cn -- %q %q %q %q %q &>/dev/null' "${PROGRAM}" "${file}" "${file}" "${file}" "${file}" "${file}"
    if [[ -n "${blocks}" ]]; then
        args+=(--parameter-list S4_BLOCKSZ "${blocks}")
        command="env S4_BLOCKSZ={S4_BLOCKSZ} ${command}"
    fi
    (
        set -x
        "${HYPERFINE}" "${args[@]}" "${command}"
    )
    results+=("${result}")
done

echo_line

branch="${GITHUB_REF_NAME-}"
if [[ -z "${branch}" ]]; then
    branch=$(git symbolic-ref --short HEAD)
fi
BENCHER_TESTBED="${BENCHER_TESTBED-}"
if [[ -z "${BENCHER_TESTBED}" ]]; then
    cpu_model=$(lscpu -p='MODELNAME' | grep -v '^#' | sort -u | paste -sd ',' -)
    cpu_cores=$(nproc)
    BENCHER_TESTBED=$(
        source /etc/os-release
        printf '%s %s, %s, %s cores' "${NAME}" "${VERSION_ID}" "${cpu_model}" "${cpu_cores}"
    )
fi
readonly branch
readonly BENCHER_TESTBED

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
