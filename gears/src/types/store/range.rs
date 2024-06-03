use std::{borrow::Cow, ops::Bound};

use database::Database;
use kv_store::{range::Range, types::prefix::range::PrefixRange};

use super::gas::{errors::GasStoreErrors, range::GasRange};

#[derive(Debug)]
enum StoreRangeBackend<'a, DB> {
    Gas(GasRange<'a, DB>),
    Kv(Range<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), DB>),
    Prefix(PrefixRange<'a, DB>),
}

#[derive(Debug)]
pub struct StoreRange<'a, DB>(StoreRangeBackend<'a, DB>);

impl<DB> StoreRange<'_, DB> {
    pub fn error(&self) -> Option<&GasStoreErrors> {
        match &self.0 {
            StoreRangeBackend::Gas(var) => var.error(),
            StoreRangeBackend::Kv(_) => None,
            StoreRangeBackend::Prefix(_) => None,
        }
    }
}

impl<'a, DB: Database> Iterator for StoreRange<'a, DB> {
    type Item = (Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            StoreRangeBackend::Gas(var) => var.next(),
            StoreRangeBackend::Kv(var) => var.next(),
            StoreRangeBackend::Prefix(var) => var.next(),
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
