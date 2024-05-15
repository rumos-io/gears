use database::{prefix::PrefixDB, Database};

use crate::{
    types::{
        kv::immutable::{KVStore, KVStoreBackend},
        query::QueryMultiStore,
    },
    CacheKind, CommitKind, QueryableMultiKVStore, StoreKey,
};

use super::MultiBank;

#[derive(Debug)]
pub(crate) enum MultiStoreBackend<'a, DB, SK> {
    Commit(&'a MultiBank<DB, SK, CommitKind>),
    Cache(&'a MultiBank<DB, SK, CacheKind>),
    Query(&'a QueryMultiStore<DB, SK>),
}

#[derive(Debug)]
pub struct MultiStore<'a, DB, SK>(pub(crate) MultiStoreBackend<'a, DB, SK>);

impl<DB: Database, SK: StoreKey> QueryableMultiKVStore<PrefixDB<DB>, SK>
    for MultiStore<'_, DB, SK>
{
    fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        match self.0 {
            MultiStoreBackend::Commit(var) => {
                KVStore(KVStoreBackend::Commit(var.kv_store(store_key)))
            }
            MultiStoreBackend::Cache(var) => {
                KVStore(KVStoreBackend::Cache(var.kv_store(store_key)))
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

impl<'a, DB, SK> From<&'a QueryMultiStore<DB, SK>> for MultiStore<'a, DB, SK> {
    fn from(value: &'a QueryMultiStore<DB, SK>) -> Self {
        MultiStore(MultiStoreBackend::Query(value))
    }
}

impl<'a, DB, SK> From<&'a MultiBank<DB, SK, CommitKind>> for MultiStore<'a, DB, SK> {
    fn from(value: &'a MultiBank<DB, SK, CommitKind>) -> Self {
        MultiStore(MultiStoreBackend::Commit(value))
    }
}

impl<'a, DB, SK> From<&'a MultiBank<DB, SK, CacheKind>> for MultiStore<'a, DB, SK> {
    fn from(value: &'a MultiBank<DB, SK, CacheKind>) -> Self {
        MultiStore(MultiStoreBackend::Cache(value))
    }
}
