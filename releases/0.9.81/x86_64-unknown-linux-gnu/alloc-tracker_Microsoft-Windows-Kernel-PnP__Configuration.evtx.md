# `Microsoft-Windows-Kernel-PnP__Configuration.evtx`

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

`$ ./target/alloc_tracker/s4 ./logs/programs/evtx/Microsoft-Windows-Kernel-PnP__Configuration.evtx`

## Allocator Tracking results

| ***File:line:col***<br/>***Call Site*** | Thread<br/>ID | Thread<br/>Name | Allocations | Bytes | Bytes<br/>per Allocation |
| :-- | ---: | :--- | ---: | ---: | ---: |
| `src/readers/evtxreader.rs:399:32`<br/>`s4lib::readers::evtxreader::EvtxReader::analyze::hed23a64afafe283a` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 467 | 27,993,336 (26.70 MiB) | 59,942 (58.54 KiB) |
| `src/s4/s4.rs:5330:53`<br/>`s4::s4::processing_loop::recv_many_chan::hcf11e7434d98ce19` | 1 | `main` | 229 | 29,312 (28.62 KiB) | 128 (128 B) |
| `src/s4/s4.rs:3403:16`<br/>`s4::s4::cli_process_args::h88e0e2c6d5cbf9e2` | 1 | `main` | 208 | 25,558 (24.96 KiB) | 122 (122 B) |
| `src/readers/evtxreader.rs:346:24`<br/>`s4lib::readers::evtxreader::EvtxReader::new::hcfc589e3f4c26d2e` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 172 | 56,590 (55.26 KiB) | 329 (329 B) |
| `src/s4/s4.rs:2499:10`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 52 | 48,546 (47.41 KiB) | 933 (933 B) |
| `src/readers/evtxreader.rs:472:38`<br/>`s4lib::readers::evtxreader::EvtxReader::analyze::hed23a64afafe283a` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 37 | 46,656 (45.56 KiB) | 1,260 (1.23 KiB) |
| `src/s4/s4.rs:3748:5`<br/>`s4::s4::set_signal_handler::h125013a81a433b00` | 1 | `main` | 7 | 181 (181 B) | 25 (25 B) |
| `src/s4/s4.rs:5242:15`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 5 | 313 (313 B) | 62 (62 B) |
| `src/s4/s4.rs:353:13`<br/>`s4::s4::LOCAL_NOW::__init::{{closure}}::h9e9a6660e9e40986` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| `src/s4/s4.rs:5238:13`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 2 | 3,800 (3.71 KiB) | 1,900 (1.86 KiB) |
| `src/readers/evtxreader.rs:348:50`<br/>`s4lib::readers::evtxreader::EvtxReader::new::hcfc589e3f4c26d2e` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 2 | 178 (178 B) | 89 (89 B) |
| `src/s4/s4.rs:2834:16`<br/>`s4::s4::cli_process_blocksz::h51454ca5305bb990` | 1 | `main` | 2 | 156 (156 B) | 78 (78 B) |
| `src/s4/s4.rs:3914:11`<br/>`s4::s4::chan_send::hbf90949ec8f45664` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 2 | 144 (144 B) | 72 (72 B) |
| `src/s4/s4.rs:5359:59`<br/>`s4::s4::processing_loop::recv_many_chan::hcf11e7434d98ce19` | 1 | `main` | 2 | 144 (144 B) | 72 (72 B) |
| `src/s4/s4.rs:5172:13`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 2 | 138 (138 B) | 69 (69 B) |
| `src/s4/s4.rs:5129:34`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 2,452 (2.39 KiB) | 2,452 (2.39 KiB) |
| `src/printer/printers.rs:793:21`<br/>`s4lib::printer::printers::PrinterLogMessage::new::had48950bffa1c119` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| `src/s4/s4.rs:5527:29`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 1,248 (1.22 KiB) | 1,248 (1.22 KiB) |
| `src/printer/printers.rs:758:22`<br/>`s4lib::printer::printers::PrinterLogMessage::new::had48950bffa1c119` | 1 | `main` | 1 | 1,024 (1.00 KiB) | 1,024 (1.00 KiB) |
| `src/s4/s4.rs:5407:34`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 884 (884 B) | 884 (884 B) |
| `src/s4/s4.rs:5044:17`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 368 (368 B) | 368 (368 B) |
| `src/s4/s4.rs:5240:9`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:4929:34`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 276 (276 B) | 276 (276 B) |
| `src/s4/s4.rs:3685:51`<br/>`s4::s4::main::hbaf861000d5581ed` | 1 | `main` | 1 | 224 (224 B) | 224 (224 B) |
| `src/s4/s4.rs:4935:49`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| `src/readers/filepreprocessor.rs:1137:22`<br/>`s4lib::readers::filepreprocessor::process_path::hbbaea7b4727b31bc` | 1 | `main` | 1 | 122 (122 B) | 122 (122 B) |
| `src/s4/s4.rs:4937:40`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 116 (116 B) | 116 (116 B) |
| `src/s4/s4.rs:2545:12`<br/>`<s4::s4::CLI_Args as clap_builder::derive::FromArgMatches>::from_arg_matches_mut::{{closure}}::h961885fdfcc3c815` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| `src/s4/s4.rs:4954:41`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4952:35`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5139:28`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5127:32`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4950:44`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:4457:9`<br/>`s4::s4::exec_evtxprocessor::hb6c33c687e0969b4` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 69 (69 B) | 69 (69 B) |
| `src/readers/evtxreader.rs:599:20`<br/>`s4lib::readers::evtxreader::EvtxReader::summary_complete::hf9ec09f57a343be4` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 69 (69 B) | 69 (69 B) |
| `src/s4/s4.rs:3448:29`<br/>`s4::s4::cli_process_args::h88e0e2c6d5cbf9e2` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| `src/readers/filepreprocessor.rs:1133:33`<br/>`s4lib::readers::filepreprocessor::process_path::hbbaea7b4727b31bc` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| `src/s4/s4.rs:5044:56`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| `src/readers/filepreprocessor.rs:1167:87`<br/>`s4lib::readers::filepreprocessor::process_path::hbbaea7b4727b31bc` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| `src/readers/evtxreader.rs:349:31`<br/>`s4lib::readers::evtxreader::EvtxReader::new::hcfc589e3f4c26d2e` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 56 (56 B) | 56 (56 B) |
| `src/readers/filepreprocessor.rs:1167:53`<br/>`s4lib::readers::filepreprocessor::process_path::hbbaea7b4727b31bc` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| `src/s4/s4.rs:5397:26`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5706:62`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5413:52`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/readers/helpers.rs:30:5`<br/>`s4lib::readers::helpers::basename::h750318f35665bb80` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:3421:33`<br/>`s4::s4::cli_process_args::h88e0e2c6d5cbf9e2` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:5241:32`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:5243:19`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:2897:5`<br/>`s4::s4::cli_process_tz_offset::h89478d5e94c24d33` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| `src/s4/s4.rs:2896:28`<br/>`s4::s4::cli_process_tz_offset::h89478d5e94c24d33` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| `src/s4/s4.rs:2600:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2759:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2614:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2660:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2708:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2720:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2796:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2694:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2586:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2682:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2568:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2648:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2534:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2547:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2557:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2782:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2672:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2747:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2735:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2630:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h3133b4155b2bf2f1` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/printer/printers.rs:760:28`<br/>`s4lib::printer::printers::PrinterLogMessage::new::had48950bffa1c119` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| `src/s4/s4.rs:5229:26`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2869:23`<br/>`s4::s4::cli_parse_blocksz::h07da46be8c72d7a9` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2790:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h79b5fc76fb20f99e` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2581:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h92423ea634403cb1` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5424:26`<br/>`s4::s4::processing_loop::h35d3236bfe375cc8` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2715:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h84f6e1bf9a3cfbc0` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2790:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h79b5fc76fb20f99e` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2730:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h542fcb16d362eec7` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2772:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hf237768366615430` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2754:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hfec2c6f7a4ac335d` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2868:32`<br/>`s4::s4::cli_parse_blocksz::h07da46be8c72d7a9` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/readers/filepreprocessor.rs:334:31`<br/>`s4lib::readers::filepreprocessor::pathbuf_to_filetype_impl::hd266c5704895e843` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/s4/s4.rs:2742:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h0556509c3310021a` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/s4/s4.rs:2687:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hacbe28393d46c9c2` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| `src/s4/s4.rs:2687:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hacbe28393d46c9c2` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |



## Allocator Tracking summary

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| normal allocations | 28,222,038 | 1,263 | normal program allocations; this is the most useful number |
| total deallocations | 210,068,369 | 215,269 | includes normal program deallocations and tracking deallocations |
| current outstanding | 67,061,014 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| total from tracking | 248,907,322 | 311,764 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 243,025,933 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 5,881,389 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 11 | 100 to 0 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| parameter | value | about |
| :--- | ---: | :--- |
| frame depth | 1 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 86 | entries in the table above |
| cached file names | 5 | |
| cached function names | 28 | |
| cached thread names | 2 | |

2026-05-28 23:06:40.778016743 -07:00
