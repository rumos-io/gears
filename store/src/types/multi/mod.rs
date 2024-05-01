use database::{Database, PrefixDB};

use crate::{QueryableMultiKVStore, StoreKey};

use self::commit::CommitMultiStore;

use super::{
    kv::{KVStore, KVStoreBackend},
    query::multi::QueryMultiStore,
};

pub mod commit;
pub mod mutable;

#[derive(Debug, Clone)]
#[allow(dead_code)] // TODO:NOW
pub(crate) enum MultiStoreBackend<'a, DB, SK> {
    Commit(&'a CommitMultiStore<DB, SK>),
    Query(&'a QueryMultiStore<'a, DB, SK>),
}

#[derive(Debug, Clone)]
pub struct MultiStore<'a, DB, SK>(pub(crate) MultiStoreBackend<'a, DB, SK>);

// impl<'a, DB: Database, SK: StoreKey> MultiStore<'a, DB, SK> {}

impl<DB: Database, SK: StoreKey> QueryableMultiKVStore<PrefixDB<DB>, SK>
    for MultiStore<'_, DB, SK>
{
    fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        match self.0 {
            MultiStoreBackend::Commit(var) => {
                KVStore(KVStoreBackend::Commit(var.kv_store(store_key)))
            }
            MultiStoreBackend::Query(var) => var.kv_store(store_key),
        }
    }

    fn head_version(&self) -> u32 {
        match self.0 {
            MultiStoreBackend::Commit(var) => var.head_version,
            MultiStoreBackend::Query(var) => var.head_version(),
        }
    }

    fn head_commit_hash(&self) -> [u8; 32] {
        match self.0 {
            MultiStoreBackend::Commit(var) => var.head_commit_hash,
            MultiStoreBackend::Query(var) => var.head_commit_hash(),
        }
    }
}

impl<'a, DB, SK> From<&'a QueryMultiStore<'a, DB, SK>> for MultiStore<'a, DB, SK> {
    fn from(value: &'a QueryMultiStore<'a, DB, SK>) -> Self {
        MultiStore(MultiStoreBackend::Query(value))
    }
}

impl<'a, DB, SK> From<&'a CommitMultiStore<DB, SK>> for MultiStore<'a, DB, SK> {
    fn from(value: &'a CommitMultiStore<DB, SK>) -> Self {
        MultiStore(MultiStoreBackend::Commit(value))
    }
}
