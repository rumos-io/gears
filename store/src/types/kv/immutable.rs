use database::Database;

use crate::{
    range::Range,
    types::{prefix::immutable::ImmutablePrefixStore, query::kv::QueryKVStore},
    CacheKind, CommitKind, QueryableKVStore,
};

use super::KVBank;

/// Internal structure which holds different stores
#[derive(Debug)]
pub(crate) enum KVStoreBackend<'a, DB> {
    Commit(&'a KVBank<DB, CommitKind>),
    Cache(&'a KVBank<DB, CacheKind>),
    Query(&'a QueryKVStore<'a, DB>),
}

/// Non mutable kv store
#[derive(Debug)]
pub struct KVStore<'a, DB>(pub(crate) KVStoreBackend<'a, DB>);

impl<'a, DB: Database> QueryableKVStore<'a, DB> for KVStore<'a, DB> {
    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        match self.0 {
            KVStoreBackend::Commit(var) => var.get(k),
            KVStoreBackend::Cache(var) => var.get(k),
            KVStoreBackend::Query(var) => var.get(k),
        }
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> ImmutablePrefixStore<'a, DB> {
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

impl<'a, DB> From<&'a KVBank<DB, CommitKind>> for KVStore<'a, DB> {
    fn from(value: &'a KVBank<DB, CommitKind>) -> Self {
        Self(KVStoreBackend::Commit(value))
    }
}

impl<'a, DB> From<&'a KVBank<DB, CacheKind>> for KVStore<'a, DB> {
    fn from(value: &'a KVBank<DB, CacheKind>) -> Self {
        Self(KVStoreBackend::Cache(value))
    }
}

impl<'a, DB> From<&'a QueryKVStore<'a, DB>> for KVStore<'a, DB> {
    fn from(value: &'a QueryKVStore<'a, DB>) -> Self {
        Self(KVStoreBackend::Query(value))
    }
}
