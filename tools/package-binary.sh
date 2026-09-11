#!/usr/bin/env bash
#
# Package an s4 binary for release.
#
# env. vars:
#   DIROUT output directory for the packaged binary and release zip

set -eEuo pipefail

readonly SCRIPT_NAME=$(basename "$0")
readonly BIN="s4"

function usage() {
    cat >&2 <<EOF
Usage: ${SCRIPT_NAME} BINARY_PATH TARGET VERSION

Package BINARY_PATH for TARGET and VERSION into DIROUT.
EOF
}

function exit_error() {
    exit 1
}

trap exit_error ERR

if [[ $# -ne 3 ]]; then
    usage
    exit 1
fi

if [[ ! "${DIROUT-}" ]]; then
    echo "ERROR must set DIROUT" >&2
    exit 1
fi

readonly s4_file="$1"
readonly TARGET="$2"
readonly VERSION="$3"

if [[ ! -f "${s4_file}" ]]; then
    echo "ERROR: file not found '${s4_file}'" >&2
    exit 1
fi

EXT=''
if [[ "${s4_file}" =~ .*\.exe ]]; then
    EXT='.exe'
fi
readonly EXT

mkdir -p "${DIROUT}"
DIROUT=$(readlink -f "${DIROUT}")
readonly DIROUT
RELEASE_DIR="${DIROUT}/release"
mkdir -p "${RELEASE_DIR}"
readonly RELEASE_DIR

readonly dest_name="${BIN}_${TARGET}_v${VERSION}${EXT}"
readonly dest_path="${DIROUT}/${dest_name}"
readonly zip_name="${BIN}_${TARGET}_v${VERSION}.zip"
readonly zip_path="${RELEASE_DIR}/${zip_name}"

if [[ -e "${zip_path}" ]]; then
    echo "ERROR: destination zip already exists '${zip_path}'" >&2
    exit 2
fi

for output_path in "${dest_path}" "${dest_path}.sha256" "${zip_path}.sha256"; do
    if [[ -e "${output_path}" ]]; then
        echo "ERROR: destination already exists '${output_path}'" >&2
        exit 1
    fi
done

function create_sha256sum() {
    declare -r file_path="$1"
    if [[ ! -f "$file_path" ]]; then
        echo "ERROR: file not found '$file_path'" >&2
        return 1
    fi
    declare -r file_name=$(basename "$file_path")
    pushd "$(dirname "$file_path")"
    (set -x; sha256sum "$file_name") > "${file_name}.sha256"
    chmod -v -w "${file_name}.sha256"
    popd
}

readonly bin="${BIN}${EXT}"

function cleanup() {
    rm -f "${DIROUT}/${bin}" "${DIROUT}/${bin}.sha256"
}

trap cleanup EXIT

# The zip file layout must match section `package.metadata.binstall` from `Cargo.toml`.
cp -av "${s4_file}" "${dest_path}"
chmod -v -w "${dest_path}"
(
    cd "${DIROUT}"
    rm -f "${bin}" "${bin}.sha256"
    create_sha256sum "${dest_name}"
    cp -av "${dest_name}" "${bin}"
    create_sha256sum "${bin}"
    zip -v9 "${zip_path}" "${bin}" "${bin}.sha256"
    chmod -v -w "${zip_path}"
    create_sha256sum "${zip_path}"
    rm -vf "${bin}" "${bin}.sha256"
)