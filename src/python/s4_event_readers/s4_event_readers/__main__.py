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
        "  etl_reader_dissect_etl: Read ETL files using dissect.etl\n"
        "  etl_reader_etl_parser : Read ETL files using etl-parser\n"
        "  odl_reader            : Read ODL files\n",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
