#!/usr/bin/env bash
#
# compress the passed file with the various compression and archive tools.
#

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: ${0} <file> [...]" >&2
    exit 1
fi

echo '# BZ2'
echo
echo

(
    set -x
    bzip2 -zf9kvv -- "${@}"
)

echo

for file in "${@}"; do
    if [[ -f "${file}.bz2" ]]; then
        (
            set -x
            bzip2 -vvv --test -- "${file}.bz2"
        )
    fi
done

echo
echo
echo '# GZIP'
echo
echo

(
    set -x
    gzip -f9kvv -- "${@}"
)

echo

for file in "${@}"; do
    (
        set -x
        gzip -vvv --list -- "${file}.gz"
    )
done

echo
echo
echo '# LZ'
echo
echo

(
    set -x
    lzip -f9kvv -- "${@}"
)

echo

for file in "${@}"; do
    (
        set -x
        lzip -vv --list -- "${file}.lz"
    )
done

echo
echo
echo '# LZ4'
echo
echo

for file in "${@}"; do
    (
        set -x
        lz4c -BX -f9kvv -- "${file}"
    )
    echo
done


echo

for file in "${@}"; do
    (
        set -x
        lz4c -vv --list -- "${file}.lz4"
    )
    echo
done

echo
echo
echo '# LZOP'
echo
echo

for file in "${@}"; do
    (
        set -x
        lzop -f9kvv -- "${file}"
    )
    echo
done

echo

for file in "${@}"; do
    (
        set -x
        lzop -vv --list -- "${file}.lzo"
    )
    echo
done

echo
echo
echo '# XZ'
echo
echo

(
    set -x
    xz -zfkve -T0 -- "${@}"
)

echo

for file in "${@}"; do
    (
        set -x
        xz --list -- "${file}.xz"
        # lzmainfo -- "${file}.xz"
    )
done

echo
echo
echo '# ZSTD'
echo
echo

(
    set -x
    zstd -f19kvv -- "${@}"
)

echo

for file in "${@}"; do
    (
        set -x
        zstd -v -l -- "${file}.zst"
    )
done

echo
echo
echo '# ZIP'
echo
echo

zip_dir=$(dirname -- "${1}")
zip_out=''
for file in "${@}"; do
    zip_out="${zip_out}_$(basename -- "${file%.*}")"
done
zip_out="${zip_dir}/${zip_out#_}.zip"
(
    set -x
    zip -9orvy "${zip_out}" -- "${@}"
)

echo

(
    set -x
    zipinfo -vl -- "${zip_out}"
)
echo
(
    set -x
    zip -vT "${zip_out}"
)

echo
echo
echo '# TAR'
echo
echo

tar_dir=$(dirname -- "${1}")
tar_out=''
for file in "${@}"; do
    tar_out="${tar_out}_$(basename -- "${file%.*}")"
    echo "tar_out='${tar_out}'"
    touch_file=${file}
done
tar_out="${tar_out#_}"
echo "tar_out='${tar_out}'"
tar_out=${tar_dir}/${tar_out}.tar
echo "tar_out='${tar_out}'"
tar_gz_out=${tar_out}.gz
tar_lz_out=${tar_out}.lz
tar_lzo_out=${tar_out}.lzo
tar_xz_out=${tar_out}.xz
for tar in "${tar_out}" "${tar_gz_out}" "${tar_lz_out}" "${tar_lzo_out}" "${tar_xz_out}"; do
    (
        set -x
        tar --create \
            --auto-compress \
            -b1 \
            --preserve-permissions --xattrs \
            --numeric-owner \
            --verify \
            --totals \
            --record-size=512 \
            --acls \
            -vv \
            "--file=${tar}" \
            -- \
            "${@}"
        touch "${tar}" --reference="${touch_file}"
    )
    echo
done

echo

for tar in "${tar_out}" "${tar_gz_out}" "${tar_lz_out}" "${tar_lzo_out}" "${tar_xz_out}"; do
    (
        set -x
        tar -tvv --totals --full-time "--file=${tar}"
    )
    echo
done
