# etl_reader_etl_parser.py
# -*- coding: utf-8 -*-
#

"""
A simple ETL file reader using the `ETLParser`.
Prints a ETL Windows log file as XML-like elements.
Designed for use with s4 EtlReader.

Multi-threaded parser and writer.

Notably slower than the dissect.etl based reader.

Ripped from https://github.com/airbus-cert/etl-parser/blob/master/bin/etl2xml
Thanks to Airbus CERT Team for this great tool!
"""

import os
import sys
from io import BufferedWriter
from queue import Empty, Queue
from threading import Event as Event_Thread
from threading import (
    Lock,
    Thread,
)
from typing import (
    List,
    Optional,
)
from xml.etree import ElementTree
from xml.etree.ElementTree import Element

from construct import (
    Container,
    ListContainer,
    Struct,
)
from etl.error import (
    EtwVersionNotFound,
    EventIdNotFound,
    EventTypeNotFound,
    GroupNotFound,
    GuidNotFound,
    InvalidType,
    TlMetaDataNotFound,
    VersionNotFound,
)

# XXX: this import takes 1096ms to complete
from etl.etl import (
    EtlFile,
    IEtlFileObserver,
    build_from_stream,
)
from etl.event import Event
from etl.parsers.etw.core import Guid
from etl.parsers.kernel import (
    DiskIo_TypeGroup1,
    FileIo_V2_Name,
    ImageLoad,
    ImageLoadProcess,
    Process_Defunct_TypeGroup1,
    Process_V3_TypeGroup1,
    Process_V4_TypeGroup1,
)
from etl.parsers.kernel.core import Mof
from etl.parsers.kernel.io import DiskIo_TypeGroup3
from etl.parsers.tracelogging import TraceLogging
from etl.perf import PerfInfo
from etl.system import SystemTraceRecord
from etl.trace import Trace
from etl.wintrace import WinTrace
from inputimeout import inputimeout, TimeoutOccurred

from . import s4_event_bytes

"""elements queue and synchronization primitives"""
global element_queue
element_queue: Queue = Queue()

global element_queue_lock
element_queue_lock: Lock = Lock()

global parser_done_event
parser_done_event: Event_Thread = Event_Thread()


def add_attribute(parent: Element, name: str, value: str) -> None:
    """
    add attribute `name` and `value` to `parent` element
    """
    attribute = ElementTree.SubElement(parent, "attribute")
    attribute.set("name", name)
    attribute.set("value", value)


def log_kernel_type(mof_object: Mof, xml: Element) -> Element:
    """
    add `mof_object` basded on it's type to the passed `xml`

    :param mof_object: Mof object to log
    :param xml: XML element to fill
    :return: XML Element
    """
    print("log_kernel_type(mof_object=%s)" % mof_object.__class__.__qualname__, file=sys.stderr)

    if isinstance(mof_object, FileIo_V2_Name):
        type_name = {0: "Name", 32: "FileCreate", 35: "FileDelete", 36: "FileRundown"}
        xml.set("type", type_name[mof_object.event_type])
    elif isinstance(mof_object, ImageLoad):
        type_name = {10: "Load", 2: "Unload", 3: "DCStart", 4: "DCEnd"}
        xml.set("type", type_name[mof_object.event_type])
    elif isinstance(mof_object, DiskIo_TypeGroup1):
        type_name = {10: "Read", 11: "Write", 55: "OpticalRead", 56: "OpticalWrite"}
        xml.set("type", type_name[mof_object.event_type])
    elif isinstance(mof_object, DiskIo_TypeGroup3):
        type_name = {14: "FlushBuffers", 57: "OpticalFlushBuffers"}
        xml.set("type", type_name[mof_object.event_type])
    elif isinstance(mof_object, Process_V3_TypeGroup1):
        type_name = {1: "Start", 2: "End", 3: "DCStart", 4: "DCEnd", 39: "Defunct"}
        xml.set("type", type_name[mof_object.event_type])
    elif isinstance(mof_object, Process_V4_TypeGroup1):
        type_name = {1: "Start", 2: "End", 3: "DCStart", 4: "DCEnd", 39: "Defunct"}
        xml.set("type", type_name[mof_object.event_type])
    elif isinstance(mof_object, Process_Defunct_TypeGroup1):
        xml.set("type", "zombie")
    elif isinstance(mof_object, ImageLoadProcess):
        type_name = {10: "Load", 2: "Unload", 3: "DCStart", 4: "DCEnd"}
        xml.set("type", type_name[mof_object.event_type])
    else:
        xml.set("type", mof_object.__class__.__name__)

    return xml


def log_construct_pattern(xml: Element, pattern: Optional[Struct], source: Container):
    """
    For all fields in the pattern, find it in the source and add it to the xml

    :param xml: xml element
    :param pattern: Pattern use by construct
    :param source: Element parsed
    :return: XML Element
    """
    if pattern is None:
        return

    for field in pattern.subcons:
        # check for string
        name = field.name
        value = source[name]
        if hasattr(value, "type"):
            try:
                if value.type == "WString":
                    b = bytearray(value.string[:-2]).decode("utf-16le")
                    add_attribute(xml, name, b)
                elif value.type == "CString":
                    b = bytearray(value.string[:-1]).decode("ascii")
                    add_attribute(xml, name, b)
                else:
                    raise InvalidType("Unsupported source field type %s" % value.type)
            except (UnicodeDecodeError, AttributeError):
                b = bytearray(value).hex()
                add_attribute(xml, name, b)
        elif isinstance(value, ListContainer):
            b = bytearray(value).hex()
            add_attribute(xml, name, b)
        elif isinstance(value, bytes):
            b = bytearray(value).hex()
            add_attribute(xml, name, b)
        elif isinstance(value, Container):
            continue
        else:
            add_attribute(xml, name, str(value))


def log_tracelogging(obj: TraceLogging) -> Element:
    """
    Create a XML element from a tracelogging object

    :param obj: tracelogging object
    :return: XML Element
    """
    xml = ElementTree.Element("tracelogging")
    xml.set("name", obj.get_name())
    for k, v in obj.items():
        if hasattr(v, "type") and v.type == "Guid":
            g = Guid(v.inner.data1, v.inner.data2, v.inner.data3, v.inner.data4)
            add_attribute(xml, k, str(g))
        else:
            add_attribute(xml, k, str(v))
    return xml


def pub_element(element: Element):
    """
    Write the XML `element` to the global queue.
    """
    global element_queue
    global element_queue_lock
    with element_queue_lock:
        element_queue.put(element)


class EtlFileLogger(IEtlFileObserver):
    """
    This an observer that logs parsing events into an XML document.
    """

    def __init__(self):
        self.xml_document = ElementTree.Element("etl")

    def on_system_trace(self, obj: SystemTraceRecord):  # type: ignore
        try:
            mof = obj.get_mof()
            data = ElementTree.SubElement(self.xml_document, "event")
            # data = ElementTree.Element("event")
            data.set("type", "system")
            pid = obj.get_process_id()
            data.set("PID", str(pid))
            tid = obj.get_thread_id()
            data.set("TID", str(tid))
            xml = ElementTree.SubElement(data, "mof")
            xml.set("provider", mof.__class__.__name__)
            log_kernel_type(mof, xml)
            log_construct_pattern(xml, mof.pattern, mof.source)
            pub_element(data)
        except (GroupNotFound, VersionNotFound, EventTypeNotFound) as e:
            print(e, file=sys.stderr)

    def on_perfinfo_trace(self, obj: PerfInfo):
        try:
            mof = obj.get_mof()
            data = ElementTree.SubElement(self.xml_document, "event")
            # data = ElementTree.Element("event")
            data.set("type", "perfinfo")
            t = obj.get_timestamp()
            data.set("timestamp", str(t))
            xml = ElementTree.SubElement(data, "mof")
            xml.set("provider", mof.__class__.__name__)
            log_kernel_type(mof, xml)
            log_construct_pattern(xml, mof.pattern, mof.source)  # type: ignore
            pub_element(data)
        except (GroupNotFound, VersionNotFound, EventTypeNotFound) as e:
            print(e, file=sys.stderr)

    def on_trace_record(self, event: Trace):
        # TODO: implement this
        #       but I can't find a .etl file with a Trace event!
        raise NotImplementedError("Trace event parsing not implemented")
        pass

    def on_event_record(self, event: Event):
        try:
            data = ElementTree.Element("event")
            data.set("type", "event")
            t = event.get_timestamp()
            data.set("timestamp", str(t))
            i = event.get_process_id()
            data.set("PID", str(i))
            tid = event.get_thread_id()
            data.set("TID", str(tid))
            ptl = event.parse_tracelogging()
            ltl = log_tracelogging(ptl)
            data.append(ltl)
            # TODO: should this call `parse_etw()` too?
            # self.xml_document.append(data)
            pub_element(data)
        except TlMetaDataNotFound as t:
            try:
                etw = event.parse_etw()
                # data = ElementTree.SubElement(self.xml_document, "event")
                data = ElementTree.Element("event")
                data.set("type", "event")
                t = event.get_timestamp()
                data.set("timestamp", str(t))
                i = event.get_process_id()
                data.set("PID", str(i))
                tid = event.get_thread_id()
                data.set("TID", str(tid))
                xml = ElementTree.SubElement(data, "etw")
                xml.set("provider", etw.__class__.__name__)
                log_construct_pattern(xml, etw.pattern, etw.source)
                pub_element(data)
            except (EtwVersionNotFound, EventIdNotFound, GuidNotFound) as e:
                print(e, file=sys.stderr)

    def on_win_trace(self, event: WinTrace):
        try:
            etw = event.parse_etw()
            # data = ElementTree.SubElement(self.xml_document, "event")
            data = ElementTree.Element("event")
            data.set("type", "event")
            xml = ElementTree.SubElement(data, "etw")
            xml.set("provider", etw.__class__.__name__)
            log_construct_pattern(xml, etw.pattern, etw.source)
            pub_element(data)
        except (EtwVersionNotFound, EventIdNotFound, GuidNotFound) as e:
            print(e, file=sys.stderr)


def parser_run(etl_file_path: str) -> None:
    """
    Thread entry point.
    Parse the ETL file `etl_file_path` and create XML elements.
    """
    logger = EtlFileLogger()
    with open(etl_file_path, "rb") as input_file:
        input_data = input_file.read()
        # the `build_from_stream` and then `parse` is where the ETL parsing occurs
        etl_file: EtlFile = build_from_stream(input_data)
        etl_file.parse(logger)

    global parser_done_event
    parser_done_event.set()


def write_element(
    element: Element,
    output_file: BufferedWriter,
    human_readable: bool,
) -> None:
    """
    Write XML `element` to `output_file`.
    If `human_readable` is True, write in human readable format.
    Otherwise, write in s4 ETL Reader format.
    """
    element_s: str = ElementTree.tostring(element, encoding="unicode", xml_declaration=False)
    # example event string (with added newlines for readability):
    #
    #    <event type="event" timestamp="2877987559860" PID="29468" TID="24484">
    #        <tracelogging name="Info"><attribute name="m" value="** Service starting **" /></tracelogging>
    #    </event>
    #
    timestamp = element.get("timestamp", "0")

    ts_a = 0
    ts_b = 0

    while True:
        if timestamp == "0":
            break
        a = element_s.find(f'timestamp="{timestamp}"')
        if a == -1:
            break
        a += len('timestamp="')
        b = element_s[a:].find('"')
        if b == -1:
            break
        ts_a = a
        ts_b = a + b
        break

    # timestamp is a Unix epoch number as milliseconds
    # BUG: the timestamp is often a value that is not a valid Epoch time

    # convert string to int
    timestamp_val: int
    try:
        timestamp_val: int = int(timestamp)
    except ValueError:
        print(f"ERROR: Invalid timestamp value: {timestamp!r}", file=sys.stderr)
        timestamp_val = 0
    except OverflowError:
        print(f"ERROR: Overflowed timestamp value: {timestamp!r}", file=sys.stderr)
        timestamp_val = 0

    # # convert int to datetime
    # timestamp_dt: Optional[datetime]
    # try:
    #     # try timestamp is in milliseconds
    #     timestamp_dt = datetime.fromtimestamp(timestamp_val / 1000.0)
    # except ValueError:
    #     try:
    #         # try timestamp is in seconds
    #         timestamp_dt = datetime.fromtimestamp(timestamp_val)
    #     except ValueError:
    #         print(f"ERROR: Invalid range timestamp value: {timestamp!r}", file=sys.stderr)
    #         timestamp_dt = None
    # except OSError:
    #     print(f"ERROR: Out of range timestamp value: {timestamp!r}", file=sys.stderr)
    #     timestamp_dt = None

    # # convert datetime to formatted string
    # timestamp_formatted: str
    # if timestamp_dt is None:
    #     timestamp_formatted = DATETIME_BACKUP
    # else:
    #     try:
    #         timestamp_formatted = timestamp_dt.strftime(DATETIME_FORMAT)
    #     except ValueError:
    #         print(f"ERROR: formatting timestamp value: {timestamp!r}", file=sys.stderr)
    #         timestamp_formatted = DATETIME_BACKUP

    if human_readable:
        element_b = \
            str(timestamp_val).encode("utf-8", errors="ignore") \
            + bytes(" ", "utf-8") \
            + element_s.encode("utf-8", errors="ignore") \
            + bytes("\n", "utf-8")
    else:
        element_b = s4_event_bytes(
            ts_a,
            ts_b,
            timestamp_val,
            element_s,
        )
    output_file.write(element_b)
    output_file.flush()


def main(argv: List[str]) -> int:
    """
    CLI entry point.
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
    wait_input_per_prints = 0
    human_readable = False
    for (i, arg) in enumerate(argv[1:]):
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

    parser_thread = Thread(target=parser_run, args=(etl_file_path,), daemon=True)
    parser_thread.start()

    global element_queue
    global element_queue_lock
    global parser_done_event

    events = 0

    while not parser_done_event.is_set():
        # BUG: the timeout is a bandaid fix to avoid a blocking bug that occurs here
        #      when run by s4
        #      this fails:
        #         ./target/release/s4 -s --etl-parser /mnt/c/Windows/Logs/SIH/SIH.20260102.233447.425.1.etl
        #      yet this succeeds:
        #         ~/.config/s4/venv/bin/python3 -OO -m s4_event_readers.etl_reader_etl_parser \
        #         /mnt/c/Windows/Logs/SIH/SIH.20260102.233447.425.1.etl --wait-input-per-prints=6
        try:
            element = element_queue.get(timeout=0.1)
        except Empty:
            continue
        try:
            write_element(
                element,
                output_file,
                human_readable,
            )
        except Exception:
            pass
        events += 1
        if wait_input_per_prints != 0 and events % wait_input_per_prints == 0:
            # wait for stdin input to continue
            # XXX: with timeout to avoid blocking forever (latent bugs)
            try:
                _ = inputimeout(timeout=0.2)
            except TimeoutOccurred:
                pass

    parser_thread.join()

    while not element_queue.empty():
        element = element_queue.get()
        try:
            write_element(
                element,
                output_file,
                human_readable,
            )
        except Exception:
            pass
        events += 1
        if wait_input_per_prints != 0 and events % wait_input_per_prints == 0:
            # wait for stdin input to continue
            # XXX: with timeout to avoid blocking forever (latent bugs)
            try:
                _ = inputimeout(timeout=0.2)
            except TimeoutOccurred:
                pass

    if output is not None:
        output_file.close()

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
