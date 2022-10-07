#!/usr/bin/env bash
#
# Update the filesystem modification datetime for files under `./logs/`.
# Uses listing of files + datetime in `log-files-time-update.txt`.
# Check correctness of files listing after processing.
#
# Setting the filesystem datetime of repository syslog files is more realisitic
# for testing. Syslog processing can require using the file's
# filesystem modified datetime.
#
# A helpful command for reviewing datetimes in files
#     find -type f \
#         -not \( -name '*.gz' -or -name '*xz' -or -name '*.tar' -or -name 'wtmp' -or -name 'btmp' -or -name '*json' -or -name '*xml' -or -name '*.zip' -or -name 'faillog' -or -name 'lastlog' \) \
#         -printf '%p|%AF %AH:%AM:%AS\n' -exec tail -n2 {} \; -exec echo \;
#

set -euo pipefail

cd "$(dirname -- "${0}")"
times_listing=$(realpath "./log-files-time-update.txt")
cd "../logs"

declare -a files_listed=()
declare -a files_nodate=()
declare -a files_noexist=()
declare -A files_touchfailed=()

#
# for each file in listing, set the filesystem datetime attributes using `touch`
#

while read -r file_date; do
    # ignore empty lines
    if [[ -z "${file_date}" ]]; then
        continue
    fi
    file=$(echo -n "${file_date}" | cut -f 1 -d '|')
    # ignore lines starting with '#'
    if [[ "${file:0:1}" = '#' ]]; then
        continue
    fi
    # remember this file from the listing
    files_listed[${#files_listed[@]}]=${file}
    if [[ ! -e "${file}" ]]; then
        files_noexist[${#files_noexist[@]}]=${file}
        continue
    fi
    date=$(echo -n "${file_date}" | cut -f 2 -d '|')
    # empty date field!
    if [[ -z "${date}" ]]; then
        files_nodate[${#files_nodate[@]}]=${file}
        continue
    fi
    (
        set -x
        touch --no-create --date="${date}" -- "${file}"
    ) || {
        files_touchfailed[${file}]=${date}
        continue
    }
    # print --full-time so developer can visually verify
    ls --full-time -- "${file}"
done <<< $(cat "${times_listing}")

#
# let developer know about potential problems
#

# touch failed?
if [[ "${#files_touchfailed[@]}" -gt 0 ]]; then
    echo -e "Files touch failed from '${times_listing}'\n" >&2
fi
for file in "${!files_touchfailed[@]}"; do
    date=${files_touchfailed["${file}"]}
    echo -e "\e[1m\e[93mdate: '${date}', file: '${file}'\e[0m" >&2
    echo
done

# file in listing did not exist?
if [[ "${#files_noexist[@]}" -gt 0 ]]; then
    echo -e "Files do not exist from '${times_listing}'\n" >&2
fi
for file in "${files_noexist[@]}"; do
    echo -e "\e[1m\e[93m${file}\e[0m" >&2
    echo
done

# file in listing did not have a date?
if [[ "${#files_nodate[@]}" -gt 0 ]]; then
    echo -e "Files without a datetime in '${times_listing}'\n" >&2
fi
for file in "${files_nodate[@]}"; do
    echo -e "\e[1m\e[31m'${file}'\e[0m" >&2
    tail -n 10 -- "${file}"
    echo
done

# are actual files found on the filesystem also listed in the listing?
banner=false
while read -r file_actual; do
    found=false
    for file_listed in "${files_listed[@]}"; do
        if [[ "${file_actual}" = "${file_listed}" ]]; then
            found=true
            break
        fi
    done
    if ! ${found}; then
        if ! ${banner}; then
            echo -e "Files found on filesystem but not found in listing '${times_listing}'\n" >&2
            banner=true
        fi
        echo -e "\e[1m\e[93m${file_actual}\e[0m" >&2
    fi
done <<< $(find . -type f | sort)
