#!/usr/bin/env bash
#
# install:
#   apt install -y linux-perf linux-tools-generic
#
# add to Cargo.toml
#   [profile.bench]
#   debug = true
#   [profile.release]
#   debug = true

set -euo pipefail

cd "$(dirname -- "${0}")/.."

function did_install () {
    echo "Did you install:" >&2
    echo "    linux-perf" >&2
    echo "    linux-tools-generic" >&2 
}

export CARGO_PROFILE_RELEASE_DEBUG=true
PERF="/usr/lib/linux-tools/$(ls -1v /usr/lib/linux-tools/ | tail -n1)/perf"
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

echo "Cargo.toml must have:

    [profile.bench]
    debug = true
    [profile.release]
    debug = true

" >&2

export RUST_BACKTRACE=1
(
    set -x
    cargo build --release
    flamegraph -o flamegraph.svg \
      ./target/release/super_speedy_syslog_searcher \
      --path $(find ./logs/other/tests/ -type f -not \( -name '*.gz' -o -name '*.xz' -o -name '*.tar' -o -name '*.zip' \) ) \
      -- 0xFFFFF '20000101T000100' >/dev/null
)
