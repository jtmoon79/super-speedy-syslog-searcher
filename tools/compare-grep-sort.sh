#!/usr/bin/env bash
#
# compare-grep-sort.sh
#
# compare run time of `super_speedy_syslog_searcher` to Unix tools `grep` and
# `sort` (preferably GNU). Passed arguments are forwarded to `/usr/bin/time`, except
# optional argument `--keep`.
#
# optional export `FILES` is string that is a list of files to search,
# separated by spaces.
#
# This script also compares the standard output of each program.
# If stdout is the same then exit 0 else 1.
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
grep_sort_name="$(basename "$grep")+$(basename "$sort")"
(set -x; $time --version) | head -n1
if which hyperfine &>/dev/null; then
    hyperfine=$(which hyperfine)
    (set -x; hyperfine --version)
fi

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
md1=$(mktemp -t "compare-s4_s4_XXXXX.md")
md2=$(mktemp -t "compare-s4_s4_XXXXX.md")
mdfinal=${DIROUT-.}/compare-s4_grep_sort.md

function exit_() {
    if ! ${do_keep}; then
        rm -f "${tmp1}" "${tmp2}" "${tmp1b}" "${tmp2b}" "${md1}" "${md2}"
    fi
}

trap exit_ EXIT

if [[ -z "${FILES-}" ]]; then
    # default files to compare
    declare -a FILES=(
        ./logs/other/tests/gen-100-1-no.log
        ./logs/other/tests/gen-100-10-.......log
        ./logs/other/tests/gen-100-10-BRAAAP.log
        ./logs/other/tests/gen-100-10-FOOBAR.log
        ./logs/other/tests/gen-100-10-______.log
        ./logs/other/tests/gen-100-10-skullcrossbones.log
        ./logs/other/tests/gen-100-4-happyface.log
        ./logs/other/tests/gen-1000-3-foobar.log
        # XXX: it would be great to also compare a compressed file
        #      using `zgrep`. However, `zgrep` has different results
        #      than `grep`. (did I find a bug in zgrep?)
        #./logs/other/tests/gen-1000-3-foobar.log.gz
        ./logs/other/tests/gen-200-1-jajaja.log
        ./logs/other/tests/gen-400-4-shamrock.log
        ./logs/other/tests/gen-99999-1-Motley_Crue.log
    )
else
    # user can export `FILES` as a string of filenames separated by spaces
    # then each substring is copied to an array element
    declare -a FILES_=()
    for ff in ${FILES}; do
        # trim whitespace
        ff=$(echo "${ff}" | tr -d '\n')
        ff=${ff## }
        ff=${ff%% }
        if [[ -z "${ff}" ]]; then
            continue
        fi
        FILES_[${#FILES_[@]}]=${ff}
    done
    unset FILES
    declare -a FILES=("${FILES_[@]}")
fi

for ff in "${FILES[@]}"; do
    if [[ ! -f "${ff}" ]]; then
        echo "ERROR: File not found '${ff}'" >&2
        exit 1
    fi
done

# search for datetimes between ...
declare -r AFTER_DT=${AFTER_DT-'20000101T080000'}
declare -r BEFORE_DT=${BEFORE_DT-'20000101T085959.999999'}
# regexp equivalent of $AFTER_DT $BEFORE_DT
declare -r regex_dt='^20000101T08[[:digit:]]{4}'
# declare s4 args once
declare -ar s4_args=(
    -a "${AFTER_DT}" -b "${BEFORE_DT}"
    "--color=never"
)
# declare grep args once
declare -ar grep_args=(
    "--color=never"
    --text
)

# run both programs using hyperfine

if [[ -n "${hyperfine-}" ]]; then
    (
        # force reading of FILES from disk to allow any possible caching,
        cat "${FILES[@]}" &> /dev/null
        set -x
        $hyperfine --style=basic --export-markdown ${md1} -N -n "s4" \
            -- \
            "${PROGRAM} ${s4_args[*]} ${FILES[*]}"
    )

    echo

    # search for datetimes between $AFTER_DT $BEFORE_DT
    # using decently constrained regexp to match meaning
    (
        # force reading of FILES from disk to allow any possible caching,
        cat "${FILES[@]}" &> /dev/null
        set -x
        $hyperfine --style=basic --export-markdown ${md2} --shell sh -n "${grep_sort_name}" \
            -- \
            "$grep -hEe '${regex_dt}' ${FILES[*]} | $sort -t ' ' -k 1 -s"
    )

    (
        cat "${md1}"
        cat "${md2}" | tail -n +3
    ) | column -t -s '|' -o '|' > "${mdfinal}"

    (set -x; cat "${mdfinal}")

    if which glow &>/dev/null; then
        glow "${mdfinal}"
    fi
fi

# run both programs using time

TIME_FORMAT='real %e s, Max RSS %M KB, %P %%CPU, (%x)'

(
    set -x
    $time --format="${TIME_FORMAT}" \
        "${@}" \
        -- \
        "${PROGRAM}" \
        "${s4_args[@]}" \
        "${FILES[@]}" \
        >/dev/null
)

echo

# search for datetimes between $AFTER_DT $BEFORE_DT
# using decently constrained regexp to match meaning
(
    set -x
    $time --format="${TIME_FORMAT}" \
        "${@}" \
        -- \
        sh -c "\
$grep ${grep_args[*]} -hEe '${regex_dt}' -- \
${FILES[*]} \
| $sort -t ' ' -k 1 -s \
>/dev/null"
)

# run both programs again, save output for comparison
# this determines the exit code of the script

"${PROGRAM}" \
    "${s4_args[@]}" \
    "${FILES[@]}" \
    > "${tmp1}"

$grep ${grep_args[*]} -hEe "${regex_dt}" -- \
    "${FILES[@]}" \
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
echo "'${grep_sort_name}' output file"
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
            "${DIFF}" --text -y --width=${COLUMNS-120} --suppress-common-lines "${tmp1b}" "${tmp2b}"
        ) || true
    ) | head -n 40
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
