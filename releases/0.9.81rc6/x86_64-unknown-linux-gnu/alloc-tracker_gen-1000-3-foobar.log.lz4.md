+ exec env S4_ALLOC_TRACKER_DEPTH=1 S4_ALLOC_TRACKER_PRINT= S4_ALLOC_TRACKER_TRACKING=1 S4_BUILD_REGEX_PRINT=0 RUST_MIN_STACK=20000000 cargo run --quiet --profile alloc_tracker --features alloc_tracker -- ./logs/other/tests/gen-1000-3-foobar.log.lz4
## Allocator Tracking results

| ***File:line:col***<br/>***Call Site*** | Thread<br/>ID | Thread<br/>Name | Allocations | Bytes | Bytes<br/>per Allocation |
| :-- | ---: | :--- | ---: | ---: | ---: |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_123::ere_match_struct_123::ENGINE_BYTES::exec::transition_epsilons_exec::h537ddf892aab2ffe` | 3 | `gen-1000-3-foobar.log.lz4` | 66,365 | 3,664,630 (3.49 MiB) | 55 (55 B) |
| `src/readers/syslinereader.rs:1471:17`<br/>`s4lib::readers::syslinereader::SyslineReader::find_datetime_in_line::h76c64a27acc37abd` | 3 | `gen-1000-3-foobar.log.lz4` | 952 | 91,392 (89.25 KiB) | 96 (96 B) |
| `src/readers/syslinereader.rs:1469:13`<br/>`s4lib::readers::syslinereader::SyslineReader::find_datetime_in_line::h76c64a27acc37abd` | 3 | `gen-1000-3-foobar.log.lz4` | 952 | 22,848 (22.31 KiB) | 24 (24 B) |
| `src/data/line.rs:513:24`<br/>`<s4lib::data::line::Line as core::default::Default>::default::h718aa466535c25c4` | 3 | `gen-1000-3-foobar.log.lz4` | 477 | 22,896 (22.36 KiB) | 48 (48 B) |
| `src/readers/syslinereader.rs:1747:13`<br/>`s4lib::readers::syslinereader::SyslineReader::parse_datetime_in_line::hf868aae741c5f3b7` | 3 | `gen-1000-3-foobar.log.lz4` | 477 | 3,816 (3.73 KiB) | 8 (8 B) |
| `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs:2060:27`<br/>`super_speedy_syslog_searcher_ere_datetimes_impl::DATETIME_PARSE_DATAS::ere_match_fn_123::ere_match_struct_123::ENGINE_BYTES::exec::haf48fb9fed91f767` | 3 | `gen-1000-3-foobar.log.lz4` | 476 | 106,624 (104.12 KiB) | 224 (224 B) |
| `src/readers/linereader.rs:473:28`<br/>`s4lib::readers::linereader::LineReader::insert_line::he3485a08fab6fb11` | 3 | `gen-1000-3-foobar.log.lz4` | 476 | 19,040 (18.59 KiB) | 40 (40 B) |
| `src/data/line.rs:423:9`<br/>`s4lib::data::line::LinePart::block_boxptr_ab::hec527f9f26121cff` | 3 | `gen-1000-3-foobar.log.lz4` | 476 | 7,616 (7.44 KiB) | 16 (16 B) |
| `src/s4/s4.rs:3403:16`<br/>`s4::s4::cli_process_args::hc2513eaa31d17c92` | 1 | `main` | 208 | 25,481 (24.88 KiB) | 122 (122 B) |
| `src/readers/linereader.rs:482:9`<br/>`s4lib::readers::linereader::LineReader::insert_line::he3485a08fab6fb11` | 3 | `gen-1000-3-foobar.log.lz4` | 78 | 15,936 (15.56 KiB) | 204 (204 B) |
| `src/readers/linereader.rs:495:9`<br/>`s4lib::readers::linereader::LineReader::insert_line::he3485a08fab6fb11` | 3 | `gen-1000-3-foobar.log.lz4` | 78 | 15,936 (15.56 KiB) | 204 (204 B) |
| `src/s4/s4.rs:2499:10`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 52 | 48,546 (47.41 KiB) | 933 (933 B) |
| `src/readers/linereader.rs:1313:29`<br/>`s4lib::readers::linereader::LineReader::find_line_in_block::hc904805666cce2d4` | 3 | `gen-1000-3-foobar.log.lz4` | 7 | 336 (336 B) | 48 (48 B) |
| `src/s4/s4.rs:3748:5`<br/>`s4::s4::set_signal_handler::h04d226b85c8bceb4` | 1 | `main` | 7 | 181 (181 B) | 25 (25 B) |
| `src/s4/s4.rs:5242:15`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 5 | 290 (290 B) | 58 (58 B) |
| `src/s4/s4.rs:353:13`<br/>`s4::s4::LOCAL_NOW::__init::{{closure}}::h7cd5d76dd27ff375` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| `src/readers/syslinereader.rs:591:47`<br/>`s4lib::readers::syslinereader::SyslineReader::new::ha5ee0ef8a85160d8` | 3 | `gen-1000-3-foobar.log.lz4` | 3 | 416 (416 B) | 138 (138 B) |
| `src/readers/linereader.rs:273:34`<br/>`s4lib::readers::linereader::LineReader::new::h35461f372328eb57` | 3 | `gen-1000-3-foobar.log.lz4` | 3 | 384 (384 B) | 128 (128 B) |
| `src/readers/syslinereader.rs:584:37`<br/>`s4lib::readers::syslinereader::SyslineReader::new::ha5ee0ef8a85160d8` | 3 | `gen-1000-3-foobar.log.lz4` | 3 | 248 (248 B) | 82 (82 B) |
| `src/readers/blockreader.rs:1836:35`<br/>`s4lib::readers::blockreader::BlockReader::new::h4ce67b93fa8e3683` | 3 | `gen-1000-3-foobar.log.lz4` | 3 | 216 (216 B) | 72 (72 B) |
| `src/readers/blockreader.rs:1047:27`<br/>`s4lib::readers::blockreader::BlockReader::new::h4ce67b93fa8e3683` | 3 | `gen-1000-3-foobar.log.lz4` | 2 | 8,388,608 (8.00 MiB) | 4,194,304 (4.00 MiB) |
| `src/readers/blockreader.rs:3473:19`<br/>`s4lib::readers::blockreader::BlockReader::read_block_FileLz4::h9d26d5f04dcb1f2d` | 3 | `gen-1000-3-foobar.log.lz4` | 2 | 8,388,608 (8.00 MiB) | 4,194,304 (4.00 MiB) |
| `src/s4/s4.rs:5238:13`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 2 | 3,800 (3.71 KiB) | 1,900 (1.86 KiB) |
| `src/s4/s4.rs:5330:53`<br/>`s4::s4::processing_loop::recv_many_chan::h4d40e21e69805ee0` | 1 | `main` | 2 | 256 (256 B) | 128 (128 B) |
| `src/readers/filepreprocessor.rs:429:18`<br/>`s4lib::readers::filepreprocessor::pathbuf_to_filetype_impl::h7ee028a2b403903e` | 1 | `main` | 2 | 179 (179 B) | 89 (89 B) |
| `src/s4/s4.rs:2834:16`<br/>`s4::s4::cli_process_blocksz::hcc5a5593f13e2c9a` | 1 | `main` | 2 | 156 (156 B) | 78 (78 B) |
| `src/s4/s4.rs:5359:59`<br/>`s4::s4::processing_loop::recv_many_chan::h4d40e21e69805ee0` | 1 | `main` | 2 | 144 (144 B) | 72 (72 B) |
| `src/s4/s4.rs:5172:13`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 2 | 88 (88 B) | 44 (44 B) |
| `src/readers/syslinereader.rs:1314:44`<br/>`s4lib::readers::syslinereader::SyslineReader::find_datetime_in_line::h76c64a27acc37abd` | 3 | `gen-1000-3-foobar.log.lz4` | 2 | 48 (48 B) | 24 (24 B) |
| `src/readers/filepreprocessor.rs:334:31`<br/>`s4lib::readers::filepreprocessor::pathbuf_to_filetype_impl::h7ee028a2b403903e` | 1 | `main` | 2 | 6 (6 B) | 3 (3 B) |
| `src/readers/blockreader.rs:3468:29`<br/>`s4lib::readers::blockreader::BlockReader::read_block_FileLz4::h9d26d5f04dcb1f2d` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 65,536 (64.00 KiB) | 65,536 (64.00 KiB) |
| `src/readers/blockreader.rs:1031:47`<br/>`s4lib::readers::blockreader::BlockReader::new::h4ce67b93fa8e3683` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 8,192 (8.00 KiB) | 8,192 (8.00 KiB) |
| `src/readers/blockreader.rs:1082:47`<br/>`s4lib::readers::blockreader::BlockReader::new::h4ce67b93fa8e3683` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 8,192 (8.00 KiB) | 8,192 (8.00 KiB) |
| `src/s4/s4.rs:5129:34`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 2,452 (2.39 KiB) | 2,452 (2.39 KiB) |
| `src/s4/s4.rs:5407:34`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 884 (884 B) | 884 (884 B) |
| `src/s4/s4.rs:5044:17`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 368 (368 B) | 368 (368 B) |
| `src/readers/syslogprocessor.rs:189:9`<br/>`<s4lib::readers::syslogprocessor::BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP as core::ops::deref::Deref>::deref::__static_ref_initialize::h0672a2de9a87b4ff` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 280 (280 B) | 280 (280 B) |
| `src/readers/syslogprocessor.rs:174:9`<br/>`<s4lib::readers::syslogprocessor::BLOCKZERO_ANALYSIS_LINE_COUNT_MIN_MAP as core::ops::deref::Deref>::deref::__static_ref_initialize::hd3d24ad1fe727c40` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:5240:9`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:4929:34`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 276 (276 B) | 276 (276 B) |
| `src/s4/s4.rs:3685:51`<br/>`s4::s4::main::hd2a00718532cbbd2` | 1 | `main` | 1 | 224 (224 B) | 224 (224 B) |
| `src/readers/blockreader.rs:2708:28`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_storage::hdaa8db484d68fd3e` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 192 (192 B) | 192 (192 B) |
| `src/readers/syslinereader.rs:553:13`<br/>`s4lib::readers::syslinereader::SyslineReader::new::ha5ee0ef8a85160d8` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 192 (192 B) | 192 (192 B) |
| `src/s4/s4.rs:4935:49`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| `src/s4/s4.rs:4937:40`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 116 (116 B) | 116 (116 B) |
| `src/readers/syslinereader.rs:1316:17`<br/>`s4lib::readers::syslinereader::SyslineReader::find_datetime_in_line::h76c64a27acc37abd` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 112 (112 B) | 112 (112 B) |
| `src/readers/blockreader.rs:2722:14`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_storage::hdaa8db484d68fd3e` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 104 (104 B) | 104 (104 B) |
| `src/s4/s4.rs:2545:12`<br/>`<s4::s4::CLI_Args as clap_builder::derive::FromArgMatches>::from_arg_matches_mut::{{closure}}::h6703d1b21dd76a2a` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| `src/readers/filepreprocessor.rs:1137:22`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 91 (91 B) | 91 (91 B) |
| `src/s4/s4.rs:4954:41`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4950:44`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4952:35`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5139:28`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5127:32`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/readers/filepreprocessor.rs:1167:53`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| `src/s4/s4.rs:5397:26`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5413:52`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/readers/linereader.rs:1129:21`<br/>`s4lib::readers::linereader::LineReader::find_line_in_block::hc904805666cce2d4` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:3421:33`<br/>`s4::s4::cli_process_args::hc2513eaa31d17c92` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/readers/syslogprocessor.rs:1466:20`<br/>`s4lib::readers::syslogprocessor::SyslogProcessor::summary_complete::hfcc5018cbbf8a99a` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 44 (44 B) | 44 (44 B) |
| `src/readers/blockreader.rs:651:20`<br/>`s4lib::readers::blockreader::BlockReader::new::h4ce67b93fa8e3683` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 44 (44 B) | 44 (44 B) |
| `src/readers/blockreader.rs:619:35`<br/>`s4lib::readers::blockreader::BlockReader::new::h4ce67b93fa8e3683` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 44 (44 B) | 44 (44 B) |
| `src/s4/s4.rs:3962:9`<br/>`s4::s4::exec_syslogprocessor::h8106c39799c9915a` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 44 (44 B) | 44 (44 B) |
| `src/s4/s4.rs:3448:29`<br/>`s4::s4::cli_process_args::hc2513eaa31d17c92` | 1 | `main` | 1 | 44 (44 B) | 44 (44 B) |
| `src/s4/s4.rs:5044:56`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 44 (44 B) | 44 (44 B) |
| `src/readers/filepreprocessor.rs:1133:33`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 44 (44 B) | 44 (44 B) |
| `src/readers/filepreprocessor.rs:1167:87`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 44 (44 B) | 44 (44 B) |
| `src/readers/blockreader.rs:3541:26`<br/>`s4lib::readers::blockreader::BlockReader::read_block_FileLz4::h9d26d5f04dcb1f2d` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 40 (40 B) | 40 (40 B) |
| `src/readers/syslinereader.rs:2098:74`<br/>`s4lib::readers::syslinereader::SyslineReader::find_sysline_in_block_year::haa1f8fb8b36f2fa7` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 40 (40 B) | 40 (40 B) |
| `src/s4/s4.rs:2897:5`<br/>`s4::s4::cli_process_tz_offset::ha0fcee891a25dea7` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| `src/readers/blockreader.rs:2689:9`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_LRU_cache::h9597576df80bd0c6` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 32 (32 B) | 32 (32 B) |
| `src/s4/s4.rs:5243:19`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 25 (25 B) | 25 (25 B) |
| `src/readers/helpers.rs:30:5`<br/>`s4lib::readers::helpers::basename::h70537354f6e5017d` | 1 | `main` | 1 | 25 (25 B) | 25 (25 B) |
| `src/s4/s4.rs:5241:32`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 25 (25 B) | 25 (25 B) |
| `src/s4/s4.rs:2896:28`<br/>`s4::s4::cli_process_tz_offset::ha0fcee891a25dea7` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| `src/s4/s4.rs:2735:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2682:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2672:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2720:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2782:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2586:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2557:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2759:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2708:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2660:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2796:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2648:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2614:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2568:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2747:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2694:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2534:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2600:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2630:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2547:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/readers/syslinereader.rs:550:39`<br/>`s4lib::readers::syslinereader::SyslineReader::new::ha5ee0ef8a85160d8` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 8 (8 B) | 8 (8 B) |
| `src/readers/syslinereader.rs:580:31`<br/>`s4lib::readers::syslinereader::SyslineReader::new::ha5ee0ef8a85160d8` | 3 | `gen-1000-3-foobar.log.lz4` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5424:26`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2790:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h1b989b6cd2979b9a` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5229:26`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2581:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h603c3d84d3cce851` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2869:23`<br/>`s4::s4::cli_parse_blocksz::hee67f7902ea5d88b` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2868:32`<br/>`s4::s4::cli_parse_blocksz::hee67f7902ea5d88b` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2754:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::habfe7a0fe57c7fe9` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2715:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::ha43c097dc248e1c4` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2772:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h1d839769a57605e6` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2790:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h1b989b6cd2979b9a` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2730:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hae6027573b2eba9b` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2742:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::he56a849edb3bf699` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/s4/s4.rs:2687:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h7dbc7c85affda2f5` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| `src/s4/s4.rs:2687:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h7dbc7c85affda2f5` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |



## Allocator Tracking summary

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| normal allocations | 20,924,351 | 71,202 | normal program allocations; this is the most useful number |
| total deallocations | 145,896,881 | 396,436 | includes normal program deallocations and tracking deallocations |
| current outstanding | 45,145,882 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| total from tracking | 170,118,389 | 408,295 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 168,403,820 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 1,714,569 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 12 | 100 to 17 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| parameter | value | about |
| :--- | ---: | :--- |
| frame depth | 1 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 111 | entries in the table above |
| cached file names | 9 | |
| cached function names | 41 | |
| cached thread names | 2 | |
