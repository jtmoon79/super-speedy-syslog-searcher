#!/usr/bin/env python3
#
# quick Python script to dump a file to hex format that can be copy+pasted into
# a rust `[u8]` array
# I couldn't get `hexdump` or `xxd` or `od` to dump binary data in the
# rust-acceptable format. This script does what I need.
#

from typing import Optional
import sys


WIDTH = 8


def main(path: Optional[str], width: int):
    if path:
        with open(path, "rb") as file_:
            data = file_.read()
    else:
        data = sys.stdin.buffer.read()

    count: int = 0
    for at, byte_ in enumerate(data):
        print(f"0x{byte_:02x},", end="")
        count += 1
        if (at + 1) % width == 0:
            print("")
        else:
            print(" ", end="")
    print(f"printed {count} bytes", file=sys.stderr)

def print_help():
    print("usage:\n  hexdump.py FILE [WIDTH]", file=sys.stderr)
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
    if len(sys.argv) == 3:
        width = int(sys.argv[2])

    main(path, width)
