use std::ops::RangeBounds;

use database::Database;

use crate::{
    kv::{application::ApplicationKVBank, transaction::TransactionKVBank},
    prefix::{immutable::ImmutablePrefixStore, mutable::MutablePrefixStore},
    range::Range,
};

use super::immutable::{KVStore, KVStoreBackend};

/// Internal structure which holds different stores
#[derive(Debug)]
pub(crate) enum KVStoreBackendMut<'a, DB> {
    App(&'a mut ApplicationKVBank<DB>),
    Tx(&'a mut TransactionKVBank<DB>),
}

/// Mutable variant of `KVStore`
#[derive(Debug)]
pub struct KVStoreMut<'a, DB>(pub(crate) KVStoreBackendMut<'a, DB>);

impl<'a, DB: Database> KVStoreMut<'a, DB> {
    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        match &mut self.0 {
            KVStoreBackendMut::App(var) => var.delete(k),
            KVStoreBackendMut::Tx(var) => var.delete(k),
        }
    }

    pub fn into_range<R: RangeBounds<Vec<u8>> + Clone>(self, range: R) -> Range<'a, DB> {
        match self.0 {
            KVStoreBackendMut::App(var) => var.range(range),
            KVStoreBackendMut::Tx(var) => var.range(range),
        }
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(
        self,
        prefix: I,
    ) -> ImmutablePrefixStore<'a, DB> {
        match self.0 {
            KVStoreBackendMut::App(var) => var.prefix_store(prefix),
            KVStoreBackendMut::Tx(var) => var.prefix_store(prefix),
        }
    }

    pub fn prefix_store_mut(
        self,
        prefix: impl IntoIterator<Item = u8>,
    ) -> MutablePrefixStore<'a, DB> {
        MutablePrefixStore {
            store: self,
            prefix: prefix.into_iter().collect(),
        }
    }
}

impl<'a, DB> KVStoreMut<'a, DB> {
    pub fn to_immutable(&self) -> KVStore<'_, DB> {
        match &self.0 {
            KVStoreBackendMut::App(var) => KVStore(KVStoreBackend::App(var)),
            KVStoreBackendMut::Tx(var) => KVStore(KVStoreBackend::Tx(var)),
        }
    }
}

impl<DB: Database> KVStoreMut<'_, DB> {
    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        match &self.0 {
            KVStoreBackendMut::App(var) => var.get(k),
            KVStoreBackendMut::Tx(var) => var.get(k),
        }
    }

    pub fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) {
        match &mut self.0 {
            KVStoreBackendMut::App(var) => var.set(key, value),
            KVStoreBackendMut::Tx(var) => var.set(key, value),
        };
    }
}

impl<'a, DB> From<&'a mut ApplicationKVBank<DB>> for KVStoreMut<'a, DB> {
    fn from(value: &'a mut ApplicationKVBank<DB>) -> Self {
        Self(KVStoreBackendMut::App(value))
    }
}

impl<'a, DB> From<&'a mut TransactionKVBank<DB>> for KVStoreMut<'a, DB> {
    fn from(value: &'a mut TransactionKVBank<DB>) -> Self {
        Self(KVStoreBackendMut::Tx(value))
    }
}
