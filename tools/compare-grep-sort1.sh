#!/usr/bin/env bash
#
# compare-grep-sort1.sh
#
# compare run time of `super_speedy_syslog_searcher` to Unix tools `grep` and
# `sort` (preferably GNU). Passed arguments are forwarded to `/usr/bin/time`
#

set -euo pipefail

cd "$(dirname "${0}")/.."

(set -x; uname -a)
if [[ -d '.git' ]]; then
    (set -x; git log -n1 --format='%h %D')
fi
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
tmp1b=$(mktemp --suffix "__super_speedy_syslog_searcher")
tmp2=$(mktemp --suffix "__grep_sort")
tmp2b=$(mktemp --suffix "__grep_sort")

function exit_() {
    rm -f "${tmp1}" "${tmp2}" "${tmp1b}" "${tmp2b}"
}

trap exit_ EXIT

declare -a files=(
    $(ls -1 ./logs/other/tests/gen-{100-10-......,100-10-BRAAAP,100-10-FOOBAR,100-10-______,100-10-skullcrossbones,100-4-happyface,1000-3-foobar,200-1-jajaja,400-4-shamrock,99999-1-Hüsker_Dü}.log)
    #$(ls -1 ./logs/other/tests/gen-99999-1-Hüsker_Dü.log.gz)
    #$(ls -1 ./logs/other/tests/dtf5-6a.log{,.gz})
)

# force reading of files from disk to allow any possible caching,
# so a little less difference in the two timed processes
cat "${files[@]}" > /dev/null

# search for datetimes between
declare -r after_dt='20000101T000000'
declare -r befor_dt='20000101T025959'
# regexp equivalent of $after_dt $befor_dt
declare -r regex_dt='^20000101T0[012][[:digit:]]{4}'
# declare s4 args once
declare -ar s4_args=(
    -a "${after_dt}" -b "${befor_dt}"
    --color never
)

# run both programs, time the runs

(
#export RUST_BACKTRACE=1
set -x
$time -p "${@}" -- \
    ./target/release/super_speedy_syslog_searcher \
    "${s4_args[@]}" \
    "${files[@]}" \
    >/dev/null
)

echo

# search for datetimes between $after_dt $befor_dt
# using decently constrained regexp to match meaning
(
set -x
$time -p "${@}" -- \
    bash -c "
    $grep -hEe '${regex_dt}' -- \
    ${files[*]} \
    | $sort -t ' ' -k 1 -s \
    >/dev/null \
    "    
)

# run both programs again, save output for comparison

./target/release/super_speedy_syslog_searcher \
    "${s4_args[@]}" \
    "${files[@]}" \
    > "${tmp1}"

$grep -hEe "${regex_dt}" -- \
    "${files[@]}" \
    | $sort -t ' ' -k 1 -s \
    > "${tmp2}"

# compare the program outputs

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

# literal output will differ!
diff --brief "${tmp1}" "${tmp2}" || true
# however...
echo
echo "The output files will differ due to sorting method differences."
echo "However Line Count and Byte Count should be the same."

declare -i ret=0
if [[ ${s4_lc} -ne ${gs_lc} ]] || [[ ${s4_bc} -ne ${gs_bc} ]]; then
    ret=1
    sort "${tmp1}" > "${tmp1b}"
    sort "${tmp2}" > "${tmp2b}"
    echo
    echo "Difference Preview:"
    (set -x; diff -y --width=$COLUMNS --suppress-common-lines "${tmp1b}" "${tmp2b}") | head -n 20
fi

exit ${ret}
