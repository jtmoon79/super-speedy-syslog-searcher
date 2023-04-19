#!/usr/bin/env python3
#
# file: print_journal_events.py
# -*- coding: utf-8 -*-
#
# helper tool to double-check .journal file entries found using
# python-systemd:journal.Reader

__doc__: str = \
"""\
Print all events from .journal file in the order returned by
`python-systemd:journal.Reader.get_next`, which itself calls
`libsytemd` API `sd_journal_next`.

Setting environemnt variable `LD_LIBRARY_PATH` can affect which
`libsystemd.so` is used.
"""

import os
from typing import Optional
import sys
try:
    from systemd import journal
except ImportError as err:
    print(err, file=sys.stderr)
    print("Install on Ubuntu with:\n    sudo apt install python3-systemd", file=sys.stderr)
    print("Or with pip:\n    pip install systemd-python", file=sys.stderr)
    sys.exit(1)

# default Journal File
JOURNAL_FILE = "./logs/programs/journal/user-1000.journal"
KEY_MONOTONIC_TIMESTAMP = "__MONOTONIC_TIMESTAMP"
KEY_SOURCE_REALTIME_TIMESTAMP = "_SOURCE_REALTIME_TIMESTAMP"
KEY__REALTIME_TIMESTAMP = "__REALTIME_TIMESTAMP"
KEY_MESSAGE_ID = "MESSAGE_ID"
KEY_MESSAGE = "MESSAGE"
SEP = "|"

if __name__ == "__main__":
    file_: str = JOURNAL_FILE
    time_: Optional[str] = None
    # primitive argument parsing
    do_help = False
    if len(sys.argv) > 1:
        file_ = sys.argv[1]
        if file_ in ("-h", "--help", "-?"):
            do_help = True
    if len(sys.argv) > 2:
        time_ = sys.argv[2]
    if len(sys.argv) > 3 or len(sys.argv) <= 1 or do_help:
        print(f"""Usage:
    {os.path.basename(__file__ )} journal_file [time]

where journal_file is the path to a .journal file.
where time is from Unix epoch in microseconds.

About:
""", file=sys.stderr)
        print(__doc__, file=sys.stderr)
        sys.exit(1)

    reader = journal.Reader(files=[file_])

    if time_ is not None:
        t: int = int(time_)
        reader.seek_realtime(t)
    else:
        reader.seek_head()

    try:
        print(f"index", end="")
        print(f"{SEP}{KEY_MONOTONIC_TIMESTAMP}", end="")
        print(f"{SEP}{KEY_SOURCE_REALTIME_TIMESTAMP}", end="")
        print(f"{SEP}{KEY__REALTIME_TIMESTAMP}", end="")
        print(f"{SEP}{KEY_MESSAGE_ID}", end="")
        print(f"{SEP}{KEY_MESSAGE}")
        sys.stdout.flush()
        i = 0
        while entry := reader.get_next():
            i += 1
            mt_ts = entry.get(KEY_MONOTONIC_TIMESTAMP, "")
            mt_ts = str(mt_ts.timestamp)
            srt_ts = entry.get(KEY_SOURCE_REALTIME_TIMESTAMP, " ")
            rt_ts = entry.get(KEY__REALTIME_TIMESTAMP, " ")
            mesg_id = entry.get(KEY_MESSAGE_ID, " ")
            mesg_id = str(mesg_id).replace("-", "")
            message = entry.get(KEY_MESSAGE, " ")
            message = message.replace("\n", " ")
            print(f"{i}", end="")
            print(f"{SEP}{mt_ts}", end="")
            print(f"{SEP}{srt_ts}", end="")
            print(f"{SEP}{rt_ts}", end="")
            print(f"""{SEP}{mesg_id}""", end="")
            print(f"{SEP}{message}")
            sys.stdout.flush()
    except BrokenPipeError:
        # this occurs when piping to `head`
        pass

    print("Printed %d entries" % i, file=sys.stderr)
