#!/usr/bin/env bash
#
# given file that is UTF-8/ASCII, create various encodings of the file using `iconv`

set -eu

FILE="${1:-}"

if [[ -z "${FILE}" ]]; then
    echo "Usage: ${0} <file> [iconv options]" >&2
    exit 1
fi
shift
extra_args=("$@")

readonly FILE
readonly FILE_DIR="$(dirname -- "${FILE}")"
readonly FILE_NAME="$(basename -- "${FILE}")"
readonly FILE_EXT="${FILE_NAME##*.}"
readonly FILE_STEM="${FILE_NAME%.*}"

declare -a files_okay=()
declare -a files_failed=()

# run iconv, print result
function iconv() {
    local enc="$1"
    local output_file="$2"
    local file="$3"
    shift
    shift
    shift
    if ! (
        set -x
        command iconv --from-code="UTF-8" --to-code="${enc}" --output="${output_file}" "${file}" "${@}"
    ); then
        files_failed+=("${output_file}")
    else
        files_okay+=("${output_file}")
    fi
}

# run iconv, print result, and add BOM to the output file
function iconv_BOM () {
    local BOM="$1"
    local enc="$2"
    local output_file="$3"
    shift
    iconv "$@"
    cp -a "${output_file}" "${output_file}.tmp"
    echo -n -e "${BOM}" > "${output_file}"
    # XXX: cannot use `cat` because it re-encodes the file
    dd if="${output_file}.tmp" of="${output_file}" bs=1 conv=notrunc oflag=append status=none
    rm -f "${output_file}.tmp"
}

# UTF-8
enc="UTF-8"
output_file="${FILE_DIR}/${FILE_STEM}.${enc}.${FILE_EXT}"
iconv "${enc}" "${output_file}" "${FILE}"

# UTF-8 with BOM
output_file="${FILE_DIR}/${FILE_STEM}.${enc}_BOM.${FILE_EXT}"
iconv_BOM '\xEF\xBB\xBF' "${enc}" "${output_file}" "${FILE}"

# UTF-16LE
enc="UTF-16LE"
output_file="${FILE_DIR}/${FILE_STEM}.${enc}.${FILE_EXT}"
iconv "${enc}" "${output_file}" "${FILE}"

# UTF-16LE with BOM
output_file="${FILE_DIR}/${FILE_STEM}.${enc}_BOM.${FILE_EXT}"
iconv_BOM '\xFF\xFE' "${enc}" "${output_file}" "${FILE}"

# UTF-16BE
enc="UTF-16BE"
output_file="${FILE_DIR}/${FILE_STEM}.${enc}.${FILE_EXT}"
iconv "${enc}" "${output_file}" "${FILE}"

# UTF-16BE with BOM
output_file="${FILE_DIR}/${FILE_STEM}.${enc}_BOM.${FILE_EXT}"
iconv_BOM '\xFE\xFF' "${enc}" "${output_file}" "${FILE}"

# UTF-32LE
enc="UTF-32LE"
output_file="${FILE_DIR}/${FILE_STEM}.${enc}.${FILE_EXT}"
iconv "${enc}" "${output_file}" "${FILE}"

# UTF-32LE with BOM
output_file="${FILE_DIR}/${FILE_STEM}.${enc}_BOM.${FILE_EXT}"
iconv_BOM '\xFF\xFE\x00\x00' "${enc}" "${output_file}" "${FILE}"

# UTF-32BE
enc="UTF-32BE"
output_file="${FILE_DIR}/${FILE_STEM}.${enc}.${FILE_EXT}"
iconv "${enc}" "${output_file}" "${FILE}"

# UTF-32BE with BOM
output_file="${FILE_DIR}/${FILE_STEM}.${enc}_BOM.${FILE_EXT}"
iconv_BOM '\x00\x00\xFE\xFF' "${enc}" "${output_file}" "${FILE}"

# UTF-1
# XXX: not supported
# enc="UTF-1"
# output_file="${FILE_DIR}/${FILE_STEM}.${enc}.${FILE_EXT}"
# iconv_BOM '\xF7\x64\x4C' "${enc}" "${output_file}" "${FILE}"

# UTF-EBCDIC
# XXX: not supported
#enc="UTF-EBCDIC"
#output_file="${FILE_DIR}/${FILE_STEM}.${enc}.${FILE_EXT}"
#iconv_BOM '\xDD\x73\x66\x73' "${enc}" "${output_file}" "${FILE}"

# SCSU
# XXX: not supported
#enc="SCSU"
#output_file="${FILE_DIR}/${FILE_STEM}.${enc}.${FILE_EXT}"
#iconv_BOM '\x0E\xFE\xFF' "${enc}" "${output_file}" "${FILE}"

# BOCU-1
# XXX: not supported
#enc="BOCU-1"
#output_file="${FILE_DIR}/${FILE_STEM}.${enc}.${FILE_EXT}"
#iconv_BOM '\xFB\xEE\x28' "${enc}" "${output_file}" "${FILE}"

# GB18030
enc="GB18030"
output_file="${FILE_DIR}/${FILE_STEM}.${enc}_BOM.${FILE_EXT}"
iconv_BOM '\x84\x31\x95\x33' "${enc}" "${output_file}" "${FILE}"

echo
# green
for f in "${files_okay[@]}"; do
    echo -e "\033[32m${f}\033[0m"
done
# red
for f in "${files_failed[@]}"; do
    echo -e "\033[31m${f}\033[0m"
done
