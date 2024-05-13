use database::{Database, PrefixDB};

use crate::{
    types::kv_2::{
        immutable::{KVStoreBackend, KVStoreV2},
        mutable::{KVStoreBackendMut, KVStoreMutV2},
    },
    CacheKind, CommitKind, QueryableMultiKVStoreV2, StoreKey, TransactionalMultiKVStoreV2,
};

use super::{
    immutable::{MultiStoreBackend, MultiStoreV2},
    MultiStorage,
};

pub(crate) enum MultiStoreBackendMut<'a, DB, SK> {
    Commit(&'a mut MultiStorage<DB, SK, CommitKind>),
    Cache(&'a mut MultiStorage<DB, SK, CacheKind>),
}

pub struct MultiStoreMut<'a, DB, SK>(pub(crate) MultiStoreBackendMut<'a, DB, SK>);

impl<DB, SK> MultiStoreMut<'_, DB, SK> {
    pub fn to_immutable(&self) -> MultiStoreV2<'_, DB, SK> {
        match &self.0 {
            MultiStoreBackendMut::Commit(var) => MultiStoreV2(MultiStoreBackend::Commit(var)),
            MultiStoreBackendMut::Cache(var) => MultiStoreV2(MultiStoreBackend::Cache(var)),
        }
    }
}

impl<'a, DB: Database, SK: StoreKey> QueryableMultiKVStoreV2<PrefixDB<DB>, SK>
    for MultiStoreMut<'a, DB, SK>
{
    fn kv_store(&self, store_key: &SK) -> KVStoreV2<'_, PrefixDB<DB>> {
        match &self.0 {
            MultiStoreBackendMut::Commit(var) => {
                KVStoreV2(KVStoreBackend::Commit(var.kv_store(store_key)))
            }
            MultiStoreBackendMut::Cache(var) => {
                KVStoreV2(KVStoreBackend::Cache(var.kv_store(store_key)))
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

impl<DB: Database, SK: StoreKey> TransactionalMultiKVStoreV2<DB, SK> for MultiStoreMut<'_, DB, SK> {
    fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMutV2<'_, PrefixDB<DB>> {
        match &mut self.0 {
            MultiStoreBackendMut::Commit(var) => {
                KVStoreMutV2(KVStoreBackendMut::Commit(var.kv_store_mut(store_key)))
            }
            MultiStoreBackendMut::Cache(var) => {
                KVStoreMutV2(KVStoreBackendMut::Cache(var.kv_store_mut(store_key)))
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

impl<'a, DB, SK> From<&'a mut MultiStorage<DB, SK, CommitKind>> for MultiStoreMut<'a, DB, SK> {
    fn from(value: &'a mut MultiStorage<DB, SK, CommitKind>) -> Self {
        MultiStoreMut(MultiStoreBackendMut::Commit(value))
    }
}

impl<'a, DB, SK> From<&'a mut MultiStorage<DB, SK, CacheKind>> for MultiStoreMut<'a, DB, SK> {
    fn from(value: &'a mut MultiStorage<DB, SK, CacheKind>) -> Self {
        MultiStoreMut(MultiStoreBackendMut::Cache(value))
    }
}
