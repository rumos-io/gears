use std::ops::RangeBounds;

use database::Database;
use kv_store::{
    ext::UnwrapInfallible, types::kv::mutable::KVStoreMut, QueryableKVStore, TransactionalKVStore,
};

use crate::types::store::gas::{
    errors::GasStoreErrors,
    guard::GasGuard,
    prefix::{mutable::GasPrefixStoreMut, GasPrefixStore},
    range::GasRange,
};

use super::GasKVStore;

#[derive(Debug)]
pub struct GasKVStoreMut<'a, DB> {
    pub(super) guard: GasGuard,
    pub(super) inner: KVStoreMut<'a, DB>,
}

impl<'a, DB> GasKVStoreMut<'a, DB> {
    pub(crate) fn new(guard: GasGuard, inner: KVStoreMut<'a, DB>) -> Self {
        Self { guard, inner }
    }

    pub fn to_immutable(&'a self) -> GasKVStore<'a, DB> {
        GasKVStore {
            guard: self.guard.clone(),
            inner: self.inner.to_immutable(),
        }
    }
}

impl<'a, DB: Database> QueryableKVStore for GasKVStoreMut<'a, DB> {
    type Prefix = GasPrefixStore<'a, DB>;

    type Range = GasRange<'a, DB>;

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

    fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, _range: R) -> Self::Range {
        // GasRange::new_kv(self.inner.range(range), self.guard.clone())
        todo!()
    }
}

impl<'a, DB: Database> TransactionalKVStore for GasKVStoreMut<'a, DB> {
    type PrefixMut = GasPrefixStoreMut<'a, DB>;

    fn prefix_store_mut(self, prefix: impl IntoIterator<Item = u8>) -> Self::PrefixMut {
        GasPrefixStoreMut::new(self.guard, self.inner.prefix_store_mut(prefix))
    }

    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) -> Result<(), Self::Err> {
        let key = key.into_iter().collect::<Vec<_>>();
        let value = value.into_iter().collect::<Vec<_>>();

        self.guard.set(key.len(), value.len())?;

        self.inner.set(key, value).unwrap_infallible();

        Ok(())
    }
}

impl<'a, DB: Database> GasKVStoreMut<'a, DB> {
    pub fn delete(&mut self, k: &[u8]) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        self.guard.delete()?;

        Ok(self.inner.delete(k))
    }
}
