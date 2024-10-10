use std::borrow::Cow;

use database::Database;
use infallible::RangeIter;
use kv_store::{range::Range, store::prefix::range::PrefixRange};

use super::{errors::GasStoreErrors, guard::GasGuard};

pub mod infallible;

#[derive(Debug)]
enum RangeBackend<'a, DB> {
    Kv(Range<'a, DB>),
    Prefix(PrefixRange<'a, DB>),
}

#[derive(Debug)]
pub struct GasRange<'a, DB> {
    inner: RangeBackend<'a, DB>,
    guard: GasGuard,
}

impl<'a, DB> GasRange<'a, DB> {
    pub fn rev_iter(self) -> Self {
        let Self { inner, guard } = self;
        let inner = match inner {
            RangeBackend::Kv(range) => RangeBackend::Kv(range.rev_iter()),
            RangeBackend::Prefix(range) => RangeBackend::Prefix(range.rev_iter()),
        };

        Self { inner, guard }
    }
}

impl<'a, DB> GasRange<'a, DB> {
    pub(super) fn new_kv(inner: Range<'a, DB>, guard: GasGuard) -> Self {
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

    pub fn to_infallible_iter(self) -> RangeIter<'a, DB> {
        RangeIter::from(self)
    }
}

impl<'a, DB: Database> Iterator for GasRange<'a, DB> {
    type Item = Result<(Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>), GasStoreErrors>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match &mut self.inner {
            RangeBackend::Kv(var) => var.next(),
            RangeBackend::Prefix(var) => var.next(),
        };

        let err = self.guard.range(
            next.as_ref()
                .map(|(key, val)| (key.len(), val.len(), &***key)),
        );

        match err {
            Ok(_) => next.map(Ok),
            Err(err) => Some(Err(err)),
        }
    }
}
