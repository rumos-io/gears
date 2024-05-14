use std::collections::HashMap;

use database::{prefix::PrefixDB, Database};
use trees::iavl::QueryTree;

use crate::{
    error::{StoreError, KEY_EXISTS_MSG, POISONED_LOCK},
    CommitKind, QueryableMultiKVStore, StoreKey,
};

use self::kv::QueryKVStore;

use super::{
    kv::immutable::{KVStore, KVStoreBackend},
    multi::MultiBank,
};

pub mod kv;

#[derive(Debug)]
pub struct QueryMultiStore<DB, SK>(pub(crate) HashMap<SK, QueryKVStore<PrefixDB<DB>>>);

impl<DB: Database + Clone, SK: StoreKey> QueryMultiStore<DB, SK> {
    pub fn new(
        multi_store: &MultiBank<DB, SK, CommitKind>, // TODO:NOW OTHER TYPE WHICH HIDES THIS. OR TRAIT Into<SomeMyType>
        version: u32,
    ) -> Result<Self, StoreError> {
        let mut stores = HashMap::with_capacity(multi_store.stores.len());

        for (key, bank) in &multi_store.stores {
            let tree = bank.persistent.read().expect(POISONED_LOCK);

            let query_kv_store = QueryKVStore::new(QueryTree::new(&tree, version)?);

            stores.insert(key.to_owned(), query_kv_store);
        }

        Ok(Self(stores))
    }
}

impl<DB: Database, SK: StoreKey> QueryableMultiKVStore<PrefixDB<DB>, SK>
    for QueryMultiStore<DB, SK>
{
    fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        KVStore(KVStoreBackend::Query(
            self.0.get(store_key).expect(KEY_EXISTS_MSG),
        ))
    }

    fn head_version(&self) -> u32 {
        unimplemented!() // TODO:NOW
    }

    fn head_commit_hash(&self) -> [u8; 32] {
        unimplemented!() // TODO:NOW
    }
}
