use std::ops::Bound;

use database::Database;
use kv_store::types::kv::immutable::KVStore;

use super::{errors::StoreErrors, gas::kv::GasKVStore, prefix::PrefixStore, range::StoreRange};

pub mod mutable;

pub enum StoreBackend<'a, DB> {
    Gas(GasKVStore<'a, DB>),
    Kv(KVStore<'a, DB>),
}

pub struct Store<'a, DB>(StoreBackend<'a, DB>);

impl<'a, DB> From<GasKVStore<'a, DB>> for Store<'a, DB> {
    fn from(value: GasKVStore<'a, DB>) -> Self {
        Self(StoreBackend::Gas(value))
    }
}

impl<'a, DB> From<KVStore<'a, DB>> for Store<'a, DB> {
    fn from(value: KVStore<'a, DB>) -> Self {
        Self(StoreBackend::Kv(value))
    }
}

impl<'a, DB: Database> Store<'a, DB> {
    pub fn range(&'a self, range: (Bound<Vec<u8>>, Bound<Vec<u8>>)) -> StoreRange<'a, DB> {
        match &self.0 {
            StoreBackend::Gas(var) => StoreRange::from(var.range(range)),
            StoreBackend::Kv(var) => StoreRange::from(var.range(range)),
        }
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> PrefixStore<'a, DB> {
        match self.0 {
            StoreBackend::Gas(var) => var.prefix_store(prefix).into(),
            StoreBackend::Kv(var) => var.prefix_store(prefix).into(),
        }
    }
}

impl<DB: Database> Store<'_, DB> {
    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Result<Option<Vec<u8>>, StoreErrors> {
        match &self.0 {
            StoreBackend::Gas(var) => Ok(var.get(k)?),
            StoreBackend::Kv(var) => Ok(var.get(k)),
        }
    }
}
