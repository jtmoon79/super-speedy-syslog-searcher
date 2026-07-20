#!/usr/bin/env bash
#
# Determine the minimum RUST_MIN_STACK needed for PROGRAM to process each file.
#
# Similar to tools/build-regex-times.sh stack probing, but this script never
# builds binaries. It just runs PROGRAM repeatedly with larger stack sizes.
#
# Environment variables:
#   PROGRAM           Program to run (default: ./target/release/s4)
#   STACK_SIZE_START  Initial RUST_MIN_STACK in bytes (default: 1024)
#   STACK_SIZE_STEP   Increment per retry in bytes (default: 1024)
#   STACK_SIZE_MAX    Optional max stack in bytes (default: 0, means no max)
#   SORT              true/1 to print sorted rows by stack size and elapsed time
#
# Usage:
#   PROGRAM=./target/release/s4 ./tools/stack-size.sh <file> [file...]

set -euo pipefail

cd "$(dirname -- "${BASH_SOURCE[0]}")/.."

if [[ -z "${EPOCHREALTIME:-}" ]]; then
    echo "EPOCHREALTIME is not available in this shell." >&2
    exit 1
fi

declare -r OUT_MD=${OUT_MD-stack-sizes.md}
declare -r PROGRAM=${PROGRAM-./target/release/s4}
declare -ir STACK_SIZE_START=${STACK_SIZE_START-1024}
declare -ir STACK_SIZE_STEP=${STACK_SIZE_STEP-1024}
declare -ir STACK_SIZE_MAX=${STACK_SIZE_MAX-9000000}

if [[ ${#@} -eq 0 ]]; then
    echo "Usage: $0 <file> [file ...]" >&2
    exit 2
fi

if [[ ! -f "$PROGRAM" ]]; then
    echo "PROGRAM '$PROGRAM' not found." >&2
    exit 1
fi

if ! "$PROGRAM" --version &> /dev/null; then
    echo "PROGRAM '$PROGRAM' failed to run --version." >&2
    exit 1
fi

if [[ $STACK_SIZE_START -le 0 ]]; then
    echo "STACK_SIZE_START must be > 0" >&2
    exit 1
fi

if [[ $STACK_SIZE_STEP -le 0 ]]; then
    echo "STACK_SIZE_STEP must be > 0" >&2
    exit 1
fi

declare -ag rows=()
declare -rg SEP_IN='║'
declare -rg SEP_MD='|'  # markdown table separator
declare -rg SEP_OUT='┊'  # separator for printed table
declare -i failed=0

declare -rg COLUMNS_N="\
FILE,\
STACK MIN,\
(B)"
declare -rg COLUMNS_RA="STACK MIN,(B)"
declare -rg COL_STACK=5
declare -rg COL_ELAPSED=8

declare C_ON='\e[93m'
declare C_OFF='\e[39m'
declare -g SORTED=false

function float_subtract() {
    awk -v a="$1" -v b="$2" 'BEGIN { printf "%.1f", (a - b) }'
}

function print_rows() {
    declare -a rows2=()
    if $SORTED; then
        mapfile -t rows2 < <(
            printf '%s\n' "${@}" \
            | sort -t"$SEP_IN" -k${COL_STACK}n -k${COL_ELAPSED}n
        )
    else
        rows2=("${@}")
    fi
    if [[ ${#rows2[@]} -eq 0 ]]; then
        echo "No results."
        return
    fi

    echo -ne "${C_ON}"
    printf '%s\n' "${rows2[@]}" \
        | column \
            --table \
            -s "${SEP_IN}" \
            -o "${SEP_OUT}" \
            -N "${COLUMNS_N}" \
            -R "${COLUMNS_RA}"
    echo -ne "${C_OFF}"
}

# write to markdown table format
function write_rows_markdown () {
    echo
    declare -ar rows2=("${@}")
    printf "${SEP_MD}%s\n" \
        "FILE${SEP_IN}RUST_MIN_STACK${SEP_IN}(B)" \
        ":---${SEP_IN}---:${SEP_IN}---:" \
        "${rows2[@]}" \
        | column \
            --table \
            --table-noheadings \
            -s "${SEP_IN}" \
            -o "${SEP_MD}" \
            -N "${COLUMNS_N}" \
            -R "${COLUMNS_RA}" > "${OUT_MD}"
    echo
    echo "Markdown table written to ${OUT_MD}"
    if which glow &> /dev/null; then
        glow "${OUT_MD}"
    else
        cat "${OUT_MD}"
    fi
}

function sanitize_field() {
    local s="$1"
    # Avoid breaking the internal table separator.
    s=${s//|/¦}
    echo -n "$s"
}

function exit_print_rows() {
    set +e
    echo
    echo "PROGRAM='$PROGRAM'"
    echo "STACK_SIZE_START=$STACK_SIZE_START"
    echo "STACK_SIZE_STEP=$STACK_SIZE_STEP"
    echo "STACK_SIZE_MAX=$STACK_SIZE_MAX"
    echo
    echo "Results:"
    print_rows "${rows[@]}"
    if [[ "${SORT-}" == "true" || "${SORT-}" == "1" ]]; then
        echo
        SORTED=true
        echo "Results sorted by stack size and elapsed time:"
        print_rows "${rows[@]}"
    fi
    write_rows_markdown "${rows[@]}"
}

trap exit_print_rows EXIT
trap "exit 1" SIGINT
trap "exit 1" SIGQUIT

declare -r TIME_START_SCRIPT=$EPOCHREALTIME

echo "To exit early:" >&2
echo "  kill -SIGINT $$" >&2
echo >&2
echo "Run plan:" >&2
for file in "$@"; do
    echo "  $PROGRAM $file" >&2
done
echo >&2

for file in "$@"; do
    if [[ ! -f "$file" ]]; then
        echo "Path not a file $file" >&2
        exit 1
    fi
done

for file in "$@"; do
    declare -i stack_size=$STACK_SIZE_START
    declare -i rc_run=1

    # confirm it can process the file at all
    if ! /usr/bin/env RUST_MIN_STACK=5000000 "$PROGRAM" -- "$file" &>/dev/null; then
        echo -e "\e[31mfailed to process ${file}\e[39m\n"
        continue
    fi

    echo "$file" >&2
    echo -n "${stack_size}" >&2
    while true; do
        echo -n '◦' >&2

        set +e
        (
            ulimit -c 0
            /usr/bin/env RUST_MIN_STACK="$stack_size" \
                "$PROGRAM" -- "$file" &>/dev/null
        ) &>/dev/null
        rc_run=$?
        set -e

        if [[ $rc_run -eq 0 ]]; then
            break
        fi

        next_stack_size=$((stack_size + STACK_SIZE_STEP))
        if [[ $STACK_SIZE_MAX -gt 0 && $next_stack_size -gt $STACK_SIZE_MAX ]]; then
            break
        fi
        stack_size=$next_stack_size
    done

    stack_size_hr="_"
    if [[ $rc_run -eq 0 ]]; then
        stack_size_hr=$(numfmt --to=iec-i -- "$stack_size")
    else
        failed=$((failed + 1))
    fi

    cmd="RUST_MIN_STACK=<step loop from ${STACK_SIZE_START}> ${PROGRAM} -- ${file}"
    row="\
$(sanitize_field "$file")${SEP_IN}\
${stack_size_hr}${SEP_IN}\
${stack_size}${SEP_IN}\
"
    rows+=("$row")

    echo >&2
    print_rows "${rows[@]}"
    echo >&2
done

if [[ $failed -ne 0 ]]; then
    exit 1
fi
