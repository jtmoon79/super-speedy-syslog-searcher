#!/usr/bin/env bash
#
# quickie script to generate a syslog file for testing

set -euo pipefail

# hardcoded data to print on each syslog line
declare -ar alphabet=(
  0 1 2 3 4 5 6 7 8 9
  a b c d e f g h i j k l m n o p q r s t u v w x y z
  A B C D E F G H I J K L M N O P Q R S T U V W Z Y Z
  À Á Â Ã Ä Å Æ Ç È É Ê Ë Ì Í Î Ï Ð Ñ Ò Ó Ô Õ Ö × Ø Ù Ú Û Ü Ý Þ ß
  à á â ã ä å æ ç è é ê ë ì í î ï ð ñ ò ó ô õ ö ÷ ø ù ú û ü ý þ ÿ
)
declare -ir alen=${#alphabet[@]}

# print this many syslog lines
declare -ir line_count=${1-100}
# repeat each syslog line this many times
declare -ir repeat_line=${2-1}
# optional string to append
declare -r append=${3-}
# https://www.epochconverter.com/
# Unix Epoch time at 2020/01/01 00:00:00 GMT
declare -ir epoch_2000_GMT=946684800
# Unix Epoch time at 2020/01/01 00:00:00 PST
declare -ir epoch_2000_PST=946713600
# first sysline datetime is this Unix Epoch time (i.e. a count in seconds)
declare -i dt_start=${4-${epoch_2000_PST}}

declare lc_str="${line_count}"  # line count as string
declare -i lc_i=${#lc_str}  # line count characters wide as integer

declare -i x=0
while [[ ${x} -lt ${line_count} ]]; do
    declare -i a=0
    declare dts=$(date --date "@${dt_start}" '+%Y%m%dT%H%M%S' | tr -d '\n')
    declare -i b=0
    # print a syslog line
    while [[ ${a} -lt ${repeat_line} ]]; do
        echo -n "${dts} $(printf "%0${lc_i}d " ${x})"
        declare -i c=$((x + b))
        declare -i c_stop=$((c + alen))
        # print a subset of the $alphabet on the line
        while [[ ${c} -lt ${c_stop} ]]; do
            echo -n "${alphabet[$((${c} % ${alen}))]}"
            c=$((c + 1))
        done
        b+=1
        a+=1
        # print the optional append string
        if [[ "${append}" = "" ]]; then
            echo
        elif [[ ${x} -eq 0 ]]; then
            echo " ${append}"
        else
            echo " ${append}"
        fi
    done
    dt_start+=1  # advance datetime by one second
    x+=1
done
