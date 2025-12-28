#!/usr/bin/env bash
#
# backup.sh
#
# a quick manual backup script using 7zip
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

if ! Zz=$(which 7z); then
    echo "7z not found in PATH" >&2
    exit 1
fi

HERE="$(basename -- "$(realpath .)")"
ZIPFILE="${BACKUPDIR}/${HERE}-$(date '+%Y%m%dT%H%M%S')-$(hostname).zip"

(
set -x

# if this fails then `7z` is probably not the 7-zip this script expects
"${Zz}" i &> /dev/null

# backup the project!
# ignore `target` directories in bindgen/ and the root
"${Zz}" a -spf -ssc -bb1 -bt -stl -snl -scsUTF-8 -tzip -- "${ZIPFILE}" \
    ./benches/ \
    ./Cargo.toml \
    ./Cargo.lock \
    ./CHANGELOG.md \
    ./deny.toml \
    ./Extended-Thoughts.md \
    ./.gitattributes \
    ./.git_hooks/ \
    ./.github/ \
    ./.gitignore \
    ./LICENSE.txt \
    ./logs/ \
    $(ls -d1 \
        ./performance-data \
        ./valgrind \
        *.log \
        ./Notes.txt \
        ./*.svg \
        ./tests \
        ./trace.fxt.gz \
        2>/dev/null || true
    ) \
    ./.mlc.toml \
    ./README.md \
    ./releases/ \
    ./rustfmt.toml \
    ./.taplo.toml \
    ./src/ \
    $(find ./subprojects/bindgen/ \
        -mindepth 1 -maxdepth 1 \
        -not -name 'target') \
    ./tools/ \
    ./.vscode/ \

"${Zz}" l "${ZIPFILE}"
)

# remove write permissions from the archive file
# this may fail on NTFS or SMB mounts
chmod -w -- "${ZIPFILE}" || true

ls -lh "${ZIPFILE}"
