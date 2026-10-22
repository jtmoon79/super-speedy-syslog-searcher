#!/usr/bin/env bash
#
# hardcoded performance comparison of GNU `grep | sort`, `s4` (all allocator
# variations), `logmerger`, `lnav`, `logdissect` and `toolong` (tl).
#
# For each log merging tool, this script runs a search for log lines
# between two datetimes in three 5,000-line log files.
# Gathers runtime, peak RSS, and hardware counters using `hyperfine` 2.0.
# Then does some scraping of those results and outputs that as a markdown table.
# The markdown output is typically for display (bragging) in the top-level
# README.md file.
#
# pass `--skip-tl` to skip processing toolong which takes over the console
# window and stalls non-interactive consoles. Otherwise it is measured once;
# manually exit its TUI when merging finishes.
#
# Set PROGRAMS_S4_LISTING to a TSV file listing other s4 programs to compare.
# Each non-empty line must have 2 tab-separated fields:
#   <path-to-s4> <extra-info>
# Hint: use `download-released-s4.sh` to quickly get old s4 binaries.
#
# PROGRAM_LNAV to override the lnav found in PATH.
# PROGRAM_LM to override the logmerger found in PATH.
# PROGRAM_TL to override the toolong (tl) found in PATH.

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

# do a little work to find Python interpreter in the PATH
PYTHON=${PYTHON-$(
    if which python3 &>/dev/null; then
        which python3
    else
        which python
    fi
)}
readonly PYTHON
(set -x; "${PYTHON}" --version) | head -n1

# check for hyperfine
readonly HYPERFINE=/home/ulug/.cargo/bin/hyperfine
if [[ ! -x "${HYPERFINE}" ]]; then
    echo "ERROR: hyperfine 2 required at ${HYPERFINE}" >&2
    echo "install:" >&2
    echo "    cargo install --locked hyperfine" >&2
    exit 1
fi
if [[ "$("${HYPERFINE}" --version)" != 'hyperfine 2.'* ]]; then
    echo "ERROR: hyperfine 2.x required at ${HYPERFINE}" >&2
    exit 1
fi
(set -x; "${HYPERFINE}" --version)
readonly METRICS='time_wall_clock:ms,memory_peak_resident:kB,cpu_cycles:M,cache_references:M,cache_misses:M,branch_misses:M'

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
    echo "       and \$PROGRAM_LNAV not set" >&2
    echo "install:" >&2
    echo "    sudo apt install lnav" >&2
    exit 1
fi
readonly PROGRAM_LNAV=${PROGRAM_LNAV-lnav}

echo

declare -ir HRUNS=30

# make sure Python packages are installed to expected versions
(
    set -x
    "${PYTHON}" -m pip install \
        --upgrade \
        --no-python-version-warning \
        --disable-pip-version-check \
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

function json_file_new() {
    local path
    path=$(mktemp -p "${TEMPD}" -t "json_XXXXX.json")
    printf '%s\n' "${path}" | tee -a "${json_files}"
}

function write_metadata() {
    printf '%s|%s|%s|%s|%s|%s\n' "$@" > "${json_file}.metadata"
}

# JSON always exports seconds, bytes, and raw counts, regardless of --metrics units.
# Schema 2: {schema_version: 2, results: [{name: "...", command: "...",
#   measurements: [{time_wall_clock: {value: 1, unit: "second"}, ...}],
#   summary: {time_wall_clock: {mean: 1, stddev: null, min: 1, max: 1,
#                             count: 1, unit: "second"}, ...}}]}
function validate_json() {
    # shellcheck disable=SC2016
    "${JQ}" -e '
        def require($ok; $message):
            if $ok then . else error($message) end;
        require(.schema_version == 2; "expected hyperfine JSON schema_version 2")
        | require((.results | type) == "array" and (.results | length) == 1;
                  "expected exactly one benchmark result")
        | .results[0]
        | (.name // .command) as $name
        | require(($name | type) == "string"; "missing benchmark name")
        | require((.measurements | type) == "array" and (.measurements | length) > 0;
                  "\($name): missing measurements")
        | . as $result
        | (.measurements | length) as $runs
        | ["time_wall_clock", "memory_peak_resident", "cpu_cycles",
           "cache_references", "cache_misses", "branch_misses"][]
        | . as $metric
        | $result
        | require(all(.measurements[]; (.[$metric].value | type) == "number");
                  "\($name): unavailable or incomplete metric \($metric)")
        | .summary[$metric]
        | require(.count == $runs and all(.mean, .min, .max; type == "number");
                  "\($name): invalid or incomplete summary for \($metric)")
        | require((if $runs == 1 then .stddev == null
                   else (.stddev | type) == "number" end);
                  "\($name): invalid standard deviation for \($metric)")
        | require((if $metric == "time_wall_clock" then
                       .unit == "second" and all($result.measurements[];
                           .time_wall_clock.unit == "second")
                   elif $metric == "memory_peak_resident" then
                       .unit == "byte" and all($result.measurements[];
                           .memory_peak_resident.unit == "byte")
                   else true end);
                  "\($name): unexpected unit for \($metric)")
        | true
    ' "${1}" > /dev/null || {
        echo "ERROR: invalid hyperfine measurements in ${1}; all requested metrics are required (check hardware counter permissions)" >&2
        return 1
    }
}

function metrics_row() {
    local mean stddev min max rss cycles references misses branches LC_NUMERIC=C
    IFS=$'\t' read -r mean stddev min max rss cycles references misses branches < <(
        # shellcheck disable=SC2016
        "${JQ}" -r '
            .results[0] | .summary as $s |
            [($s.time_wall_clock.mean * 1000),
             (if $s.time_wall_clock.stddev == null then "N/A"
              else ($s.time_wall_clock.stddev * 1000) end),
             ($s.time_wall_clock.min * 1000),
             ($s.time_wall_clock.max * 1000),
             ($s.memory_peak_resident.mean / 1000),
             ($s.cpu_cycles.mean / 1000000),
             ($s.cache_references.mean / 1000000),
             ($s.cache_misses.mean / 1000000),
             ($s.branch_misses.mean / 1000000)] | @tsv
        ' "${1}"
    )
    if [[ "${stddev}" != 'N/A' ]]; then
        printf -v stddev '%.1f' "${stddev}"
    fi
    printf '%.1f ± %s|%.1f|%.1f|%.1f|%.3f|%.3f|%.3f|%.3f|\n' \
        "${mean}" "${stddev}" "${min}" "${max}" "${rss}" \
        "${cycles}" "${references}" "${misses}" "${branches}"
}

function exit_() {
    rm -rf -- "${TEMPD}"
}
trap exit_ EXIT


if [[ ! "${PROGRAMS_S4_LISTING-}" ]]; then
    echo "Environment variable PROGRAMS_S4_LISTING not set to a TSV file" >&2
    echo "Use found values:" >&2
    readonly PROGRAMS_S4_LISTING=${TEMPD}/programs-s4-listing.tsv
    for p in \
            ./target/release/s4 \
            ./target/release_O0_cgu1/s4 \
            ./target/release_O0_cgu512_pa/s4 \
            ./target/release_O1_cgu1/s4 \
            ./target/release_O1_cgu256_pa/s4 \
            ./target/release_O1_cgu512_pa/s4 \
            ./target/release_O2_cgu1/s4 \
            ./target/release_O2_cgu256_pa/s4 \
            ./target/release_O2_cgu512_pa/s4 \
            ./target/release_O3_cgu1_pa/s4 \
            ./target/release_O3_cgu256_pa/s4 \
            ./target/release_O3_cgu512_pa/s4 \
            ./target/release_Opt0/s4 \
            ./target/jemalloc/s4 \
            ./target/mimalloc/s4 \
            ./target/rpmalloc/s4 \
            ./target/tcmalloc/s4 \
            ./target/x86_64-unknown-linux-gnu/release/s4 \
            ./target/x86_64-unknown-linux-musl/release/s4 \
            ./target/x86_64-unknown-linux-ohos/release/s4 \
            ./target/x86_64-v4/x86_64-unknown-linux-musl/release/s4 \
    ; do
        if [[ -e "${p}" ]]; then
            echo "${p}"
        fi
    done > "${PROGRAMS_S4_LISTING}"
    cat "${PROGRAMS_S4_LISTING}" >&2
    echo >&2
    sleep 1
fi

if [[ ! -f "${PROGRAMS_S4_LISTING}" ]]; then
    echo "ERROR: PROGRAMS_S4_LISTING must point to a TSV file: ${PROGRAMS_S4_LISTING}" >&2
    exit 1
fi

# datetime range for s4, lnav
readonly after_dt="2000-01-01T00:20:00"
readonly befor_dt="2000-01-01T00:50:00"
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


# print the s4 version string from the --version string
#
# s4 newer --version format <=0.10
#    s4 (Super Speedy Syslog Searcher)
#    Version: 0.10.83
#    MSRV: 1.88.0
#    Profile: release_Opt0
#    Allocator: system
#    Features:
#    Platform: x86_64-unknown-linux-gnu
#    Target OS: linux
#    Target OS Family: unix
#    Arch: x86_64
#    CPU Target: default
#    CPU Features: fxsr,sse,sse2
#    Compiled Regular Expressions: 188
#    Compiler Version: 1.88.0
#    Rust Build Flags:
#    Optimization Level: 0
#    Git Commit: 8003ab87a10130dfd56897b0cfae712b94466023
#    Build Date: 2026-10-13T20:03:29
#    License: MIT
#    Repository: https://github.com/jtmoon79/super-speedy-syslog-searcher
#    Author: James Thomas Moon
#
# s4 0.8.80 version format
#    $ s4 --version
#    s4 (Super Speedy Syslog Searcher)
#    Version: 0.9.81
#    MSRV: 1.88.0
#    Allocator: system
#
# s4 older --version formats <0.8.80
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

function s4_cpu_target() {
    declare -r prog=$1
    declare out=
    out=$("${prog}" --version)
    # check if new version
    if echo -n "${out}" | grep -qE '^CPU Target: ' &> /dev/null; then
        # new version format
        echo -n "${out}" | grep -Ee '^CPU Target: ' | sed -E 's/^CPU Target: (.+)$/\1/'
    else
        # old version format
        echo_warn "no CPU Target found for s4 program ${prog}" >&2
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
while IFS=$'\t' read -r s4_prog_path s4_prog_extra || [[ -n "${s4_prog_path}" ]]; do
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
    (
        files_caching
        if [[ -n "${s4_prog_extra-}" ]]; then
            s4_prog_extra=" ${s4_prog_extra}"
        fi
        echo_command "${s4_full_path} -a='${after_dt}' -b='${befor_dt}' -cn ${files[*]}"
        set -x
        "${HYPERFINE}" \
            --metrics "${METRICS}" \
            --warmup=2 \
            --style=basic \
            --runs=${HRUNS} \
            --export-json "${json_file}" \
            -N \
            -n "s4${s4_prog_extra-}" \
            -- \
            "'${s4_full_path}' -a='${after_dt}' -b='${befor_dt}' -cn ${files[*]}"
    )
    validate_json "${json_file}"
    (
        version=$(s4_version "${s4_full_path}")
        profile=$(s4_profile "${s4_full_path}")
        allocator=$(s4_allocator "${s4_full_path}")
        platform=$(s4_platform "${s4_full_path}")
        cpu_target=$(s4_cpu_target "${s4_full_path}")
        opt_lvl=$(s4_opt_lvl "${s4_full_path}")
        write_metadata "${version}" "${profile}" "${allocator}" "${platform}" "${cpu_target}" "${opt_lvl}"
    )
done < "${PROGRAMS_S4_LISTING}"

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
(
    files_caching
    # search for datetimes between $after_dt $befor_dt
    # using decently constrained regexp to match meaning
    echo_command "${GREP} -hEe '${regex_dt}' -- ${files[*]} | ${SORT} -t ' ' -k 1 -s"
    pipeline="'${GREP}' -hEe '${regex_dt}' -- ${files[*]} | '${SORT}' -t ' ' -k 1 -s"
    # Hardware counters require --shell=none; execute the pipeline shell directly.
    set -x
    "${HYPERFINE}" \
        --metrics "${METRICS}" \
        --warmup=2 \
        --style=basic \
        --runs=${HRUNS} \
        --export-json "${json_file}" \
        -N \
        -n "grep+sort" \
        -- \
        "sh -c '${pipeline//\'/\'\\\'\'}'"
)
validate_json "${json_file}"
(
    files_caching
    version=$("${GREP}" --version | head -n1 | cut -f4 -d' ')
    profile=' '
    allocator=' '
    platform=$(arch)
    cpu_target=' '
    opt_lvl=' '
    write_metadata "${version}" "${profile}" "${allocator}" "${platform}" "${cpu_target}" "${opt_lvl}"
    "${GREP}" -hEe "${regex_dt}" -- "${files[@]}" | "${SORT}" -t ' ' -k 1 -s > "${tmpOut}"
)

echo_line
echo_title "lnav"

# Keep the grep/sort reference, normalizing equal-timestamp ordering.
readonly lnav_expected="${TEMPD}/lnav-expected.out"
LC_ALL=C "${SORT}" "${tmpOut}" > "${lnav_expected}"

mkdir -p "${TEMPD}/lnav-home" "${TEMPD}/lnav-config"
# show-file restores identical input files that lnav otherwise hides.
# Numeric datetime comparison includes the upper endpoint with either precision.
declare -ar lnav_args=(
    env "HOME=${TEMPD}/lnav-home" "XDG_CONFIG_HOME=${TEMPD}/lnav-config"
    "${PROGRAM_LNAV}" -I "${PROJ_DIR}/tools/compare-log-mergers/config" -N -n
    -c ':show-file *'
    -c ";SELECT log_raw_text FROM lnav1 WHERE julianday(log_time) BETWEEN julianday('${after_dt}') AND julianday('${befor_dt}') ORDER BY log_time, log_raw_text"
    -c ':write-raw-to -'
    "${files[@]}"
)
# hyperfine --shell=none parses POSIX quoting, not Bash printf %q escapes.
lnav_command=
for arg in "${lnav_args[@]}"; do
    lnav_command+="'${arg//\'/\'\\\'\'}' "
done
readonly lnav_command

json_file=$(json_file_new)
(
    files_caching
    echo_command "${lnav_command}"
    set -x
    "${PROGRAM_LNAV}" --version
    "${HYPERFINE}" \
        --metrics "${METRICS}" \
        --warmup=2 \
        --style=basic \
        --runs=${HRUNS} \
        --export-json "${json_file}" \
        -N \
        -n "${PROGRAM_LNAV}" \
        -- \
        "${lnav_command}"
)
validate_json "${json_file}"

(
    files_caching
    version=$("${PROGRAM_LNAV}" --version | head -n1 | cut -d' ' -f2)
    profile=' '
    allocator=' '
    platform=$(arch)
    cpu_target=' '
    opt_lvl=' '
    write_metadata "${version}" "${profile}" "${allocator}" "${platform}" "${cpu_target}" "${opt_lvl}"
    "${lnav_args[@]}" > "${tmpOut}"
)

if ! LC_ALL=C "${SORT}" --check --stable --key=1,1 "${tmpOut}"; then
    echo "ERROR: lnav output is not sorted by datetime" >&2
    exit 1
fi
LC_ALL=C "${SORT}" "${tmpOut}" > "${TEMPD}/lnav-actual.out"
if ! cmp -s "${lnav_expected}" "${TEMPD}/lnav-actual.out"; then
    echo "ERROR: lnav output does not match the filtered grep/sort reference" >&2
    exit 1
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
(
    files_caching
    echo_command "${PROGRAM_LM} --inline --output=- --start '${after_dt}' --end '${befor_dt}' ${files[*]}"
    set -x
    "${HYPERFINE}" \
        --metrics "${METRICS}" \
        -i \
        --warmup=2 \
        --style=basic \
        --runs=${HRUNS} \
        --export-json "${json_file}" \
        -N -n "${PROGRAM_LM}" \
        --env PYTHONOPTIMIZE=2 \
        -- \
        "'${PROGRAM_LM}' --inline --output=- --start '${after_dt}' --end '${befor_dt}' ${files[*]}"
)
validate_json "${json_file}"
(
    version=$("${PYTHON}" -m pip list | grep -Fe 'logmerger' | awk '{print $2}')
    profile=' '
    allocator=' '
    platform=$("${PYTHON}" --version)
    cpu_target=' '
    opt_lvl='2'
    write_metadata "${version}" "${profile}" "${allocator}" "${platform}" "${cpu_target}" "${opt_lvl}"
)

echo_line
echo_title "logdissect"

PROGRAM_LD=${PROGRAM_LD-logdissect}
(set -x; "${PROGRAM_LD}" --version)

echo

echo "TODO: figure out how to use logdissect. I'm unable to get it to match on ANY files." >&2
echo

echo_line
echo_title "TooLong"

# TODO: how to force toolong to not create a TUI window so it doesn't need
#       to be forcefully killed?
if ! ${skip_tl}; then
    PROGRAM_TL=${PROGRAM_TL-tl}
    PROGRAM_TL_RP=$(realpath "${PROGRAM_TL}")
    (set -x; "${PROGRAM_TL}" --version)
    (
        set -x
        "${PYTHON}" -m compileall -o 0 -o 1 -o 2 -q "${PYSITE_PKG_PATH}/toolong"
    )
    json_file=$(json_file_new)
    (
        files_caching
        # tl, version 1.5.0
        version=$("${PROGRAM_TL}" --version | head -n1 | cut -d' ' -f3)
        allocator=' '
        profile=' '
        platform=$("${PYTHON}" --version)
        cpu_target=' '
        opt_lvl='2'
        echo_command "${PROGRAM_TL}" --merge --output-merge "${tmpOut}" "${files[@]}"
        write_metadata "${version}" "${profile}" "${allocator}" "${platform}" "${cpu_target}" "${opt_lvl}"
        set -x
        "${HYPERFINE}" \
            --metrics "${METRICS}" \
            --warmup=0 \
            --runs=1 \
            --show-output \
            --export-json "${json_file}" \
            -N \
            -n "toolong" \
            --env "PYTHONOPTIMIZE=2" \
            --ignore-failure \
            -- \
            "'${PROGRAM_TL}' --merge --output-merge '${tmpOut}' ${files[*]}" &
    )
    sleep 3
    echo
    # BUG: this kill `tl` in a way that causes the terminal window to become unresponsive.
    (set -x; pkill --signal SIGINT -f "${PROGRAM_TL_RP}")
    sleep 1
    validate_json "${json_file}"
else
    echo "Skipping toolong (tl)" >&2
fi
echo

echo_line

# markdown table header
echo '|Program|Version|Profile|Allocator|Platform|CPU Target|Opt Level|Mean (ms)|Min (ms)|Max (ms)|Mean peak RSS (kB)|Mean CPU cycles (M)|Mean cache references (M)|Mean cache misses (M)|Mean branch misses (M)|' > "${tmpOut}"
echo '|:---|:---|:---|:---|:---|:---|:---|---:|---:|---:|---:|---:|---:|---:|---:|' >> "${tmpOut}"

json_count=$(wc -l < "${json_files}")
echo "Processing ${json_count} JSON files to generate markdown table..." >&2

# markdown table rows
exec 3< "${json_files}"
while IFS= read -r json <&3; do
    validate_json "${json}"
    if [[ ! -s "${json}.metadata" ]]; then
        echo "ERROR: missing benchmark metadata for ${json}" >&2
        exit 1
    fi
    echo "Processing file: ${json}" >&2
    (
        command=$("${JQ}" -r '.results[0].name // .results[0].command' "${json}")
        printf "|\`%s\`|%s|" "${command}" "$(cat "${json}.metadata")"
        metrics_row "${json}"
    ) >> "${tmpOut}"
done
exec 3<&-

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
    declare -i col=180
    if [[ $COLUMNS -lt $col ]]; then
        col=${COLUMNS}
    fi
    glow "${mdFinalFinal}" --width ${col}
    echo
else
    echo "install 'glow' for pretty markdown viewing" >&2
    echo "    go install github.com/charmbracelet/glow/v2@latest" >&2
fi
