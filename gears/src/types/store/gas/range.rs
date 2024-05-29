use std::borrow::Cow;

use database::Database;
use kv_store::types::prefix::range::PrefixRange;

use super::guard::GasGuard;

#[derive(Debug)]
enum RangeBackend<'a, DB> {
    // Kv(Range<'a, R, DB>),
    Prefix(PrefixRange<'a, DB>),
}

pub struct GasRange<'a, DB> {
    inner: RangeBackend<'a, DB>,
    guard: GasGuard,
}

impl<'a, DB> GasRange<'a, DB> {
    // pub(super) fn new_kv(inner: Range<'a, R, DB>, guard: GasGuard) -> Self {
    //     Self {
    //         inner: RangeBackend::Kv(inner),
    //         guard,
    //     }
    // }

    pub(super) fn new_prefix(inner: PrefixRange<'a, DB>, guard: GasGuard) -> Self {
        Self {
            inner: RangeBackend::Prefix(inner),
            guard,
        }
    }
}

impl<'a, DB: Database> Iterator for GasRange<'a, DB> {
    type Item = (Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = match &mut self.inner {
            // RangeBackend::Kv(var) => var.next(),
            RangeBackend::Prefix(var) => var.next(),
        };

        // TODO:NOW What to do with all this error handling?
        self.guard
            .range(next.as_ref().map(|(key, val)| (key.len(), val.len())))
            .ok()?;

        next
    }
}
