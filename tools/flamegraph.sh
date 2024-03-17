#!/usr/bin/env bash
#
# Run `cargo flamegraph` with preferred options.
# Creates a CPU flamegraph of the program run.
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
    set -x
    cargo flamegraph --version
    "${PERF}" --version
)

declare -r PROGRAM=${PROGRAM-./target/flamegraph/s4}
declare -r BIN=${BIN-s4}

export CARGO_PROFILE_RELEASE_DEBUG=true
#export RUSTFLAGS="-Clink-arg=-fuse-ld=lld -Clink-arg=-Wl,--no-rosegment"
#export RUSTC_LINKER=$(which clang)
export RUST_BACKTRACE=1

OUT='flamegraph.svg'

(
    set -x
    # verify flamegraph can run the binary (just prints the version)
    # ignore errors; will not capture any data because the program run-time is
    # too short
    cargo flamegraph --profile flamegraph --bin "${BIN}" -- --version
) || true
rm -f -- perf.data perf.data.old "${OUT}"

# NOTE: if $NOTES contains a '--' then .svg will fail to render
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
                echo './logs/programs/journal/Ubuntu20_system.journal'
                echo './logs/programs/journal/Ubuntu22-user-1000.journal'
                echo './logs/programs/evtx/Microsoft-Windows-Kernel-PnP%4Configuration.evtx'
               )
fi

# Sampling frequency.
# This is higher than default 997 and should not cause CPU/IO overload
# warning and dropped chunks (found by trial and error, probably host dependent).
FREQ=${FREQ-3000}

NOTES+="; -freq ${FREQ}; $(date +%Y%m%dT%H%M%S%z)"

set -x

# force important variables to echo in debug output
PERF=${PERF}
CARGO_PROFILE_RELEASE_DEBUG=${CARGO_PROFILE_RELEASE_DEBUG}
RUST_BACKTRACE=${RUST_BACKTRACE}

exec \
cargo flamegraph \
    --verbose \
    --flamechart \
    --profile flamegraph \
    --deterministic \
    --output "${OUT}" \
    --bin "${BIN}" \
    --notes "${NOTES}" \
    --root \
    --ignore-status \
    --freq ${FREQ} \
    "${@}" \
    -- \
        "${args[@]}" \
        1>/dev/null \
