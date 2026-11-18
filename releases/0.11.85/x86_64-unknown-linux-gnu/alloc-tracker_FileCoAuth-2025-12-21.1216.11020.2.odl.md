# `FileCoAuth-2025-12-21.1216.11020.2.odl`

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

`$ ./target/alloc_tracker/s4 ./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/Common/FileCoAuth-2025-12-21.1216.11020.2.odl`

## Allocator Tracking results

| **File:line:col**<br/>**Call Site** | **Thread<br/>ID** | **Thread<br/>Name** | **Allocations** | **Bytes** | **Bytes<br/>per Allocation** |
| :-- | ---: | :--- | ---: | ---: | ---: |
| [`src/s4/s4.rs:3879:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3879)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | 165 | 18,044 (17.62 KiB) | 109 (109 B) |
| [`src/s4/s4.rs:4140:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4140)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:598:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L598)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlParser<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManaged>>::read_record` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 71 | 35,913 (35.07 KiB) | 505 (505 B) |
| [`src/readers/odlparser.rs:684:57`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L684)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlParser<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManaged>>::next_event` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/data/odl.rs:102:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/data/odl.rs#L102)<br/>`<s4lib[6747c811d5b9f10f]::data::odl::OdlEvent>::render` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 71 | 35,553 (34.72 KiB) | 500 (500 B) |
| [`src/readers/odlreader.rs:317:40`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L317)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlreader::OdlReader>::analyze` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:658:47`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L658)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlParser<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManaged>>::read_record::{closure#1}` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 71 | 1,704 (1.66 KiB) | 24 (24 B) |
| [`src/readers/odlparser.rs:615:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L615)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlParser<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManaged>>::read_record` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:636:62`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L636)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlParser<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManaged>>::read_record::{closure#1}` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 55 | 29,261 (28.58 KiB) | 532 (532 B) |
| [`src/readers/odlparser.rs:615:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L615)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlParser<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManaged>>::read_record` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:741:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L741)<br/>`s4lib[6747c811d5b9f10f]::readers::odlparser::read_string` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 54 | 2,269 (2.22 KiB) | 42 (42 B) |
| [`src/readers/odlparser.rs:635:43`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L635)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlParser<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManaged>>::read_record::{closure#1}` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 38 | 10,658 (10.41 KiB) | 280 (280 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:741:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L741)<br/>`s4lib[6747c811d5b9f10f]::readers::odlparser::read_string` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 34 | 1,088 (1.06 KiB) | 32 (32 B) |
| [`src/readers/odlparser.rs:624:46`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L624)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlParser<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManaged>>::read_record::{closure#1}` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:425:20`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L425)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::decode_parameter` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 28 | 83,236 (81.29 KiB) | 2,972 (2.90 KiB) |
| [`src/readers/odlparser.rs:779:48`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L779)<br/>`s4lib[6747c811d5b9f10f]::readers::odlparser::extract_parameters` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:788:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L788)<br/>`s4lib[6747c811d5b9f10f]::readers::odlparser::extract_parameters` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 26 | 2,496 (2.44 KiB) | 96 (96 B) |
| [`src/readers/odlparser.rs:638:17`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L638)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlParser<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManaged>>::read_record::{closure#1}` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:408:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L408)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::decode_parameter` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 16 | 11,042 (10.78 KiB) | 690 (690 B) |
| [`src/readers/odlparser.rs:779:48`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L779)<br/>`s4lib[6747c811d5b9f10f]::readers::odlparser::extract_parameters` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlreader.rs:389:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L389)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlreader::OdlReader>::analyze` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 11 | 10,920 (10.66 KiB) | 992 (992 B) |
| [`src/s4/s4.rs:5108:25`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5108)<br/>`s4[565658153264a5bc]::s4::exec_odlparser` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:344:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L344)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::decode_token` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 9 | 18,990 (18.54 KiB) | 2,110 (2.06 KiB) |
| [`src/readers/odlparser.rs:417:40`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L417)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::decode_parameter` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:285:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L285)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::load_keystore::<&mut dyn alloc[c23e483dde3f7cfb]::io::read::Read>` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 9 | 851 (851 B) | 94 (94 B) |
| [`src/readers/odlreader.rs:453:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L453)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlreader::OdlCompanions>::load` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:348:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L348)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::decode_token` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 8 | 25,296 (24.70 KiB) | 3,162 (3.09 KiB) |
| [`src/readers/odlparser.rs:417:40`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L417)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::decode_parameter` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:345:35`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L345)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::decode_token::{closure#0}` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 8 | 18,963 (18.52 KiB) | 2,370 (2.31 KiB) |
| [`src/readers/odlparser.rs:345:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L345)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::decode_token` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:411:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L411)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::decode_parameter` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 6 | 1,146 (1.12 KiB) | 191 (191 B) |
| [`src/readers/odlparser.rs:779:48`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L779)<br/>`s4lib[6747c811d5b9f10f]::readers::odlparser::extract_parameters` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlreader.rs:559:41`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L559)<br/>`s4lib[6747c811d5b9f10f]::readers::odlreader::load_companions` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 6 | 549 (549 B) | 91 (91 B) |
| [`src/readers/odlreader.rs:143:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L143)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlreader::OdlReader>::new` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:4188:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4188)<br/>`s4[565658153264a5bc]::s4::set_signal_handler` | 1 | `main` | 6 | 149 (149 B) | 24 (24 B) |
| [`src/s4/s4.rs:5627:15`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5627)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:80:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L80)<br/>`s4lib[6747c811d5b9f10f]::readers::odlparser::read_bounded::<&mut dyn alloc[c23e483dde3f7cfb]::io::read::Read>` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 4 | 480 (480 B) | 120 (120 B) |
| [`src/readers/odlparser.rs:282:35`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L282)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::load_keystore::<&mut dyn alloc[c23e483dde3f7cfb]::io::read::Read>` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5832:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5832)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 4 | 247 (247 B) | 61 (61 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:420:48`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L420)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManagerState>::new` | 1 | `main` | 3 | 34,880 (34.06 KiB) | 11,626 (11.35 KiB) |
| [`src/readers/filehandlemanager.rs:880:40`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L880)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::new_open_max` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:378:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L378)<br/>`s4[565658153264a5bc]::s4::LOCAL_NOW::__rust_std_internal_init_fn::{closure#0}` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| [`src/s4/s4.rs:377:17`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L377)<br/>`s4[565658153264a5bc]::s4::LOCAL_NOW::__rust_std_internal_init_fn` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5930:66`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5930)<br/>`s4[565658153264a5bc]::s4::processing_loop::recv_many_chan` | 1 | `main` | 3 | 240 (240 B) | 80 (80 B) |
| [`src/s4/s4.rs:6080:52`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6080)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:455:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L455)<br/>`<s4[565658153264a5bc]::s4::CLI_Color_Choice as clap_builder[639015aad05df3ec]::derive::ValueEnum>::to_possible_value` | 1 | `main` | 3 | 24 (24 B) | 8 (8 B) |
| [`src/s4/s4.rs:3879:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3879)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5825:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5825)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 2 | 3,960 (3.87 KiB) | 1,980 (1.93 KiB) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5622:38`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5622)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 2 | 648 (648 B) | 324 (324 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:945:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L945)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::request_open_managed` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 2 | 179 (179 B) | 89 (89 B) |
| [`src/readers/odlreader.rs:560:44`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L560)<br/>`s4lib[6747c811d5b9f10f]::readers::odlreader::load_companions` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:4333:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4333)<br/>`s4[565658153264a5bc]::s4::chan_send` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 2 | 144 (144 B) | 72 (72 B) |
| [`src/s4/s4.rs:5112:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5112)<br/>`s4[565658153264a5bc]::s4::exec_odlparser` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5828:47`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5828)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 2 | 96 (96 B) | 48 (48 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/odlreader.rs:489:49`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L489)<br/>`s4lib[6747c811d5b9f10f]::readers::odlreader::load_companions` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 2 | 66 (66 B) | 33 (33 B) |
| [`src/readers/odlreader.rs:143:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L143)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlreader::OdlReader>::new` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:547:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L547)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManagerState>::touch` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 2 | 48 (48 B) | 24 (24 B) |
| [`src/readers/filehandlemanager.rs:773:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L773)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManagerState>::open_entry_once` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3173:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3173)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 20,480 (20.00 KiB) | 20,480 (20.00 KiB) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3079:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3079)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 10,240 (10.00 KiB) | 10,240 (10.00 KiB) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:529:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L529)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlParser<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManaged>>::new` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 8,192 (8.00 KiB) | 8,192 (8.00 KiB) |
| [`src/readers/odlreader.rs:281:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L281)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlreader::OdlReader>::analyze` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
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
| [`src/printer/printers.rs:839:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/printer/printers.rs#L839)<br/>`<s4lib[6747c811d5b9f10f]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| [`src/s4/s4.rs:6408:54`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6408)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/printer/printers.rs:838:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/printer/printers.rs#L838)<br/>`<s4lib[6747c811d5b9f10f]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| [`src/s4/s4.rs:6408:54`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6408)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
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
| [`src/readers/filehandlemanager.rs:880:20`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L880)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::new_open_max` | 1 | `main` | 1 | 384 (384 B) | 384 (384 B) |
| [`src/readers/filehandlemanager.rs:873:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L873)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::new` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5554:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5554)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 320 (320 B) | 320 (320 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:951:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L951)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::request_open_managed` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 276 (276 B) | 276 (276 B) |
| [`src/readers/odlreader.rs:560:44`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L560)<br/>`s4lib[6747c811d5b9f10f]::readers::odlreader::load_companions` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
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
| [`src/readers/filepreprocessor.rs:1852:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L1852)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 153 (153 B) | 153 (153 B) |
| [`src/s4/s4.rs:4149:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4149)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5438:49`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5438)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:318:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L318)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::load_keystore::<&mut dyn alloc[c23e483dde3f7cfb]::io::read::Read>::{closure#1}` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 128 (128 B) | 128 (128 B) |
| [`src/readers/odlparser.rs:286:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L286)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::load_keystore::<&mut dyn alloc[c23e483dde3f7cfb]::io::read::Read>` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:316:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L316)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::load_keystore::<&mut dyn alloc[c23e483dde3f7cfb]::io::read::Read>::{closure#1}` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 128 (128 B) | 128 (128 B) |
| [`src/readers/odlparser.rs:286:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L286)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::load_keystore::<&mut dyn alloc[c23e483dde3f7cfb]::io::read::Read>` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5971:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5971)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 128 (128 B) | 128 (128 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:115:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L115)<br/>`s4lib[6747c811d5b9f10f]::readers::odlparser::decode_text` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 111 (111 B) | 111 (111 B) |
| [`src/readers/odlparser.rs:283:35`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L283)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::load_keystore::<&mut dyn alloc[c23e483dde3f7cfb]::io::read::Read>` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5095:56`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5095)<br/>`s4[565658153264a5bc]::s4::exec_odlparser` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:5352:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5352)<br/>`s4[565658153264a5bc]::s4::exec_fileprocessor_thread` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:945:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L945)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::request_open_managed` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/readers/odlreader.rs:153:40`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L153)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlreader::OdlReader>::new` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlreader.rs:413:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L413)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlreader::OdlReader>::summary_complete` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:5120:65`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5120)<br/>`s4[565658153264a5bc]::s4::exec_odlparser` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3924:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3924)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:4140:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4140)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:1895:88`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L1895)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:4149:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4149)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5554:61`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5554)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5713:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5713)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5440:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5440)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:1848:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L1848)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:4149:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4149)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5713:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5713)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2976:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2976)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::FromArgMatches>::from_arg_matches_mut::{closure#0}` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| [`src/s4/s4.rs:2976:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2976)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::FromArgMatches>::from_arg_matches_mut` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:374:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L374)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileEntryManaged>::new` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 86 (86 B) | 86 (86 B) |
| [`src/readers/filehandlemanager.rs:951:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L951)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::request_open_managed` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
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
| [`src/readers/helpers.rs:122:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/helpers.rs#L122)<br/>`s4lib[6747c811d5b9f10f]::readers::helpers::path_to_fpath` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 78 (78 B) | 78 (78 B) |
| [`src/readers/odlreader.rs:567:56`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L567)<br/>`s4lib[6747c811d5b9f10f]::readers::odlreader::load_companions` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
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
| [`src/s4/s4.rs:5457:41`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5457)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5455:35`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5455)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3897:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3897)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/s4/s4.rs:4140:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4140)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3232:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3232)<br/>`s4[565658153264a5bc]::s4::cli_process_tz_offset` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| [`src/s4/s4.rs:3879:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3879)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/helpers.rs:116:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/helpers.rs#L116)<br/>`s4lib[6747c811d5b9f10f]::readers::helpers::basename` | 1 | `main` | 1 | 38 (38 B) | 38 (38 B) |
| [`src/s4/s4.rs:5828:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5828)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5830:29`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5830)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 38 (38 B) | 38 (38 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:6319:62`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6319)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 36 (36 B) | 36 (36 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:6004:52`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6004)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 36 (36 B) | 36 (36 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5977:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5977)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 36 (36 B) | 36 (36 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/odlparser.rs:310:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L310)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::load_keystore::<&mut dyn alloc[c23e483dde3f7cfb]::io::read::Read>::{closure#1}` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 33 (33 B) | 33 (33 B) |
| [`src/readers/odlparser.rs:286:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlparser.rs#L286)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlparser::OdlDecodingContext>::load_keystore::<&mut dyn alloc[c23e483dde3f7cfb]::io::read::Read>` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5674:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5674)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 32 (32 B) | 32 (32 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/odlreader.rs:487:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L487)<br/>`s4lib[6747c811d5b9f10f]::readers::odlreader::load_companions` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 24 (24 B) | 24 (24 B) |
| [`src/readers/odlreader.rs:143:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L143)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlreader::OdlReader>::new` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/readers/odlreader.rs:489:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L489)<br/>`s4lib[6747c811d5b9f10f]::readers::odlreader::load_companions` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 22 (22 B) | 22 (22 B) |
| [`src/readers/odlreader.rs:143:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L143)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlreader::OdlReader>::new` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3231:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3231)<br/>`s4[565658153264a5bc]::s4::cli_process_tz_offset` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| [`src/s4/s4.rs:3879:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3879)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/odlreader.rs:488:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L488)<br/>`s4lib[6747c811d5b9f10f]::readers::odlreader::load_companions` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/readers/odlreader.rs:143:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/odlreader.rs#L143)<br/>`<s4lib[6747c811d5b9f10f]::readers::odlreader::OdlReader>::new` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3147:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3147)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3101:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3101)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3181:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3181)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3043:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3043)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3077:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3077)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3015:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3015)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3111:25`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3111)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3029:17`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3029)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3058:20`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3058)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2997:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2997)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3171:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3171)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3159:19`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3159)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2986:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2986)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3122:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3122)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3135:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3135)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3089:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3089)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2976:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2976)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/printer/printers.rs:804:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/printer/printers.rs#L804)<br/>`<s4lib[6747c811d5b9f10f]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| [`src/s4/s4.rs:6408:54`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6408)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5813:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5813)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3012:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3012)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#0}` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:3012:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3012)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3855:51`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3855)<br/>`s4[565658153264a5bc]::s4::unescape::unescape_str` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:4021:43`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4021)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | | | |
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
| [`src/s4/s4.rs:3119:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3119)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#1}` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| [`src/s4/s4.rs:3119:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3119)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3119:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3119)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#1}` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| [`src/s4/s4.rs:3119:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3119)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |


## Allocator Tracking summary

| **tracked** | **bytes** | **calls** | **about** |
| :--- | ---: | ---: | :--- |
| normal allocations | 416,680 | 812 | normal program allocations; this is the most useful number |
| total deallocations | 189,834,074 | 236,394 | includes normal program deallocations and tracking deallocations |
| current outstanding | 76,602,408 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| **tracked** | **bytes** | **calls** | **about** |
| :--- | ---: | ---: | :--- |
| total from tracking | 266,019,777 | 345,277 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 264,305,825 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 1,713,952 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 0 | 100 to 0 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| **parameter** | **value** | **about** |
| :--- | ---: | :--- |
| frame depth | 2 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 118 | entries in the table above |
| cached file names | 9 | |
| cached function names | 55 | |
| cached thread names | 2 | |

*2026-11-15 13:26:13.396680675 -08:00*
