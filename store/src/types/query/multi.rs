use std::collections::HashMap;

use trees::database::{Database, PrefixDB};

use crate::{
    error::{StoreError, KEY_EXISTS_MSG},
    types::multi::MultiStore,
    ReadMultiKVStore, StoreKey,
};

use super::kv::QueryKVStore;

pub struct QueryMultiStore<'a, DB, SK> {
    //head_version: u32,
    //head_commit_hash: [u8; 32],
    stores: HashMap<&'a SK, QueryKVStore<'a, PrefixDB<DB>>>,
}

impl<'a, DB: Database, SK: StoreKey> QueryMultiStore<'a, DB, SK> {
    pub fn new(multi_store: &'a MultiStore<DB, SK>, version: u32) -> Result<Self, StoreError> {
        let mut stores = HashMap::new();
        for (store, kv_store) in &multi_store.stores {
            stores.insert(store, QueryKVStore::new(kv_store, version)?);
        }

        Ok(Self {
            //head_version: version,
            //head_commit_hash: multi_store.head_commit_hash, //TODO: get the proper commit hash,
            stores,
        })
    }
}

impl<'a, DB: Database, SK: StoreKey> ReadMultiKVStore<PrefixDB<DB>, SK>
    for QueryMultiStore<'a, DB, SK>
{
    type KvStore = QueryKVStore<'a, PrefixDB<DB>>;

    fn kv_store(&self, store_key: &SK) -> &Self::KvStore {
        self.stores.get(store_key).expect(KEY_EXISTS_MSG)
    }

    fn head_version(&self) -> u32 {
        unimplemented!()
    }

    fn head_commit_hash(&self) -> [u8; 32] {
        unimplemented!()
    }
}
