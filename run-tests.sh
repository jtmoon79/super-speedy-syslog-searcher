#!/usr/bin/env bash
#
# hardcoded tests with file display

set -o pipefail
set -eu

cd "$(dirname "${0}")"

(
set -x
cargo build && cargo build --release
)

prog="./target/release/super_speedy_syslog_searcher"
if ! [[ -x "${prog}" ]]; then
    echo "ERROR: cannot find or exec '$prog'" >&2
    exit 1
fi

function hexdump () {
  # https://github.com/kiedtl/hxd
  # hxd -cu -l 32

  # cargo install -f xd
  xd --color=always --table reverse
}

function filesz () {
  stat -tc '%s' "${1}"
}


# file_=/mnt/c/Users/user1/Projects/syslog-datetime-searcher/logs/debian9/syslog
rootd="."

declare -a files=(
    "${rootd}/logs/other/tests/zero.log"
    "${rootd}/logs/other/tests/test0-nlx1.log"
    "${rootd}/logs/other/tests/test0-nlx1_Win.log"
    "${rootd}/logs/other/tests/test0-nlx2.log"
    "${rootd}/logs/other/tests/test0-nlx2_Win.log"
    "${rootd}/logs/other/tests/test0-nlx3.log"
    "${rootd}/logs/other/tests/test0-nlx3_Win.log"
    "${rootd}/logs/other/tests/test0-no-nl.log"
    "${rootd}/logs/other/tests/test1-nl.log"
    "${rootd}/logs/other/tests/test1-nl_Win.log"
    "${rootd}/logs/other/tests/test1-no-nl.log"
    "${rootd}/logs/other/tests/test2.log"
    "${rootd}/logs/other/tests/test3-hex.log"
    "${rootd}/logs/other/tests/basic-dt.log"
    "${rootd}/logs/other/tests/basic-basic-dt20.log"
    "${rootd}/logs/Ubuntu18/samba/log.10.7.220.19"  # multi-line syslines
    "${rootd}/logs/debian9/syslog"  # very large file!
)

for file_ in "${files[@]}"; do
    for sz in 1 2 3 4 5 6 8 10 12 14 16 18 19 20 21 22 32 64 128 1024 2056 4096 8192 16284 32568 65536 131702
    do
        if [[ ! -r "${file_}" ]]; then
            echo -e "\e[33mWarning: skip file not readable or not exist '${file_}'\e[39m" >&2
            continue
        fi
        declare -i fsz=
        fsz=$(filesz "${file_}")
        if [[ ${fsz} -gt 100000 ]] && [[ ${sz} -lt 64 ]]; then
            continue
        fi
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "File: '${file_}'"
        echo "----------------------------------------------------------------------------------------------------"
        if [[ ${fsz} -lt 9999 ]]; then 
            cat "${file_}"
            echo "----------------------------------------------------------------------------------------------------"
            (
                set -x
                cat "${file_}"
            ) | hexdump
            echo "----------------------------------------------------------------------------------------------------"
            (
                set +e
                set -x
                "${prog}" --filepath "${file_}" "${sz}"
            ) | hexdump 
            echo "----------------------------------------------------------------------------------------------------"
            (
                set +e
                set -x
                "${prog}" --filepath "${file_}" "${sz}"
            )
            echo
            echo "----------------------------------------------------------------------------------------------------"
        fi
        echo
        echo "${prog} --filepath '${file_}' ${sz}"
        time md5_prog=$(
            set +e
            "${prog}" --filepath "${file_}" "${sz}" | md5sum
        ) 2>&1
        md5_prog=$(echo -n "${md5_prog}" | cut -f1 -d' ')
        echo
        echo "cat '${file_}' | md5sum"
        time md5_cat=$(cat "${file_}" | md5sum) 2>&1
        md5_cat=$(echo -n "${md5_cat}" | cut -f1 -d' ')
        echo
        if [[ "${md5_prog}" = "${md5_cat}" ]]; then
            echo -e "\e[32m${md5_prog} = ${md5_cat}\e[39m"
        else
            echo -e "\e[31m${md5_prog} ≠ ${md5_cat}\e[39m"
            exit 1
        fi
    done
done
