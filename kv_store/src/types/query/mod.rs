use std::collections::HashMap;

use database::{prefix::PrefixDB, Database};
use trees::iavl::QueryTree;

use crate::{
    error::{KVStoreError, KEY_EXISTS_MSG, POISONED_LOCK},
    ApplicationStore, StoreKey,
};

use self::kv::QueryKVStore;

use super::{
    kv::immutable::{KVStore, KVStoreBackend},
    multi::MultiBank,
};

pub mod kv;

pub struct QueryStoreOptions<'a, DB, SK>(
    &'a HashMap<SK, super::kv::KVBank<PrefixDB<DB>, ApplicationStore>>,
    u32,
    [u8; 32],
);

impl<'a, DB, SK> From<&'a MultiBank<DB, SK, ApplicationStore>> for QueryStoreOptions<'a, DB, SK> {
    fn from(value: &'a MultiBank<DB, SK, ApplicationStore>) -> Self {
        Self(&value.stores, value.head_version, value.head_commit_hash)
    }
}

#[derive(Debug)]
pub struct QueryMultiStore<DB, SK> {
    pub(crate) head_version: u32,
    pub(crate) head_commit_hash: [u8; 32],
    pub(crate) inner: HashMap<SK, QueryKVStore<PrefixDB<DB>>>,
}

impl<DB: Database, SK: StoreKey> QueryMultiStore<DB, SK> {
    pub fn new<'a>(
        opt: impl Into<QueryStoreOptions<'a, DB, SK>>,
        version: u32,
    ) -> Result<Self, KVStoreError>
    where
        DB: 'a,
    {
        let QueryStoreOptions(inner, head_version, head_commit_hash) = opt.into();

        let mut stores = HashMap::with_capacity(inner.len());

        for (key, bank) in inner {
            let tree = bank.persistent.read().expect(POISONED_LOCK);

            let query_kv_store = QueryKVStore::new(QueryTree::new(&tree, version)?);

            stores.insert(key.to_owned(), query_kv_store);
        }

        Ok(Self {
            head_version,
            head_commit_hash,
            inner: stores,
        })
    }
}

impl<DB: Database, SK: StoreKey> QueryMultiStore<DB, SK> {
    pub fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        KVStore(KVStoreBackend::Query(
            self.inner.get(store_key).expect(KEY_EXISTS_MSG),
        ))
    }

    pub fn head_version(&self) -> u32 {
        self.head_version
    }

    pub fn head_commit_hash(&self) -> [u8; 32] {
        self.head_commit_hash
    }
}
