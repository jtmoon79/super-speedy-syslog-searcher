#!/usr/bin/env bash
#
# Easily download a released s4 binary

set -euo pipefail

readonly SCRIPT_NAME=$(basename "$0")
readonly DIROUT=${DIROUT:-.}

usage() {
    cat >&2 <<EOF
Usage: ${SCRIPT_NAME} VERSION PLATFORM

Downloads and verifies a released s4 binary zip, extracts it, and copies it to DIROUT,
with naming s4_PLATFORM_vVERSION.

Arguments:
  VERSION   Release version (example: 0.9.81)
  PLATFORM  Target triple (example: x86_64-unknown-linux-gnu)

Environment:
  DIROUT    Output directory for renamed binary (default: .)
EOF
}

die() {
    echo "ERROR: $*" >&2
    exit 1
}

need_cmd() {
    local cmd=$1
    command -v "${cmd}" >/dev/null 2>&1 || die "required command not found: ${cmd}"
}

if [[ $# -ne 2 ]]; then
    usage
    exit 1
fi

readonly VERSION=$1
readonly PLATFORM=$2
readonly ZIP_NAME="s4_${PLATFORM}_v${VERSION}.zip"
readonly SHA_NAME="${ZIP_NAME}.sha256"
readonly URL_BASE="https://github.com/jtmoon79/super-speedy-syslog-searcher/releases/download/${VERSION}"
readonly ZIP_URL="${URL_BASE}/${ZIP_NAME}"
readonly SHA_URL="${URL_BASE}/${SHA_NAME}"
readonly OUT_NAME="s4_${PLATFORM}_v${VERSION}"

need_cmd curl
need_cmd unzip
need_cmd sha256sum
need_cmd mktemp
need_cmd rm
need_cmd mv
need_cmd chmod

mkdir -p -- "${DIROUT}"
[[ -d "${DIROUT}" ]] || die "DIROUT is not a directory: ${DIROUT}"

workdir=$(mktemp -d "${SCRIPT_NAME}.XXXXXX")
trap 'rm -rf -- "${workdir}"' EXIT

zip_path="${workdir}/${ZIP_NAME}"
sha_path="${workdir}/${SHA_NAME}"

(set -x; curl --fail --location --retry 2 --output "${zip_path}" "${ZIP_URL}")

(set -x; curl --fail --location --retry 2 --output "${sha_path}" "${SHA_URL}")

(
    cd -- "${workdir}"
    set -x
    sha256sum -c "${SHA_NAME}" 1>&2
)

unzip -o -d "${workdir}" "${zip_path}" >/dev/null

binary_path="${workdir}/s4"
[[ -f "${binary_path}" ]] || die "archive did not contain expected binary: s4"

final_path="${DIROUT}/${OUT_NAME}"
mv -f -- "${binary_path}" "${final_path}"
(set -x; chmod -w -- "${final_path}" && chmod +x -- "${final_path}")

rm -f -- "${zip_path}" "${sha_path}"

echo "${final_path}"
