#!/usr/bin/env bash
#
# compare-grep-sort1.sh
#
# compare run time of `super_speedy_syslog_searcher` to Unix tools `grep` and `sort` (preferably GNU) 
# passed arguments are forwarded to `/usr/bin/time`
#

set -euo pipefail

cd "$(dirname "${0}")/.."

(set -x; uname -a)
(set -x; git log -n1 --format='%h %D')
(set -x; ./target/release/super_speedy_syslog_searcher --version)
# use full path to Unix tools
grep=$(which grep)
(set -x; $grep --version) | head -n1
sort=$(which sort)
(set -x; $sort --version) | head -n1
time=$(which time)
(set -x; $time --version) | head -n1

echo

tmp1=$(mktemp --suffix "__super_speedy_syslog_searcher")
tmp2=$(mktemp --suffix "__grep_sort")

trap 'rm -f "${tmp1}" "${tmp2}"' EXIT

declare -a files=(
    $(ls -1 ./logs/other/tests/gen-{100-10-......,100-10-BRAAAP,100-10-FOOBAR,100-10-______,100-10-skullcrossbones,100-4-happyface,1000-3-foobar,200-1-jajaja,400-4-shamrock}.log)
)

# force reading of files from disk to allow any possible caching,
# so a little less difference in the two timed processes
cat "${files[@]}" > /dev/null

(
export RUST_BACKTRACE=1
set -x
$time -p "${@}" -- \
    ./target/release/super_speedy_syslog_searcher \
    -z 0xFFFF \
    -a 20000101T000000 -b 20000101T080000 \
    --color never \
    "${files[@]}" \
    >/dev/null
)

echo

(
set -x
$time -p "${@}" -- \
    bash -c "
    $grep -hEe '^20000101T00[01234567][[:digit:]]{3}|^20000101T080000' -- \
    ${files[*]} \
    | $sort -t ' ' -k 1 -s \
    >/dev/null \
    "    
)
# run again, save output
./target/release/super_speedy_syslog_searcher \
    -z 0xFFFF \
    -a 20000101T000000 -b 20000101T080000 \
    --color never \
    "${files[@]}" \
    > "${tmp1}"

$grep -hEe '^20000101T00[01234567][[:digit:]]{3}|^20000101T080000' -- \
    "${files[@]}" \
    | $sort -t ' ' -k 1 -s \
    > "${tmp2}"

echo
s4_lc=$(wc -l < "${tmp1}")
s4_bc=$(wc -c < "${tmp1}")
echo "super_speedy_syslog_searcher output file"
echo "  Line Count ${s4_lc}"
echo "  Byte Count ${s4_bc}"
gs_lc=$(wc -l < "${tmp2}")
gs_bc=$(wc -c < "${tmp2}")
echo "'grep | sort' output file"
echo "  Line Count ${gs_lc}"
echo "  Byte Count ${gs_bc}"
# output will diff!
diff --brief "${tmp1}" "${tmp2}" || true

echo
echo "The output files will differ due to sorting method differences."
echo "However Line Count and Byte Count should be the same."

[[ ${s4_lc} -eq ${gs_lc} ]] && [[ ${s4_bc} -eq ${gs_bc} ]]
