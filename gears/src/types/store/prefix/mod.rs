use database::Database;
use store_crate::{types::prefix::immutable::ImmutablePrefixStore, ReadPrefixStore};

use super::{errors::GasStoreErrors, guard::GasGuard};

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

impl<DB: Database> GasStorePrefix<'_, DB> {
    fn get<T: AsRef<[u8]>>(&mut self, k: &T) -> Result<Vec<u8>, GasStoreErrors> {
        let value = self.inner.get(&k);

        self.guard
            .get(k.as_ref().len(), value.as_ref().map(|this| this.len()))?;

        value.ok_or(GasStoreErrors::NotFound)
    }
}
