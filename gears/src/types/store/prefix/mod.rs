use std::ops::RangeBounds;

use database::Database;
use store_crate::{types::prefix::immutable::ImmutablePrefixStore, ReadPrefixStore};

use super::{errors::GasStoreErrors, guard::GasGuard, range::GasRange};

pub mod mutable;

pub struct GasStorePrefix<'a, DB> {
    inner: ImmutablePrefixStore<'a, DB>,
    guard: GasGuard<'a>,
}

impl<'a, DB> GasStorePrefix<'a, DB> {
    pub(crate) fn new(guard: GasGuard<'a>, inner: ImmutablePrefixStore<'a, DB>) -> Self {
        Self { inner, guard }
    }
}

impl<DB: Database> ReadPrefixStore for GasStorePrefix<'_, DB> {
    type GetErr = GasStoreErrors;

    fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Result<Vec<u8>, Self::GetErr> {
        let value = self.inner.get(&k).ok();

        self.guard
            .get(k.as_ref().len(), value.as_ref().map(|this| this.len()))?;

        value.ok_or(GasStoreErrors::NotFound)
    }
}

impl<'a, DB: Database> GasStorePrefix<'a, DB> {
    pub fn range<R: RangeBounds<Vec<u8>> + Clone>(&'a self, range: R) -> GasRange<'a, R, DB> {
        GasRange::new_prefix(self.inner.range(range), self.guard.clone())
    }
}
