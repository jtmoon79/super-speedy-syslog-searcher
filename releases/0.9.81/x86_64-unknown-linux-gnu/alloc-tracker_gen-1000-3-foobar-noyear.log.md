# `gen-1000-3-foobar-noyear.log`

## `--version`

```text
s4 (Super Speedy Syslog Searcher)
Version: 0.9.81
MSRV: 1.88.0
Allocator: alloc_tracker
Platform: x86_64-unknown-linux-gnu
Target OS: linux
Target OS Family: unix
Arch: x86_64
Compiled Regular Expressions: 178
Compiler Version: 1.88.0
Rust Build Flags: 
Optimization Level: 0
Build Date: 2026-05-28T21:08:06
License: MIT
Repository: https://github.com/jtmoon79/super-speedy-syslog-searcher
Author: James Thomas Moon

```

## Command

`$ ./target/alloc_tracker/s4 ./logs/other/tests/gen-1000-3-foobar-noyear.log`

## Allocator Tracking results

| ***File:line:col***<br/>***Call Site*** | Thread<br/>ID | Thread<br/>Name | Allocations | Bytes | Bytes<br/>per Allocation |
| :-- | ---: | :--- | ---: | ---: | ---: |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33::ENGINE_BYTES::exec::transition_epsilons_exec::hb22fab4d5b8f83e6` | 3 | `gen-1000-3-foobar-noyear.log` | 87,174 | 80,993,664 (77.24 MiB) | 929 (929 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33::ENGINE_BYTES::exec::transition_symbols_exec::h5f44d4acfce6f7b0` | 3 | `gen-1000-3-foobar-noyear.log` | 6,012 | 7,502,976 (7.16 MiB) | 1,248 (1.22 KiB) |
| `src/data/line.rs:423:9`<br/>`s4lib::data::line::LinePart::block_boxptr_ab::h2c0df7717986d2ec` | 3 | `gen-1000-3-foobar-noyear.log` | 3,037 | 48,592 (47.45 KiB) | 16 (16 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33::ENGINE_BYTES::exec::h22e74203e6403246` | 3 | `gen-1000-3-foobar-noyear.log` | 3,006 | 1,250,496 (1.19 MiB) | 416 (416 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_33::h669f85e574dbbbc0` | 3 | `gen-1000-3-foobar-noyear.log` | 3,006 | 360,720 (352.27 KiB) | 120 (120 B) |
| `src/readers/syslinereader.rs:1747:13`<br/>`s4lib::readers::syslinereader::SyslineReader::parse_datetime_in_line::heae8fb62a17d4e28` | 3 | `gen-1000-3-foobar-noyear.log` | 3,006 | 28,296 (27.63 KiB) | 9 (9 B) |
| `src/s4/s4.rs:5330:53`<br/>`s4::s4::processing_loop::recv_many_chan::hcf11e7434d98ce19` | 1 | `main` | 3,005 | 384,640 (375.62 KiB) | 128 (128 B) |
| `src/readers/syslinereader.rs:1003:34`<br/>`s4lib::readers::syslinereader::SyslineReader::insert_sysline::h594b7fc547658a04` | 3 | `gen-1000-3-foobar-noyear.log` | 3,005 | 216,360 (211.29 KiB) | 72 (72 B) |
| `src/data/sysline.rs:144:20`<br/>`s4lib::data::sysline::Sysline::new_no_lines::h5ae5223dc07796e8` | 3 | `gen-1000-3-foobar-noyear.log` | 3,005 | 24,040 (23.48 KiB) | 8 (8 B) |
| `src/data/line.rs:513:24`<br/>`<s4lib::data::line::Line as core::default::Default>::default::he2c7c9dc7de8c25c` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 144,144 (140.77 KiB) | 48 (48 B) |
| `src/readers/linereader.rs:473:28`<br/>`s4lib::readers::linereader::LineReader::insert_line::h30f7de6a00b83b23` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 120,120 (117.30 KiB) | 40 (40 B) |
| `src/data/datetime.rs:1504:46`<br/>`s4lib::data::datetime::captures_to_buffer_bytes::hb75bb15821eec9a2` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 24,024 (23.46 KiB) | 8 (8 B) |
| `src/readers/syslinereader.rs:1026:9`<br/>`s4lib::readers::syslinereader::SyslineReader::insert_sysline::h594b7fc547658a04` | 3 | `gen-1000-3-foobar-noyear.log` | 500 | 146,720 (143.28 KiB) | 293 (293 B) |
| `src/readers/syslinereader.rs:1011:9`<br/>`s4lib::readers::syslinereader::SyslineReader::insert_sysline::h594b7fc547658a04` | 3 | `gen-1000-3-foobar-noyear.log` | 500 | 102,720 (100.31 KiB) | 205 (205 B) |
| `src/readers/linereader.rs:482:9`<br/>`s4lib::readers::linereader::LineReader::insert_line::h30f7de6a00b83b23` | 3 | `gen-1000-3-foobar-noyear.log` | 499 | 102,528 (100.12 KiB) | 205 (205 B) |
| `src/readers/linereader.rs:495:9`<br/>`s4lib::readers::linereader::LineReader::insert_line::h30f7de6a00b83b23` | 3 | `gen-1000-3-foobar-noyear.log` | 499 | 102,528 (100.12 KiB) | 205 (205 B) |
| `src/s4/s4.rs:3403:16`<br/>`s4::s4::cli_process_args::h88e0e2c6d5cbf9e2` | 1 | `main` | 208 | 25,492 (24.89 KiB) | 122 (122 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25::ENGINE_BYTES::exec::transition_epsilons_exec::hff33c926106dce95` | 3 | `gen-1000-3-foobar-noyear.log` | 145 | 17,808 (17.39 KiB) | 122 (122 B) |
| `src/s4/s4.rs:2499:10`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 52 | 48,546 (47.41 KiB) | 933 (933 B) |
| `src/readers/syslinereader.rs:553:13`<br/>`s4lib::readers::syslinereader::SyslineReader::new::hfb59a22ff3184b15` | 3 | `gen-1000-3-foobar-noyear.log` | 29 | 5,952 (5.81 KiB) | 205 (205 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32::ENGINE_BYTES::exec::transition_epsilons_exec::hdc19b96702693826` | 3 | `gen-1000-3-foobar-noyear.log` | 26 | 140,592 (137.30 KiB) | 5,407 (5.28 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27::ENGINE_BYTES::exec::transition_epsilons_exec::h226f65d715647fe2` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 39,600 (38.67 KiB) | 1,650 (1.61 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29::ENGINE_BYTES::exec::transition_epsilons_exec::hf4eaa7fdb29665c5` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 39,024 (38.11 KiB) | 1,626 (1.59 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28::ENGINE_BYTES::exec::transition_epsilons_exec::hdfe5cdb4455a0ec6` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 39,024 (38.11 KiB) | 1,626 (1.59 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30::ENGINE_BYTES::exec::transition_epsilons_exec::hc34a350c4fa072a2` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 39,024 (38.11 KiB) | 1,626 (1.59 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31::ENGINE_BYTES::exec::transition_epsilons_exec::h8829c4a31a8aff20` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 34,992 (34.17 KiB) | 1,458 (1.42 KiB) |
| `src/readers/syslinereader.rs:1059:44`<br/>`s4lib::readers::syslinereader::SyslineReader::drop_data::h44e3dde208c9aecf` | 3 | `gen-1000-3-foobar-noyear.log` | 22 | 278,000 (271.48 KiB) | 12,636 (12.34 KiB) |
| `src/readers/blockreader.rs:2775:26`<br/>`s4lib::readers::blockreader::BlockReader::read_block_File::h2724c37489099d5b` | 3 | `gen-1000-3-foobar-noyear.log` | 14 | 894,569 (873.60 KiB) | 63,897 (62.40 KiB) |
| `src/readers/blockreader.rs:2796:30`<br/>`s4lib::readers::blockreader::BlockReader::read_block_File::h2724c37489099d5b` | 3 | `gen-1000-3-foobar-noyear.log` | 14 | 560 (560 B) | 40 (40 B) |
| `src/data/line.rs:597:9`<br/>`s4lib::data::line::Line::prepend::hf2efc19b868993bf` | 3 | `gen-1000-3-foobar-noyear.log` | 13 | 2,496 (2.44 KiB) | 192 (192 B) |
| `src/readers/syslinereader.rs:1824:27`<br/>`s4lib::readers::syslinereader::SyslineReader::parse_datetime_in_line_cached::ha2ccb3c0af421bb0` | 3 | `gen-1000-3-foobar-noyear.log` | 11 | 704 (704 B) | 64 (64 B) |
| `src/s4/s4.rs:3748:5`<br/>`s4::s4::set_signal_handler::h125013a81a433b00` | 1 | `main` | 7 | 181 (181 B) | 25 (25 B) |
| `src/s4/s4.rs:5242:15`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 5 | 293 (293 B) | 58 (58 B) |
| `src/readers/syslinereader.rs:1571:17`<br/>`s4lib::readers::syslinereader::SyslineReader::dt_patterns_indexes_refresh::h7d62d40024924a2f` | 3 | `gen-1000-3-foobar-noyear.log` | 4 | 8,608 (8.41 KiB) | 2,152 (2.10 KiB) |
| `src/readers/syslinereader.rs:2550:13`<br/>`s4lib::readers::syslinereader::SyslineReader::find_sysline_year::he654892adaa19d71` | 3 | `gen-1000-3-foobar-noyear.log` | 4 | 192 (192 B) | 48 (48 B) |
| `src/readers/blockreader.rs:2689:9`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_LRU_cache::h5305638563a478ee` | 3 | `gen-1000-3-foobar-noyear.log` | 4 | 128 (128 B) | 32 (32 B) |
| `src/s4/s4.rs:353:13`<br/>`s4::s4::LOCAL_NOW::__init::{{closure}}::h9e9a6660e9e40986` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_4::ere_match_struct_4::ENGINE_BYTES::exec::transition_epsilons_exec::hf0ba5b43c127770b` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 912 (912 B) | 304 (304 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_10::ere_match_struct_10::ENGINE_BYTES::exec::transition_epsilons_exec::had2e008b10c38c40` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 848 (848 B) | 282 (282 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_24::ere_match_struct_24::ENGINE_BYTES::exec::transition_epsilons_exec::h40a9a670496fcfc4` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 832 (832 B) | 277 (277 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_18::ere_match_struct_18::ENGINE_BYTES::exec::transition_epsilons_exec::h5843151b6affe98e` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 816 (816 B) | 272 (272 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_22::ere_match_struct_22::ENGINE_BYTES::exec::transition_epsilons_exec::hdb4b22d405b9be88` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 816 (816 B) | 272 (272 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_14::ere_match_struct_14::ENGINE_BYTES::exec::transition_epsilons_exec::h50c5dad63541d70c` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 784 (784 B) | 261 (261 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_1::ere_match_struct_1::ENGINE_BYTES::exec::transition_epsilons_exec::h70d105e748753c60` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 768 (768 B) | 256 (256 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_3::ere_match_struct_3::ENGINE_BYTES::exec::transition_epsilons_exec::hd02562c1eedf6971` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 768 (768 B) | 256 (256 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_2::ere_match_struct_2::ENGINE_BYTES::exec::transition_epsilons_exec::hefaefa742077d291` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 768 (768 B) | 256 (256 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_5::ere_match_struct_5::ENGINE_BYTES::exec::transition_epsilons_exec::h6451e58bad0fe711` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 720 (720 B) | 240 (240 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_6::ere_match_struct_6::ENGINE_BYTES::exec::transition_epsilons_exec::h79a11391f5e4da06` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 720 (720 B) | 240 (240 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_8::ere_match_struct_8::ENGINE_BYTES::exec::transition_epsilons_exec::ha8d74dd4d5d41862` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 704 (704 B) | 234 (234 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_9::ere_match_struct_9::ENGINE_BYTES::exec::transition_epsilons_exec::h8faa9c8f05ea6491` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 704 (704 B) | 234 (234 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_7::ere_match_struct_7::ENGINE_BYTES::exec::transition_epsilons_exec::h1f1bf4b24803ec5d` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 704 (704 B) | 234 (234 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_26::ere_match_struct_26::ENGINE_BYTES::exec::transition_epsilons_exec::hfa923468facde263` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 704 (704 B) | 234 (234 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_21::ere_match_struct_21::ENGINE_BYTES::exec::transition_epsilons_exec::h00df9ef981f6ab5d` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 688 (688 B) | 229 (229 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_20::ere_match_struct_20::ENGINE_BYTES::exec::transition_epsilons_exec::heea13f2f95ce4aba` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 688 (688 B) | 229 (229 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_17::ere_match_struct_17::ENGINE_BYTES::exec::transition_epsilons_exec::h092df1784048b53e` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 688 (688 B) | 229 (229 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_16::ere_match_struct_16::ENGINE_BYTES::exec::transition_epsilons_exec::he9ebb4eae0d08835` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 688 (688 B) | 229 (229 B) |
| `src/readers/blockreader.rs:2708:28`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_storage::hc30d4f1ef8950ec7` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 672 (672 B) | 224 (224 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_11::ere_match_struct_11::ENGINE_BYTES::exec::transition_epsilons_exec::h46308bdc74abb99f` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 640 (640 B) | 213 (213 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_19::ere_match_struct_19::ENGINE_BYTES::exec::transition_epsilons_exec::h9d8756d07336793f` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 624 (624 B) | 208 (208 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_12::ere_match_struct_12::ENGINE_BYTES::exec::transition_epsilons_exec::h40c0be9e02a2c28b` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 576 (576 B) | 192 (192 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_15::ere_match_struct_15::ENGINE_BYTES::exec::transition_epsilons_exec::h8091017b03ca32dc` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 576 (576 B) | 192 (192 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_13::ere_match_struct_13::ENGINE_BYTES::exec::transition_epsilons_exec::hfa04cc33ad8fb915` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 576 (576 B) | 192 (192 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_23::ere_match_struct_23::ENGINE_BYTES::exec::transition_epsilons_exec::ha68d766548e3c0d5` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 480 (480 B) | 160 (160 B) |
| `src/readers/syslinereader.rs:591:47`<br/>`s4lib::readers::syslinereader::SyslineReader::new::hfb59a22ff3184b15` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 416 (416 B) | 138 (138 B) |
| `src/readers/blockreader.rs:2722:14`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_storage::hc30d4f1ef8950ec7` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 408 (408 B) | 136 (136 B) |
| `src/readers/linereader.rs:273:34`<br/>`s4lib::readers::linereader::LineReader::new::hbd79e7e029f4fee5` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 384 (384 B) | 128 (128 B) |
| `src/readers/syslinereader.rs:584:37`<br/>`s4lib::readers::syslinereader::SyslineReader::new::hfb59a22ff3184b15` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 248 (248 B) | 82 (82 B) |
| `src/readers/blockreader.rs:1836:35`<br/>`s4lib::readers::blockreader::BlockReader::new::h981894b8168c10ac` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 216 (216 B) | 72 (72 B) |
| `src/readers/linereader.rs:2266:13`<br/>`s4lib::readers::linereader::LineReader::find_line::hbe0e0f8baa045431` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 144 (144 B) | 48 (48 B) |
| `src/s4/s4.rs:5238:13`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 2 | 3,800 (3.71 KiB) | 1,900 (1.86 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27::ENGINE_BYTES::exec::transition_symbols_exec::h91af42a154ffe963` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29::ENGINE_BYTES::exec::transition_symbols_exec::h2bc3d4aa491ff7e5` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30::ENGINE_BYTES::exec::transition_symbols_exec::h0992494176295abe` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28::ENGINE_BYTES::exec::transition_symbols_exec::h9e21b191b0fea16a` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32::ENGINE_BYTES::exec::transition_symbols_exec::h0191cf908bafbc23` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,264 (3.19 KiB) | 1,632 (1.59 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31::ENGINE_BYTES::exec::transition_symbols_exec::h53dd4374ec94577b` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,264 (3.19 KiB) | 1,632 (1.59 KiB) |
| `src/readers/syslinereader.rs:3079:9`<br/>`s4lib::readers::syslinereader::SyslineReader::summary::h30f8fbc66b42561c` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 384 (384 B) | 192 (192 B) |
| `src/readers/syslinereader.rs:3085:9`<br/>`s4lib::readers::syslinereader::SyslineReader::summary::h30f8fbc66b42561c` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 384 (384 B) | 192 (192 B) |
| `src/s4/s4.rs:2834:16`<br/>`s4::s4::cli_process_blocksz::h51454ca5305bb990` | 1 | `main` | 2 | 156 (156 B) | 78 (78 B) |
| `src/s4/s4.rs:3914:11`<br/>`s4::s4::chan_send::hbf90949ec8f45664` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 144 (144 B) | 72 (72 B) |
| `src/s4/s4.rs:5359:59`<br/>`s4::s4::processing_loop::recv_many_chan::hcf11e7434d98ce19` | 1 | `main` | 2 | 144 (144 B) | 72 (72 B) |
| `src/readers/syslinereader.rs:3086:13`<br/>`s4lib::readers::syslinereader::SyslineReader::summary::h30f8fbc66b42561c` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 128 (128 B) | 64 (64 B) |
| `src/readers/syslinereader.rs:2275:13`<br/>`s4lib::readers::syslinereader::SyslineReader::find_sysline_in_block_year::h99505e21781aa7ce` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 96 (96 B) | 48 (48 B) |
| `src/readers/linereader.rs:1313:29`<br/>`s4lib::readers::linereader::LineReader::find_line_in_block::h09a67eff25dae940` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 96 (96 B) | 48 (48 B) |
| `src/readers/syslogprocessor.rs:1466:20`<br/>`s4lib::readers::syslogprocessor::SyslogProcessor::summary_complete::h2958cdc585a2f35c` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 94 (94 B) | 47 (47 B) |
| `src/s4/s4.rs:5172:13`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 2 | 94 (94 B) | 47 (47 B) |
| `src/s4/s4.rs:5129:34`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 2,452 (2.39 KiB) | 2,452 (2.39 KiB) |
| `src/printer/printers.rs:793:21`<br/>`s4lib::printer::printers::PrinterLogMessage::new::had48950bffa1c119` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25::ENGINE_BYTES::exec::transition_symbols_exec::hcb7757f20312f3d0` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 1,472 (1.44 KiB) | 1,472 (1.44 KiB) |
| `src/readers/syslinereader.rs:550:39`<br/>`s4lib::readers::syslinereader::SyslineReader::new::hfb59a22ff3184b15` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 1,424 (1.39 KiB) | 1,424 (1.39 KiB) |
| `src/s4/s4.rs:5527:29`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 1,248 (1.22 KiB) | 1,248 (1.22 KiB) |
| `src/printer/printers.rs:758:22`<br/>`s4lib::printer::printers::PrinterLogMessage::new::had48950bffa1c119` | 1 | `main` | 1 | 1,024 (1.00 KiB) | 1,024 (1.00 KiB) |
| `src/s4/s4.rs:5407:34`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 884 (884 B) | 884 (884 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_24::ere_match_struct_24::ENGINE_BYTES::exec::ha0d11f81dfa37e0e` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 800 (800 B) | 800 (800 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_4::ere_match_struct_4::ENGINE_BYTES::exec::h37cc0c24d2175806` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_2::ere_match_struct_2::ENGINE_BYTES::exec::hc39b88b30478ffd0` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_3::ere_match_struct_3::ENGINE_BYTES::exec::h2c64622e87737317` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_1::ere_match_struct_1::ENGINE_BYTES::exec::h6399d0f900e66e35` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25::ENGINE_BYTES::exec::h439bdc7b260f5fb7` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_8::ere_match_struct_8::ENGINE_BYTES::exec::hfefc575a1ff333bc` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_5::ere_match_struct_5::ENGINE_BYTES::exec::hc7c71b23dc5de80e` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_6::ere_match_struct_6::ENGINE_BYTES::exec::hd37bf73c766d7ad2` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_9::ere_match_struct_9::ENGINE_BYTES::exec::h8d9a5d36b93b7b94` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_26::ere_match_struct_26::ENGINE_BYTES::exec::ha9463d1b8390a208` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_10::ere_match_struct_10::ENGINE_BYTES::exec::h86c7629dc1189eeb` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_7::ere_match_struct_7::ENGINE_BYTES::exec::h54719030062ce62c` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28::ENGINE_BYTES::exec::he9936056ee92e716` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_21::ere_match_struct_21::ENGINE_BYTES::exec::hc204b521042dbf66` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_11::ere_match_struct_11::ENGINE_BYTES::exec::hbf3f4ae66332fbfc` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_20::ere_match_struct_20::ENGINE_BYTES::exec::h19c6de46316ff205` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_17::ere_match_struct_17::ENGINE_BYTES::exec::hee146b17da210559` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29::ENGINE_BYTES::exec::h7ab69500b9ef7e73` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_18::ere_match_struct_18::ENGINE_BYTES::exec::h3e83ea0e1169c347` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_14::ere_match_struct_14::ENGINE_BYTES::exec::h81ecacd7c75d7f67` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_22::ere_match_struct_22::ENGINE_BYTES::exec::h2de8fae816b1c8ad` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30::ENGINE_BYTES::exec::hff307d1cb4d9c6a5` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_16::ere_match_struct_16::ENGINE_BYTES::exec::hb47b8ee8aa22ad0d` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27::ENGINE_BYTES::exec::hf83be952d5a37f30` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31::ENGINE_BYTES::exec::h7f176b8a4cd6577b` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_15::ere_match_struct_15::ENGINE_BYTES::exec::h10dc44225f50882f` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32::ENGINE_BYTES::exec::h700cdadf73388bec` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_13::ere_match_struct_13::ENGINE_BYTES::exec::hc2c039f482d32594` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_19::ere_match_struct_19::ENGINE_BYTES::exec::h2efae9500323498a` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_12::ere_match_struct_12::ENGINE_BYTES::exec::h12f07bd0b6a90551` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:1838:9`<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl::GROUP_NAMES_MAP_STR as core::ops::deref::Deref>::deref::__static_ref_initialize::ha2143ce49d1007ce` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 416 (416 B) | 416 (416 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_23::ere_match_struct_23::ENGINE_BYTES::exec::h52df5da4945b9a96` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 416 (416 B) | 416 (416 B) |
| `src/s4/s4.rs:5044:17`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 368 (368 B) | 368 (368 B) |
| `src/readers/syslogprocessor.rs:189:9`<br/>`<s4lib::readers::syslogprocessor::BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP as core::ops::deref::Deref>::deref::__static_ref_initialize::hd21ab7038f03a3f4` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 280 (280 B) | 280 (280 B) |
| `src/readers/syslogprocessor.rs:174:9`<br/>`<s4lib::readers::syslogprocessor::BLOCKZERO_ANALYSIS_LINE_COUNT_MIN_MAP as core::ops::deref::Deref>::deref::__static_ref_initialize::he407f09919983604` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:5240:9`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:4929:34`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 276 (276 B) | 276 (276 B) |
| `src/s4/s4.rs:3685:51`<br/>`s4::s4::main::hbaf861000d5581ed` | 1 | `main` | 1 | 224 (224 B) | 224 (224 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:1834:9`<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl::GROUP_NAMES_MAP_STR as core::ops::deref::Deref>::deref::__static_ref_initialize::ha2143ce49d1007ce` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 216 (216 B) | 216 (216 B) |
| `src/s4/s4.rs:4935:49`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:1831:9`<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl::GROUP_NAMES_MAP_STR as core::ops::deref::Deref>::deref::__static_ref_initialize::ha2143ce49d1007ce` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 116 (116 B) | 116 (116 B) |
| `src/s4/s4.rs:4937:40`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 116 (116 B) | 116 (116 B) |
| `src/readers/filepreprocessor.rs:1137:22`<br/>`s4lib::readers::filepreprocessor::process_path::hbbaea7b4727b31bc` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| `src/s4/s4.rs:2545:12`<br/>`<s4::s4::CLI_Args as clap_builder::derive::FromArgMatches>::from_arg_matches_mut::{{closure}}::h961885fdfcc3c815` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| `src/s4/s4.rs:4954:41`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5127:32`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5139:28`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4950:44`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4952:35`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/readers/filepreprocessor.rs:1167:53`<br/>`s4lib::readers::filepreprocessor::process_path::hbbaea7b4727b31bc` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| `src/s4/s4.rs:5397:26`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5706:62`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5413:52`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/readers/linereader.rs:1129:21`<br/>`s4lib::readers::linereader::LineReader::find_line_in_block::h09a67eff25dae940` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 48 (48 B) | 48 (48 B) |
| `src/readers/linereader.rs:759:17`<br/>`s4lib::readers::linereader::LineReader::check_store::h111e14efc71959e6` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 48 (48 B) | 48 (48 B) |
| `src/readers/linereader.rs:742:21`<br/>`s4lib::readers::linereader::LineReader::check_store::h111e14efc71959e6` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:3421:33`<br/>`s4::s4::cli_process_args::h88e0e2c6d5cbf9e2` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/readers/blockreader.rs:619:35`<br/>`s4lib::readers::blockreader::BlockReader::new::h981894b8168c10ac` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| `src/readers/blockreader.rs:651:20`<br/>`s4lib::readers::blockreader::BlockReader::new::h981894b8168c10ac` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| `src/s4/s4.rs:3962:9`<br/>`s4::s4::exec_syslogprocessor::he009d79440229472` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| `src/s4/s4.rs:3448:29`<br/>`s4::s4::cli_process_args::h88e0e2c6d5cbf9e2` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| `src/s4/s4.rs:5044:56`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| `src/readers/filepreprocessor.rs:1167:87`<br/>`s4lib::readers::filepreprocessor::process_path::hbbaea7b4727b31bc` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| `src/readers/filepreprocessor.rs:1133:33`<br/>`s4lib::readers::filepreprocessor::process_path::hbbaea7b4727b31bc` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| `src/s4/s4.rs:2897:5`<br/>`s4::s4::cli_process_tz_offset::h89478d5e94c24d33` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| `src/s4/s4.rs:5241:32`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 28 (28 B) | 28 (28 B) |
| `src/s4/s4.rs:5243:19`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 28 (28 B) | 28 (28 B) |
| `src/readers/helpers.rs:30:5`<br/>`s4lib::readers::helpers::basename::h750318f35665bb80` | 1 | `main` | 1 | 28 (28 B) | 28 (28 B) |
| `src/readers/syslinereader.rs:1384:34`<br/>`s4lib::readers::syslinereader::SyslineReader::find_datetime_in_line::h22262d5ac6b8288e` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 22 (22 B) | 22 (22 B) |
| `src/s4/s4.rs:2896:28`<br/>`s4::s4::cli_process_tz_offset::h89478d5e94c24d33` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| `src/data/line.rs:360:9`<br/>`s4lib::data::line::LinePart::block_boxptr_a::he80bfe951fa07f79` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 16 (16 B) | 16 (16 B) |
| `src/data/line.rs:381:9`<br/>`s4lib::data::line::LinePart::block_boxptr_b::hc21a66aae5668e99` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2759:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2682:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2796:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2747:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2557:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2600:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2648:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2694:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2586:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2568:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2782:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2614:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2534:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2672:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2547:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2735:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2660:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2720:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2630:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2708:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/printer/printers.rs:760:28`<br/>`s4lib::printer::printers::PrinterLogMessage::new::had48950bffa1c119` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| `src/readers/syslinereader.rs:580:31`<br/>`s4lib::readers::syslinereader::SyslineReader::new::hfb59a22ff3184b15` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2869:23`<br/>`s4::s4::cli_parse_blocksz::h07da46be8c72d7a9` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5229:26`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5424:26`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2790:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h79b5fc76fb20f99e` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2581:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h92423ea634403cb1` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2754:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hfec2c6f7a4ac335d` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2868:32`<br/>`s4::s4::cli_parse_blocksz::h07da46be8c72d7a9` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2730:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h542fcb16d362eec7` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2772:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hf237768366615430` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2790:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h79b5fc76fb20f99e` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2715:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h84f6e1bf9a3cfbc0` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2742:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h0556509c3310021a` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/readers/filepreprocessor.rs:334:31`<br/>`s4lib::readers::filepreprocessor::pathbuf_to_filetype_impl::hd266c5704895e843` | 1 | `main` | 1 | 3 (3 B) | 3 (3 B) |
| `src/s4/s4.rs:2687:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hacbe28393d46c9c2` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| `src/s4/s4.rs:2687:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hacbe28393d46c9c2` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |



## Allocator Tracking summary

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| normal allocations | 93,256,605 | 126,191 | normal program allocations; this is the most useful number |
| total deallocations | 282,433,140 | 694,425 | includes normal program deallocations and tracking deallocations |
| current outstanding | 66,988,477 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| total from tracking | 256,164,989 | 666,219 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 254,445,572 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 1,719,417 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 36 | 100 to 19 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| parameter | value | about |
| :--- | ---: | :--- |
| frame depth | 1 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 203 | entries in the table above |
| cached file names | 12 | |
| cached function names | 130 | |
| cached thread names | 2 | |

2026-05-28 23:04:48.920331110 -07:00
