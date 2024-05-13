pub mod cache;
use std::collections::HashMap;

use crate::{error::KEY_EXISTS_MSG, StoreKey};
use database::{Database, PrefixDB};

use super::kv_2::KVStorage;

pub mod commit;
pub mod immutable;
pub mod mutable;

pub struct MultiStorage<DB, SK, ST> {
    pub(crate) head_version: u32,
    pub(crate) head_commit_hash: [u8; 32],
    pub(crate) stores: HashMap<SK, KVStorage<PrefixDB<DB>, ST>>,
}

impl<DB: Database, SK: StoreKey, ST> MultiStorage<DB, SK, ST> {
    pub fn kv_store(&self, store_key: &SK) -> &KVStorage<PrefixDB<DB>, ST> {
        self.stores.get(store_key).expect(KEY_EXISTS_MSG)
    }

    pub fn kv_store_mut(&mut self, store_key: &SK) -> &mut KVStorage<PrefixDB<DB>, ST> {
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
}
