Python scripts intended to be called by Super Speedy Syslog Searcher (s4) via a Pyhon interpreter.

These Python scripts use Python log parsers implemented in Python and not available in Rust.

Script `etl_reader_dissect_etl.py` uses [`dissect.etl`](https://pypi.org/project/dissect.etl/)
The `dissect.etl` parser is far fast and simple.

Script `etl_reader_etl_parser.py` uses [`etl-parser`](https://pypi.org/project/etl-parser/)
The `etl-parser` parser is slow and thorough.
