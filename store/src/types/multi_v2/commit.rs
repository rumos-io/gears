use std::{collections::HashMap, sync::Arc};

use database::{Database, PrefixDB};

use crate::{hash::StoreInfo, types::kv_2::KVStorage, CacheKind, CommitKind, StoreKey};

use super::{cache::CacheCommitData, MultiStorage};

impl<DB: Database, SK: StoreKey> MultiStorage<DB, SK, CommitKind> {
    pub fn new(db: DB) -> Self {
        let db = Arc::new(db);

        let mut store_infos = Vec::new();
        let mut stores = HashMap::new();
        let mut head_version = 0;

        for store in SK::iter() {
            // TODO: check that store names are not prefixes
            let prefix = store.name().as_bytes().to_vec();
            let kv_store = KVStorage::new(PrefixDB::new(Arc::clone(&db), prefix), None).unwrap();

            let store_info = StoreInfo {
                name: store.name().into(),
                hash: kv_store.head_commit_hash(),
            };

            head_version = kv_store.last_committed_version();

            stores.insert(store, kv_store);
            store_infos.push(store_info)
        }

        MultiStorage {
            head_version,
            head_commit_hash: crate::hash::hash_store_infos(store_infos),
            stores,
        }
    }

    pub fn to_cache_kind(&self) -> MultiStorage<DB, SK, CacheKind> {
        MultiStorage {
            head_version: self.head_version,
            head_commit_hash: self.head_commit_hash,
            stores: self
                .stores
                .iter()
                .map(|(sk, store)| (sk.to_owned(), store.to_cache_kind()))
                .collect(),
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

    pub fn sync(&mut self, data: CacheCommitData<SK>) {
        for (store_key, set, delete) in data.into_iter() {
            let store = self.kv_store_mut(&store_key);
            for (key, value) in set {
                store.set(key, value);
            }

            for key in delete {
                store.delete(&key);
            }
        }
    }
}
