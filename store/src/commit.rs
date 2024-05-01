use database::Database;

use crate::{hash::StoreInfo, types::multi::MultiStore, StoreKey};

pub trait CommitMultiStore {
    fn commit(&mut self) -> [u8; 32];
}

impl<DB: Database, SK: StoreKey> CommitMultiStore for MultiStore<DB, SK> {
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
}
