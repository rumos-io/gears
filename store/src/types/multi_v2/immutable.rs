use database::{Database, PrefixDB};

use crate::{
    types::{
        kv_2::immutable::{KVStoreBackend, KVStoreV2},
        query::multi::QueryMultiStore,
    },
    CacheKind, CommitKind, QueryableMultiKVStoreV2, StoreKey,
};

use super::MultiStorage;

pub(crate) enum MultiStoreBackend<'a, DB, SK> {
    Commit(&'a MultiStorage<DB, SK, CommitKind>),
    Cache(&'a MultiStorage<DB, SK, CacheKind>),
    Query(&'a QueryMultiStore<'a, DB, SK>),
}

pub struct MultiStoreV2<'a, DB, SK>(pub(crate) MultiStoreBackend<'a, DB, SK>);

impl<DB: Database, SK: StoreKey> QueryableMultiKVStoreV2<PrefixDB<DB>, SK>
    for MultiStoreV2<'_, DB, SK>
{
    fn kv_store(&self, store_key: &SK) -> KVStoreV2<'_, PrefixDB<DB>> {
        match self.0 {
            MultiStoreBackend::Commit(var) => {
                KVStoreV2(KVStoreBackend::Commit(var.kv_store(store_key)))
            }
            MultiStoreBackend::Cache(var) => {
                KVStoreV2(KVStoreBackend::Cache(var.kv_store(store_key)))
            }
            MultiStoreBackend::Query(var) => var.kv_store(store_key),
        }
    }

    fn head_version(&self) -> u32 {
        match self.0 {
            MultiStoreBackend::Commit(var) => var.head_version,
            MultiStoreBackend::Cache(var) => var.head_version,
            MultiStoreBackend::Query(var) => var.head_version(),
        }
    }

    fn head_commit_hash(&self) -> [u8; 32] {
        match self.0 {
            MultiStoreBackend::Commit(var) => var.head_commit_hash,
            MultiStoreBackend::Cache(var) => var.head_commit_hash,
            MultiStoreBackend::Query(var) => var.head_commit_hash(),
        }
    }
}

impl<'a, DB, SK> From<&'a QueryMultiStore<'a, DB, SK>> for MultiStoreV2<'a, DB, SK> {
    fn from(value: &'a QueryMultiStore<'a, DB, SK>) -> Self {
        MultiStoreV2(MultiStoreBackend::Query(value))
    }
}

impl<'a, DB, SK> From<&'a MultiStorage<DB, SK, CommitKind>> for MultiStoreV2<'a, DB, SK> {
    fn from(value: &'a MultiStorage<DB, SK, CommitKind>) -> Self {
        MultiStoreV2(MultiStoreBackend::Commit(value))
    }
}

impl<'a, DB, SK> From<&'a MultiStorage<DB, SK, CacheKind>> for MultiStoreV2<'a, DB, SK> {
    fn from(value: &'a MultiStorage<DB, SK, CacheKind>) -> Self {
        MultiStoreV2(MultiStoreBackend::Cache(value))
    }
}
