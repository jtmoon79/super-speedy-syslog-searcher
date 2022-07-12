#!/usr/bin/env bash
#
# quickie script to generate a syslog file for testing

set -euo pipefail

# hardcoded data to print on each syslog line
declare -ar alphabet=(
  #0 1 2 3 4 5 6 7 8 9
  #a b c d e f g h i j k l m n o p q r s t u v w x y z
  #A B C D E F G H I J K L M N O P Q R S T U V W Z Y Z
  #À Á Â Ã Ä Å Æ Ç È É Ê Ë Ì Í Î Ï Ð Ñ Ò Ó Ô Õ Ö × Ø Ù Ú Û Ü Ý Þ ß
  #à á â ã ä å æ ç è é ê ë ì í î ï ð ñ ò ó ô õ ö ÷ ø ù ú û ü ý þ ÿ
  😀 😁 😂 😃 😄 😅 😆 😇 😈 😉 😊 😋 😌 😍 😎 😏 😐 😑 😒 😓 😔 😕 😖 😗 😘 😙 😚 😛 😜 😝 😞 😟 😠 😡 😢 😣 😤 😥 😦 😧 😨 😩 😪 😫 😬 😭 😮 😯 😰 😱 😲 😳 😴 😵 😶 😷 😸 😹 😺 😻 😼 😽 😾 😿 🙀 🙁 🙂 🙃
)
declare -ir alen=${#alphabet[@]}

# print this many syslog lines
declare -ir line_count=$((${1-100} + 1))
# repeat each syslog line this many times
declare -ir repeat_line=${2-1}
# optional string to append
declare -r append=${3-}
if [[ -z "${append}" ]]; then
    declare -r append_empty=true
else
    declare -r append_empty=false
fi
# https://www.epochconverter.com/
# Unix Epoch time at 2020/01/01 00:00:00 GMT
declare -ir epoch_2000_GMT=946684800
# Unix Epoch time at 2020/01/01 00:00:00 PST
declare -ir epoch_2000_PST=946713600
# first sysline datetime is this Unix Epoch time (i.e. a count in seconds)
declare -i dt_start=${4-${epoch_2000_PST}}

declare lc_str=$((line_count - 1))  # line count as string
declare -ir lc_i=${#lc_str}  # line count characters wide as integer

# pre-create all possible alphabet sylines
declare -a alphas=()
declare -i a=0
while [[ ${a} -lt ${alen} ]]; do
    declare alpha=''
    declare -i c=a
    declare -i c_stop=$((a + alen))
    while [[ ${c} -lt ${c_stop} ]]; do
        alpha+="${alphabet[$((${c} % ${alen}))]}"
        c+=1
    done
    alphas[${a}]=${alpha}
    a+=1
done

declare -i loop=0
declare -i line_at=0
while [[ ${loop} -lt ${line_count} ]]; do
    declare -i a=0
    declare dts=$(date --date "@${dt_start}" '+%Y%m%dT%H%M%S' | tr -d '\n')
    while [[ ${a} -lt ${repeat_line} ]]; do
        # gather a subset of the alphabet
        declare -i alpha_at=$((${line_at} % ${alen}))
        # print one sysline
        if ${append_empty}; then
            echo "${dts} $(printf "%0${lc_i}d" ${line_at}) ${alphas[${alpha_at}]}"
        else
            echo "${dts} $(printf "%0${lc_i}d" ${line_at}) ${alphas[${alpha_at}]} ${append}"
        fi
        line_at+=1
        a+=1
    done
    dt_start+=1  # advance datetime by one second
    loop+=1
done
