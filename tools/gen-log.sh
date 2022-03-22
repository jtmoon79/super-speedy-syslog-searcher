#!/usr/bin/env bash
#
# quickie script to generate a testing file

set -eu

declare -ar alphabet=(
  0 1 2 3 4 5 6 7 8 9
  a b c d e f g h i j k l m n o p q r s t u v w x y z
  A B C D E F G H I J K L M N O P Q R S T U V W Z Y Z
  À Á Â Ã Ä Å Æ Ç È É Ê Ë Ì Í Î Ï Ð Ñ Ò Ó Ô Õ Ö × Ø Ù Ú Û Ü Ý Þ ß
  à á â ã ä å æ ç è é ê ë ì í î ï ð ñ ò ó ô õ ö ÷ ø ù ú û ü ý þ ÿ
)
declare -ir alen=${#alphabet[@]}

declare -ir line_count=${1-100}
# https://www.epochconverter.com/
# Unix time at 2020/01/01 00:00:00 GMT
declare -ir epoch_2000_GMT=946684800
# Unix time at 2020/01/01 00:00:00 PST
declare -ir epoch_2000_PST=946713600
declare -i dt_start=${epoch_2000_PST}

# repeat count of syslog line with this datetime
declare -ir repeat_dt=${2-2}

declare -r uniq_str=${3-}

declare -i x=0
while [[ ${x} -lt ${line_count} ]]; do
    declare -i a=0
    declare dts=$(date --date "@${dt_start}" '+%Y%m%dT%H%M%S' | tr -d '\n')
    declare -i b=0
    while [[ ${a} -lt ${repeat_dt} ]]; do
        # print a syslog line
        echo -n "${dts} "
        declare -i c=0
        while [[ ${c} -lt ${x} ]]; do
            declare -i at=$(((${c} + ${b}) % ${alen}))
            echo -n "${alphabet[${at}]}"
            c+=1
        done
        b+=c
        a+=1
        if [[ "${uniq_str}" = "" ]]; then
            echo
        elif [[ ${x} -eq 0 ]]; then
            echo "${uniq_str}"
        else
            echo " ${uniq_str}"
        fi
    done
    dt_start+=1
    x+=1
done
