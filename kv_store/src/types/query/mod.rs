use std::collections::HashMap;

use database::{prefix::PrefixDB, Database};
use trees::iavl::QueryTree;

use crate::{
    error::{KVStoreError, KEY_EXISTS_MSG, POISONED_LOCK},
    ApplicationStore, QueryableMultiKVStore, StoreKey,
};

use self::kv::QueryKVStore;

use super::{
    kv::immutable::{KVStore, KVStoreBackend},
    multi::MultiBank,
};

pub mod kv;

pub struct QueryStoreOptions<'a, DB, SK>(
    &'a HashMap<SK, super::kv::KVBank<PrefixDB<DB>, ApplicationStore>>,
);

impl<'a, DB, SK> From<&'a MultiBank<DB, SK, ApplicationStore>> for QueryStoreOptions<'a, DB, SK> {
    fn from(value: &'a MultiBank<DB, SK, ApplicationStore>) -> Self {
        Self(&value.stores)
    }
}

#[derive(Debug)]
pub struct QueryMultiStore<DB, SK>(pub(crate) HashMap<SK, QueryKVStore<PrefixDB<DB>>>);

impl<DB: Database + Clone, SK: StoreKey> QueryMultiStore<DB, SK> {
    pub fn new<'a>(
        opt: impl Into<QueryStoreOptions<'a, DB, SK>>,
        version: u32,
    ) -> Result<Self, KVStoreError>
    where
        DB: 'a,
    {
        let opt = opt.into();

        let mut stores = HashMap::with_capacity(opt.0.len());

        for (key, bank) in opt.0 {
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
