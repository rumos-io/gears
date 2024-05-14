use database::{Database, PrefixDB};

use crate::{
    types::kv::{
        immutable::{KVStore, KVStoreBackend},
        mutable::{KVStoreBackendMut, KVStoreMut},
    },
    CacheKind, CommitKind, QueryableMultiKVStore, StoreKey, TransactionalMultiKVStore,
};

use super::{
    immutable::{MultiStore, MultiStoreBackend},
    MultiBank,
};

#[derive(Debug)]
pub(crate) enum MultiStoreBackendMut<'a, DB, SK> {
    Commit(&'a mut MultiBank<DB, SK, CommitKind>),
    Cache(&'a mut MultiBank<DB, SK, CacheKind>),
}

#[derive(Debug)]
pub struct MultiStoreMut<'a, DB, SK>(pub(crate) MultiStoreBackendMut<'a, DB, SK>);

impl<DB, SK> MultiStoreMut<'_, DB, SK> {
    pub fn to_immutable(&self) -> MultiStore<'_, DB, SK> {
        match &self.0 {
            MultiStoreBackendMut::Commit(var) => MultiStore(MultiStoreBackend::Commit(var)),
            MultiStoreBackendMut::Cache(var) => MultiStore(MultiStoreBackend::Cache(var)),
        }
    }
}

impl<'a, DB: Database, SK: StoreKey> QueryableMultiKVStore<PrefixDB<DB>, SK>
    for MultiStoreMut<'a, DB, SK>
{
    fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        match &self.0 {
            MultiStoreBackendMut::Commit(var) => {
                KVStore(KVStoreBackend::Commit(var.kv_store(store_key)))
            }
            MultiStoreBackendMut::Cache(var) => {
                KVStore(KVStoreBackend::Cache(var.kv_store(store_key)))
            }
        }
    }

    fn head_version(&self) -> u32 {
        match &self.0 {
            MultiStoreBackendMut::Commit(var) => var.head_version,
            MultiStoreBackendMut::Cache(var) => var.head_version,
        }
    }

    fn head_commit_hash(&self) -> [u8; 32] {
        match &self.0 {
            MultiStoreBackendMut::Commit(var) => var.head_commit_hash,
            MultiStoreBackendMut::Cache(var) => var.head_commit_hash,
        }
    }
}

impl<DB: Database, SK: StoreKey> TransactionalMultiKVStore<DB, SK> for MultiStoreMut<'_, DB, SK> {
    fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        match &mut self.0 {
            MultiStoreBackendMut::Commit(var) => {
                KVStoreMut(KVStoreBackendMut::Commit(var.kv_store_mut(store_key)))
            }
            MultiStoreBackendMut::Cache(var) => {
                KVStoreMut(KVStoreBackendMut::Cache(var.kv_store_mut(store_key)))
            }
        }
    }

    fn caches_clear(&mut self) {
        match &mut self.0 {
            MultiStoreBackendMut::Commit(var) => var.caches_clear(),
            MultiStoreBackendMut::Cache(var) => var.caches_clear(),
        }
    }
}

impl<'a, DB, SK> From<&'a mut MultiBank<DB, SK, CommitKind>> for MultiStoreMut<'a, DB, SK> {
    fn from(value: &'a mut MultiBank<DB, SK, CommitKind>) -> Self {
        MultiStoreMut(MultiStoreBackendMut::Commit(value))
    }
}

impl<'a, DB, SK> From<&'a mut MultiBank<DB, SK, CacheKind>> for MultiStoreMut<'a, DB, SK> {
    fn from(value: &'a mut MultiBank<DB, SK, CacheKind>) -> Self {
        MultiStoreMut(MultiStoreBackendMut::Cache(value))
    }
}
