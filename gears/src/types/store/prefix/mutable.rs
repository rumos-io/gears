use database::Database;

use crate::types::store::{errors::GasStoreErrors, kv::mutable::GasKVStoreMut};

use super::GasStorePrefix;

pub struct GasStorePrefixMut<'a, DB> {
    inner: GasKVStoreMut<'a, DB>,
    prefix: Vec<u8>,
}

impl<'a, DB> GasStorePrefixMut<'a, DB> {
    pub(crate) fn new(inner: GasKVStoreMut<'a, DB>, prefix: impl IntoIterator<Item = u8>) -> Self {
        Self {
            inner,
            prefix: prefix.into_iter().collect(),
        }
    }

    pub fn to_immutable(&mut self) -> GasStorePrefix<'_, DB> {
        GasStorePrefix {
            inner: self.inner.to_immutable(),
            prefix: self.prefix.clone(),
        }
    }
}

impl<DB: Database> GasStorePrefixMut<'_, DB> {
    fn get<T: AsRef<[u8]> + ?Sized>(&mut self, k: &T) -> Result<Vec<u8>, GasStoreErrors> {
        let full_key = [&self.prefix, k.as_ref()].concat();
        self.inner.get(&full_key)
    }

    pub fn delete(&mut self, k: &[u8]) -> Result<Vec<u8>, GasStoreErrors> {
        let full_key = [&self.prefix, k].concat();
        self.inner.delete(&full_key)
    }

    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        k: KI,
        v: VI,
    ) -> Result<(), GasStoreErrors> {
        let full_key = [self.prefix.clone(), k.into_iter().collect()].concat();
        self.inner.set(full_key, v)
    }
}
