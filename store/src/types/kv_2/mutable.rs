use std::ops::RangeBounds;

use database::Database;

use crate::{
    range::Range,
    types::prefix_v2::{immutable::ImmutablePrefixStoreV2, mutable::MutablePrefixStoreV2},
    CacheKind, CommitKind, QueryableKVStoreV2, TransactionalKVStoreV2,
};

use super::{
    immutable::{KVStoreBackend, KVStoreV2},
    KVStorage,
};

/// Internal structure which holds different stores
pub(crate) enum KVStoreBackendMut<'a, DB> {
    Commit(&'a mut KVStorage<DB, CommitKind>),
    Cache(&'a mut KVStorage<DB, CacheKind>),
}

/// Mutable variant of `KVStore`
pub struct KVStoreMutV2<'a, DB>(pub(crate) KVStoreBackendMut<'a, DB>);

impl<'a, DB: Database> KVStoreMutV2<'a, DB> {
    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        match &mut self.0 {
            KVStoreBackendMut::Commit(var) => var.delete(k),
            KVStoreBackendMut::Cache(var) => var.delete(k),
        }
    }

    pub fn to_immutable(&self) -> KVStoreV2<'_, DB> {
        match &self.0 {
            KVStoreBackendMut::Commit(var) => KVStoreV2(KVStoreBackend::Commit(var)),
            KVStoreBackendMut::Cache(var) => KVStoreV2(KVStoreBackend::Cache(var)),
        }
    }
}

impl<'a, DB: Database> QueryableKVStoreV2<'a, DB> for KVStoreMutV2<'a, DB> {
    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        match &self.0 {
            KVStoreBackendMut::Commit(var) => var.get(k),
            KVStoreBackendMut::Cache(var) => var.get(k),
        }
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> ImmutablePrefixStoreV2<'a, DB> {
        match self.0 {
            KVStoreBackendMut::Commit(var) => var.prefix_store(prefix),
            KVStoreBackendMut::Cache(var) => var.prefix_store(prefix),
        }
    }

    fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB> {
        match &self.0 {
            KVStoreBackendMut::Commit(var) => var.range(range),
            KVStoreBackendMut::Cache(var) => var.range(range),
        }
    }
}

impl<'a, DB: Database> TransactionalKVStoreV2<'a, DB> for KVStoreMutV2<'a, DB> {
    fn prefix_store_mut(
        self,
        prefix: impl IntoIterator<Item = u8>,
    ) -> MutablePrefixStoreV2<'a, DB> {
        MutablePrefixStoreV2 {
            store: self,
            prefix: prefix.into_iter().collect(),
        }
    }

    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) {
        match &mut self.0 {
            KVStoreBackendMut::Commit(var) => var.set(key, value),
            KVStoreBackendMut::Cache(var) => var.set(key, value),
        }
    }
}

impl<'a, DB> From<&'a mut KVStorage<DB, CommitKind>> for KVStoreMutV2<'a, DB> {
    fn from(value: &'a mut KVStorage<DB, CommitKind>) -> Self {
        Self(KVStoreBackendMut::Commit(value))
    }
}

impl<'a, DB> From<&'a mut KVStorage<DB, CacheKind>> for KVStoreMutV2<'a, DB> {
    fn from(value: &'a mut KVStorage<DB, CacheKind>) -> Self {
        Self(KVStoreBackendMut::Cache(value))
    }
}
