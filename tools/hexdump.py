#!/usr/bin/env python3
#
# quick Python script to dump a file to hex format that can be copy+pasted into
# a rust `[u8]` array
# believe it or not, I couldn't get `hexdump` or `xxd` or `od` to dump binary
# data in the format I wanted

import sys


WIDTH = 8


def main(path: str, width: int):
    with open(path, "rb") as file_:
        data = file_.read()
    for at, byte_ in enumerate(data):
        print(f"0x{byte_:02x},", end="")
        if (at + 1) % width == 0:
            print("")
        else:
            print(" ", end="")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("usage:\n  hexdump.py FILE [WIDTH]", file=sys.stderr)
        sys.exit(1)
    path = sys.argv[1]
    width = WIDTH
    if len(sys.argv) == 3:
        width = int(sys.argv[2])
    main(path, width)
