# CHANGELOG<!-- omit in toc -->

<!--
Helper script `tools/changelog-link-gen.sh` can generate the addendum of markdown links for this CHANGELOG.md.
-->

Manual changelog for [super speedy syslog searcher](https://github.com/jtmoon79/super-speedy-syslog-searcher).

---

<!-- TODO per release: update TOC -->
<!-- Table of Contents updated by "Markdown All In One" extension for Visual Studio Code -->

- [Unreleased](#unreleased)
- [0.7.72](#0772)
  - [New](#new)
  - [Changes](#changes)
  - [Fixes](#fixes)
- [0.6.71](#0671)
  - [New](#new-1)
  - [Changes](#changes-1)
  - [Fixes](#fixes-1)
- [0.6.70](#0670)
  - [New](#new-2)
  - [Changes](#changes-2)
- [0.6.69](#0669)
  - [New](#new-3)
  - [Changes](#changes-3)
- [0.6.68](#0668)
  - [New](#new-4)
  - [Changes](#changes-4)
- [0.6.67](#0667)
  - [New](#new-5)
  - [Changes](#changes-5)
  - [Fixes](#fixes-2)
- [0.6.66](#0666)
  - [Changes](#changes-6)
  - [Fixes](#fixes-3)
- [0.6.65](#0665)
  - [New](#new-6)
  - [Changes](#changes-7)
- [0.6.64](#0664)
  - [New](#new-7)
  - [Changes](#changes-8)
  - [Fixes](#fixes-4)
- [0.6.63](#0663)
  - [New](#new-8)
  - [Changes](#changes-9)
  - [Fixes](#fixes-5)
- [0.6.62](#0662)
  - [Fixes](#fixes-6)
- [0.6.61](#0661)
  - [New](#new-9)
  - [Changes](#changes-10)
  - [Fixes](#fixes-7)
- [0.6.60](#0660)
  - [New](#new-10)
- [0.5.59](#0559)
  - [Changes](#changes-11)
  - [Fixes](#fixes-8)
- [0.5.58](#0558)
  - [New](#new-11)
  - [Changes](#changes-12)
- [0.4.57](#0457)
  - [Changes](#changes-13)
  - [Fixes](#fixes-9)
- [0.4.56](#0456)
  - [New](#new-12)
  - [Changes](#changes-14)
  - [Fixes](#fixes-10)
- [0.3.55](#0355)
  - [New](#new-13)
  - [Changes](#changes-15)
  - [Fixes](#fixes-11)
- [0.3.54](#0354)
  - [New](#new-14)
  - [Fixes](#fixes-12)
- [0.3.53](#0353)
  - [New](#new-15)
  - [Changes](#changes-16)
- [0.2.52](#0252)
  - [New](#new-16)
- [0.2.51](#0251)
  - [New](#new-17)
- [0.2.50](#0250)
  - [New](#new-18)
  - [Changes](#changes-17)
  - [Fixes](#fixes-13)
- [0.2.49](#0249)
  - [Changes](#changes-18)
  - [Fixes](#fixes-14)
- [0.2.48](#0248)
  - [New](#new-19)
  - [Changes](#changes-19)
  - [Fixes](#fixes-15)
- [0.2.47](#0247)
- [0.2.46](#0246)
  - [New](#new-20)
  - [Changes](#changes-20)
  - [Fixes](#fixes-16)
- [0.1.45](#0145)
  - [New](#new-21)
  - [Changes](#changes-21)
- [0.1.44](#0144)
  - [New](#new-22)
  - [Changes](#changes-22)
  - [Fixes](#fixes-17)
- [0.1.43](#0143)
  - [New](#new-23)
  - [Changes](#changes-23)
- [0.1.42](#0142)
  - [Changes](#changes-24)
- [0.1.41](#0141)
  - [Changes](#changes-25)
  - [Fixes](#fixes-18)
- [0.1.40](#0140)
  - [New](#new-24)
  - [Changes](#changes-26)
- [0.1.39](#0139)
  - [Changes](#changes-27)
- [0.1.38](#0138)
  - [New](#new-25)
  - [Changes](#changes-28)
- [0.0.37](#0037)
  - [New](#new-26)
  - [Changes](#changes-29)
- [0.0.36](#0036)
  - [New](#new-27)
  - [Changes](#changes-30)
  - [Fixes](#fixes-19)
- [0.0.35](#0035)
  - [New](#new-28)
  - [Fixes](#fixes-20)
- [0.0.34](#0034)
  - [New](#new-29)
  - [Fixes](#fixes-21)
- [0.0.33](#0033)
  - [New](#new-30)
- [0.0.32](#0032)
  - [New](#new-31)
  - [Fixes](#fixes-22)
- [0.0.31](#0031)
  - [New](#new-32)
- [0.0.30](#0030)
  - [New](#new-33)
  - [Changes](#changes-31)
- [0.0.29](#0029)
  - [Changes](#changes-32)
- [0.0.28](#0028)
  - [New](#new-34)
  - [Changes](#changes-33)
  - [Fixes](#fixes-23)
- [0.0.27](#0027)
  - [New](#new-35)
  - [Changes](#changes-34)
- [0.0.26](#0026)
  - [New](#new-36)
  - [Changes](#changes-35)
  - [Fixes](#fixes-24)
- [0.0.25](#0025)
  - [New](#new-37)
  - [Changes](#changes-36)
  - [Fixes](#fixes-25)
- [0.0.24](#0024)
  - [New](#new-38)
  - [Changes](#changes-37)
- [0.0.23](#0023)
  - [New](#new-39)
  - [Changes](#changes-38)
  - [Fixes](#fixes-26)
- [0.0.22](#0022)
  - [New](#new-40)
  - [Changes](#changes-39)
  - [Fixes](#fixes-27)
- [0.0.21](#0021)
  - [New](#new-41)
  - [Fixes](#fixes-28)

---

<!--
TODO per release:

1. Developers must manually create the sections. Do not create empty sections.
2. Developers must manually prefix categories (listed below).

Run `tools/changelog-link-gen.sh` after done editing the sections.

---

Sections:

  ### New

  ### Fixes

  ### Changes

Categories:

  (LIB) - changes to the library source
  (BIN) - changes to the binary source (bin.rs, CLI options, etc.)
  (DEBUG) - changes to the either source only affecting debug builds
  (BUILD) - changes to the build (i.e. Cargo.toml)
  (DOCS) - changes to docstrings ("rustdocs"), docs.rs stuff, READMEs
  (CI) - changes to github workflows, codecov
  (TEST) - changes to tests
  (TOOLS) - changes to scripts under `tools/`
  (PROJECT) - changes to READMEs, CHANGELOG, other non-source tweaks

Helpful `git log` command for generating changelog entries:

    git log --pretty=format:'- %s %Cred([%H])%Creset%C(yellow)%d%Creset' --abbrev-commit <tag-previous-release>...<tag-current-release>
-->

## Unreleased

[unreleased diff]

[unreleased diff]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/latest...main

<!-- TODO per release: Add Section(s) -->

---

## 0.7.72

_Released 2024-05-28_

_MSRV 1.70.0_

[0.6.71..0.7.72]

### New

- (BIN) s4.rs check for FileEmpty FileTooSmall sooner [Issue #270] ([2160556])
- (BIN) s4.rs -h README.md describe file name guessing, directory filter ([a7d3a39])
- (BIN) s4.rs summary includes run time, threads spawned, thread errors ([c3a4d2a8])
- (BIN) summary.rs print canonical path for archived files ([962de8e3])
- (LIB) src/ allow processing journal, evtx that are compressed, arch [Issue #291] [Issue #284] ([f2e5b4d])
- (LIB) src/ add strace output datetime pattern ([13f1ef4])
- (LIB) datetime.rs match cv_debug.log [Issue #260] ([132a18a])
- (LIB) journal,evtx tempfile when gz,lz4,tar,xz [Issue #284] ([cea1c6f])
- (LIB) evtx,journal,readers.rs Modifed Time always Unix Epoch [Issue #284] ([6947cb9f])
- (LIB) src support lz4, refactor processing linear syslogs [Issue #128] [Issue #201] [Issue #283] [Issue #291] ([a223eeb])
- (LIB) blockreader.rs syslinereader.rs add is_streamed_file() [Issue #182] [Issue #283] ([8a6af55])
- (LIB) filepreprocessor.rs use jwalk for WalkDir ([219e2cf])
- (PROJECT) add logs simple-*log ([b5dd71e])

### Changes

- (BIN) bin/ move --summary printing to src/printer/summary.rs ([35fd92f])
- (BIN) s4.rs NFC more debug logs ([9d3ff3e])
- (BIN) s4.rs summary prints bytes for blocks ([25060e8])
- (BIN) s4.rs truncate summary for empty files ([e5c4c6e])
- (BIN) summary.rs empty files do not print full --summary ([4017242])
- (CI) rust-workflow.sh cargo nextest ([93129e2])
- (CI) rust.yml add 10 cross targets ([5dd008a])
- (CI) rust.yml add job_flamegraph ([2e95d61])
- (CI) rust.yml force flamegraph to succeed ([b875e10])
- (CI) rust.yml job_cross_targets_macos swap moonrepo/setup-rust@v1 for Swatinem/rust-cache@v2 ([dfd5f6b])
- (CI) rust.yml rm builds stable beta nightly ([d13dffa])
- (CI) rust.yml set log file Modified Times before tests ([e30fd62])
- (CI) rust.yml update cross targets for MSRV 1.70.0 ([a9902eb])
- (DOCS) tweak tagline 's4 aims...' ([b371ef1])
- (DOCS) README remove 'wtmp' hyperlink ([67f779c])
- (DOCS) README.md note Windows VSC, simplify some formatting ([bc5ea85])
- (DOCS) README.md NFC move build badges ([58d6ec9])
- (DOCS) README tweak wording ([abdd5ba])
- (DOCS) README add badge lib.rs, badge downloads per version, rearrange badges ([145c45b])
- (DOCS) README update wiki link ([c98bf25])
- (DOCS) README tweak opening wording ([01c11d8])
- (DOCS) README tweak wording 'On Windows' ([2b7a60b])
- (DOCS) README note error too many files open [Issue #270] ([e6d03e4])
- (DOCS) README NFC tweak syntax per linter ([5eaa48f])
- (DOCS) README add more example commands; utmp files, Windows logs [Issue #16] ([8ec6e24])
- (DOCS) README update Issue links ([b4e9cf2])
- (DOCS) README simplify wording in sections 'software examples' ([ea56625])
- (DOCS) README tweak wording at 3 ([236802f])
- (DOCS) README opening paragraph... again! ([81959bd])
- (DOCS) README tweak opening paragraph ([ee9d6c0])
- (DOCS) README shield downloads highlight 'Downloads all time' ([518514e])
- (LIB) bump version 0.6.72 ([a64be8e])
- (LIB) blockreader.rs add read_block_last [Issue #283] ([6d89273])
- (LIB) blockreader.rs attempt to parse more of the XZ header [Issue #12] [Issue #283] ([5750cf3])
- (LIB) blockreader.rs NFC comment note Issue #293 [Issue #293] ([4236304])
- (LIB) filepreprocessor.rs handle empty archive file sooner, add tests ([8265bff2])
- (LIB) filepreprocessor_tests.rs NFC prefer macro e_err! ([a8d5902])
- (LIB) syslinereader.rs add file_offset_last [Issue #283] ([f5d9f89])
- (LIB) refactor `enum FileType` and file type guesstimating, remove Mimeguess [Issue #15] [Issue #257] [Issue #285] ([6eff5c5])
- (LIB) refactor path_to_filetype for archivetype for all file types [Issue #257] [Issue #258] ([dd3637a])
- (LIB) filepreprocessor.rs refactor process_path_tar [Issue #7] [Issue #14] [Issue #16] [Issue #285] ([2c70ee3])
- (LIB) blockreader.rs bad subpath returns InvalidInput ([2ff222e])
- (LIB) Cargo.lock update libc=0.2.155 ([3ada5b0])
- (LIB) Cargo.toml bump MSRV 1.70.0 ([29a5558])
- (LIB) common.rs NFC simplify some match FileType statements ([da392e6])
- (LIB) datetime.rs NFC add TODO for more DTPD! with RP_LEVELS ([0522f1e])
- (LIB) datetime.rs regex RP_LEVELS also match 'PANIC' ([ecbdca1])
- (LIB) filepreprocessor.rs map PermissionDenied to FileErrNoPermissions ([7ca59ef])
- (LIB) src/ NFC debug print lazy_static init ([a069776])
- (LIB) src/ NFC use common FIXEDOFFSET0 ([ae5bbae])
- (LIB) src/ replace ::static_assertions::const_assert! with ::const_format::assertcp! ([016e8b7])
- (LIB) dependabot: bump crossbeam-channel from 0.5.12 to 0.5.13 [(#295)] ([26979a3])
- (LIB) dependabot: bump flate2 from 1.0.28 to 1.0.30 [(#289)] ([bc58af3])
- (LIB) dependabot: bump itertools from 0.12.1 to 0.13.0 [(#296)] ([c2a2fc7])
- (LIB) dependabot: bump libc from 0.2.153 to 0.2.154 [(#292)] ([b63e848])
- (LIB) dependabot: bump unicode-width from 0.1.11 to 0.1.12 [(#290)] ([a2ebe40])
- (PROJECT) .gitignore .virtualenv/ ([43d4cca])
- (PROJECT) add archive tars of wtmp and journal ([c622174])
- (PROJECT) add log files dtf7_\* empty\* many variations ([fb82255])
- (PROJECT) Cargo.toml defalt-run=s4 ([7bdfa4d])
- (PROJECT) logs/ add MacOS13 logs, compressed variations ([808ef68])
- (TEST) filepreprocessor_tests.rs add more tests of oddities and archive combos ([91d33d0])
- (TEST) src/ add test test_new_*Reader_no_file_permissions ([867a6fe])
- (TOOLS) compare-current-and-expected update after bc972fb9 ([2fef744])
- (TOOLS) compare-current-and-expected update.sh always run ./tools/log-files-time-update.sh ([7e42275])
- (TOOLS) flamegraph.sh NFC $PERF dump more info ([f6b28c8])
- (TOOLS) flamegraph.sh title small, add version flamegraph rustc ([e8e7c9c])
- (TOOLS) log-files-time-update.sh MacOS tools compatibility ([57a149c])
- (TOOLS) cargo-test.sh debug print nextest list ([4a79d15])
- (TOOLS) compare-current-and-expected add all variations of log file dtf2-2 ([6f5862e])
- (TOOLS) compare-current-and-expected improve summary of diffs ([8633d66])
- (TOOLS) compare-grep-sort.sh allow user set $FILES ([cb9a91e])
- (TOOLS) compare-log-mergers force install to expected version, update versions ([0f822c5]) (origin/main, origin/HEAD)
- (TOOLS) compare-log-mergers.sh note git commit ([a8a89bd])
- (TOOLS) flamegraphs.sh add flamegraph-syslog-no-matches.svg ([b8a8ea7])
- (TOOLS) flamegraphs.sh bump FREQ, echo more ([6f4a255])
- (TOOLS) rust-workflow call log-files-time-update.sh ([3d962df8])
- (TOOLS) compare-debug-release add more files ([d3313e4c])
- (TOOLS) compare-current-and-expected/update.sh, add various empty and small logs ([54b7d896])

### Fixes

- (BIN) s4.rs minor --help fixes ([aeed87f])
- (BIN) summary.rs fix summary first line printed in wrong color ([6eb17c7c])
- (BIN) summary.rs datetime consistent printing ([d4060a9])
- (DOCS) src/ various docstring fixes and tweaks ([8e46d08])
- (DOCS) README fix typo in link ([dc17064])
- (DOCS) README fix logmerge does support pcap ([5239fb7])
- (TEST) datetime_tests.rs fix test_DATETIME_PARSE_DATAS_test_cases missing tests ([d9c3bf3])

---

## 0.6.71

_Released 2024-04-16_

_MSRV 1.67.1_

[0.6.70..0.6.71]

### New

- (BUILD) Cargo.toml categories add 'date-and-time' ([7d9bf20])
- (LIB) compile regex on-demand [Issue #84] ([5ae0f03])
- (LIB) datetime.rs add common JSONL patterns Timestamp|Datetime ([75d48b0])
- (PROJECT) add logs/MacOS12.6/ ([995bfb0])
- (PROJECT) add CrowdStrike logs ([fd1a1da])

### Changes

- (BIN) bin.rs change --help 'syslog' to 'log' ([ba75da9])
- (CI) chmod +x ./tool/cargo-publish,outdated ([bf7596e])
- (DOCS) README move badges up ([6d67cdb])
- (DOCS) README user accounting link to docs.rs, not code ([91df1d0])
- (DOCS) README link to comparison table instead of Job ([a6099bc])
- (DOCS) README add section stargazers ([13b6039])
- (DOCS) README add section commercial logs Mac OS 12.6 ([b715be5])
- (DOCS) README badge use MSRV badge, link to Cargo.toml ([b199791])
- (DOCS) README oxford commas in opening ([0da5f9d])
- (DOCS) README install advice uses '--locked' ([d6caa25])
- (LIB) datetime.rs do not panic if failed u8_to_str ([7a22f9f])
- (LIB) dependabot: bump bstr from 1.8.0 to 1.9.1 [(#279)] ([475529e])
- (LIB) dependabot: bump chrono from 0.4.35 to 0.4.37 [(#275)] ([2f16d92])
- (LIB) dependabot: bump chrono from 0.4.37 to 0.4.38 [(#281)] ([54c3e09])
- (LIB) dependabot: bump crossbeam-channel from 0.5.8 to 0.5.12 ([d23220a])
- (LIB) dependabot: bump dlopen2 from 0.6.1 to 0.7.0 [(#274)] ([42ccefd])
- (LIB) dependabot: bump encoding_rs from 0.8.33 to 0.8.34 [(#282)] ([2544716])
- (LIB) dependabot: bump evtx from 0.8.1 to 0.8.2 ([84abef3])
- (LIB) dependabot: bump filetime from 0.2.22 to 0.2.23 ([b1acadc])
- (LIB) dependabot: bump memchr from 2.6.4 to 2.7.2 [(#272)] ([5de9a32])
- (LIB) dependabot: bump memoffset from 0.9.0 to 0.9.1 [(#280)] ([9ee5e79])
- (LIB) dependabot: bump regex from 1.10.2 to 1.10.4 ([6408589])
- (LIB) dependabot: bump tempfile from 3.8.1 to 3.10.1 [(#278)] ([0664eeb])
- (LIB) dependabot: bump test-case from 3.2.1 to 3.3.1 ([8821b40])
- (LIB) dependabot: bump walkdir from 2.4.0 to 2.5.0 ([8bf2c81])
- (PROJECT) add releases/0.6.71 ([d8af716])
- (PROJECT) add MacOS11 log and crash files ([145efc2])
- (TEST) datetime.rs add 5 tests using different brackets ([2340084])
- (TOOLS) backup.sh fix flamegraph file listed twice ([12871b7])
- (TOOLS) compare-current-and-expected fix stderr differing commands ([ab11dae])
- (TOOLS) compare-current-and-expected fix stderr comparison to not truncate entire file, run update.sh ([d7d264b])
- (TOOLS) flamegraphs.sh add flamegraph-help.svg ([9e2c253])
- (TOOLS) flamegraph.sh --no-inline ([966c336])

### Fixes

- (LIB) printers.rs fix unset color on first print [Issue #258] ([567394f])

---

## 0.6.70

_Released 2024-03-24_

_MSRV 1.67.1_

[0.6.69..0.6.70]

### New

- (CI) rust.yml cargo package ([cd6584d])
- (CI) rust.yml add summary step for job_rust_msrv_os ([a069057])
- (LIB) datetime.rs add pattern 'year then timezone' DTFSS_BdHMSYZ ([91c3cfb])
- (LIB) datetime.rs add flask web server datetime pattern ([7f700e6])
- (PROJECT) add MacOS11/asl/*.asl files ([8ba2106])
- (PROJECT) add logs/Windows11Pro/{Local,Temp} ([e790e68])
- (PROJECT) add log flask/server.log ([85bb57f])
- (PROJECT) add releases/0.6.70/ files ([5f8a63e])
- (TOOLS) add tools/cargo-publish.sh ([aa4cf9d])

### Changes

- (BIN) rename bin.rs -> s4.rs ([ad55722])
- (LIB) printers.rs improve colors for dark background ([cd52d9d])
- (LIB) bump chrono 0.4.35 ([89209f9])
- (LIB) dependabot: bump rangemap from 1.4.0 to 1.5.1 (#254) ([8d48787])
- (LIB) dependabot: bump lru from 0.12.0 to 0.12.3 (#252) ([fada0fc])
- (LIB) dependabot: bump utf8_iter from 1.0.3 to 1.0.4 (#253) ([f271285])
- (TOOLS) rust-workflow.sh clean first ([f05b395])

---

## 0.6.69

_Released 2024-03-21_

[0.6.68..0.6.69]

_MSRV 1.67.1_

### New

- (CI) rust.yml enable cross targets ([8bee504])
- (DOCS) syslogprocessor.rs docstring about "syslog file" ([c618a19])
- (DOCS) datetime.rs NFC docstring Result_Filter_DateTime1 ([4c7b6f1])
- (LIB) refactor for fixedstruct [Issue #100] [Issue #108] [Issue #109] [Issue #121] [Issue #171] [Issue #217] ([1aed862])
- (LIB) replace uapi for utmpx processing [Issue #100] [Issue #108] [Issue #109] [Issue #121] [Issue #171] [Issue #217] ([e8406c5])
- (LIB) datetime.rs,bin add DateTimeParseDatasCompiledCount summary stat ([9a23849])
- (PROJECT) add log CT-Log 2023-09-04 05-44-16.csv ([48c5a5e])
- (PROJECT) add log ./logs/other/tests/dtf9c-23-12x2.log.xz ([0c1f565])
- (PROJECT) add logs MacOS11 MacOS12.6 NetBSD9.3 FreeBSD13.1 OpenBSD7.2 ([7d3305c])
- (PROJECT) add logs/Android9 ([c9a5de8])
- (PROJECT) add logs gen-1000-3-foobar.log.{tar,xz} ([8ec4d01])
- (PROJECT) add logRFC-5424-Z.log ([a08cc1d])
- (PROJECT) add log ISO8601-YYYY-DD-MMTHH-MM-SS_2.log ([2d1541c])
- (PROJECT) add releases/0.6.69/flamegraph*,compare-log-mergers.txt ([34ef716])
- (PROJECT) add requirements.txt ([bd29d11])
- (TEST) blockreader_tests.rs mark should_panic [Issue #201] ([a746b06])
- (TEST) add datetime pattern dtf14a ([70a30a2])
- (TEST) datetime_tests.rs add test cases to test_slice_contains_X_2 ([a468ffa])
- (TOOLS) add tools/flamegraphs.sh ([b0ade00])
- (TOOLS) add tools/cargo-udeps.sh ([df2bcde])

### Changes

- (BIN) bin.rs NFC debug print syslogprocessor stages more clearly ([9bfb3ca7c5eeeaa20a9a5e6071206a98a3e7fa17]
- (BIN) bin.rs ordered invalid results ([e75daad])
- (BIN) bin.rs do not print after printing error [Issue #241] [Issue #250] ([157d54a])
- (CI) rust.yml NFC move log-files-time-update.sh ([25b35ee])
- (CI) rust.yml build 1.76.0, NFC vertical declaration ([baff6da])
- (CI) bump MSRV 1.67.1 ([505280a])
- (CI) rust.yml pin yamllint==1.35.1 ([f32d765])
- (CI) rust.yml clippy uses default profile ([89f67f3])
- (CI) rust.yml use moonrepo/setup-rust@v1 ([cef66e5])
- (CI) rust.yml add job_test_wasm ([aec8ceb])
- (CI) rust.yml comment job_test_wasm [Issue #171] ([da6f91f])
- (CI) rust.yml use download-artifact@v4 ([8c23fd0])
- (CI) rust.yml NFC adjust S4_TEST_FILES indent ([92b9754])
- (CI) rust.yml update setup-python@v5 ([f46be6a])
- (CI) rust.yml update upload-artifact@v4 ([e4f8059])
- (CI) rust.yml add more S4_TEST_FILES ([8f8e8b7])
- (CI) rust.yml update checkout@v4 ([7432b7c])
- (CI) rust.yml run for Cargo.lock ([6a327d1])
- (CI) rust.yml correctly override rust version ([068a0a2])
- (CI) rust.yml add job_cross_targets ([d1dfe28])
- (CI) rust.yml remove tar logs.tgz upload ([ef7d59a])
- (CI) rust.yml test on Windows, Mac OS, binstall nextest [Issue #202] [Issue #219] [Issue #218] ([659c824])
- (CI) rust.yml job grcov uses rust 1.67.0 [Issue #170] ([78579a9])
- (CI) rust.yml uploaded artifacts have specific name ([007f752])
- (CI) rust.yml remove job_grcov ([b2a0ab5])
- (CI) rust.yml use script compare-debug-release.sh ([3591489])
- (CI) rust.yml tweak `on` for `pull_request` ([616bef1])
- (CI) rust.yml use newer rust for job llvm_cov ([f363682])
- (CI) rust.yml use script compare-debug-release.sh ([231fcc4])
- (CI) rust.yml tweak `on` for `pull_request` ([24c21a1])
- (CI) rust.yml msrv add 1.72.0, drop 1.67.1 ([f968b46])
- (CI) rust.yml use newer rust for job llvm_cov ([69767ad])
- (CI) rust.yml remove call cargo-llvm-cov-run.sh ([2ff0d19])
- (CI) rust.yml remove tar logs.tgz upload ([9fbc631])
- (CI) rust.yml test on Windows, Mac OS, binstall nextest ([2e10dd1])
- (CI) rust.yml NFC move log-files-time-update.sh ([00171bb])
- (DOCS) README add more sections to examples in "logging chaos" ([071536e7a77bc4fefcbba6874040d0a4c77ce4b])
- (DOCS) README fix missing link to local file ([c13d082])
- (DOCS) README update section HACKS ([c4f86ec])
- (LIB) update chrono 0.4.28 ([0021f05])
- (LIB) bump chrono 0.4.27, add MINUS SIGN tests ([498ad5f])
- (LIB) bump MSRV 1.67.0, lock clap 4.1.0 ([4db0056])
- (LIB) Cargo.toml criterion 0.5.1 ([2f1fc58])
- (LIB) Cargo.toml clap downgrade 4.1.0 to support MSRV 1.66.0 ([afc0dab])
- (LIB) Cargo.toml remove dependency `backtrace` ([b339c88])
- (LIB) Cargo remove flamegraph as a dependency ([f08dd9c])
- (LIB) dependabot: bump bstr from 1.6.0 to 1.7.0 [(#199)] ([cff9336])
- (LIB) dependabot: bump bstr from 1.7.0 to 1.9.0 [(#237)] ([a09e9f6])
- (LIB) dependabot: bump clap from 4.3.12 to 4.3.19 ([2c1a38e])
- (LIB) dependabot: bump clap from 4.3.19 to 4.3.21 [(#172)] ([1c6ccb1])
- (LIB) dependabot: bump clap from 4.3.21 to 4.3.23 [(#175)] ([0f4521c])
- (LIB) dependabot: bump clap from 4.3.23 to 4.4.6 [(#193)] ([50e3c52])
- (LIB) dependabot: bump dlopen2 from 0.5.0 to 0.6.1 [(#166)] ([f420393])
- (LIB) dependabot: bump encoding_rs from 0.8.32 to 0.8.33 [(#177)] ([bdafe96])
- (LIB) dependabot: bump filetime from 0.2.21 to 0.2.22 ([0c6ba91])
- (LIB) dependabot: bump flate2 from 1.0.26 to 1.0.27 [(#176)] ([53e8a75])
- (LIB) dependabot: bump itertools from 0.11.0 to 0.12.1 [(#246)] ([e375577])
- (LIB) dependabot: bump libc from 0.2.149 to 0.2.153 [(#247)] ([2469178])
- (LIB) dependabot: bump lru from 0.11.0 to 0.12.0 [(#197)] ([08fed12])
- (LIB) dependabot: bump memchr from 2.5.0 to 2.6.4 [(#195)] ([8123ef1])
- (LIB) dependabot: bump nix from 0.26.2 to 0.27.1 [(#178)] ([68583d8])
- (LIB) dependabot: bump regex from 1.9.1 to 1.9.3 [(#169)] ([ddc2a80])
- (LIB) dependabot: bump regex from 1.9.3 to 1.9.4 [(#181)] ([eb5a1c3])
- (LIB) dependabot: bump regex from 1.9.4 to 1.10.2 [(#203)] ([a01e57d])
- (LIB) dependabot: bump rustix from 0.36.9 to 0.36.16 [(#200)] ([3e70a60])
- (LIB) dependabot: bump tar from 0.4.39 to 0.4.40 [(#173)] ([e3b58f9])
- (LIB) dependabot: bump tempfile from 3.6.0 to 3.7.0 ([5543ff7])
- (LIB) dependabot: bump tempfile from 3.7.0 to 3.7.1 [(#168)] ([46c1502])
- (LIB) dependabot: bump tempfile from 3.7.1 to 3.8.0 [(#174)] ([5d5ce99])
- (LIB) dependabot: bump webpki from 0.22.0 to 0.22.4 [(#204)] ([6671e40])
- (LIB) dependabot: bump zerocopy from 0.7.25 to 0.7.32 [(#249)] ([f3749f5])
- (LIB) src/ remove debug prints with addresses [Issue #213] ([fdc1899])
- (LIB) fixedstruct.rs swap numtoa for lexical ([4e32f5c])
- (LIB) fixedstruct.rs use i64 to satisfy cross targets ([6c1954e])
- (LIB) src/ NFC code comment updates, docstring links ([bd9b494])
- (LIB) read gz files sequentially [Issue #182] ([02261be])
- (LIB) blockreader.rs docstring tweaks BlockReader ([4b80fa5])
- (LIB) blockreader.rs don't panic for missing Block, return Error ([7f9535d])
- (LIB) blockreader.rs NFC debug print mtime() [Issue #245] ([daded23])
- (LIB) datetime.rs remove to_byte_array; use built-in const bytes ([699015e])
- (LIB) syslinereader.rs InvalidInput error tells short line length ([6c82506])
- (LIB) syslogprocess.rs stage runtime check is debug-only ([463d93f])
- (LIB) syslogprocessor.rs Year min_diff is global static ([1e552fe])
- (LIB) syslinereader.rs use stable feature `pop_last` ([4268bc6])
- (LIB) syslinereader.rs fix mixup summary regex_captures_attempted get_boxptrs_singleptr ([f94d2a0])
- (LIB) syslogprocessor.rs utmpxreader.rs set_error prints error ([4370ada])
- (LIB) src/ cargo fmt suggestions ([8fc4181])
- (LIB) journalreader.rs explicit static ([97b69b2])
- (LIB) codespell fixes ([8f0eee7])
- (LIB) src/ tweak drop_block return value logic ([c240b82])
- (LIB) sd_journal_h.rs allow(clippy::all) ([487138f])
- (LIB) line.rs sysline.rs debug print no address ([e083e73])
- (LIB) src/ refactor error messages: include paths ([c83e433])
- (PROJECT) logs truncate large logs, rm very large .journal ([9c4e8e9])
- (TEST) datetime_tests.rs use test_case::test_matrix ([a5ad717])
- (TEST) tests strategic skip journalreader utmpx on macos windows ([5618d05])
- (TEST) syslinereader_tests.rs fix test ntf_gz_8byte_fpath ([58d9d79])
- (TEST) src/ fix code and tests that fail on target_os="windows" [Issue #202] ([a7cfd10])
- (TEST) tests/mod.rs utmpxreader_tests, utmpx_tests not on "macos" [Issue #217] ([2c8bf0f])
- (TOOLS) log-files-clean*sh less aggressive cleaning ([2c34a47])
- (TOOLS) flamegraph.sh modify SVG title with more info ([04558a3])
- (TOOLS) flamegraph.sh xmllint ([2a2d372])
- (TOOLS) compare-*.sh allow colordiff ([1d6bc01])
- (TOOLS) hexdump.py fix offset option, allow hex ([fcfc729])
- (TOOLS) backup.sh allow exported BACKUPDIR ([50cc3c2])
- (TOOLS) backup.sh backup flamegraph* releases/ ([4a43a7b])
- (TOOLS) compare-current-and-expected use individual logs [Issue #213] ([d073674])
- (TOOLS) compare-current-and-expected check file hashes, fix stdin [Issue #206] ([34b75c5])
- (TOOLS) compare-current-and-expected comment "ERROR: " [Issue #224] ([c7ec4eb])

## 0.6.68

_Released 2023-07-22_

[0.6.67..0.6.68]

_MSRV 1.67.0_

### New

- (BIN) bin.rs parse -a -b fractional seconds %6f [Issue #145] ([54b0860])
- (BIN) bin.rs better pattern coverage for -a -b ([05d974c])

### Changes

- (LIB) dependabot: bump clap from 4.3.10 to 4.3.12 ([844c1e0])
- (LIB) dependabot: bump tar from 0.4.38 to 0.4.39 ([26daee6])
- (LIB) dependabot: bump const-str from 0.5.5 to 0.5.6 ([2f9a434])
- (LIB) dependabot: bump lru from 0.10.0 to 0.11.0 ([1e3e789])
- (LIB) dependabot: bump bstr from 1.5.0 to 1.6.0 ([6f36d21])
- (LIB) dependabot: bump itertools from 0.10.5 to 0.11.0 ([d205c41])
- (LIB) dependabot: bump regex from 1.8.4 to 1.9.1 ([a0edb15])
- (LIB) bump si_trace_print to 0.3.10 ([0997ecd])
- (BIN) bin.rs adjust --help wording, spacing ([93a58cc])
- (DOCS) README update --help ([9948f0a])
- (DOCS) README add color to some other logs ([3805df7])
- (DOCS) README coveralls.io badge ([7d97c64])
- (CI) rust.yml nextest all early on ([b8a8c34])
- (CI) rust.yml consistent toolchains, use MSRV ([7daa34c])
- (CI) rust.yml add coverage llvm-cov to coveralls ([d0f4166])
- (BIN) bin.rs NFC clippy recommendations ([3aaddac])
- (BIN) bin.rs NFC fixup test case names ([4ac8430])

## 0.6.67

_Released 2023-07-08_

[0.6.66..0.6.67]

_MSRV 1.66.0_

### New

- (BIN) bin.rs parse -a -b fractional seconds %3f [Issue #145] ([4b328c5])

### Changes

- (LIB) dependabot: bump bstr from 1.4.0 to 1.5.0 ([f5bd4e3])
- (LIB) dependabot: bump chrono from 0.4.24 to 0.4.25 ([bea60aa])
- (LIB) dependabot: bump chrono from 0.4.25 to 0.4.26 ([9522093])
- (LIB) dependabot: bump clap from 4.2.7 to 4.3.0 ([87a884b])
- (LIB) dependabot: bump clap from 4.3.0 to 4.3.8 ([7cc5fbc])
- (LIB) dependabot: bump clap from 4.3.8 to 4.3.10 ([d340bd2])
- (LIB) dependabot: bump const-str from 0.4.3 to 0.5.4 ([56adf88])
- (LIB) dependabot: bump const-str from 0.5.4 to 0.5.5 ([2030a73])
- (LIB) dependabot: bump const_format from 0.2.30 to 0.2.31 ([861be67])
- (LIB) dependabot: bump dlopen2 from 0.4.1 to 0.5.0 ([eeb3632])
- (LIB) dependabot: bump filetime from 0.2.20 to 0.2.21 ([11c9d0b])
- (LIB) dependabot: bump flamegraph from 0.6.2 to 0.6.3 ([addbc64])
- (LIB) dependabot: bump libc from 0.2.144 to 0.2.147 ([5359667])
- (LIB) dependabot: bump lzma-rs from 0.2.0 to 0.3.0 ([d31c145])
- (LIB) dependabot: bump phf from 0.11.1 to 0.11.2 ([ebf4fd3])
- (LIB) dependabot: bump regex from 1.8.1 to 1.8.3 ([f4cdd62])
- (LIB) dependabot: bump regex from 1.8.3 to 1.8.4 ([ee4c9d0])
- (LIB) dependabot: bump tempfile from 3.5.0 to 3.6.0 ([ff0f469])
- (LIB) dependabot: bump test-case from 2.2.2 to 3.1.0 ([2e77297])
- (TOOLS) flamegraph.sh use `--profile flamegraph` ([d4f5e0a])
- (TEST) bin.rs add 3 more test cases for datetime parsing ([943619b])
- (CI) rust.yml update grcov flags ([707d472])
- (CI) rust.yml upload binary for all platforms at MSRV [Issue #152] ([6ff633c])
- (CI) rust.yml expand version matrix ([597c080])
- (CI) rust.yml NFC reword job name ([1677328])
- (CI) rust.yml add job codecov.yml validation ([a4eabe1])
- (CI) rust.yml fix upload if statements ([a7e93c6]) (HEAD -> main)
- (CI) rust.yml update action rust-toolchain@stable ([40e428d])

### Fixes

- (BIN) bin.rs fix parsing of example input strings ([99b8c46])
- (LIB) summary.rs clippy fix unused warning BLOCKSZ_MIN ([e317c87])
- (LIB) src/ cargo clippy fix missing ymdhmsl ([b50ce99])
- (CI) rust.yml relax yamllint line-length [Issue #120] ([01b80ff])

---

## 0.6.66

_Released 2023-05-13_

[0.6.65..0.6.66]

_MSRV 1.66.0_

### Changes

- (LIB) (TESTS) (TOOLS) improve tests and logs for `BlockReader::mtime()` and files that are `FileType::Unknown` ([b2530a5]) ([f5abc7a])

### Fixes

- (LIB) blockreader.rs fix panic on `FileType::Unknown` in mtime() ([307c86c])

---

## 0.6.65

_Released 2023-05-11_

[0.6.64..0.6.65]

_MSRV 1.66.0_

### New

- (CI) rust.yml add job_yamllint [Issue #120] ([8bdeafd]) ([6e68808])
- (TEST) add logs dtf13a.log dtf13b.log dtf13c.log dtf13d.log ([c3d0621])

### Changes

- (LIB) datetime.rs allow lenient matching timezone ([fe422d6])
- (TOOLS) compare-current-and-expected.sh --prepend-dt-format='%Y%m%dT%H%M%S.%9f' ([e5e7f45])

---

## 0.6.64

_Released 2023-05-09_

[0.6.63..0.6.64]

_MSRV 1.66.0_

### New

- (BIN) bin.rs print datetimes as UTC dimmed ([e51c30f])
- (LIB) parse Red Hat Audit logs, parse epochs [Issue #112] ([0fceba2]) ([69ef9f7])
- (BIN) (LIB) src/ Summary statistics for regex capture attempts ([281adc0])
- (TOOLS) yamlllint.yml add rules for yamllint.sh ([fa5ff73])

### Changes

- (LIB) dependabot: bump clap from 4.2.1 to 4.2.7  ([33447dd])
- (LIB) dependabot: bump crossbeam-channel from 0.5.7 to 0.5.8 ([06640e3])
- (LIB) dependabot: bump libc from 0.2.141 to 0.2.144 ([8e98a8f])
- (LIB) dependabot: bump lru from 0.8.1 to 0.10.0 ([75f7c9f])
- (LIB) dependabot: bump regex from 1.7.1 to 1.8.1 ([66414e9])
- (LIB) dependabot: bump tempfile from 3.4.0 to 3.5.0  ([210f01c])
- (BIN) bin.rs refactor channel data passing  [Issue #104] [Issue #60] ([0ea897a])
- (LIB) journalreader.rs efficient key tracking in `next_short` [Issue #84] ([7810632])
- (LIB) (BIN) miscellaneous codepspell fixes ([524e269]) ([0c6af5d]) ([5bb8a5d]) ([860b213]) ([af93d66])
- (LIB) datetime.rs remove duplicate enum `DTFS_Hour::Hs` ([cc1cb8a])
- (LIB) syslogprocessor.rs add `blockzero_analysis_bytes` ([cdd64df])
- (TEST) datetime_test.rs test builtins using slice ([b8989f3])
- (TOOLS) compare-current-and-expected common args, more args ([d395d94])
- (TOOLS) compare-debug-release.sh pass more args to s4 ([dfab1e7])
- (CI) github add `dependabot.yml` ([877177b])

### Fixes

- (LIB) syslinereader.rs fix sort of indexes ([f1baa4d])
- (LIB) datetime.rs fix too short slice recommendation ([2af24cb])

---

## 0.6.63

_Released 2023-05-01_

[0.6.62..0.6.63]

_MSRV 1.66.0_

### New

- (LIB) datetime.rs match single-digit days and hours [Issue #98] ([830dbbd])

### Changes

- (LIB) datetime.rs use compile-time map timezone names to values [Issue #84] ([98ebe68])
- (LIB) syslinreader.rs pre-create FixedOffset strings [Issue #84] ([3b95001])
- (LIB) datetime.rs support RFC 2822 timezone "UT" ([dd8248c])

### Fixes

- (LIB) datetime.rs fix missing and out-of-order timezones ([cf9153b])
- (LIB) src/ allow `FileType::Unknown`, fix panic on Unknown ([2cb0412])

---

## 0.6.62

_Released 2023-04-27_

[0.6.61..0.6.62]

_MSRV 1.66.0_

### Fixes

- (LIB) fails to build on Debian 11 aarch64 [Issue #108] ([67cb45a])

---

## 0.6.61

_Released 2023-04-23_

[0.6.60...0.6.61]

_MSRV 1.66.0_

### New

- (LIB) filepreprocessor.rs handle trailing junk like "~" ([23dfeb3])
- (TEST) add `summary()` tests for various readers ([57e2a4d]) ([8be5e30]) ([efb694d])
- (PROJECT) add logs `SIH.20230422.034724.362.1.etl` `FreeBSD13_utx.lastlogin.utmp` ([ce8518e]) ([ccd4f0c])

### Changes

- (TOOLS) journal_print.py allow user passing fields ([3c5a18a])

### Fixes

- (LIB) s4 panics on some .etl files [Issue #105] ([5f77a0f])

---

## 0.6.60

_Released 2023-04-22_

[0.5.59...0.6.60]

_MSRV 1.66.0_

### New

- (LIB) (BIN) (DOCS) (TOOLS) (PROJECT) (BUILD) systemd journal parsing [Issue #17] ([3a6eac6])

---

## 0.5.59

_Released 2023-03-31_

[0.5.58...0.5.59]

_MSRV 1.66.0_

### Changes

- (LIB) Efficiency hack EZCHECK12D2 ([6f7831f]) ([08738c4])
- (BIN) bin.rs summary spacing, linerize About section ([dc7b7c2])
- (TOOLS) compare-current-and-expected-update checks stdout and stderr ([f5f2be2]) ([cf91c1d]) ([85d5ba2]) ([f5bf771]) ([52777c1]) ([3df00ac]) ([05f04e3]) ([81f94b8])
- (CI) split up jobs into more parallel jobs [Issue #63] ([2edda45]) ([c1262d4])

### Fixes

- (LIB) datetime.rs fix one-off error in slice_contiains_X_2 ([d159553])
- (DOCS) src/ fix various docs, pub some functions ([476ed60])

---

## 0.5.58

_Released 2023-03-29_

[0.4.57...0.5.58]

_MSRV 1.66.0_

### New

- (BIN) Allow user-passed timezone for prepended datetime [Issue #27] ([630b8ce])
- (PROJECT) add logs/programs/{AWS,Microsoft IIS,apache} ([ee4515f])
- (LIB) Parse RFC 2822 [Issue #29] ([38d1c47])

### Changes

- (TEST) add test test_PrinterLogMessage_print_evtx ([e6931ed])
- (BUILD) clap 0.4.21 ([f8f977d]) ([8f75091])

---

## 0.4.57

_Released 2023-03-26_

[0.4.56...0.4.57]

_MSRV 1.66.0_

### Changes

- (LIB) Print evtx files in chronological order [Issue #86] ([e42d021])
- (BIN) bin.rs summary linerize more ([cebf281])
- (LIB) datetime.rs NFC refactor dt_pass_filters ([bbdb2cb])
- (PROJECT) CHANGLOG revise h2 naming ([ef80a0d])

### Fixes

- (BIN) bin.rs tweak --help wording ([3c34d09])

---

## 0.4.56

_Released 2023-03-24_

[0.3.55...0.4.56]

_MSRV 1.66.0_

### New

- (LIB) process Windows Event Log evtx files [Issue #86] [Issue #87] ([368eba9])

### Changes

- (TEST) add various `Summary` tests ([7f3911b]) ([9f5391b]) ([5b2b6f8]) ([0923408]) ([d03737c])

### Fixes

- (BIN) bin.rs fix -a "" (passing empty string) ([d4ed03e])
- (BIN) bin.rs fix panic for multiple non-existent paths ([da980d8])

---

## 0.3.55

_Released 2023-03-18_

[0.3.54...0.3.55]

_MSRV 1.66.0_

### New

- (LIB) syslinereader.rs track EZCHECK12 use ([29072ac])
- (LIB) datetime.rs add patterns derived from hawkeye.log  ([d091a79])
- (TOOLS) add tools/cargo-outdated.sh ([44fa812])
- (TOOLS) add tools/cargo-upgrade.sh ([cb698ec])

### Changes

- (LIB) datetime.rs use slice as ref TryInto ([b723fed])
- (BUILD) Cargo cargo update and upgrade ([f2199b3])

### Fixes

- (BIN) bin.rs fix --separator, use for utmpx ([24f00e7])
- (LIB) utmpxreader.rs fix errant println! ([50fec20])

---

## 0.3.54

_Released 2023-03-17_

[0.2.53...0.3.54]

_MSRV 1.66.0_

### New

- (BIN) use shrink_to_fit on maps [Issue #84] ([e9b501f])
- (BUILD) Cargo.toml improve release optimizations ([06e500f])

### Fixes

- (BIN) bin.rs user-passed %Z for filters [Issue #85]([98c4b36])
- (LIB) fix FreeBSD compile of `uapi` and define `umptx` ([62e89e2])

---

## 0.3.53

_Released 2023-03-10_

[0.2.52...0.3.53]

_MSRV 1.66.0_

### New

- (LIB) add support for utmpx login records (major refactoring) [Issue #70] ([b227f53])

### Changes

- (BUILD) MSRV 1.66.0

---

## 0.2.52

_Released 2023-02-15_

[0.2.51...0.2.52]

_MSRV 1.64.0_

### New

- (LIB) datetime.rs add format catalina apache access  [Issue #82] ([5337dd9]) ([997a365])

---

## 0.2.51

_Released 2023-02-09_

[0.2.50...0.2.51]

_MSRV 1.64.0_

### New

- (BIN) bin.rs option --sysline-separator [Issue #80] ([b6d359f])
- (BIN) print bytes counts in hex ([e46b1f9])
- (LIB) datetime pattern for tomcat catalina [Issue #81] ([8def2f6])

---

## 0.2.50

_Released 2023-01-29_

[0.2.49...0.2.50]

_MSRV 1.64.0_

### New

- (BUILD) github code coverage --all-targets [Issue #77] ([6baae7c])
- (TEST) printers_tests.rs add initial tests for printers.rs ([5cabf7b])
- (TEST) syslinereader_tests.rs add basic tests SyslineReader::new ([0bee449])
- (TEST) tests/common.rs add eprint_file_blocks ([361e986])
- (TOOLS) add valgrind-massif.sh ([84f3059])
- (TOOLS) add heaptrack.sh ([749f8ce])
- (TOOLS) add s4-wait.sh ([e1dfb0a])

### Changes

- (BUILD) bump MSRV to 1.64.0 ([ac5749d])
- (BUILD) rust.yml remove build 1.68.0 ([aee27e4])
- (LIB) bump si_trace_print = "0.3.9" ([9055612])
- (LIB) syslinereader.rs add ezcheck12_min ([713bb73])

### Fixes

- (BIN) bin.rs bounded channel queue [Issue #76] ([ac509a2])

---

## 0.2.49

_Released 2023-01-26_

[0.2.48...0.2.49]

_MSRV 1.64.0_

### Changes

- (LIB) src/ refactor find_line_in_block find_sysline_in_block partial ([cda6e99])
- (LIB) rust.yml remove build 1.68.0 ([aee27e4])
- (LIB) syslinereader.rs add ezcheck12_min ([713bb73])
- (LIB) debug/printers.rs improve buffer_to_String_noraw improve `buffer_to_String_noraw` to be more robust ([0c7efef])
- (LIB) src/ refactor patterns analysis [Issue #74] [Issue #75] ([8575cd8])
- (LIB) src/ revise all si_trace_prints calls ([df628a7])
- (TEST) debug/helpers.rs add create_temp_file_data ([e3c0e0a])
- (TEST) syslogprocessor_tests.rs add tests for short files ([d48099c])

### Fixes

- (LIB) syslinereader.rs fix one-off error in get_boxptr ([619da41])
- (LIB) syslogprocessor.rs fix expected sysline count in blockzero ([e3ca0e2])
- (TEST) common.rs fix PartialEq for FileProcessingResult ([85d51b6])

---

## 0.2.48

_Released 2023-01-15_

_MSRV 1.61.0_

### New

- (BUILD) rust.yml build on more versions of rust ([c80a044])
- (LIB) datetime.rs add more general purpose matches ([d6bb2d1])
- (LOGS) add logs/FreeBSD12.3 ([9128a71])
- (LOGS) add logs dtf12-*.log ([715cff5])
- (LOGS) add logs/programs/ntp/ ([8fbb9f8])
- (TEST) syslinereader_tests.rs add basic tests SyslineReader::new ([0bee449])
- (TEST) syslogprocessor_tests.rs add tests for short files ([d48099c])
- (TOOLS) add cargo-call-stack.sh ([6b47b9c])

### Changes

- (BUILD) rust.yml explicit shell, print version more often ([9ba41e3])
- (DOCS) README fix shields link for github ([08d198a])
- (DOCS) blockreader.rs doc string links ([7d8d35a])
- (LIB) src/ revise all si_trace_prints calls ([df628a7])
- (LIB) debug/printers.rs improve buffer_to_String_noraw ([0c7efef])
- (LIB) syslinereader.rs debug print DTPI attempts ([35fbb1d])
- (LIB) datetime.rs add test for a DTPD ([d9f70ce])
- (LIB) datetime.rs refactor DTFSSet::fmt::Debug ([0d9d80b])
- (LIB) datetime.rs more precise syslog matching, tests, notes ([7db097f])
- (LIB) syslinereader.rs manual pop_last ([c225eb6])
- (TEST) tests/common.rs add eprint_file_blocks ([361e986])
- (TEST) blockreader_tests.rs add basic new BlockReader tests [Issue #22] ([d3f723e])
- (TEST) linereader.rs add basic new LineReader tests ([b7a25d0])
- (TEST) debug/helpers.rs add create_temp_file_data ([e3c0e0a])
- (TOOLS) cargo-test.sh remove --test-threads=1 ([308628c])

### Fixes

- (LIB) src/ refactor patterns analysis [Issue #75] [Issue #74] ([8575cd8])
- (LIB) src/ refactor find_line_in_block find_sysline_in_block partial [Issue #22] ([cda6e99])
- (LIB) common.rs fix PartialEq for FileProcessingResult ([85d51b6])
- (LIB) syslogprocessor.rs break when fo_prev >= fo_prev_prev [Issue #75] ([8225996])
- (LIB) syslinereader.rs fix one-off error in get_boxptr ([619da41])
- (LIB) syslogprocessor.rs fix expected sysline count in blockzero ([e3ca0e2])

---

## 0.2.47

_Released 2023-01-09_

[0.2.46...0.2.47]

_MSRV 1.61.0_

- (BIN) bin.rs fix typo in clap help ([b03da48])
- (DOCS) README update --help ([cdaad46])

---

## 0.2.46

_Released 2023-01-09_

[0.1.45...0.2.46]

_MSRV 1.61.0_

### New

- (BIN) bin.rs print canonical path ([9ceb5b4])
- (BIN) bin.rs use verbatim_doc_comment for all args ([a88c662])
- (LIB) datetime.rs refactor day matching [Issue #58] [Issue #42] [Issue #47] ([1e58094])
- (LIB) datetime.rs support synoreport.log [Issue #45] ([7d6d9f2])
- (LIB) datetime add pattern for pacman.log [Issue #41] ([989ecdd])
- (LIB) datetime pattern for explicit syslog format ([ef73bd5])
- (LIB) datetime.rs add Windows 10 ReportingEvents.log, fix missing tests ([50870c1]) ([c4c7f30])
- (CI) rust-workflow.sh add MSRV verify ([6e284ff])
- (CI) github add task compare-current-and-expected.sh ([133cb5c]) ([82255c0]) ([26ec11b]) ([b1dc6f9]) ([ee95b63])
- (CI) rust-workflow.sh run compare scripts ([3ac5374])
- (CI) github run log-files-time-update.sh ([705dd66]) ([69abc77])
- (TOOLS) add tools/compare-debug-release.sh ([acc9b5b]) ([5e97590]) ([b1dc6f9]) ([8514bb9])
- (TOOLS) tools add cargo-msrv.sh ([e43d48b])
- (TOOLS) add tools/compare-current-and-expected ([f0146fc]) ([8c9a919]) ([c87ceff]) ([dd63214]) ([80d06dd])
- (TEST) add logs Windows10Pro/Panther/* ([2975c9a])
- (TEST) add log MPDetection-12162017-091732.log ([8e6fc80])
- (TEST) add logs Orbi/tmp ([19adf7e])
- (TEST) add busybox logs ([e1e4606])

### Changes

- (BIN) filepreprocessor.rs allow crossing filesystems [Issue #71] ([c6f1899])
- (BIN) bin.rs refactor summary printing ([5f5606d])
- (BUILD) Cargo downgrade const-str for MSRV support ([dfd2898])
- (LIB) datetime refactor ([1e58094])
- (LIB) src/ refactor process_path ([e80df38])
- (LIB) blockreader.rs change tar path separator char ([9a37f84])
- (LIB) src/ store path in Summary ([34320a7])
- (LIB) filepreprocessor.rs fix unreachable match ([70dcb6e])
- (LIB) cargo fmt, cargo clippy warnings ([09a885d]) ([a256992]) ([bab5ee5]) ([8a50df1]) ([a8a5f36])
- (PROJECT) add tools/.gitignore ([44bd6b1])

### Fixes

- (CI) github fix MSRV check ([58d2f20])
- (PROJECT) .gitignore add leading path for dirs ([f355964])
- (TOOLS) hexdump.py flush stderr, stdout ([bbe3b00])

---

## 0.1.45

_Released 2023-01-01_

[0.1.44...0.1.45]

_MSRV 1.61.0_

### New

- (LIB) datetime.rs add pattern for apport.log [Issue #55] ([7557a59])
- (LIB) datetime.rs add pattern for openftp.log ([Issue #48]) ([fda61f8])

### Changes

- (LIB) be more sure of matching year ([fda61f8])
- (TEST) more stringent and precise DTPD check ([fda61f8]) ([60aa5d1])

---

## 0.1.44

_Released 2022-12-29_

[0.1.43...0.1.44]

_MSRV 1.61.0_

### New

- (LIB) datetime.rs add 6 DTPD! entries ([d3f5d8a])

### Changes

- (BIN) print summary information for files with bad permissions [Issue #69]([3ee20b9])
- (BIN) better align summary output ([f6a72ff])
- (CI) github do more runs of s4 with default blocksz ([48bd3aa])
- (LIB) src/ refactor FileType ([d882f96])
- (LIB) src/ add ProcessPathResult::FileErrNotExist,FileErrNotParseable ([e9ea121])
- (TOOLS) tools add rust-workflow.sh ([8e3b72a])

### Fixes

- (LIB) syslinereader do not panic in case of unexpected Done [Issue #67] ([2da3398])

---

## 0.1.43

_Released 2022-12-26_

[0.1.42...0.1.43]

_MSRV 1.61.0_

### New

- (CI) add codecov.yml ([bd49cdc])

### Changes

- (BUILD) adjust "filetime" dependency ([ff2cd81])
- (DOCS) update README ([cb8ed82])
- (BIN) update --help for `--blocksz` ([2157861])

---

## 0.1.42

_Released 2022-12-19_

[0.1.41...0.1.42]

_MSRV 1.61.0_

### Changes

- (BIN) (BUILD) update clap from 3 to 4 ([f58f506])
- (BUILD) cargo update ([41bb25a])

---

## 0.1.41

_Released 2022-12-18_

[0.1.40...0.1.41]

_MSRV 1.61.0_

### Changes

- (TOOLS) gen-log.sh add option for extra lines ([610785f])
- (BIN) re-arrange trailing summary print of datetimes ([a80facf])
- (LIB) for logs without year, skip processing outside filters [Issue #65] ([33418d0])
- (PROJECT) add gen-20-2-2-faces.log ([8a47ad8])

### Fixes

- (LIB)(BIN) fix drop, add more summary stats, tests [Issue #66] ([916259b])

---

## 0.1.40

_Released 2022-11-22_

[0.1.39...0.1.40]

_MSRV 1.61.0_

### New

- (BIN) add CLI option `--prepend-separator` ([467b14d])

### Changes

- (BIN) add summary _syslines stored high_ ([d1f5895f1e5a55cbbcbfc4072bbde53a7a85fc])

---

## 0.1.39

_Released 2022-10-19_

[0.1.38...0.1.39]

_MSRV 1.61.0_

### Changes

- (LIB) Cargo.toml rust MSRV 1.61.0 ([3c4e8b1]) 
- (BUILD) rust.yml add job_rust_versions, jo_rust_msrv_os 1.61.0 ([94d6862])
- (LIB) const-str downgrade to 0.4.3 ([7cdaa4b])
- (DOCS) README add more Windows examples, wording ([db5b6a5])
- (LIB) (BIN) codespell fixes ([b9d4c2c]) ([aaaf78e])

---

## 0.1.38

_Released 2022-10-16_

[0.0.37...0.1.38]

_MSRV 1.62.0_

### New

- (BIN) bin.rs --summary print -a -b as UTC ([186c747])
- (BIN) bin.rs allow relative offset datetimes [Issue #35] ([4eef622])

### Changes

- (DOCS) README add Windows log snippets, tweak wording ([65c0078])

---

## 0.0.37

_Released 2022-10-12_

[0.0.36...0.0.37]

_MSRV 1.62.0_

### New

- (LIB) datetime.rs patterns for Windows compsetup.log mrt.log ([0f225ce])

### Changes

- (LIB) blockreader.rs eprintln path ([3e1607f])
- (TEST) add logs Windows10Pro ([1c746c2])

---

## 0.0.36

_Released 2022-10-10_

[0.0.35...0.0.36]

_MSRV 1.62.0_

### New

- (LIB) datetime.rs parsing nginx apache logs [Issue #53] ([003b29b])

### Changes

- (BIN) bin.rs --help tweak, comment cleanup ([3963e07])
- (TEST) add more logs ([9ddaaee]) ([effffe8]) ([66eea98]) ([0743e41]) ([e736a71])
- (DOCS) README changes, docstring changes ([6579485]) ([001f0c3]) ([4b51b30]) ([fda30e5]) ([8f14374]) ([44291c7])

### Fixes

- (LIB) datetime.rs add RP_NOALPHA after CGP_TZZ ([1cfc72e])

---

## 0.0.35

_Released 2022-10-09_

[0.0.34...0.0.35]

_MSRV 1.62.0_

### New

- (LIB) datetime.rs handle format with syslog levels [Issue #57] ([d75fdfc])
- (LIB) datetime.rs add RP_NOALPHA after CGP_TZZ ([1cfc72e])
- (DOCS) rustdocs improvements ([1de420a]) ([1b88a1e])
- (PROJECT) README add section "syslog definition chaos" ([ec82a50])
- (BUILD) Cargo update to latest dependencies ([7185ba4])
- (BUILD) Cargo.toml exclude more READMEs ([bd44896])

### Fixes

- (TEST) cargo-test.sh fix --version ([2a1b108])

---

## 0.0.34

_Released 2022-10-07_

[0.0.33...0.0.34]

_MSRV 1.62.0_

### New

- (BIN) bin.rs allow user opt dt format string [Issue #28] ([6660f68])
- (TEST) add logs ISO8601\*log RFC3164\*log RFC3339\*log ([3980d5b])
- (TEST, TOOLS) log files add systems, cleaning scripts, touch set ([55a1e55])
- (LIB) datetime.rs allow Unicode "minus sign" [Issue #38] ([fc2a837])
- (PROJECT) README updated Features, Limitations ([0ca431c])
- (PROJECT) README README fill section About ([aa3992c])

### Fixes

- (TEST) cargo-test.sh `rm -f` to avoid possible error during exit_ ([2343d26])

---

## 0.0.33

_Released 2022-09-21_

[0.0.32...0.0.33]

_MSRV 1.62.0_

### New

- (BIN) bin.rs allow user opt dt format string [Issue #28] ([6660f68])
- (DEBUG) use `si_trace_print` 0.2.5 ([fc54823])

---

## 0.0.32

_Released 2022-09-20_

[0.0.31...0.0.32]

_MSRV 1.62.0_

### New

- (TEST) datetime_tests.rs add tests cases datetime_parse_from_str ([c9bc19e])

### Fixes

- (LIB) datetime.rs fix copying fractional [Issue #23] ([764689f])

---

## 0.0.31

_Released 2022-09-19_

[0.0.30...0.0.31]

_MSRV 1.62.0_

### New

- (DOCS) improved README badges ([30553b7]) ([b5d4d91]) ([17cd497])

---

## 0.0.30

_Released 2022-09-18_

[0.0.29...0.0.30]

_MSRV 1.62.0_

### New

- (LIB) allow -t %Z or %z, allow Zulu ([a71c5e8])

### Changes

- (TEST) add log dtf11-A.log ([56078e8])
- (BIN) bin.rs stack_offset_set only debug builds ([d2158ee])

---

## 0.0.29

_Released 2022-09-17_

[0.0.28...0.0.29]

### Changes

- (DOCS) README update install and run instructions ([01f3959])
- (BUILD) Cargo.toml remove unused bench, exclude unused benches ([b088be7])
- (LIB) Cargo.toml rust-version 1.62 ([aaf976b])
- (BUILD) rust.yml add "Check Publish" ([7f751c1])
- (TEST) add logs gen-100-10. gz xz tar tgz ([33a492b])
- (TOOLS) valgrind-dhat.sh add more logs to parse ([6d64fd6])
- (LIB) Cargo update dev-dependencies ([6805e2b])

---

## 0.0.28

_Released 2022-09-17_

[0.0.27...0.0.28]

### New

- (BUILD) Cargo.toml rust-version 1.62 ([aaf976b])
- (BUILD) rust.yml add "Check Publish" ([7f751c1])
- (BUILD) Cargo update dev-dependencies ([6805e2b])
- (BUILD) Cargo.toml ready to publish ([d97f0ab])
- (CI) Check clippy ([e7172e4]) ([ac8d29b])
- (CI) rust.yml add grcov job ([fcf91c9])
- (CI) rust.yml add step for "cargo check" ([17f8902])
- (TOOLS) add cargo-clippy.sh ([676633a])

### Changes

- (TEST) add new test logs
- (LIB) src/ clippy recommendations ([84cc63c])
- (LIB) src/ mv src/printer_debug -> src/debug ([d70104f])

### Fixes

- (TOOLS) tools chmod 100755 ([a137866])

---

## 0.0.27

_Released 2022-09-16_

[0.0.26...0.0.27]

### New

- (LIB) datetime.rs allow lowercase named timezone ([cae9877])
- (TOOLS) add cargo-test-cov-tarpaulin.sh ([5f93e4a])
- (TOOLS) add cargo-test-cov-llvm.sh ([97cf45e])
- (TOOLS) add cargo-test-cov-grcov.sh ([55113bc])
- (TOOLS) add changelog-link-gen.sh ([0579522])

### Changes

- (BIN) bin.rs prepend only files with Syslines [Issue #21] ([e189fd2])
- (LIB) src/ cargo fmt ([4fda60f]) ([53892a3]) ([a1e1a68]) ([dff2927]) ([4b784a7]) ([78581db])
- (LIB) use crate si_trace_print, remove local impl ([f49fb33])
- (TOOLS) rename compare-cat.sh ([25e4bf6])
- (TOOLS) rename cargo-test.sh ([0a5ce1e])
- (TOOLS) rename rust-gdb.sh ([f00ebc4])
- (TOOLS) rename cargo-doc.sh ([9957cc5])
- (TOOLS) rename compare-grep-sort.sh ([acc34ed])
- (TESTS) datetime.rs add begin,end indexes to test cases ([5dfc932])
- (LIB) Cargo.* update dependencies ([7fb6abe]) ([d8faf4f])
- (LIB) datetime.rs improve precision of patterns, var names ([238df6c])
- (LIB) datetime.rs add x4 DTPI for a pattern lie `VERBOSE Tuesday Jun 28 2022 01:51:12 +1230` ([34595fe])
- (LIB) datetime.rs match named pattern `<dayIgnore>` ([07214ab])

---

## 0.0.26

_Released 2022-08-12_

[0.0.25...0.0.26]

### New

- (DOCS) rustdocs
- (LIB) add datetime patterns for history.log ([6955a7b])
- (BIN) bin.rs allow user to pass only dates ([0a46b5a])
- (TOOLS) add rust-doc.sh ([f56045a])
- (TOOLS) add valgrind-dhat-extract.sh ([aa5bdbb])

### Changes

- (BIN) refactor logic for process return code ([09df0b6])
- (CI) github tests build documentation ([f7b4533])
- (BIN) remove unsupported `%Z` in `--help` [Issue #20] ([b55341e])
- (BIN) bin.rs simplify check for printing error ([79c1ea1])
- (BIN) bin.rs print trailing `'\n'` if needed ([3c7984d])
- (BIN) bin.rs remove panics in processing_loop ([dc30ca6])
- (LIB) line.rs printers.rs refactor some slice statements ([aab71b5])
- (DOCS) blockreader.rs NFC docstring about read_block_FileTar [Issue #13] ([a503554])
- (LIB) filepreprocessor.rs NFC comment [Issue #15] ([ed3d1fe])
- (LIB) src/ NFC comment [Issue #16] ([c35066c])
- (LIB) blockreader.rs NFC comment [Issue #13] ([1c58a77])
- (LIB) blockreader.rs NFC comment [Issue #12] ([bc41128])
- (LIB) blockreader.rs NFC comment [Issue #11] ([908b2f5])
- (LIB) blockreader.rs NFC comment [Issue #10] ([c9dc70a])
- (LIB) blockreader.rs NFC comment [Issue #9] ([8cd40b5])
- (LIB) blockreader.rs NFC comment [Issue #8] ([c8328f3])
- (LIB) blockreader.rs NFC comment [Issue #7] ([943ad62])
- (LIB) datetime.rs NFC comment [Issue #6] ([af46851])
- (LIB) datetime.rs NFC add commented [Issue #4] ([d8d5641])
- (BIN) bin.rs NFC link [Issue #5] ([81c437b])
- (LIB) improve supplementary filetype matching in `path_to_filetype` ([48687e8])
- (BIN) bin.rs consistent eprintln for failed parsing ([880e35a])

### Fixes

- (TOOLS) tools chmod 100755 ([a137866])
- (LIB) src/blockreader* fix handling zero byte gz, xz ([f708e15]) ([21745ee])
- (BIN) bin.rs fix -a -b parsing ([f6b52fc])
- (LIB) Massive amount of code cleanup
- (TEST) Many more tests

---

## 0.0.25

_Released 2022-07-28_

[0.0.24...0.0.25]

### New

- (LIB) add handling for missing year in datetime ([ab57920])

### Changes

- (BIN) Fix flood of error messages due to printing failure ([c332a73])

### Fixes

- (LIB) Fix handling GZIP XZ ([8d5e686])
- (DEBUG) Fix debug print Sysline String with unknown glyph ([b2d6de5])
- (DEBUG) Fix debug print Line String with unknown glyph ([ed5c04a])
- (TEST) Many more tests

---


## 0.0.24

_Released 2022-07-20_

[0.0.23...0.0.24]

### New

- (LIB) handle tar files ([adf4007])

### Changes

- (LIB) datetime transforms `%b` `%B` to `%m` ([22980ab])

---

## 0.0.23

_Released 2022-07-12_

[0.0.22...0.0.23]

### New

- (LIB) WIP handle tar files ([b8deef3])
- (PROJECT) add CHANGELOG.md ([ca1c967])

### Changes

- (LIB) add enum_BoxPtrs::DoublePtr ([61f15e1]) ([cb74da3])
- (LIB) refactor to use `regex::bytes::Regex` instead of `str`-based `regex::Regex` ([dfd60d4]) ([3d78b0d])
- (LIB) refactor name `enum_BoxPtrs` to `LinePartPtrs` ([b550573])
- (TOOLS) rust-test.sh use nextest if available ([1bf2784])
- (TEST) faster tests reuse single `NamedTempFile` more often ([db2e8f3])
- (CI) github run args change ([a82e25b]) ([c8fc525]) ([febfd00])

### Fixes

- (BIN) fix --blocksz minimum check ([07baf6d])
- (LIB) printers.rs fix macro `print_color_highlight_dt` ([6659509])
- (DEBUG) line.rs impl LinePart::count_bytes,len ([9d9179c])
  Fix `LinePart::count_bytes`
- (DEBUG) printers.rs fix `char_to_char_noraw` ([ced4667])
- (DEBUG) line.rs fix `_to_String_raw` ([d5af77d])

---

## 0.0.22

_Released 2022-07-10_

[0.0.21...0.0.22]

### New

- (LIB) refactor datetime string matching ([3562638])
  refactor datetime string matching within a `Line` to use regex.
- (TOOLS) add hexdump.py ([031434f])
- (LIB) printers.rs highlight datetime in call cases ([a4fd91f])
- (LIB) printer.rs all color lines highlight dt ([9c5fa57])
- (LIB) filepreprocessor also check supplement with ext removed ([0f4ac9a])
  During filetype search, also call supplmentary check based on name
  using a path with file extension removed. Allows matching files like
  `kern.log.gz.1`.

### Changes

- (BUILD) remove crate chain-cmp ([7109c46])
- (LIB) set `const` for funcs `slice_contains...` ([eeb20bb])

### Fixes

- (LIB) fix errant �� printed at block edges ([5caa8dd])

---

## 0.0.21

_Released 2022-06-24_

[0.0.1...0.0.21]

### New

- (LIB) add XZ support ([607a23c])
- (BIN) bin.rs set default -t in help message ([e346e18])

### Fixes

- (LIB) src/ print summary invalid, fix dir walk error ([09a04c1])
  Print summary of each invalid file and it's MimeGuess and FileType result. This helps the use understand why a file was not parsed.
  (LIB) Fix directory walk in `process_path` only checking the root directoy and giving errant `FileType` to files.

---

<!--
All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](http://semver.org/) and [Keep a Changelog](http://keepachangelog.com/).
-->

This document template generated by [changelog-cli](https://pypi.org/project/changelog-cli/).

<!--
TODO per release: update links (run `tools/changelog-link-gen.sh`)

DO NOT CHANGE THE FOLLOWING COMMENT
EVERYTHING AFTER THE FOLLOWING COMMENT WILL BE DELETED AND REPLACED BY `tools/changelog-link-gen.sh`
-->

<!-- LINKS BEGIN -->

[Issue #4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/4
[Issue #5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/5
[Issue #6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/6
[Issue #7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/7
[Issue #8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/8
[Issue #9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/9
[Issue #10]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/10
[Issue #11]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/11
[Issue #12]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/12
[Issue #13]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/13
[Issue #14]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/14
[Issue #15]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/15
[Issue #16]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/16
[Issue #17]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/17
[Issue #20]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/20
[Issue #21]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/21
[Issue #22]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/22
[Issue #23]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/23
[Issue #27]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/27
[Issue #28]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/28
[Issue #29]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/29
[Issue #35]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/35
[Issue #38]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/38
[Issue #41]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/41
[Issue #42]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/42
[Issue #45]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/45
[Issue #47]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/47
[Issue #48]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/48
[Issue #53]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/53
[Issue #55]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/55
[Issue #57]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/57
[Issue #58]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/58
[Issue #60]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/60
[Issue #63]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/63
[Issue #65]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/65
[Issue #66]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/66
[Issue #67]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/67
[Issue #69]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/69
[Issue #70]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/70
[Issue #71]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/71
[Issue #74]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/74
[Issue #75]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/75
[Issue #76]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/76
[Issue #77]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/77
[Issue #80]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/80
[Issue #81]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/81
[Issue #82]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/82
[Issue #84]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/84
[Issue #85]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/85
[Issue #86]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/86
[Issue #87]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/87
[Issue #98]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/98
[Issue #100]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/100
[Issue #104]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/104
[Issue #105]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/105
[Issue #108]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/108
[Issue #109]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/109
[Issue #112]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/112
[Issue #120]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/120
[Issue #121]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/121
[Issue #128]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/128
[Issue #145]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/145
[Issue #152]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/152
[Issue #170]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/170
[Issue #171]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/171
[Issue #182]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/182
[Issue #201]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/201
[Issue #202]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/202
[Issue #206]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/206
[Issue #213]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/213
[Issue #217]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/217
[Issue #218]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/218
[Issue #219]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/219
[Issue #224]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/224
[Issue #241]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/241
[Issue #245]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/245
[Issue #250]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/250
[Issue #257]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/257
[Issue #258]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/258
[Issue #260]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/260
[Issue #270]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/270
[Issue #283]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/283
[Issue #284]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/284
[Issue #285]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/285
[Issue #291]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/291
[Issue #293]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/293
[(#166)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/166
[(#168)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/168
[(#169)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/169
[(#172)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/172
[(#173)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/173
[(#174)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/174
[(#175)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/175
[(#176)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/176
[(#177)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/177
[(#178)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/178
[(#181)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/181
[(#193)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/193
[(#195)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/195
[(#197)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/197
[(#199)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/199
[(#200)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/200
[(#203)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/203
[(#204)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/204
[(#237)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/237
[(#246)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/246
[(#247)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/247
[(#249)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/249
[(#272)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/272
[(#274)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/274
[(#275)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/275
[(#278)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/278
[(#279)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/279
[(#280)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/280
[(#281)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/281
[(#282)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/282
[(#289)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/289
[(#290)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/290
[(#292)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/292
[(#295)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/295
[(#296)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/296
[0.0.1...0.0.21]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.1...0.0.21
[0.0.21...0.0.22]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.21...0.0.22
[0.0.22...0.0.23]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.22...0.0.23
[0.0.23...0.0.24]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.23...0.0.24
[0.0.24...0.0.25]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.24...0.0.25
[0.0.25...0.0.26]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.25...0.0.26
[0.0.26...0.0.27]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.26...0.0.27
[0.0.27...0.0.28]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.27...0.0.28
[0.0.28...0.0.29]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.28...0.0.29
[0.0.29...0.0.30]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.29...0.0.30
[0.0.30...0.0.31]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.30...0.0.31
[0.0.31...0.0.32]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.31...0.0.32
[0.0.32...0.0.33]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.32...0.0.33
[0.0.33...0.0.34]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.33...0.0.34
[0.0.34...0.0.35]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.34...0.0.35
[0.0.35...0.0.36]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.35...0.0.36
[0.0.36...0.0.37]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.36...0.0.37
[0.0.37...0.1.38]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.37...0.1.38
[0.1.38...0.1.39]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.38...0.1.39
[0.1.39...0.1.40]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.39...0.1.40
[0.1.40...0.1.41]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.40...0.1.41
[0.1.41...0.1.42]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.41...0.1.42
[0.1.42...0.1.43]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.42...0.1.43
[0.1.43...0.1.44]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.43...0.1.44
[0.1.44...0.1.45]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.44...0.1.45
[0.1.45...0.2.46]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.45...0.2.46
[0.2.46...0.2.47]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.46...0.2.47
[0.2.48...0.2.49]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.48...0.2.49
[0.2.49...0.2.50]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.49...0.2.50
[0.2.50...0.2.51]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.50...0.2.51
[0.2.51...0.2.52]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.51...0.2.52
[0.2.52...0.3.53]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.52...0.3.53
[0.2.53...0.3.54]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.53...0.3.54
[0.3.54...0.3.55]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.3.54...0.3.55
[0.3.55...0.4.56]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.3.55...0.4.56
[0.4.56...0.4.57]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.4.56...0.4.57
[0.4.57...0.5.58]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.4.57...0.5.58
[0.5.58...0.5.59]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.5.58...0.5.59
[0.5.59...0.6.60]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.5.59...0.6.60
[0.6.60...0.6.61]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.60...0.6.61
[0.6.61..0.6.62]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.61..0.6.62
[0.6.62..0.6.63]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.62..0.6.63
[0.6.63..0.6.64]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.63..0.6.64
[0.6.64..0.6.65]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.64..0.6.65
[0.6.65..0.6.66]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.65..0.6.66
[0.6.66..0.6.67]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.66..0.6.67
[0.6.67..0.6.68]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.67..0.6.68
[0.6.68..0.6.69]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.68..0.6.69
[0.6.69..0.6.70]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.69..0.6.70
[0.6.70..0.6.71]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.70..0.6.71
[0.6.71..0.6.72]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.71..0.6.72
[00171bb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/00171bbdf238fd9c1ba6d89fa29a730318332d7e
[001f0c3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/001f0c3db2c5751a35946f572aca6bf07c9efcaf
[0021f05]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0021f0576d0d629c72028443f2a266f957e5b084
[003b29b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/003b29bab508b32750cb303c70db9dc75cc04eab
[007f752]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/007f752fdd69eb37df38171a7e485f5ae026ec6c
[016e8b7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/016e8b73852789c607b467c02e44c1ccf7933da3
[01b80ff]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/01b80ffaf666111674a7c11f33d913f8a0118d19
[01c11d8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/01c11d89d6d5b3f329462b009c950f53c56a30e8
[01f3959]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/01f395903cff248be11ecf6f12974a3951aa7e92
[02261be]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/02261be4a6779a73ee08a3075bccb5effc31818f
[031434f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/031434f4d9dfb4e0f8190a720f8db57a3772e3a2
[04558a3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/04558a3054bc365543fcc40e33123248cd49f66e
[0522f1e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0522f1eb34453c68c8f1abca0efeb5dfac5c2bf9
[0579522]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0579522ff7609e22c14b33aa6c6a70cec6372226
[05d974c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/05d974c6c4ced7b380343cbff1710e99a2a2ce28
[05f04e3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/05f04e30dbf5985f01dabc1daa2fa36d10e900a1
[06640e3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/06640e3218bbbe8bdf97c9a54907fcb1a9491876
[0664eeb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0664eebccafedc636d0c03f67e5098bcad95b99a
[068a0a2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/068a0a2893f6007a51aa9e65c997b9e08e72c3a5
[06e500f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/06e500f1d0148e0f9b50ab5907d7f6103533d5f7
[07214ab]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/07214abde6479431cc1a9f87f50f3b713e5ea503
[0743e41]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0743e4157daa108569d99746d8a6314cfe6e0248
[07baf6d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/07baf6df44ec3ccd2da43f3c5cb9f5ef30a6b0e8
[08738c4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/08738c41a371749b9aac26c0ab319129d8be0c9f
[08d198a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/08d198ae57fc5b97013bdda5e883d7df383755f9
[08fed12]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/08fed12cf9f2e7a4003a02d2a3e3efecddf49c80
[0923408]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0923408bff8036c1b1c37bfba0a71012845c0935
[0997ecd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0997ecdc607501cf45e1e8c043210660a290646e
[09a04c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/09a04c14146af1916aeda14e8134d02baf088d5d
[09a885d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/09a885de20cffeabbfaae72f2d597e007c9b6593
[09df0b6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/09df0b6551fec2ea22cee7dca2cd308cf11b531a
[0a46b5a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0a46b5aee7eb99e19a9a2a91ed81d759978b6024
[0a5ce1e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0a5ce1e0011920909cfa5bc022f95b3a502ff244
[0bee449]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0bee4492533b7a88dfb43a9965b9026bcdefc705
[0c1f565]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0c1f5655a108865ec3fbccc964d59d455f7e27a1
[0c6af5d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0c6af5d6d031fd90fd472452bd42ddffab313da4
[0c6ba91]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0c6ba914d48380fae289077ea08b282484e075b5
[0c7efef]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0c7efef500543e3176b1538c90065cad3d624c50
[0ca431c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0ca431ce8b510b6714420a8954f587eccd84a01d
[0d9d80b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0d9d80be29fc5051429cf53924d4a7ac3f6010a7
[0da5f9d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0da5f9d5a4001a7055e96061d485075e2a9a5cdb
[0ea897a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0ea897a7665eff58d9c148ee53559504301e4a52
[0f225ce]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0f225cee04b5443a58369b95bc8e6f10ed3f6401
[0f4521c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0f4521cd7cfe059fddab74cfd29f7920d6070ad7
[0f4ac9a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0f4ac9ae4cb4d11247a40cf1a3c09f78a9a42399
[0f822c5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0f822c54ef4959aba65b4bd24969dbf44e19c6ea
[0fceba2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0fceba274b8dbefb01ed890d3c211fd85211822b
[11c9d0b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/11c9d0bbe8651fb8e057e88166afb450534d03f4
[12871b7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/12871b740007754071764243b290be6cc4fe272b
[132a18a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/132a18a8de9131a5d101f24f35a5a7d773beb8ef
[133cb5c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/133cb5c7dcab6f018c0422bde1f8ee6f9a304258
[13b6039]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/13b6039f771dc97e6a2792cce81d0752a6cf1806
[13f1ef4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/13f1ef40476078a0a6904e1056dcf0153a9b1b43
[145c45b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/145c45be9c3e02027211c02d5b97c63f9ef98944
[145efc2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/145efc2f581de83aa8c639adbd49ad926f392be7
[157d54a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/157d54a4dda13b1f0b4743daa55b77d20887e82a
[1677328]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1677328e42072057bcac2622726bd973255477c5
[17cd497]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/17cd497307d04f3d8a9b058a72e3ea415a9a9f89
[17f8902]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/17f89020870b8bc8ad8322e314c187b6e0836226
[186c747]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/186c74720db2b33e5c0df17ee690eddcdee360a7
[19adf7e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/19adf7ec9e2a687b6df19d2e3121c2683f3fc840
[1aed862]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1aed86246f06e1e3f68f692d81b45c2e22be60b5
[1b88a1e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1b88a1e35a66004ea5016525bcbb1e125aa64db9
[1bf2784]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1bf2784185df479a3a17975f773e3a505f735e26
[1c58a77]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1c58a778dc5bd05e455ea25af60e8600b8b72857
[1c6ccb1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1c6ccb188129272267fd14d0f16fff42f67d81c6
[1c746c2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1c746c24b7e0ad7e7481cce626fb6488eb0076d6
[1cfc72e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1cfc72e99382ab47b55c9410ab531c0baf8ac46e
[1d6bc01]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1d6bc01b3f26c8362f08a4adc73c24ae5b968d8e
[1de420a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1de420a5907cf62ae91a06732a8ef43e01f17598
[1e3e789]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1e3e789ba02d8378d590f61487c0beff5bb39d4f
[1e552fe]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1e552fe9a673dc759b583ff1c434b00385015025
[1e58094]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1e58094eafae95c9c09b35c63aa000a0edfd5845
[2030a73]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2030a7392e792e00727f80f9a2d83257b851f519
[210f01c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/210f01c36f0e7b8415ae595fbda857cff44277fb
[2157861]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2157861027eff2cade51aa950a6a4300e86a1e50
[2160556]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2160556dd15491acfa5e13d9de69a8db4af224d5
[21745ee]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/21745ee99eb04a4204164825ca5c50e6f8b34fee
[219e2cf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/219e2cfcf9124054da2d34732a1290387daa4344
[22980ab]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/22980abf582aa61c5e4c9ce94d8298997fb5bbbc
[231fcc4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/231fcc46d921f781a1cdaa188c9f7e189e2709cd
[2340084]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2340084f30e25df0e81af73ef2d06f60f42963aa
[2343d26]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2343d26300c5a139066081648054e5e299eb8a80
[236802f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/236802fa5ca3266119e20d4d1dd968a6e5f80a97
[238df6c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/238df6c7b1b569f724778c85bfead20cb14be59d
[23dfeb3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/23dfeb32d0a9d8a7b272ef748fca9b8556b5b0c1
[2469178]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/24691784f79d33a5dd5497f53e064302dfb161d3
[24c21a1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/24c21a15728132fcbcc6c3ce9724ed8ff19ec8a1
[24f00e7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/24f00e77839701e01123b61e4d7daefcab264a9b
[25060e8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/25060e8bba5a158a6cfa000babdbb67d758f2ddc
[2544716]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/25447169bfe6c6236b8b87e6b3f451a209c7d9d5
[25b35ee]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/25b35ee8e1f7b7ed7d5119bca73becc6ad721617
[25e4bf6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/25e4bf65d9d5af300b99092e189f0caea3164f5f
[26979a3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/26979a36cbbaeaf0f4266c2717757eab58662a80
[26daee6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/26daee68627b16262717b7091fb192a029896cf5
[26ec11b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/26ec11b7fff8c478b4aa48ed1a4cec01b683a318
[281adc0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/281adc0d2ebea05a6f47fca2ccabffe865295c16
[29072ac]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/29072ac5c184215f8c10547e5019bf1845864296
[2975c9a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2975c9af59b515ee71824cd156c0b3b1bfba3f7d
[29a5558]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/29a555822f24429491a5e5586dac8c23c93057e7
[2a1b108]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2a1b10859a31649a7ef31db9474e3a6ed526c9a4
[2a2d372]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2a2d372e0c7bccb98ae3577a256caa710bff2e7d
[2af24cb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2af24cbfbb1645e2cd364a9ab4434e0892619939
[2b7a60b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2b7a60b90bbedf5f1c21e40832f16d131dcac7f5
[2c1a38e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2c1a38eaca7a66c54938c66abd046fc21e34b58e
[2c34a47]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2c34a47fc1adc5d59ce6edd35b377659084f4819
[2c70ee3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2c70ee35718d8c03435139516bcb5e95e5af9f43
[2c8bf0f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2c8bf0f1bf9c7237b20849b195c52f926c0e43ff
[2cb0412]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2cb0412d714078b17402d5bcfa2b1175f4f71bb3
[2d1541c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2d1541c4efefde26182963fb4c82e546ab2f36a4
[2da3398]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2da339822a4f62266149b8d53925840c0860c9a2
[2e10dd1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2e10dd1cab1dd3a68fa66207f03d66e4e2e72c0c
[2e77297]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2e772970ab86a9541ad56a1702b4a219412ea88b
[2e95d61]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2e95d616dc5d6cdc9b907935ccd006cdf6e88f0c
[2edda45]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2edda45071e3593c83d16514bcfa2a81192a6d35
[2f16d92]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2f16d924762ee070ae1095f97e2bb0583125e27b
[2f1fc58]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2f1fc58e6727c42cfc83298c0e421743f63899af
[2f9a434]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2f9a434cd13200d95d8ce5e5f0a3f8af1b822a92
[2fef744]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2fef7441f3c7db1b558f451f1233919ab7b05414
[2ff0d19]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2ff0d197842c39450275d6d09bdd7ed06db1e735
[2ff222e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2ff222eaf061cbefe372cb88accd2be39b141817
[30553b7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/30553b7989b55c802704c42deefe9424347092ee
[307c86c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/307c86c22c96ca90ca5456e8dcaf6a83534efbf6
[308628c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/308628ccfa8cef32aa093817b78983739f52548f
[33418d0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/33418d0311fc75fa7fda97ac621ddf2da493c128
[33447dd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/33447dd116c091bd968eedf78675dc8c94b46982
[33a492b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/33a492b9c01c57a71191d7f1b46d457d5ff67059
[34320a7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/34320a79819fceba1810067606990ab35bcf45b0
[34595fe]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/34595fe2693385b0cdff69ecf6306071d058b638
[34b75c5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/34b75c5ba38708d8ed2f63b3135c0b42b57ab065
[34ef716]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/34ef7167ac032e42c020aaae405efcf3e3e26ad1
[3562638]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3562638d37272b2befa7f9007307fd4088cdd00c
[3591489]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/35914891bd90cfadbe17867c370c65429883e879
[35fbb1d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/35fbb1dade0bbfd40042b5154430df5754caa92e
[35fd92f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/35fd92ff3c1195eb1e549aa990fa6ef70183c5da
[361e986]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/361e986710d8c97932b87bffc096e6af122ef58e
[368eba9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/368eba9b473b0c31ebd232bd89bc2aabd5a15d53
[3805df7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3805df7db5cc3090568a8ae2316b136f758dc962
[38d1c47]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/38d1c47305125d9bd4e9275ef99d9767af3f1380
[3963e07]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3963e070fd8849ce327d9cdb4ef7bbbe52d0d7e2
[3980d5b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3980d5b67bbd371d84cbb313f51e950dae436d54
[3a6eac6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3a6eac6bab6e45b5cb413176a614cb329c4d3f67
[3aaddac]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3aaddac4d39967807fa2156e11fe5ef31dac8bc8
[3ac5374]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3ac5374edd67a53e0c1492e487db90e9d36a91fd
[3ada5b0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3ada5b0ad71dcbdf9fc67aa1a9976ea54f0960ba
[3b95001]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3b950014411d743e3e5527f652e5a2d4aff9a847
[3c34d09]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c34d099f162ee65423dbee77946622b391955a3
[3c4e8b1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c4e8b1b37415ad0662019d1792525ab0b00a8f9
[3c5a18a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c5a18a47f168dfc463411e81b07f3250ba68df0
[3c7984d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c7984d49df0d91037729a45c24a2a7b5a109687
[3d78b0d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3d78b0d0b6918dab784bbe2332b3a26928bb8f90
[3df00ac]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3df00ac9e826042b31d9617d81f54df998525031
[3e1607f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3e1607f076afe7a6e10578776a07d3feb0a2b9a8
[3e70a60]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3e70a605e7642596337bc49deda5a542c75aaddf
[3ee20b9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3ee20b9c743ac1ab72652b4ea4ab61bd722d8a16
[4017242]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/40172427877f7a020afebd8aa772810feeba85b3
[40e428d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/40e428d6022a4b800d96c72b57f46263d3bf3212
[41bb25a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/41bb25a10b5bc70c228f9f5930d4f0aaba9eafbd
[4236304]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4236304bb6d74f76ebc62959b0a5decb5f96ffb7
[4268bc6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4268bc6bc4657036534627f43966754a4a419ed5
[42ccefd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/42ccefd392bfdc1aed8036f948aa74371308a8df
[4370ada]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4370adae232ecac190d46a6828e6a7661cc0d96e
[43d4cca]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/43d4cca5446981ff9aa3bb5d5b5653c83e75e530
[44291c7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/44291c749bfae647cf130fdc298dd2cc5d1876ba
[44bd6b1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/44bd6b10290eaa4e9ede11765d25eba4b171cbe2
[44fa812]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/44fa812ad1f50f90cf5fcf88603fad3a44d09783
[463d93f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/463d93f309ab8a320b53711f67a52072566de69c
[467b14d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/467b14dbc59a60a808e7a71a1083f2490cf31d48
[46c1502]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/46c1502f07d0481c3aaab5e05899296f25c6ea13
[475529e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/475529ee30fe2a2e8587ffee57a35190626570bd
[476ed60]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/476ed604e7b4201efe5b6e5f7c4a588c3abaa157
[48687e8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/48687e8a65af56cf9c6279702ccaa6a66c127a06
[487138f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/487138fca7e5cac02347e8597f90b9e1237bd531
[48bd3aa]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/48bd3aa9c24f5420f54ffdddabd061ec5a25d55b
[48c5a5e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/48c5a5e40fcf29e581fb329dd8c2e778a4e08b92
[498ad5f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/498ad5f4cc0af31ac552e7aa6f9ee1b7ef030e13
[4a43a7b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4a43a7b33b59744e5b070498613ec1fb61e440e2
[4a79d15]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4a79d15d303a92515b7ee970ddabc3cdd9994b85
[4ac8430]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4ac84307c9432001c1b010ff2aafec0be3b2d4cc
[4b328c5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4b328c5a416cc6daeeeedf81f1dc76bc9bbf849a
[4b51b30]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4b51b30d598a6e076f3d2a8b9d3e170deea1325f
[4b784a7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4b784a723b8c02c7bdb4b51e7d7b76147f97d569
[4b80fa5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4b80fa51f0034f4adf03fad5fc66329e23602f07
[4c7b6f1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4c7b6f1adf7398dfa570224ca470f8a71870a831
[4db0056]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4db00567813d9b236c77a49a33e399ad5c0c94ab
[4e32f5c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4e32f5ceab4b77a533fcc62ea68377d209b7a282
[4eef622]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4eef6221708137928458ed8445b4f67196500082
[4fda60f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4fda60f22505ebba9ff86873386d0524d364765c
[505280a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/505280afb5459be37c9905f1e7b23983b2e7e287
[50870c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/50870c1bf3cca434ec2bd03624fd690fa59dd588
[50cc3c2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/50cc3c2d086b15af580ec6e190059f2b59c0233c
[50e3c52]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/50e3c5235ab8aa95ffe58b7114bdf257d4bdeff5
[50fec20]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/50fec201f1094269a1dc53bca88b25e33d7ceec4
[518514e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/518514e9770a169f75e672382474d32abd4f2173
[5239fb7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5239fb758f907c043362b302c29787f5dd955447
[524e269]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/524e269e8b6584fdcd60ff551a4f0a5d49e7384e
[52777c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/52777c1eb5ff968430cb678630f01a100763b967
[5337dd9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5337dd907a456236ebd038f7b3df6fa4e1687a68
[5359667]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/53596672c4cc4e9b47ee60d4e96af69aeb21d3dd
[53892a3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/53892a3a2d46c3b7dcad3b0fd7b4141118485e9e
[53e8a75]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/53e8a75974dfe4bf11740ad80c0fe769dfa0ebdb
[54b0860]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/54b0860302e2b691ef6ca54c1bde09fa97e1e3b2
[54c3e09]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/54c3e09ce49771e62fcc3bb2938f259767e002ba
[55113bc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/55113bc5705d5c9ace1da6bde8b05c1260ddb935
[5543ff7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5543ff7204b50723dbdcd9042bd9747b74821bfb
[55a1e55]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/55a1e55ed94f8e8a4202098c1fd4f85e337bfae4
[56078e8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/56078e8bb713fa861ccf9ebd1a58415ee6173819
[5618d05]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5618d051819a9874a5db33747523553ba1f906c9
[567394f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/567394fb63861fa0ad809e5fc1da6bab9e790540
[56adf88]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/56adf88149e87aebbf87c70fc4531545d2c11daa
[5750cf3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5750cf34d80a90e7134cbf8c57619d3ff1abbd98
[57a149c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/57a149c601e6dbfa2266c499370fded5e5a95290
[57e2a4d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/57e2a4d7c2b8a169d83f52162b87c52e09d23f67
[58d2f20]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/58d2f2059a7f43f7bdaff90043799e64ede338b6
[58d6ec9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/58d6ec9e31775e24ead4876dd543d8cb9cf559a4
[58d9d79]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/58d9d79a5482fa0d1a555623e33f588de9665bbd
[597c080]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/597c0807e426e2d17f6a6b49a37665899b6bc074
[5ae0f03]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5ae0f033f00db93b45494ca7f0d93946e3633c63
[5b2b6f8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5b2b6f808c100077fc94c7019821a01896f7b652
[5bb8a5d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5bb8a5d1c4331d8e4b0391509abae2277012215d
[5caa8dd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5caa8dd6b7f8f2735366a23ab1005df89aaf565f
[5cabf7b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5cabf7b91b44fb508cbb90ea8299fd78088323be
[5d5ce99]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5d5ce99c92198d0e843259c1d32f08ba87d0039b
[5dd008a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5dd008a2e5b22bc4e8d3760ce54d6fa0f7a634bc
[5de9a32]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5de9a3260cc3b171df72886919b41abb44fb0517
[5dfc932]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5dfc932d8f62e295f93accafb98c533fd8e39625
[5e97590]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5e975901312e13a35d8599fc06bd0536f4c61e9e
[5eaa48f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5eaa48ff8fe3b9db875ea81d6f0e7a7fb408b448
[5f5606d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5f5606def37b70ac96d7045fa2ee36156b4d4f28
[5f77a0f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5f77a0f7ddbc194ffdc1e45556e2c85910002af6
[5f8a63e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5f8a63e4ececc6d35568835017fe1c3149b1c086
[5f93e4a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5f93e4ad56fdbda6b5ceeaeca94848063064cc9a
[607a23c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/607a23c00aff0d9b34fb3d678bdfd5c14290582d
[60aa5d1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/60aa5d1c1e983aad9b0921e3e066935742605b52
[610785f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/610785f3d98a4032fe7053076f9db45d4c1d1717
[616bef1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/616bef16cacceb26dae625be830141b8ab2252e7
[619da41]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/619da4158649e2fc038bc0ecb9b36e82931508b6
[61f15e1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/61f15e13d086a5d6c0e5a18d44c730ebe77a046a
[62e89e2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/62e89e2917a36d73110a860a5f490e4fbb19a6b2
[630b8ce]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/630b8ce945dd2f87d88c357afec26a0a5bdbed60
[6408589]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6408589ca5c49c28822b5fc2e3cacb1903db1fca
[6579485]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/657948516a05c40cd0d9c35dc639d05eeafa5dc5
[659c824]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/659c824e36450d279d6fd684bdf848530da137f5
[65c0078]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/65c007844cc6c275b86b36a2ff1b48340622a681
[66414e9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/66414e9db930cd116e78a692fa0590a3f574aea2
[6659509]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6659509095d19163bd65bd24a9a554cf25207395
[6660f68]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6660f686f02ca2d98c9cdfe3c72cc906e446df1f
[6671e40]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6671e40e458f0068097135fb37f7f5a279367396
[66eea98]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/66eea98eb83cb5d80ff5ce094c8da7b63e8c74d6
[676633a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/676633a72f464a1f71b369281207390fb1c2efd5
[67cb45a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/67cb45a47f6c277bc0afc9ac9689b2a05d7b5049
[67f779c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/67f779c32db3b4d5857dbbcd76c6fee77e050505
[6805e2b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6805e2b9257cecb545417531a008ec139a0b5c54
[68583d8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/68583d84a3722a27bec69a77984cd9e1167929bc
[6955a7b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6955a7b5c389a9b16651bf7e2350e12df2bc22a2
[69767ad]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/69767ad626e27e0fda881c1e62b374165bd17825
[699015e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/699015e02fa89058bb1379f5944bde296b0603e6
[69abc77]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/69abc77c352e813dc24128e9952da72c77979f1a
[69ef9f7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/69ef9f7b8d04b0afa5885040b51ef50c18873fea
[6a327d1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6a327d18341b245c61839b70cba29dc91f888f1b
[6b47b9c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6b47b9c1bc8b1cf297c987b3d4321cfe654238f5
[6baae7c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6baae7cc71bf42de7584025bf53843f3c0ff8f6c
[6c1954e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6c1954e7475b3ff64bedce86b8301b1e0f624c95
[6c82506]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6c825065687ef0469d4a4d1a64b9ef9e75e9ebea
[6d64fd6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6d64fd6d8ee1b5338877004d22ecfaf18ed47ba7
[6d67cdb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6d67cdbe11918f47fc440ea285a73d2be7b870d3
[6d89273]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6d89273608a68aaf4abc32f52a6739cca4fe3a77
[6e284ff]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6e284ff06182c3f684c16d49c6bfba8795a862b6
[6e68808]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6e68808588e0bb24fee292f2b236ed4adcbcbfd2
[6eff5c5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6eff5c553e8d094df5a0c4fe16fde501f0ee08f2
[6f36d21]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6f36d21e829ced48a2de9dc1ee6ed4e51b02aa78
[6f4a255]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6f4a255762be50bcee6f667f6af422b6eb5a2fef
[6f5862e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6f5862e346d005ab5087d012eb6a448bd65924d0
[6f7831f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6f7831f10b187cb72f0ec7568db8ae9c8482a146
[6ff633c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6ff633ccb93a9e75e0e0b7291a2571921d85092a
[705dd66]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/705dd66a0d7771b67d2d1b57de9619cd969939f7
[707d472]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/707d472928022d51dc7da2fe5322194928871f5b
[70a30a2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/70a30a2b0b651a438223f7249c6ce47931acaa92
[70dcb6e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/70dcb6e4bebf26ed60cd26df4eb321417f106da5
[7109c46]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7109c46d835f4d6f32b6284681a6286b68179abc
[713bb73]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/713bb7354358091926e524d3f29330f16da3646e
[715cff5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/715cff55bf0dd38c2538a3a522fa7503f2e86ec1
[7185ba4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7185ba477d0d184f9cdf28eb485e3ec4e5963f3b
[7432b7c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7432b7c5ea20165e443d6440fcc3cb84393f3a96
[749f8ce]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/749f8ce7aab2be9f0bf16133127a0d7fde3046c3
[7557a59]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7557a59e99faf297d2055d5d9ea86b4fbfe8ba5e
[75d48b0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/75d48b0ea51d6cc73085bd253d1abd7989a3a059
[75f7c9f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/75f7c9fa0fdb16e471281c701b71759e728df81d
[764689f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/764689fe0693c6a8588d13cde1c73f42e08b2a39
[7810632]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/781063204d0437481e6033a3f1cf5c6c66db102f
[78579a9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/78579a9158dc463e33c7b5ce1d248258bac89ae3
[78581db]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/78581dba9d33c9565fa25f0a829ca383471335f2
[79c1ea1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/79c1ea1edbed94e3376aed37b382d069144d6fab
[7a22f9f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7a22f9f4125fd49fcbb65a54c85f69c2ef728467
[7bdfa4d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7bdfa4df5ac857c185a7ce1dd4851637de86fa63
[7ca59ef]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7ca59ef47803ae183c7d72461324730f4a65f25b
[7cc5fbc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7cc5fbcc0c214c4daedfd3cc447fd788864fd9f9
[7cdaa4b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7cdaa4b6ac2c12f3829f345c8c56bd7bf6c19b13
[7d3305c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7d3305cc028ee0f963e0def854350e9d3eb69cb0
[7d6d9f2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7d6d9f2d701713046452cae3eb740a7ea6c2ea59
[7d8d35a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7d8d35aa8649386937ec73db7b20ea67eb7bd54f
[7d97c64]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7d97c64e3d28115e395e28c89e56fadc8b26f0af
[7d9bf20]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7d9bf20bd40e7c3f26c9b116af9e68d57ceca44d
[7daa34c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7daa34c8b08e3f3d05aa8257b172d96441015321
[7db097f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7db097f35da98d6166b671a714d7c307b4f8958f
[7e42275]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7e4227598054e4845ef2f4114b4ffa13313f0e9a
[7f3911b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7f3911b07cc4788fe2cdb4e8d421fe5f156cac59
[7f700e6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7f700e642e89cc7e65d89430f8f02e501ad7cdd1
[7f751c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7f751c12debde6b2dcd7377d880b20d2aa834f40
[7f9535d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7f9535da6c2513ffa99d5b4888864d0c911000d6
[7fb6abe]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7fb6abe6a51d0fa63c6ef1a543d5888cd43d5550
[808ef68]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/808ef6827cce5519a37305d7d02062ba079299c9
[80d06dd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/80d06ddf9245d7653827efa9aa8315ed2c634b11
[8123ef1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8123ef1843ffed2f79a403105d3bdc819c9bb0ba
[81959bd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/81959bd1a9129ff2f8419150d7dcf5fae8feacd0
[81c437b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/81c437b02b967b56dcb9f5fa0a25b083dfa3ed25
[81f94b8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/81f94b8ba9c8e0d35fddd828b1a1c4f10a9202bc
[82255c0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/82255c0ccbe5a57f2906ebb5626b75047f1ce20e
[8225996]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/822599689d7cef3844b5b602352e3e18197a00b7
[830dbbd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/830dbbd5e18ad8d53727026536b1b07c58411c35
[844c1e0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/844c1e06cd698eced1c6cd6f50645180b340ee82
[84abef3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/84abef39f6067a82a3fd749938d17b372bbf660b
[84cc63c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/84cc63c2d8c1398a4aa11da4e4e2d07abed4c04b
[84f3059]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/84f30592bad9b4395dc770d44dc807125d2ced02
[8514bb9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8514bb9e5dd831640c5a05509c67ed7573c23975
[8575cd8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8575cd87bd06ba3ad185a1be33aadd4022bbae40
[85bb57f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/85bb57fb3465c4ff46d220a69d02fef0c304fb4a
[85d51b6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/85d51b6b68b108f4a7c8cb9455961420c2cfff43
[85d5ba2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/85d5ba25c3b919f1c4b1159630de4702e126d5a9
[860b213]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/860b213f7690873f076c098c74b83bb8822a1ba9
[861be67]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/861be6713cfc5f4996251fe23e26f67dd80001d8
[8633d66]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8633d667dbb20d7dafa673925ad445d3e4691afc
[867a6fe]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/867a6fe259da60c9b4a5b9f4dc7b108605dca294
[877177b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/877177bc4a0ca42544ece0facd2f40273b86c239
[87a884b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/87a884bbfc07b43cf6b2cf8dadc64eab8bf7a702
[880e35a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/880e35aeedfb5449626a03c9131a1ccd33e017e3
[8821b40]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8821b40c3d4992a09b44e1e69d676d91296277be
[89209f9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/89209f92231ab984cf10552584f08e6bf7b2767b
[89f67f3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/89f67f3024dd9805313a2cff67ca6e0dc901fb40
[8a47ad8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8a47ad83cd68c7eec60db4ff734f8ead3d54b977
[8a50df1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8a50df10142c2c8d6c81eaabfb10919d1c3efa0b
[8a6af55]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8a6af558540e081c7da03c451bae439fca147e49
[8ba2106]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8ba2106c388aa4f785b02962ea408b1945b61280
[8bdeafd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8bdeafddf2131da83ad916da83ddacb27c363132
[8be5e30]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8be5e30f4b82fc97cb03e05d086412e050b333db
[8bee504]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8bee504d983335151c4aae8ae6ec94e1ff04e949
[8bf2c81]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8bf2c81214b7da088b688badfbc40abd54e9c07b
[8c23fd0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8c23fd059412310208b811d5d771caf617f3d0c0
[8c9a919]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8c9a919deb3aed74a11f45ca375f28ded421f4c5
[8cd40b5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8cd40b522d2e87dd69dd21704c5f128d6d05847b
[8d48787]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8d487876af1303b41c5a73c786d268eac57de224
[8d5e686]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8d5e6860ed3b6b5c3743bf5d9a5122a78cdccb3c
[8def2f6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8def2f69f1d0b55c73ccb0fe7e35435b67d79c6f
[8e3b72a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8e3b72a7c1e70dad9eacc62cb3171754799c79a6
[8e46d08]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8e46d08ec9543912761fe24c8cae5162b417586d
[8e6fc80]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8e6fc80beb3c1cbc52fcab7bdd8aad57c84806fe
[8e98a8f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8e98a8f387132a3a13f53d359086a80caa484cfd
[8ec4d01]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8ec4d012fffb16013940d723077cdc44af0b156a
[8ec6e24]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8ec6e2403291c0554fac111da98c4c89f6973801
[8f0eee7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8f0eee74270820d5b04eb0c6f48934969ed5bc4c
[8f14374]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8f1437483337f24a4c728b61d1754f9455ee0f5b
[8f75091]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8f7509161b267921fa4f4703c57280e6f1ede86f
[8f8e8b7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8f8e8b7a856f3b3fd3a529d85830032e73510a88
[8fbb9f8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8fbb9f8e4a058e79ebd9ea45752c62133c14cac8
[8fc4181]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8fc4181814ff995835076b5ad2dbb77492c52e6a
[9055612]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9055612289c8499748001d18c2a232cbf23fe30f
[908b2f5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/908b2f594fdbc1aa51313bba5f26db74ee332a4a
[9128a71]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9128a71a8131362709c35d506cf413db5b0bda00
[916259b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/916259ba70c903d2b2d85b4bd3eddffa98cec370
[91c3cfb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/91c3cfb23253f1745969924aaed183e8f108412c
[91d33d0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/91d33d02013968cc879e6c5d09e3d6e8eb9ee1e6
[91df1d0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/91df1d0162814f42958283517137d62f1c0afed2
[92b9754]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/92b97540ecc5b69f957552118a47498022d5a9c1
[93129e2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/93129e27339a653d4cbb635de37a0bea31a16e99
[93a58cc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/93a58cc0254e8f4965fe7e3d5cd702489a237ee0
[943619b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/943619bec76c9f49eac11ca7e94543bba2b8d8d7
[943ad62]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/943ad6258c6d01c3df3f97e35b7d0a2aa4f00136
[94d6862]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/94d6862e0d558e69f0e5b07db5a63ad7700d515b
[9522093]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9522093d0c183c35dec4c457214a219da905baa6
[966c336]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/966c3366ada13f8266e1d19b5761e05b1ef5fbbb
[97b69b2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/97b69b23cba21ec59e6a30a5e1fc1d6a642fccda
[97cf45e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/97cf45e94786b87b5a2d3fb2ecf2e696aeb4d1d9
[989ecdd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/989ecdd98ce86d9e4156dbd693c067f9a185a8ba
[98c4b36]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/98c4b362743dbf5b5ef95234caa389e74dcac1ac
[98ebe68]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/98ebe687ce608c985a5bce2d3e9410fa234a931a
[9948f0a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9948f0a9035b0883644f0a37d63d16a77158be5b
[9957cc5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9957cc56452652f87ac037175d3b16f273a735ea
[995bfb0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/995bfb074c2d01a91f940c30fb761b7833b50c22
[997a365]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/997a365d6a6c72f8a3e847f1c253b1f236f05a5f
[99b8c46]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/99b8c469d683365998e278c50e7a4a400cfc61c6
[9a23849]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9a2384992a0bdb78769a7aeba8393ab4767713d1
[9a37f84]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9a37f841ad435b4c36bf8b4fe93da7645fe61865
[9ba41e3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9ba41e3564b3058b238f0a05787373f788583b6e
[9c4e8e9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9c4e8e9b5006801ea8310baad780daffe6a7e0a9
[9c5fa57]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9c5fa576899d1529b06acf89221d44d262092d04
[9ceb5b4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9ceb5b48698e16b62a380d2c1f577f54156c4ac2
[9d3ff3e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9d3ff3ec738efcfd8240f6611953212c3aeb4a9b
[9d9179c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9d9179cf63c4167ac46b5c398b2c6b718ea9a022
[9ddaaee]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9ddaaeedecdd175672c38ba3d39c7521f08acc68
[9e2c253]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9e2c2533a3409676e470d716a76fe18189ded5ae
[9ee5e79]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9ee5e79687bba2e992ea01d990b86556ac8b7a85
[9f5391b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9f5391ba1ff4b7c8aa43d6ab3da57ee7693e0b9d
[9fbc631]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9fbc6318f6931ba60d43843b387c4bf049d4742e
[a01e57d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a01e57d237f55bba7e9541559c7bc0b6286cf8c0
[a069057]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a06905735804f5b63c6963fc10b2eb512b32488a
[a069776]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a0697763bf80d26b453027f27bbc22baec502052
[a08cc1d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a08cc1dee13e4e69cecb5d8c70646290b48025b1
[a09e9f6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a09e9f660ce2de3327a34879a5e184b3ef91a79e
[a0edb15]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a0edb157905810d46d3418098b829744b3444d0f
[a137866]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a13786623e5b9117418dc6ff86c1f0519e9074f0
[a1e1a68]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a1e1a680278843d4f871f5556bee679282a8d268
[a223eeb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a223eeb41ce8408366fd8944cb70559d3fb50202
[a256992]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a25699295eed0a20eeb3571e0c401d4c901928eb
[a2ebe40]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a2ebe40b2cf4a864a83a80ac302df1715e25d173
[a468ffa]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a468ffadc41916adb608b633acf0dd8f45d255a9
[a4eabe1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a4eabe11f15c788abeabbad8d11a447a99d3414c
[a4fd91f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a4fd91f4b1340a754754b8bec841eb60102988bf
[a503554]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a503554d9c0bbae7751b1e448156a7dc43f32def
[a5ad717]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a5ad717dc7d37586785b7375068defe352927e24
[a6099bc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a6099bc36884788f51bc5d9af5ad15451b1f2186
[a64be8e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a64be8e97cddfe1836aa5e1df20ff22836bc791f
[a71c5e8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a71c5e81761deb547c315296004167e13f82fe9b
[a746b06]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a746b065aa719a05f477224eba7fe551e62ebddc
[a7cfd10]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a7cfd10ee270b2ed0c25a952c83b8ffe7235ea02
[a7d3a39]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a7d3a394bfd68656fccfe7c2794294361d8836fa
[a7e93c6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a7e93c6068199f6b826e7aa1d21e2397d4c8e390
[a80facf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a80facf0bb435346b0c8c3a05d22b8e428ba680f
[a82e25b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a82e25b56c80e37c5ea6450c4a27a9ff1feb021b
[a88c662]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a88c662583ec0222b1842048b0d2f021582ebb6f
[a8a5f36]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a8a5f364fc531b08d8f5fb6245d64b0c70ab95ba
[a8a89bd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a8a89bd22db468b3d845d113eb1a2a843f7dbc67
[a8d5902]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a8d5902d3973b1c8fcbfa0925cb6870a28ba7db5
[a9902eb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a9902ebf6d78778516d39b7d4150494616ae1bb3
[aa3992c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aa3992cc919c644dc7fe3bc41abc2dd970fe3d2e
[aa4cf9d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aa4cf9d3fbc0b046ecfa814b298a27769783d209
[aa5bdbb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aa5bdbbcdbd2d36b08f11c0a252603526b7adce8
[aaaf78e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aaaf78e17cfeeea087fe9562fc65907b3847bc9e
[aab71b5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aab71b50e4464cae19f1add8b28613260345d9db
[aaf976b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aaf976b84513bcdb2395fab5349fc035e4601068
[ab11dae]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ab11dae73a6e56dd68815c076c1e86cbbf6ac058
[ab57920]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ab579207ea14141d3d4327f39b5fd23830a89f3a
[abdd5ba]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/abdd5bace78c295e5ac8c5a15fabd39b452b826d
[ac509a2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ac509a2c0f12541ac4db4107a423ada59732c4dd
[ac5749d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ac5749d32d335f800fc8f3636cfecd321ebefb42
[ac8d29b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ac8d29bb53de9b0bc06572f85073a1ac06f54087
[acc34ed]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/acc34edc4502c691381df03a3bf9c2aebde1a038
[acc9b5b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/acc9b5b8722b130d1551ea716628f096e7805c9b
[ad55722]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ad5572283ee4ee8263e337284a82db986f13742a
[addbc64]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/addbc642be40f93ba3df1588dcb165cbc9b4f0d1
[adf4007]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/adf400700122f4eb23fd63971b3f048e014d1781
[ae5bbae]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ae5bbaecb182211a8a96a5dd64c53fca7316c568
[aec8ceb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aec8cebca812844afec8050e30d93fb8fa3bb203
[aee27e4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aee27e45bc52c5a6839a66266d03a304d2608351
[aeed87f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aeed87f64b21581afb83924035cd22fd98d3dddc
[af46851]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/af46851919ced5582dd8d6c5b236edd3ac078061
[af93d66]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/af93d662852bbed6a3c13ca4f54ae4a63af56c20
[afc0dab]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/afc0dab53064bef4aec0f5181e25b8f96e0169f4
[b03da48]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b03da48883f07bd1e089f080dc4bc6fa9cfc8578
[b088be7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b088be725c367aabae07d4b60553693a5c2ddd80
[b0ade00]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b0ade003bea1149486c0a41240c6e66283a36e0d
[b199791]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b199791bdf4a131fc6873e84400fd17fdf802c68
[b1acadc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b1acadcc75c1b160281308cbf061e23abbf6a5b0
[b1dc6f9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b1dc6f927b19e6f1d722454b6792d467834096df
[b227f53]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b227f531a6f348cdd9b3fa5fe010adf979dd8e98
[b2530a5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b2530a582f9edcab94d80f9e53142ee801c8335f
[b2a0ab5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b2a0ab53877db5bf91b216baf3ba5e08853da559
[b2d6de5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b2d6de5072f1506077fa649b15912b7cb3064211
[b339c88]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b339c881ae94eb1b14c02462042c4c8e8416e951
[b371ef1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b371ef116f0ff7e4bf8f8d82d58e9a0daafb8427
[b4e9cf2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b4e9cf241b87b9a0aecfea1d59a9e110bf7d7091
[b50ce99]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b50ce9932b2b8502a113b85714f6ac9564c2645d
[b550573]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b5505730100a9780877eb3e1cb4d280f02845863
[b55341e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b55341ecd717344211bd79557f56f7fecaad2479
[b5d4d91]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b5d4d91779599bae9fc15d78c5e3db3f4a43f18b
[b5dd71e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b5dd71e8e19bc0094fa253ef1345a12996ced46d
[b63e848]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b63e84804022258362d0238f7a5a7b199da524f8
[b6d359f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b6d359fe3efb94ba8f85c7eaa1788665c392021d
[b715be5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b715be5564b434b10e566aba23a1737860ccc37f
[b723fed]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b723fed816b98dc1bfa9484909c53a8078a1335d
[b7a25d0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b7a25d0905f7aa8426eb97ada89a516620d81e77
[b875e10]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b875e10f8792292446465e8855b7dbb7048d4c4a
[b8989f3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b8989f3f0e848138b6de90b81b2c774e775a015d
[b8a8c34]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b8a8c34b650c47b815fc307346aecc69f35d192a
[b8a8ea7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b8a8ea751cd1db7cf2e973872f6a3f2d483c800e
[b8deef3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b8deef3439f8e8b9a949a0a1cfa16d2c027c391f
[b9d4c2c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b9d4c2c24c13c8f629c7ca6cab36941a1dc7a4b5
[ba75da9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ba75da987cde9b47cd1f3f298f0a350873898125
[bab5ee5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bab5ee53d297fd4d3cb21ce411cef4c01748d082
[baff6da]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/baff6da9a9bc313db65a613d5edae82d67aee4c2
[bbdb2cb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bbdb2cb61fabac44421596f4e3c64e725532e5c7
[bbe3b00]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bbe3b00626693af8310454616c08b8358fedb042
[bc41128]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bc4112866bb713538fc48c209408313c634306b2
[bc58af3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bc58af3466a3513570103d8b5ac4bbb53266f561
[bc5ea85]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bc5ea85c879fad5c7c4ae88cdc9de71290455d6a
[bd29d11]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bd29d11b49c2f0c8c2640200231df82eba1ed4ab
[bd44896]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bd44896a30627bafefa64c1cbc78229113130b9d
[bd49cdc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bd49cdc8220e8adcfea71f04c6ebcfb51946336b
[bd9b494]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bd9b494e083a2861f8c991cfe75f80f61d72ddef
[bdafe96]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bdafe96f41ea33ec27a840dbda74ed909f6f7532
[bea60aa]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bea60aae98c6f7b6ffbb23a30fc58d825397a3e0
[bf7596e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bf7596e325ac8a7514ca0427695c79074951fb2d
[c1262d4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c1262d43fcdfdf9b2d3604786757bdf3a8ed77cf
[c13d082]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c13d0820b77a54b1e15a0f42ecea6d6b250a9fc2
[c225eb6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c225eb65b2330d6f61580c37504421144308febc
[c240b82]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c240b82c292586648b2dbd345eba39d716ddd43f
[c2a2fc7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c2a2fc73c451a652c8bf917031567903ca8bf75a
[c332a73]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c332a73363492a1e1874e68fc0c12e3bfd2b96ae
[c35066c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c35066cd2cc01344259f00559186fbd1a12db527
[c3d0621]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c3d0621bef3a9d3ca2c3d9967860f839b4389fd6
[c4c7f30]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c4c7f3014b51280932244d5c132031f23642cf79
[c4f86ec]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c4f86ec51cd2e3b21260d7314398b34d0661fdd7
[c618a19]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c618a199f59706ad2cfca64e2c37bbe4b615faf1
[c622174]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c62217434c9ff7efb5ec9067464d5ba8b841ffb9
[c6f1899]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c6f189916c9fc9cbc4f69ea7a42c110497e7e819
[c7ec4eb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c7ec4ebfe41385a409265ef9dcb3ff4fa9222b03
[c80a044]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c80a0449b8729d3c64775e56de8fe27f21017c6f
[c8328f3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c8328f3ab256bf76a92b205f8eeebc49447bd25e
[c83e433]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c83e433c50687c9611cb298e64823ba9a2dcec6a
[c87ceff]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c87ceff11912ec3788f390cf454b1a84db5fd8a3
[c8fc525]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c8fc525dff93e1b29c0df61bf6cc593376910043
[c98bf25]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c98bf2527c00302e0f8c7aabbfe6de8c44136322
[c9a5de8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c9a5de8fb611785c5d8bfd6e6942f48006cf9814
[c9bc19e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c9bc19ecd6cad88742cfa3758e48fd606f489220
[c9dc70a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c9dc70a51be61bc46b43082e7227f873cb77ac10
[ca1c967]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ca1c967a1dd169b73f3002f120c40c7127060041
[cae9877]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cae987706e31a6c223e5af997fee32b537714efd
[cb698ec]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cb698ec9a74329fc7947db489769472f0cbaf4e8
[cb74da3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cb74da327e27b73e9724d8a28aafc164e6c9e0df
[cb8ed82]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cb8ed82e9ee18dc1b0f3ffdd2c22d99402a8c870
[cb9a91e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cb9a91ed337581abe68ce57ecada73b5bbbe5b26
[cc1cb8a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cc1cb8aa305b3dc17f9df7c0ad8c898bc931b0c2
[ccd4f0c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ccd4f0c94c53343c239d61e5cee680e7df8d312b
[cd52d9d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cd52d9d639e7f03836a5a36b197a04806c9a4ca6
[cd6584d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cd6584d727529af79bf61e7d8ef13876f8cf9b04
[cda6e99]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cda6e991ea22413221c90eacd9b5a16c875ef316
[cdaad46]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cdaad462bfea78e0e079853e198a32ec89a5d7bc
[cdd64df]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cdd64dfe9773aa85ccdcf1099290b273519169d6
[ce8518e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ce8518ead73f36025e708c567cfaa1d9d74d5f2c
[cea1c6f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cea1c6f892bc15f7aaa0f6b3e4cdeff074f7d5b5
[cebf281]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cebf2818fe60d8509af94fe623bf0d1f7ff44b17
[ced4667]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ced4667fd5f16682a46e70d435a9a473885c70b6
[cef66e5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cef66e50bd7b149177a635d0f2bb17e1b77799ec
[cf9153b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cf9153bc3cec7f038ae47397c9d0a9942d5f364e
[cf91c1d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cf91c1d2808a7658da8eb6263c3aca0ff3e5fb04
[cff9336]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cff93366ae59c85d01f5d818ea2e8c8c73cedb87
[d03737c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d03737cda4c53aba353a32f33fd32f7fa74738ad
[d073674]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d0736743fca8af8a4dde7d8317b72de269f5655b
[d091a79]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d091a792aa369ea4bff566bd321a4a9c9cbb589c
[d0f4166]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d0f4166b6610b624b6bb2d28a7acba407aea7ca5
[d13dffa]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d13dffa8e80c9f30c04ce18143e669a04e817a45
[d159553]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d159553e8eaf2166c6d3b6187c007ad3dfc21400
[d1dfe28]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d1dfe28abda8f6f8b46a1aebaee5750521fb5854
[d205c41]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d205c419687d0908828ff4f06f4e56351a7ea2f4
[d2158ee]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d2158ee2b1b23a68b3c4dd764863acadec08d6bb
[d23220a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d23220a23c9bd0a3eb7ba6a10bab7322b6523fb9
[d31c145]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d31c14564c2bc27ed4e7790a54b16d09a01c3be9
[d340bd2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d340bd2985295b3ccf4559c4ab1ac3588501ca4b
[d395d94]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d395d94cddeea82f7117682882407feb35258fad
[d3f5d8a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d3f5d8a4cd60ec6007977e7ebe4558c4a14789cd
[d3f723e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d3f723ed85b0c433c1c6c0a424ccf33ccb11a17d
[d4060a9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d4060a99ee69974ab4fc34ebc41522fd0a89ad4d
[d48099c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d48099c07d95b49914e4e4271679b4846dd6b608
[d4ed03e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d4ed03e046d292888e555de3b6955b396ef7fad0
[d4f5e0a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d4f5e0af96e5ce9d83c12e46a345dc5525d27a95
[d5af77d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d5af77deed057d599fd1c4b5c1f6222a7edba4c3
[d6bb2d1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d6bb2d1da026c16c4a301fa675653d8a0688a679
[d6caa25]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d6caa25bdea45f7a70323efa4bb7f93624644024
[d70104f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d70104fb19ee3e133188a14d49f2c57ab0a55e06
[d75fdfc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d75fdfc0fb7b34f4e6b5ac2cfbcbfca7df0ccf59
[d7d264b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d7d264ba4c9eb13170cfbcf5297ade21df622d48
[d882f96]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d882f968ae9011b112cb8f195171e5357747a6af
[d8af716]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d8af716b6de4335d7a5016274436d031d35cb78d
[d8d5641]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d8d56414c28f5ca7ba2db10420c1805270d80d7b
[d8faf4f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d8faf4fd010e303dad42c8a0a51520c03fd197b8
[d97f0ab]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d97f0ab7ba5ef0cfd4a7ea0ed9cb21f3770fc5da
[d9c3bf3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d9c3bf3b11b81434c738b00a40d13fd53a51f250
[d9f70ce]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d9f70ce89f21bc8e48184856257fddac0a0372e1
[da392e6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/da392e6d23a7ab2490d26b94b1ad8c213c0db4bb
[da6f91f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/da6f91faf961dff4f1adbf528cd4025d98cd3624
[da980d8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/da980d8bdcb4ac506db0862b11987de8eb859179
[daded23]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/daded23ce694301aadd19c09a07bb1d384668ce9
[db2e8f3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/db2e8f3cf4db912d32e74fcbdf09094c8b2f5128
[db5b6a5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/db5b6a5fbc301716f84682c4dae7e1691fcba413
[dc17064]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dc170648f88426308ea6e6622215b7c3618c3a32
[dc30ca6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dc30ca638c88714942f282de4cd464336e41f8de
[dc7b7c2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dc7b7c27f7d239fcf02d78981ea13a5563c88f88
[dd3637a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dd3637a70efb9a639e0b4e7a3db9d6c868752bfd
[dd63214]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dd63214e4877ab17811d0e1db6867cff6bb72e61
[dd8248c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dd8248c388a6f8df54c12f5dd010de613a0e21ee
[ddc2a80]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ddc2a8036d9ea83b75bdc5cf506c365f5b09a3a7
[df2bcde]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/df2bcdef317ceb778093641fe97b8cf5664bf4bd
[df628a7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/df628a72730e677e16a3053988983f752d71940a
[dfab1e7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dfab1e709a370d468ffb3540f3c6d3e280e97017
[dfd2898]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dfd2898d64411f280bbe7d04280a9c73d3a3b310
[dfd5f6b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dfd5f6b83b1c667dcf166e37fb2a2eaac7d3c99b
[dfd60d4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dfd60d4b29ce3ba0afe581c746d643cc5a6eccfa
[dff2927]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dff2927698abcac250fd3f0df7910c02818f6776
[e083e73]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e083e73e8fbdeab7c9421e729521c08bd9c77fbc
[e189fd2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e189fd21f8689048e404ddf19c279ad743203924
[e1dfb0a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e1dfb0a281d3d922ada33f53013accb2c765bd9d
[e1e4606]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e1e4606847459e742f9c5e51a860b8903b2bc5ce
[e30fd62]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e30fd625450cd6a6dd103487e881dec747daa8f2
[e317c87]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e317c87ece614341553f2d4b7926f1614a1a5b5a
[e346e18]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e346e184d9ab0af7969a796ef4c43814267aa7a3
[e375577]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e37557772d193a2e812598ed06ea0ab8656dd293
[e3b58f9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e3b58f922ce8b312ee1fc9b04b39e5dcf75cf1c4
[e3c0e0a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e3c0e0a430d6e27060b00db05c17f01d68361547
[e3ca0e2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e3ca0e225065cf4fe610fd0f49748dc8cab48f71
[e42d021]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e42d021c20b90e50c464541fb3d358ac24ce3b3a
[e43d48b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e43d48bfd451fe4aac2f90b0e19a357bf5a1c1b9
[e46b1f9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e46b1f943753dc0a5bf1b45b458f0fde643ebdf5
[e4f8059]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e4f8059e97c3ba25401f8752607a66fea4dee10d
[e51c30f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e51c30f16a3fb478829bade3350a429d54ee3e94
[e5c4c6e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e5c4c6ed1500f2e5636155f5935dda25e73ae5cb
[e5e7f45]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e5e7f45a1bc577211908f98bc9a9bbbf335cf332
[e6931ed]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e6931ed967f7ea795ecdecfaeeead533642445f5
[e6d03e4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e6d03e4d12efe115374a0279fd14fa2616275918
[e7172e4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e7172e4519383c352ed147aa42b3aeca646a690e
[e736a71]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e736a714f4b2a84e4b5d578c8789049c1bbc4df6
[e75daad]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e75daad23339e77fe0f36b3ef666c68f9d28b60a
[e790e68]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e790e68174ba0c42c75d5cf1783aac6a101e9c11
[e80df38]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e80df38691580c8377c5e3fd30a02617765ee69d
[e8406c5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e8406c56c77869ad8e70e9e1e7e448a0f458a204
[e8e7c9c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e8e7c9ca3934f030d2410e92880b836e2ec23868
[e9b501f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e9b501fc77259d0c1c050bedc5a61c3516e4c307
[e9ea121]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e9ea121ac4a53e44e02f63f4f5ffee16c83dd72a
[ea56625]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ea56625a23b270cc5bab87ca6e22541e80b89b34
[eb5a1c3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/eb5a1c333d495d5ffdd95c390992de1e2a26e92d
[ebf4fd3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ebf4fd3f494ad12521f1f9ef1d4548282447e8d0
[ec82a50]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ec82a5009bcf7a16aaa694eb478216b9567c87c1
[ecbdca1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ecbdca19f3817c77ba330e9e6413d3af94713422
[ed3d1fe]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ed3d1feb788121161ba66f9c1826a67ded941337
[ed5c04a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ed5c04ade1af13f2e22afc184336f9713f2b76e0
[ee4515f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ee4515f1fd7e5161b5eab5bce0262971996f843f
[ee4c9d0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ee4c9d0c34ef366f008f83767e0b2b88a9e90a4d
[ee95b63]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ee95b6364d51c7d8a6bd4259ceda8ec63d13f56b
[ee9d6c0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ee9d6c0f865bd38f0e0feb0b92cba6252fa83cd7
[eeb20bb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/eeb20bb8431bf75c9e2be3fbba8e64daafae3098
[eeb3632]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/eeb3632bd465d4937204f1d4c3e5f72a953bcfa6
[ef73bd5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ef73bd5c114916a2f430dcd9c26eb49ec98f3fcc
[ef7d59a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ef7d59a60d323655aa6b3616f0d10a78ab11b565
[ef80a0d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ef80a0d5d844ce2b8ec80391305f0b71fc18b518
[efb694d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/efb694dfe1f34fee33210b9b5e3a749cb9468be4
[effffe8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/effffe87f8390d5894ab8dcf1806b2dd5b54e493
[f00ebc4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f00ebc4ccb3e82ae2d54787d9e39a6bce3044032
[f0146fc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f0146fc0172a0f95718c22f531d43494740166f7
[f05b395]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f05b395b3006536c1221dff369c23644ed6a58b5
[f08dd9c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f08dd9c4bdc950c70d380d0a98c9546d8efd8c00
[f1baa4d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f1baa4d5f07e31c179c983a0b855cbc240903859
[f2199b3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f2199b30ca34e9d46d1e51436b2cfba7c9b2f64c
[f271285]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f271285d727e9aee05398bd101cf64628ff9e008
[f2e5b4d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f2e5b4d1afb28d371ada56f0f4d206e67fb9b644
[f32d765]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f32d7650d306ec4a5b0c6686dd7461f6323f6f12
[f355964]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f355964d7b4c6bcc0d5cd726df4ff360f2adac23
[f363682]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f363682cfb202a8fcfaf591c0db5f6fbbd472fa1
[f3749f5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f3749f5ba0323c8b5c685ff5bed0b63f472be3e9
[f420393]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f42039339adc2bbb24d983232ba5c9f52cf03316
[f46be6a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f46be6a032569b5726d4df69efc519cec1e8fb29
[f49fb33]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f49fb33dab085714a8050d36442c04bf504f731e
[f4cdd62]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f4cdd62e98bd1edb356650f70f116c44927f9673
[f56045a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f56045aa6c147246f30635240835e92bea224520
[f58f506]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f58f506f17d6b76343d5bd814749259e3b380cc2
[f5abc7a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f5abc7a12684e6ebf12721a64c95e76a7a620c6b
[f5bd4e3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f5bd4e3a8260d8bc5224c5cb851ac0dfe854ee7e
[f5bf771]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f5bf771e6f26407fd2066f4765193adb250955c9
[f5d9f89]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f5d9f894525051000b6779c4d8cc38675f45f37f
[f5f2be2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f5f2be2dd7d45cf1cc4df2638b6ec3e98a0075b3
[f6a72ff]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f6a72ff1328766f733fe6314ecdbc1429bb57e61
[f6b28c8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f6b28c8b819135e35facc425f739c2a0b7c20c4d
[f6b52fc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f6b52fc20a8893ce30443bdd27f8da11108d0e17
[f708e15]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f708e15eab0ca601699461565b7a396f84394526
[f7b4533]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f7b4533f180ccc94c27f8e42b9806199d147f5c1
[f8f977d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f8f977d1bde282c350758aa2ebcca56eaef81c4a
[f94d2a0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f94d2a0dddd5aa8afe73eb06963af6c3b40e3b01
[f968b46]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f968b462d74e05c806bf5560f356799aa40b7104
[fa5ff73]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fa5ff7329049623be8379968adf2946360a780cb
[fada0fc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fada0fc46b9d0516b4876a0d67593e43079084cd
[fb82255]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fb822555116d65cc3c7b43fd066e397b071f31de
[fc2a837]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fc2a8379ad2f848990c749418ebe4123cacbcf8b
[fc54823]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fc5482359615f1f1a0d83c4f34a1ca89834d38ff
[fcf91c9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fcf91c96ea0dd598594aec0fac23726426b4cd3b
[fcfc729]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fcfc7294018d7e2e559b42be2f70fd1df853514f
[fd1a1da]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fd1a1da02ab3378b0327d1ccaf19a08434f60baf
[fda30e5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fda30e592981b402a192fe6f74ac36febdc946c8
[fda61f8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fda61f8ffc7ddd95556f4109b9e735cdde2c1b93
[fdc1899]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fdc1899ac00ddde0355f09a5c6aaf6d79a1aeec7
[fe422d6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fe422d64df17d550cac10ae4306b02f5bf99964b
[febfd00]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/febfd00d66ac8586584882ec6c7a5b2a97683571
[ff0f469]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ff0f46959254dd193a3b7abb63699ac58106e204
[ff2cd81]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ff2cd81bbd533c59df2c8bac3c6ff2afea4c1048
