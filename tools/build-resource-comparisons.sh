#!/usr/bin/env bash
#
# Compare builds across the Cartesian product of these CSVs:
#   MSRV_LIST                         (default: 1.99.0)
#   JOBS_LIST                         (default: 1)
#   PROFILES_LIST                     (default: dev)
#   CARGO_PROFILE_DEV_CODEGEN_UNITS    (default: 256)
#   CARGO_PROFILE_DEV_DEBUG            (default: 2)
#   CARGO_PROFILE_DEV_INCREMENTAL      (default: true)
#
# Example:
#   MSRV_LIST=1.88.0,1.92.0 JOBS_LIST=1,2 PROFILES_LIST=dev,release_Opt0 \
#   CARGO_PROFILE_DEV_CODEGEN_UNITS=1,16 \
#   CARGO_PROFILE_DEV_DEBUG=0,2 CARGO_PROFILE_DEV_INCREMENTAL=false,true \
#     ./tools/build-profile.sh
#
# Requires Linux ps, GNU time, rustup, and Bash 5+. Builds and diagnostics go to
# stderr; the summary goes to stdout. Installs requested toolchains if needed,
# without changing the default toolchain. Cleans the selected profile BEFORE
# EVERY build, including the first. The DEV-named input lists are applied to
# each selected profile using its CARGO_PROFILE_<NAME>_* variables.
# Other inherited build settings remain unchanged.
#
# Memory is the peak sampled sum of Cargo and descendant RSS, in KiB, sampled
# every $SAMPLE_RATE seconds. Shared pages can be counted more than once; swapped pages
# are excluded, and peaks between samples can be missed. This is not GNU time %M.

set -euo pipefail
export LC_ALL=C
cd "$(dirname -- "${BASH_SOURCE[0]}")/.."

function fail() {
    echo "$*" >&2
    exit 1
}

[[ -n "${EPOCHREALTIME:-}" ]] || fail "Bash 5+ is required."
[[ $# -eq 0 ]] || fail "Configure builds with environment variables; no arguments are accepted."
for command in cargo rustup ps awk mktemp sleep; do
    command -v "$command" >/dev/null || fail "Required command not found: $command"
done
[[ -x /usr/bin/time ]] || fail "GNU time is required at /usr/bin/time."
[[ -x /usr/bin/column ]] || fail "column is required at /usr/bin/column."

declare -gr SAMPLE_RATE=${SAMPLE_RATE-1}

function parse_list() {
    local name=$1
    local value=$2
    local pattern=$3
    local item
    local -n result=$4
    [[ -n "$value" && "$value" != ,* && "$value" != *, && "$value" != *,,* &&
       "$value" != *$'\n'* && "$value" != *$'\r'* ]] \
        || fail "$name must be a nonempty comma-separated list without empty entries."
    IFS=, read -r -a result <<< "$value"
    for item in "${result[@]}"; do
        [[ "$item" =~ $pattern ]] || fail "Invalid $name value: $item"
    done
}

declare -a versions
declare -a jobs_list
declare -a profiles
declare -a codegen_list
declare -a debug_list
declare -a incremental_list
declare -a rows=()
parse_list MSRV_LIST "${MSRV_LIST-1.99.0}" '^[[:alnum:]][[:alnum:]._-]*$' versions
parse_list JOBS_LIST "${JOBS_LIST-1}" '^[1-9][0-9]*$' jobs_list
parse_list PROFILES_LIST "${PROFILES_LIST-dev}" '^[[:alnum:]][[:alnum:]_-]*$' profiles
parse_list CARGO_PROFILE_DEV_CODEGEN_UNITS "${CARGO_PROFILE_DEV_CODEGEN_UNITS-256}" \
    '^[1-9][0-9]*$' codegen_list
parse_list CARGO_PROFILE_DEV_DEBUG "${CARGO_PROFILE_DEV_DEBUG-2}" \
    '^(0|1|2|none|limited|full|line-tables-only|line-directives-only)$' debug_list
parse_list CARGO_PROFILE_DEV_INCREMENTAL "${CARGO_PROFILE_DEV_INCREMENTAL-true}" \
    '^(true|false)$' incremental_list
unset CARGO_PROFILE_DEV_CODEGEN_UNITS CARGO_PROFILE_DEV_DEBUG CARGO_PROFILE_DEV_INCREMENTAL

time_file=$(mktemp -t s4-build-profile.XXXXX)
build_pid=
declare -i failed=0

# The first line is aggregate RSS; remaining lines are descendant PIDs.
function process_tree() {
    ps -eo pid=,ppid=,rss= | awk -v root="$build_pid" '
        { parent[$1] = $2; rss[$1] = $3 }
        END {
            member[root] = 1
            do {
                changed = 0
                for (pid in parent)
                    if (!(pid in member) && (parent[pid] in member)) {
                        member[pid] = 1
                        changed = 1
                    }
            } while (changed)
            for (pid in member)
                if (pid != root) total += rss[pid]
            printf "%.0f\n", total
            for (pid in member)
                if (pid != root) print pid
        }
    '
}

function print_table() {
    {
        echo 'RSS_KiB|TIME|JOBS|RUST|PROFILE|CODEGEN_UNITS|DEBUG|INCREMENTAL|EXIT'
        local row
        for row in "${rows[@]}"; do
            echo "$row"
        done
    } | /usr/bin/column \
        --table \
        --separator '|' \
        --output-separator ' | ' \
        --table-right 1,2,3,6,9 \
        | awk '
            BEGIN { print "" }
            {
                if (NR == 1) width = length($0)
                while (length($0) < width) $0 = $0 " "
                print "| " $0 " |"
                if (NR == 1) {
                    gsub(/[^|]/, "-")
                    print "|-" $0 "-|"
                }
            }
        '
}

function finish() {
    local status=$?
    local snapshot
    local pid
    local -a descendants=()
    trap - EXIT INT TERM
    if [[ -n "$build_pid" ]]; then
        if snapshot=$(process_tree); then
            mapfile -t descendants <<< "$snapshot"
            for pid in "${descendants[@]:1}" "$build_pid"; do
                if kill -0 "$pid" 2>/dev/null; then
                    kill "$pid" || echo "Unable to terminate process $pid" >&2
                fi
            done
        else
            echo "Unable to inspect active build processes during cleanup." >&2
            kill "$build_pid" || echo "Unable to terminate process $build_pid" >&2
        fi
        wait "$build_pid" || :
    fi
    rm -f -- "$time_file"
    print_table
    exit "$status"
}

function sec_to_minsec() {
    declare -i total_seconds=$1
    declare -i minutes=$((total_seconds / 60))
    declare -i seconds=$((total_seconds % 60))
    # XXX: `printf "%02d" 5` prints nothing!?
    awk "BEGIN { print sprintf(\"%02d:%02d\", $minutes, $seconds) } "
}

trap finish EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

export S4_BUILD_REGEX=ALL
unset S4_BUILD_REGEX_NO_REBUILD
for version in "${versions[@]}"; do
    (set -x; rustup toolchain install "$version" --profile minimal >&2)
    for profile in "${profiles[@]}"; do
        prefix=${profile^^}
        prefix=${prefix//-/_}
        for jobs in "${jobs_list[@]}"; do
            for codegen in "${codegen_list[@]}"; do
                for debug in "${debug_list[@]}"; do
                    for incremental in "${incremental_list[@]}"; do
                        printf '%0.s-' {1..80}
                        echo
                        printf 'Building: rust=%s profile=%s jobs=%s codegen=%s debug=%s incremental=%s\n' \
                            "$version" "$profile" "$jobs" "$codegen" "$debug" "$incremental" >&2
                        echo
                        set -x
                        export "CARGO_PROFILE_${prefix}_CODEGEN_UNITS=$codegen"
                        export "CARGO_PROFILE_${prefix}_DEBUG=$debug"
                        export "CARGO_PROFILE_${prefix}_INCREMENTAL=$incremental"
                        set +x
                        (set -x; cargo "+$version" clean "--profile=$profile" >&2)
                        declare -i start=$SECONDS
                        (set -x; cargo "+$version" --version >&2)
                        (
                            set -x
                            cargo "+$version" build --quiet --locked --profile "$profile" --jobs "$jobs" >&2
                        ) &
                        build_pid=$!
                        peak=0
                        while kill -0 "$build_pid" 2>/dev/null; do
                            snapshot=$(process_tree)
                            rss=${snapshot%%$'\n'*}
                            if (( rss > peak )); then
                                peak=$rss
                            fi
                            sleep $SAMPLE_RATE
                        done
                        declare -i rc=0
                        wait "$build_pid" || rc=$?
                        build_pid=
                        declare -i elapsed=$((SECONDS - start))
                        elapsed_ms=$(sec_to_minsec "$elapsed")
                        rows+=("$peak|$elapsed_ms|$jobs|$version|$profile|$codegen|$debug|$incremental|$rc")
                        print_table
                        if (( rc != 0 )); then
                            echo "Build failed with exit $rc; continuing the matrix." >&2
                            failed=1
                        fi
                    done
                done
            done
        done
        unset "CARGO_PROFILE_${prefix}_CODEGEN_UNITS" \
            "CARGO_PROFILE_${prefix}_DEBUG" "CARGO_PROFILE_${prefix}_INCREMENTAL"
    done
done
exit "$failed"
