#!/usr/bin/env bash
#
# Cleanup a log directory copied from another real system.
#
# For log directories copied from real systems, "cleans" the directory to
# prepare for committing to public git repository.
#

set -euo pipefail

if [[ "${#}" -lt 1 ]]; then
    echo "Must pass directory path(s) to clean" >&2
    exit 1
fi

for path in "${@}"; do
    (
        set -x
        # remove empty files
        find "${path}" -xdev \
            -type f \
            -empty \
            -print -delete
        # remove symlinks
        find "${path}" -xdev \
            -type l \
            -print -delete
        # remove empty directory paths
        find "${path}" -xdev \
            -type d \
            -empty \
            -print -delete
    )
done
