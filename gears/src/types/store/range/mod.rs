pub mod infallible;
pub mod result;
use std::{borrow::Cow, ops::Bound};

use database::Database;
use infallible::RangeIter;
use kv_store::{range::Range, types::prefix::range::PrefixRange};

use super::{errors::StoreErrors, gas::range::GasRange};

#[derive(Debug)]
enum StoreRangeBackend<'a, DB> {
    Gas(GasRange<'a, DB>),
    Kv(Range<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), DB>),
    Prefix(PrefixRange<'a, DB>),
}

#[derive(Debug)]
pub struct StoreRange<'a, DB>(StoreRangeBackend<'a, DB>);

impl<'a, DB> StoreRange<'a, DB> {
    pub fn to_infallible_iter(self) -> RangeIter<'a, DB> {
        match self.0 {
            StoreRangeBackend::Gas(var) => var.to_infallible_iter().into(),
            StoreRangeBackend::Kv(var) => var.into(),
            StoreRangeBackend::Prefix(var) => var.into(),
        }
    }
}

impl<'a, DB: Database> Iterator for StoreRange<'a, DB> {
    type Item = Result<(Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>), StoreErrors>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            StoreRangeBackend::Gas(var) => {
                var.next().map(|this| this.map_err(|e| StoreErrors::Gas(e)))
            }
            StoreRangeBackend::Kv(var) => var.next().map(|e| Ok(e)),
            StoreRangeBackend::Prefix(var) => var.next().map(|e| Ok(e)),
        }
    }
}

impl<'a, DB> From<GasRange<'a, DB>> for StoreRange<'a, DB> {
    fn from(value: GasRange<'a, DB>) -> Self {
        Self(StoreRangeBackend::Gas(value))
    }
}

impl<'a, DB> From<Range<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), DB>> for StoreRange<'a, DB> {
    fn from(value: Range<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), DB>) -> Self {
        Self(StoreRangeBackend::Kv(value))
    }
}

impl<'a, DB> From<PrefixRange<'a, DB>> for StoreRange<'a, DB> {
    fn from(value: PrefixRange<'a, DB>) -> Self {
        Self(StoreRangeBackend::Prefix(value))
    }
}
