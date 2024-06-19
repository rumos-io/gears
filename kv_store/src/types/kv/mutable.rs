use std::ops::Bound;

use database::Database;

use crate::{
    range::Range,
    types::prefix::{immutable::ImmutablePrefixStore, mutable::MutablePrefixStore},
    ApplicationStore, TransactionStore,
};

use super::{
    immutable::{KVStore, KVStoreBackend},
    KVBank,
};

/// Internal structure which holds different stores
#[derive(Debug)]
pub(crate) enum KVStoreBackendMut<'a, DB> {
    Commit(&'a mut KVBank<DB, ApplicationStore>),
    Cache(&'a mut KVBank<DB, TransactionStore>),
}

/// Mutable variant of `KVStore`
#[derive(Debug)]
pub struct KVStoreMut<'a, DB>(pub(crate) KVStoreBackendMut<'a, DB>);

impl<'a, DB: Database> KVStoreMut<'a, DB> {
    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        match &mut self.0 {
            KVStoreBackendMut::Commit(var) => var.delete(k),
            KVStoreBackendMut::Cache(var) => var.delete(k),
        }
    }

    pub fn range(
        self,
        range: (Bound<Vec<u8>>, Bound<Vec<u8>>),
    ) -> Range<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), DB> {
        match self.0 {
            KVStoreBackendMut::Commit(var) => var.range(range),
            KVStoreBackendMut::Cache(var) => var.range(range),
        }
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(
        self,
        prefix: I,
    ) -> ImmutablePrefixStore<'a, DB> {
        match self.0 {
            KVStoreBackendMut::Commit(var) => var.prefix_store(prefix),
            KVStoreBackendMut::Cache(var) => var.prefix_store(prefix),
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
            KVStoreBackendMut::Commit(var) => KVStore(KVStoreBackend::Commit(var)),
            KVStoreBackendMut::Cache(var) => KVStore(KVStoreBackend::Cache(var)),
        }
    }
}

impl<DB: Database> KVStoreMut<'_, DB> {
    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        match &self.0 {
            KVStoreBackendMut::Commit(var) => var.get(k),
            KVStoreBackendMut::Cache(var) => var.get(k),
        }
    }

    pub fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) {
        match &mut self.0 {
            KVStoreBackendMut::Commit(var) => var.set(key, value),
            KVStoreBackendMut::Cache(var) => var.set(key, value),
        };
    }
}

impl<'a, DB> From<&'a mut KVBank<DB, ApplicationStore>> for KVStoreMut<'a, DB> {
    fn from(value: &'a mut KVBank<DB, ApplicationStore>) -> Self {
        Self(KVStoreBackendMut::Commit(value))
    }
}

impl<'a, DB> From<&'a mut KVBank<DB, TransactionStore>> for KVStoreMut<'a, DB> {
    fn from(value: &'a mut KVBank<DB, TransactionStore>) -> Self {
        Self(KVStoreBackendMut::Cache(value))
    }
}
