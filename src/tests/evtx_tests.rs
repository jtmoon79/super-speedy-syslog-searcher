// src/tests/evtx_tests.rs
// …

//! tests for `evtx.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::str; // for `from_utf8`

use ::lazy_static::lazy_static;
use ::test_case::test_case;

use crate::data::common::DtBegEndPairOpt;
use crate::data::evtx::{
    Evtx,
    EvtxRS,
    RecordId,
    Timestamp,
};
use crate::tests::common::{
    EVTX_KPNP_DATA1_S,
    EVTX_KPNP_ENTRY1_DT,
};

pub const EVTX_KPNP_DATA1_ID: RecordId = 1;

/// Error, broken data
pub const EVTX_KPNP_DATA1_S_E: &str = r#"
<?xml version="1.0" encoding="utf-8"?>
<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
  <System>
    <TimeCreated SystemTime="2023-03-10T03:49:43"#;

lazy_static! {
    static ref EVTX_KPNP_DATA1_TIMESTAMP: Timestamp = Timestamp::new(1678420183, 0).unwrap();
    /// EVTX #1
    static ref EVTX_1: Evtx = Evtx::new_forced(
        *EVTX_KPNP_ENTRY1_DT,
        DtBegEndPairOpt::Some((420, 447)),
        EvtxRS {
            event_record_id: EVTX_KPNP_DATA1_ID,
            timestamp: *EVTX_KPNP_DATA1_TIMESTAMP,
            data: String::from(EVTX_KPNP_DATA1_S),
        }
    );
}

#[test_case("", None)]
#[test_case(EVTX_KPNP_DATA1_S, DtBegEndPairOpt::Some((420, 447)))]
#[test_case(EVTX_KPNP_DATA1_S_E, None)]
fn test_get_dt_beg_end(
    input: &str,
    expect: DtBegEndPairOpt,
) {
    let result = Evtx::get_dt_beg_end(input);
    assert_eq!(result, expect);
}

#[test_case(&EVTX_1, 1, true)]
fn test_evtx(
    evtx: &Evtx,
    id_: RecordId,
    nl: bool,
) {
    assert_eq!(evtx.id(), id_, "mismatched id");
    assert_eq!(evtx.ends_with_newline(), nl, "mismatched newline status");
}
