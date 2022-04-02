#!/usr/bin/env bash
#
# compare-grep-sort1.sh
#
# compare run time of `super_speedy_syslog_searcher` to Unix tools `grep` and `sort` (preferably GNU) 
#

set -euo pipefail

(set -x; uname -a)
(set -x; git log -n1 --format='%h %D')
(set -x; ./target/release/super_speedy_syslog_searcher --version)
# use full path to Unix tools
sort=$(which sort)
grep=$(which grep)
(set -x; $grep --version) | head -n1
(set -x; $sort --version) | head -n1

echo

(
export RUST_BACKTRACE=1
set -x
/usr/bin/time -- \
    ./target/release/super_speedy_syslog_searcher \
    -z 0xFFFF \
    -a 20000101T000000 -b 20000101T080000 \
    ./logs/other/tests/gen-{100-10-......,100-10-BRAAAP,100-10-FOOBAR,100-10-______,100-10-skullcrossbones,100-4-happyface,1000-3-foobar,200-1-jajaja,400-4-shamrock}.log \
    >/dev/null
)

echo

(
set -x
/usr/bin/time -- \
    bash -c "
    $grep -hEe '^20000101T00[01234567][[:digit:]]{3}|^20000101T080000' -- \
    ./logs/other/tests/gen-{100-10-......,100-10-BRAAAP,100-10-FOOBAR,100-10-______,100-10-skullcrossbones,100-4-happyface,1000-3-foobar,200-1-jajaja,400-4-shamrock}.log \
    | $sort -t ' ' -k 1 -s \
    >/dev/null \
    "    
)
