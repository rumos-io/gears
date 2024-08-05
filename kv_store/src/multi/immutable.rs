use database::{prefix::PrefixDB, Database};

use crate::{
    ApplicationStore, StoreKey, TransactionStore,
    {
        kv::immutable::{KVStore, KVStoreBackend},
        query::QueryMultiStore,
    },
};

use super::MultiBank;

#[derive(Debug)]
pub(crate) enum MultiStoreBackend<'a, DB, SK> {
    Commit(&'a MultiBank<DB, SK, ApplicationStore>),
    Cache(&'a MultiBank<DB, SK, TransactionStore>),
    Query(&'a QueryMultiStore<DB, SK>),
}

#[derive(Debug)]
pub struct MultiStore<'a, DB, SK>(pub(crate) MultiStoreBackend<'a, DB, SK>);

impl<DB: Database, SK: StoreKey> MultiStore<'_, DB, SK> {
    pub fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
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

    pub fn head_version(&self) -> u32 {
        match self.0 {
            MultiStoreBackend::Commit(var) => var.head_version,
            MultiStoreBackend::Cache(var) => var.head_version,
            MultiStoreBackend::Query(var) => var.head_version,
        }
    }

    pub fn head_commit_hash(&self) -> [u8; 32] {
        match self.0 {
            MultiStoreBackend::Commit(var) => var.head_commit_hash,
            MultiStoreBackend::Cache(var) => var.head_commit_hash,
            MultiStoreBackend::Query(var) => var.head_commit_hash,
        }
    }
}

impl<'a, DB, SK> From<&'a QueryMultiStore<DB, SK>> for MultiStore<'a, DB, SK> {
    fn from(value: &'a QueryMultiStore<DB, SK>) -> Self {
        MultiStore(MultiStoreBackend::Query(value))
    }
}

impl<'a, DB, SK> From<&'a MultiBank<DB, SK, ApplicationStore>> for MultiStore<'a, DB, SK> {
    fn from(value: &'a MultiBank<DB, SK, ApplicationStore>) -> Self {
        MultiStore(MultiStoreBackend::Commit(value))
    }
}

impl<'a, DB, SK> From<&'a MultiBank<DB, SK, TransactionStore>> for MultiStore<'a, DB, SK> {
    fn from(value: &'a MultiBank<DB, SK, TransactionStore>) -> Self {
        MultiStore(MultiStoreBackend::Cache(value))
    }
}
