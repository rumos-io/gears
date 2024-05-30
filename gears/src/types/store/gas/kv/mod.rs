pub mod mutable;

use std::ops::Bound;

use database::Database;
use kv_store::{ext::UnwrapInfallible, types::kv::immutable::KVStore, QueryableKVStore};

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
}

impl<'a, DB: Database> QueryableKVStore for GasKVStore<'a, DB> {
    type Prefix = GasPrefixStore<'a, DB>;

    type Err = GasStoreErrors;

    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Result<Option<Vec<u8>>, Self::Err> {
        let value = self.inner.get(&k).unwrap_infallible();

        self.guard
            .get(k.as_ref().len(), value.as_ref().map(|this| this.len()))?;

        Ok(value)
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> Self::Prefix {
        GasPrefixStore::new(self.guard, self.inner.prefix_store(prefix))
    }
}
