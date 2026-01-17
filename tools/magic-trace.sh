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
