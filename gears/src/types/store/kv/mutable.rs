use std::{cell::RefCell, ops::RangeBounds, sync::Arc};

use database::Database;
use store_crate::{types::kv::mutable::KVStoreMut, QueryableKVStore, TransactionalKVStore};

use crate::types::{
    gas::{kind::TxKind, GasMeter},
    store::{
        errors::GasStoreErrors,
        guard::GasGuard,
        prefix::{mutable::GasStorePrefixMut, GasStorePrefix},
        range::GasRange,
    },
};

use super::GasKVStore;

#[derive(Debug)]
pub struct GasKVStoreMut<'a, DB> {
    pub(super) guard: GasGuard<'a>,
    pub(super) inner: KVStoreMut<'a, DB>,
}

impl<'a, DB> GasKVStoreMut<'a, DB> {
    pub(crate) fn new(guard: &'a mut GasMeter<TxKind>, inner: KVStoreMut<'a, DB>) -> Self {
        Self {
            guard: GasGuard(Arc::new(RefCell::new(guard))),
            inner,
        }
    }

    pub fn to_immutable(&'a self) -> GasKVStore<'a, DB> {
        GasKVStore {
            guard: self.guard.clone(),
            inner: self.inner.to_immutable(),
        }
    }
}

impl<'a, DB: Database> GasKVStoreMut<'a, DB> {
    pub fn get<R: AsRef<[u8]>>(&'a self, k: R) -> Result<Vec<u8>, GasStoreErrors> {
        self.to_immutable().get(k)
    }

    pub fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) -> Result<(), GasStoreErrors> {
        let key = key.into_iter().collect::<Vec<_>>();
        let value = value.into_iter().collect::<Vec<_>>();

        self.guard.set(key.len(), value.len())?;

        self.inner.set(key, value);

        Ok(())
    }

    pub fn delete(&mut self, k: &[u8]) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        self.guard.delete()?;

        Ok(self.inner.delete(k))
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> GasStorePrefix<'a, DB> {
        GasStorePrefix::new(self.guard, self.inner.prefix_store(prefix))
    }

    pub fn prefix_store_mut<I: IntoIterator<Item = u8>>(
        self,
        prefix: I,
    ) -> GasStorePrefixMut<'a, DB> {
        GasStorePrefixMut::new(self.guard, self.inner.prefix_store_mut(prefix))
    }

    pub fn range<R: RangeBounds<Vec<u8>> + Clone>(&'a self, range: R) -> GasRange<'a, R, DB> {
        GasRange::new_kv(self.inner.range(range), self.guard.clone())
    }
}
