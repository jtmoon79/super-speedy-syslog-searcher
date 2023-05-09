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
        end: Optional[int]
):
    if path:
        with open(path, "rb") as file_:
            data = file_.read()
    else:
        data = sys.stdin.buffer.read()

    count: int = 0
    for at, byte_ in enumerate(data):
        sep = False
        match (start, end):
            case (None, None):
                print(f"0x{byte_:02x},", end="")
                count += 1
                sep = True
            case (start, None):
                if start <= at:
                    print(f"0x{byte_:02x},", end="")
                    count += 1
                    sep = True
            case (None, end_):
                if at < end_:
                    print(f"0x{byte_:02x},", end="")
                    count += 1
                    sep = True
            case (start, end_):
                if start <= at < end_:
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
    if len(sys.argv) == 0:
        print_help()
    elif len(sys.argv) >= 2:
        path = sys.argv[1]

    if path == "-h" or path == "--help":
        print_help()

    width = WIDTH
    if len(sys.argv) >= 3:
        width = int(sys.argv[2])
    start = None
    if len(sys.argv) >= 4:
        start = int(sys.argv[3])
    end = None
    if len(sys.argv) >= 5:
        end = int(sys.argv[4])
 
    main(path, width, start, end)
