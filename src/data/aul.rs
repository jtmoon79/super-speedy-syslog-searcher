//! A reconstructed Apple Unified Log record rendered as JSONL.

use crate::data::common::{DtBegEndPair, DtBegEndPairOpt, PrintableEvent};
use crate::data::datetime::DateTimeL;

#[derive(Debug)]
pub struct Aul {
    dt: DateTimeL,
    dt_beg_end: DtBegEndPairOpt,
    json: Vec<u8>,
}

impl Aul {
    pub fn new(dt: DateTimeL, dt_beg_end: DtBegEndPair, mut json: Vec<u8>) -> Self {
        json.push(b'\n');

        Self {
            dt,
            dt_beg_end: Some(dt_beg_end),
            json,
        }
    }
}

impl PrintableEvent for Aul {
    fn dt(&self) -> &DateTimeL {
        &self.dt
    }

    fn dt_beg_end(&self) -> &DtBegEndPairOpt {
        &self.dt_beg_end
    }

    fn as_bytes(&self) -> &[u8] {
        &self.json
    }
}
