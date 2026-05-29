+ exec env S4_ALLOC_TRACKER_DEPTH=1 S4_ALLOC_TRACKER_PRINT= S4_ALLOC_TRACKER_TRACKING=1 S4_BUILD_REGEX_PRINT=0 RUST_MIN_STACK=20000000 cargo run --quiet --profile alloc_tracker --features alloc_tracker -- ./logs/OpenBSD7.4/x86_64/utmp
## Allocator Tracking results

| ***File:line:col***<br/>***Call Site*** | Thread<br/>ID | Thread<br/>Name | Allocations | Bytes | Bytes<br/>per Allocation |
| :-- | ---: | :--- | ---: | ---: | ---: |
| `src/s4/s4.rs:3403:16`<br/>`s4::s4::cli_process_args::hc2513eaa31d17c92` | 1 | `main` | 208 | 25,436 (24.84 KiB) | 122 (122 B) |
| `src/s4/s4.rs:2499:10`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 52 | 48,546 (47.41 KiB) | 933 (933 B) |
| `src/s4/s4.rs:3748:5`<br/>`s4::s4::set_signal_handler::h04d226b85c8bceb4` | 1 | `main` | 7 | 181 (181 B) | 25 (25 B) |
| `src/s4/s4.rs:5242:15`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 5 | 269 (269 B) | 53 (53 B) |
| `src/s4/s4.rs:5330:53`<br/>`s4::s4::processing_loop::recv_many_chan::h4d40e21e69805ee0` | 1 | `main` | 4 | 512 (512 B) | 128 (128 B) |
| `src/s4/s4.rs:353:13`<br/>`s4::s4::LOCAL_NOW::__init::{{closure}}::h7cd5d76dd27ff375` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| `src/readers/blockreader.rs:1836:35`<br/>`s4lib::readers::blockreader::BlockReader::new::h4ce67b93fa8e3683` | 3 | `utmp` | 3 | 216 (216 B) | 72 (72 B) |
| `src/s4/s4.rs:5238:13`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 2 | 3,800 (3.71 KiB) | 1,900 (1.86 KiB) |
| `src/data/fixedstruct.rs:4306:17`<br/>`s4lib::data::fixedstruct::buffer_to_fixedstructptr::hf2c0aa5c893ea919` | 3 | `utmp` | 2 | 608 (608 B) | 304 (304 B) |
| `src/s4/s4.rs:2834:16`<br/>`s4::s4::cli_process_blocksz::hcc5a5593f13e2c9a` | 1 | `main` | 2 | 156 (156 B) | 78 (78 B) |
| `src/s4/s4.rs:5359:59`<br/>`s4::s4::processing_loop::recv_many_chan::h4d40e21e69805ee0` | 1 | `main` | 2 | 144 (144 B) | 72 (72 B) |
| `src/readers/fixedstructreader.rs:1074:17`<br/>`s4lib::readers::fixedstructreader::FixedStructReader::score_file::hd2c7dc2d405de62f` | 3 | `utmp` | 2 | 80 (80 B) | 40 (40 B) |
| `src/s4/s4.rs:5172:13`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 2 | 58 (58 B) | 29 (29 B) |
| `src/readers/blockreader.rs:2775:26`<br/>`s4lib::readers::blockreader::BlockReader::read_block_File::h7b2ca2ec4ce957ae` | 3 | `utmp` | 1 | 6,992 (6.83 KiB) | 6,992 (6.83 KiB) |
| `src/s4/s4.rs:5129:34`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 2,452 (2.39 KiB) | 2,452 (2.39 KiB) |
| `src/printer/printers.rs:793:21`<br/>`s4lib::printer::printers::PrinterLogMessage::new::ha16f81626adf2867` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| `src/s4/s4.rs:5527:29`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 1,248 (1.22 KiB) | 1,248 (1.22 KiB) |
| `src/printer/printers.rs:758:22`<br/>`s4lib::printer::printers::PrinterLogMessage::new::ha16f81626adf2867` | 1 | `main` | 1 | 1,024 (1.00 KiB) | 1,024 (1.00 KiB) |
| `src/s4/s4.rs:5407:34`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 884 (884 B) | 884 (884 B) |
| `src/readers/fixedstructreader.rs:823:9`<br/>`s4lib::readers::fixedstructreader::FixedStructReader::insert_cache_entry::h60ff21d44ea92574` | 3 | `utmp` | 1 | 808 (808 B) | 808 (808 B) |
| `src/s4/s4.rs:5044:17`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 368 (368 B) | 368 (368 B) |
| `src/readers/fixedstructreader.rs:1294:13`<br/>`s4lib::readers::fixedstructreader::FixedStructReader::preprocess_timevalues::hf0a41a733df75642` | 3 | `utmp` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:5240:9`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:4929:34`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 276 (276 B) | 276 (276 B) |
| `src/s4/s4.rs:3685:51`<br/>`s4::s4::main::hd2a00718532cbbd2` | 1 | `main` | 1 | 224 (224 B) | 224 (224 B) |
| `src/readers/blockreader.rs:2708:28`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_storage::hdaa8db484d68fd3e` | 3 | `utmp` | 1 | 192 (192 B) | 192 (192 B) |
| `src/readers/fixedstructreader.rs:500:25`<br/>`s4lib::readers::fixedstructreader::FixedStructReader::new::h4f3278026223ab21` | 3 | `utmp` | 1 | 192 (192 B) | 192 (192 B) |
| `src/s4/s4.rs:4935:49`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| `src/s4/s4.rs:4937:40`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 116 (116 B) | 116 (116 B) |
| `src/readers/blockreader.rs:2722:14`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_storage::hdaa8db484d68fd3e` | 3 | `utmp` | 1 | 104 (104 B) | 104 (104 B) |
| `src/s4/s4.rs:2545:12`<br/>`<s4::s4::CLI_Args as clap_builder::derive::FromArgMatches>::from_arg_matches_mut::{{closure}}::h6703d1b21dd76a2a` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| `src/s4/s4.rs:4950:44`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4952:35`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5127:32`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4954:41`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5139:28`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/readers/filepreprocessor.rs:1137:22`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 76 (76 B) | 76 (76 B) |
| `src/readers/filepreprocessor.rs:1167:53`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| `src/data/fixedstruct.rs:3770:17`<br/>`s4lib::data::fixedstruct::filesz_to_types::h005d266273a4761a` | 3 | `utmp` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5397:26`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5706:62`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5413:52`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:3421:33`<br/>`s4::s4::cli_process_args::hc2513eaa31d17c92` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/readers/blockreader.rs:2796:30`<br/>`s4lib::readers::blockreader::BlockReader::read_block_File::h7b2ca2ec4ce957ae` | 3 | `utmp` | 1 | 40 (40 B) | 40 (40 B) |
| `src/s4/s4.rs:2897:5`<br/>`s4::s4::cli_process_tz_offset::ha0fcee891a25dea7` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| `src/readers/blockreader.rs:2689:9`<br/>`s4lib::readers::blockreader::BlockReader::store_block_in_LRU_cache::h9597576df80bd0c6` | 3 | `utmp` | 1 | 32 (32 B) | 32 (32 B) |
| `src/s4/s4.rs:4197:9`<br/>`s4::s4::exec_fixedstructprocessor::hb56e9f809f8a546c` | 3 | `utmp` | 1 | 29 (29 B) | 29 (29 B) |
| `src/readers/fixedstructreader.rs:348:54`<br/>`s4lib::readers::fixedstructreader::FixedStructReader::new::h4f3278026223ab21` | 3 | `utmp` | 1 | 29 (29 B) | 29 (29 B) |
| `src/readers/fixedstructreader.rs:1530:20`<br/>`s4lib::readers::fixedstructreader::FixedStructReader::summary_complete::h9f81c4b6645fb445` | 3 | `utmp` | 1 | 29 (29 B) | 29 (29 B) |
| `src/readers/blockreader.rs:651:20`<br/>`s4lib::readers::blockreader::BlockReader::new::h4ce67b93fa8e3683` | 3 | `utmp` | 1 | 29 (29 B) | 29 (29 B) |
| `src/readers/blockreader.rs:619:35`<br/>`s4lib::readers::blockreader::BlockReader::new::h4ce67b93fa8e3683` | 3 | `utmp` | 1 | 29 (29 B) | 29 (29 B) |
| `src/readers/filepreprocessor.rs:1133:33`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 29 (29 B) | 29 (29 B) |
| `src/s4/s4.rs:5044:56`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 29 (29 B) | 29 (29 B) |
| `src/readers/filepreprocessor.rs:1167:87`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 29 (29 B) | 29 (29 B) |
| `src/s4/s4.rs:3448:29`<br/>`s4::s4::cli_process_args::hc2513eaa31d17c92` | 1 | `main` | 1 | 29 (29 B) | 29 (29 B) |
| `src/s4/s4.rs:2896:28`<br/>`s4::s4::cli_process_tz_offset::ha0fcee891a25dea7` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| `src/s4/s4.rs:2648:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2600:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2747:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2660:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2586:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2720:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2568:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2672:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2708:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2735:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2557:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2694:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2630:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2796:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2614:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2759:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2782:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2547:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2682:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2534:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/printer/printers.rs:760:28`<br/>`s4lib::printer::printers::PrinterLogMessage::new::ha16f81626adf2867` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| `src/s4/s4.rs:2581:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h603c3d84d3cce851` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2790:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h1b989b6cd2979b9a` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2869:23`<br/>`s4::s4::cli_parse_blocksz::hee67f7902ea5d88b` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5229:26`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5424:26`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2772:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h1d839769a57605e6` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2790:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h1b989b6cd2979b9a` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2730:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hae6027573b2eba9b` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2868:32`<br/>`s4::s4::cli_parse_blocksz::hee67f7902ea5d88b` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2715:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::ha43c097dc248e1c4` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2754:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::habfe7a0fe57c7fe9` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:5243:19`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/s4/s4.rs:5241:32`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/readers/helpers.rs:30:5`<br/>`s4lib::readers::helpers::basename::h70537354f6e5017d` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/s4/s4.rs:2742:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::he56a849edb3bf699` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/readers/filepreprocessor.rs:645:32`<br/>`s4lib::readers::filepreprocessor::pathbuf_to_filetype_impl::h7ee028a2b403903e` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/s4/s4.rs:2687:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h7dbc7c85affda2f5` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| `src/s4/s4.rs:2687:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h7dbc7c85affda2f5` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |



## Allocator Tracking summary

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| normal allocations | 105,297 | 376 | normal program allocations; this is the most useful number |
| total deallocations | 120,177,832 | 177,943 | includes normal program deallocations and tracking deallocations |
| current outstanding | 45,111,238 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| total from tracking | 165,183,750 | 260,524 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 163,470,047 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 1,713,703 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 0 | 100 to 0 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| parameter | value | about |
| :--- | ---: | :--- |
| frame depth | 1 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 95 | entries in the table above |
| cached file names | 7 | |
| cached function names | 35 | |
| cached thread names | 2 | |
