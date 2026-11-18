# `Microsoft-Windows-Kernel-PnP__Configuration.evtx`

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

`$ ./target/alloc_tracker/s4 ./logs/programs/evtx/Microsoft-Windows-Kernel-PnP__Configuration.evtx`

## Allocator Tracking results

| **File:line:col**<br/>**Call Site** | **Thread<br/>ID** | **Thread<br/>Name** | **Allocations** | **Bytes** | **Bytes<br/>per Allocation** |
| :-- | ---: | :--- | ---: | ---: | ---: |
| [`src/s4/s4.rs:3879:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3879)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | 165 | 17,951 (17.53 KiB) | 108 (108 B) |
| [`src/s4/s4.rs:4140:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4140)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/evtxreader.rs:370:50`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/evtxreader.rs#L370)<br/>`<s4lib[6747c811d5b9f10f]::readers::evtxreader::EvtxReader>::new` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 156 | 55,823 (54.51 KiB) | 357 (357 B) |
| [`src/s4/s4.rs:4898:44`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4898)<br/>`s4[565658153264a5bc]::s4::exec_evtxprocessor` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 38 | 10,658 (10.41 KiB) | 280 (280 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/evtxreader.rs:510:50`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/evtxreader.rs#L510)<br/>`<s4lib[6747c811d5b9f10f]::readers::evtxreader::EvtxReader>::analyze` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 37 | 49,912 (48.74 KiB) | 1,348 (1.32 KiB) |
| [`src/s4/s4.rs:4944:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4944)<br/>`s4[565658153264a5bc]::s4::exec_evtxprocessor` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | | | |
| | | | | | | | |
| [`src/readers/evtxreader.rs:432:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/evtxreader.rs#L432)<br/>`<s4lib[6747c811d5b9f10f]::readers::evtxreader::EvtxReader>::analyze` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 27 | 1,325,936 (1.26 MiB) | 49,108 (47.96 KiB) |
| [`src/s4/s4.rs:4944:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4944)<br/>`s4[565658153264a5bc]::s4::exec_evtxprocessor` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:4188:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4188)<br/>`s4[565658153264a5bc]::s4::set_signal_handler` | 1 | `main` | 6 | 149 (149 B) | 24 (24 B) |
| [`src/s4/s4.rs:5627:15`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5627)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/evtxreader.rs:434:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/evtxreader.rs#L434)<br/>`<s4lib[6747c811d5b9f10f]::readers::evtxreader::EvtxReader>::analyze` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 4 | 384 (384 B) | 96 (96 B) |
| [`src/s4/s4.rs:4944:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4944)<br/>`s4[565658153264a5bc]::s4::exec_evtxprocessor` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5832:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5832)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 4 | 257 (257 B) | 64 (64 B) |
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
| [`src/s4/s4.rs:4333:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4333)<br/>`s4[565658153264a5bc]::s4::chan_send` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 2 | 144 (144 B) | 72 (72 B) |
| [`src/s4/s4.rs:4952:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4952)<br/>`s4[565658153264a5bc]::s4::exec_evtxprocessor` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5828:47`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5828)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 2 | 96 (96 B) | 48 (48 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
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
| [`src/readers/filehandlemanager.rs:951:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L951)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::request_open_managed` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 276 (276 B) | 276 (276 B) |
| [`src/readers/evtxreader.rs:334:65`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/evtxreader.rs#L334)<br/>`<s4lib[6747c811d5b9f10f]::readers::evtxreader::EvtxReader>::new` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | | | |
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
| [`src/s4/s4.rs:5438:49`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5438)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5971:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5971)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 128 (128 B) | 128 (128 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:1852:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L1852)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 122 (122 B) | 122 (122 B) |
| [`src/s4/s4.rs:4149:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4149)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5440:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5440)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2976:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2976)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::FromArgMatches>::from_arg_matches_mut::{closure#0}` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| [`src/s4/s4.rs:2976:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2976)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::FromArgMatches>::from_arg_matches_mut` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5976:38`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5976)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5657:36`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5657)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5975:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5975)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:4900:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4900)<br/>`s4[565658153264a5bc]::s4::exec_evtxprocessor` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 69 (69 B) | 69 (69 B) |
| [`src/s4/s4.rs:5350:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5350)<br/>`s4[565658153264a5bc]::s4::exec_fileprocessor_thread` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:374:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L374)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileEntryManaged>::new` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 69 (69 B) | 69 (69 B) |
| [`src/readers/filehandlemanager.rs:951:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L951)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManager>::request_open_managed` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | | | |
| | | | | | | | |
| [`src/readers/evtxreader.rs:644:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/evtxreader.rs#L644)<br/>`<s4lib[6747c811d5b9f10f]::readers::evtxreader::EvtxReader>::summary_complete` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 69 (69 B) | 69 (69 B) |
| [`src/s4/s4.rs:4967:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4967)<br/>`s4[565658153264a5bc]::s4::exec_evtxprocessor` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3924:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3924)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| [`src/s4/s4.rs:4140:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4140)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:1848:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L1848)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| [`src/s4/s4.rs:4149:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4149)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5713:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5713)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:1895:88`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L1895)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| [`src/s4/s4.rs:4149:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4149)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5554:61`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5554)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5713:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5713)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 69 (69 B) | 69 (69 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/evtxreader.rs:372:63`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/evtxreader.rs#L372)<br/>`<s4lib[6747c811d5b9f10f]::readers::evtxreader::EvtxReader>::new` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 56 (56 B) | 56 (56 B) |
| [`src/s4/s4.rs:4898:44`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4898)<br/>`s4[565658153264a5bc]::s4::exec_evtxprocessor` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | | | |
| | | | | | | | |
| [`src/readers/evtxreader.rs:373:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/evtxreader.rs#L373)<br/>`<s4lib[6747c811d5b9f10f]::readers::evtxreader::EvtxReader>::new` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 56 (56 B) | 56 (56 B) |
| [`src/s4/s4.rs:4898:44`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4898)<br/>`s4[565658153264a5bc]::s4::exec_evtxprocessor` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:1895:49`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L1895)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| [`src/s4/s4.rs:4149:42`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4149)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5453:44`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5453)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5653:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5653)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5455:35`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5455)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5457:41`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5457)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5830:29`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5830)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/helpers.rs:116:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/helpers.rs#L116)<br/>`s4lib[6747c811d5b9f10f]::readers::helpers::basename` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/s4/s4.rs:5828:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5828)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3897:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3897)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/s4/s4.rs:4140:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4140)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
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
| [`src/s4/s4.rs:5674:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5674)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 32 (32 B) | 32 (32 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filehandlemanager.rs:547:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L547)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManagerState>::touch` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | 1 | 24 (24 B) | 24 (24 B) |
| [`src/readers/filehandlemanager.rs:773:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filehandlemanager.rs#L773)<br/>`<s4lib[6747c811d5b9f10f]::readers::filehandlemanager::FileHandleManagerState>::open_entry_once` | 3 | `Microsoft-Windows-Kernel-PnP__Configuration.evtx` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3231:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3231)<br/>`s4[565658153264a5bc]::s4::cli_process_tz_offset` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| [`src/s4/s4.rs:3879:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3879)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3122:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3122)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3171:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3171)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3181:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3181)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2976:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2976)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3058:20`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3058)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
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
| [`src/s4/s4.rs:2986:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2986)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3147:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3147)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3029:17`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3029)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3015:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3015)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3043:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3043)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3101:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3101)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3135:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3135)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3077:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3077)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:2997:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2997)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2931:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L2931)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::CommandFactory>::command` | 1 | `main` | | | |
| | | | | | | | |
| [`src/printer/printers.rs:804:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/printer/printers.rs#L804)<br/>`<s4lib[6747c811d5b9f10f]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| [`src/s4/s4.rs:6408:54`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6408)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3855:51`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3855)<br/>`s4[565658153264a5bc]::s4::unescape::unescape_str` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:4021:43`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4021)<br/>`s4[565658153264a5bc]::s4::cli_process_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5813:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5813)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3012:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3012)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#0}` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:3012:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3012)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3144:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3144)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#3}` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| [`src/s4/s4.rs:3144:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3144)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3168:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3168)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#5}` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| [`src/s4/s4.rs:3168:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3168)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:3156:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3156)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#4}` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:3156:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L3156)<br/>`<s4[565658153264a5bc]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:5683:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L5683)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/s4/s4.rs:6015:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L6015)<br/>`s4[565658153264a5bc]::s4::processing_loop` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:4155:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/s4/s4.rs#L4155)<br/>`s4[565658153264a5bc]::s4::main` | 1 | `main` | | | |
| | | | | | | | |
| [`src/readers/filepreprocessor.rs:821:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/tree/0.11.85/src/readers/filepreprocessor.rs#L821)<br/>`s4lib[6747c811d5b9f10f]::readers::filepreprocessor::pathbuf_to_filetype_impl` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
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
| normal allocations | 1,559,631 | 536 | normal program allocations; this is the most useful number |
| total deallocations | 192,119,978 | 236,825 | includes normal program deallocations and tracking deallocations |
| current outstanding | 76,781,293 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| **tracked** | **bytes** | **calls** | **about** |
| :--- | ---: | ---: | :--- |
| total from tracking | 267,341,615 | 346,110 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 264,511,732 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 2,829,883 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 1 | 100 to 0 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| **parameter** | **value** | **about** |
| :--- | ---: | :--- |
| frame depth | 2 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 95 | entries in the table above |
| cached file names | 7 | |
| cached function names | 38 | |
| cached thread names | 2 | |

*2026-11-15 13:26:07.786082534 -08:00*
