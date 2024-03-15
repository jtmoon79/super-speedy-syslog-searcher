#!/usr/bin/env bash
#
# compare-grep-sort.sh
#
# compare run time of `super_speedy_syslog_searcher` to Unix tools `grep` and
# `sort` (preferably GNU). Passed arguments are forwarded to `/usr/bin/time`, except
# optional argument `--keep`.
#

set -euo pipefail

cd "$(dirname "${0}")/.."

(set -x; uname -a)
if [[ -d '.git' ]]; then
    (set -x; git log -n1 --format='%h %D')
fi
PROGRAM=${PROGRAM-./target/release/s4}
(set -x; "${PROGRAM}" --version)
# use full path to Unix tools
grep=$(which grep)
(set -x; $grep --version) | head -n1
sort=$(which sort)
(set -x; $sort --version) | head -n1
time=$(which time)
(set -x; $time --version) | head -n1

echo

do_keep=false
if [[ "${1-}" = "--keep" ]]; then
    do_keep=true
    shift
fi

tmp1=$(mktemp -t "compare-s4_s4_XXXXX")
tmp1b=$(mktemp -t "compare-s4-sorted_s4_XXXXX")
tmp2=$(mktemp -t "compare-s4_grep_XXXXX")
tmp2b=$(mktemp -t "compare-s4-sorted_grep_XXXXX")

function exit_() {
    if ! ${do_keep}; then
        rm -f "${tmp1}" "${tmp2}" "${tmp1b}" "${tmp2b}"
    fi
}

trap exit_ EXIT

declare -a files=(
    ./logs/other/tests/gen-100-10-.......log
    ./logs/other/tests/gen-100-10-BRAAAP.log
    ./logs/other/tests/gen-100-10-FOOBAR.log
    ./logs/other/tests/gen-100-10-______.log
    ./logs/other/tests/gen-100-10-skullcrossbones.log
    ./logs/other/tests/gen-100-4-happyface.log
    ./logs/other/tests/gen-1000-3-foobar.log
    ./logs/other/tests/gen-200-1-jajaja.log
    ./logs/other/tests/gen-400-4-shamrock.log
    ./logs/other/tests/gen-99999-1-Motley_Crue.log
)

# force reading of files from disk to allow any possible caching,
# so a little less difference in the two timed processes
cat "${files[@]}" > /dev/null

# search for datetimes between
declare -r after_dt='20000101T000000'
declare -r befor_dt='20000101T025959.999999'
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
    "${PROGRAM}" \
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

"${PROGRAM}" \
    "${s4_args[@]}" \
    "${files[@]}" \
    > "${tmp1}"

$grep -hEe "${regex_dt}" -- \
    "${files[@]}" \
    | $sort -t ' ' -k 1 -s \
    > "${tmp2}"

# compare the program outputs

echo
echo "The output files will differ due to sorting method differences."
echo "However Line Count and Byte Count should be the same."
echo
# s4 line count byte count
s4_lc=$(wc -l < "${tmp1}")
s4_bc=$(wc -c < "${tmp1}")
echo "super_speedy_syslog_searcher output file"
echo "  Line Count ${s4_lc}"
echo "  Byte Count ${s4_bc}"
# grep|sort line count byte count
gs_lc=$(wc -l < "${tmp2}")
gs_bc=$(wc -c < "${tmp2}")
echo "'grep | sort' output file"
echo "  Line Count ${gs_lc}"
echo "  Byte Count ${gs_bc}"

DIFF=diff
if which colordiff &>/dev/null; then
    DIFF=colordiff
fi

# literal output will differ!
diff --brief "${tmp1}" "${tmp2}" || true

declare -i ret=0
if [[ ${s4_lc} -ne ${gs_lc} ]] || [[ ${s4_bc} -ne ${gs_bc} ]]; then
    ret=1
    sort "${tmp1}" > "${tmp1b}"
    sort "${tmp2}" > "${tmp2b}"
    echo
    echo "Line Count and Byte Count are not the same. (ಠ_ಠ)"
    echo
    echo "Difference Preview:"
    (
        (
            set -x;
            "${DIFF}" -y --width=${COLUMNS-120} --suppress-common-lines "${tmp1b}" "${tmp2b}"
        ) || true
    ) | head -n 20
    echo
    if ! ${do_keep}; then
        echo "Pass --keep to keep the temporary files for further analysis"
    else
        echo "Files remain:"
        echo "  ${tmp1}"
        echo "  ${tmp2}"
        echo "  ${tmp1b}"
        echo "  ${tmp2b}"
    fi
else
    echo
    echo "Line Count and Byte Count are the same. (ʘ‿ʘ)"
    echo
fi

exit ${ret}
