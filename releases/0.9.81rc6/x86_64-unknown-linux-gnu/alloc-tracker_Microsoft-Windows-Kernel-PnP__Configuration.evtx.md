+ exec env S4_ALLOC_TRACKER_DEPTH=1 S4_ALLOC_TRACKER_PRINT= S4_ALLOC_TRACKER_TRACKING=1 S4_BUILD_REGEX_PRINT=0 RUST_MIN_STACK=20000000 cargo run --quiet --profile alloc_tracker --features alloc_tracker -- ./logs/programs/evtx/Microsoft-Windows-Kernel-PnP__Configuration.evtx
## Allocator Tracking results

| ***File:line:col***<br/>***Call Site*** | Thread<br/>ID | Thread<br/>Name | Allocations | Bytes | Bytes<br/>per Allocation |
| :-- | ---: | :--- | ---: | ---: | ---: |
| `src/readers/evtxreader.rs:399:32`<br/>`s4lib::readers::evtxreader::EvtxReader::analyze::hc282157f989aa6f2` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 467 | 27,993,336 (26.70 MiB) | 59,942 (58.54 KiB) |
| `src/s4/s4.rs:5330:53`<br/>`s4::s4::processing_loop::recv_many_chan::h4d40e21e69805ee0` | 1 | `main` | 229 | 29,312 (28.62 KiB) | 128 (128 B) |
| `src/s4/s4.rs:3403:16`<br/>`s4::s4::cli_process_args::hc2513eaa31d17c92` | 1 | `main` | 208 | 25,556 (24.96 KiB) | 122 (122 B) |
| `src/readers/evtxreader.rs:346:24`<br/>`s4lib::readers::evtxreader::EvtxReader::new::h988f46a347b782fe` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 172 | 56,590 (55.26 KiB) | 329 (329 B) |
| `src/s4/s4.rs:2499:10`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 52 | 48,546 (47.41 KiB) | 933 (933 B) |
| `src/readers/evtxreader.rs:472:38`<br/>`s4lib::readers::evtxreader::EvtxReader::analyze::hc282157f989aa6f2` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 37 | 46,656 (45.56 KiB) | 1,260 (1.23 KiB) |
| `src/s4/s4.rs:3748:5`<br/>`s4::s4::set_signal_handler::h04d226b85c8bceb4` | 1 | `main` | 7 | 181 (181 B) | 25 (25 B) |
| `src/s4/s4.rs:5242:15`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 5 | 313 (313 B) | 62 (62 B) |
| `src/s4/s4.rs:353:13`<br/>`s4::s4::LOCAL_NOW::__init::{{closure}}::h7cd5d76dd27ff375` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| `src/s4/s4.rs:5238:13`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 2 | 3,800 (3.71 KiB) | 1,900 (1.86 KiB) |
| `src/readers/evtxreader.rs:348:50`<br/>`s4lib::readers::evtxreader::EvtxReader::new::h988f46a347b782fe` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 2 | 172 (172 B) | 86 (86 B) |
| `src/s4/s4.rs:2834:16`<br/>`s4::s4::cli_process_blocksz::hcc5a5593f13e2c9a` | 1 | `main` | 2 | 156 (156 B) | 78 (78 B) |
| `src/s4/s4.rs:3914:11`<br/>`s4::s4::chan_send::hd2431125fff1d7f8` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 2 | 144 (144 B) | 72 (72 B) |
| `src/s4/s4.rs:5359:59`<br/>`s4::s4::processing_loop::recv_many_chan::h4d40e21e69805ee0` | 1 | `main` | 2 | 144 (144 B) | 72 (72 B) |
| `src/s4/s4.rs:5172:13`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 2 | 138 (138 B) | 69 (69 B) |
| `src/s4/s4.rs:5129:34`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 2,452 (2.39 KiB) | 2,452 (2.39 KiB) |
| `src/printer/printers.rs:793:21`<br/>`s4lib::printer::printers::PrinterLogMessage::new::ha16f81626adf2867` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| `src/s4/s4.rs:5527:29`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 1,248 (1.22 KiB) | 1,248 (1.22 KiB) |
| `src/printer/printers.rs:758:22`<br/>`s4lib::printer::printers::PrinterLogMessage::new::ha16f81626adf2867` | 1 | `main` | 1 | 1,024 (1.00 KiB) | 1,024 (1.00 KiB) |
| `src/s4/s4.rs:5407:34`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 884 (884 B) | 884 (884 B) |
| `src/s4/s4.rs:5044:17`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 368 (368 B) | 368 (368 B) |
| `src/s4/s4.rs:5240:9`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:4929:34`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 276 (276 B) | 276 (276 B) |
| `src/s4/s4.rs:3685:51`<br/>`s4::s4::main::hd2a00718532cbbd2` | 1 | `main` | 1 | 224 (224 B) | 224 (224 B) |
| `src/s4/s4.rs:4935:49`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| `src/readers/filepreprocessor.rs:1137:22`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 116 (116 B) | 116 (116 B) |
| `src/s4/s4.rs:4937:40`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 116 (116 B) | 116 (116 B) |
| `src/s4/s4.rs:2545:12`<br/>`<s4::s4::CLI_Args as clap_builder::derive::FromArgMatches>::from_arg_matches_mut::{{closure}}::h6703d1b21dd76a2a` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| `src/s4/s4.rs:4954:41`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5139:28`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4950:44`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4952:35`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5127:32`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/readers/evtxreader.rs:599:20`<br/>`s4lib::readers::evtxreader::EvtxReader::summary_complete::hf89fdde27897c997` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 69 (69 B) | 69 (69 B) |
| `src/s4/s4.rs:4457:9`<br/>`s4::s4::exec_evtxprocessor::h7846ffe9b231bcdb` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 69 (69 B) | 69 (69 B) |
| `src/readers/filepreprocessor.rs:1133:33`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| `src/readers/filepreprocessor.rs:1167:87`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| `src/s4/s4.rs:5044:56`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| `src/s4/s4.rs:3448:29`<br/>`s4::s4::cli_process_args::hc2513eaa31d17c92` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| `src/readers/evtxreader.rs:349:31`<br/>`s4lib::readers::evtxreader::EvtxReader::new::h988f46a347b782fe` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 56 (56 B) | 56 (56 B) |
| `src/readers/filepreprocessor.rs:1167:53`<br/>`s4lib::readers::filepreprocessor::process_path::hf84d56678281c100` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| `src/s4/s4.rs:5413:52`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5397:26`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5706:62`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/readers/helpers.rs:30:5`<br/>`s4lib::readers::helpers::basename::h70537354f6e5017d` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:3421:33`<br/>`s4::s4::cli_process_args::hc2513eaa31d17c92` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:5243:19`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:5241:32`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:2897:5`<br/>`s4::s4::cli_process_tz_offset::ha0fcee891a25dea7` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| `src/s4/s4.rs:2896:28`<br/>`s4::s4::cli_process_tz_offset::ha0fcee891a25dea7` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| `src/s4/s4.rs:2568:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2648:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2586:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2660:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2735:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2547:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2720:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2708:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2747:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2534:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2759:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2782:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2614:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2557:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2796:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2682:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2672:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2630:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2600:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2694:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::ha061695984d749d3` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/printer/printers.rs:760:28`<br/>`s4lib::printer::printers::PrinterLogMessage::new::ha16f81626adf2867` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| `src/s4/s4.rs:2790:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h1b989b6cd2979b9a` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2581:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h603c3d84d3cce851` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5229:26`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5424:26`<br/>`s4::s4::processing_loop::h0d303c79a1a5d50a` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2869:23`<br/>`s4::s4::cli_parse_blocksz::hee67f7902ea5d88b` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2754:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::habfe7a0fe57c7fe9` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2790:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h1b989b6cd2979b9a` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2868:32`<br/>`s4::s4::cli_parse_blocksz::hee67f7902ea5d88b` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2772:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h1d839769a57605e6` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2730:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hae6027573b2eba9b` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2715:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::ha43c097dc248e1c4` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/readers/filepreprocessor.rs:334:31`<br/>`s4lib::readers::filepreprocessor::pathbuf_to_filetype_impl::h7ee028a2b403903e` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/s4/s4.rs:2742:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::he56a849edb3bf699` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/s4/s4.rs:2687:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h7dbc7c85affda2f5` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| `src/s4/s4.rs:2687:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h7dbc7c85affda2f5` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |



## Allocator Tracking summary

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| normal allocations | 28,222,024 | 1,263 | normal program allocations; this is the most useful number |
| total deallocations | 152,697,377 | 183,281 | includes normal program deallocations and tracking deallocations |
| current outstanding | 45,314,735 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| total from tracking | 169,790,065 | 265,218 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 163,909,207 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 5,880,858 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 17 | 100 to 0 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| parameter | value | about |
| :--- | ---: | :--- |
| frame depth | 1 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 86 | entries in the table above |
| cached file names | 5 | |
| cached function names | 28 | |
| cached thread names | 2 | |
