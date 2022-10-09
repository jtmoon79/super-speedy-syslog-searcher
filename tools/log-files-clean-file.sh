#!/usr/bin/env bash
#
# Cleanup a log file copied from another real system.
#
# Log files copied from real systems may have sensitive data.
# This script attempts to "clean" the file to prepare for committing to public
# git repository.
#
# Environment variable LOG_CLEAN_FILE is '|' separated list of string pairs.
# the string-to-remove is left-side, then '=', then string-to-replace is
# right-side.
# e.g. LOG_CLEAN_FILE='my-secret-host=host1|jerry=user2|.specific-internal-network-name=.local'
# Useful for replacing sensitive information likes hostnames, user names,
# IPv4 Addresses, etc.
#
# Environment variable LOG_CLEAN_FILE_LINES is the line truncation count.
# Defaults to 20
#

set -euo pipefail

function trunc ()
{
    # truncate lines
    declare -ir count=${LOG_CLEAN_FILE_LINES-50}
    sed -i -s -e "${count}"',$ d' "${@}"
}

function replace_MAC () {
    # found
    #    31:a2:cc:9d:ac:50
    # becomes
    #    01:02:03:04:05:50
    sed -i -s -E \
        -e 's/([[:xdigit:]]{2}:){5}([[:xdigit:]]{2})/01:02:03:04:05:\2/g' \
        "${@}"
}

function replace_IPv4 () {
    # found
    #    10.100.244.33
    # becomes
    #    192.168.0.33
    sed -i -s -E \
        -e 's/\b(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\b/192.168.0.\4/g' \
        "${@}"
}


function replace_GUID () {
    # found
    #   6bccceae-9980-4e41-9735-46bd1687a025
    # becomes
    #   01234567-8911-2345-6789-212345678931
    #
    # found
    #   c711101c91964b9e910ef61b154c9584
    # becomes
    #   01234567891123456789212345678931
    #
    # found
    #   444FDD668333FFFAAA31
    # becomes
    #   01234567891123456789
    #
    # found
    #   35696F43FC7DB4C2
    # becomes
    #   35696F43FC7DB4C2
    #
    sed -i -s -E \
        -e 's/[[:xdigit:]]{8}-[[:xdigit:]]{4}-[[:xdigit:]]{4}-([[:xdigit:]]{4})-[[:xdigit:]]{12}/00000000-1111-2222-\1-444444444444/g' \
        -e 's/\b[[:xdigit:]]{28}([[:xdigit:]]{4})\b/0123456789112345678900000000\1/g' \
        -e 's/\b[[:xdigit:]]{22}\b/0123456789112345678921/g' \
        -e 's/\b[[:xdigit:]]{20}\b/01234567891123456789/g' \
        -e 's/\b[[:xdigit:]]{18}\b/012345678911234567/g' \
        -e 's/\b[[:xdigit:]]{17}\b/01234567891123456/g' \
        -e 's/\b[[:xdigit:]]{16}\b/0123456789112345/g' \
        -e 's/\b[[:xdigit:]]{15}\b/012345678911234/g' \
        -e 's/\b[[:xdigit:]]{12}\b/012345678911/g' \
        -e 's/\b[[:xdigit:]]{10}\b/0123456789/g' \
        "${@}"
}

function replace_port () {
    # found
    #    port 43523
    # becomes
    #    port 1234
    #
    sed -i -s -E \
        -e 's/\bport ([[:digit:]])+\b/port '"${RANDOM:0:5}"'/g' \
        "${@}"
}

function replace_num () {
    # found
    #    port 64074
    # becomes
    #    port 49399
    #
    sed -i -s -E \
        -e 's/\b[[:digit:]]{5}\b/'"${RANDOM:0:5}"'/g' \
        -e 's/\b[[:digit:]]{6}\b/'"${RANDOM:0:6}"'/g' \
        -e 's/\b[[:digit:]]{7}\b/'"${RANDOM:0:7}"'/g' \
        -e 's/\b[[:digit:]]{9}\b/'"${RANDOM:0:9}"'/g' \
        "${@}"
}

function replace_SSH_hash () {
    # found
    #    SHA256:OAZpLxIo/bb44ttttPOWkA4BlCwML8889k9IVVA2yyU
    # becomes
    #    SHA256:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopq
    #
    sed -i -s -E \
        -e 's/\bSHA256:([[:alnum:]\/])+\b/SHA256:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopq/g' \
        "${@}"
}

PYTHON=$(which python 2>/dev/null || which python3 2>/dev/null)
function re_escape() {
    "${PYTHON}" -c 'import re; print(re.escape(""" '"${1-}"' """[1:-1]).replace("/", r"\/"))'
}

# count occurences of $2 within $1
function count_str() {
    echo -n "${1}" | grep -oFe "${2}" | wc -l
}

declare -ag LOG_CLEAN_FILE_SED_ARGS=()

# Process environment variable $LOG_CLEAN_FILE into array
# $LOG_CLEAN_FILE_SED_ARGS.
#
# Each variable is a string, field separator '|' character,
# with '=' signifying the replacement.
# e.g. LOG_CLEAN_FILE='my-www-server=host1|mary=user1|jerry=user2'
# Useful for passing particular host names that should be replaced in log files
# from real systems.
#
# bash array $LOG_CLEAN_FILE_SED_ARGS is passed directly to `sed`.
#
# Only call this once.
function setup_sed_args() {
    if [[ -z "${LOG_CLEAN_FILE-}" ]]; then
        return
    fi
    if [[ "${#LOG_CLEAN_FILE_SED_ARGS[@]}" -ne 0 ]]; then
        return
    fi
    declare -i i=0
    declare -ir total=$(count_str "${LOG_CLEAN_FILE}" '|')
    while [[ ${i} -le ${total} ]]; do
        declare pair=$(echo -n "${LOG_CLEAN_FILE}" | cut -f $((${i}+1)) -d '|')
        if [[ -z "${pair}" ]]; then
            i+=1
            continue
        fi
        if [[ $(count_str "${pair}" '=') -ne 1 ]]; then
            echo "ERROR Bad pair '${pair}', no '='" >&2
            exit 1
        fi
        declare a=$(echo -n "${pair}" | cut -f 1 -d '=')
        declare b=$(echo -n "${pair}" | cut -f 2 -d '=')
        a=$(re_escape "${a}")
        b=$(re_escape "${b}")
        if [[ -z "${a}" ]]; then
            echo "ERROR Bad pair: a='${a}'" >&2
            exit 1
        fi
        LOG_CLEAN_FILE_SED_ARGS[${#LOG_CLEAN_FILE_SED_ARGS[@]}]='-e'
        LOG_CLEAN_FILE_SED_ARGS[${#LOG_CLEAN_FILE_SED_ARGS[@]}]='s/'"${a}"'/'"${b}"'/g'
        i+=1
    done
}

setup_sed_args

function replace_user_passed () {
    if [[ "${#LOG_CLEAN_FILE_SED_ARGS[@]}" -eq 0 ]]; then
        return 0
    fi
    sed -i -s -E \
        "${LOG_CLEAN_FILE_SED_ARGS[@]}" \
        "${@}"
}

function replace() {
    # could be more efficient by combining these into one call to `sed`
    # but *meh*, this script is run rarely, so good enough
    replace_MAC "${@}"
    replace_IPv4 "${@}"
    replace_GUID "${@}"
    replace_port "${@}"
    replace_num "${@}"
    replace_SSH_hash "${@}"
    replace_user_passed "${@}"
}

if [[ "${#}" -lt 1 ]]; then
    echo "Must pass a file to edit" >&2
    exit 1
fi

# self-test
if [[ "${1-}" == '--test' ]]; then
    tmpf1=$(mktemp)
    tmpf2=$(mktemp)
    [[ -w ${tmpf1} ]] && [[ -w ${tmpf2} ]]
    trap "rm ${tmpf1} ${tmpf2}" EXIT
    LOG_CLEAN_FILE_LINES=99
    echo '=== FOLLOWING LINES SHOULD CHANGE ===
Device MAC: e1:00:ee:ea:77:c5
config is CC:33:DD:AA:88:E8 192.168.1.5 -97 V3.1.2 1 CC:BB:DD:BB:55:6F wired 1
10.100.45.5:443
session id: 444FDD668333FFFAAA31
MAC AF:6A:12:3E:B6:EE
MAC aa:bb:cc:dd:ee:ff
MAC1 aa:bb:cc:dd:ee:f1 MAC2 AA:BB:CC:DD:EE:F2
IPv4 10.44.55.66 10.44.55.67
GUID 02683b97-9286-4dbf-a429-60ee7b3df1c3 007bb996-9455-4552-b0a9-1a945cb48926
GUID {c6c84804-dde2-4f33-887e-463319dc766a}
GUID {CE70C15A-C84E-45B3-9A4A-521B0ACD0FC2}
GUID aeb3de2901ee401d8094d23c9247f2bb
GUID d919b3c21efa4f5eb65c0b191f4005ed
port 1
port 12
port 123
port 1234
port 12345
port 123456
port 1234567
port 12345678
port 123456789
apport 123456789
ssh2: RSA SHA256:OAZLLXXX/bb44ttttPOWPABBlCCML8889kIIVVAAyyU
=== FOLLOWING LINES SHOULD *NOT* CHANGE ===
apport 12
apport 123
apport 1234
Date 20220101
DateTime 20220101T003050
DateTime 20220101003050
Number 1
Number 12
Number 123
Number 12345678
' \
        > "${tmpf1}"
    cat "${tmpf1}" > "${tmpf2}"

    trunc "${tmpf1}"
    replace "${tmpf1}"
    diff --side-by-side --color=always --width ${COLUMNS} "${tmpf2}" "${tmpf1}"
    exit
fi

for file in "${@}"; do
    trunc "${file}"
    replace "${file}"
    echo -e "\e[1m\e[93m${file}\e[0m" >&2
    cat "${file}"
    echo >&2
done
