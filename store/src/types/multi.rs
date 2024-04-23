use std::{collections::HashMap, sync::Arc};

use database::{Database, PrefixDB};

use crate::{
    error::KEY_EXISTS_MSG, hash::StoreInfo, QueryableMultiKVStore, StoreKey, TransactionalKVStore,
    TransactionalMultiKVStore,
};

use super::kv::KVStore;

#[derive(Debug)]
pub struct MultiStore<DB, SK> {
    pub(crate) head_version: u32,
    pub(crate) head_commit_hash: [u8; 32],
    pub(crate) stores: HashMap<SK, KVStore<PrefixDB<DB>>>,
}

impl<DB: Database, SK: StoreKey> MultiStore<DB, SK> {
    pub fn new(db: Arc<DB>) -> Self {
        // let db = Arc::new(db);
        let mut store_infos = vec![];
        let mut stores = HashMap::new();
        let mut head_version = 0;

        for store in SK::iter() {
            // TODO: check that store names are not prefixes
            let prefix = store.name().as_bytes().to_vec();
            let kv_store = KVStore::new(PrefixDB::new(db.clone(), prefix), None).unwrap();

            let store_info = StoreInfo {
                name: store.name().into(),
                hash: kv_store.head_commit_hash(),
            };

            head_version = kv_store.last_committed_version();

            stores.insert(store, kv_store);
            store_infos.push(store_info)
        }

        MultiStore {
            head_version,
            head_commit_hash: crate::hash::hash_store_infos(store_infos),
            stores,
        }
    }
}

impl<DB: Database, SK: StoreKey> QueryableMultiKVStore<PrefixDB<DB>, SK> for MultiStore<DB, SK> {
    type KvStore = KVStore<PrefixDB<DB>>;

    fn kv_store(&self, store_key: &SK) -> &Self::KvStore {
        self.stores.get(store_key).expect(KEY_EXISTS_MSG)
    }

    fn head_version(&self) -> u32 {
        self.head_version
    }

    fn head_commit_hash(&self) -> [u8; 32] {
        self.head_commit_hash
    }
}

impl<DB: Database, SK: StoreKey> TransactionalMultiKVStore<PrefixDB<DB>, SK>
    for MultiStore<DB, SK>
{
    type KvStoreMut = KVStore<PrefixDB<DB>>;

    fn kv_store_mut(&mut self, store_key: &SK) -> &mut Self::KvStoreMut {
        self.stores.get_mut(store_key).expect(KEY_EXISTS_MSG)
    }

    fn commit(&mut self) -> [u8; 32] {
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

    fn tx_caches_write_then_clear(&mut self) {
        for (_, store) in &mut self.stores {
            store.write_then_clear_tx_cache();
        }
    }

    fn tx_caches_clear(&mut self) {
        for (_, store) in &mut self.stores {
            store.clear_tx_cache();
        }
    }
}
