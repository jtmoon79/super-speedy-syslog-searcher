# etl_reader_dissect_etl.py
# -*- coding: utf-8 -*-
#

"""
A simple ETL file reader using the `dissect.etl` library.
Prints a ETL Windows log file as XML-like elements.
Designed for use with s4 EtlReader.

Skips use of complex ArgParser or other niceties to keep this code fast.

Notably faster than the etl_reader_etl_parser.py based reader.
"""

import inspect
import os
import sys
from datetime import (
    datetime,
    timezone,
)
from io import (
    BufferedWriter,
    TextIOWrapper,
)
from typing import List, Optional

from dissect.etl import ETL
from dissect.etl.etl import Event, EventRecord  # noqa: F401

from . import s4_event_bytes


def log(logf: TextIOWrapper, message: str, end="\n") -> None:
    """simple wrapper for simple debug logging"""
    logf.write(message)
    logf.write(end)
    logf.flush()


def main(argv: List[str]) -> int:
    """
    CLI entry point.

    Parse the arguments and print ETL events to stdout in manner that
    can be consumed by s4 ETL Reader.
    Option --human-readable prints the events in human readable format
    otherwise prints in s4 ETL Reader format.
    """

    usage = f"Usage: {os.path.basename(sys.argv[0])}" \
        " <etl-file>" \
        " [--output=<output-file>]" \
        " [--wait-input-per-prints=<wait-input-per-prints>]" \
        " [--human-readable]"
    if len(argv) < 1:
        print(usage, file=sys.stderr)
        return 1

    # primitive arg parsing avoids creation of argparse objects
    etl_file_path = argv[0]
    output: Optional[str] = None
    wait_input_per_prints = 0  # default: do not wait
    human_readable = False
    for arg in argv[1:]:
        if arg.startswith("--output=") or arg.startswith("-o="):
            output = arg.split("=", 1)[1]
        elif arg.startswith("--wait-input-per-prints=") or arg.startswith("-w="):
            w = arg.split("=", 1)[1]
            wait_input_per_prints = int(w)
        elif arg == "--human-readable":
            human_readable = True
        else:
            print(f"Unknown argument: {arg!r}\n", file=sys.stderr)
            print(usage, file=sys.stderr)
            return 1

    output_file: BufferedWriter = \
        sys.stdout.buffer if output is None else open(output, "ab")  # type: ignore

    # `debug_` vars only for debugging and testing purposes
    debug_count_raise: int = int(os.getenv("S4_ETL_READER_DEBUG_COUNT_RAISE", -1))
    """raise exception after processing this many events; -1 means do not raise"""
    debug_file_path: Optional[str] = os.getenv("S4_ETL_READER_DEBUG_FILE_PATH", None)
    """path to a debug log file; None means do not log"""
    debug_breakpoint: Optional[str] = os.getenv("S4_ETL_READER_DEBUG_BREAKPOINT", None)
    """if not None, will invoke breakpoint() for each event processed"""

    count: int = 0

    # primitive debug log file to track progress
    if debug_file_path:
        logf = open(debug_file_path, "a")
        log(logf, f"Starting processing ETL file {etl_file_path!r}")

    bytes_printed: int = 0

    with open(etl_file_path, "rb") as fh:
        etl: ETL = ETL(fh)
        # BUG: type checker thinks `record` is an `Event` but
        #      runtime proves that `record` is an `EventRecord`
        # record: EventRecord
        record: Event
        for record in etl:
            r"""
            Two example printed Events look like

                <SystemHeader version=2 provider_id=68fdd910-4a3e-11d1-84f4-0000f80464e3 timestamp=2025-10-05 11:30:19.201590+00:00 ThreadId=24484 ProcessId=29468 ProcessorTime=0> <EventTraceEvent/Header <EventTrace_Header BufferSize=0x2000 Version=0x501000a ProviderVersion=0x5867 NumberOfProcessors=0x1 EndTime=0x1dc35eb91cabb86 TimerResolution=0x2625a MaxFileSize=0x800 LogFileMode=0x11002002 BuffersWritten=0x2 StartBuffers=0x1 PointerSize=0x8 EventsLost=0x0 CPUSpeed=0x118b LoggerName=10 LogFileName=7 TimeZoneInformation=[224, 1, 0, 0, 64, 0, 116, 0, 122, 0, 114, 0, 101, 0, 115, 0, 46, 0, 100, 0, 108, 0, 108, 0, 44, 0, 45, 0, 50, 0, 49, 0, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 116, 0, 122, 0, 114, 0, 101, 0, 115, 0, 46, 0, 100, 0, 108, 0, 108, 0, 44, 0, 45, 0, 50, 0, 49, 0, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 196, 255, 255, 255, 0, 0, 0, 0] BootTime=0x1dc334d5cb2eac0 PerfFreq=0x989680 StartTime=0x1dc35eb6deb9a24 ReservedFlags=0x1 BuffersLost=0x0 SessionNameString='ECCB175F-1EB2-43DA-BFB5-A8D58A40A4D7' LogFileNameString='C:\\Windows\\logs\\waasmedic\\waasmedic.20251005_113019_195.etl'>>

                <EventHeader version=0 provider_id=2451d231-68a4-5c50-de82-8411646eb8b5 timestamp=2025-10-05 11:30:19.202053+00:00 ThreadId=24484 ProcessId=29468 ActivityId=00000000-0000-0000-0000-000000000000 Extensions=[{'ExtType': <ExtType.PROV_TRAITS: 12>, 'TraitSize': 36, 'ProviderName': b'Microsoft.Windows.WaaSMedic.Local', 'Traits': []}, {'ExtType': <ExtType.EVENT_SCHEMA_TL: 11>, 'EventSchema': b'\x0b\x00\x00Info\x00m\x00\x01'}]> <None None>

            """  # noqa: E501
            count += 1
            event: Event = record.event
            try:
                r"""
                The example of the project website suggesting searching
                among returned Event field attributes.
                But it was found in practice that the most accurate way to
                retrieve the timestamp is to scan for substring "timestamp=".
                This methods also gets us the string offsets of that timestamp
                substring which also needs to be returned.
                TODO: find the URL of where I found that example code

                presumes timestamp substring looks like:

                        ... timestamp=2025-10-05 11:30:20.957701+00:00 ...

                for each Event, print:
                        <ts_a><DELIMITER_TS_EVENT><ts_c><DELIMITER_TS_EVENT><timestamp><DELIMITER_TS_EVENT><event><DELIMITER_EVENT_END>
                """
                if debug_breakpoint is not None:
                    breakpoint()

                event_s = str(event)

                # remove junk "<None None>" at end if present
                if event_s.endswith("<None None>"):
                    event_s = event_s[: -len("<None None>")].rstrip()

                # append each interesting Event attribute as XML-like element
                # if they are not already in event_s
                event_s += "<Event"
                for (name, value) in inspect.getmembers(event):
                    # do not include private or methods/functions
                    # skip ts() as that is already printed in `event_s`
                    if name.startswith("_") or name in ("ts", "provider_id"):
                        continue
                    if not inspect.ismethod(value) and not inspect.isfunction(value):
                        if f"{name}=" not in event_s:
                            event_s += f" {name}=\"{str(value)}\""
                    else:
                        try:
                            val = value()
                            if val:
                                if name == "event_values":
                                    # event_values is a dict, print as key="value" pairs
                                    for (k, v) in val.items():
                                        if f"{k}=" not in event_s:
                                            event_s += f" {k}=\"{str(v)}\""
                                else:
                                    if f"{name}=" not in event_s:
                                        event_s += f" {name}=\"{str(val)}\""
                        except Exception:
                            pass
                event_s += " />"

                if human_readable:
                    # write to stdout as bytes
                    output_file.write(event_s.encode("utf-8", errors="ignore"))
                    output_file.write(b"\n")
                    output_file.flush()
                    if wait_input_per_prints > 0 and count % wait_input_per_prints == 0:
                        _ = input()
                    continue

                # it was found that scanning the string for "timestamp="
                # is the most reliable way to get the timestamp substring
                timestamp = None
                ts_a = 0
                ts_c = 0
                ts_a = event_s.find("timestamp=")
                if ts_a == -1:
                    raise ValueError(f"Timestamp not found in {event_s[:200]!r}")
                ts_a += len("timestamp=")
                ts_b = event_s[ts_a:].find(" ")
                if ts_b == -1:
                    raise ValueError(f"Timestamp not found in {event_s[:200]!r}")
                ts_b = ts_a + ts_b + len(" ")
                ts_c = event_s[ts_b:].find(" ")
                if ts_c == -1:
                    raise ValueError(f"Timestamp not found in {event_s[:200]!r}")
                ts_c = ts_b + ts_c
                timestamp = event_s[ts_a:ts_c]

                # convert timestamp to milliseconds since Unix epoch in UTC
                dt = datetime.fromisoformat(timestamp)
                dt_utc = dt.astimezone(timezone.utc)
                timestamp_val = int(dt_utc.timestamp() * 1000)

                # event record as bytes
                event_b = s4_event_bytes(
                    ts_a,
                    ts_c,
                    timestamp_val,
                    event_s,
                )
                # write to stdout as bytes
                output_file.write(event_b)
                output_file.flush()

                if debug_file_path:
                    bytes_printed += len(event_b)
                    log(logf, f"Processed event {count} ({len(event_b)} bytes of {bytes_printed} total bytes)")

                if wait_input_per_prints > 0 and count % wait_input_per_prints == 0:
                    if debug_file_path:
                        log(logf, f"Waiting for input after {count} events")
                    # wait for stdin input to continue
                    _in = input()
                    if debug_file_path:
                        log(logf, f"Received input {_in!r}")
            except ValueError as ve:
                print(ve, file=sys.stderr)
                if debug_file_path:
                    log(logf, f"ValueError: {ve}")
            except Exception as e:
                print(e, file=sys.stderr)
                if debug_file_path:
                    log(logf, f"Exception: {e}")
                # should this be fatal?
            # only intended to aid debugging and testing
            if debug_count_raise != -1 and count > debug_count_raise:
                if debug_file_path:
                    log(logf, f"Debug limit of {debug_count_raise} events reached")
                raise RuntimeError(f"Debug limit of {debug_count_raise} events reached")

    if debug_file_path:
        log(logf, f"Completed processing {count} events from {etl_file_path!r}")
        logf.close()

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
