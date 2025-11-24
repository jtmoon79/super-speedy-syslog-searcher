#!/usr/bin/env bash
#
# Run `cargo flamegraph` with preferred options.
# Creates a CPU flamegraph of the program run.
#
# build the program with:
#
#     RUSTFLAGS=-g cargo build --profile flamegraph
#
# install:
#   apt install -y linux-perf linux-tools-generic
#   cargo install flamegraph
#
# note the `perf` program installed to `/usr/bin/perf` is often a stub.
#
# noted at https://nnethercote.github.io/perf-book/profiling.html
#
# User may set environment variable $PROGRAM and $BIN.
# Arguments passed to this script are passed to $BIN and override default
# arguments.
#

set -euo pipefail

cd "$(dirname -- "${0}")/.."

function did_install () {
    echo "Did you:" >&2
    echo "    cargo install flamegraph" >&2
    echo "    apt install -y linux-perf linux-tools-generic" >&2
    echo "or for newer Debian-based distros:" >&2
    echo "    apt install -y linux-tools-generic"
    echo "" >&2
    echo "You can also set environment variable 'PERF'" >&2
}

export GIT_PAGER=

# make a best-effort attempt to find the `perf` program which may reside
# at an unusual path not in the environment PATH.
# the perf installed to `/usr/bin/perf` is often a stub.
# print the full path of perf.
function perf_path() {
    local perf_path_candidate
    for perf_path_candidate in \
        "/usr/lib/linux-tools/$(ls -1v /usr/lib/linux-tools/ 2>/dev/null | tail -n1 || true)/perf" \
        "/usr/lib/linux-tools-$(uname -r)/perf" \
        "$(find /usr/lib/ -name perf -type f 2>/dev/null | sort | head -n1 || true)" \
        "/usr/lib64/perf" \
        "/usr/lib/perf" \
        "/usr/lib/linux-tools/$(uname -r)/perf" \
        "$(which perf 2>/dev/null)"
    do
        if [[ -e "${perf_path_candidate}" ]]; then
            echo -n "${perf_path_candidate}"
            return 0
        fi
    done

    return 1
}

if [[ ! "${PERF+x}" ]]; then
    PERF=${PERF-$(perf_path)} || true
    if [[ ! -e "${PERF}" ]]; then
        echo "WARNING: PERF tool not found at '${PERF}'" >&2
        did_install
        exit 1
    fi
fi
readonly PERF
if [[ ! -e "${PERF}" ]]; then
    echo "ERROR: PERF tool does not exist '${PERF}'" >&2
    did_install
    exit 1
fi
echo "using PERF at '${PERF}'" >&2
if [[ ! -x "${PERF}" ]]; then
    echo "ERROR: PERF tool is not executable '${PERF}'" >&2
    exit 1
fi
export PERF

(
    export PAGER=cat
    set -x
    cargo flamegraph --version
    "${PERF}" --version
)

declare -r PROGRAM=${PROGRAM-./target/flamegraph/s4}
if [[ ! -x "${PROGRAM}" ]]; then
    echo "ERROR: PROGRAM does not exist or is not executable '${PROGRAM}'" >&2
    echo "build with:" >&2
    echo "    RUSTFLAGS=-g cargo build --profile flamegraph" >&2
    exit 1
fi
if ! "${PROGRAM}" --version &>/dev/null; then
    echo "ERROR: PROGRAM failed to run '${PROGRAM}'" >&2
    exit 1
fi
declare -r BIN=${BIN-s4}

export CARGO_PROFILE_RELEASE_DEBUG=true
export RUSTFLAGS=-g
export RUST_BACKTRACE=1
#export RUSTC_LINKER=$(which clang)

OUT=${OUT-flamegraph.svg}

# Sampling frequency.
# This is higher than default 997, hopefully it does not cause CPU/IO overload
# warning and dropped chunks (found by trial and error, probably host dependent).
FREQ=${FREQ-3000}

(
    set -x
    # verify flamegraph can run the binary (s4 only prints it's version)
    cargo flamegraph \
        --verbose \
        --flamechart \
        --profile flamegraph \
        --deterministic \
        --output "${OUT}" \
        --bin "${BIN}" \
        --root \
        --ignore-status \
        --no-inline \
        --freq ${FREQ} \
        -- --version
)
echo >&2
echo >&2
rm -f -- perf.data perf.data.old "${OUT}"
FLAMEGRAPH_VERSION=$(cargo flamegraph --version)
RUST_VERSION_SHORT=$(rustc -vV | sed -n 's|release: ||p')
RUST_HOST=$(rustc -vV | sed -n 's|host: ||p')

NOTES=$("${PROGRAM}" --version)
if GITLOG_HASH1=$(git log -n1 --pretty=format:%h 2>/dev/null); then
    NOTES+="; git: ${GITLOG_HASH1}"
fi

declare -a args=()
if [[ ${#} -ge 1 ]]; then
    # use user-passed arguments
    for arg in "${@}"; do
        args+=("${arg}")
        shift
    done
else
    # default arguments
    args+=(
        --color never
        -a '20000101T000100'
    )
    while read line; do
        args+=("${line}")
        # use first 50 files listed in `log-files-time-update.txt`
        # append a few known files of varying types
    done <<< $(sed -Ee 's/\|.*//' ./tools/log-files-time-update.txt \
               | sed -Ee '/^#/d' \
               | head -n 50;
                echo './logs/other/tests/gen-1000-3-foobar.log.gz'
                echo './logs/other/tests/gen-100-10-skullcrossbones.log.xz'
                echo './logs/other/tests/gen-100-10-skullcrossbones.tar'
                echo './logs/programs/journal/CentOS_7_system.journal'
                echo './logs/programs/journal/RHE_91_system.journal'
                echo './logs/programs/journal/Ubuntu22-user-1000.journal'
                echo './logs/programs/evtx/Microsoft-Windows-Kernel-PnP%4Configuration.evtx'
               )
fi

NOTES+="; -freq ${FREQ}; created $(date +%Y%m%dT%H%M%S%z); ${FLAMEGRAPH_VERSION}; rust ${RUST_VERSION_SHORT} ${RUST_HOST}"

function html_sed_escape() {
    # escape for HTML and for sed
    python3 -B -E -s -c "\
from html import escape
print(escape(r''' ${1-} '''[1:-1]).replace('/', r'\/'))
"
}

NOTES_ESCAPED=$(html_sed_escape "${NOTES}")
(

echo PERF=${PERF} >&2
echo CARGO_PROFILE_RELEASE_DEBUG=${CARGO_PROFILE_RELEASE_DEBUG} >&2
echo RUST_BACKTRACE=${RUST_BACKTRACE} >&2

set -x

cargo flamegraph \
    --verbose \
    --flamechart \
    --profile flamegraph \
    --deterministic \
    --output "${OUT}" \
    --bin "${BIN}" \
    --notes "foo<br/>bar<br/>${NOTES_ESCAPED}" \
    --root \
    --ignore-status \
    --no-inline \
    --freq ${FREQ} \
    "${@}" \
    -- \
        "${args[@]}" \
        1>/dev/null \
)

# Forcibly update the .svg title element with $NOTES and $args
# The title element looks like:
#      <text id="title" fill="rgb(0,0,0)" x="50.0000%" y="24.00">Flame Graph</text>

BIN_ESCAPED=$(html_sed_escape "${BIN}")
ARGS_ESCAPED=$(html_sed_escape "${args[*]}")

sed -i -Ee 's/(<text id="title" .*>)Flame Graph(<\/text>)/\1Flame Graph: '"${NOTES_ESCAPED}"'<br\/>; command: '"${BIN_ESCAPED}"' '"${ARGS_ESCAPED}"'\2/' -- "${OUT}"
# the title is now a long string so make the font smaller
sed -i -Ee 's/<text id="title" /<text id="title" style="font-size:xx-small" /' --  "${OUT}"

if which xmllint &>/dev/null; then
    # the generated .svg file is a few huge lines so make it git-friendly (more lines more often)
    xmllint --format --recover --output "${OUT}" "${OUT}"
else
    echo "WARNING: xmllint not found; skip formatting of ${OUT}" >&2
    echo "         apt install libxml2-utils" >&2
fi
