// src/tests/pydataevent_tests.rs

//! tests for `pydataevent.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::data::common::DtBegEndPairOpt;
use crate::data::datetime::ymdhmsl;
use crate::data::pydataevent::{
    PyDataEvent,
    EventBytes,
};
use crate::tests::common::FO_0;

#[test]
fn test_pydataevent() {
    let dt = ymdhmsl(&FO_0, 2024, 10, 5, 11, 30, 19, 195);
    let mut e_data = EventBytes::new();
    e_data.extend_from_slice(b"2024-10-05 11:30:19.195Z Sample py data event data");
    let pydataevent: PyDataEvent = PyDataEvent::new(
        e_data,
        dt,
        DtBegEndPairOpt::Some((0, 24)),
    );
    assert_eq!(pydataevent.len(), 50);
    assert_eq!(pydataevent.dt(), &dt);
    assert_eq!(pydataevent.dt_beg_end(), &DtBegEndPairOpt::Some((0, 24)));
    _ = pydataevent.as_bytes();
}
