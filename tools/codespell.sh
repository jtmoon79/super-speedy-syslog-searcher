#!/usr/bin/env bash
#
# run codespell with preferred settings
#

set -eu

if ! which codespell; then
    echo "ERROR cannot find codespell in PATH" >&2
    echo "    python -m pip install codespell" >&2
    exit 1
fi

# ignore list words
RUST='crate,mut'
VARS='fo,syslog,datas,DATAS'
tz='ist,pont'
TZ='ACDT,ACST,ACT,ADT,AEDT,AEST,AET,AFT,AKDT,AKST,ALMT,AMST,AMT,ANAT,AQTT,ART,AST,AWST,AZOST,AZOT,AZT,BIOT,BIT,BNT,BOT,BRST,BRT,BST,BTT,CAT,CCT,CDT,CEST,CET,CHOST,CHOT,CHST,CHUT,CIST,CKT,CLST,CLT,COST,COT,CST,CT,CVT,CWST,CXT,DAVT,DDUT,DFT,EASST,EAST,EAT,ECT,EDT,EEST,EET,EGST,EGT,EST,ET,FET,FJT,FKST,FKT,FNT,GALT,GAMT,GET,GFT,GILT,GIT,GMT,GST,GYT,HAEC,HDT,HKT,HMT,HOVST,HOVT,HST,ICT,IDLW,IDT,IOT,IRDT,IRKT,IRST,IST,JST,KALT,KGT,KOST,KRAT,KST,LHST,LINT,MAGT,MART,MAWT,MDT,MEST,MET,MHT,MIST,MIT,MMT,MSK,MST,MUT,MVT,MYT,NCT,NDT,NFT,NOVT,NPT,NST,NT,NUT,NZDT,NZST,OMST,ORAT,PDT,PET,PETT,PGT,PHOT,PHST,PHT,PKT,PMDT,PMST,PONT,PST,PWT,PYST,PYT,RET,ROTT,SAKT,SAMT,SAST,SBT,SCT,SDT,SGT,SLST,SRET,SRT,SST,SYOT,TAHT,TFT,THA,TJT,TKT,TLT,TMT,TOT,TRT,TVT,ULAST,ULAT,UTC,UYST,UYT,UZT,VET,VLAT,VOLT,VOST,VUT,WAKT,WAST,WAT,WEST,WET,WGST,WGT,WIB,WIT,WITA,WST,YAKT,YEKT,ZULU,Z'
SKIP_FILES="compare-current-and-expected_expected.stderr,\
compare-current-and-expected_expected.stdout,\
compare-current-and-expected_current.stderr,\
compare-current-and-expected_current.stdout\
"
# an important distinction about the ignore list and dictionaries
# https://github.com/codespell-project/codespell/issues/2451#issuecomment-1218084118

cd "$(dirname -- "${0}")/.."

set -x

exec \
    codespell \
    --builtin "clear,rare,usage,names" \
    --skip "${SKIP_FILES}" \
    --ignore-words-list="${RUST},${VARS},${tz},${TZ}" \
    ./benches \
    ./Extended-Thoughts.md \
    ./README.md \
    ./src/ \
    $(find ./tools -maxdepth 1 -type f) \
