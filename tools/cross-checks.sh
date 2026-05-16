#!/usr/bin/env bash
#
# run `cargo cross` checks on all supported targets.
# these targets should match those in the `rust.yml` workflow file.
# requires:
#     cargo install --locked cross cargo-cross
#

# add targets with command:
#     rustup target add $TARGET
for TARGET in \
    aarch64-unknown-linux-gnu \
    i686-pc-windows-gnu \
    `# i686-pc-windows-msvc` \
    i686-unknown-linux-gnu \
    x86_64-pc-windows-gnu \
    `# x86_64-pc-windows-msvc` \
    x86_64-unknown-linux-gnu \
    loongarch64-unknown-linux-gnu \
    aarch64-unknown-linux-musl \
    arm-unknown-linux-gnueabi \
    arm-unknown-linux-gnueabihf \
    armv7-unknown-linux-gnueabihf \
    powerpc-unknown-linux-gnu \
    powerpc64-unknown-linux-gnu \
    riscv64gc-unknown-linux-gnu \
    x86_64-unknown-freebsd \
    `# x86_64-unknown-illumos` \
    x86_64-unknown-linux-musl \
    x86_64-unknown-netbsd \
    aarch64-linux-android \
    i686-linux-android \
    x86_64-pc-solaris \
    x86_64-sun-solaris \
    x86_64-linux-android \
    x86_64-unknown-redox \
    `# mips64-unknown-linux-gnuabi64` \
; do
    (
        set -x
        S4_BUILD_REGEX=1 S4_BUILD_REGEX_PRINT=1 cross check --lib --bins --target $TARGET
    )
done
