use database::Database;

use crate::{
    range::Range,
    types::{prefix_v2::immutable::ImmutablePrefixStoreV2, query_v2::kv::QueryKVStoreV2},
    CacheKind, CommitKind, QueryableKVStoreV2,
};

use super::KVStorage;

/// Internal structure which holds different stores
pub(crate) enum KVStoreBackend<'a, DB> {
    Commit(&'a KVStorage<DB, CommitKind>),
    Cache(&'a KVStorage<DB, CacheKind>),
    Query(&'a QueryKVStoreV2<'a, DB>),
}

/// Non mutable kv store
pub struct KVStoreV2<'a, DB>(pub(crate) KVStoreBackend<'a, DB>);

impl<'a, DB: Database> QueryableKVStoreV2<'a, DB> for KVStoreV2<'a, DB> {
    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        match self.0 {
            KVStoreBackend::Commit(var) => var.get(k),
            KVStoreBackend::Cache(var) => var.get(k),
            KVStoreBackend::Query(var) => var.get(k),
        }
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> ImmutablePrefixStoreV2<'a, DB> {
        match self.0 {
            KVStoreBackend::Commit(var) => var.prefix_store(prefix),
            KVStoreBackend::Cache(var) => var.prefix_store(prefix),
            KVStoreBackend::Query(var) => var.prefix_store(prefix),
        }
    }

    fn range<R: std::ops::RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB> {
        match self.0 {
            KVStoreBackend::Commit(var) => var.range(range),
            KVStoreBackend::Cache(var) => var.range(range),
            KVStoreBackend::Query(var) => var.range(range),
        }
    }
}

impl<'a, DB> From<&'a KVStorage<DB, CommitKind>> for KVStoreV2<'a, DB> {
    fn from(value: &'a KVStorage<DB, CommitKind>) -> Self {
        Self(KVStoreBackend::Commit(value))
    }
}

impl<'a, DB> From<&'a KVStorage<DB, CacheKind>> for KVStoreV2<'a, DB> {
    fn from(value: &'a KVStorage<DB, CacheKind>) -> Self {
        Self(KVStoreBackend::Cache(value))
    }
}

impl<'a, DB> From<&'a QueryKVStoreV2<'a, DB>> for KVStoreV2<'a, DB> {
    fn from(value: &'a QueryKVStoreV2<'a, DB>) -> Self {
        Self(KVStoreBackend::Query(value))
    }
}
