use std::{borrow::Cow, ops::Bound};

use database::Database;
use kv_store::{range::Range, types::prefix::range::PrefixRange};

use super::{errors::GasStoreErrors, guard::GasGuard};

#[derive(Debug)]
enum RangeBackend<'a, DB> {
    Kv(Range<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), DB>),
    Prefix(PrefixRange<'a, DB>),
}

pub struct GasRange<'a, DB> {
    inner: RangeBackend<'a, DB>,
    guard: GasGuard,
    err: Option<GasStoreErrors>,
}

impl<'a, DB> GasRange<'a, DB> {
    pub(super) fn new_kv(
        inner: Range<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), DB>,
        guard: GasGuard,
    ) -> Self {
        Self {
            inner: RangeBackend::Kv(inner),
            guard,
            err: None,
        }
    }

    pub(super) fn new_prefix(inner: PrefixRange<'a, DB>, guard: GasGuard) -> Self {
        Self {
            inner: RangeBackend::Prefix(inner),
            guard,
            err: None,
        }
    }
}

impl<'a, DB: Database> Iterator for GasRange<'a, DB> {
    type Item = (Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = match &mut self.inner {
            RangeBackend::Kv(var) => var.next(),
            RangeBackend::Prefix(var) => var.next(),
        };

        let err = self
            .guard
            .range(next.as_ref().map(|(key, val)| (key.len(), val.len())));

        if let Err(err) = err {
            self.err = Some(err);

            None
        } else {
            next
        }
    }
}
