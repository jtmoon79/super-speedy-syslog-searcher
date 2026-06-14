#!/usr/bin/env bash
#
# This script builds regexes and measures the time and memory usage for each build.
# It prints a summary table of results.
# The purpose is to help diagnose where build times become significant due to
# the number or complexity of regexes being built.
#
# User can control the range of regexes to build by setting $M and/or $N,
# or a single regex by setting $MN, or multiple regexes per build build
# by setting $MSTART.
# Sequences may be jumped by setting $STEP.
# Sorting may be enabled by setting $SORT to "true" or "1".
#
# examples:
#   # build regexes 1 through 10 individually
#   # that is 10 build commands with one regex per build
#   M=1 N=10 ./tools/build-regex-times.sh
#        S4_BUILD_REGEX=1 cargo build
#        S4_BUILD_REGEX=2 cargo build
#        ...
#        S4_BUILD_REGEX=10 cargo build
#
#   # build regexes 10 through 20 individually
#   # that is 10 build commands with one regex per build
#   # sort the results by elapsed time and memory usage
#   M=1 N=10 SORT=true ./tools/build-regex-times.sh
#        S4_BUILD_REGEX=10 cargo build
#        S4_BUILD_REGEX=11 cargo build
#        ...
#        S4_BUILD_REGEX=20 cargo build
#
#   # build regexes 44, 46, 48, 50 individually
#   # that is 4 build commands with one regex per build
#   M=44 N=50 STEP=2 ./tools/build-regex-times.sh
#        S4_BUILD_REGEX=44 cargo build
#        S4_BUILD_REGEX=46 cargo build
#        S4_BUILD_REGEX=48 cargo build
#        S4_BUILD_REGEX=50 cargo build
#
#   # build only regex 5, that is 1 build command with one regex per build
#   MN=5 ./tools/build-regex-times.sh
#        S4_BUILD_REGEX=5 cargo build
#
#   # build regex 1 through 10
#   # first build is regex 1, second build is regex 1 and 2, up to
#   # tenth build is regex 1 through 10
#   M=1 N=10 MSTART=1 ./tools/build-regex-times.sh
#        S4_BUILD_REGEX=1 cargo build
#        S4_BUILD_REGEX=1-2 cargo build
#        ...
#        S4_BUILD_REGEX=1-10 cargo build
#
# Extra arguments are passed to `cargo build`
#
# May also set `$LOG_FILE` for use in stack size testing.
# If not set, a blank file will be used. `$LOG_FILE` may be a file or
# directory path.
# This can be used to measure non-text files, e.g.:
#       LOG_FILE=./logs/programs/journal/CentOS_7_system.journal MN=1 ./tools/build-regex-times.sh -q --release
#

set -euo pipefail

cd "$(dirname -- "${BASH_SOURCE[0]}")/.."

if [[ -z "${EPOCHREALTIME:-}" ]]; then
    echo "EPOCHREALTIME is not available in this shell." >&2
    exit 1
fi

declare -i M=${M-1}
declare -i N=${N-179}  # XXX: keep in sync with DATETIME_PARSE_DATAS_LEN_MAX
# allow a convenience for running a single regex
if [[ -n "${MN-}" ]]; then
    M=$MN
    N=$MN
fi
readonly M N
declare -ag rows=()
declare -rg SEP='|'  # internal column separator
declare -rg BL='┊'  # blank column separator
declare -i rc=0
declare -i failed_rc=0
declare -r tmpf1=$(mktemp -t "s4-tmp-$(basename "$0").time.XXXXX.tmp")
declare -r tmpf2=$(mktemp -t "s4-tmp-$(basename "$0").cargo-expand.XXXXX.tmp")
declare -r tmpd=$(mktemp -d -t "s4-tmp-$(basename "$0").XXXXX.tmpdir")
# summary columns for `column` command
declare -rg COLUMNS_N="\
REGEX,\
COUNT,\
${BL},\
BUILD TIME,\
(s),\
−PRIOR (s),\
−BASELINE (s),\
/REGEX (s),\
${BL},\
BUILD MRSS,\
(KB),\
−BASELINE (KB),\
/REGEX (KB),\
${BL},\
RUN STACK MIN,\
(B),\
−BASELINE (B),\
${BL},\
EXPANDED (LOC),\
−BASELINE,\
${BL},\
COMMAND"
declare -rg COL_TIME=5  # for sort
declare -rg COL_MRSS=11  # for sort
# summary columns to display right aligned
declare -rg COLUMNS_RA=${COLUMNS_N%%COMMAND}
declare -a CMD_BUILD=(cargo build)
if [[ ${#@} -ne 0 ]]; then
    CMD_BUILD+=("$@")
fi
readonly CMD_BUILD

# color on
declare C_ON='\e[93m'  # light yellow
# color off
declare C_OFF='\e[39m'  # default
# start from this stack size and increase
declare -ir STACK_SIZE=${STACK_SIZE-32 * 1024}  # 32 KiB
# increase this amount per attempt
declare -ir STACK_SIZE_STEP=${STACK_SIZE_STEP-128}

function print_row() {
    echo -ne "${C_ON}"
    echo "$1" \
        | column \
            --table \
            -s "$SEP" \
            -N "$COLUMNS_N" \
            -R "$COLUMNS_RA"
    echo -ne "${C_OFF}"
}

declare -g SORTED=false

function print_rows() {
    # sort by elapsed time then by MRSS
    declare -a rows2=()
    if $SORTED; then
        mapfile -t rows2 < <(
            printf '%s\n' "${@}" \
            | sort -t"$SEP" -k${COL_TIME}n -k${COL_MRSS}nr
        )
    else
        rows2=("${@}")
    fi
    if [[ ${#rows2[@]} -eq 0 ]]; then
        echo "No results."
        exit 1
    fi

    # print summary of results
    echo
    echo -ne "${C_ON}"
    printf '%s\n' "${rows2[@]}" \
        | column \
            --table \
            -s "$SEP" \
            -o ' ' \
            -N "$COLUMNS_N" \
            -R "$COLUMNS_RA"
    echo -ne "${C_OFF}"
}

function exit_print_rows() {
    set +e
    rm -f "$tmpf1" "$tmpf2"
    rm -rf "$tmpd"
    echo
    echo "LOG_FILE='${LOG_FILE-}'"
    echo
    echo "Results:"
    print_rows "${rows[@]}"
    echo
    SORTED=true
    echo "Results sorted by build time and memory usage:"
    print_rows "${rows[@]}"
}
# make sure to always print any available results
trap exit_print_rows EXIT
trap "exit 1" SIGINT
trap "exit 1" SIGQUIT

if [[ ! "${LOG_FILE-}" ]]; then
    # Due to s4 return code logic, passing the file directly will
    # cause a non-zero exit code which will be treated as a build failure.
    # But passing a directory will not cause a non-zero exit code. So this file
    # is created in a temporary directory and the directory path passed to s4.
    # So create an unparseable file in the temp directory for the `cargo run` command.
    # This file does not have a datetime but a regex will be attempted.
    # The file must not be under the minimum file size of 6 bytes (s4 will not attempt
    # to process files under that size). It must also contain a `1` or `2` to
    # bypass the `ezcheck12`.
    echo "12                                                                       \
                                                                                " \
    > "$tmpd/blanks.log"
else
    cp -arv "$LOG_FILE" "$tmpd/"
fi

# parse the S4_BUILD_REGEX string to count the number of regexes
# e.g. "1" is 1 regex, "5" is 1 regex, "1-3" is 3 regexes,
# and "3-1" is also 3 regexes
function count_regexes_from_string() {
    local str="$1"
    if [[ "$str" =~ '-' ]]; then
        local start="${str%-*}"
        local stop="${str#*-}"
        if [[ $start -le $stop ]]; then
            echo -n $(((stop - start) + 1))
        else
            echo -n $(((start - stop) + 1))
        fi
    else
        echo -n 1
    fi
}

function float_subtract() {
    awk -v a="$1" -v b="$2" 'BEGIN { printf "%.1f", (a - b) }'
}

function float_divide() {
    awk -v a="$1" -v b="$2" 'BEGIN { printf "%.1f", (a / b) }'
}

function float_le() {
    awk -v a="$1" -v b="$2" 'BEGIN { if (a <= b) exit 0; else exit 1 }'
}

function float_secs_to_hourminsec() {
    awk -v e="$1" 'BEGIN {
        total = int(e)
        hours = int(total / 3600)
        minutes = int((total % 3600) / 60)
        seconds = total % 60
        printf "%dh%02dm%02ds", hours, minutes, seconds
    }'
}

function float_to_int() {
    awk -v e="$1" 'BEGIN { printf "%d", e }'
}

declare -g STACK_SIZE_NOCRASH="_"
declare -g STACK_SIZE_NOCRASH_hr="_"

# find the first stack size where the built program does not crash.
# sets global variables STACK_SIZE_NOCRASH and STACK_SIZE_NOCRASH_hr
function run_to_crash_for_stack_size() {
    declare -r r_=${1}
    declare -i col_at=0

    STACK_SIZE_NOCRASH="_"
    STACK_SIZE_NOCRASH_hr="_"
    # start stack_size at `stack_size` and increase by `stack_size_add` until
    # the program does not crash
    declare -i stack_size=${STACK_SIZE}  # bytes
    declare -ir stack_size_add=${STACK_SIZE_STEP}  # bytes
    # since this will be run many times, just print once in a trace-like manner.
    # tracing every command will fill up the screen
    echo "${PS4}/usr/bin/env S4_BUILD_REGEX=$r_ RUST_MIN_STACK=$stack_size ${CMD_RUN[*]} -- $tmpd &>/dev/null" >&2
    declare -i cols_expect=$((${COLUMNS:-80} - ${#stack_size}))
    echo -n "$stack_size" >&2
    while true; do
        echo -n '◦' >&2
        set +e
        # this is expected to core dump so smother all output to avoid filling up
        # the screen with core dump messages.
        (
            # try to "disable" core dump activity by setting core dump size to zero
            # XXX: this may not work in all environments.
            ulimit -c 0
            /usr/bin/env S4_BUILD_REGEX="$r_" RUST_MIN_STACK="$stack_size" \
                "${CMD_RUN[@]}" -- "$tmpd" &>/dev/null
        ) &>/dev/null
        declare -i rc_run=$?
        set -e
        if [[ $rc_run -eq 0 ]]; then
            STACK_SIZE_NOCRASH=$stack_size
            STACK_SIZE_NOCRASH_hr=$(numfmt --to=iec-i -- "${stack_size}")
            echo >&2
            return
        fi
        stack_size=$((stack_size + stack_size_add))
        # Printing the stack_size for every run would fill up the screen too fast,
        # so only print the stack size once per console row.
        # Track how many dots have been printed.
        # When nearing the end of the console width
        # print a newline and on that next line print the current stack size.
        col_at=$((col_at + 1))
        if [[ $col_at -eq $cols_expect ]]; then
            echo >&2
            echo -n "$stack_size" >&2
            col_at=0
            cols_expect=$((${COLUMNS:-80} - ${#stack_size}))
        fi
    done
}

# print the number of lines of code in the macro expanded file
# only for regex
function cargo_expand_macros() {
    declare -r r="$1"
    (
        set -e
        cd subprojects/ere/ere_datetimes_impl
        set -x
        /usr/bin/env S4_BUILD_REGEX="$r" cargo -q expand 1> "$tmpf2" 2>/dev/null
    )
    # `wc -l` output looks like:
    # 123 /file/path.rs
    wc -l "$tmpf2" | cut -f1 -d' '
}

declare -r TIME_START_SCRIPT=$EPOCHREALTIME

echo "To exit early:" >&2
echo "  kill -SIGINT $$" >&2
echo >&2
# Print the build plan so the user can verify which regexes will be built
# as this script may take a long time to run
echo "Build plan:" >&2
echo "  S4_BUILD_REGEX=1 ${CMD_BUILD[*]} (pre baseline)" >&2
echo "  S4_BUILD_REGEX=0 ${CMD_BUILD[*]} (baseline)" >&2
for n in $(seq $M ${STEP-1} $N); do
    m=''
    if [[ -n "${MSTART-}" ]]; then
        m="${MSTART}-"
    fi
    r="$m$n"
    echo "  S4_BUILD_REGEX=$r ${CMD_BUILD[*]}" >&2
done
echo >&2
sleep 1

elapsed_prior=0

# establish baseline of zero regexes
# the very first run may need to rebuild everything, do not measure it
(
    set -x
    /usr/bin/env S4_BUILD_REGEX=1 \
        "${CMD_BUILD[@]}" 2>/dev/null
)

r=0
cmd="S4_BUILD_REGEX=$r ${CMD_BUILD[*]}"
start="$EPOCHREALTIME"
TARGET=$(
    # The second run is the true baseline of building zero regexes.
    # At the same time, constrain the output to print the built binary path.
    # taken from https://github.com/rust-lang/cargo/issues/3670#issuecomment-2964336289
    # The time cost of this extra step is negligible.
    set -x
    /usr/bin/env S4_BUILD_REGEX=$r \
        /usr/bin/time -o "$tmpf1" -f "%M" -- \
            "${CMD_BUILD[@]}" --message-format=json \
                | jq -r 'select(.target.kind[0] == "bin") | .executable'
)
stop="$EPOCHREALTIME"
declare -r baseline_elapsed=$(float_subtract "$stop" "$start")
elapsed_hourminsec=$(float_secs_to_hourminsec "$baseline_elapsed")
declare -r baseline_mrss=$(cat "$tmpf1" | tail -n1 | tr -d '\n')
mrss_hr=$(numfmt --to=iec-i --from=iec -- "${baseline_mrss}K")
loc_baseline_expanded=$(cargo_expand_macros "$r")
readonly loc_baseline_expanded

if [[ ! -f "$TARGET" ]]; then
    echo "Failed to find binary path '$TARGET'" >&2
    exit 1
fi
if ! $TARGET --version &>/dev/null; then
    echo "Failed to run the built binary '$TARGET'." >&2
    exit 1
fi
echo >&2
echo "TARGET: $TARGET" >&2
echo >&2

declare -arg CMD_RUN=("$TARGET")

run_to_crash_for_stack_size "$r"
declare -ir stack_size_no_crash_baseline="$STACK_SIZE_NOCRASH"

row="\
_${SEP}\
0${SEP}\
${BL}${SEP}\
${elapsed_hourminsec}${SEP}\
${baseline_elapsed}${SEP}\
_${SEP}\
_${SEP}\
_${SEP}\
${BL}${SEP}\
${mrss_hr}${SEP}\
${baseline_mrss}${SEP}\
_${SEP}\
_${SEP}\
${BL}${SEP}\
${STACK_SIZE_NOCRASH_hr}${SEP}\
${stack_size_no_crash_baseline}${SEP}\
_${SEP}\
${BL}${SEP}\
${loc_baseline_expanded}${SEP}\
_${SEP}\
${BL}${SEP}\
${cmd} (BASELINE)"
rows+=("$row")
print_rows "${rows[@]}"
echo

for n in $(seq $M ${STEP-1} $N); do
    m=''
    # MSTART allows building sequences of regexes
    if [[ -n "${MSTART-}" ]]; then
        m="${MSTART}-"
    fi
    r="$m$n"
    cmd="S4_BUILD_REGEX=$r ${CMD_BUILD[*]}"
    # do the build, collect time and memory usage
    set +e
    start=${EPOCHREALTIME}
    (
        set -x
        /usr/bin/timeout -v -k 1m --preserve-status 30m \
            /usr/bin/env S4_BUILD_REGEX="$r" \
                /usr/bin/time -o "$tmpf1" -f "%M" -- \
                    "${CMD_BUILD[@]}"
    )
    declare -i rc_build=$?
    stop=${EPOCHREALTIME}

    loc_expanded="_"
    loc_expanded_minus_baseline="_"
    stack_size_no_crash="_"
    stack_size_no_crash_hr="_"
    stack_size_no_crash_minus_baseline="_"
    # run the built program, find the stack size at which it first does not crash
    if [[ $rc_build -eq 0 ]]; then
        # first get macro expanded
        loc_expanded=$(cargo_expand_macros "$r")
        loc_expanded_minus_baseline=$(float_to_int $(float_subtract "$loc_expanded" "$loc_baseline_expanded"))
        # then determine stack size
        run_to_crash_for_stack_size "$r"
        stack_size_no_crash="${STACK_SIZE_NOCRASH}"
        stack_size_no_crash_hr="${STACK_SIZE_NOCRASH_hr}"
        stack_size_no_crash_minus_baseline=$(float_to_int $(float_subtract "$stack_size_no_crash" "$stack_size_no_crash_baseline"))
    fi

    # count of regexes compiled
    regex_count=$(count_regexes_from_string "$r")

    # time in seconds
    elapsed=$(float_subtract "$stop" "$start")
    # time in hours, minutes, and seconds
    elapsed_hourminsec=$(float_secs_to_hourminsec "$elapsed")
    # difference in elapsed time from the previous regex
    if float_le "$elapsed_prior" 0; then
        elapsed_diff=$elapsed
    else
        elapsed_diff=$(float_subtract "$elapsed" "$elapsed_prior")
    fi
    elapsed_prior=$elapsed
    elapsed_minus_baseline=$(float_subtract "$elapsed" "$baseline_elapsed")
    # average time to build each regex
    elapsed_minus_baseline=$(float_subtract "$elapsed" "$baseline_elapsed")
    elapsed_per_regex=$(float_divide "$elapsed_minus_baseline" "$regex_count")

    # maximum resident set size
    mrss=$(cat "$tmpf1" | tail -n1 | tr -d '\n')
    if [[ "$mrss" == "" ]]; then
        mrss=0
    fi
    # human readable `mrss`
    mrss_hr=$(numfmt --to=iec-i --from=iec -- "${mrss}K")
    mrss_minus_baseline=$(float_to_int $(float_subtract "$mrss" "$baseline_mrss"))
    # MRSS per regex
    mrss_per_regex=$(float_to_int $(float_divide "$mrss_minus_baseline" "$regex_count"))

    if [[ $rc_build -ne 0 ]]; then
        r="‼ $r"
        failed_rc+=1
    fi
    row="\
${r}${SEP}\
${regex_count}${SEP}\
${BL}${SEP}\
${elapsed_hourminsec}${SEP}\
${elapsed}${SEP}\
${elapsed_diff}${SEP}\
${elapsed_minus_baseline}${SEP}\
${elapsed_per_regex}${SEP}\
${BL}${SEP}\
${mrss_hr}${SEP}\
${mrss}${SEP}\
${mrss_minus_baseline}${SEP}\
${mrss_per_regex}${SEP}\
${BL}${SEP}\
${stack_size_no_crash_hr}${SEP}\
${stack_size_no_crash}${SEP}\
${stack_size_no_crash_minus_baseline}${SEP}\
${BL}${SEP}\
${loc_expanded}${SEP}\
${loc_expanded_minus_baseline}${SEP}\
${BL}${SEP}\
${cmd}"
    rows+=("$row")
    # print results as updated because this script can take a long
    # time to run, and user wants to see progress as it happens
    print_rows "${rows[@]}"
    # for long runs, print datetime helps to know how long ago each result
    # was printed
    declare time_until_now=$(float_subtract "$EPOCHREALTIME" "$TIME_START_SCRIPT")
    declare time_until_now_hourminsec=$(float_secs_to_hourminsec "$time_until_now")
    echo
    echo -e "\e[37mTime so far ${time_until_now_hourminsec}${C_OFF}"
    echo
done

if [[ $failed_rc -ne 0 ]]; then
    exit 1
fi
