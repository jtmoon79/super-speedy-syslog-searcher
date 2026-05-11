#!/usr/bin/env bash
#
# backup.sh
#
# a quick manual backup script using zip
#
# set BACKUPDIR to the final backup destination, defaults to ".."
#

set -euo pipefail

cd "$(dirname "${0}")/.."

if [[ ! -d "${BACKUPDIR-.}" ]]; then
    echo "BACKUPDIR is not a directory or does not exist '${BACKUPDIR-}'" >&2
    exit 1
fi

BACKUPDIR=${BACKUPDIR:-".."}

if [[ ! -w "${BACKUPDIR}" ]]; then
    echo "BACKUPDIR is not writable '${BACKUPDIR}'" >&2
    exit 1
fi

HERE="$(basename -- "$(realpath .)")"
ZIPFILE="${BACKUPDIR}/${HERE}-$(date '+%Y%m%dT%H%M%S')-$(hostname).zip"

(
set -x
zip --version
find . \( -type d \( -name target -or -name .venv \) -prune \) -or -type f -print \
    | sort \
    | zip -9 "${ZIPFILE}" -o -@
)

# remove write permissions from the archive file
# this may fail on NTFS or SMB mounts
chmod -v -w -- "${ZIPFILE}" || true

ls -lh "${ZIPFILE}"
