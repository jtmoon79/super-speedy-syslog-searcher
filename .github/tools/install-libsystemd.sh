#!/usr/bin/env bash
#
# install libsystemd if is not found

set -ux

find /bin /sbin /usr /var -xdev \
        \( -type f -o -type l \) \
        -name 'libsystemd*.so*' 2>/dev/null \
    | sort \
    | tee /tmp/libsystemd-find.txt

if [[ -s /tmp/libsystemd-find.txt ]]; then
    echo "libsystemd found; skip apt install"
    exit 0
fi

set -e

sudo apt update --yes
sudo apt install --yes libsystemd0

set +e

find /bin /sbin /usr /var -xdev \
        \( -type f -o -type l \) \
        -name 'libsystemd*.so*' 2>/dev/null \
    | sort
