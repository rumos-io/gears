use std::ops::Bound;

use database::Database;
use kv_store::{
    ext::UnwrapInfallible, types::kv::mutable::KVStoreMut, QueryableKVStore, TransactionalKVStore,
};

use crate::types::store::{
    gas::{errors::GasStoreErrors, kv::mutable::GasKVStoreMut},
    prefix::{mutable::PrefixStoreMut, PrefixStore},
};

pub enum StoreMutBackend<'a, DB> {
    Gas(GasKVStoreMut<'a, DB>),
    Kv(KVStoreMut<'a, DB>),
}

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

impl<'a, DB: Database> QueryableKVStore for StoreMut<'a, DB> {
    type Prefix = PrefixStore<'a, DB>;

    type Range = kv_store::range::Range<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), DB>;

    type Err = GasStoreErrors;

    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Result<Option<Vec<u8>>, Self::Err> {
        match &self.0 {
            StoreMutBackend::Gas(var) => var.get(k),
            StoreMutBackend::Kv(var) => Ok(var.get(k).unwrap_infallible()),
        }
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> Self::Prefix {
        match self.0 {
            StoreMutBackend::Gas(var) => var.prefix_store(prefix).into(),
            StoreMutBackend::Kv(var) => var.prefix_store(prefix).into(),
        }
    }

    fn range<R: std::ops::RangeBounds<Vec<u8>> + Clone>(&self, _range: R) -> Self::Range {
        todo!() // TODO:NOW
    }
}

impl<'a, DB: Database> TransactionalKVStore for StoreMut<'a, DB> {
    type PrefixMut = PrefixStoreMut<'a, DB>;

    fn prefix_store_mut(self, prefix: impl IntoIterator<Item = u8>) -> Self::PrefixMut {
        match self.0 {
            StoreMutBackend::Gas(var) => var.prefix_store_mut(prefix).into(),
            StoreMutBackend::Kv(var) => var.prefix_store_mut(prefix).into(),
        }
    }

    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) -> Result<(), Self::Err> {
        match &mut self.0 {
            StoreMutBackend::Gas(var) => var.set(key, value),
            StoreMutBackend::Kv(var) => Ok(var.set(key, value).unwrap_infallible()),
        }
    }
}
