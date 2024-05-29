use std::ops::RangeBounds;

use database::Database;
use kv_store::{
    ext::UnwrapInfallible, types::prefix::immutable::ImmutablePrefixStore, ReadPrefixStore,
};

use super::{errors::GasStoreErrors, guard::GasGuard, range::GasRange};

pub mod mutable;

pub struct GasPrefixStore<'a, DB> {
    inner: ImmutablePrefixStore<'a, DB>,
    guard: GasGuard,
}

impl<'a, DB> GasPrefixStore<'a, DB> {
    pub(crate) fn new(guard: GasGuard, inner: ImmutablePrefixStore<'a, DB>) -> Self {
        Self { inner, guard }
    }
}

impl<DB: Database> ReadPrefixStore for GasPrefixStore<'_, DB> {
    type Err = GasStoreErrors;

    fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Result<Option<Vec<u8>>, Self::Err> {
        let value = self.inner.get(&k).unwrap_infallible();

        self.guard
            .get(k.as_ref().len(), value.as_ref().map(|this| this.len()))?;

        Ok(value)
    }
}

impl<'a, DB: Database> GasPrefixStore<'a, DB> {
    pub fn range<R: RangeBounds<Vec<u8>> + Clone>(&'a self, range: R) -> GasRange<'a, DB> {
        GasRange::new_prefix(self.inner.range(range), self.guard.clone())
    }
}
