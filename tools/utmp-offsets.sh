# utmp-offsets.sh
#
# wrapper to compile and run utmp-offsets.c

set -eu

pushd "$(dirname "${0}")"

rm -f ./utmp-offsets
(
    set -x
    cc -o ./utmp-offsets utmp-offsets.c
)
PROG=$(readlink -f ./utmp-offsets)
if [[ -r /etc/os-release ]]; then
    . /etc/os-release
else
    NAME=$(uname -s | tr -d '\n'; echo -n "_")
    VERSION_ID=$(uname -r | tr -d '\n'; echo -n "_")
fi
# replace any '/' with '_'
NAME=${NAME//\//_}
ARCH=$((uname -p | tr -d '\n') && echo -n "_")
popd
OUTPUT="./utmp-offsets_${ARCH}${NAME}${VERSION_ID-}.out"
set -x
uname -srvmpio > "${OUTPUT}"
if [[ -n "${PRETTY_NAME-}" ]]; then
    echo "${PRETTY_NAME-}" >> "${OUTPUT}"
fi
echo >> "${OUTPUT}"
"${PROG}" | tee -a "${OUTPUT}"
