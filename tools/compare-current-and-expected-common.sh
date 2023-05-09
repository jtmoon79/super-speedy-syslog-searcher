# common variables for
# `tools/compare-current-and-expected.sh` and
# `tools/compare-current-and-expected-update.sh`
#
# this should be sourced by both scripts

PROGRAM=${PROGRAM-./target/release/s4}
readonly PROGRAM

declare -arg S4_ARGS=(
    --color=never
    --tz-offset=+08:00
    --prepend-filepath
    --prepend-file-align
    --prepend-utc
    --prepend-separator='┋'
    --separator='⇳\n'
    --journal-output=export
    --dt-after='19990303T000000+0000'
    --dt-before='20230410T221032+0000'
    --summary
    '-'
    "${@}"
)
