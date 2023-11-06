# common.sh
#
# common variables for `compare.sh` and `update.sh`
#
# presumes $PWD is the project root
#
# `common.sh` should be sourced by both scripts

PROGRAM=${PROGRAM-./target/release/s4}
readonly PROGRAM
if [[ ! -e "${PROGRAM}" ]]; then
    echo "ERROR no PROGRAM at '${PROGRAM}'" >&2
    exit 1
fi

# assuming $PWD is project root
declare -rg _HERE='./tools/compare-current-and-expected'
if [[ ! -d "${_HERE}" ]]; then
    echo "ERROR cannot find directory '${_HERE}'; was this run from the project root?"
    exit 1
fi
declare -rg CURRENT_OUT="${_HERE}/current.stdout"
declare -rg EXPECT_OUT="${_HERE}/expected.stdout"
declare -rg CURRENT_ERR="${_HERE}/current.stderr"
declare -rg EXPECT_ERR="${_HERE}/expected.stderr"
declare -rg LOGS="${_HERE}/logs.txt"

declare -arg S4_ARGS=(
    --color=never
    --tz-offset='+08:00'
    --prepend-filepath
    --prepend-file-align
    --prepend-utc
    --prepend-dt-format='%Y%m%dT%H%M%S.%9f'
    --prepend-separator='┋'
    --separator='⇳\n'
    --journal-output=export
    --dt-after='19990303T000000+0000'
    --dt-before='20230410T221032+0000'
    --blocksz='0x100'
    --summary
)
declare -rg S4_ARGS_QUOTED=$(for arg in "${S4_ARGS[@]}"; do echo -n "'${arg}' "; done)

function stderr_clean () {
    # remove text lines from `s4` stderr that vary from run to run
    # $1 is a file path
    #
    # - remove the printing of the current time `Datetime Now`
    # - remove the printing of the `datetime first` and `datetime last`.
    #   It might use the local system timezone which varies from system to
    #   system.
    # - remove the printing of `Modified Time` as it may vary based on the
    #   filesystem.
    # - remove the realpath as it varies depending on the repo. path.
    # - remove warnings as they are printed in an unpredictable order
    # - remove `streaming: `, `blocks high:`, `lines high:` from the
    #   "streaming" summary. Explained in Issue #213
    # - remove `ERROR:` because they are sometimes printed by processing threads
    #   and so the timing of prints may vary
    # - remove `DateTimeParseInstr` as it varies due to changes in the
    #   `datetime.rs` as `DateTimeParseInstr` includes a line number.
    # - remove `ERROR:` as it varies. See Issue #224
    if [[ ${#} -ne 1 ]]; then
        echo "ERROR function stderr_clean must be passed one file argument" >&2
        exit 1
    fi
    sed -i \
        -E \
        -e '/^Datetime Now[ ]*:.*$/d' \
        -e '/^[ ]*datetime first[ ]*.*$/d' \
        -e '/^[ ]*datetime last[ ]*.*$/d' \
        -e '/^Datetime printed first[ ]*:.*$/d' \
        -e '/^Datetime printed last[ ]*:.*$/d' \
        -e '/^[ ]+Modified Time [ ]*:.*$/d' \
        -e '/^[ ]+realpath .*$/d' \
        -e '/^[ ]+streaming: .*$/d' \
        -e '/^[ ]+blocks high[ ]+: .*$/d' \
        -e '/^[ ]+lines high[ ]+: .*$/d' \
        -e '/^ERROR: .*$/d' \
        -e '0,/^\+ \..*$/d' \
        -e '/.*DateTimeParseInstr:.*/d' \
        "${1}"
}
