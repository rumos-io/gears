use std::{borrow::Cow, ops::RangeBounds};

use database::Database;
use kv_store::{range::Range, types::prefix::range::PrefixRange};

use super::guard::GasGuard;

#[derive(Debug)]
enum RangeBackend<'a, R: RangeBounds<Vec<u8>>, DB> {
    Kv(Range<'a, R, DB>),
    Prefix(PrefixRange<'a, DB>),
}

pub struct GasRange<'a, R: RangeBounds<Vec<u8>>, DB> {
    inner: RangeBackend<'a, R, DB>,
    guard: GasGuard,
}

impl<'a, R: RangeBounds<Vec<u8>>, DB> GasRange<'a, R, DB> {
    pub(super) fn new_kv(inner: Range<'a, R, DB>, guard: GasGuard) -> Self {
        Self {
            inner: RangeBackend::Kv(inner),
            guard,
        }
    }

    pub(super) fn new_prefix(inner: PrefixRange<'a, DB>, guard: GasGuard) -> Self {
        Self {
            inner: RangeBackend::Prefix(inner),
            guard,
        }
    }
}

impl<'a, R: RangeBounds<Vec<u8>>, DB: Database> Iterator for GasRange<'a, R, DB> {
    type Item = (Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = match &mut self.inner {
            RangeBackend::Kv(var) => var.next(),
            RangeBackend::Prefix(var) => var.next(),
        };

        // TODO:NOW What to do with all this error handling?
        self.guard
            .range(next.as_ref().map(|(key, val)| (key.len(), val.len())))
            .ok()?;

        next
    }
}
