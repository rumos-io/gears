pub mod mutable;

use std::ops::Bound;

use database::Database;
use kv_store::types::kv::immutable::KVStore;

use super::{errors::GasStoreErrors, guard::GasGuard, prefix::GasPrefixStore, range::GasRange};

#[derive(Debug)]
pub struct GasKVStore<'a, DB> {
    pub(super) guard: GasGuard,
    pub(super) inner: KVStore<'a, DB>,
}

impl<'a, DB: Database> GasKVStore<'a, DB> {
    pub(crate) fn new(guard: GasGuard, inner: KVStore<'a, DB>) -> Self {
        Self { guard, inner }
    }

    pub fn range(&'a self, range: (Bound<Vec<u8>>, Bound<Vec<u8>>)) -> GasRange<'a, DB> {
        GasRange::new_kv(self.inner.range(range), self.guard.clone())
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> GasPrefixStore<'a, DB> {
        GasPrefixStore::new(self.guard, self.inner.prefix_store(prefix))
    }
}

impl<DB: Database> GasKVStore<'_, DB> {
    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        let value = self.inner.get(&k);

        self.guard
            .get(k.as_ref().len(), value.as_ref().map(|this| this.len()))?;

        Ok(value)
    }
}
