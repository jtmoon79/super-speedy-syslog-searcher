# __init__.py
# -*- coding: utf-8 -*-
#

__version__ = "0.7.79"
"""
s4_event_readers version string
Must match pyproject.toml
"""

DELIMITER_EVENTS: str = chr(0)
"""Delimiter between Events (Null character)"""

DELIMITER_TS_EVENT: str = chr(30)
"""Delimiter between Timestamp and Event (Record Separator character)"""

DELIMITER_EVENT_END: str = "\n" + DELIMITER_EVENTS
"""Printed delimiter between Events"""


def s4_event_bytes(ts_beg: int, ts_end: int, timestamp: int, event: str) -> bytes:
    """
    Convert an Event to a bytes representation suitable for consumption
    by s4 Event readers.

    :param ts_beg: Beginning index of timestamp substring within `event` string
    :param ts_end: Ending index of timestamp substring within `event` string
    :param timestamp: Timestamp as milliseconds since Unix epoch
    :param event: Event as a string
    :return: Bytes representation of the Event
    """
    # assemble the event string
    # make sure there are no DELIMITER_EVENTS in the event string
    event_s = \
        f"{ts_beg}{DELIMITER_TS_EVENT}" \
        f"{ts_end}{DELIMITER_TS_EVENT}" \
        f"{timestamp}{DELIMITER_TS_EVENT}" \
        f"{event.replace(DELIMITER_EVENTS, ' ')}{DELIMITER_EVENT_END}"
    event_b = event_s.encode("utf-8", errors="ignore")
    return event_b
