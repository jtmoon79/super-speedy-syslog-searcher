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

#if ! which flamegraph; then
#    echo "flamegraph is not in the PATH" >&2
#    did_install
#    exit 1
#fi

#echo "Cargo.toml must have:
#
#    [profile.bench]
#    debug = true
#    [profile.release]
#    debug = true
#
#" >&2

declare -r bin=./target/release/s4
declare -r bin_target=s4

export CARGO_PROFILE_RELEASE_DEBUG=true
#export RUSTFLAGS="-Clink-arg=-fuse-ld=lld -Clink-arg=-Wl,--no-rosegment"
#export RUSTC_LINKER=$(which clang)
export RUST_BACKTRACE=1

OUT='flamegraph.svg'

(
    set -x
    cargo flamegraph -- --version
)
rm perf.data perf.data.old

# XXX: if $NOTES contains a '--' then flamegraph.svg will fail to render
NOTES=$("${bin}" --version)

declare -a logs=()
while read line; do
    logs[${#logs[@]}]=${line}
done <<< $(sed -Ee 's/\|.*//' ./tools/log-files-time-update.txt | head -n 50)

set -x

cargo flamegraph --version

exec \
cargo flamegraph \
    --verbose \
    --flamechart \
    --deterministic \
    --output "${OUT}" \
    --bin "${bin_target}" \
    --notes "${NOTES}" \
    "${@}" \
    -- \
        --color never \
        -a '20000101T000100' \
        "${logs[@]}" \
        1>/dev/null \
