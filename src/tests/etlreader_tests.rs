// src/tests/etlreader_tests.rs

//! tests for `etlreader.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::too_many_arguments)]

use crate::common::{
    Bytes,
};
use crate::data::common::{
    DtBegEndPairOpt,
};
use crate::data::datetime::{
    ymdhmsn,
    FixedOffset,
};
use crate::data::etl::{
    Etl,
    EtlDecoder,
    EtlEvent,
};
use crate::readers::etlreader::EtlReader;

/// File for test.
///
/// ```text
/// $ ./target/release/s4 ./logs/Windows11Pro/Logs/SIH/SIH.20230422.034724.362.1.etl
/// 2023-04-22T03:47:24.3632943-07:00 Provider={68fdd900-4a3e-11d1-84f4-0000f80464e3} ProviderName="MSNT_SystemTrace" EventName="Header" EventId=0 Version=2 Level=0 Opcode=0 Task=0 Keywords=0x0 PID=6412 TID=3240 HookId=0x0000 BufferSize=4096 Version=83951626 ProviderVersion=22621 NumberOfProcessors=1 EndTime=133266341204136027 TimerResolution=156250 MaxFileSize=128 LogFileMode=0x11002009 BuffersWritten=2 StartBuffers=1 PointerSize=8 EventsLost=0 CPUSpeed=4491 LoggerName=0xA LogFileName=0x7 BootTime=133264396075000000 PerfFreq=10000000 StartTime=133266340443632943 ReservedFlags=0x1 BuffersLost=0 SessionNameString="SIH_trace_log" LogFileNameString="C:\\Windows\\Logs\\SIH\\SIH.20230422.034724.362.1.etl"
/// 2023-04-22T03:47:24.3632943-07:00 Provider={68fdd900-4a3e-11d1-84f4-0000f80464e3} ProviderName="MSNT_SystemTrace" EventName="PartitionInfoExtension" EventId=0 Version=2 Level=0 Opcode=80 Task=0 Keywords=0x0 PID=6412 TID=3240 HookId=0x0050 Payload=0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
/// 2023-04-22T03:47:24.4722782-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="wmain"
/// 2023-04-22T03:47:24.4724118-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="cV = r4azpSFmbE6m+FuC09jWSA.0.1"
/// 2023-04-22T03:47:24.5091471-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="Retrieving SLS response from server using ETAG \"XAopazV00XDWnJCwkmEWRv6JkbjRA9QSSZ2+e/3MzEk=_1440\"..."
/// 2023-04-22T03:47:25.5884987-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="DNS Resiliency Feature is switched ON."
/// 2023-04-22T03:47:26.6136426-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="report [0] on [fe3cr.delivery.mp.microsoft.com] with Alternate DNS needed[false], fPassDefault[true], fPassFallback[false]"
/// 2023-04-22T03:47:45.0305483-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="report [0] on [slscr.update.microsoft.com] with Alternate DNS needed[false], fPassDefault[true], fPassFallback[false]"
/// 2023-04-22T03:47:45.0316204-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="Normal start."
/// 2023-04-22T03:47:45.0382128-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="Retrieving SLS response from server using ETAG \"MT1EoJH/qrWpWwGRkx7sbsS28A32Rz55YIcdvvtjCGK=_1440\"..."
/// 2023-04-22T03:47:45.7255414-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=3 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="*FAILED* [80245108] DoWithCatchHResult caught"
/// 2023-04-22T03:47:45.7255624-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="NoOp success."
/// ```
const ETL_FILE1_PATH: &str = "./logs/Windows11Pro/Logs/SIH/SIH.20230422.034724.362.1.etl";
#[allow(non_upper_case_globals)]
const FO_m7: FixedOffset = FixedOffset::east_opt(-7 * 3600).unwrap();

lazy_static::lazy_static! {
    static ref ETL_FILE1_DATA: Vec<Etl> = {
        let etl_data: Vec<Etl> = vec![
            Etl::new(
                ymdhmsn(&FO_m7, 2023, 4, 22, 3, 47, 24, 363294300),
                DtBegEndPairOpt::Some((0, 38)),
                Bytes::from(br#"2023-04-22T03:47:24.3632943-07:00 Provider={68fdd900-4a3e-11d1-84f4-0000f80464e3} ProviderName="MSNT_SystemTrace" EventName="Header" EventId=0 Version=2 Level=0 Opcode=0 Task=0 Keywords=0x0 PID=6412 TID=3240 HookId=0x0000 BufferSize=4096 Version=83951626 ProviderVersion=22621 NumberOfProcessors=1 EndTime=133266341204136027 TimerResolution=156250 MaxFileSize=128 LogFileMode=0x11002009 BuffersWritten=2 StartBuffers=1 PointerSize=8 EventsLost=0 CPUSpeed=4491 LoggerName=0xA LogFileName=0x7 BootTime=133264396075000000 PerfFreq=10000000 StartTime=133266340443632943 ReservedFlags=0x1 BuffersLost=0 SessionNameString="SIH_trace_log" LogFileNameString="C:\\Windows\\Logs\\SIH\\SIH.20230422.034724.362.1.etl""#),
            ),
            Etl::new(
                ymdhmsn(&FO_m7, 2023, 4, 22, 3, 47, 24, 363294300),
                DtBegEndPairOpt::Some((0, 38)),
                Bytes::from(br#"2023-04-22T03:47:24.3632943-07:00 Provider={68fdd900-4a3e-11d1-84f4-0000f80464e3} ProviderName="MSNT_SystemTrace" EventName="PartitionInfoExtension" EventId=0 Version=2 Level=0 Opcode=80 Task=0 Keywords=0x0 PID=6412 TID=3240 HookId=0x0050 Payload=0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"#),
            ),
            Etl::new(
                ymdhmsn(&FO_m7, 2023, 4, 22, 3, 47, 24, 472278200),
                DtBegEndPairOpt::Some((0, 38)),
                Bytes::from(br#"2023-04-22T03:47:24.4722782-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="wmain""#),
            ),
            Etl::new(
                ymdhmsn(&FO_m7, 2023, 4, 22, 3, 47, 24, 472411800),
                DtBegEndPairOpt::Some((0, 38)),
                Bytes::from(br#"2023-04-22T03:47:24.4724118-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="cV = r4azpSFmbE6m+FuC09jWSA.0.1""#),
            ),
            Etl::new(
                ymdhmsn(&FO_m7, 2023, 4, 22, 3, 47, 25, 509147100),
                DtBegEndPairOpt::Some((0, 38)),
                Bytes::from(br#"2023-04-22T03:47:24.5091471-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="Retrieving SLS response from server using ETAG \"XAopazV00XDWnJCwkmEWRv6JkbjRA9QSSZ2+e/3MzEk=_1440\"...""#),
            ),
            Etl::new(
                ymdhmsn(&FO_m7, 2023, 4, 22, 3, 47, 25, 509147100),
                DtBegEndPairOpt::Some((0, 38)),
                Bytes::from(br#"2023-04-22T03:47:24.5091471-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="Retrieving SLS response from server using ETAG \"XAopazV00XDWnJCwkmEWRv6JkbjRA9QSSZ2+e/3MzEk=_1440\"...""#),
            ),
            Etl::new(
                ymdhmsn(&FO_m7, 2023, 4, 22, 3, 47, 25, 588498700),
                DtBegEndPairOpt::Some((0, 38)),
                Bytes::from(br#"2023-04-22T03:47:25.5884987-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="DNS Resiliency Feature is switched ON.""#),
            ),
            Etl::new(
                ymdhmsn(&FO_m7, 2023, 4, 22, 3, 47, 45, 030548300),
                DtBegEndPairOpt::Some((0, 38)),
                Bytes::from(br#"2023-04-22T03:47:45.0305483-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="report [0] on [slscr.update.microsoft.com] with Alternate DNS needed[false], fPassDefault[true], fPassFallback[false]""#),
            ),
            Etl::new(
                ymdhmsn(&FO_m7, 2023, 4, 22, 3, 47, 45, 0316204),
                DtBegEndPairOpt::Some((0, 38)),
                Bytes::from(br#"2023-04-22T03:47:45.0316204-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="Normal start.""#),
            ),
            Etl::new(
                ymdhmsn(&FO_m7, 2023, 4, 22, 3, 47, 45, 0382128),
                DtBegEndPairOpt::Some((0, 38)),
                Bytes::from(br#"2023-04-22T03:47:45.0382128-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="Retrieving SLS response from server using ETAG \"MT1EoJH/qrWpWwGRkx7sbsS28A32Rz55YIcdvvtjCGK=_1440\"...""#),
            ),
            Etl::new(
                ymdhmsn(&FO_m7, 2023, 4, 22, 3, 47, 45, 7255414),
                DtBegEndPairOpt::Some((0, 38)),
                Bytes::from(br#"2023-04-22T03:47:45.7255414-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=3 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="*FAILED* [80245108] DoWithCatchHResult caught""#),
            ),
            Etl::new(
                ymdhmsn(&FO_m7, 2023, 4, 22, 3, 47, 45, 7255624),
                DtBegEndPairOpt::Some((0, 38)),
                Bytes::from(br#"2023-04-22T03:47:45.7255624-07:00 Provider={9906081d-e45a-4f41-a53f-2ac2e0225de1} ProviderName="SIHTraceLogging" EventName="SIH" EventId=0 Version=0 Level=4 Opcode=0 Task=0 Keywords=0x400000 PID=6412 TID=3240 Info="NoOp success.""#),
            ),
        ];

        etl_data
    };
}

/// create a `EtlReader` for the file at `ETL_FILE1_PATH`
/// process the Events, compare them against expected results
#[test]
fn test_etlreader_file1() {
    // TODO: assert matches to `ETL_FILE1_DATA`
}
