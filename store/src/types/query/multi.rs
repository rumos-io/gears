use std::collections::HashMap;

use database::{Database, PrefixDB};

use crate::{
    error::{StoreError, KEY_EXISTS_MSG},
    types::kv_2::immutable::KVStoreV2,
    QueryableMultiKVStore, StoreKey,
};

use super::kv::QueryKVStore;

#[derive(Debug)]
pub struct QueryMultiStore<'a, DB, SK> {
    //head_version: u32,
    //head_commit_hash: [u8; 32],
    stores: HashMap<&'a SK, QueryKVStore<'a, PrefixDB<DB>>>,
}

impl<'a, DB: Database, SK: StoreKey> QueryMultiStore<'a, DB, SK> {
    // pub fn new(
    //     multi_store: &'a CommitMultiStore<DB, SK>,
    //     version: u32,
    // ) -> Result<Self, StoreError> {
    //     let mut stores = HashMap::new();
    //     for (store, kv_store) in &multi_store.stores {
    //         stores.insert(
    //             store,
    //             QueryKVStore::new(&kv_store.persistent_store, version)?,
    //         );
    //     }

    //     Ok(Self {
    //         //head_version: version,
    //         //head_commit_hash: multi_store.head_commit_hash, //TODO: get the proper commit hash,
    //         stores,
    //     })
    // }
}

impl<DB: Database, SK: StoreKey> QueryableMultiKVStore<PrefixDB<DB>, SK>
    for QueryMultiStore<'_, DB, SK>
{
    fn kv_store(&self, store_key: &SK) -> KVStoreV2<'_, PrefixDB<DB>> {
        // KVStore(KVStoreBackend::Query(
        //     self.stores.get(store_key).expect(KEY_EXISTS_MSG),
        // ))
        todo!()
    }

    fn head_version(&self) -> u32 {
        unimplemented!()
    }

    fn head_commit_hash(&self) -> [u8; 32] {
        unimplemented!()
    }
}
