use std::{collections::HashMap, marker::PhantomData};

use application::ApplicationStore;
use database::Database;
use transaction::TransactionStore;

use crate::{error::KEY_EXISTS_MSG, StoreKey};

pub mod application;
pub mod transaction;
// pub mod immutable;
// pub mod mutable;

pub trait MultiBankBackend<DB, SK> {
    type Bank;

    fn stores(&self) -> &HashMap<SK, Self::Bank>;
    fn stores_mut(&mut self) -> &mut HashMap<SK, Self::Bank>;
}

pub type ApplicationMultiBank<DB, SK> = MultiBank<DB, SK, ApplicationStore<DB, SK>>;
pub type TransactionMultiBank<DB, SK> = MultiBank<DB, SK, TransactionStore<DB, SK>>;

/// Bank which stores all KVBanks
#[derive(Debug)]
pub struct MultiBank<DB, SK, SB> {
    pub(crate) head_version: u32,
    pub(crate) head_commit_hash: [u8; 32],
    pub(crate) backend: SB,
    _marker: PhantomData<(DB, SK)>,
}

impl<DB: Database, SK: StoreKey, SB: MultiBankBackend<DB, SK>> MultiBank<DB, SK, SB> {
    pub fn kv_store(&self, store_key: &SK) -> &SB::Bank {
        self.backend.stores().get(store_key).expect(KEY_EXISTS_MSG)
    }

    pub fn kv_store_mut(&mut self, store_key: &SK) -> &mut SB::Bank {
        self.backend
            .stores_mut()
            .get_mut(store_key)
            .expect(KEY_EXISTS_MSG)
    }

    pub fn head_version(&self) -> u32 {
        self.head_version
    }

    pub fn head_commit_hash(&self) -> [u8; 32] {
        self.head_commit_hash
    }

    // pub fn clear_block_cache(&mut self) {
    //     for store in self.backend.stores().values_mut() {
    //         store.clear_block_cache();
    //     }
    // }

    // pub fn clear_tx_cache(&mut self) {
    //     for store in self.stores.values_mut() {
    //         store.clear_tx_cache();
    //     }
    // }

    // pub fn caches_copy(&self) -> Vec<(SK, KVCache)> {
    //     let mut map: Vec<(SK, KVCache)> = Vec::with_capacity(self.stores.len());

    //     for (sk, store) in &self.stores {
    //         let cache = store.tx.clone();
    //         map.push((sk.to_owned(), cache));
    //     }

    //     map
    // }

    // pub fn caches_update(&mut self, cache: Vec<(SK, KVCache)>) {
    //     for (store_key, KVCache { storage, delete }) in cache {
    //         let store = self.kv_store_mut(&store_key);

    //         store.tx.storage.extend(storage);
    //         store.tx.delete.extend(delete);
    //     }
    // }

    // pub fn upgrade_cache(&mut self) {
    //     for store in self.stores.values_mut() {
    //         store.upgrade_cache();
    //     }
    // }
}
