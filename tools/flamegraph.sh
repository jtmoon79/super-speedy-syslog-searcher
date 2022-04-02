#!/usr/bin/env bash
#
# install:
#   apt install -y linux-perf linux-tools-generic
#   cargo install flamegraph
#

set -euo pipefail

cd "$(dirname -- "${0}")/.."

function did_install () {
    echo "Did you:" >&2
    echo "    apt install -y linux-perf linux-tools-generic" >&2
    echo "    cargo install flamegraph" >&2
}

PERF=${PERF-"/usr/lib/linux-tools/$(ls -1v /usr/lib/linux-tools/ | tail -n1)/perf"}
if [[ ! -e "${PERF}" ]]; then
    echo "PERF tool does not exist '${PERF}'" >&2
    did_install
    exit 1
fi
export PERF

if ! which flamegraph; then
    echo "flamegraph is not in the PATH" >&2
    did_install
    exit 1
fi

#echo "Cargo.toml must have:
#
#    [profile.bench]
#    debug = true
#    [profile.release]
#    debug = true
#
#" >&2

export CARGO_PROFILE_RELEASE_DEBUG=true
export RUST_BACKTRACE=1
(
    set -x
    cargo flamegraph --version
    cargo flamegraph -v --deterministic -o 'flamegraph.svg' "${@}" -- \
        ./target/release/super_speedy_syslog_searcher \
          -z 0x10000 -a '20000101T000100' \
           $(find ./logs/other/tests/ -type f -not \( -name '*.gz' -o -name '*.xz' -o -name '*.tar' -o -name '*.zip' -o -name 'invalid*' \) ) \
           >/dev/null
    #flamegraph -o flamegraph.svg \
    #  ./target/release/super_speedy_syslog_searcher \
    #  --path $(find ./logs/other/tests/ -type f -not \( -name '*.gz' -o -name '*.xz' -o -name '*.tar' -o -name '*.zip' -o -name 'invalid*' \) ) \
    #  -- 0xFFFFF '20000101T000100' >/dev/null
)
