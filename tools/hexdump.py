#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# file: hexdump.py
#
# quick Python script to dump a file to hex format that can be copy+pasted into
# a rust `[u8]` array
# I couldn't get `hexdump` or `xxd` or `od` to dump binary data in the
# rust-acceptable format. This script does what I need.
#

import sys
from typing import Optional


WIDTH = 8


def main(
        path: Optional[str],
        width: int,
        start: Optional[int],
        offset: Optional[int]
):
    if width < 1:
        raise ValueError("width must be positive")
    if start is not None and start < 0:
        raise ValueError("start must be non-negative")
    if offset is not None and offset < 0:
        raise ValueError("offset must be non-negative")

    if path:
        with open(path, "rb") as file_:
            data = file_.read()
    else:
        data = sys.stdin.buffer.read()

    if start is not None and offset is not None:
        end = start + offset
    else:
        end = None

    count: int = 0
    for at, byte_ in enumerate(data):
        sep = False
        match (start, end):
            case (None, None):
                print(f"0x{byte_:02x},", end="")
                count += 1
                sep = True
            case (start_, None):
                if start_ <= at:
                    print(f"0x{byte_:02x},", end="")
                    count += 1
                    sep = True
            case (None, end_):
                if at < end_:
                    print(f"0x{byte_:02x},", end="")
                    count += 1
                    sep = True
            case (start_, end_):
                if start_ <= at < end_:
                    print(f"0x{byte_:02x},", end="")
                    count += 1
                    sep = True
        if sep:
            if count % width == 0:
                print()
            else:
                print(" ", end="")
    print()
    sys.stdout.flush()
    print(f"printed {count} bytes", file=sys.stderr)


def print_help():
    print("usage:\n  hexdump.py FILE [WIDTH] [START BYTE] [END BYTE]", file=sys.stderr)
    sys.exit(1)


if __name__ == "__main__":
    path = None
    if len(sys.argv) <= 1:
        print_help()
    elif len(sys.argv) >= 2:
        path = sys.argv[1]

    if path == "-h" or path == "--help":
        print_help()

    width = WIDTH
    if len(sys.argv) >= 3:
        base = 10
        if sys.argv[2].startswith("0x"):
            base = 16
        width = int(sys.argv[2], base=base)
    start = None
    if len(sys.argv) >= 4:
        base = 10
        if sys.argv[3].startswith("0x"):
            base = 16
        start = int(sys.argv[3], base=base)
    offset = None
    if len(sys.argv) >= 5:
        base = 10
        if sys.argv[4].startswith("0x"):
            base = 16
        offset = int(sys.argv[4], base=base)
 
    main(path, width, start, offset)
