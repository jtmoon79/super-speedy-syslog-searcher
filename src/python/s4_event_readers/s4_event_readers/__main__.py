# __main__.py
# -*- coding: utf-8 -*-
#

import sys

from . import __version__


def main():
    print("s4_event_readers command-line interface", file=sys.stderr)
    print(
        f"s4_event_readers version: {__version__}\n"
        "Call with submodules:\n"
        "  odl_reader            : Read ODL files (.odl, .aodl, .odlgz, .odlsent)\n"
        "  ccl_asldb             : Read ASL files (.asl)\n",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
