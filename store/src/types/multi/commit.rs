use std::{collections::HashMap, sync::Arc};

use database::{Database, PrefixDB};

use crate::{error::KEY_EXISTS_MSG, hash::StoreInfo, types::kv::commit::CommitKVStore, StoreKey};

use super::{mutable::MultiStoreMut, MultiStore};

/// MultiStore which stores all commitable KVStore and has right to commit changes too
#[derive(Debug)]
pub struct CommitMultiStore<DB, SK> {
    pub(crate) head_version: u32,
    pub(crate) head_commit_hash: [u8; 32],
    pub(crate) stores: HashMap<SK, CommitKVStore<PrefixDB<DB>>>,
}

impl<DB: Database, SK: StoreKey> CommitMultiStore<DB, SK> {
    pub fn new(db: DB) -> Self {
        let db = Arc::new(db);

        let mut store_infos = vec![];
        let mut stores = HashMap::new();
        let mut head_version = 0;

        for store in SK::iter() {
            // TODO: check that store names are not prefixes
            let prefix = store.name().as_bytes().to_vec();
            let kv_store =
                CommitKVStore::new(PrefixDB::new(Arc::clone(&db), prefix), None).unwrap();

            let store_info = StoreInfo {
                name: store.name().into(),
                hash: kv_store.head_commit_hash(),
            };

            head_version = kv_store.last_committed_version();

            stores.insert(store, kv_store);
            store_infos.push(store_info)
        }

        CommitMultiStore {
            head_version,
            head_commit_hash: crate::hash::hash_store_infos(store_infos),
            stores,
        }
    }

    pub fn commit(&mut self) -> [u8; 32] {
        let mut store_infos = vec![];
        for (store, kv_store) in &mut self.stores {
            let store_info = StoreInfo {
                name: store.name().into(),
                hash: kv_store.commit(),
            };

            store_infos.push(store_info)
        }

        let hash = crate::hash::hash_store_infos(store_infos);

        self.head_commit_hash = hash;
        self.head_version += 1;
        hash
    }

    pub fn kv_store(&self, store_key: &SK) -> &CommitKVStore<PrefixDB<DB>> {
        self.stores.get(store_key).expect(KEY_EXISTS_MSG)
    }

    pub fn head_version(&self) -> u32 {
        self.head_version
    }

    pub fn head_commit_hash(&self) -> [u8; 32] {
        self.head_commit_hash
    }

    pub fn kv_store_mut(&mut self, store_key: &SK) -> &mut CommitKVStore<PrefixDB<DB>> {
        self.stores.get_mut(store_key).expect(KEY_EXISTS_MSG)
    }

    /// Upgrade cache of TX to block in all stores
    pub fn tx_cache_to_block(&mut self) {
        for (_, store) in &mut self.stores {
            store.cache.tx_upgrade_to_block();
        }
    }

    /// Clear TX cache in all stores
    pub fn tx_caches_clear(&mut self) {
        for (_, store) in &mut self.stores {
            store.cache.tx.clear();
        }
    }

    pub fn to_immutable(&self) -> MultiStore<'_, DB, SK> {
        MultiStore::from(self)
    }

    pub fn to_mutable(&mut self) -> MultiStoreMut<'_, DB, SK> {
        MultiStoreMut::from(self)
    }
}
