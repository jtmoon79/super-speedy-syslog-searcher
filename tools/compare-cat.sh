#!/usr/bin/env bash
#
# compare `s4` and `cat` on varying files
#
# does not clean up temporary files
#

set -euo pipefail

cd "$(dirname "${0}")/.."

PROGRAM=${PROGRAM-./target/release/s4}

if ! [[ -x "${PROGRAM}" ]]; then
    echo "ERROR: cannot find '$PROGRAM'" >&2
    exit 1
fi

do_keep=false

# print the file size
function filesz () {
    stat -tc '%s' "${1}"
}

DIFF=diff
if which colordiff &>/dev/null; then
    DIFF=colordiff
fi

declare -a files=(
    ./logs/Debian9/user.log.1.gz
    ./logs/MacOS11/install.log.0.gz
    ./logs/MacOS11/system.log.0.gz
    ./logs/other/tests/dtf9c-23-12x2.log
    ./logs/other/tests/dtf2-2.log
    ./logs/other/tests/dtf2-2.log.bz2
    ./logs/other/tests/dtf2-2.log.gz
    ./logs/other/tests/dtf2-2.log.lz4
    ./logs/other/tests/dtf2-2.log.tar
    ./logs/other/tests/dtf2-2.log.xz
    ./logs/other/tests/dtf7-20-LEVELS.log
    ./logs/other/tests/dtf7-20-LEVELS.log.bz2
    ./logs/other/tests/dtf7-20-LEVELS.log.gz
    ./logs/other/tests/dtf7-20-LEVELS.log.lz4
    ./logs/other/tests/dtf7-20-LEVELS.log.tar
    ./logs/other/tests/dtf7-20-LEVELS.log.xz
    ./logs/other/tests/gen-1000-3-foobar.log
    ./logs/other/tests/gen-1000-3-foobar.log.bz2
    ./logs/other/tests/gen-1000-3-foobar.log.gz
    ./logs/other/tests/gen-1000-3-foobar.log.lz4
    ./logs/other/tests/gen-1000-3-foobar.log.tar
    ./logs/other/tests/gen-1000-3-foobar.log.xz
    ./logs/other/tests/simple-12.bz2
    ./logs/other/tests/simple-12.lz4
    ./logs/other/tests/simple-12.gz
    ./logs/other/tests/simple-12.xz
    ./logs/other/tests/simple-12.log
    ./logs/other/tests/simple-12.log.bz2
    ./logs/other/tests/simple-12.log.gz
    ./logs/other/tests/simple-12.log.lz4
    ./logs/other/tests/simple-12.log.xz
    ./logs/other/tests/simple-12.tar
    ./logs/programs/journal/CentOS_7_system.journal
    ./logs/programs/journal/RHE_91_system.journal
    ./logs/programs/journal/RHE_91_system.journal.bz2
    ./logs/programs/journal/RHE_91_system.journal.gz
    ./logs/programs/journal/RHE_91_system.journal.lz4
    ./logs/programs/journal/RHE_91_system.journal.xz
    ./logs/programs/journal/Ubuntu22-user-1000.journal
    ./logs/programs/journal/Ubuntu22-user-1000x3.journal
    ./logs/programs/journal/Ubuntu22-user-1000x3.journal.bz2
    ./logs/programs/journal/Ubuntu22-user-1000x3.journal.gz
    ./logs/programs/journal/Ubuntu22-user-1000x3.journal.lz4
    ./logs/programs/journal/Ubuntu22-user-1000x3.journal.xz
    ./logs/RedHatEnterprise9/sssd/sssd_kcm.log-20230507.gz
    ./logs/Ubuntu16/kern.log.2.gz
    # TODO: add .evtx files
)

# pick blocksz larger than smallest file and smaller than largest file
BLOCKSZ=1024

declare -a files_failed=( )
declare -a files_passed=( )

function file_ends_with_nl() {
    tail -c 1 "${1}" | grep -q -Ee '^$'
}

function delete_last_char_of_file() {
    (
        set -x
        truncate -s-1 "${1}"
    )
}

function decompress_file() {
    declare -r file_=${1}
    declare -r tmp_=${2}
    if [[ "${file_:$((${#file_}-4))}" = '.bz2' ]]; then
        (set -x; bzip2 -cdk "${file_}" > "${tmp_}")
    elif [[ "${file_:$((${#file_}-3))}" = '.gz' ]]; then
        (set -x; gzip -cdk "${file_}" > "${tmp_}")
    elif [[ "${file_:$((${#file_}-4))}" = '.lz4' ]]; then
        (set -x; lz4 -cdk "${file_}" > "${tmp_}")
    elif [[ "${file_:$((${#file_}-3))}" = '.xz' ]]; then
        (set -x; xz -cdk "${file_}" > "${tmp_}")
    elif [[ "${file_:$((${#file_}-4))}" = '.tar' ]]; then
        (set -x; tar -xOf "${file_}" > "${tmp_}")
    else
        (set -x; cp -av "${file_}" "${tmp_}")
    fi
}

for file in "${files[@]}"; do
    tmp1=$(mktemp -t "compare-s4_XXXXX")
    tmp2=$(mktemp -t "compare-cat_XXXXX")
    journal_arg=

    echo "File: '${file}'"
    if ! [[ -r "${file}" ]]; then
        echo "file not found '${file}'" >&2
        exit 1
    fi

    if echo -n "${file}" | grep -qEe '\.journal'; then
        # call journalctl with `cat` output which does not print
        # datetimestamps. There may be differences in printed datetimestamps
        # for `journalctl` and `s4` output.
        # See Issue #101.
        (
            export SYSTEMD_COLORS=false
            export SYSTEMD_PAGER=
            export PAGER=
            # if $file is compressed then decompress it for use with journalctl
            tmp3=$(mktemp -t "compare-journal_XXXXX.journal")
            decompress_file "${file}" "${tmp3}"
            set -x
            journalctl --output=cat --no-tail --file="${tmp3}" > "${tmp2}"
        )
        journal_arg="--journal-output=cat"
    else
        decompress_file "${file}" "${tmp2}"
    fi

    (
        set -x
        "${PROGRAM}" --blocksz=${BLOCKSZ} ${journal_arg} --color=never "${file}" > "${tmp1}"
    )
    # delete last newline char added by `s4` only if the same file read by `cat` has no ending newline
    # see `fn processing_loop` in `src/bin/s4.rs`
    if ! file_ends_with_nl "${tmp2}"; then
        (
            set -x
            delete_last_char_of_file "${tmp1}"
        )
    fi
    match=true
    # compare line count
    lc1=$(cat "${tmp1}" | wc -l)
    lc2=$(cat "${tmp2}" | wc -l)
    (
        set -x
        wc -l "${tmp1}" "${tmp2}"
    )
    if [[ "${lc1}" != "${lc2}" ]]; then
        echo "ERROR: line count comparison failed for '${file}'" >&2
        if $match; then
            files_failed[${#files_failed[@]}]=${file}
        fi
        match=false
    fi
    # compare byte count
    bc1=$(cat "${tmp1}" | wc -c)
    bc2=$(cat "${tmp2}" | wc -c)
    (
        set -x
        wc -c "${tmp1}" "${tmp2}"
    )
    if [[ "${bc1}" != "${bc2}" ]]; then
        echo "ERROR: byte count comparison failed for '${file}'" >&2
        if $match; then
            files_failed[${#files_failed[@]}]=${file}
        fi
        match=false
    fi
    # compare checksum
    sum1=$(cat "${tmp1}" | sha256sum | cut -f1 -d ' ')
    sum2=$(cat "${tmp2}" | sha256sum | cut -f1 -d ' ')
    (
        set -x
        sha256sum "${tmp1}" "${tmp2}"
    )
    if [[ "${sum1}" != "${sum2}" ]]; then
        echo "ERROR: checksum comparison failed for '${file}'" >&2
        if [[ "${lc1}" = "${lc2}" ]] && $match; then
            files_failed[${#files_failed[@]}]=${file}
        fi
        match=false
    fi
    if ! ${match}; then
        echo "Difference:"
        (
                set -x;
                "${DIFF}" --text -y --width=${COLUMNS-120} --suppress-common-lines "${tmp1}" "${tmp2}"
        ) || true
    fi
    echo
    echo
    if ${match}; then
        files_passed[${#files_passed[@]}]=${file}
    fi
done

echo "Files passed ${#files_passed[@]}"
(
    for file in "${files_passed[@]}"; do
        echo "${file}"
    done 
) | sort | uniq -u

declare -i count_failed=${#files_failed[@]}
if [[ 0 -eq ${count_failed} ]]; then
    echo -e "\nAll files passed"
    exit 0
fi

echo
echo "Files failed ${count_failed}"
(
    for file in "${files_failed[@]}"; do
        echo "${file}"
    done
) | sort | uniq -u

exit 1
