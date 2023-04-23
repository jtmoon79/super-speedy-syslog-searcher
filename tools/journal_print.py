#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# vim: set fileencoding=utf-8 :
# file: print_journal_events.py
#
# helper tool to double-check .journal file entries found using
# python-systemd:journal.Reader

__doc__: str = """\
Print all events from .journal file in the order returned by
`python-systemd:journal.Reader.get_next`, which itself calls
`libsytemd` API `sd_journal_next`.

Setting environemnt variable `LD_LIBRARY_PATH` can affect which
`libsystemd.so` is used.
"""

import argparse
import os
from typing import List, Optional
import sys

KEY_MONOTONIC_TIMESTAMP = "__MONOTONIC_TIMESTAMP"
KEY_SOURCE_REALTIME_TIMESTAMP = "_SOURCE_REALTIME_TIMESTAMP"
KEY__REALTIME_TIMESTAMP = "__REALTIME_TIMESTAMP"
KEY_MESSAGE_ID = "MESSAGE_ID"
KEY_MESSAGE = "MESSAGE"
SEP = "|"

if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description=__doc__,
        prog=os.path.basename(__file__),
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
        # TODO: how to both show defaults and preserve newlines in help message?
        epilog=(
            """Works well when piped to column.

"""
            + os.path.basename(__file__)
            + """ -j /var/log/journal/$(hostname)/system.journal | column -t -s '|'"""
        ),
    )
    parser.add_argument(
        "--journal-file",
        "-j",
        type=str,
        required=True,
    )
    parser.add_argument(
        "fields",
        metavar="field",
        type=str,
        nargs="*",
        default=",".join(
            (
                KEY_MONOTONIC_TIMESTAMP,
                KEY_SOURCE_REALTIME_TIMESTAMP,
                KEY__REALTIME_TIMESTAMP,
                KEY_MESSAGE_ID,
                KEY_MESSAGE,
            )
        ),
        help="print these journal entry fields separated by commas",
    )
    parser.add_argument(
        "--time",
        "-t",
        type=int,
        default=None,
        help="Unix epoch in microseconds to start printing from",
    )
    parser.add_argument(
        "--oneline",
        "-o",
        type=bool,
        default=False,
        help="Strip newlines from MESSAGE field",
    )
    parser.add_argument(
        "--separator",
        "-s",
        type=str,
        default=SEP,
        help="Separator between fields",
    )

    args = parser.parse_args()

    # late import to allow user to see `--help` message without this error
    try:
        from systemd import journal
    except ImportError as err:
        print(err, file=sys.stderr)
        print(
            "Install on Ubuntu with:\n    sudo apt install python3-systemd",
            file=sys.stderr,
        )
        print("Or with pip:\n    pip install systemd-python", file=sys.stderr)
        sys.exit(1)

    journal_file: str = args.journal_file
    fields: List = []
    if isinstance(args.fields, str):
        fields += args.fields.split(",")
    elif isinstance(args.fields, list):
        for field in args.fields:
            fields += field.split(",")
    time_: Optional[str] = args.time
    oneline: bool = args.oneline
    sep = args.separator

    reader = journal.Reader(files=[journal_file])

    if time_ is not None:
        t: int = int(time_)
        reader.seek_realtime(t)
    else:
        reader.seek_head()

    try:
        print(f"index", end="")
        for field in fields:
            print(f"{sep}{field}", end="")
        print()
        sys.stdout.flush()
        i = 0
        while entry := reader.get_next():
            i += 1
            print(f"{i}", end="")
            for field in fields:
                # special handling for monotonic timestamp
                if field == KEY_MONOTONIC_TIMESTAMP:
                    mt_ts = entry.get(KEY_MONOTONIC_TIMESTAMP, " ")
                    mt_ts = str(mt_ts.timestamp)
                    print(f"{sep}{mt_ts}", end="")
                else:
                    # defalt print at least a space to piping stdout to
                    # `column -t -s '|'` will correctly parse
                    value = entry.get(field, " ")
                    # replace some control characters with glyph representations
                    if oneline:
                        value = value.replace("\0", "␀")
                        value = value.replace("\n", "␤")
                        value = value.replace("\1", "␁")
                        value = value.replace("\r", "␊")
                    print(f"{sep}{value}", end="")
            print()
            sys.stdout.flush()
    except BrokenPipeError:
        # this occurs when piping to `head`
        pass

    print("Printed %d entries" % i, file=sys.stderr)
