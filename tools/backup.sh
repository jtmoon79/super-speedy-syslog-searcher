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

Zz=$(which 7z)

# limit archived log files to 30M or less
declare -a logs=()
while read log; do
    logs[${#logs[@]}]=${log}
done <<< $(find ./logs -xdev -type f -size -30M | sort)

HERE="$(basename -- "$(realpath .)")"
ZIPFILE="${BACKUPDIR}/${HERE}-$(date '+%Y%m%dT%H%M%S')-$(hostname).zip"

(
set -x

# if this fails then `7z` is probably not the 7-zip this script expects
"${Zz}" i &> /dev/null

# backup the project!
# ignore `target` directories in bindgen/ and the root
"${Zz}" a -spf -ssc -bb1 -bt -stl -snl -tzip -- "${ZIPFILE}" \
    ./benches/ \
    $(find ./bindgen/ \
        -mindepth 1 -maxdepth 1 \
        -not -name 'target') \
    ./Cargo.toml \
    ./Cargo.lock \
    ./CHANGELOG.md \
    ./Extended-Thoughts.md \
    ./.github/ \
    ./.gitignore \
    ./LICENSE.txt \
    "${logs[@]}" \
    $(ls -d1 \
        ./performance-data \
        ./valgrind \
        ./Notes.txt \
        ./flamegraph.svg* \
        ./flamegraph*.svg \
        ./releases \
        ./tests \
        ./.vscode \
        2>/dev/null || true
    ) \
    ./README.md \
    ./rustfmt.toml \
    ./src/ \
    ./tools/ \

"${Zz}" l "${ZIPFILE}"
)

# remove write permissions from the archive file
chmod -w -- "${ZIPFILE}" || true

ls -lh "${ZIPFILE}"
