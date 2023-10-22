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
declare -rg HASHES_STDOUT="${_HERE}/hashes.stdout.txt"
declare -rg HASHES_STDERR="${_HERE}/hashes.stderr.txt"
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

function md5sum_clean () {
    # run md5sum for piped binary data and trim trailing text
    if [[ ${#} -ne 0 ]]; then
        echo "ERROR function md5sum_clean is pipe-only, no arguemnts" >&2
        exit 1
    fi
    md5sum --binary - | sed -Ee 's/ \*-$//'
}

function stderr_clean () {
    # remove text lines from `s4` stderr that vary from run to run
    # $1 is a file path
    #
    # - remove the printing of the current time
    # - remove the printing of the datetime first and last. It might use
    #   the local system timezone
    # - remove warnings as they are printed in an unpredictable order
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
        -e '0,/^\+ \..*$/d' \
        -e '/.*DateTimeParseInstr:.*/d' \
        "${1}"
}

function stderr_clean_1 () {
    # remove datetime text lines from `s4` stderr
    if [[ ${#} -ne 0 ]]; then
        echo "ERROR function stderr_clean_1 reads stdin pipe, no arguments" >&2
        exit 1
    fi
    sed -E \
        -e '/^Datetime Now[ ]*:.*$/d'
}
