// src/tests/evtx_tests.rs
// …

//! tests for `evtx.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::data::common::DtBegEndPairOpt;
use crate::data::evtx::{
    Evtx,
    RecordId,
};
use crate::tests::common::{
    EVTX_KPNP_DATA1_S,
    EVTX_KPNP_ENTRY1_DT,
};

use std::str; // for `from_utf8`

use ::lazy_static::lazy_static;
use ::test_case::test_case;
#[allow(unused_imports)]
use ::more_asserts::{
    assert_ge,
    assert_gt,
    assert_le,
    assert_lt,
    debug_assert_ge,
    debug_assert_gt,
    debug_assert_le,
    debug_assert_lt,
};


/// Error, broken data
pub const EVTX_KPNP_DATA1_S_E: &str = r#"
<?xml version="1.0" encoding="utf-8"?>
<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
  <System>
    <TimeCreated SystemTime="2023-03-10T03:49:43"#;

lazy_static! {
    /// EVTX #1
    static ref EVTX_1: Evtx = Evtx::new_(
        1,
        *EVTX_KPNP_ENTRY1_DT,
        String::from(EVTX_KPNP_DATA1_S),
        DtBegEndPairOpt::Some((420, 447)),
    );
}

#[test_case("", None)]
#[test_case(EVTX_KPNP_DATA1_S, DtBegEndPairOpt::Some((420, 447)))]
#[test_case(EVTX_KPNP_DATA1_S_E, None)]
fn test_get_dt_beg_end(input: &str, expect: DtBegEndPairOpt) {
    let result = Evtx::get_dt_beg_end(input);
    assert_eq!(result, expect);
}

#[test_case(&EVTX_1, 1, true)]
fn test_evtx(evtx: &Evtx, id_: RecordId, nl: bool) {
    assert_eq!(evtx.id(), id_);
    assert_eq!(evtx.ends_with_newline(), nl);
}
