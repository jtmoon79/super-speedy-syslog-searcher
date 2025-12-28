# odl_reader.py
# -*- coding: utf-8 -*-
#
# ripped from https://github.com/ydkhatri/OneDrive/blob/9ad135ecf56cd2086256cf8440b98b5eaa50c0ab/odl.py
#
# Original Author: Yogesh Khatri, @SwiftForensics
# Modified for super speedy syslog searcher by: James Thomas Moon, @jtmoon79

r"""
(c) 2021-2024 Yogesh Khatri, @SwiftForensics

___Modified for super speedy syslog searcher by James Thomas Moon, @jtmoon79___

This script will read OneDrive sync logs. These logs are produced by
OneDrive, and are stored in a binary format having the extensions
.odl .odlgz .oldsent .aold

Sometimes the ObfuscationMap stores old and new values of Keys. By
default, only the latest value is fetched. Use -k option to get all
possible values (values will be | delimited).

Newer versions of OneDrive since at least April 2022 do not use the
ObfuscationStringMap file. Data to be obfuscated is now AES encrypted
with the key stored in the file general.keystore

By default, irrelevant functions and/or those with empty parameters
are not displayed. This can be toggled with the -d option.

## Read OneDrive .ODL files

OneDrive logs are stored as binary files with extensions .odl,
.odlgz, .odlsent and .aold usually found in the profile folder of
a user under the following paths on Windows:

    \AppData\Local\Microsoft\OneDrive\logs\Business1
    \AppData\Local\Microsoft\OneDrive\logs\Personal

On macOS, they will usually be under:

    /Users/<USER>/Library/Logs/OneDrive/Business1
    /Users/<USER>/Library/Logs/OneDrive/Personal
    /Users/<USER>/Library/Logs/OneDrive/Common

Author: Yogesh Khatri, yogesh@swiftforensics.com
License: MIT
Version: 2.0, 2024-11-03

---

Ripped from https://github.com/ydkhatri/OneDrive/blob/9ad135ecf56cd2086256cf8440b98b5eaa50c0ab/odl.py
by @jtmoon79 for super speedy syslog searcher.
"""

import argparse
import base64
import datetime
import io
import json
import re
import string
import struct
import sys
import zlib
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import (
    Dict,
    List,
    LiteralString,
    Optional,
    Tuple,
    Union,
)

from colorist import Color
from construct import (
    Byte,
    Bytes,
    Const,
    ConstError,
    ConstructError,
    Struct,
)
from construct.core import (
    Int16ul,
    Int32ul,
    Int64ul,
)
from Crypto.Cipher import AES
from Crypto.Util.Padding import unpad

from . import (
    s4_event_bytes,
    __version__,
)
from .__init__ import (
    DELIMITER_EVENT_END,
    DELIMITER_EVENTS,
    DELIMITER_TS_EVENT,
)

global PRINTS
PRINTS: bool = True
"""Set to `False` to disable all print statements."""

global CHARS_ASCII_ONLY
CHARS_ASCII_ONLY: bool = True

FILE_EXTENSIONS = (".odl", ".odlgz", ".odlsent", ".aodl")
OBFUSCATION_MAP_NAME_DEFAULT = "ObfuscationStringMap.txt"
KEYSTORE_MAP_NAME_DEFAULT = "general.keystore"

if not CHARS_ASCII_ONLY:
    CONTROL_CHARS: str = "".join(map(chr, range(0, 32))) + "".join(map(chr, range(127, 160)))
    NOT_CONTROL_CHAR_RE: re.Pattern = re.compile(f'[^{CONTROL_CHARS}]' + '{4,}')
else:
    PRINTABLE_CHARS_FOR_RE: bytes = (
        string.printable.replace("\\", "\\\\").replace("[", "\\[").replace("]", "\\]").encode()
    )
    ASCII_CHARS_RE: re.Pattern = re.compile(b"[{" + PRINTABLE_CHARS_FOR_RE + b"}]" + b"{4,}")


CDEF_V2 = Struct(
    "signature" / Const(b"\xcc\xdd\xee\xff"),  # / Int32ul, # CCDDEEFF
    "unknown_flag" / Int32ul,  # pyright: ignore[reportOperatorIssue]
    "timestamp" / Int64ul,  # pyright: ignore[reportOperatorIssue]
    "unk1" / Int32ul,  # pyright: ignore[reportOperatorIssue]
    "unk2" / Int32ul,  # pyright: ignore[reportOperatorIssue]
    "unknown" / Byte[20],  # pyright: ignore[reportOperatorIssue, reportIndexIssue]
    "one" / Int32ul,  # 1  # pyright: ignore[reportOperatorIssue]
    "data_len" / Int32ul,  # pyright: ignore[reportOperatorIssue]
    "reserved" / Int32ul,  # 0  # pyright: ignore[reportOperatorIssue]
    # followed by Data
)
"""ODL Header Version 2"""

CDEF_V3 = Struct(
    "signature" / Const(b"\xcc\xdd\xee\xff"),  # / Int32ul, # CCDDEEFF
    "context_data_len" / Int16ul,  # pyright: ignore[reportOperatorIssue]
    "unknown_flag" / Int16ul,  # pyright: ignore[reportOperatorIssue]
    "timestamp" / Int64ul,  # pyright: ignore[reportOperatorIssue]
    "unk1" / Int32ul,  # pyright: ignore[reportOperatorIssue]
    "unk2" / Int32ul,  # pyright: ignore[reportOperatorIssue]
    "data_len" / Int32ul,  # pyright: ignore[reportOperatorIssue]
    "reserved" / Int32ul,  # 0  # pyright: ignore[reportOperatorIssue]
    # followed by Data
)
"""ODL Header Version 3"""

Odl_header = Struct(
    "signature" / Bytes(8),  # 'EBFGONED'
    "odl_version" / Int32ul,  # pyright: ignore[reportOperatorIssue]
    "unknown_2" / Int32ul,  # pyright: ignore[reportOperatorIssue]
    "unknown_3" / Int64ul,  # pyright: ignore[reportOperatorIssue]
    "unknown_4" / Int32ul,  # pyright: ignore[reportOperatorIssue]
    "one_drive_version" / Byte[0x40],  # pyright: ignore[reportOperatorIssue, reportIndexIssue]
    "windows_version" / Byte[0x40],  # pyright: ignore[reportOperatorIssue, reportIndexIssue]
    "reserved" / Byte[0x64],  # pyright: ignore[reportOperatorIssue, reportIndexIssue]
)
"""ODL Header Version 1"""

global no_color
no_color: bool = False


def print_color(color: Union[str, LiteralString], *args, **kwargs):
    """
    Print to stderr with color.
    Wrapper for `print` to `stderr` and with color if enabled via `--no-color` option.
    `args` and `kwargs` are passed to `print`.
    """
    global no_color
    if not no_color:
        print(f"{color}", end="", file=sys.stderr)
    # have to save `end` and print it later otherwise Color.OFF has no effect.
    end_val = "\n"
    if "end" in kwargs:
        end_val = kwargs["end"]
        del kwargs["end"]
    print(*args, end="", **kwargs, file=sys.stderr)
    if not no_color:
        print(f"{Color.OFF}", end=end_val, file=sys.stderr)
    else:
        print(end=end_val, file=sys.stderr)


def printe(*args, **kwargs):
    """
    Print Error.
    """
    print_color(Color.RED, *args, **kwargs)  # pyright: ignore[reportArgumentType]


def printw(*args, **kwargs):
    """
    Print Warning.
    """
    print_color(Color.YELLOW, *args, **kwargs)  # pyright: ignore[reportArgumentType]


def printi(*args, **kwargs):
    """
    Print Info.
    """
    print_color(Color.GREEN, *args, **kwargs)  # pyright: ignore[reportArgumentType]


def printd(*args, **kwargs):
    """
    Print Debug.
    """
    # Dark Gray
    print_color("\033[90m", *args, **kwargs)  # pyright: ignore[reportArgumentType]


def convert_epochms_datetime(unix_time_ms: Union[str, int, float]) -> Optional[datetime.datetime]:
    """
    Returns datetime object or None if error

    :param unix_time_ms: Unix epoch time in milliseconds
    :type unix_time_ms: Union[str, int, float]
    :return: the new `datetime` object or `None` if error
    :rtype: datetime | None
    """
    if not unix_time_ms:
        return None

    try:
        unix_time_ms = float(unix_time_ms)
        dt = datetime.datetime.fromtimestamp(unix_time_ms / 1000.0, tz=datetime.timezone.utc)
        dt = dt.replace(microsecond=int((unix_time_ms % 1000) * 1000))
        return dt
    except Exception:
        pass

    return None


def read_string(data: bytes) -> Tuple[int, str]:
    """read string, return tuple (bytes_consumed, string)"""
    if len(data) >= 4:
        str_len = struct.unpack("<I", data[0:4])[0]
        if str_len:
            if str_len > len(data):
                if PRINTS:
                    printe(f"ERROR: read_string() bad str_len {str_len}")
            else:
                return (4 + str_len, data[4: 4 + str_len].decode("utf8", "ignore"))

    return (4, "")


def guess_encoding(obfuscation_map_path: Path) -> str:
    """
    Returns either 'UTF8' or 'UTF16LE' after checking the file
    """

    encoding = "utf-16le"  # on windows this is the default
    with open(obfuscation_map_path, "rb") as f:
        data = f.read(4)
        if len(data) == 4:
            if data[1] == 0 and data[3] == 0 and data[0] != 0 and data[2] != 0:
                pass  # confirmed utf-16le
            else:
                encoding = "utf8"

    return encoding


# UnObfuscation code
global key
global utf_type
key: Union[bytes, str] = ""
utf_type: str = "utf16"
"""passed to `decoding` function as `encoding` parameter"""


def decrypt(cipher_text: str) -> Optional[str]:
    """cipher_text is expected to be base64 encoded"""
    global key
    global utf_type

    cipher_text_fallback = cipher_text if cipher_text.isprintable() else None

    if not key:
        # key not available
        if PRINTS:
            printw(f"WARNING: Unobfuscation key not available; return cipher_text={cipher_text_fallback!r}")
        return cipher_text_fallback

    if len(cipher_text) < 22:
        # invalid or it was not encrypted
        # return as-is because most often it was just not encrypted
        if PRINTS:
            printw(f"WARNING: cipher_text length {len(cipher_text)} too short, "
                   f"must be less than 22; return cipher_text={cipher_text_fallback!r}")
        return cipher_text_fallback
    # add proper base64 padding
    remainder = len(cipher_text) % 4
    if remainder == 1:
        # invalid b64 or it was not encrypted
        if PRINTS:
            printe(f"ERROR: invalid cipher_text length {len(cipher_text)}; return cipher_text={cipher_text_fallback!r}")
        return cipher_text_fallback
    elif remainder in (2, 3):
        cipher_text += "=" * (4 - remainder)
    try:
        cipher_text = cipher_text.replace("_", "/").replace("-", "+")
        cipher_bytes = base64.b64decode(cipher_text)
    except Exception as ex:
        if PRINTS:
            printe(f"ERROR: base64.b64decode failed: {ex}; return cipher_text={cipher_text_fallback!r}")
        return cipher_text_fallback

    if len(cipher_bytes) % 16 != 0:
        if PRINTS:
            printe(f"ERROR: invalid cipher bytes length {len(cipher_bytes)}; return cipher_text={cipher_text_fallback!r}")
        return cipher_text_fallback

    try:
        cipher = AES.new(key, AES.MODE_CBC, iv=b"\0" * 16)  # pyright: ignore[reportCallIssue, reportArgumentType]
        raw = cipher.decrypt(cipher_bytes)
    except ValueError as ex:
        if PRINTS:
            printe(f"ERROR while decrypting data {ex}; return cipher_text as-is; cipher_text={cipher_text_fallback!r}")
        return cipher_text_fallback
    try:
        plain_text_bytes = unpad(raw, 16)
    except ValueError as ex:
        if PRINTS:
            printe(f"ERROR: unpad failed: {ex}; return None; {raw=}; return cipher_text={cipher_text_fallback!r}")
        return cipher_text_fallback
    try:
        plain_text_str = plain_text_bytes.decode(encoding=utf_type)
    except ValueError as ex:
        if PRINTS:
            printe(f"ERROR: decode failed: {ex}; return cipher_text={cipher_text_fallback!r}")
        return cipher_text_fallback

    if PRINTS:
        printi(f"INFO: Decrypted {len(cipher_text)} bytes of cipher_text to {len(plain_text_str)} bytes of plain_text_str")

    return plain_text_str


def read_keystore(keystore_path: Path):
    """
    Set global key and utf_type variables

    :param keystore_path: Path to the keystore file
    :type keystore_path: Path
    """
    global key
    global utf_type
    encoding = guess_encoding(keystore_path)
    with open(keystore_path, "r", encoding=encoding) as file:
        try:
            j = json.load(file)
            key = j[0]["Key"]
            version = j[0]["Version"]
            utf_type = "utf16"
            # XXX: I'm still not sure if this is the correct way to detect utf32.
            #      The original code had:
            #            key.endswith('\\u0000\\u0000')
            #      It looks like the author meant to look for four null bytes
            #      at the end. But that expression above checks for two null bytes.
            #      But if it were UTF-32 then we should expect four null bytes.
            #      So it's possible the author's original code had a bug here
            #      and I'm continuing that bug.
            if key.endswith("\x00\x00"):  # pyright: ignore[reportArgumentType]
                utf_type = "utf32"
            if PRINTS:
                printi(f"INFO: Recovered Unobfuscation key {key=}, {version=}, {utf_type=}")
            key = base64.b64decode(key)
            if version != 1:
                if PRINTS:
                    printw(f"WARNING: Key version {version} is unsupported. This may fail.")
        except ValueError as ex:
            if PRINTS:
                printe(f"ERROR: reading keystore json: {ex}")


MapObfuscationType = Dict[str, str]


def read_obfuscation_map(obfuscation_map_path: Path, store_all_key_values: bool) -> MapObfuscationType:
    map: MapObfuscationType = dict()
    repeated_items_found = False
    encoding = guess_encoding(obfuscation_map_path)
    last_val = ""
    last_key = ""
    with open(obfuscation_map_path, "r", encoding=encoding) as f:
        for line in f.readlines():
            line = line.rstrip("\n")
            terms = line.split("\t")
            if len(terms) == 2:
                if terms[0] in map:
                    # repeated item found!
                    repeated_items_found = True
                    if not store_all_key_values:
                        # newer items are on top, skip older items found below.
                        continue
                    old_val = map[terms[0]]
                    new_val = f"{old_val}|{terms[1]}"
                    map[terms[0]] = new_val
                    last_key = terms[0]
                    last_val = new_val
                else:
                    map[terms[0]] = terms[1]
                    last_key = terms[0]
                    last_val = terms[1]
            else:
                if terms[0] in map:
                    if not store_all_key_values:
                        continue
                last_val += "\n" + line
                map[last_key] = last_val
                # printe('Error? ' + str(terms))
    if repeated_items_found:
        if PRINTS:
            printw(f"WARNING: Multiple instances of some keys were found in the ObfuscationMap at {obfuscation_map_path}")

    return map


def tokenized_replace(string: str, map: MapObfuscationType) -> str:
    output = ""
    tokens = ":\\.@%#&*|{}!?<>;:~()//\"'"
    parts: List[Tuple[str, int]] = list()  # [ ('word', 1), (':', 0), ..] word=1, token=0
    last_word = ""
    last_token = ""
    for char in string:
        if char in tokens:
            if last_word:
                parts.append((last_word, 1))
                last_word = ""
            if last_token:
                last_token += char
            else:
                last_token = char
        else:
            if last_token:
                parts.append((last_token, 0))
                last_token = ""
            if last_word:
                last_word += char
            else:
                last_word = char

    if last_token:
        parts.append((last_token, 0))
    if last_word:
        parts.append((last_word, 1))

    # now join all parts replacing the words
    for part in parts:
        if part[1] == 0:  # token
            output += part[0]
        else:  # word
            word = part[0]
            decrypted_word = decrypt(word)
            if decrypted_word:
                output += decrypted_word
            elif word in map:
                output += map[word]
            else:
                output += word

    return output


def extract_strings(data, map: MapObfuscationType, unobfuscate: bool = True) -> List[str]:
    extracted = []
    global CHARS_ASCII_ONLY
    if not CHARS_ASCII_ONLY:
        # This gets all unicode chars, can include lot of garbage
        # if you only care about English, will miss out other languages
        global NOT_CONTROL_CHAR_RE
        find_iter_ = NOT_CONTROL_CHAR_RE.finditer(data)
    else:
        # Matches ONLY Ascii (old behavior), good if you only care about English
        global ASCII_CHARS_RE
        find_iter_ = ASCII_CHARS_RE.finditer(data)
    for match in find_iter_:
        text = match.group()
        if match.start() >= 4:
            match_len = match.end() - match.start()
            y = data[match.start() - 4: match.start()]
            stored_len = struct.unpack("<I", y)[0]
            if match_len - stored_len <= 5:
                x = text[0:stored_len].decode("utf8", "ignore")
                x = x.rstrip("\n").rstrip("\r")
                x = x.replace("\r", "").replace("\n", " ")
                if unobfuscate:
                    x = tokenized_replace(x, map)
                extracted.append(x)
            else:
                if PRINTS:
                    printe("ERROR: invalid match - not text , match_len - stored_len: "
                           f"{match_len - stored_len}, text: {text}")

    return extracted


InFileType = Union[io.BufferedReader, io.BytesIO]
OutFileType = Union[io.BufferedWriter, io.StringIO, io.TextIOWrapper]


@dataclass
class OdlRow:
    Datetime: Optional[datetime.datetime]
    Timestamp: float
    Code_File: str
    Function: str
    Params_Decoded: Optional[str]

    def dict(self):
        return {k: str(v) for k, v in asdict(self).items()}


global OdlRowPrintedCount
OdlRowPrintedCount: int = 0

global OdlRowNotPrintedCount
OdlRowNotPrintedCount: int = 0

global OdlDataRejectedCount
OdlDataRejectedCount: int = 0


def write_odl_row(
    odl: OdlRow,
    show_all_data: bool,
    human_readable: bool,
) -> None:
    """
    Print or write the `OdlRow` to stdout in s4 Event format.
    If `show_all_data` is `False` then do not print some common less interesting rows.
    """
    reject = False

    if not show_all_data:
        # filter out irrelevant
        if odl.Function == "Find" and odl.Code_File == "cache.cpp":
            # cache.cpp Find function provides no value
            # as search term or result is not present
            reject = True
        elif odl.Function == "RecordCallTimeTaken" and odl.Code_File == "AclHelper.cpp":
            reject = True
        elif odl.Function == "UpdateSyncStatusText" and odl.Code_File == "ActivityCenterHeaderModel.cpp":
            reject = True
        elif odl.Function == "FireEvent" and odl.Code_File == "EventMachine.cpp":
            reject = True
        elif odl.Code_File in (
            "LogUploader2.cpp",
            "LogUploader.cpp",
            "ServerRefreshState.cpp",
            "SyncTelemetry.cpp",
        ):
            reject = True
        elif not odl.Params_Decoded:
            reject = True

        if reject:
            global OdlRowNotPrintedCount
            OdlRowNotPrintedCount += 1
            return

    global OdlRowPrintedCount

    if human_readable:
        # human readable output
        if odl.Params_Decoded:
            print(
                f"Timestamp: {odl.Timestamp} ({odl.Datetime}); "
                f"Code_File: {odl.Code_File}; "
                f"Function: {odl.Function}; "
                f"Params_Decoded: {odl.Params_Decoded}"
            )
        else:
            print(
                f"Timestamp: {odl.Timestamp} ({odl.Datetime}); "
                f"Code_File: {odl.Code_File}; "
                f"Function: {odl.Function}"
            )
        OdlRowPrintedCount += 1
        return

    # else write to stdout in s4 Event format
    timestamp_s = str(int(odl.Timestamp))
    if odl.Params_Decoded:
        event_s = timestamp_s + " " + odl.Code_File + ":" + odl.Function + "; " + odl.Params_Decoded
    else:
        event_s = timestamp_s + " " + odl.Code_File + ":" + odl.Function + ";"

    event_bytes = s4_event_bytes(
        0,
        len(timestamp_s),
        int(odl.Timestamp),
        event_s,
    )
    sys.stdout.buffer.write(event_bytes)
    sys.stdout.buffer.flush()
    OdlRowPrintedCount += 1


def process_odl_v2(
    path: Path,
    map: MapObfuscationType,
    show_all_data: bool,
    f: InFileType,
    wait_input_per_prints: int,
    human_readable: bool,
) -> bool:
    size = 56
    """size of an ODL CDEF V2 header structure"""

    header = f.read(size)
    file_pos = f.tell()

    global OdlDataRejectedCount

    i = 1
    try:
        while header and len(header) == size:
            odl: OdlRow = OdlRow(None, 0.0, "", "", "")
            # let `parse` raise if it fails, will abandon further processing
            header = CDEF_V2.parse(header)
            if header is None:
                if PRINTS:
                    printe("ERROR: CDEF_V2.parse() returned None")
                # TODO: why not continue?
                OdlDataRejectedCount += 1
                break
            odl.Timestamp = float(header.timestamp)
            timestamp_dt = convert_epochms_datetime(odl.Timestamp)
            if not timestamp_dt:
                if PRINTS:
                    printe(f"ERROR: convert_epochms_datetime({header.timestamp!r}) returned None")
                OdlDataRejectedCount += 1
                # BUG: this will loop forever, right?
                continue
            odl.Datetime = timestamp_dt
            if header.data_len <= 4:
                if PRINTS:
                    printw(f"WARNING: Row {i} too short {header.data_len} bytes, skip")
                OdlDataRejectedCount += 1
                break
            header_data_len = header.data_len
            data = f.read(header_data_len)
            data_pos, code_file_name = read_string(data)
            _flags = struct.unpack("<I", data[data_pos: data_pos + 4])[0]  # noqa: F841
            data_pos += 4
            temp_pos, code_function_name = read_string(data[data_pos:])
            data_pos += temp_pos
            strings_extracted = []
            if data_pos < header_data_len:
                params = data[data_pos:]
                try:
                    strings_extracted = extract_strings(params, map)
                except Exception as ex:
                    if PRINTS:
                        printe(f"ERROR: extract_strings: {ex}")
            odl.Code_File = code_file_name
            odl.Function = code_function_name
            if strings_extracted:
                odl.Params_Decoded = " ".join(strings_extracted)
            else:
                odl.Params_Decoded = None

            write_odl_row(odl, show_all_data, human_readable)

            # if requested, wait for input to continue
            if wait_input_per_prints > 0 and i % wait_input_per_prints == 0:
                input()

            # advance to next header
            file_pos += header_data_len
            header = f.read(size)
            file_pos += size
            i += 1
    except (ConstructError, ConstError) as ex:
        if PRINTS:
            printe(f"ERROR: Exception reading structure: {ex} at filepos {file_pos} i={i}")
        return False

    return True


def process_odl_v3(
    _,
    map: MapObfuscationType,
    show_all_data: bool,
    f: InFileType,
    wait_input_per_prints: int,
    human_readable: bool,
) -> bool:
    size = 32
    """size of an ODL CDEF V3 header structure"""

    global OdlDataRejectedCount

    header = f.read(size)
    file_pos = f.tell()

    i = 1
    try:
        while header and len(header) == size:
            odl: OdlRow = OdlRow(None, 0.0, "", "", "")
            header = CDEF_V3.parse(header)
            if header is None:
                if PRINTS:
                    printe("ERROR: CDEF_V3.parse(header) returned None")
                OdlDataRejectedCount += 1
                # TODO: why not continue?
                break
            odl.Timestamp = float(header.timestamp)
            timestamp_dt = convert_epochms_datetime(odl.Timestamp)
            if not timestamp_dt:
                if PRINTS:
                    printe(f"ERROR: convert_epochms_datetime({header.timestamp!r}) returned None")
                OdlDataRejectedCount += 1
                # BUG: this might loop forever, right?
                continue
            odl.Datetime = timestamp_dt
            if header.data_len <= 4:
                if PRINTS:
                    printw(f"WARNING: Row {i} too short {header.data_len} bytes, skip")
                OdlDataRejectedCount += 1
                break
            header_context_data_len = header.context_data_len
            if header_context_data_len == 0:
                # seek past the guid and other unknown fields
                f.seek(24, io.SEEK_CUR)
                file_pos += 24
                header_data_len = header.data_len - 24
            else:
                _context_data = f.read(header_context_data_len)  # noqa: F841
                file_pos += header_context_data_len
                header_data_len = header.data_len - header_context_data_len
            data = f.read(header_data_len)
            file_pos += header_data_len
            data_pos, code_file_name = read_string(data)
            _flags = struct.unpack("<I", data[data_pos: data_pos + 4])[0]  # noqa: F841
            data_pos += 4
            temp_pos, code_function_name = read_string(data[data_pos:])
            data_pos += temp_pos
            strings_extracted = []
            if data_pos < header_data_len:
                params = data[data_pos:]
                try:
                    strings_extracted = extract_strings(params, map)
                except Exception as ex:
                    if PRINTS:
                        printe(f"ERROR: extract_strings {ex}")
            odl.Code_File = code_file_name
            odl.Function = code_function_name
            if strings_extracted:
                odl.Params_Decoded = " ".join(strings_extracted)
            else:
                odl.Params_Decoded = ""

            write_odl_row(odl, show_all_data, human_readable)

            # if requested, wait for input to continue
            if wait_input_per_prints > 0 and i % wait_input_per_prints == 0:
                input()

            # advance to next header
            header = f.read(size)
            file_pos += size
            i += 1
    except (ConstructError, ConstError) as ex:
        if PRINTS:
            printe(f"ERROR: Exception reading structure: {ex} at filepos {file_pos} i={i}")
        return False

    return True


def process_odl(
    path: Path,
    map: MapObfuscationType,
    show_all_data: bool,
    wait_input_per_prints: int,
    human_readable: bool,
) -> Tuple[bool, int]:
    """
    return (bool success, int odl_version)

    odl_version value 0 means the ODL failed to process
    odl_version value 2 means ODL version 2
    odl_version value 3 means ODL version 3
    """

    odl_version = 2  # default
    ret = False

    file = open(path, "rb")

    file_header = file.read(0x100)
    odl_header = Odl_header.parse(file_header)
    if odl_header is None:
        if PRINTS:
            printe("ERROR: Odl_header.parse() returned None")
        return False, 0
    odl_version = odl_header.odl_version
    if odl_version not in (2, 3):
        if PRINTS:
            printe(f"ERROR: Unknown odl_version {odl_version}")
        return False, 0
    if PRINTS:
        printd(f"DEBUG: ODL version {odl_version} found in '{path}'")
    header = odl_header.signature
    if header[0:8] == b"EBFGONED":  # ODL header
        file.seek(0x100)
        header = file.read(8)
        file_pos = 0x108
    else:
        file_pos = 8

    # Now either we have the gzip header here
    # or the CDEF_xx header (compressed or uncompressed handles both)
    if header[0:4] == b"\x1f\x8b\x08\x00":
        # gzip header found, decompress using zlib
        if PRINTS:
            printi(f"INFO: zlib compressed data found in '{path}'")
        try:
            file.seek(file_pos - 8)
            all_data = file.read()
            z = zlib.decompressobj(31)
            file_data = z.decompress(all_data)
            if PRINTS:
                printi(f"INFO: zlib decompressed {len(file_data)} bytes")
        except (zlib.error, OSError) as ex:
            if PRINTS:
                printe(f"ERROR: zlib decompression error for '{path}'; {ex}")
            return False, 0
        file.close()
        file = io.BytesIO(file_data)
        header = file.read(8)

    if header[0:4] != b"\xcc\xdd\xee\xff":
        # CDEF_Vx header not found
        if PRINTS:
            printe("ERROR: bad CDEF_Vx header; expected 0xCCDDEEFF")
        return False, 0
    else:
        file.seek(-8, io.SEEK_CUR)
        file_pos -= 8
        if odl_version == 2:
            ret = process_odl_v2(path, map, show_all_data, file, wait_input_per_prints, human_readable)
        elif odl_version == 3:
            ret = process_odl_v3(path, map, show_all_data, file, wait_input_per_prints, human_readable)
        else:
            # should not reach here
            raise ValueError(f"Unsupported odl_version {odl_version}")

    file.close()

    return ret, odl_version


def main() -> int:
    parser = argparse.ArgumentParser(
        description="OneDrive Log (ODL) reader, modified for super speedy syslog searcher",
        epilog=__doc__,
        formatter_class=argparse.RawTextHelpFormatter,
    )
    parser.add_argument(
        "odl_file",
        type=Path,
        action="store",
        help="Path to ODL file",
    )
    parser.add_argument(
        "-s",
        "--obfuscationstringmap_path",
        type=Path,
        default=None,
        help=f"Path to {OBFUSCATION_MAP_NAME_DEFAULT} (if not in odl_folder)",
    )
    parser.add_argument(
        "-k",
        "--all_key_values",
        action="store_true",
        help="For repeated keys in ObfuscationMap, get all values",
    )
    parser.add_argument(
        "-d",
        "--all_data",
        action="store_true",
        help="Show all data",
    )
    parser.add_argument(
        "--wait-input-per-prints",
        default=0,
        type=int,
        help="Wait for user input after each print to stderr",
    )
    parser.add_argument(
        "--no-color",
        action="store_true",
        help="Print with color disabled",
    )
    parser.add_argument(
        "--no-prints",
        action="store_true",
        help="Disable all informational prints to stderr",
    )
    parser.add_argument(
        "--human-readable",
        action="store_true",
        help="Print output in a human-readable format",
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")

    args = parser.parse_args()

    odl_file = args.odl_file
    wait_input_per_prints: int = args.wait_input_per_prints
    global no_color
    no_color = args.no_color
    global PRINTS
    PRINTS = not args.no_prints
    human_readable = args.human_readable

    if PRINTS:
        printd(f"DEBUG: wait_input_per_prints={wait_input_per_prints}, no_color={no_color}, "
               f"PRINTS={PRINTS}, human_readable={human_readable}")


    if not odl_file.is_file():
        printe(f"ERROR: path '{odl_file}' is not a file")
        return 1

    odl_folder = odl_file.parent

    # look for obfuscation map, try to read it

    paths_tried = []
    obfuscation_map_path: Optional[Path] = args.obfuscationstringmap_path
    if not obfuscation_map_path:
        for map_name in (OBFUSCATION_MAP_NAME_DEFAULT, KEYSTORE_MAP_NAME_DEFAULT):
            possible_path = Path(odl_folder) / map_name
            paths_tried.append(str(possible_path))
            if possible_path.exists():
                obfuscation_map_path = possible_path
                break

    map: MapObfuscationType = {}
    if obfuscation_map_path is None:
        if PRINTS:
            printw("WARNING: Obfuscation map path could not be determined. "
                   "It can provided with --obfuscationstringmap_path")
            printw("         Tried paths:")
            for p in paths_tried:
                printw(f"           {p}")
            printw("         Use empty obfuscation map.")
    elif not obfuscation_map_path.exists():
        if PRINTS:
            printw(f"WARNING: file {obfuscation_map_path.name!r} not found in '{obfuscation_map_path.parent}'.\n"
                    "         Use empty obfuscation map.")
    else:
        map: MapObfuscationType = read_obfuscation_map(obfuscation_map_path, args.all_key_values)
        if PRINTS:
            printd(f"DEBUG: Read {len(map)} items from obfuscation map at '{obfuscation_map_path}'")

    # look for key store, try to read it

    keystore_path = Path(odl_folder) / "general.keystore"
    if not keystore_path.exists():
        # Try new path
        keystore_path = Path(odl_folder) / "EncryptionKeyStoreCopy" / "general.keystore"
        if not keystore_path.exists():
            if PRINTS:
                printw(f'WARNING: "general.keystore" not found in "{odl_folder}". Strings will not be decoded')
        else:
            read_keystore(keystore_path)
    else:
        read_keystore(keystore_path)

    # process the odl file

    path = Path(odl_file)
    if not path.exists():
        printe(f"ERROR: File does not exist {path}")
        return 1
    if path.stat().st_size == 0:
        printw(f"WARNING: File is empty {path}")
        return 0

    ret, odl_version = process_odl(path, map, args.all_data, wait_input_per_prints, human_readable)
    if PRINTS and human_readable:
        global OdlRowPrintedCount
        global OdlRowNotPrintedCount
        global OdlDataRejectedCount
        printi(
            f"INFO: ODL Version {str(odl_version) if odl_version else 'Unknown'}; "
            f"printed {OdlRowPrintedCount} rows, "
            f"skipped {OdlRowNotPrintedCount} rows, "
            f"failed to process {OdlDataRejectedCount} rows"
        )

    return 0 if ret else 1


if __name__ == "__main__":
    sys.exit(main())
