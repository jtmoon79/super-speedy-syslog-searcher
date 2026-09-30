#!/usr/bin/env python3
"""
Compare `s4` output for `.etl` files against `tracerpt.exe` XML dumps.

For each `FILE.etl` found under DIR that has a sibling `FILE.etl.xml`
(created with `tracerpt.exe FILE.etl -o FILE.etl.xml -of XML`), run
`s4 --color never -t +00:00 FILE.etl` and compare:

  1. event counts
  2. the per-event tuple (timestamp, provider GUID, EventID, PID, TID)
     as a multiset
  3. for TraceLogging events, the `Data Name="..."` values against the
     `name=value` fields printed by `s4` (string and integer fields)

`tracerpt` writes `RawTime` in the session clock; it is converted to
FILETIME using the `EventTrace` header event `StartTime`, `PerfFreq`, and
`ReservedFlags` (1 QPC, 2 SystemTime, 3 CPU cycle) using the same formula
as `EtlParser`.

Only the Python standard library is used.
"""

import argparse
import datetime
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from collections import Counter
from pathlib import Path

NS_EVENT = "{http://schemas.microsoft.com/win/2004/08/events/event}"
NS_TRACE = "{http://schemas.microsoft.com/win/2004/08/events/trace}"
FILETIME_UNIX_EPOCH = 116444736000000000
# tracerpt reports kernel events under the NT Kernel Logger provider GUID;
# s4 reports the kernel group class GUID (`ExtendedTracingInfo/EventGuid`)
SYSTEM_TRACE_CONTROL_GUID = "{9e814aad-3204-11d2-9a82-006008a86939}"
NO_PID = 0xFFFFFFFF
RE_FILETIME_Z = re.compile(r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{7})\d{2}Z$")
RE_LINE = re.compile(
    r"^(?P<ts>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{7}[+-]\d{2}:\d{2}) "
    r"Provider=(?P<guid>\{[0-9a-f-]{36}\})"
    r"(?P<rest>.*)$"
)
RE_KV = re.compile(r' (?P<k>[A-Za-z_][\w.]*)=(?P<v>"(?:[^"\\]|\\.)*"|\S+)')


def filetime_to_iso(ft: int) -> str:
    """FILETIME (100ns since 1601) -> `YYYY-mm-ddTHH:MM:SS.fffffff+00:00`."""
    us, frac100 = divmod(ft - FILETIME_UNIX_EPOCH, 10)
    dt = datetime.datetime(1970, 1, 1, tzinfo=datetime.timezone.utc) + datetime.timedelta(
        microseconds=us
    )
    return dt.strftime("%Y-%m-%dT%H:%M:%S.%f") + str(frac100) + "+00:00"


def read_xml_text(path: Path) -> str:
    data = path.read_bytes()
    if data[:2] in (b"\xff\xfe", b"\xfe\xff"):
        return data.decode("utf-16")
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return data.decode("utf-16")


def parse_tracerpt(path: Path):
    """Return (header_info, events) where events is a list of dicts."""
    text = read_xml_text(path)
    # tracerpt may omit the XML declaration and emit stray NULs
    text = text.replace("\x00", "")
    root = ET.fromstring(text)
    events = []
    header = None
    for ev in root.iter(f"{NS_EVENT}Event"):
        sysel = ev.find(f"{NS_EVENT}System")
        if sysel is None:
            continue
        prov = sysel.find(f"{NS_EVENT}Provider")
        guid = (prov.get("Guid") or "").lower() if prov is not None else ""
        pname = prov.get("Name") if prov is not None else None
        if pname == "Unknown":
            pname = None
        event_guid = ev.findtext(f"{NS_TRACE}ExtendedTracingInfo/{NS_TRACE}EventGuid")
        if event_guid and (guid == "" or guid == SYSTEM_TRACE_CONTROL_GUID):
            guid = event_guid.lower()
        raw = int(sysel.find(f"{NS_EVENT}TimeCreated").get("RawTime"))
        exe = sysel.find(f"{NS_EVENT}Execution")
        pid = int(exe.get("ProcessID")) if exe is not None else None
        tid = int(exe.get("ThreadID")) if exe is not None else None
        if pid == NO_PID:
            pid = None
        if tid == NO_PID:
            tid = None
        eid = int(sysel.findtext(f"{NS_EVENT}EventID") or 0)
        data = {}
        ed = ev.find(f"{NS_EVENT}EventData")
        if ed is not None:
            for d in ed.findall(f"{NS_EVENT}Data"):
                data[d.get("Name")] = (d.text or "").strip()
        rendering = ev.find(f"{NS_EVENT}RenderingInfo")
        event_name = None
        if rendering is not None:
            for child in rendering:
                if child.tag.endswith("EventName"):
                    event_name = child.text
        e = dict(
            raw=raw, guid=guid, pname=pname, eid=eid, pid=pid, tid=tid, data=data, event_name=event_name
        )
        events.append(e)
        if header is None and event_name == "EventTrace" and "StartTime" in data:
            header = dict(
                start_time=int(data["StartTime"]),
                perf_freq=int(data["PerfFreq"]),
                flags=int(data["ReservedFlags"], 16),
                cpu_speed=int(data.get("CPUSpeed", "0")),
                raw=raw,
            )
    return header, events


def raw_to_filetime(raw: int, header) -> int:
    if header is None or header["flags"] == 2:
        return raw
    if header["flags"] == 3:
        freq = header["cpu_speed"] * 1_000_000
    else:
        freq = header["perf_freq"]
    if freq == 0:
        return raw
    delta = raw - header["raw"]
    return header["start_time"] + (delta * 10_000_000) // freq


def parse_s4(s4: str, etl: Path):
    proc = subprocess.run(
        [s4, "--color", "never", "-t", "+00:00", str(etl)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if proc.returncode != 0:
        print(f"  s4 exit {proc.returncode}: {proc.stderr.decode(errors='replace').strip()[:400]}")
    events = []
    for line in proc.stdout.decode("utf-8", errors="replace").splitlines():
        m = RE_LINE.match(line)
        if not m:
            print(f"  s4 unparseable line: {line[:120]!r}")
            continue
        kv = {}
        for km in RE_KV.finditer(m.group("rest")):
            v = km.group("v")
            if v.startswith('"'):
                v = bytes(v[1:-1], "utf-8").decode("unicode_escape")
            kv[km.group("k")] = v
        events.append(dict(ts=m.group("ts"), guid=m.group("guid").lower(), kv=kv))
    return events


def norm_int(s: str):
    s = s.strip()
    try:
        return int(s, 0)
    except ValueError:
        return None


def values_equal(tval: str, sval: str) -> bool:
    # the XML parser normalizes CRLF to LF; s4 prints the raw string
    sval = sval.replace("\r\n", "\n")
    if sval == tval or sval.strip() == tval.strip():
        return True
    ti, si = norm_int(tval), norm_int(sval)
    if ti is not None and si is not None and ti == si:
        return True
    # tracerpt renders BOOLEAN as 1/0
    if (ti, sval) in ((1, "true"), (0, "false")):
        return True
    # tracerpt renders FILETIME as `...T..:..:..fffffffffZ` (nine digits)
    m = RE_FILETIME_Z.match(tval.strip())
    if m and sval == m.group(1) + "+00:00":
        return True
    return False


def compare_file(s4: str, etl: Path, xml: Path, verbose: bool) -> bool:
    header, t_events = parse_tracerpt(xml)
    s_events = parse_s4(s4, etl)
    ok = True

    if len(t_events) != len(s_events):
        print(f"  COUNT mismatch: tracerpt={len(t_events)} s4={len(s_events)}")
        ok = False
    else:
        print(f"  count {len(t_events)}")

    def t_key(e):
        return (
            filetime_to_iso(raw_to_filetime(e["raw"], header)),
            e["guid"],
            e["eid"],
            e["pid"],
            e["tid"],
        )

    def s_key(e):
        kv = e["kv"]
        return (
            e["ts"],
            e["guid"],
            int(kv.get("EventId", "0")),
            int(kv["PID"]) if "PID" in kv else None,
            int(kv["TID"]) if "TID" in kv else None,
        )

    tc = Counter(t_key(e) for e in t_events)
    sc = Counter(s_key(e) for e in s_events)
    only_t = tc - sc
    only_s = sc - tc
    if only_t or only_s:
        ok = False
        print(f"  KEY mismatch: only in tracerpt={sum(only_t.values())} only in s4={sum(only_s.values())}")
        if verbose:
            for k in list(only_t)[:5]:
                print(f"    tracerpt: {k}")
            for k in list(only_s)[:5]:
                print(f"    s4      : {k}")
    else:
        print("  keys match (ts, guid, eventid, pid, tid)")

    # payload field comparison for TraceLogging events; match by key order
    t_by_key = {}
    for e in t_events:
        t_by_key.setdefault(t_key(e), []).append(e)
    checked = 0
    mism = 0
    for se in s_events:
        k = s_key(se)
        cands = t_by_key.get(k)
        if not cands:
            continue
        te = cands.pop(0)
        if te["pname"] is None:
            # not TraceLogging (tracerpt shows Name only when self-described or manifest known)
            continue
        for name, tval in te["data"].items():
            if name not in se["kv"]:
                mism += 1
                if verbose:
                    print(f"    field missing in s4: {k[0]} {name}={tval!r}")
                continue
            checked += 1
            sval = se["kv"][name]
            if values_equal(tval, sval):
                continue
            mism += 1
            if verbose:
                print(f"    field differs: {k[0]} {name}: tracerpt={tval!r} s4={sval!r}")
    print(f"  fields checked {checked} mismatched {mism}")
    if mism:
        ok = False
    return ok


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("dir", nargs="?", default="/mnt/c/Temp/ETL/Logs", help="directory searched recursively")
    ap.add_argument("--s4", default="./target/release_Opt0/s4", help="path to s4 binary")
    ap.add_argument("-v", "--verbose", action="store_true")
    args = ap.parse_args()

    root = Path(args.dir)
    etls = sorted(p for p in root.rglob("*.etl") if Path(str(p) + ".xml").exists())
    if not etls:
        print(f"no .etl files with sibling .etl.xml under {root}", file=sys.stderr)
        return 2
    failed = 0
    for etl in etls:
        print(etl)
        if not compare_file(args.s4, etl, Path(str(etl) + ".xml"), args.verbose):
            failed += 1
    print(f"\n{len(etls) - failed}/{len(etls)} files matched")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
