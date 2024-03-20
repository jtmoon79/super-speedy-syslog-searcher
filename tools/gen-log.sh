#!/usr/bin/env bash
#
# quick script to generate a syslog file for testing
#
# $ gen-log.sh [LINE COUNT] [REPEAT EACH LINE] [APPEND TEXT] [EXTRA NON-DATETIME LINES COUNT] [DATETIME START]
#
# XXX: slow for large LINE COUNT
#
# XXX: It would be nice to have a proper python script to do this.
#      This is good enough for now
#

set -euo pipefail

# hardcoded data to print on each syslog line
declare -ar alphabet=(
  # 0 1 2 3 4 5 6 7 8 9
  # a b c d e f g h i j k l m n o p q r s t u v w x y z
  # A B C D E F G H I J K L M N O P Q R S T U V W Z Y Z
  # À Á Â Ã Ä Å Æ Ç È É Ê Ë Ì Í Î Ï Ð Ñ Ò Ó Ô Õ Ö × Ø Ù Ú Û Ü Ý Þ ß
  # à á â ã ä å æ ç è é ê ë ì í î ï ð ñ ò ó ô õ ö ÷ ø ù ú û ü ý þ ÿ
  😀 😁 😂 😃 😄 😅 😆 😇 😈 😉 😊 😋 😌 😍 😎 😏 😐 😑 😒 😓 😔 😕
  😖 😗 😘 😙 😚 😛 😜 😝 😞 😟 😠 😡 😢 😣 😤 😥 😦 😧 😨 😩 😪 😫
  😬 😭 😮 😯 😰 😱 😲 😳 😴 😵 😶 😷 😸 😹 😺 😻 😼 😽 😾 😿 🙀 🙁
  🙂 🙃
)
declare -ir alen=${#alphabet[@]}

# print this many syslog lines
declare -ir sysline_count=$((${1-100} + 1))
# repeat each syslog line (same datetimestamp) this many times
declare -ir sysline_repeat=${2-1}
# optional text to append
declare -r text_append=${3-}
if [[ -z "${text_append}" ]]; then
    declare -r text_append_empty=true
else
    declare -r text_append_empty=false
fi
declare -ir line_extra=${4-0}
# https://www.epochconverter.com/
# Unix Epoch time at 2020/01/01 00:00:00 GMT
declare -ir epoch_2000_GMT=946684800
# Unix Epoch time at 2020/01/01 00:00:00 PST
declare -ir epoch_2000_PST=946713600
# first sysline datetime is this Unix Epoch time (i.e. a count in seconds)
declare -i dt_start=${5-${epoch_2000_PST}}

declare lc_str=$((sysline_count - 1))  # line count as string
declare -ir lc_i=${#lc_str}  # line count characters wide as integer

# pre-create multiple alphabet lines
declare -i linelen=0
declare -a alphas=()
declare -i a=0
while [[ ${a} -lt ${alen} ]]; do
    declare alpha=''
    declare -i c=a
    # growing line length
    declare -i c_stop=$((c + alen + linelen))
    while [[ ${c} -lt ${c_stop} ]]; do
        alpha+="${alphabet[$((${c} % ${alen}))]}"
        c+=1
    done
    alphas[${a}]=${alpha}
    a+=1
    linelen+=1
done

function print_line_at () {
    printf "%0${lc_i}d" "${1}"
}

declare -r dt_format=${DT_FORMAT-'+%Y%m%dT%H%M%S'}

declare -i sysline_loop=0
declare -i line_at=0
while [[ ${sysline_loop} -lt ${sysline_count} ]]; do
    declare -i sysline_extra=0
    # datetimestamp
    dts=$(date --date "@${dt_start}" "${dt_format}" | tr -d '\n')
    while [[ ${sysline_extra} -lt ${sysline_repeat} ]]; do
        # gather a subset of the alphabet
        declare -i alpha_at=$((${line_at} % ${alen}))
        # print one sysline
        if ${text_append_empty}; then
            echo "${dts} $(print_line_at ${line_at}) ${alphas[${alpha_at}]}"
        else
            echo "${dts} $(print_line_at ${line_at}) ${alphas[${alpha_at}]} ${text_append}"
        fi
        line_at+=1
        declare -i line_extra_at=0
        while [[ ${line_extra_at} -lt ${line_extra} ]]; do
            if ${text_append_empty}; then
                echo "$(print_line_at ${line_at}) ${alphas[${alpha_at}]}"
            else
                echo "$(print_line_at ${line_at}) ${alphas[${alpha_at}]} ${text_append}"
            fi
            line_extra_at+=1
            line_at+=1
        done
        sysline_extra+=1
    done
    # advance datetime by one second
    dt_start+=1
    sysline_loop+=1
done
