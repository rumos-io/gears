use std::{collections::HashMap, sync::Arc};

use database::{prefix::PrefixDB, Database};

use crate::{hash::StoreInfo, kv::application::ApplicationKVBank, StoreKey};

use super::*;

#[derive(Debug)]
pub struct ApplicationStore<DB, SK>(pub(crate) HashMap<SK, ApplicationKVBank<PrefixDB<DB>>>);

impl<SK, DB> MultiBankBackend<DB, SK> for ApplicationStore<DB, SK> {
    type Bank = ApplicationKVBank<PrefixDB<DB>>;

    fn stores(&self) -> &HashMap<SK, Self::Bank> {
        &self.0
    }

    fn stores_mut(&mut self) -> &mut HashMap<SK, Self::Bank> {
        &mut self.0
    }
}

impl<DB: Database, SK: StoreKey> MultiBank<DB, SK, ApplicationStore<DB, SK>> {
    pub fn new(db: DB) -> Self {
        let db = Arc::new(db);

        let mut store_infos = Vec::new();
        let mut stores = HashMap::new();
        let mut head_version = 0;

        for store in SK::iter() {
            let prefix = store.name().as_bytes().to_vec(); // TODO:NOW check that store names are not prefixes
            let kv_store =
                ApplicationKVBank::new(PrefixDB::new(Arc::clone(&db), prefix), None).unwrap();

            let store_info = StoreInfo {
                name: store.name().into(),
                hash: kv_store.persistent().root_hash(),
            };

            head_version = kv_store.persistent().loaded_version();

            stores.insert(store, kv_store);
            store_infos.push(store_info)
        }

        MultiBank {
            head_version,
            head_commit_hash: crate::hash::hash_store_infos(store_infos),
            backend: ApplicationStore(stores),
            _marker: PhantomData,
        }
    }

    // pub fn to_cache_kind(&self) -> MultiBank<DB, SK, TransactionStore> {
    //     MultiBank {
    //         head_version: self.head_version,
    //         head_commit_hash: self.head_commit_hash,
    //         stores: self
    //             .stores
    //             .iter()
    //             .map(|(sk, store)| (sk.to_owned(), store.to_cache_kind()))
    //             .collect(),
    //     }
    // }

    pub fn commit(&mut self) -> [u8; 32] {
        let mut store_infos = vec![];
        for (store, kv_store) in &mut self.backend.0 {
            let store_info = StoreInfo {
                name: store.name().into(),
                hash: kv_store.commit(),
            };

            store_infos.push(store_info)
        }

        let hash = crate::hash::hash_store_infos(store_infos);

        self.head_commit_hash = hash;
        self.head_version += 1; //TODO: wraps on overflow - should halt the chain (panic)
        hash
    }

    // pub fn sync(&mut self, data: CacheCommitList<SK>) {
    //     if data.is_empty() {
    //         return;
    //     }

    //     for (store_key, set, delete) in data.into_iter() {
    //         let store = self.kv_store_mut(&store_key);

    //         store.cache.storage.extend(set);
    //         store.cache.delete.extend(delete);
    //     }
    // }
}
