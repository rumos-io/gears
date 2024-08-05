use std::borrow::Cow;

use database::Database;
use kv_store::{range::Range, store::prefix::range::PrefixRange};

use crate::types::store::gas::errors::GasStoreErrors;

#[derive(Debug)]
enum InfallibleRangeBackend<'a, DB> {
    Gas(crate::types::store::gas::range::infallible::RangeIter<'a, DB>),
    Kv(Range<'a, DB>),
    Prefix(PrefixRange<'a, DB>),
}

pub struct RangeIter<'a, DB>(InfallibleRangeBackend<'a, DB>);

impl<DB> RangeIter<'_, DB> {
    pub fn error(&self) -> Option<GasStoreErrors> {
        match &self.0 {
            InfallibleRangeBackend::Gas(var) => var.error().cloned(),
            InfallibleRangeBackend::Kv(_) => None,
            InfallibleRangeBackend::Prefix(_) => None,
        }
    }
}

impl<'a, DB: Database> Iterator for RangeIter<'a, DB> {
    type Item = (Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            InfallibleRangeBackend::Gas(var) => var.next(),
            InfallibleRangeBackend::Kv(var) => var.next(),
            InfallibleRangeBackend::Prefix(var) => var.next(),
        }
    }
}

impl<'a, DB> From<crate::types::store::gas::range::infallible::RangeIter<'a, DB>>
    for RangeIter<'a, DB>
{
    fn from(value: crate::types::store::gas::range::infallible::RangeIter<'a, DB>) -> Self {
        Self(InfallibleRangeBackend::Gas(value))
    }
}

impl<'a, DB> From<Range<'a, DB>> for RangeIter<'a, DB> {
    fn from(value: Range<'a, DB>) -> Self {
        Self(InfallibleRangeBackend::Kv(value))
    }
}

impl<'a, DB> From<PrefixRange<'a, DB>> for RangeIter<'a, DB> {
    fn from(value: PrefixRange<'a, DB>) -> Self {
        Self(InfallibleRangeBackend::Prefix(value))
    }
}
