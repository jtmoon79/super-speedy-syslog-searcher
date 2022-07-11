# CHANGELOG

Manual changelog for super_speedy_syslog_searcher.

## Unreleased
---

### New
* WIP handle tar files (b8deef3439f8e8b9a949a0a1cfa16d2c027c391f )
* add enum_BoxPtrs::DoublePtr (61f15e13d086a5d6c0e5a18d44c730ebe77a046a)
* add CHANGELOG.md

### Changes
* refactor name `enum_BoxPtrs` to `LinePartPtrs` (b5505730100a9780877eb3e1cb4d280f02845863)
* (TOOLS) rust-test.sh use nextest if available (1bf2784185df479a3a17975f773e3a505f735e26)

### Fixes
* fix --blocksz minimum check (07baf6df44ec3ccd2da43f3c5cb9f5ef30a6b0e8)
* printers.rs fix macro `print_color_highlight_dt` (6659509095d19163bd65bd24a9a554cf25207395)
* (DEBUG) line.rs impl LinePart::count_bytes,len (9d9179cf63c4167ac46b5c398b2c6b718ea9a022)
  Fix `LinePart::count_bytes`
* (DEBUG) printers.rs fix `char_to_char_noraw` (ced4667fd5f16682a46e70d435a9a473885c70b6)

## 0.0.22 - (2022-07-10)
---

### New
* refactor datetime string matching (3562638d37272b2befa7f9007307fd4088cdd00c)
  refactor datetime string matching within a `Line` to use regex.
* (TOOLS) add hexdump.py (031434f4d9dfb4e0f8190a720f8db57a3772e3a2)

### Changes

### Fixes

### Breaks

----

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/) and [Keep a Changelog](http://keepachangelog.com/).
