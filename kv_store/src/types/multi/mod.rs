pub mod cache;
use std::collections::HashMap;

use crate::{error::KEY_EXISTS_MSG, StoreKey};
use database::{prefix::PrefixDB, Database};

use super::kv::{store_cache::{CacheCommitList, KVCache}, KVBank};

pub mod commit;
pub mod immutable;
pub mod mutable;

/// Bank which stores all KVBanks
#[derive(Debug, Clone)]
pub struct MultiBank<DB, SK, ST> {
    pub(crate) head_version: u32,
    pub(crate) head_commit_hash: [u8; 32],
    pub(crate) stores: HashMap<SK, KVBank<PrefixDB<DB>, ST>>,
}

impl<DB: Database, SK: StoreKey, ST> MultiBank<DB, SK, ST> {
    pub fn kv_store(&self, store_key: &SK) -> &KVBank<PrefixDB<DB>, ST> {
        self.stores.get(store_key).expect(KEY_EXISTS_MSG)
    }

    pub fn kv_store_mut(&mut self, store_key: &SK) -> &mut KVBank<PrefixDB<DB>, ST> {
        self.stores.get_mut(store_key).expect(KEY_EXISTS_MSG)
    }

    pub fn head_version(&self) -> u32 {
        self.head_version
    }

    pub fn head_commit_hash(&self) -> [u8; 32] {
        self.head_commit_hash
    }

    pub fn caches_clear(&mut self) {
        for (_, store) in &mut self.stores {
            store.clear_cache();
        }
    }

    pub fn caches_copy(&self) -> Vec<(SK, KVCache)> {
        let mut map: Vec<(SK, KVCache)> = Vec::with_capacity(self.stores.len());

        for (sk, store) in &self.stores {
            let cache = store.cache.clone();
            map.push((sk.to_owned(), cache));
        }

        map
    }

    pub fn caches_update(&mut self, cache: Vec<(SK, KVCache)>) {
        for (store_key, KVCache { storage, delete }) in cache {
            let store = self.kv_store_mut(&store_key);

            store.cache.storage.extend(storage);
            store.cache.delete.extend(delete);
        }
    }

    pub fn sync(&mut self, data: CacheCommitList<SK>) {
        if data.is_empty() {
            return;
        }

        for (store_key, set, delete) in data.into_iter() {
            let store = self.kv_store_mut(&store_key);

            store.cache.storage.extend(set);
            store.cache.delete.extend(delete);
        }
    }
}
