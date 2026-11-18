# `gen-1000-3-foobar-noyear.log`

## `--version`

```text
s4 (Super Speedy Syslog Searcher)
Version: 0.11.84
MSRV: 1.99.1
Profile: alloc_tracker
Allocator: alloc_tracker
Features: alloc_tracker
Platform: x86_64-unknown-linux-gnu
Target OS: linux
Target OS Family: unix
Arch: x86_64
CPU Target: default
CPU Features: fxsr,sse,sse2
Compiled Regular Expressions: 188
Compiler Version: 1.99.1
Rust Build Flags: 
Optimization Level: 0
Git Commit: 8ce20292e705cac0c418a6ce78ff2fe489e3ef28
Build Date: 2026-11-15T09:14:13Z
License: MIT
Repository: https://github.com/jtmoon79/super-speedy-syslog-searcher
Author: James Thomas Moon

```

## Command

`$ ./target/alloc_tracker/s4 ./logs/other/tests/gen-1000-3-foobar-noyear.log`

## Allocator Tracking results

| **File:line:col**<br/>**Call Site** | **Thread<br/>ID** | **Thread<br/>Name** | **Allocations** | **Bytes** | **Bytes<br/>per Allocation** |
| :-- | ---: | :--- | ---: | ---: | ---: |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 18,036 | 78,781,248 (75.13 MiB) | 4,368 (4.27 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33>::ENGINE_BYTES::exec::transition_symbols_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 6,012 | 7,502,976 (7.16 MiB) | 1,248 (1.22 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/data/line.rs:553:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/line.rs#L553)<br/>`<s4lib[6747c811d5b9f10f]::data::line::LinePart>::block_boxptr_ab` | 3 | `gen-1000-3-foobar-noyear.log` | 3,037 | 48,592 (47.45 KiB) | 16 (16 B) |
| [`src/data/line.rs:889:57`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/line.rs#L889)<br/>`<s4lib[6747c811d5b9f10f]::data::line::Line>::get_boxptrs` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 3,006 | 1,250,496 (1.19 MiB) | 416 (416 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_33` | 3 | `gen-1000-3-foobar-noyear.log` | 3,006 | 360,720 (352.27 KiB) | 120 (120 B) |
| [`src/data/datetime.rs:1915:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/datetime.rs#L1915)<br/>`s4lib[6747c811d5b9f10f]::data::datetime::bytes_to_regex_to_datetime` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:1509:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L1509)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::insert_sysline` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 288,288 (281.53 KiB) | 96 (96 B) |
| [`src/readers/syslinereader.rs:3178:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L3178)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/data/line.rs:643:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/line.rs#L643)<br/>`<s4lib[6747c811d5b9f10f]::data::line::Line as core[efd5e57aef94d71e]::default::Default>::default` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 144,144 (140.77 KiB) | 48 (48 B) |
| [`src/data/line.rs:658:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/line.rs#L658)<br/>`<s4lib[6747c811d5b9f10f]::data::line::Line>::new` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/data/datetime.rs:1523:51`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/datetime.rs#L1523)<br/>`s4lib[6747c811d5b9f10f]::data::datetime::captures_to_buffer_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 30,030 (29.33 KiB) | 10 (10 B) |
| [`src/data/datetime.rs:1941:25`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/datetime.rs#L1941)<br/>`s4lib[6747c811d5b9f10f]::data::datetime::bytes_to_regex_to_datetime` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/data/sysline.rs:187:20`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/sysline.rs#L187)<br/>`<s4lib[6747c811d5b9f10f]::data::sysline::Sysline>::new_no_lines_with_offsets` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 24,024 (23.46 KiB) | 8 (8 B) |
| [`src/readers/syslinereader.rs:3028:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L3028)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:666:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L666)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::insert_line` | 3 | `gen-1000-3-foobar-noyear.log` | 3,000 | 120,000 (117.19 KiB) | 40 (40 B) |
| [`src/readers/linereader.rs:2621:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L2621)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:1528:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L1528)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::insert_sysline` | 3 | `gen-1000-3-foobar-noyear.log` | 499 | 146,440 (143.01 KiB) | 293 (293 B) |
| [`src/readers/syslinereader.rs:3178:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L3178)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:1517:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L1517)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::insert_sysline` | 3 | `gen-1000-3-foobar-noyear.log` | 499 | 102,528 (100.12 KiB) | 205 (205 B) |
| [`src/readers/syslinereader.rs:3178:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L3178)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:676:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L676)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::insert_line` | 3 | `gen-1000-3-foobar-noyear.log` | 498 | 102,336 (99.94 KiB) | 205 (205 B) |
| [`src/readers/linereader.rs:2621:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L2621)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:692:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L692)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::insert_line` | 3 | `gen-1000-3-foobar-noyear.log` | 498 | 102,336 (99.94 KiB) | 205 (205 B) |
| [`src/readers/linereader.rs:2621:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L2621)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3879:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3879)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | 165 | 17,885 (17.47 KiB) | 108 (108 B) |
| [`src/s4/s4.rs:4140:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4140)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:504:45`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L504)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::detect_filetype_text_encoding` | 3 | `gen-1000-3-foobar-noyear.log` | 39 | 3,655 (3.57 KiB) | 93 (93 B) |
| [`src/readers/syslogprocessor.rs:1166:57`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1166)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::blockzero_analysis_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 38 | 10,658 (10.41 KiB) | 280 (280 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:978:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L978)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | 32 | 6,624 (6.47 KiB) | 207 (207 B) |
| [`src/readers/syslogprocessor.rs:362:29`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L362)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::new` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:1563:44`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L1563)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::drop_data` | 3 | `gen-1000-3-foobar-noyear.log` | 22 | 278,032 (271.52 KiB) | 12,637 (12.34 KiB) |
| [`src/readers/syslogprocessor.rs:866:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L866)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::drop_data` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/blockreader.rs:3577:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L3577)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::read_block_File` | 3 | `gen-1000-3-foobar-noyear.log` | 14 | 894,569 (873.60 KiB) | 63,897 (62.40 KiB) |
| [`src/readers/blockreader.rs:5038:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L5038)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::read_block` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/blockreader.rs:3598:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L3598)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::read_block_File` | 3 | `gen-1000-3-foobar-noyear.log` | 14 | 560 (560 B) | 40 (40 B) |
| [`src/readers/blockreader.rs:5038:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L5038)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::read_block` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/data/line.rs:732:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/line.rs#L732)<br/>`<s4lib[6747c811d5b9f10f]::data::line::Line>::prepend` | 3 | `gen-1000-3-foobar-noyear.log` | 13 | 2,496 (2.44 KiB) | 192 (192 B) |
| [`src/readers/linereader.rs:2535:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L2535)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 8 | 138,720 (135.47 KiB) | 17,340 (16.93 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:2387:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2387)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::parse_datetime_in_line_cached` | 3 | `gen-1000-3-foobar-noyear.log` | 8 | 640 (640 B) | 80 (80 B) |
| [`src/readers/syslinereader.rs:3019:52`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L3019)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 6 | 38,304 (37.41 KiB) | 6,384 (6.23 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 6 | 38,304 (37.41 KiB) | 6,384 (6.23 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 6 | 38,304 (37.41 KiB) | 6,384 (6.23 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 6 | 38,304 (37.41 KiB) | 6,384 (6.23 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 6 | 34,272 (33.47 KiB) | 5,712 (5.58 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:4188:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4188)<br/>`s4[565658153264a5bc]::s4::set_signal_handler` | 1 | `main` | 6 | 149 (149 B) | 24 (24 B) |
| [`src/s4/s4.rs:5627:15`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5627)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 4 | 11,040 (10.78 KiB) | 2,760 (2.70 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5832:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5832)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 4 | 237 (237 B) | 59 (59 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:3184:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L3184)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_year` | 3 | `gen-1000-3-foobar-noyear.log` | 4 | 192 (192 B) | 48 (48 B) |
| [`src/readers/syslogprocessor.rs:657:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L657)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::process_missing_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/blockreader.rs:3492:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L3492)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::store_block_in_LRU_cache` | 3 | `gen-1000-3-foobar-noyear.log` | 4 | 128 (128 B) | 32 (32 B) |
| [`src/readers/blockreader.rs:3600:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L3600)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::read_block_File` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:420:48`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L420)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManagerState>::new` | 1 | `main` | 3 | 34,880 (34.06 KiB) | 11,626 (11.35 KiB) |
| [`src/readers/filehandlemanager.rs:880:40`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L880)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::new_open_max` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:2126:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2126)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::dt_patterns_indexes_refresh` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 9,024 (8.81 KiB) | 3,008 (2.94 KiB) |
| [`src/readers/syslinereader.rs:2105:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2105)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::dt_patterns_update` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:378:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L378)<br/>`s4[565658153264a5bc]::s4::LOCAL_NOW::__rust_std_internal_init_fn::{closure#0}` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| [`src/s4/s4.rs:377:17`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L377)<br/>`s4[565658153264a5bc]::s4::LOCAL_NOW::__rust_std_internal_init_fn` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/blockreader.rs:3512:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L3512)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::store_block_in_storage` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 672 (672 B) | 224 (224 B) |
| [`src/readers/blockreader.rs:3599:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L3599)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::read_block_File` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:987:65`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L987)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 448 (448 B) | 149 (149 B) |
| [`src/readers/syslogprocessor.rs:362:29`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L362)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::new` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/blockreader.rs:3524:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L3524)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::store_block_in_storage` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 408 (408 B) | 136 (136 B) |
| [`src/readers/blockreader.rs:3599:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L3599)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::read_block_File` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:278:50`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L278)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 384 (384 B) | 128 (128 B) |
| [`src/readers/syslinereader.rs:968:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L968)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:984:56`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L984)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 248 (248 B) | 82 (82 B) |
| [`src/readers/syslogprocessor.rs:362:29`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L362)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::new` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5930:66`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5930)<br/>`s4[565658153264a5bc]::s4::processing_loop::recv_many_chan` | 1 | `main` | 3 | 240 (240 B) | 80 (80 B) |
| [`src/s4/s4.rs:6080:52`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6080)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/blockreader.rs:2519:52`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L2519)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 216 (216 B) | 72 (72 B) |
| [`src/readers/linereader.rs:269:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L269)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:2626:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L2626)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 144 (144 B) | 48 (48 B) |
| [`src/readers/syslinereader.rs:2985:58`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2985)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:455:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L455)<br/>`<s4[565658153264a5bc]::s4::CLI_Color_Choice as clap_builder[639015aad05df3ec]::derive::ValueEnum>::to_possible_value` | 1 | `main` | 3 | 24 (24 B) | 8 (8 B) |
| [`src/s4/s4.rs:3879:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3879)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5825:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5825)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 2 | 3,960 (3.87 KiB) | 1,980 (1.93 KiB) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27>::ENGINE_BYTES::exec::transition_symbols_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30>::ENGINE_BYTES::exec::transition_symbols_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29>::ENGINE_BYTES::exec::transition_symbols_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28>::ENGINE_BYTES::exec::transition_symbols_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31>::ENGINE_BYTES::exec::transition_symbols_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,264 (3.19 KiB) | 1,632 (1.59 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32>::ENGINE_BYTES::exec::transition_symbols_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,264 (3.19 KiB) | 1,632 (1.59 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5622:38`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5622)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 2 | 648 (648 B) | 324 (324 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:3719:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L3719)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::summary` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 384 (384 B) | 192 (192 B) |
| [`src/readers/syslogprocessor.rs:1516:55`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1516)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::summary_complete` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:3713:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L3713)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::summary` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 384 (384 B) | 192 (192 B) |
| [`src/readers/syslogprocessor.rs:1516:55`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1516)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::summary_complete` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:1509:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L1509)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::insert_sysline` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 192 (192 B) | 96 (96 B) |
| [`src/readers/syslinereader.rs:2868:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2868)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_in_block_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:2387:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2387)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::parse_datetime_in_line_cached` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 160 (160 B) | 80 (80 B) |
| [`src/readers/syslinereader.rs:2833:52`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2833)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_in_block_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:4333:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4333)<br/>`s4[565658153264a5bc]::s4::chan_send` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 144 (144 B) | 72 (72 B) |
| [`src/s4/s4.rs:4548:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4548)<br/>`s4[565658153264a5bc]::s4::exec_syslogprocessor` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:2873:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2873)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_in_block_year` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 96 (96 B) | 48 (48 B) |
| [`src/readers/syslinereader.rs:2596:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2596)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_in_block` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:1551:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L1551)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line_in_block` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 96 (96 B) | 48 (48 B) |
| [`src/readers/syslogprocessor.rs:1231:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1231)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::blockzero_analysis_lines` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:666:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L666)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::insert_line` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 80 (80 B) | 40 (40 B) |
| [`src/readers/linereader.rs:1510:49`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L1510)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line_in_block` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:3722:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L3722)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::summary` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 32 (32 B) | 16 (16 B) |
| [`src/readers/syslogprocessor.rs:1516:55`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1516)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::summary_complete` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/data/sysline.rs:187:20`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/sysline.rs#L187)<br/>`<s4lib[6747c811d5b9f10f]::data::sysline::Sysline>::new_no_lines_with_offsets` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 16 (16 B) | 8 (8 B) |
| [`src/readers/syslinereader.rs:2716:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2716)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_in_block_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3173:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3173)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 20,480 (20.00 KiB) | 20,480 (20.00 KiB) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3079:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3079)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 10,240 (10.00 KiB) | 10,240 (10.00 KiB) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3017:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3017)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 5,120 (5.00 KiB) | 5,120 (5.00 KiB) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5655:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5655)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 2,580 (2.52 KiB) | 2,580 (2.52 KiB) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2967:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2967)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 2,560 (2.50 KiB) | 2,560 (2.50 KiB) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/printer/printers.rs:838:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/printer/printers.rs#L838)<br/>`<s4lib[6747c811d5b9f10f]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| [`src/s4/s4.rs:6408:54`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6408)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/printer/printers.rs:839:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/printer/printers.rs#L839)<br/>`<s4lib[6747c811d5b9f10f]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| [`src/s4/s4.rs:6408:54`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6408)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:976:38`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L976)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 1,504 (1.47 KiB) | 1,504 (1.47 KiB) |
| [`src/readers/syslogprocessor.rs:362:29`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L362)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::new` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:975:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L975)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 1,504 (1.47 KiB) | 1,504 (1.47 KiB) |
| [`src/readers/syslogprocessor.rs:362:29`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L362)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::new` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25>::ENGINE_BYTES::exec::transition_symbols_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 1,472 (1.44 KiB) | 1,472 (1.44 KiB) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:6125:46`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6125)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 1,288 (1.26 KiB) | 1,288 (1.26 KiB) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/printer/printers.rs:802:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/printer/printers.rs#L802)<br/>`<s4lib[6747c811d5b9f10f]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 1,024 (1.00 KiB) | 1,024 (1.00 KiB) |
| [`src/s4/s4.rs:6408:54`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6408)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5998:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5998)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 980 (980 B) | 980 (980 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_24::ere_match_struct_24>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 800 (800 B) | 800 (800 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_24::ere_match_struct_24>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_24::ere_match_struct_24>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 800 (800 B) | 800 (800 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_24::ere_match_struct_24>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_4::ere_match_struct_4>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_4::ere_match_struct_4>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_3::ere_match_struct_3>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_3::ere_match_struct_3>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_4::ere_match_struct_4>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_4::ere_match_struct_4>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_2::ere_match_struct_2>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_2::ere_match_struct_2>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_3::ere_match_struct_3>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_3::ere_match_struct_3>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_2::ere_match_struct_2>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_2::ere_match_struct_2>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_1::ere_match_struct_1>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_1::ere_match_struct_1>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_1::ere_match_struct_1>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_1::ere_match_struct_1>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_8::ere_match_struct_8>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_8::ere_match_struct_8>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_6::ere_match_struct_6>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_6::ere_match_struct_6>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_26::ere_match_struct_26>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_26::ere_match_struct_26>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_8::ere_match_struct_8>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_8::ere_match_struct_8>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_5::ere_match_struct_5>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_5::ere_match_struct_5>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_6::ere_match_struct_6>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_6::ere_match_struct_6>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_7::ere_match_struct_7>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_7::ere_match_struct_7>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_10::ere_match_struct_10>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_10::ere_match_struct_10>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_9::ere_match_struct_9>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_9::ere_match_struct_9>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_7::ere_match_struct_7>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_7::ere_match_struct_7>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_26::ere_match_struct_26>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_26::ere_match_struct_26>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_10::ere_match_struct_10>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_10::ere_match_struct_10>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_9::ere_match_struct_9>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_9::ere_match_struct_9>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_5::ere_match_struct_5>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_5::ere_match_struct_5>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_18::ere_match_struct_18>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_18::ere_match_struct_18>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_20::ere_match_struct_20>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_20::ere_match_struct_20>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_21::ere_match_struct_21>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_21::ere_match_struct_21>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_11::ere_match_struct_11>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_11::ere_match_struct_11>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_14::ere_match_struct_14>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_14::ere_match_struct_14>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_17::ere_match_struct_17>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_17::ere_match_struct_17>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_17::ere_match_struct_17>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_17::ere_match_struct_17>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_21::ere_match_struct_21>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_21::ere_match_struct_21>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_11::ere_match_struct_11>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_11::ere_match_struct_11>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_14::ere_match_struct_14>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_14::ere_match_struct_14>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_22::ere_match_struct_22>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_22::ere_match_struct_22>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_22::ere_match_struct_22>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_22::ere_match_struct_22>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_16::ere_match_struct_16>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_16::ere_match_struct_16>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_18::ere_match_struct_18>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_18::ere_match_struct_18>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_16::ere_match_struct_16>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_16::ere_match_struct_16>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_20::ere_match_struct_20>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_20::ere_match_struct_20>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_15::ere_match_struct_15>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_15::ere_match_struct_15>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_15::ere_match_struct_15>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_15::ere_match_struct_15>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_12::ere_match_struct_12>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_12::ere_match_struct_12>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_12::ere_match_struct_12>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_12::ere_match_struct_12>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_13::ere_match_struct_13>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_13::ere_match_struct_13>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_13::ere_match_struct_13>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_13::ere_match_struct_13>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_19::ere_match_struct_19>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_19::ere_match_struct_19>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_19::ere_match_struct_19>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_19::ere_match_struct_19>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_23::ere_match_struct_23>::ENGINE_BYTES::exec::transition_epsilons_exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 416 (416 B) | 416 (416 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_23::ere_match_struct_23>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_23::ere_match_struct_23>::ENGINE_BYTES::exec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 416 (416 B) | 416 (416 B) |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2254:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2254)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::DATETIME_PARSE_DATAS::ere_match_fn_23::ere_match_struct_23>::exec_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2032:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2032)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::GROUP_NAMES_MAP_STR as core[efd5e57aef94d71e]::ops::deref::Deref>::deref::__static_ref_initialize` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 416 (416 B) | 416 (416 B) |
| [`src/data/datetime.rs:1979:38`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/datetime.rs#L1979)<br/>`s4lib[6747c811d5b9f10f]::data::datetime::bytes_to_regex_to_datetime` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:880:20`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L880)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::new_open_max` | 1 | `main` | 1 | 384 (384 B) | 384 (384 B) |
| [`src/readers/filehandlemanager.rs:873:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L873)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::new` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5554:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5554)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 320 (320 B) | 320 (320 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/syslogprocessor.rs:180:11`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L180)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::BLOCKZERO_ANALYSIS_LINE_COUNT_MIN_MAP as core[efd5e57aef94d71e]::ops::deref::Deref>::deref::__static_ref_initialize` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 280 (280 B) | 280 (280 B) |
| [`src/readers/syslogprocessor.rs:1222:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1222)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::blockzero_analysis_lines` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslogprocessor.rs:195:11`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L195)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP as core[efd5e57aef94d71e]::ops::deref::Deref>::deref::__static_ref_initialize` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 280 (280 B) | 280 (280 B) |
| [`src/readers/syslogprocessor.rs:1304:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1304)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::blockzero_analysis_syslines` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:1528:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L1528)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::insert_sysline` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 280 (280 B) | 280 (280 B) |
| [`src/readers/syslinereader.rs:2868:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2868)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_in_block_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:951:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L951)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::request_open_managed` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 276 (276 B) | 276 (276 B) |
| [`src/readers/blockreader.rs:1358:72`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L1358)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5431:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5431)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 276 (276 B) | 276 (276 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5827:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5827)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 232 (232 B) | 232 (232 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:4146:51`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4146)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | 1 | 224 (224 B) | 224 (224 B) |
| [`src/s4/mod.rs:13:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/mod.rs#L13)<br/>`s4[565658153264a5bc]::main` | 1 | `main` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2028:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2028)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::GROUP_NAMES_MAP_STR as core[efd5e57aef94d71e]::ops::deref::Deref>::deref::__static_ref_initialize` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 216 (216 B) | 216 (216 B) |
| [`src/data/datetime.rs:1979:38`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/datetime.rs#L1979)<br/>`s4lib[6747c811d5b9f10f]::data::datetime::bytes_to_regex_to_datetime` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:692:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L692)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::insert_line` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 192 (192 B) | 192 (192 B) |
| [`src/readers/linereader.rs:1345:37`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L1345)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line_in_block` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:1517:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L1517)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::insert_sysline` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 192 (192 B) | 192 (192 B) |
| [`src/readers/syslinereader.rs:2868:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2868)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_in_block_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:676:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L676)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::insert_line` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 192 (192 B) | 192 (192 B) |
| [`src/readers/linereader.rs:1345:37`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L1345)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line_in_block` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:2222:35`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2222)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::dt_patterns_analysis` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 192 (192 B) | 192 (192 B) |
| [`src/readers/syslogprocessor.rs:1357:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1357)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::blockzero_analysis_syslines` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5438:49`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5438)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5971:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5971)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 128 (128 B) | 128 (128 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2025:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs#L2025)<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl[7f5a9a19a6f75348]::GROUP_NAMES_MAP_STR as core[efd5e57aef94d71e]::ops::deref::Deref>::deref::__static_ref_initialize` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 116 (116 B) | 116 (116 B) |
| [`src/data/datetime.rs:1979:38`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/datetime.rs#L1979)<br/>`s4lib[6747c811d5b9f10f]::data::datetime::bytes_to_regex_to_datetime` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5440:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5440)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:1852:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L1852)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:4149:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4149)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2976:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2976)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::FromArgMatches>::from_arg_matches_mut::{closure#0}` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| [`src/s4/s4.rs:2976:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2976)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::FromArgMatches>::from_arg_matches_mut` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5976:38`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5976)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5975:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5975)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5657:36`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5657)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:2387:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2387)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::parse_datetime_in_line_cached` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 80 (80 B) | 80 (80 B) |
| [`src/readers/syslinereader.rs:2707:52`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2707)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_sysline_in_block_year` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:1895:49`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L1895)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| [`src/s4/s4.rs:4149:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4149)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5653:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5653)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5453:44`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5453)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5455:35`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5455)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5457:41`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5457)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:956:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L956)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::check_store` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/readers/linereader.rs:1860:36`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L1860)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:1359:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L1359)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line_in_block` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/readers/syslogprocessor.rs:1231:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1231)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::blockzero_analysis_lines` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:939:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L939)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::check_store` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/readers/linereader.rs:1860:36`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L1860)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3897:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3897)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/s4/s4.rs:4140:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4140)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:374:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L374)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileEntryManaged>::new` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| [`src/readers/filehandlemanager.rs:951:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L951)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::request_open_managed` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslogprocessor.rs:1510:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1510)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::summary_complete` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| [`src/readers/syslogprocessor.rs:1116:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1116)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::process_stage4_summary` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslogprocessor.rs:1510:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1510)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::summary_complete` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| [`src/s4/s4.rs:4591:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4591)<br/>`s4[565658153264a5bc]::s4::exec_syslogprocessor` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/blockreader.rs:1322:40`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L1322)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| [`src/readers/linereader.rs:269:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L269)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/blockreader.rs:1354:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/blockreader.rs#L1354)<br/>`<s4lib[6747c811d5b9f10f]::readers::blockreader::BlockReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| [`src/readers/linereader.rs:269:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L269)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:4387:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4387)<br/>`s4[565658153264a5bc]::s4::exec_syslogprocessor` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| [`src/s4/s4.rs:5353:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5353)<br/>`s4[565658153264a5bc]::s4::exec_fileprocessor_thread` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5554:61`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5554)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5713:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5713)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3924:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3924)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| [`src/s4/s4.rs:4140:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4140)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:1848:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L1848)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| [`src/s4/s4.rs:4149:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4149)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:1895:88`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L1895)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| [`src/s4/s4.rs:4149:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4149)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5713:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5713)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/linereader.rs:666:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L666)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::insert_line` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 40 (40 B) | 40 (40 B) |
| [`src/readers/linereader.rs:1345:37`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/linereader.rs#L1345)<br/>`<s4lib[6747c811d5b9f10f]::readers::linereader::LineReader>::find_line_in_block` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3232:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3232)<br/>`s4[565658153264a5bc]::s4::cli_process_tz_offset` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| [`src/s4/s4.rs:3879:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3879)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:6004:52`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6004)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 36 (36 B) | 36 (36 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5977:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5977)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 36 (36 B) | 36 (36 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:6319:62`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6319)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 36 (36 B) | 36 (36 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5828:47`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5828)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 32 (32 B) | 32 (32 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5674:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5674)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 32 (32 B) | 32 (32 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5830:29`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5830)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 28 (28 B) | 28 (28 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/helpers.rs:116:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/helpers.rs#L116)<br/>`s4lib[6747c811d5b9f10f]::readers::helpers::basename` | 1 | `main` | 1 | 28 (28 B) | 28 (28 B) |
| [`src/s4/s4.rs:5828:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5828)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:547:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L547)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManagerState>::touch` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 24 (24 B) | 24 (24 B) |
| [`src/readers/filehandlemanager.rs:773:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L773)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManagerState>::open_entry_once` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:1902:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L1902)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::find_datetime_in_line` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 22 (22 B) | 22 (22 B) |
| [`src/readers/syslinereader.rs:2314:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2314)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::parse_datetime_in_line` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3231:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3231)<br/>`s4[565658153264a5bc]::s4::cli_process_tz_offset` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| [`src/s4/s4.rs:3879:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3879)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/data/line.rs:490:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/line.rs#L490)<br/>`<s4lib[6747c811d5b9f10f]::data::line::LinePart>::block_boxptr_a` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/data/line.rs:892:40`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/line.rs#L892)<br/>`<s4lib[6747c811d5b9f10f]::data::line::Line>::get_boxptrs` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:2126:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2126)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::dt_patterns_indexes_refresh` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/readers/syslinereader.rs:2241:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2241)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::dt_patterns_analysis` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/data/line.rs:511:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/line.rs#L511)<br/>`<s4lib[6747c811d5b9f10f]::data::line::LinePart>::block_boxptr_b` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/data/line.rs:898:74`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/line.rs#L898)<br/>`<s4lib[6747c811d5b9f10f]::data::line::Line>::get_boxptrs` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:2225:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2225)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::dt_patterns_analysis` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/readers/syslogprocessor.rs:1357:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1357)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::blockzero_analysis_syslines` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2976:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2976)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3058:20`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3058)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2986:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2986)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2997:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2997)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3043:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3043)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3029:17`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3029)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3101:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3101)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3147:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3147)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3077:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3077)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3181:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3181)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3111:25`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3111)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3159:19`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3159)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3089:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3089)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3135:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3135)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3171:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3171)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3122:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3122)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3015:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3015)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/printer/printers.rs:804:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/printer/printers.rs#L804)<br/>`<s4lib[6747c811d5b9f10f]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| [`src/s4/s4.rs:6408:54`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6408)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:509:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L509)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::detect_filetype_text_encoding` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/readers/syslogprocessor.rs:1166:57`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1166)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::blockzero_analysis_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:1018:41`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L1018)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::new` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/readers/syslogprocessor.rs:362:29`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L362)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::new` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/readers/syslinereader.rs:2148:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2148)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::dt_patterns_buffer_refresh` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/readers/syslinereader.rs:2243:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslinereader.rs#L2243)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslinereader::SyslineReader>::dt_patterns_analysis` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3012:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3012)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#0}` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:3012:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3012)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5813:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5813)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3855:51`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3855)<br/>`s4[565658153264a5bc]::s4::unescape::unescape_str` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:4021:43`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4021)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:506:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L506)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::detect_filetype_text_encoding` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 5 (5 B) | 5 (5 B) |
| [`src/readers/syslogprocessor.rs:1166:57`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/syslogprocessor.rs#L1166)<br/>`<s4lib[6747c811d5b9f10f]::readers::syslogprocessor::SyslogProcessor>::blockzero_analysis_bytes` | 3 | `gen-1000-3-foobar-noyear.log` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3144:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3144)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#3}` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| [`src/s4/s4.rs:3144:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3144)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3168:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3168)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#5}` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| [`src/s4/s4.rs:3168:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3168)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5683:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5683)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3156:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3156)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#4}` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:3156:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3156)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:6015:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6015)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:821:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L821)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::pathbuf_to_filetype_impl` | 1 | `main` | 1 | 3 (3 B) | 3 (3 B) |
| [`src/readers/filepreprocessor.rs:1278:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L1278)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::pathbuf_to_filetype` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3119:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3119)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#1}` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| [`src/s4/s4.rs:3119:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3119)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3119:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3119)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#1}` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| [`src/s4/s4.rs:3119:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3119)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |


## Allocator Tracking summary

| **tracked** | **bytes** | **calls** | **about** |
| :--- | ---: | ---: | :--- |
| normal allocations | 90,732,879 | 50,745 | normal program allocations; this is the most useful number |
| total deallocations | 286,395,311 | 475,074 | includes normal program deallocations and tracking deallocations |
| current outstanding | 76,735,204 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| **tracked** | **bytes** | **calls** | **about** |
| :--- | ---: | ---: | :--- |
| total from tracking | 272,397,611 | 534,517 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 270,680,335 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 1,717,276 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 33 | 100 to 9 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| **parameter** | **value** | **about** |
| :--- | ---: | :--- |
| frame depth | 2 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 229 | entries in the table above |
| cached file names | 14 | |
| cached function names | 189 | |
| cached thread names | 2 | |

*2026-11-15 13:24:48.892933751 -08:00*
