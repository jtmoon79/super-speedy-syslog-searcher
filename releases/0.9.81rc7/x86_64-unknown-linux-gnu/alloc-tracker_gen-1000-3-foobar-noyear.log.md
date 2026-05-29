# Command

`$ s4  ./logs/other/tests/gen-1000-3-foobar-noyear.log`

## Allocator Tracking results

| ***File:line:col***<br/>***Call Site*** | Thread<br/>ID | Thread<br/>Name | Allocations | Bytes | Bytes<br/>per Allocation |
| :-- | ---: | :--- | ---: | ---: | ---: |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33::ENGINE_BYTES::exec::transition_epsilons_exec::hca835e4891ca8795` | 3 | `gen-1000-3-foobar-noyear.log` | 87,174 | 95,374,368 (90.96 MiB) | 1,094 (1.07 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33::ENGINE_BYTES::exec::transition_symbols_exec::hd07132c343b2e4be` | 3 | `gen-1000-3-foobar-noyear.log` | 6,012 | 7,502,976 (7.16 MiB) | 1,248 (1.22 KiB) |
| `src/data/line.rs:423:9`<br/>`s4lib::data::line::LinePart::block_boxptr_ab::hec527f9f26121cff` | 3 | `gen-1000-3-foobar-noyear.log` | 3,037 | 48,592 (47.45 KiB) | 16 (16 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_33::ere_match_struct_33::ENGINE_BYTES::exec::hf60205f4b869d961` | 3 | `gen-1000-3-foobar-noyear.log` | 3,006 | 1,250,496 (1.19 MiB) | 416 (416 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_33::hc2caaa680a18fb37` | 3 | `gen-1000-3-foobar-noyear.log` | 3,006 | 360,720 (352.27 KiB) | 120 (120 B) |
| `src/readers/syslinereader.rs:1747:13`<br/>`s4lib::readers::syslinereader::SyslineReader::parse_datetime_in_line::hf868aae741c5f3b7` | 3 | `gen-1000-3-foobar-noyear.log` | 3,006 | 28,296 (27.63 KiB) | 9 (9 B) |
| `src/s4/s4.rs:5330:53`<br/>`s4::s4::processing_loop::recv_many_chan::h4d40e21e69805ee0` | 1 | `main` | 3,005 | 384,640 (375.62 KiB) | 128 (128 B) |
| `src/readers/syslinereader.rs:1003:34`<br/>`s4lib::readers::syslinereader::SyslineReader::insert_sysline::hb0a3347079ef1f13` | 3 | `gen-1000-3-foobar-noyear.log` | 3,005 | 216,360 (211.29 KiB) | 72 (72 B) |
| `src/data/sysline.rs:144:20`<br/>`s4lib::data::sysline::Sysline::new_no_lines::h5734abe02afe5ad5` | 3 | `gen-1000-3-foobar-noyear.log` | 3,005 | 24,040 (23.48 KiB) | 8 (8 B) |
| `src/data/line.rs:513:24`<br/>`<s4lib::data::line::Line as core::default::Default>::default::h718aa466535c25c4` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 144,144 (140.77 KiB) | 48 (48 B) |
| `src/readers/linereader.rs:473:28`<br/>`s4lib::readers::linereader::LineReader::insert_line::he3485a08fab6fb11` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 120,120 (117.30 KiB) | 40 (40 B) |
| `src/data/datetime.rs:1504:46`<br/>`s4lib::data::datetime::captures_to_buffer_bytes::h2b318c62d98c6c54` | 3 | `gen-1000-3-foobar-noyear.log` | 3,003 | 24,024 (23.46 KiB) | 8 (8 B) |
| `src/readers/syslinereader.rs:1026:9`<br/>`s4lib::readers::syslinereader::SyslineReader::insert_sysline::hb0a3347079ef1f13` | 3 | `gen-1000-3-foobar-noyear.log` | 500 | 146,720 (143.28 KiB) | 293 (293 B) |
| `src/readers/syslinereader.rs:1011:9`<br/>`s4lib::readers::syslinereader::SyslineReader::insert_sysline::hb0a3347079ef1f13` | 3 | `gen-1000-3-foobar-noyear.log` | 500 | 102,720 (100.31 KiB) | 205 (205 B) |
| `src/readers/linereader.rs:482:9`<br/>`s4lib::readers::linereader::LineReader::insert_line::he3485a08fab6fb11` | 3 | `gen-1000-3-foobar-noyear.log` | 499 | 102,528 (100.12 KiB) | 205 (205 B) |
| `src/readers/linereader.rs:495:9`<br/>`s4lib::readers::linereader::LineReader::insert_line::he3485a08fab6fb11` | 3 | `gen-1000-3-foobar-noyear.log` | 499 | 102,528 (100.12 KiB) | 205 (205 B) |
| `src/s4/s4.rs:3403:16`<br/>`s4::s4::cli_process_args::hc2513eaa31d17c92` | 1 | `main` | 208 | 25,490 (24.89 KiB) | 122 (122 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25::ENGINE_BYTES::exec::transition_epsilons_exec::hf431b243c46c3a8d` | 3 | `gen-1000-3-foobar-noyear.log` | 145 | 58,416 (57.05 KiB) | 402 (402 B) |
| `src/s4/s4.rs:2499:10`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 52 | 48,546 (47.41 KiB) | 933 (933 B) |
| `src/readers/syslinereader.rs:553:13`<br/>`s4lib::readers::syslinereader::SyslineReader::new::ha5ee0ef8a85160d8` | 3 | `gen-1000-3-foobar-noyear.log` | 29 | 5,952 (5.81 KiB) | 205 (205 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32::ENGINE_BYTES::exec::transition_epsilons_exec::ha6795349b9542e4c` | 3 | `gen-1000-3-foobar-noyear.log` | 26 | 153,264 (149.67 KiB) | 5,894 (5.76 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27::ENGINE_BYTES::exec::transition_epsilons_exec::h226aaae2ecb2a8df` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 48,132 (47.00 KiB) | 2,005 (1.96 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28::ENGINE_BYTES::exec::transition_epsilons_exec::hec6bcd7da6cd9c0b` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 43,326 (42.31 KiB) | 1,805 (1.76 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29::ENGINE_BYTES::exec::transition_epsilons_exec::h494cbb3f9716482b` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 43,308 (42.29 KiB) | 1,804 (1.76 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30::ENGINE_BYTES::exec::transition_epsilons_exec::h8a54c7641e50f51d` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 43,272 (42.26 KiB) | 1,803 (1.76 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31::ENGINE_BYTES::exec::transition_epsilons_exec::h0de51b7091396c66` | 3 | `gen-1000-3-foobar-noyear.log` | 24 | 39,042 (38.13 KiB) | 1,626 (1.59 KiB) |
| `src/readers/syslinereader.rs:1059:44`<br/>`s4lib::readers::syslinereader::SyslineReader::drop_data::he8d16442e1738fd7` | 3 | `gen-1000-3-foobar-noyear.log` | 22 | 278,000 (271.48 KiB) | 12,636 (12.34 KiB) |
| `src/readers/blockreader.rs:2775:26`<br/>`s4lib::readers::blockreader::BlockReader::read_block_File::h7b2ca2ec4ce957ae` | 3 | `gen-1000-3-foobar-noyear.log` | 14 | 894,569 (873.60 KiB) | 63,897 (62.40 KiB) |
| `src/readers/blockreader.rs:2796:30`<br/>`s4lib::readers::blockreader::BlockReader::read_block_File::h7b2ca2ec4ce957ae` | 3 | `gen-1000-3-foobar-noyear.log` | 14 | 560 (560 B) | 40 (40 B) |
| `src/data/line.rs:597:9`<br/>`s4lib::data::line::Line::prepend::h2349f6f866dd146f` | 3 | `gen-1000-3-foobar-noyear.log` | 13 | 2,496 (2.44 KiB) | 192 (192 B) |
| `src/readers/syslinereader.rs:1824:27`<br/>`s4lib::readers::syslinereader::SyslineReader::parse_datetime_in_line_cached::hf3fcdd8a4bda44a2` | 3 | `gen-1000-3-foobar-noyear.log` | 11 | 704 (704 B) | 64 (64 B) |
| `src/s4/s4.rs:3748:5`<br/>`s4::s4::set_signal_handler::h04d226b85c8bceb4` | 1 | `main` | 7 | 181 (181 B) | 25 (25 B) |
| `src/s4/s4.rs:5242:15`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 5 | 293 (293 B) | 58 (58 B) |
| `src/readers/syslinereader.rs:1571:17`<br/>`s4lib::readers::syslinereader::SyslineReader::dt_patterns_indexes_refresh::hc42f1427612aca0d` | 3 | `gen-1000-3-foobar-noyear.log` | 4 | 8,608 (8.41 KiB) | 2,152 (2.10 KiB) |
| `src/readers/syslinereader.rs:2550:13`<br/>`s4lib::readers::syslinereader::SyslineReader::find_sysline_year::hc512a80a0e5e0c7a` | 3 | `gen-1000-3-foobar-noyear.log` | 4 | 192 (192 B) | 48 (48 B) |
| `src/readers/blockreader.rs:2689:9`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_LRU_cache::h9597576df80bd0c6` | 3 | `gen-1000-3-foobar-noyear.log` | 4 | 128 (128 B) | 32 (32 B) |
| `src/s4/s4.rs:353:13`<br/>`s4::s4::LOCAL_NOW::__init::{{closure}}::h7cd5d76dd27ff375` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_22::ere_match_struct_22::ENGINE_BYTES::exec::transition_epsilons_exec::h84fc79cbc8b5dae3` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 2,272 (2.22 KiB) | 757 (757 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_18::ere_match_struct_18::ENGINE_BYTES::exec::transition_epsilons_exec::hafb9394e9747bd48` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 2,272 (2.22 KiB) | 757 (757 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_4::ere_match_struct_4::ENGINE_BYTES::exec::transition_epsilons_exec::h2a02f762b80f852e` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 2,036 (1.99 KiB) | 678 (678 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_10::ere_match_struct_10::ENGINE_BYTES::exec::transition_epsilons_exec::h69adf72a2e171bef` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 1,998 (1.95 KiB) | 666 (666 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_14::ere_match_struct_14::ENGINE_BYTES::exec::transition_epsilons_exec::h05cefb05a960f767` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 1,890 (1.85 KiB) | 630 (630 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_20::ere_match_struct_20::ENGINE_BYTES::exec::transition_epsilons_exec::h78a25b65632be000` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 1,184 (1.16 KiB) | 394 (394 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_16::ere_match_struct_16::ENGINE_BYTES::exec::transition_epsilons_exec::h0cb294bdaa9d0201` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 1,184 (1.16 KiB) | 394 (394 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_21::ere_match_struct_21::ENGINE_BYTES::exec::transition_epsilons_exec::hf32772c5c1e20dc8` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 1,178 (1.15 KiB) | 392 (392 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_17::ere_match_struct_17::ENGINE_BYTES::exec::transition_epsilons_exec::hc416fba3e729afb5` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 1,178 (1.15 KiB) | 392 (392 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_19::ere_match_struct_19::ENGINE_BYTES::exec::transition_epsilons_exec::h6c0aa9313a549e10` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 1,092 (1.07 KiB) | 364 (364 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_24::ere_match_struct_24::ENGINE_BYTES::exec::transition_epsilons_exec::h418c269ed831d371` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 1,042 (1.02 KiB) | 347 (347 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_6::ere_match_struct_6::ENGINE_BYTES::exec::transition_epsilons_exec::hc043abf5605ebc5b` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 968 (968 B) | 322 (322 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_2::ere_match_struct_2::ENGINE_BYTES::exec::transition_epsilons_exec::h8bcfc96b132abc09` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 962 (962 B) | 320 (320 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_1::ere_match_struct_1::ENGINE_BYTES::exec::transition_epsilons_exec::hda5f7ad7c797a37d` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 960 (960 B) | 320 (320 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_3::ere_match_struct_3::ENGINE_BYTES::exec::transition_epsilons_exec::ha234499a6f5ce63e` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 956 (956 B) | 318 (318 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_5::ere_match_struct_5::ENGINE_BYTES::exec::transition_epsilons_exec::hcb315b983f1059fe` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 952 (952 B) | 317 (317 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_23::ere_match_struct_23::ENGINE_BYTES::exec::transition_epsilons_exec::ha1988cd0de631724` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 914 (914 B) | 304 (304 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_7::ere_match_struct_7::ENGINE_BYTES::exec::transition_epsilons_exec::h75312d29cf1c9b84` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 910 (910 B) | 303 (303 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_8::ere_match_struct_8::ENGINE_BYTES::exec::transition_epsilons_exec::hb3a3b478ba609b59` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 908 (908 B) | 302 (302 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_9::ere_match_struct_9::ENGINE_BYTES::exec::transition_epsilons_exec::hb07b395537211cfa` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 904 (904 B) | 301 (301 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_26::ere_match_struct_26::ENGINE_BYTES::exec::transition_epsilons_exec::h2e91a74fe174210d` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 864 (864 B) | 288 (288 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_11::ere_match_struct_11::ENGINE_BYTES::exec::transition_epsilons_exec::h2a1e5dfde2ff353a` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 816 (816 B) | 272 (272 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_12::ere_match_struct_12::ENGINE_BYTES::exec::transition_epsilons_exec::h2dff066ef2412335` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 716 (716 B) | 238 (238 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_13::ere_match_struct_13::ENGINE_BYTES::exec::transition_epsilons_exec::h5ffd5ce64b9163f3` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 710 (710 B) | 236 (236 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_15::ere_match_struct_15::ENGINE_BYTES::exec::transition_epsilons_exec::hc417f005ef64c539` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 708 (708 B) | 236 (236 B) |
| `src/readers/blockreader.rs:2708:28`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_storage::hdaa8db484d68fd3e` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 672 (672 B) | 224 (224 B) |
| `src/readers/syslinereader.rs:591:47`<br/>`s4lib::readers::syslinereader::SyslineReader::new::ha5ee0ef8a85160d8` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 416 (416 B) | 138 (138 B) |
| `src/readers/blockreader.rs:2722:14`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_storage::hdaa8db484d68fd3e` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 408 (408 B) | 136 (136 B) |
| `src/readers/linereader.rs:273:34`<br/>`s4lib::readers::linereader::LineReader::new::h35461f372328eb57` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 384 (384 B) | 128 (128 B) |
| `src/readers/syslinereader.rs:584:37`<br/>`s4lib::readers::syslinereader::SyslineReader::new::ha5ee0ef8a85160d8` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 248 (248 B) | 82 (82 B) |
| `src/readers/blockreader.rs:1836:35`<br/>`s4lib::readers::blockreader::BlockReader::new::h4ce67b93fa8e3683` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 216 (216 B) | 72 (72 B) |
| `src/readers/linereader.rs:2266:13`<br/>`s4lib::readers::linereader::LineReader::find_line::h86f8d51ac7804e43` | 3 | `gen-1000-3-foobar-noyear.log` | 3 | 144 (144 B) | 48 (48 B) |
| `src/s4/s4.rs:5238:13`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 2 | 3,800 (3.71 KiB) | 1,900 (1.86 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27::ENGINE_BYTES::exec::transition_symbols_exec::h069b09425fb4fc9e` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29::ENGINE_BYTES::exec::transition_symbols_exec::h4c1a1aeeadd44ec1` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30::ENGINE_BYTES::exec::transition_symbols_exec::hc9323080d918454b` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28::ENGINE_BYTES::exec::transition_symbols_exec::hd549aba0aceee3e6` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,648 (3.56 KiB) | 1,824 (1.78 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32::ENGINE_BYTES::exec::transition_symbols_exec::he90017eba3fb6699` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,264 (3.19 KiB) | 1,632 (1.59 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31::ENGINE_BYTES::exec::transition_symbols_exec::hc6b9f6aae3a4a803` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 3,264 (3.19 KiB) | 1,632 (1.59 KiB) |
| `src/readers/syslinereader.rs:3085:9`<br/>`s4lib::readers::syslinereader::SyslineReader::summary::h9e5c4493202bd2db` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 384 (384 B) | 192 (192 B) |
| `src/readers/syslinereader.rs:3079:9`<br/>`s4lib::readers::syslinereader::SyslineReader::summary::h9e5c4493202bd2db` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 384 (384 B) | 192 (192 B) |
| `src/s4/s4.rs:2834:16`<br/>`s4::s4::cli_process_blocksz::hcc5a5593f13e2c9a` | 1 | `main` | 2 | 156 (156 B) | 78 (78 B) |
| `src/s4/s4.rs:3914:11`<br/>`s4::s4::chan_send::hd2431125fff1d7f8` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 144 (144 B) | 72 (72 B) |
| `src/s4/s4.rs:5359:59`<br/>`s4::s4::processing_loop::recv_many_chan::h4d40e21e69805ee0` | 1 | `main` | 2 | 144 (144 B) | 72 (72 B) |
| `src/readers/syslinereader.rs:3086:13`<br/>`s4lib::readers::syslinereader::SyslineReader::summary::h9e5c4493202bd2db` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 128 (128 B) | 64 (64 B) |
| `src/readers/syslinereader.rs:2275:13`<br/>`s4lib::readers::syslinereader::SyslineReader::find_sysline_in_block_year::haa1f8fb8b36f2fa7` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 96 (96 B) | 48 (48 B) |
| `src/readers/linereader.rs:1313:29`<br/>`s4lib::readers::linereader::LineReader::find_line_in_block::hc904805666cce2d4` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 96 (96 B) | 48 (48 B) |
| `src/readers/syslogprocessor.rs:1466:20`<br/>`s4lib::readers::syslogprocessor::SyslogProcessor::summary_complete::hfcc5018cbbf8a99a` | 3 | `gen-1000-3-foobar-noyear.log` | 2 | 94 (94 B) | 47 (47 B) |
| `src/s4/s4.rs:5172:13`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 2 | 94 (94 B) | 47 (47 B) |
| `src/s4/s4.rs:5129:34`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 2,452 (2.39 KiB) | 2,452 (2.39 KiB) |
| `src/printer/printers.rs:793:21`<br/>`s4lib::printer::printers::PrinterLogMessage::new::ha16f81626adf2867` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25::ENGINE_BYTES::exec::transition_symbols_exec::he9284ba922872101` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 1,472 (1.44 KiB) | 1,472 (1.44 KiB) |
| `src/readers/syslinereader.rs:550:39`<br/>`s4lib::readers::syslinereader::SyslineReader::new::ha5ee0ef8a85160d8` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 1,424 (1.39 KiB) | 1,424 (1.39 KiB) |
| `src/s4/s4.rs:5527:29`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 1,248 (1.22 KiB) | 1,248 (1.22 KiB) |
| `src/printer/printers.rs:758:22`<br/>`s4lib::printer::printers::PrinterLogMessage::new::ha16f81626adf2867` | 1 | `main` | 1 | 1,024 (1.00 KiB) | 1,024 (1.00 KiB) |
| `src/s4/s4.rs:5407:34`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 884 (884 B) | 884 (884 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_24::ere_match_struct_24::ENGINE_BYTES::exec::h674f8147e54178ca` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 800 (800 B) | 800 (800 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_1::ere_match_struct_1::ENGINE_BYTES::exec::h4e18a17cb9906248` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_4::ere_match_struct_4::ENGINE_BYTES::exec::h38e897cfc833e380` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_3::ere_match_struct_3::ENGINE_BYTES::exec::h31049fc93e837670` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_2::ere_match_struct_2::ENGINE_BYTES::exec::h137772a7aa038826` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_25::ere_match_struct_25::ENGINE_BYTES::exec::h776ba08f46d59977` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 736 (736 B) | 736 (736 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_9::ere_match_struct_9::ENGINE_BYTES::exec::hed7573b2ed3140b1` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_6::ere_match_struct_6::ENGINE_BYTES::exec::h054e109e89ec12fb` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_10::ere_match_struct_10::ENGINE_BYTES::exec::h9e2bcfa9dfa7a98e` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_8::ere_match_struct_8::ENGINE_BYTES::exec::hd7299a8fdafb791d` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_7::ere_match_struct_7::ENGINE_BYTES::exec::h3f7cd62d4c124a06` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_5::ere_match_struct_5::ENGINE_BYTES::exec::h30adb9b76b7f4679` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_26::ere_match_struct_26::ENGINE_BYTES::exec::hffd0706c1d576513` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 672 (672 B) | 672 (672 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_21::ere_match_struct_21::ENGINE_BYTES::exec::h8770246dae1ef557` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_22::ere_match_struct_22::ENGINE_BYTES::exec::h8819caab03015b07` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_28::ere_match_struct_28::ENGINE_BYTES::exec::h4c08c7e71bff4b5b` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_11::ere_match_struct_11::ENGINE_BYTES::exec::h98443c799f375653` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_16::ere_match_struct_16::ENGINE_BYTES::exec::h1c58c0c60b92fad7` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_14::ere_match_struct_14::ENGINE_BYTES::exec::h657f4a6ed857e4ef` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_18::ere_match_struct_18::ENGINE_BYTES::exec::h773a633871fcda7f` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_30::ere_match_struct_30::ENGINE_BYTES::exec::h22491ffd9bbe8fdb` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_27::ere_match_struct_27::ENGINE_BYTES::exec::h5ab219cbde215f72` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_20::ere_match_struct_20::ENGINE_BYTES::exec::h19292e9ba9a57084` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_29::ere_match_struct_29::ENGINE_BYTES::exec::hbacf30e78fe2329c` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_17::ere_match_struct_17::ENGINE_BYTES::exec::hfb202f91f820a3ef` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 608 (608 B) | 608 (608 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_12::ere_match_struct_12::ENGINE_BYTES::exec::h1ded05930f344926` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_15::ere_match_struct_15::ENGINE_BYTES::exec::h8032be8a17cd6e7b` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_13::ere_match_struct_13::ENGINE_BYTES::exec::h9295f3f1048eb70c` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_31::ere_match_struct_31::ENGINE_BYTES::exec::hbc82d0ef105d73e6` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_19::ere_match_struct_19::ENGINE_BYTES::exec::h2d78d8bebe3e9b88` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_32::ere_match_struct_32::ENGINE_BYTES::exec::h6828b38d5a70fded` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 544 (544 B) | 544 (544 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:1838:9`<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl::GROUP_NAMES_MAP_STR as core::ops::deref::Deref>::deref::__static_ref_initialize::hed6593d334a6f619` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 416 (416 B) | 416 (416 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_23::ere_match_struct_23::ENGINE_BYTES::exec::h047bf774c61511e0` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 416 (416 B) | 416 (416 B) |
| `src/s4/s4.rs:5044:17`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 368 (368 B) | 368 (368 B) |
| `src/readers/syslogprocessor.rs:189:9`<br/>`<s4lib::readers::syslogprocessor::BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP as core::ops::deref::Deref>::deref::__static_ref_initialize::h0672a2de9a87b4ff` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 280 (280 B) | 280 (280 B) |
| `src/readers/syslogprocessor.rs:174:9`<br/>`<s4lib::readers::syslogprocessor::BLOCKZERO_ANALYSIS_LINE_COUNT_MIN_MAP as core::ops::deref::Deref>::deref::__static_ref_initialize::hd3d24ad1fe727c40` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:5240:9`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:4929:34`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 276 (276 B) | 276 (276 B) |
| `src/s4/s4.rs:3685:51`<br/>`s4::s4::main::hd2a00718532cbbd2` | 1 | `main` | 1 | 224 (224 B) | 224 (224 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:1834:9`<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl::GROUP_NAMES_MAP_STR as core::ops::deref::Deref>::deref::__static_ref_initialize::hed6593d334a6f619` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 216 (216 B) | 216 (216 B) |
| `src/s4/s4.rs:4935:49`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:1831:9`<br/>`<super_speedy_syslog_searcher_ere_datetimes_impl::GROUP_NAMES_MAP_STR as core::ops::deref::Deref>::deref::__static_ref_initialize::hed6593d334a6f619` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 116 (116 B) | 116 (116 B) |
| `src/s4/s4.rs:4937:40`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 116 (116 B) | 116 (116 B) |
| `src/s4/s4.rs:2545:12`<br/>`<s4::s4::CLI_Args as clap_builder::derive::FromArgMatches>::from_arg_matches_mut::{{closure}}::h6703d1b21dd76a2a` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| `src/readers/filepreprocessor.rs:1137:22`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 94 (94 B) | 94 (94 B) |
| `src/s4/s4.rs:5127:32`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4952:35`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4954:41`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4950:44`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5139:28`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/readers/filepreprocessor.rs:1167:53`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| `src/s4/s4.rs:5413:52`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5397:26`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5706:62`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/readers/linereader.rs:1129:21`<br/>`s4lib::readers::linereader::LineReader::find_line_in_block::hc904805666cce2d4` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 48 (48 B) | 48 (48 B) |
| `src/readers/linereader.rs:742:21`<br/>`s4lib::readers::linereader::LineReader::check_store::h0bbcb4e9b17ca0fa` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 48 (48 B) | 48 (48 B) |
| `src/readers/linereader.rs:759:17`<br/>`s4lib::readers::linereader::LineReader::check_store::h0bbcb4e9b17ca0fa` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:3421:33`<br/>`s4::s4::cli_process_args::hc2513eaa31d17c92` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/readers/blockreader.rs:619:35`<br/>`s4lib::readers::blockreader::BlockReader::new::h4ce67b93fa8e3683` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| `src/readers/blockreader.rs:651:20`<br/>`s4lib::readers::blockreader::BlockReader::new::h4ce67b93fa8e3683` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| `src/s4/s4.rs:3962:9`<br/>`s4::s4::exec_syslogprocessor::h8106c39799c9915a` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 47 (47 B) | 47 (47 B) |
| `src/readers/filepreprocessor.rs:1133:33`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| `src/s4/s4.rs:3448:29`<br/>`s4::s4::cli_process_args::hc2513eaa31d17c92` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| `src/s4/s4.rs:5044:56`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| `src/readers/filepreprocessor.rs:1167:87`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 47 (47 B) | 47 (47 B) |
| `src/s4/s4.rs:2897:5`<br/>`s4::s4::cli_process_tz_offset::ha0fcee891a25dea7` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| `src/readers/helpers.rs:30:5`<br/>`s4lib::readers::helpers::basename::h70537354f6e5017d` | 1 | `main` | 1 | 28 (28 B) | 28 (28 B) |
| `src/s4/s4.rs:5241:32`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 28 (28 B) | 28 (28 B) |
| `src/s4/s4.rs:5243:19`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 28 (28 B) | 28 (28 B) |
| `src/readers/syslinereader.rs:1384:34`<br/>`s4lib::readers::syslinereader::SyslineReader::find_datetime_in_line::h76c64a27acc37abd` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 22 (22 B) | 22 (22 B) |
| `src/s4/s4.rs:2896:28`<br/>`s4::s4::cli_process_tz_offset::ha0fcee891a25dea7` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| `src/data/line.rs:360:9`<br/>`s4lib::data::line::LinePart::block_boxptr_a::hcae88c72609e351d` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 16 (16 B) | 16 (16 B) |
| `src/data/line.rs:381:9`<br/>`s4lib::data::line::LinePart::block_boxptr_b::hc317650bc7eee208` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2747:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2720:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2630:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2648:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2782:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2534:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2735:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2682:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2796:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2568:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2614:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2557:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2547:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2759:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2694:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2660:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2586:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2600:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2708:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2672:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/printer/printers.rs:760:28`<br/>`s4lib::printer::printers::PrinterLogMessage::new::ha16f81626adf2867` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| `src/readers/syslinereader.rs:580:31`<br/>`s4lib::readers::syslinereader::SyslineReader::new::ha5ee0ef8a85160d8` | 3 | `gen-1000-3-foobar-noyear.log` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5229:26`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2581:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h603c3d84d3cce851` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2869:23`<br/>`s4::s4::cli_parse_blocksz::hee67f7902ea5d88b` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2790:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h1b989b6cd2979b9a` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5424:26`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2772:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h1d839769a57605e6` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2754:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::habfe7a0fe57c7fe9` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2730:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hae6027573b2eba9b` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2790:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h1b989b6cd2979b9a` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2715:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::ha43c097dc248e1c4` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2868:32`<br/>`s4::s4::cli_parse_blocksz::hee67f7902ea5d88b` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2742:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::he56a849edb3bf699` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/readers/filepreprocessor.rs:334:31`<br/>`s4lib::readers::filepreprocessor::pathbuf_to_filetype_impl::h7ee028a2b403903e` | 1 | `main` | 1 | 3 (3 B) | 3 (3 B) |
| `src/s4/s4.rs:2687:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h7dbc7c85affda2f5` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| `src/s4/s4.rs:2687:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h7dbc7c85affda2f5` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |



## Allocator Tracking summary

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| normal allocations | 107,727,779 | 126,191 | normal program allocations; this is the most useful number |
| total deallocations | 296,479,831 | 622,726 | includes normal program deallocations and tracking deallocations |
| current outstanding | 67,769,035 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| total from tracking | 256,521,064 | 592,981 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 254,801,909 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 1,719,155 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 42 | 100 to 21 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| parameter | value | about |
| :--- | ---: | :--- |
| frame depth | 1 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 203 | entries in the table above |
| cached file names | 12 | |
| cached function names | 130 | |
| cached thread names | 2 | |


Generated on Wed May 27 12:23:28 PM PDT 2026 by `./tools/s4-alloc_trackers.sh`

