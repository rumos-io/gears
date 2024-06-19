use std::ops::RangeBounds;

use database::Database;
use kv_store::types::prefix::immutable::ImmutablePrefixStore;

use super::{errors::GasStoreErrors, guard::GasGuard, range::GasRange};

pub mod mutable;

#[derive(Debug, Clone)]
pub struct GasPrefixStore<'a, DB> {
    inner: ImmutablePrefixStore<'a, DB>,
    guard: GasGuard,
}

impl<'a, DB> GasPrefixStore<'a, DB> {
    pub(crate) fn new(guard: GasGuard, inner: ImmutablePrefixStore<'a, DB>) -> Self {
        Self { inner, guard }
    }
}

impl<DB: Database> GasPrefixStore<'_, DB> {
    pub fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        let value = self.inner.get(&k);

        self.guard
            .get(k.as_ref().len(), value.as_ref().map(|this| this.len()))?;

        Ok(value)
    }
}

impl<'a, DB: Database> GasPrefixStore<'a, DB> {
    pub fn into_range<R: RangeBounds<Vec<u8>> + Clone>(self, range: R) -> GasRange<'a, DB> {
        GasRange::new_prefix(self.inner.into_range(range), self.guard.clone())
    }
}
