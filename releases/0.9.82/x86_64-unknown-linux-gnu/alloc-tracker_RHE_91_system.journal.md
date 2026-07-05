# `RHE_91_system.journal`

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

`$ ./target/alloc_tracker/s4 ./logs/programs/journal/RHE_91_system.journal`

## Allocator Tracking results

| ***File:line:col***<br/>***Call Site*** | Thread<br/>ID | Thread<br/>Name | Allocations | Bytes | Bytes<br/>per Allocation |
| :-- | ---: | :--- | ---: | ---: | ---: |
| `src/readers/journalreader.rs:2015:31`<br/>`s4lib::readers::journalreader::JournalReader::next_short::h0a702c0904f4ed76` | 3 | `RHE_91_system.journal` | 8,324 | 97,807 (95.51 KiB) | 11 (11 B) |
| `src/readers/journalreader.rs:1227:43`<br/>`s4lib::readers::journalreader::JournalReader::Error_from_Errno::h6c6da5ff737f560b` | 3 | `RHE_91_system.journal` | 2,823 | 237,132 (231.57 KiB) | 84 (84 B) |
| `src/s4/s4.rs:5406:53`<br/>`s4::s4::processing_loop::recv_many_chan::hf2b1c552a15b43da` | 1 | `main` | 2,083 | 266,624 (260.38 KiB) | 128 (128 B) |
| `src/readers/journalreader.rs:1874:35`<br/>`s4lib::readers::journalreader::JournalReader::next_short::h0a702c0904f4ed76` | 3 | `RHE_91_system.journal` | 2,082 | 2,664,960 (2.54 MiB) | 1,280 (1.25 KiB) |
| `src/readers/journalreader.rs:1227:9`<br/>`s4lib::readers::journalreader::JournalReader::Error_from_Errno::h6c6da5ff737f560b` | 3 | `RHE_91_system.journal` | 1,882 | 45,168 (44.11 KiB) | 24 (24 B) |
| `src/s4/s4.rs:3466:16`<br/>`s4::s4::cli_process_args::hbb85d08869f2b49d` | 1 | `main` | 204 | 25,211 (24.62 KiB) | 123 (123 B) |
| `src/s4/s4.rs:2517:10`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 52 | 48,677 (47.54 KiB) | 936 (936 B) |
| `src/s4/s4.rs:3825:5`<br/>`s4::s4::set_signal_handler::h1be51f5b5f6b4d86` | 1 | `main` | 7 | 181 (181 B) | 25 (25 B) |
| `src/s4/s4.rs:5318:15`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 5 | 286 (286 B) | 57 (57 B) |
| `src/s4/s4.rs:363:13`<br/>`s4::s4::LOCAL_NOW::__init::{{closure}}::h3a2963b7ba27b12e` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| `src/s4/s4.rs:440:5`<br/>`<s4::s4::CLI_Color_Choice as clap_builder::derive::ValueEnum>::to_possible_value::hea38343b8f5aa490` | 1 | `main` | 3 | 24 (24 B) | 8 (8 B) |
| `src/s4/s4.rs:5314:13`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 2 | 3,800 (3.71 KiB) | 1,900 (1.86 KiB) |
| `src/s4/s4.rs:2855:16`<br/>`s4::s4::cli_process_blocksz::hdb09b92225a953df` | 1 | `main` | 2 | 156 (156 B) | 78 (78 B) |
| `src/s4/s4.rs:5435:59`<br/>`s4::s4::processing_loop::recv_many_chan::hf2b1c552a15b43da` | 1 | `main` | 2 | 144 (144 B) | 72 (72 B) |
| `src/s4/s4.rs:5248:13`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 2 | 90 (90 B) | 45 (45 B) |
| `src/libload/systemd_dlopen2.rs:495:24`<br/>`s4lib::libload::systemd_dlopen2::load_library_systemd::h957613bfb9884fff` | 1 | `main` | 2 | 39 (39 B) | 19 (19 B) |
| `src/s4/s4.rs:5205:34`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 2,452 (2.39 KiB) | 2,452 (2.39 KiB) |
| `src/printer/printers.rs:829:32`<br/>`s4lib::printer::printers::PrinterLogMessage::new::hb43a01fede4ab37f` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| `src/printer/printers.rs:828:21`<br/>`s4lib::printer::printers::PrinterLogMessage::new::hb43a01fede4ab37f` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| `src/s4/s4.rs:5603:29`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 1,248 (1.22 KiB) | 1,248 (1.22 KiB) |
| `src/printer/printers.rs:792:22`<br/>`s4lib::printer::printers::PrinterLogMessage::new::hb43a01fede4ab37f` | 1 | `main` | 1 | 1,024 (1.00 KiB) | 1,024 (1.00 KiB) |
| `src/s4/s4.rs:5483:34`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 980 (980 B) | 980 (980 B) |
| `src/s4/s4.rs:5121:17`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 368 (368 B) | 368 (368 B) |
| `src/s4/s4.rs:5316:9`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:5006:34`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 276 (276 B) | 276 (276 B) |
| `src/s4/s4.rs:3762:51`<br/>`s4::s4::main::h0da61b24b42f6b8a` | 1 | `main` | 1 | 224 (224 B) | 224 (224 B) |
| `src/s4/s4.rs:5012:49`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| `src/libload/systemd_dlopen2.rs:445:50`<br/>`s4lib::libload::systemd_dlopen2::set_systemd_journal_api::h214f904a50116b95` | 1 | `main` | 1 | 120 (120 B) | 120 (120 B) |
| `src/s4/s4.rs:5014:40`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 116 (116 B) | 116 (116 B) |
| `src/readers/filepreprocessor.rs:1472:22`<br/>`s4lib::readers::filepreprocessor::process_path::h940fb84aa9eeba43` | 1 | `main` | 1 | 98 (98 B) | 98 (98 B) |
| `src/s4/s4.rs:2566:12`<br/>`<s4::s4::CLI_Args as clap_builder::derive::FromArgMatches>::from_arg_matches_mut::{{closure}}::h8f4c636549f04554` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| `src/s4/s4.rs:5029:35`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5027:44`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5031:41`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5215:28`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5203:32`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/readers/filepreprocessor.rs:1502:53`<br/>`s4lib::readers::filepreprocessor::process_path::h940fb84aa9eeba43` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| `src/s4/s4.rs:5784:62`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5473:26`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5489:52`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:3484:33`<br/>`s4::s4::cli_process_args::hbb85d08869f2b49d` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/readers/journalreader.rs:1086:21`<br/>`s4lib::readers::journalreader::JournalReader::new::hc00196d87b6a267e` | 3 | `RHE_91_system.journal` | 1 | 46 (46 B) | 46 (46 B) |
| `src/s4/s4.rs:4786:9`<br/>`s4::s4::exec_journalprocessor::h30699d5ba4460ebb` | 3 | `RHE_91_system.journal` | 1 | 45 (45 B) | 45 (45 B) |
| `src/readers/journalreader.rs:2943:20`<br/>`s4lib::readers::journalreader::JournalReader::summary_complete::h57ab00ea2b4dd49b` | 3 | `RHE_91_system.journal` | 1 | 45 (45 B) | 45 (45 B) |
| `src/s4/s4.rs:3511:29`<br/>`s4::s4::cli_process_args::hbb85d08869f2b49d` | 1 | `main` | 1 | 45 (45 B) | 45 (45 B) |
| `src/s4/s4.rs:5121:56`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 45 (45 B) | 45 (45 B) |
| `src/readers/filepreprocessor.rs:1502:87`<br/>`s4lib::readers::filepreprocessor::process_path::h940fb84aa9eeba43` | 1 | `main` | 1 | 45 (45 B) | 45 (45 B) |
| `src/readers/filepreprocessor.rs:1468:33`<br/>`s4lib::readers::filepreprocessor::process_path::h940fb84aa9eeba43` | 1 | `main` | 1 | 45 (45 B) | 45 (45 B) |
| `src/s4/s4.rs:2918:5`<br/>`s4::s4::cli_process_tz_offset::h3231a46bb9d02e0c` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| `src/readers/journalreader.rs:530:66`<br/>`<s4lib::readers::journalreader::KEY_SOURCE_REALTIME_TIMESTAMP_CSTR as core::ops::deref::Deref>::deref::__static_ref_initialize::h77f25366405c2828` | 3 | `RHE_91_system.journal` | 1 | 27 (27 B) | 27 (27 B) |
| `src/s4/s4.rs:5317:32`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 21 (21 B) | 21 (21 B) |
| `src/readers/helpers.rs:30:5`<br/>`s4lib::readers::helpers::basename::hb2b98f4ae9040d70` | 1 | `main` | 1 | 21 (21 B) | 21 (21 B) |
| `src/s4/s4.rs:5319:19`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 21 (21 B) | 21 (21 B) |
| `src/s4/s4.rs:2917:28`<br/>`s4::s4::cli_process_tz_offset::h3231a46bb9d02e0c` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| `src/s4/s4.rs:2555:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2716:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2703:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2803:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2681:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2729:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2768:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2651:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2669:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2635:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2621:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2756:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2780:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2578:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2589:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2607:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2568:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2817:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2693:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2741:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/printer/printers.rs:794:28`<br/>`s4lib::printer::printers::PrinterLogMessage::new::hb43a01fede4ab37f` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| `src/libload/systemd_dlopen2.rs:499:52`<br/>`s4lib::libload::systemd_dlopen2::load_library_systemd::h957613bfb9884fff` | 1 | `main` | 1 | 13 (13 B) | 13 (13 B) |
| `src/s4/s4.rs:3440:9`<br/>`s4::s4::unescape::unescape_str::h8985d470cc66fb45` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2811:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hcf68c27431597d26` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2890:23`<br/>`s4::s4::cli_parse_blocksz::h7d22fd388ec587af` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5500:26`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2602:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h79a17608cfbbd63c` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5305:26`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/readers/filepreprocessor.rs:666:31`<br/>`s4lib::readers::filepreprocessor::pathbuf_to_filetype_impl::h64191eb604206c63` | 1 | `main` | 1 | 7 (7 B) | 7 (7 B) |
| `src/s4/s4.rs:2775:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h9f9bfb2cadb5032e` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2811:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hcf68c27431597d26` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2793:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h740ed821b86ae503` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2736:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h6b55cb440a7a965d` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2751:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h5b3e8b9496df1070` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2889:32`<br/>`s4::s4::cli_parse_blocksz::h7d22fd388ec587af` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2763:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h961ed6d112333819` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/s4/s4.rs:2709:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::ha45a23112ac7249a` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| `src/s4/s4.rs:2709:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::ha45a23112ac7249a` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |



## Allocator Tracking summary

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| normal allocations | 3,409,349 | 17,554 | normal program allocations; this is the most useful number |
| total deallocations | 184,989,610 | 285,441 | includes normal program deallocations and tracking deallocations |
| current outstanding | 67,878,003 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| total from tracking | 249,458,241 | 366,804 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 247,744,251 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 1,713,990 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 1 | 100 to 5 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| parameter | value | about |
| :--- | ---: | :--- |
| frame depth | 1 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 92 | entries in the table above |
| cached file names | 6 | |
| cached function names | 33 | |
| cached thread names | 2 | |

2026-07-03 19:48:22.242633279 -07:00
