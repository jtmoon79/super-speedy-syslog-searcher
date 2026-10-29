#!/usr/bin/env bash

set -euo pipefail

if [[ -z "${BENCHER_RUN_KEY:-}" ]]; then
    echo "ERROR: BENCHER_RUN_KEY must be set" >&2
    exit 1
fi
if [[ -z "${BENCHER_REPORT:-}" ]]; then
    echo "ERROR: BENCHER_REPORT must be set" >&2
    exit 1
fi
BENCHER_PROJECT=${BENCHER_PROJECT:-super-speedy-syslog-searcher}

set -x

exec \
    curl \
    --fail-with-body \
    --show-error \
    --request DELETE \
    --header "Authorization: Bearer ${BENCHER_RUN_KEY}" \
    "https://api.bencher.dev/v0/projects/${BENCHER_PROJECT}/reports/${BENCHER_REPORT}"
