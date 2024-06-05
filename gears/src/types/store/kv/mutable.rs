use std::ops::Bound;

use database::Database;
use kv_store::types::kv::mutable::KVStoreMut;

use crate::types::store::{
    gas::{errors::GasStoreErrors, kv::mutable::GasKVStoreMut},
    prefix::{mutable::PrefixStoreMut, PrefixStore},
    range::StoreRange,
};

#[derive(Debug)]
pub enum StoreMutBackend<'a, DB> {
    Gas(GasKVStoreMut<'a, DB>),
    Kv(KVStoreMut<'a, DB>),
}

#[derive(Debug)]
pub struct StoreMut<'a, DB>(StoreMutBackend<'a, DB>);

impl<'a, DB> From<GasKVStoreMut<'a, DB>> for StoreMut<'a, DB> {
    fn from(value: GasKVStoreMut<'a, DB>) -> Self {
        Self(StoreMutBackend::Gas(value))
    }
}

impl<'a, DB> From<KVStoreMut<'a, DB>> for StoreMut<'a, DB> {
    fn from(value: KVStoreMut<'a, DB>) -> Self {
        Self(StoreMutBackend::Kv(value))
    }
}

impl<'a, DB: Database> StoreMut<'a, DB> {
    pub fn range(&'a self, range: (Bound<Vec<u8>>, Bound<Vec<u8>>)) -> StoreRange<'a, DB> {
        match &self.0 {
            StoreMutBackend::Gas(var) => StoreRange::from(var.range(range)),
            StoreMutBackend::Kv(var) => StoreRange::from(var.range(range)),
        }
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> PrefixStore<'a, DB> {
        match self.0 {
            StoreMutBackend::Gas(var) => var.prefix_store(prefix).into(),
            StoreMutBackend::Kv(var) => var.prefix_store(prefix).into(),
        }
    }

    pub fn prefix_store_mut(self, prefix: impl IntoIterator<Item = u8>) -> PrefixStoreMut<'a, DB> {
        match self.0 {
            StoreMutBackend::Gas(var) => var.prefix_store_mut(prefix).into(),
            StoreMutBackend::Kv(var) => var.prefix_store_mut(prefix).into(),
        }
    }
}

impl<DB: Database> StoreMut<'_, DB> {
    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        match &self.0 {
            StoreMutBackend::Gas(var) => Ok(var.get(k)?),
            StoreMutBackend::Kv(var) => Ok(var.get(k)),
        }
    }

    pub fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) -> Result<(), GasStoreErrors> {
        match &mut self.0 {
            StoreMutBackend::Gas(var) => Ok(var.set(key, value)?),
            StoreMutBackend::Kv(var) => Ok(var.set(key, value)),
        }
    }
}
