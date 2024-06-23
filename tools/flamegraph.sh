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
    echo "(sometimes only need linux-tools-generic)" >&2
}

if [[ ! -d /usr/lib/linux-tools/ ]]; then
    echo "Warning: cannot find '/usr/lib/linux-tools/'" >&2
    did_install
fi

export GIT_PAGER=

PERF=${PERF-"/usr/lib/linux-tools/$(ls -1v /usr/lib/linux-tools/ | tail -n1)/perf"}
if [[ ! -e "${PERF}" ]]; then
    echo "PERF tool does not exist '${PERF}'" >&2
    did_install
    exit 1
fi
if [[ ! -x "${PERF}" ]]; then
    echo "PERF tool is not executable '${PERF}'" >&2
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
    echo "PROGRAM does not exist or is not executable '${PROGRAM}'" >&2
    echo "build with:" >&2
    echo "    RUSTFLAGS=-g cargo build --profile flamegraph" >&2
    exit 1
fi
if ! "${PROGRAM}" --version &>/dev/null; then
    echo "PROGRAM failed to run '${PROGRAM}'" >&2
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
rm -f -- perf.data perf.data.old "${OUT}"
FLAMEGRAPH_VERSION=$(cargo flamegraph --version)
RUST_VERSION_SHORT=$(rustc --version | cut -f1,2 -d' ')

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

NOTES+="; -freq ${FREQ}; created $(date +%Y%m%dT%H%M%S%z); ${FLAMEGRAPH_VERSION}; ${RUST_VERSION_SHORT}"

function html_sed_escape() {
    # escape for HTML and for sed
    python3 -B -E -s -c "\
from html import escape
print(escape(r''' ${1-} '''[1:-1]).replace('/', r'\/'))
"
}

NOTES_ESCAPED=$(html_sed_escape "${NOTES}")
(
set -x

# force important variables to echo in debug output
PERF=${PERF}
CARGO_PROFILE_RELEASE_DEBUG=${CARGO_PROFILE_RELEASE_DEBUG}
RUST_BACKTRACE=${RUST_BACKTRACE}

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
fi
