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
LOGS_COUNT=$(cat "${LOGS}" | sed '/^$/d' | wc -l)
declare -ir LOGS_COUNT
export TZ='America/New_York'

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
    --dt-before='20260102T000000+0000'
    --blocksz='0x100'
    --summary
)
declare -rg S4_ARGS_QUOTED=$(for arg in "${S4_ARGS[@]}"; do echo -n "'${arg}' "; done)
declare -rg S4_VENV_PIP=~/.config/s4/venv/pip.conf

function stderr_clean () {
    # remove text lines from `s4` stderr that vary from run to run
    # $1 is a file path
    #
    # - remove the printing of the current time `Datetime Now`
    # - remove the printing of `Modified Time` as it may vary based on the
    #   filesystem.
    # - remove the realpath as it varies depending on the repo. path.
    # - remove the temporary path as it always varies.
    # - remove warnings as they are printed in an unpredictable order
    # - remove `streaming: `, `blocks high:`, `lines high:`, `caching:` from the
    #   "streaming" summary. Explained in Issue #213
    # - remove Python process reads/writes/polls/waits/runtime as they vary
    #   depending on system load.
    # - remove `Python Interpreter` as it varies depending on the system.
    # - remove `Program Run Time` as it varies depending on system load.
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
        -e '/^[ ]+Modified Time [ ]*:.*$/d' \
        -e '/^[ ]+modified time [ ]*:.*$/d' \
        -e '/^[ ]+Python process reads stderr[ ]*:.*$/d' \
        -e '/^[ ]+Python process reads stdout[ ]*:.*$/d' \
        -e '/^[ ]+Python process writes stdin[ ]*:.*$/d' \
        -e '/^[ ]+Python pipe recv stderr[ ]*:.*$/d' \
        -e '/^[ ]+Python pipe recv stdout[ ]*:.*$/d' \
        -e '/^[ ]+Python process polls[ ]*:.*$/d' \
        -e '/^[ ]+Python process waits[ ]*:.*$/d' \
        -e '/^[ ]+Python process runtime[ ]*:.*$/d' \
        -e '/^[ ]+Python script arguments[ ]*:.*$/d' \
        -e '/^[ ]+realpath .*$/d' \
        -e '/^[ ]+real path .*$/d' \
        -e '/^[ ]+temporary path .*$/d' \
        -e '/^[ ]+streaming: .*$/d' \
        -e '/^[ ]+caching: BlockReader::read_block.*$/d' \
        -e '/^[ ]+storage: BlockReader::read_block.*$/d' \
        -e '/^[ ]+blocks high[ ]+: .*$/d' \
        -e '/^[ ]+lines high[ ]+: .*$/d' \
        -e '/^Datetime Now[ ]*:.*$/d' \
        -e '/^Python Interpreter [ ]*:.*$/d' \
        -e '/^Program Run Time[ ]+: .*$/d' \
        -e '/^ERROR: .*$/d' \
        -e '/.*DateTimeParseInstr:.*/d' \
        "${1}"
}
