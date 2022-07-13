#!/usr/bin/env bash
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
export RUST_BACKTRACE=1
OUT='flamegraph.svg'

NOTES=$("${bin}" --version)
# XXX: if $NOTES contains a '--' then flamegraph.svg will fail to render
#if [[ -d '.git' ]]; then
#    NOTES+="; $(git log -n1 --format='%h %D')"
#fi

set -x

cargo flamegraph --version

exec \
cargo flamegraph \
    -v \
    --flamechart \
    --deterministic \
    -o "${OUT}" \
    --bin "${bin_target}" \
    --notes "${NOTES}" \
    "${@}" \
    -- \
        --color never \
        -a '20000101T000100' \
        $(find ./logs/other/tests/ -type f -not \( -name '*.tar' -o -name '*.zip' -o -name 'invalid*' \) ) \
        >/dev/null
