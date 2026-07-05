# `gen-1000-3-foobar-noyear.log`

## `--version`

```text
s4 (Super Speedy Syslog Searcher)
Version: 0.9.82
MSRV: 1.88.0
Profile: alloc_tracker
Allocator: alloc_tracker
Features: alloc_tracker
Platform: x86_64-unknown-linux-gnu
Target OS: linux
Target OS Family: unix
Arch: x86_64
Compiled Regular Expressions: 188
Compiler Version: 1.88.0
Rust Build Flags: 
Optimization Level: 0
Git Commit: 4d4a4cc9f70f8f1548d32fd3660414050e3d6b0e
Build Date: 2026-07-03T16:43:55
License: MIT
Repository: https://github.com/jtmoon79/super-speedy-syslog-searcher
Author: James Thomas Moon

```

## Command

`$ ./target/alloc_tracker/s4 ./logs/other/tests/gen-1000-3-foobar-noyear.log`

## Allocator Tracking results

| ***File:line:col***<br/>***Call Site*** | Thread<br/>ID | Thread<br/>Name | Allocations | Bytes | Bytes<br/>per Allocation |
| :-- | ---: | :--- | ---: | ---: | ---: |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33::ENGINE_BYTES::exec::transition_epsilons_exec::h3e00100e8a190377` | 3 | `gen-1000-3-foobar-noyear.log` | 87,174 | 80,993,664 (77.24 MiB) | 929 (929 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33::ENGINE_BYTES::exec::transition_symbols_exec::h4fde8a2c9ef4687b` | 3 | `gen-1000-3-foobar-noyear.log` | 6,012 | 7,502,976 (7.16 MiB) | 1,248 (1.22 KiB) |
| `src/data/line.rs:553:9`<br/>`s4lib::data::line::LinePart::block_boxptr_ab::heef3b5e54dbe152a` | 3 | `gen-1000-3-foobar-noyear.log` | 3,037 | 48,592 (47.45 KiB) | 16 (16 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33::ENGINE_BYTES::exec::hb45f9861b268ee6b` | 3 | `gen-1000-3-foobar-noyear.log` | 3,006 | 1,250,496 (1.19 MiB) | 416 (416 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_33::hfd64b7d408fa9a2b` | 3 | `gen-1000-3-foobar-noyear.log` | 3,006 | 360,720 (352.27 KiB) | 120 (120 B) |
| `src/readers/syslinereader.rs:2226:13`<br/>`s4lib::readers::syslinereader::SyslineReader::parse_datetime_in_line::hfda3491022f6d360` | 3 | `gen-1000-3-foobar-noyear.log` | 3,006 | 28,536 (27.87 KiB) | 9 (9 B) |
| `src/s4/s4.rs:5406:53`<br/>`s4::s4::processing_loop::recv_many_chan::hf2b1c552a15b43da` | 1 | `main` | 3,005 | 384,640 (375.62 KiB) | 128 (128 B) |
| `src/readers/syslinereader.rs:1436:34`<br/>`s4lib::readers::syslinereader::SyslineReader::insert_sysline::h6d134c5737c430e4` | 3 | `gen-1000-3-foobar-noyear.log` | 3,005 | 288,480 (281.72 KiB) | 96 (96 B) |
| `src/data/sysline.rs:187:20`<br/>`s4lib::data::sysline::Sysline::new_no_lines_with_offsets::h65a69eba1207dc56` | 3 | `gen-1000-3-foobar-noyear.log` | 3,005 | 24,040 (23.48 KiB) | 8 (8 B) |
| `src/data/line.rs:643:24`<br/>`<s4lib::data::line::Line as core::default::Default>::default::h2e9d379392e46c71` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 144,144 (140.77 KiB) | 48 (48 B) |
| `src/readers/linereader.rs:610:28`<br/>`s4lib::readers::linereader::LineReader::insert_line::h77cbfe2a3367d47d` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 120,120 (117.30 KiB) | 40 (40 B) |
| `src/data/datetime.rs:1505:46`<br/>`s4lib::data::datetime::captures_to_buffer_bytes::hdaafb777d3e8b48d` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 24,024 (23.46 KiB) | 8 (8 B) |
| `src/readers/syslinereader.rs:1453:13`<br/>`s4lib::readers::syslinereader::SyslineReader::insert_sysline::h6d134c5737c430e4` | 3 | `gen-1000-3-foobar-noyear.log` | 500 | 146,720 (143.28 KiB) | 293 (293 B) |
| `src/readers/syslinereader.rs:1444:9`<br/>`s4lib::readers::syslinereader::SyslineReader::insert_sysline::h6d134c5737c430e4` | 3 | `gen-1000-3-foobar-noyear.log` | 500 | 102,720 (100.31 KiB) | 205 (205 B) |
| `src/readers/linereader.rs:632:9`<br/>`s4lib::readers::linereader::LineReader::insert_line::h77cbfe2a3367d47d` | 3 | `gen-1000-3-foobar-noyear.log` | 499 | 102,528 (100.12 KiB) | 205 (205 B) |
| `src/readers/linereader.rs:619:9`<br/>`s4lib::readers::linereader::LineReader::insert_line::h77cbfe2a3367d47d` | 3 | `gen-1000-3-foobar-noyear.log` | 499 | 102,528 (100.12 KiB) | 205 (205 B) |
| `src/s4/s4.rs:3466:16`<br/>`s4::s4::cli_process_args::hbb85d08869f2b49d` | 1 | `main` | 204 | 25,217 (24.63 KiB) | 123 (123 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25::ENGINE_BYTES::exec::transition_epsilons_exec::hd0d4a2315ba73965` | 3 | `gen-1000-3-foobar-noyear.log` | 145 | 17,808 (17.39 KiB) | 122 (122 B) |
| `src/s4/s4.rs:2517:10`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 52 | 48,677 (47.54 KiB) | 936 (936 B) |
| `src/readers/filepreprocessor.rs:357:45`<br/>`s4lib::readers::filepreprocessor::detect_filetype_text_encoding::h7dd6336f00607e8f` | 3 | `gen-1000-3-foobar-noyear.log` | 39 | 3,655 (3.57 KiB) | 93 (93 B) |
| `src/readers/syslinereader.rs:953:13`<br/>`s4lib::readers::syslinereader::SyslineReader::new::h7f733daf2353a4cc` | 3 | `gen-1000-3-foobar-noyear.log` | 32 | 6,624 (6.47 KiB) | 207 (207 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32::ENGINE_BYTES::exec::transition_epsilons_exec::h6703165466f8bb78` | 3 | `gen-1000-3-foobar-noyear.log` | 26 | 140,592 (137.30 KiB) | 5,407 (5.28 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27::ENGINE_BYTES::exec::transition_epsilons_exec::h014bd6cebdd2cb3f` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 39,600 (38.67 KiB) | 1,650 (1.61 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30::ENGINE_BYTES::exec::transition_epsilons_exec::h0e033f9f47dcd2f3` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 39,024 (38.11 KiB) | 1,626 (1.59 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29::ENGINE_BYTES::exec::transition_epsilons_exec::h83022f11a5976a41` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 39,024 (38.11 KiB) | 1,626 (1.59 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28::ENGINE_BYTES::exec::transition_epsilons_exec::hd0bfadd4221ee7ba` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 39,024 (38.11 KiB) | 1,626 (1.59 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31::ENGINE_BYTES::exec::transition_epsilons_exec::hed6d4213f7f44796` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 34,992 (34.17 KiB) | 1,458 (1.42 KiB) |
| `src/readers/syslinereader.rs:1489:44`<br/>`s4lib::readers::syslinereader::SyslineReader::drop_data::h4771b0d956895d77` | 3 | `gen-1000-3-foobar-noyear.log` | 22 | 278,000 (271.48 KiB) | 12,636 (12.34 KiB) |
| `src/readers/blockreader.rs:2825:26`<br/>`s4lib::readers::blockreader::BlockReader::read_block_File::h2676b52d4fd63831` | 3 | `gen-1000-3-foobar-noyear.log` | 14 | 894,569 (873.60 KiB) | 63,897 (62.40 KiB) |
| `src/readers/blockreader.rs:2846:30`<br/>`s4lib::readers::blockreader::BlockReader::read_block_File::h2676b52d4fd63831` | 3 | `gen-1000-3-foobar-noyear.log` | 14 | 560 (560 B) | 40 (40 B) |
| `src/data/line.rs:731:9`<br/>`s4lib::data::line::Line::prepend::h462a79a95eaa7f20` | 3 | `gen-1000-3-foobar-noyear.log` | 13 | 2,496 (2.44 KiB) | 192 (192 B) |
| `src/readers/syslinereader.rs:2307:27`<br/>`s4lib::readers::syslinereader::SyslineReader::parse_datetime_in_line_cached::h55c043d2ab286904` | 3 | `gen-1000-3-foobar-noyear.log` | 11 | 880 (880 B) | 80 (80 B) |
| `src/s4/s4.rs:3825:5`<br/>`s4::s4::set_signal_handler::h1be51f5b5f6b4d86` | 1 | `main` | 7 | 181 (181 B) | 25 (25 B) |
| `src/s4/s4.rs:5318:15`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 5 | 293 (293 B) | 58 (58 B) |
| `src/readers/syslinereader.rs:2035:17`<br/>`s4lib::readers::syslinereader::SyslineReader::dt_patterns_indexes_refresh::hdae1c0b51deb51c7` | 3 | `gen-1000-3-foobar-noyear.log` | 4 | 9,088 (8.88 KiB) | 2,272 (2.22 KiB) |
| `src/readers/syslinereader.rs:3105:13`<br/>`s4lib::readers::syslinereader::SyslineReader::find_sysline_year::h7d80c10abf45d788` | 3 | `gen-1000-3-foobar-noyear.log` | 4 | 192 (192 B) | 48 (48 B) |
| `src/readers/blockreader.rs:2739:9`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_LRU_cache::ha31a5c120f795d96` | 3 | `gen-1000-3-foobar-noyear.log` | 4 | 128 (128 B) | 32 (32 B) |
| `src/s4/s4.rs:363:13`<br/>`s4::s4::LOCAL_NOW::__init::{{closure}}::h3a2963b7ba27b12e` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_4::ere_match_struct_4::ENGINE_BYTES::exec::transition_epsilons_exec::hfb6024639b490ac1` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 912 (912 B) | 304 (304 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_10::ere_match_struct_10::ENGINE_BYTES::exec::transition_epsilons_exec::h1a35cb6b1c56b141` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 848 (848 B) | 282 (282 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_24::ere_match_struct_24::ENGINE_BYTES::exec::transition_epsilons_exec::h736ce821cbc2cf18` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 832 (832 B) | 277 (277 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_18::ere_match_struct_18::ENGINE_BYTES::exec::transition_epsilons_exec::h0718a46d3aa57fab` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 816 (816 B) | 272 (272 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_22::ere_match_struct_22::ENGINE_BYTES::exec::transition_epsilons_exec::h4c346f2f396ceadc` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 816 (816 B) | 272 (272 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_14::ere_match_struct_14::ENGINE_BYTES::exec::transition_epsilons_exec::h9abee15f24d52e00` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 784 (784 B) | 261 (261 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_2::ere_match_struct_2::ENGINE_BYTES::exec::transition_epsilons_exec::he9364fffc74b5d0b` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 768 (768 B) | 256 (256 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_1::ere_match_struct_1::ENGINE_BYTES::exec::transition_epsilons_exec::had7bcd1e0dcaecac` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 768 (768 B) | 256 (256 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_3::ere_match_struct_3::ENGINE_BYTES::exec::transition_epsilons_exec::hac97c49fbfd11442` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 768 (768 B) | 256 (256 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_5::ere_match_struct_5::ENGINE_BYTES::exec::transition_epsilons_exec::h525d46a79c6043b1` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 720 (720 B) | 240 (240 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_6::ere_match_struct_6::ENGINE_BYTES::exec::transition_epsilons_exec::heae450109ec68651` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 720 (720 B) | 240 (240 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_7::ere_match_struct_7::ENGINE_BYTES::exec::transition_epsilons_exec::hdd7620693554e69a` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 704 (704 B) | 234 (234 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_8::ere_match_struct_8::ENGINE_BYTES::exec::transition_epsilons_exec::h1c6405d2d92404ad` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 704 (704 B) | 234 (234 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_26::ere_match_struct_26::ENGINE_BYTES::exec::transition_epsilons_exec::h348d0c860d38d955` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 704 (704 B) | 234 (234 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_9::ere_match_struct_9::ENGINE_BYTES::exec::transition_epsilons_exec::hc027fb05544c5037` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 704 (704 B) | 234 (234 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_16::ere_match_struct_16::ENGINE_BYTES::exec::transition_epsilons_exec::h67ab113aad6c25d8` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 688 (688 B) | 229 (229 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_20::ere_match_struct_20::ENGINE_BYTES::exec::transition_epsilons_exec::h0b45b18ccd37ab00` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 688 (688 B) | 229 (229 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_17::ere_match_struct_17::ENGINE_BYTES::exec::transition_epsilons_exec::h53a128b69b842287` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 688 (688 B) | 229 (229 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_21::ere_match_struct_21::ENGINE_BYTES::exec::transition_epsilons_exec::h0cc9a52ae4f2c88d` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 688 (688 B) | 229 (229 B) |
| `src/readers/blockreader.rs:2758:28`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_storage::hc6eed8e8372528ca` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 672 (672 B) | 224 (224 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_11::ere_match_struct_11::ENGINE_BYTES::exec::transition_epsilons_exec::h9abecd61fa6ce625` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 640 (640 B) | 213 (213 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_19::ere_match_struct_19::ENGINE_BYTES::exec::transition_epsilons_exec::h12e38349381db954` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 624 (624 B) | 208 (208 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_12::ere_match_struct_12::ENGINE_BYTES::exec::transition_epsilons_exec::hff1b362ae3f7de08` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 576 (576 B) | 192 (192 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_15::ere_match_struct_15::ENGINE_BYTES::exec::transition_epsilons_exec::ha07292851e4459bc` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 576 (576 B) | 192 (192 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_13::ere_match_struct_13::ENGINE_BYTES::exec::transition_epsilons_exec::hd4221bfe9ecf3b40` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 576 (576 B) | 192 (192 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_23::ere_match_struct_23::ENGINE_BYTES::exec::transition_epsilons_exec::h1e0dfbc664483514` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 480 (480 B) | 160 (160 B) |
| `src/readers/syslinereader.rs:993:47`<br/>`s4lib::readers::syslinereader::SyslineReader::new::h7f733daf2353a4cc` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 448 (448 B) | 149 (149 B) |
| `src/readers/blockreader.rs:2772:14`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_storage::hc6eed8e8372528ca` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 408 (408 B) | 136 (136 B) |
| `src/readers/linereader.rs:273:34`<br/>`s4lib::readers::linereader::LineReader::new::h1c941ff0ea52f85f` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 384 (384 B) | 128 (128 B) |
| `src/readers/syslinereader.rs:986:37`<br/>`s4lib::readers::syslinereader::SyslineReader::new::h7f733daf2353a4cc` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 248 (248 B) | 82 (82 B) |
| `src/readers/blockreader.rs:1867:35`<br/>`s4lib::readers::blockreader::BlockReader::new::hdd8ac647502e11e0` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 216 (216 B) | 72 (72 B) |
| `src/readers/linereader.rs:2493:13`<br/>`s4lib::readers::linereader::LineReader::find_line::he7f185391d226fb6` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 144 (144 B) | 48 (48 B) |
| `src/s4/s4.rs:440:5`<br/>`<s4::s4::CLI_Color_Choice as clap_builder::derive::ValueEnum>::to_possible_value::hea38343b8f5aa490` | 1 | `main` | 3 | 24 (24 B) | 8 (8 B) |
| `src/s4/s4.rs:5314:13`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 2 | 3,800 (3.71 KiB) | 1,900 (1.86 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30::ENGINE_BYTES::exec::transition_symbols_exec::hae30c1eb87056bce` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27::ENGINE_BYTES::exec::transition_symbols_exec::h6cb4a9fa596ba043` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28::ENGINE_BYTES::exec::transition_symbols_exec::hcfdd2c0695c075b0` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29::ENGINE_BYTES::exec::transition_symbols_exec::h0a34b20d01aa7051` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32::ENGINE_BYTES::exec::transition_symbols_exec::h7a20fe7b1ae41e91` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,264 (3.19 KiB) | 1,632 (1.59 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31::ENGINE_BYTES::exec::transition_symbols_exec::h03b9eef6db1ca7cc` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,264 (3.19 KiB) | 1,632 (1.59 KiB) |
| `src/readers/syslinereader.rs:3640:9`<br/>`s4lib::readers::syslinereader::SyslineReader::summary::hf8852898e7fd08fb` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 384 (384 B) | 192 (192 B) |
| `src/readers/syslinereader.rs:3634:9`<br/>`s4lib::readers::syslinereader::SyslineReader::summary::hf8852898e7fd08fb` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 384 (384 B) | 192 (192 B) |
| `src/s4/s4.rs:2855:16`<br/>`s4::s4::cli_process_blocksz::hdb09b92225a953df` | 1 | `main` | 2 | 156 (156 B) | 78 (78 B) |
| `src/s4/s4.rs:3991:11`<br/>`s4::s4::chan_send::h1a0f0b137ecf813a` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 144 (144 B) | 72 (72 B) |
| `src/s4/s4.rs:5435:59`<br/>`s4::s4::processing_loop::recv_many_chan::hf2b1c552a15b43da` | 1 | `main` | 2 | 144 (144 B) | 72 (72 B) |
| `src/readers/syslinereader.rs:3641:13`<br/>`s4lib::readers::syslinereader::SyslineReader::summary::hf8852898e7fd08fb` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 128 (128 B) | 64 (64 B) |
| `src/readers/syslinereader.rs:2794:13`<br/>`s4lib::readers::syslinereader::SyslineReader::find_sysline_in_block_year::h39642b95a9f8f7f3` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 96 (96 B) | 48 (48 B) |
| `src/readers/linereader.rs:1472:29`<br/>`s4lib::readers::linereader::LineReader::find_line_in_block::hffc66ed160414a51` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 96 (96 B) | 48 (48 B) |
| `src/readers/syslogprocessor.rs:1501:20`<br/>`s4lib::readers::syslogprocessor::SyslogProcessor::summary_complete::h31682bfdffb22ad0` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 94 (94 B) | 47 (47 B) |
| `src/s4/s4.rs:5248:13`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 2 | 94 (94 B) | 47 (47 B) |
| `src/readers/filepreprocessor.rs:358:24`<br/>`s4lib::readers::filepreprocessor::detect_filetype_text_encoding::h7dd6336f00607e8f` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 13 (13 B) | 6 (6 B) |
| `src/s4/s4.rs:5205:34`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 2,452 (2.39 KiB) | 2,452 (2.39 KiB) |
| `src/printer/printers.rs:829:32`<br/>`s4lib::printer::printers::PrinterLogMessage::new::hb43a01fede4ab37f` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| `src/printer/printers.rs:828:21`<br/>`s4lib::printer::printers::PrinterLogMessage::new::hb43a01fede4ab37f` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| `src/readers/syslinereader.rs:950:39`<br/>`s4lib::readers::syslinereader::SyslineReader::new::h7f733daf2353a4cc` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 1,504 (1.47 KiB) | 1,504 (1.47 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25::ENGINE_BYTES::exec::transition_symbols_exec::hd2f45a2b5675fc3e` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 1,472 (1.44 KiB) | 1,472 (1.44 KiB) |
| `src/s4/s4.rs:5603:29`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 1,248 (1.22 KiB) | 1,248 (1.22 KiB) |
| `src/printer/printers.rs:792:22`<br/>`s4lib::printer::printers::PrinterLogMessage::new::hb43a01fede4ab37f` | 1 | `main` | 1 | 1,024 (1.00 KiB) | 1,024 (1.00 KiB) |
| `src/s4/s4.rs:5483:34`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 980 (980 B) | 980 (980 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_24::ere_match_struct_24::ENGINE_BYTES::exec::hd91089af15b85295` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 800 (800 B) | 800 (800 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_1::ere_match_struct_1::ENGINE_BYTES::exec::h8de770d56d98f549` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_4::ere_match_struct_4::ENGINE_BYTES::exec::hcfaa080c8ada9930` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_3::ere_match_struct_3::ENGINE_BYTES::exec::h0d6f7a4d6b3d55be` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25::ENGINE_BYTES::exec::h423c6354adbdbc77` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_2::ere_match_struct_2::ENGINE_BYTES::exec::h492fe28417467720` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_6::ere_match_struct_6::ENGINE_BYTES::exec::h8153a825416ee1dc` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_8::ere_match_struct_8::ENGINE_BYTES::exec::h007cb0210dc064e5` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_10::ere_match_struct_10::ENGINE_BYTES::exec::hae30fe98b588113e` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_5::ere_match_struct_5::ENGINE_BYTES::exec::hcdab0664c2381a8d` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_26::ere_match_struct_26::ENGINE_BYTES::exec::h290e3087388c951c` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_9::ere_match_struct_9::ENGINE_BYTES::exec::hae31f6c5b461bc74` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_7::ere_match_struct_7::ENGINE_BYTES::exec::h1e793bda2124adac` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_20::ere_match_struct_20::ENGINE_BYTES::exec::hc1a8769398b9e302` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_22::ere_match_struct_22::ENGINE_BYTES::exec::h3fc7423cbca2beec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_17::ere_match_struct_17::ENGINE_BYTES::exec::h956430932c429a60` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27::ENGINE_BYTES::exec::h8de1ca59b435ffcf` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_16::ere_match_struct_16::ENGINE_BYTES::exec::h1cb2dc3ac8df53ec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_14::ere_match_struct_14::ENGINE_BYTES::exec::h926a46745862bff7` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_18::ere_match_struct_18::ENGINE_BYTES::exec::h6523f43b56c1afee` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_21::ere_match_struct_21::ENGINE_BYTES::exec::h91b88588030992b3` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29::ENGINE_BYTES::exec::h9551bf9d49ceee65` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28::ENGINE_BYTES::exec::hdb20027d1dee9ab9` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_11::ere_match_struct_11::ENGINE_BYTES::exec::h2120c85cb1ec2fcf` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30::ENGINE_BYTES::exec::h5729d114a5a5babb` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_15::ere_match_struct_15::ENGINE_BYTES::exec::he7017f1be1f7c3a8` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_12::ere_match_struct_12::ENGINE_BYTES::exec::ha67a66caa9f2bf45` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_19::ere_match_struct_19::ENGINE_BYTES::exec::h355fb1d5029df9b9` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_13::ere_match_struct_13::ENGINE_BYTES::exec::h996bc82f60bc8693` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32::ENGINE_BYTES::exec::hf178a61f66aeee4f` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31::ENGINE_BYTES::exec::h6e0da10f5561890b` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2018:9`<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl::GROUP_NAMES_MAP_STR as core::ops::deref::Deref>::deref::__static_ref_initialize::hb4824988cc1c875b` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 416 (416 B) | 416 (416 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2240:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_23::ere_match_struct_23::ENGINE_BYTES::exec::h6f657581f37f41a5` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 416 (416 B) | 416 (416 B) |
| `src/s4/s4.rs:5121:17`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 368 (368 B) | 368 (368 B) |
| `src/readers/syslogprocessor.rs:177:9`<br/>`<s4lib::readers::syslogprocessor::BLOCKZERO_ANALYSIS_LINE_COUNT_MIN_MAP as core::ops::deref::Deref>::deref::__static_ref_initialize::h3ae58caf9d3fb05a` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 280 (280 B) | 280 (280 B) |
| `src/readers/syslogprocessor.rs:192:9`<br/>`<s4lib::readers::syslogprocessor::BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP as core::ops::deref::Deref>::deref::__static_ref_initialize::ha52d9ce305f011e6` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:5316:9`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:5006:34`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 276 (276 B) | 276 (276 B) |
| `src/s4/s4.rs:3762:51`<br/>`s4::s4::main::h0da61b24b42f6b8a` | 1 | `main` | 1 | 224 (224 B) | 224 (224 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2014:9`<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl::GROUP_NAMES_MAP_STR as core::ops::deref::Deref>::deref::__static_ref_initialize::hb4824988cc1c875b` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 216 (216 B) | 216 (216 B) |
| `src/s4/s4.rs:5012:49`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2011:9`<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl::GROUP_NAMES_MAP_STR as core::ops::deref::Deref>::deref::__static_ref_initialize::hb4824988cc1c875b` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 116 (116 B) | 116 (116 B) |
| `src/s4/s4.rs:5014:40`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 116 (116 B) | 116 (116 B) |
| `src/readers/filepreprocessor.rs:1472:22`<br/>`s4lib::readers::filepreprocessor::process_path::h940fb84aa9eeba43` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| `src/s4/s4.rs:2566:12`<br/>`<s4::s4::CLI_Args as clap_builder::derive::FromArgMatches>::from_arg_matches_mut::{{closure}}::h8f4c636549f04554` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| `src/s4/s4.rs:5203:32`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5031:41`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5029:35`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5027:44`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5215:28`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/readers/filepreprocessor.rs:1502:53`<br/>`s4lib::readers::filepreprocessor::process_path::h940fb84aa9eeba43` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| `src/s4/s4.rs:5784:62`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5473:26`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5489:52`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/readers/linereader.rs:895:17`<br/>`s4lib::readers::linereader::LineReader::check_store::h26a4a88ddf2bf1dd` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 48 (48 B) | 48 (48 B) |
| `src/readers/linereader.rs:878:21`<br/>`s4lib::readers::linereader::LineReader::check_store::h26a4a88ddf2bf1dd` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 48 (48 B) | 48 (48 B) |
| `src/readers/linereader.rs:1288:21`<br/>`s4lib::readers::linereader::LineReader::find_line_in_block::hffc66ed160414a51` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:3484:33`<br/>`s4::s4::cli_process_args::hbb85d08869f2b49d` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:4039:9`<br/>`s4::s4::exec_syslogprocessor::h50c304c304cbf5f0` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| `src/readers/blockreader.rs:649:35`<br/>`s4lib::readers::blockreader::BlockReader::new::hdd8ac647502e11e0` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| `src/readers/blockreader.rs:681:20`<br/>`s4lib::readers::blockreader::BlockReader::new::hdd8ac647502e11e0` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| `src/s4/s4.rs:3511:29`<br/>`s4::s4::cli_process_args::hbb85d08869f2b49d` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| `src/readers/filepreprocessor.rs:1502:87`<br/>`s4lib::readers::filepreprocessor::process_path::h940fb84aa9eeba43` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| `src/readers/filepreprocessor.rs:1468:33`<br/>`s4lib::readers::filepreprocessor::process_path::h940fb84aa9eeba43` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| `src/s4/s4.rs:5121:56`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| `src/s4/s4.rs:2918:5`<br/>`s4::s4::cli_process_tz_offset::h3231a46bb9d02e0c` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| `src/s4/s4.rs:5319:19`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 28 (28 B) | 28 (28 B) |
| `src/s4/s4.rs:5317:32`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 28 (28 B) | 28 (28 B) |
| `src/readers/helpers.rs:30:5`<br/>`s4lib::readers::helpers::basename::hb2b98f4ae9040d70` | 1 | `main` | 1 | 28 (28 B) | 28 (28 B) |
| `src/readers/syslinereader.rs:1828:34`<br/>`s4lib::readers::syslinereader::SyslineReader::find_datetime_in_line::ha98fff24fea7260a` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 22 (22 B) | 22 (22 B) |
| `src/s4/s4.rs:2917:28`<br/>`s4::s4::cli_process_tz_offset::h3231a46bb9d02e0c` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| `src/data/line.rs:511:9`<br/>`s4lib::data::line::LinePart::block_boxptr_b::h7b607d68c9f36a57` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 16 (16 B) | 16 (16 B) |
| `src/data/line.rs:490:9`<br/>`s4lib::data::line::LinePart::block_boxptr_a::h0434b7a627f45bfb` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2716:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2607:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2635:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2803:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2756:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2568:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2729:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2768:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2555:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2589:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2621:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2681:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2651:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2578:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2703:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2741:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2669:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2780:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2693:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2817:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/printer/printers.rs:794:28`<br/>`s4lib::printer::printers::PrinterLogMessage::new::hb43a01fede4ab37f` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| `src/readers/syslinereader.rs:981:31`<br/>`s4lib::readers::syslinereader::SyslineReader::new::h7f733daf2353a4cc` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2602:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h79a17608cfbbd63c` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2811:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hcf68c27431597d26` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2890:23`<br/>`s4::s4::cli_parse_blocksz::h7d22fd388ec587af` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5305:26`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5500:26`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:3440:9`<br/>`s4::s4::unescape::unescape_str::h8985d470cc66fb45` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2751:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h5b3e8b9496df1070` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2775:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h9f9bfb2cadb5032e` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2889:32`<br/>`s4::s4::cli_parse_blocksz::h7d22fd388ec587af` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2793:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h740ed821b86ae503` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2811:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hcf68c27431597d26` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2736:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h6b55cb440a7a965d` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2763:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h961ed6d112333819` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/readers/filepreprocessor.rs:666:31`<br/>`s4lib::readers::filepreprocessor::pathbuf_to_filetype_impl::h64191eb604206c63` | 1 | `main` | 1 | 3 (3 B) | 3 (3 B) |
| `src/s4/s4.rs:2709:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::ha45a23112ac7249a` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| `src/s4/s4.rs:2709:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::ha45a23112ac7249a` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |



## Allocator Tracking summary

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| normal allocations | 93,336,113 | 126,236 | normal program allocations; this is the most useful number |
| total deallocations | 284,730,965 | 698,444 | includes normal program deallocations and tracking deallocations |
| current outstanding | 68,029,963 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| total from tracking | 259,424,792 | 671,696 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 257,705,168 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 1,719,624 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 36 | 100 to 19 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| parameter | value | about |
| :--- | ---: | :--- |
| frame depth | 1 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 208 | entries in the table above |
| cached file names | 12 | |
| cached function names | 133 | |
| cached thread names | 2 | |

2026-07-03 19:37:29.401951223 -07:00
