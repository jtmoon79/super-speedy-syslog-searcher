#!/usr/bin/env bash
#
# helper to run magic-trace with preferred options.
# arguments are passed to s4
#
# s4 must be built with
#    RUSTFLAGS=-g cargo build --profile flamegraph
#

set -euo pipefail

if [[ ${#} -eq 0 ]]; then
    echo "Usage: ${0} [s4 arguments]" >&2
    exit 1
fi

MAGIC_TRACE=${MAGIC_TRACE-$(which magic-trace 2>/dev/null || true)}
if ! command -v "${MAGIC_TRACE}" &>/dev/null 2>&1; then
    echo "magic-trace not found in the path." >&2
    echo "Please download from https://github.com/janestreet/magic-trace/releases" >&2
    echo "You can also set environment variable MAGIC_TRACE to the path to a custom binary." >&2
    exit 1
fi
(
    # verify magic-trace is functional
    set -x
    "${MAGIC_TRACE}" version
)

# make a best-effort attempt to find the `perf` program which may reside
# at an unusual path not in the environment PATH.
# the perf installed to `/usr/bin/perf` is often a stub.
# print the full path of perf.
# TODO: move `perf_path` to a sourced `common.sh`
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
        echo "ERROR: PERF tool not found at '${PERF}'" >&2
        exit 1
    fi
fi
export PERF
echo "using perf '$PERF'" >&2
export PATH=${PATH}:$(dirname "$PERF")

S4_PROGRAM=${S4_PROGRAM-"$(dirname -- "${0}")/../target/flamegraph/s4"}
S4_DIR=$(dirname "${S4_PROGRAM}")

if [[ ! -f "${S4_PROGRAM}" ]]; then
    echo "Must build with --profile flamegraph" >&2
    echo "Run:" >&2
    echo "    RUSTFLAGS=-g cargo build --profile flamegraph" >&2
    exit 1
fi

set -x

exec \
    "${MAGIC_TRACE}" \
        run \
        -multi-thread \
        -working-directory "${S4_DIR}" \
        -full-execution \
        "${S4_PROGRAM}" \
        -- "$@"
