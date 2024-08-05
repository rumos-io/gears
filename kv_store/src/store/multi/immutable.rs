use database::{prefix::PrefixDB, Database};

use crate::{
    bank::multi::{ApplicationMultiBank, TransactionMultiBank},
    query::QueryMultiStore,
    store::kv::immutable::{KVStore, KVStoreBackend},
    StoreKey,
};

#[derive(Debug)]
pub(crate) enum MultiStoreBackend<'a, DB, SK> {
    App(&'a ApplicationMultiBank<DB, SK>),
    Tx(&'a TransactionMultiBank<DB, SK>),
    Query(&'a QueryMultiStore<DB, SK>),
}

#[derive(Debug)]
pub struct MultiStore<'a, DB, SK>(pub(crate) MultiStoreBackend<'a, DB, SK>);

impl<DB: Database, SK: StoreKey> MultiStore<'_, DB, SK> {
    pub fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        match self.0 {
            MultiStoreBackend::App(var) => KVStore(KVStoreBackend::App(var.kv_store(store_key))),
            MultiStoreBackend::Tx(var) => KVStore(KVStoreBackend::Tx(var.kv_store(store_key))),
            MultiStoreBackend::Query(var) => var.kv_store(store_key),
        }
    }

    pub fn head_version(&self) -> u32 {
        match self.0 {
            MultiStoreBackend::App(var) => var.head_version,
            MultiStoreBackend::Tx(var) => var.head_version,
            MultiStoreBackend::Query(var) => var.head_version(),
        }
    }

    pub fn head_commit_hash(&self) -> [u8; 32] {
        match self.0 {
            MultiStoreBackend::App(var) => var.head_commit_hash,
            MultiStoreBackend::Tx(var) => var.head_commit_hash,
            MultiStoreBackend::Query(var) => var.head_commit_hash(),
        }
    }
}

impl<'a, DB, SK> From<&'a QueryMultiStore<DB, SK>> for MultiStore<'a, DB, SK> {
    fn from(value: &'a QueryMultiStore<DB, SK>) -> Self {
        MultiStore(MultiStoreBackend::Query(value))
    }
}

impl<'a, DB, SK> From<&'a ApplicationMultiBank<DB, SK>> for MultiStore<'a, DB, SK> {
    fn from(value: &'a ApplicationMultiBank<DB, SK>) -> Self {
        MultiStore(MultiStoreBackend::App(value))
    }
}

impl<'a, DB, SK> From<&'a TransactionMultiBank<DB, SK>> for MultiStore<'a, DB, SK> {
    fn from(value: &'a TransactionMultiBank<DB, SK>) -> Self {
        MultiStore(MultiStoreBackend::Tx(value))
    }
}
