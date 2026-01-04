#!/usr/bin/env python3

"""
Copyright (c) 2012, CCL Forensics
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:
    * Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above copyright
      notice, this list of conditions and the following disclaimer in the
      documentation and/or other materials provided with the distribution.
    * Neither the name of the CCL Forensics nor the
      names of its contributors may be used to endorse or promote products
      derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL CCL FORENSICS BE LIABLE FOR ANY
DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
"""

import argparse
import datetime
import os
import struct
import sys
from pathlib import Path
from typing import BinaryIO, Dict, Generator, List, Optional, Tuple

try:
    from . import s4_event_bytes
except ImportError:
    # allow running from local directory for development/testing
    sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
    from s4_event_readers import s4_event_bytes
    sys.path.pop(0)

__version__ = "0.7.79"
__description__ = "Parses Apple .asl log files"
__author__ = "modified by James Thomas Moon for super-speedy-syslog-searcher; " \
    "taken from ccl_asldb by Alex Caithness at https://github.com/cclgroupltd/ccl-asl"

MAGIC = b"ASL DB\x00\x00\x00\x00\x00\x00"

MESSAGE_LEVELS = ["Emergency", "Alert", "Critical", "Error", "Warning", "Notice", "Info", "Debug"]

UNIX_EPOCH = datetime.datetime(1970, 1, 1)

FILES_HAVE_MASK = True

QUIET = False

global record_field_sep
record_field_sep = "\t"

KeyValueDict = Dict[str, str]


def print_log(s: str) -> None:
    sys.stderr.write("LOG: ")
    sys.stderr.write(s)
    sys.stderr.write("\n")
    sys.stderr.flush()


def print_err(s: str) -> None:
    sys.stderr.write("ERROR: ")
    sys.stderr.write(s)
    sys.stderr.write("\n")
    sys.stderr.flush()


def parse_epoch_value(sec: int) -> datetime.datetime:
    return UNIX_EPOCH + datetime.timedelta(seconds=sec)


class AslDbError(Exception):
    pass


class AslRecord:
    """
    Class representing a log record entry in an ASL file.
    """

    def __init__(
        self,
        offset: int,
        id: int,
        timestamp: datetime.datetime,
        level: int,
        flags: int,
        pid: int,
        gid: int,
        ruid: int,
        rgid: int,
        refpid: int,
        host: str,
        sender: str,
        facility: str,
        message: str,
        refproc: str,
        session: str,
        key_value_dict: KeyValueDict,
    ):
        """
        Constructor, should really only be called from AslDb._parse_record
        """
        self.offset = offset
        self.id = id
        self.timestamp = timestamp
        self.level = level
        self.level_str = MESSAGE_LEVELS[level] if level < len(MESSAGE_LEVELS) else "Other"
        self.flags = flags
        self.pid = pid
        self.gid = gid
        self.ruid = ruid
        self.rgid = rgid
        self.refpid = refpid
        self.host = host
        self.sender = sender
        self.facility = facility
        self.message = message
        self.refproc = refproc
        self.session = session
        self.key_value_dict = key_value_dict

    def __repr__(self) -> str:
        global record_field_sep
        s = self.timestamp.isoformat() + record_field_sep
        s += "id={0}".format(self.id) + record_field_sep
        s += "level={0}".format(self.level_str) + record_field_sep
        s += "pid={0}".format(self.pid) + record_field_sep
        s += "gid={0}".format(self.gid) + record_field_sep
        s += "read_uid={0}".format(self.ruid) + record_field_sep
        s += "read_gid={0}".format(self.rgid) + record_field_sep
        if self.host:
            s += "host={0}".format(self.host) + record_field_sep
        if self.refproc:
            s += "RefProc={0}".format(self.refproc) + record_field_sep
        if self.session:
            s += "session={0}".format(self.session) + record_field_sep
        s += "sender={0}".format(self.sender) + record_field_sep
        s += "facility={0}".format(self.facility) + record_field_sep
        s += "message='{0}'".format(self.message) + record_field_sep
        for key, value in self.key_value_dict.items():
            s += "{0}={1}".format(key, value) + record_field_sep
        # remove last separator
        if s.endswith(record_field_sep):
            s = s[:-len(record_field_sep)]

        return s

    def __str__(self) -> str:
        return self.__repr__()


class AslDb:
    """
    Class representing an ASL file.
    """

    def _parse_asl_str(self, val: int) -> str:
        """
        Takes an 64bit integer and depending on the top bit, either extracts the string encoded in
        the integer, or takes the value as an offset and grabs the string from that position in the
        file.

        Should only really be called by AslDb._parse_record
        """
        if val == 0:
            string = ""
        elif val & 0x8000000000000000 == 0:
            # is a reference
            self.f.seek(val)
            str_tag = self.f.read(2)
            if str_tag != b"\x00\x01":
                raise AslDbError("String field does not begin with \x00\x01")
            (str_len,) = struct.unpack(">I", self.f.read(4))
            string = self.f.read(str_len - 1).decode()  # minus 1 as it is nul-terminated
        else:
            # is embedded
            str_bytes = struct.pack(">Q", val)
            str_len = str_bytes[0] & 0x7F
            string = str_bytes[1:1 + str_len].decode()

        return string

    def _parse_record(self, offset: int) -> AslRecord:
        """
        Parses the record at offset.
        """
        if not QUIET:
            print_log(f"Parsing record at offset {offset}")

        # Get the data
        self.f.seek(offset)
        (
            rec_len,
            next_rec,
            id,
            timestamp_seconds,
            timestamp_nano,
            level,
            flags,
            pid,
            uid,
            gid,
            ruid,
            rgid,
            refpid,
            kv_count,
            host_ref,
            sender_ref,
            facility_ref,
            message_ref,
            refproc_ref,
            session_ref,
        ) = struct.unpack(">2xI3QI2H7I6Q", self.f.read(114))
        key_value_refs: List[Tuple[int, int]] = []

        for i in range(kv_count // 2):
            key_value_refs.append(struct.unpack(">2Q", self.f.read(16)))

        # _prev_rec = struct.unpack(">Q", self.f.read(8))
        _ = self.f.read(8)

        # Parse the data
        timestamp = parse_epoch_value(timestamp_seconds + (timestamp_nano / 1000000000))
        host = self._parse_asl_str(host_ref)
        sender = self._parse_asl_str(sender_ref)
        refproc = self._parse_asl_str(refproc_ref)
        facility = self._parse_asl_str(facility_ref)
        message = self._parse_asl_str(message_ref)
        session = self._parse_asl_str(session_ref)

        key_value_dict: KeyValueDict = {}
        for key, value in key_value_refs:
            key_value_dict[self._parse_asl_str(key)] = self._parse_asl_str(value)

        return AslRecord(
            offset,
            id,
            timestamp,
            level,
            flags,
            pid,
            gid,
            ruid,
            rgid,
            refpid,
            host,
            sender,
            facility,
            message,
            refproc,
            session,
            key_value_dict,
        )

    def __init__(self, stream: BinaryIO) -> None:
        if not QUIET:
            print_log(f"Creating AslDb from stream {stream}")

        self.f = stream

        # Read header magic
        magic = self.f.read(12)
        if magic != MAGIC:
            raise AslDbError(f"Invalid header (Expected: {MAGIC!r} Received: {magic!r})")

        (self.version,) = struct.unpack(">I", self.f.read(4))
        (first_record_offset,) = struct.unpack(">Q", self.f.read(8))
        ts = struct.unpack(">q", self.f.read(8))[0]
        self.timestamp = parse_epoch_value(ts)
        (self.string_cache_size,) = struct.unpack(">I", self.f.read(4))
        if FILES_HAVE_MASK:
            # _filter_mask = self.f.read(1)[0]  # WUT??!
            _ = self.f.read(1)
        (self.last_record_offset,) = struct.unpack(">Q", self.f.read(8))
        _ = self.f.read(36)  # should be all 0x00 - maybe worth checking?

        # initialise offset list - just read the "next" field from each record so that initialisation is speedy
        self._record_offsets: List[int] = []
        self._record_offsets.append(first_record_offset)
        next_offset = first_record_offset

        while next_offset != self.last_record_offset:
            # first 6 bytes of a record: 2 bytes of 0x00 followed by 32bit int for length
            self.f.seek(next_offset + 6)
            (n,) = struct.unpack(">Q", self.f.read(8))
            self._record_offsets.append(n)
            next_offset = n

    def __iter__(self) -> Generator[AslRecord, None, None]:
        for o in self._record_offsets:
            yield self._parse_record(o)

    def __getitem__(self, index: int) -> AslRecord:
        if index > self.__len__() - 1 or index < 0:
            raise IndexError("Index must be greater than 0 and less that the number of records - 1")
        return self._parse_record(self._record_offsets[index])

    def __len__(self) -> int:
        return self._record_offsets.__len__()


#
# Command line stuff
#

_TSV_HEADER_ROW = (
    "Timestamp\tHost\tSender\tPID\tReference Process\tReference PID\tFacility\tLevel\tMessage\tOther details\n"
)


def record_to_tsv(record: AslRecord) -> str:
    # header row:
    # Timestamp, Host, Sender[PID], PID, Reference Sender, Reference PI Facility, Level, Message, Other details
    # Newlines in the message field get replaced by single spaces and tabs by 4 spaces for ease of viewing
    return "\t".join(
        (
            record.timestamp.isoformat(),
            record.host,
            record.sender,
            str(record.pid),
            str(record.refproc),
            str(record.refpid),
            record.facility,
            record.level_str,
            record.message.replace("\n", " ").replace("\t", "    "),
            "; ".join(["{0}='{1}'".format(key, record.key_value_dict[key]) for key in record.key_value_dict])
            .replace("\n", " ")
            .replace("\t", "    "),
        )
    )


def main() -> int:
    # Parse arguments
    parser = argparse.ArgumentParser(
        description=__description__,
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
        epilog=f"Version {__version__} by {__author__}",
    )

    parser.add_argument(
        "-v",
        "--version",
        action="version",
        version=__version__,
    )
    parser.add_argument(
        "-q",
        "--quiet",
        action="store_true",
        dest="quiet",
        default=False,
        help="if present, suppresses log messages being shown",
    )
    parser.add_argument(
        "-o",
        "--outputlocation",
        type=Path,  # Optional[Path]
        action="store",
        dest="output_location",
        default=None,
        help="path of output file (stdout if none defined)",
    )
    parser.add_argument(
        "-t",
        "--outputformat",
        type=str,
        choices=["tsv", "text", "s4"],
        action="store",
        dest="output_type",
        default="tsv",
        help="format of output",
    )
    parser.add_argument(
        "-a",
        "--append",
        action="store_true",
        dest="append_data",
        default=False,
        help="if present, the output will be appended to the file specified"
             " (by default, any existing files will be overwritten)",
    )
    parser.add_argument(
        "--wait-input-per-prints",
        type=int,
        dest="wait_input_per_prints",
        default=0,
        help="if greater than 0, wait for user input after this many records have been printed;"
             " default is 0 (no waiting)",
    )
    parser.add_argument(
        "inputpath",
        nargs="+",
        help="ASL files or a directory containing ASL files",
    )

    args = parser.parse_args()
    input_path: List[str] = args.inputpath
    global QUIET
    QUIET = args.quiet
    output_type = args.output_type
    output_location: Optional[Path] = args.output_location
    append_data = args.append_data
    wait_input_per_prints = args.wait_input_per_prints

    files_to_process: List[Path] = []

    for p_ in input_path:
        input_p = Path(p_)
        if not input_p.exists():
            print_err(f"Input path {input_p!r} does not exist")
        if input_p.is_dir():
            for entry in os.listdir(input_p):
                full_path = input_p / entry
                if full_path.is_file():
                    files_to_process.append(full_path)
        else:
            files_to_process.append(input_p)

    if not files_to_process:
        print_err("No valid files to process")
        sys.exit(1)

    # set up output
    if output_type not in ("tsv", "text", "s4"):
        raise ValueError(f"Output Type {output_type} is unknown")

    if output_location:
        file_mode = "a" if append_data else "w"
        out_f = open(output_location, file_mode, encoding="utf-8")
    else:
        out_f = sys.stdout

    if output_type == "tsv":
        out_f.write(_TSV_HEADER_ROW)
    elif output_type in ("text", "s4"):
        global record_field_sep
        record_field_sep = "  "
    else:
        raise ValueError(f"Output Type {output_type!r} is unknown")

    for file in files_to_process:
        if not QUIET:
            print_log(f"Processing file {file!r}")
        try:
            f = open(file, "rb")
        except IOError as e:
            print_err(f"Could not open file {file!r} ({e}): Skipping this file")
            continue

        if not QUIET:
            print_log(f"Reading ASL DB from file {file!r}")
        try:
            asl_db = AslDb(f)
        except AslDbError as e:
            print_err(f"Could not read file as ASL DB {file!r} ({e}): Skipping this file")
            f.close()
            continue

        for i, record in enumerate(asl_db):
            if not QUIET:
                print_log(f"Processing record at offset {record.offset} (ID: {record.id})")

            if output_type == "tsv":
                rec_s = record_to_tsv(record)
                out_f.write(rec_s)
                out_f.write("\n")
                out_f.flush()
            elif output_type == "text":
                rec_s = str(record)
                out_f.write(rec_s)
                out_f.write("\n")
                out_f.flush()
            elif output_type == "s4":
                rec_s = str(record)
                dt_b = rec_s.find(" ")
                ts = int(record.timestamp.timestamp() * 1000)
                rec_b = s4_event_bytes(0, dt_b, ts, rec_s)
                out_f.buffer.write(rec_b)
                out_f.flush()
            else:
                raise ValueError(f"Output Type {output_type!r} is unknown")

            if wait_input_per_prints > 0 and (i + 1) % wait_input_per_prints == 0:
                if not QUIET:
                    print_log(f"Processed {i + 1} records, waiting for user input to continue...")
                input()

        f.close()

    return 0


if __name__ == "__main__":
    main()
