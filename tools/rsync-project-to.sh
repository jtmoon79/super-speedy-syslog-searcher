#!/usr/bin/env bash
#
# rsync this project to another directory.
# only copy relevant project files, not `target` directory, `.git` directory,
# and other temporary files.
#
# On occasion, I want to quickly rsync this project to another directory,
# and then run some `cargo` command in that directory.
#

set -eu

if [[ ${#} != 1 ]]; then
    echo "usage: ${0} <target-directory>" >&2
    exit 1
fi

cd "$(dirname "${0}")/.."
HERE="$(basename -- "$(realpath .)")"
RSYNC=$(which rsync)
TARGET=${1}

if [[ ! -e "${TARGET}" ]]; then
    mkdir -vp -- "${TARGET}"
fi
if [[ ! -d "${TARGET}" ]]; then
    echo "ERROR: ${TARGET} exists and is not a directory" >&2
    exit 1
fi
if [[ -n "$(ls -A "${TARGET}/" 2>/dev/null)" ]]; then
    echo "ERROR: directory ${TARGET} is not empty" >&2
    exit 1
fi

set -x
exec \
"${RSYNC}" \
    --verbose \
    --recursive \
    --archive \
    --copy-links \
    --exclude 'subprojects/bindgen/target/' \
    ./benches \
    ./Cargo.toml \
    ./Cargo.lock \
    ./CHANGELOG.md \
    ./Extended-Thoughts.md \
    ./.github \
    ./LICENSE.txt \
    ./logs \
    $(ls -d1 \
        ./valgrind \
        ./Notes.txt \
        ./flamegraph.svg* \
        ./tests \
        2>/dev/null || true
    ) \
    ./README.md \
    ./rustfmt.toml \
    ./src \
    ./subprojects \
    ./tools \
    "${TARGET}" \
