use std::ops::RangeBounds;

use database::Database;

use crate::{
    kv::{application::ApplicationKVBank, transaction::TransactionKVBank},
    prefix::immutable::ImmutablePrefixStore,
    query::kv::QueryKVStore,
    range::Range,
};

/// Internal structure which holds different stores
#[derive(Debug, Clone)]
pub(crate) enum KVStoreBackend<'a, DB> {
    App(&'a ApplicationKVBank<DB>),
    Tx(&'a TransactionKVBank<DB>),
    Query(&'a QueryKVStore<DB>),
}

/// Non mutable kv store
#[derive(Debug, Clone)]
pub struct KVStore<'a, DB>(pub(crate) KVStoreBackend<'a, DB>);

impl<'a, DB: Database> KVStore<'a, DB> {
    pub fn into_range<R: RangeBounds<Vec<u8>> + Clone>(self, range: R) -> Range<'a, DB> {
        match self.0 {
            KVStoreBackend::App(var) => var.range(range),
            KVStoreBackend::Tx(var) => var.range(range),
            KVStoreBackend::Query(var) => var.range(range),
        }
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(
        self,
        prefix: I,
    ) -> ImmutablePrefixStore<'a, DB> {
        match self.0 {
            KVStoreBackend::App(var) => var.prefix_store(prefix),
            KVStoreBackend::Tx(var) => var.prefix_store(prefix),
            KVStoreBackend::Query(var) => var.prefix_store(prefix),
        }
    }
}

impl<DB: Database> KVStore<'_, DB> {
    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        match self.0 {
            KVStoreBackend::App(var) => var.get(k),
            KVStoreBackend::Tx(var) => var.get(k),
            KVStoreBackend::Query(var) => var.get(k),
        }
    }
}

impl<'a, DB> From<&'a ApplicationKVBank<DB>> for KVStore<'a, DB> {
    fn from(value: &'a ApplicationKVBank<DB>) -> Self {
        Self(KVStoreBackend::App(value))
    }
}

impl<'a, DB> From<&'a TransactionKVBank<DB>> for KVStore<'a, DB> {
    fn from(value: &'a TransactionKVBank<DB>) -> Self {
        Self(KVStoreBackend::Tx(value))
    }
}

impl<'a, DB> From<&'a QueryKVStore<DB>> for KVStore<'a, DB> {
    fn from(value: &'a QueryKVStore<DB>) -> Self {
        Self(KVStoreBackend::Query(value))
    }
}
