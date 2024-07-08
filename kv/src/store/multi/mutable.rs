use database::{prefix::PrefixDB, Database};

use crate::{
    bank::multi::{ApplicationMultiBank, TransactionMultiBank},
    store::kv::{
        immutable::{KVStore, KVStoreBackend},
        mutable::{KVStoreBackendMut, KVStoreMut},
    },
    StoreKey,
};

use super::immutable::{MultiStore, MultiStoreBackend};

#[derive(Debug)]
pub(crate) enum MultiStoreBackendMut<'a, DB, SK> {
    App(&'a mut ApplicationMultiBank<DB, SK>),
    Tx(&'a mut TransactionMultiBank<DB, SK>),
}

#[derive(Debug)]
pub struct MultiStoreMut<'a, DB, SK>(pub(crate) MultiStoreBackendMut<'a, DB, SK>);

impl<DB, SK> MultiStoreMut<'_, DB, SK> {
    pub fn to_immutable(&self) -> MultiStore<'_, DB, SK> {
        match &self.0 {
            MultiStoreBackendMut::App(var) => MultiStore(MultiStoreBackend::App(var)),
            MultiStoreBackendMut::Tx(var) => MultiStore(MultiStoreBackend::Tx(var)),
        }
    }
}

impl<DB: Database, SK: StoreKey> MultiStoreMut<'_, DB, SK> {
    pub fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        match &self.0 {
            MultiStoreBackendMut::App(var) => KVStore(KVStoreBackend::App(var.kv_store(store_key))),
            MultiStoreBackendMut::Tx(var) => KVStore(KVStoreBackend::Tx(var.kv_store(store_key))),
        }
    }

    pub fn head_version(&self) -> u32 {
        match &self.0 {
            MultiStoreBackendMut::App(var) => var.head_version,
            MultiStoreBackendMut::Tx(var) => var.head_version,
        }
    }

    pub fn head_commit_hash(&self) -> [u8; 32] {
        match &self.0 {
            MultiStoreBackendMut::App(var) => var.head_commit_hash,
            MultiStoreBackendMut::Tx(var) => var.head_commit_hash,
        }
    }

    pub fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        match &mut self.0 {
            MultiStoreBackendMut::App(var) => {
                KVStoreMut(KVStoreBackendMut::App(var.kv_store_mut(store_key)))
            }
            MultiStoreBackendMut::Tx(var) => {
                KVStoreMut(KVStoreBackendMut::Tx(var.kv_store_mut(store_key)))
            }
        }
    }

    // pub fn clear_tx_cache(&mut self) {
    //     match &mut self.0 {
    //         MultiStoreBackendMut::App(var) => var.clear_tx_cache(),
    //         MultiStoreBackendMut::Tx(var) => var.clear_tx_cache(),
    //     }
    // }

    // pub fn upgrade_cache(&mut self) {
    //     match &mut self.0 {
    //         MultiStoreBackendMut::App(var) => var.upgrade_cache(),
    //         MultiStoreBackendMut::Tx(var) => var.upgrade_cache(),
    //     }
    // }
}

impl<'a, DB, SK> From<&'a mut ApplicationMultiBank<DB, SK>> for MultiStoreMut<'a, DB, SK> {
    fn from(value: &'a mut ApplicationMultiBank<DB, SK>) -> Self {
        MultiStoreMut(MultiStoreBackendMut::App(value))
    }
}

impl<'a, DB, SK> From<&'a mut TransactionMultiBank<DB, SK>> for MultiStoreMut<'a, DB, SK> {
    fn from(value: &'a mut TransactionMultiBank<DB, SK>) -> Self {
        MultiStoreMut(MultiStoreBackendMut::Tx(value))
    }
}
