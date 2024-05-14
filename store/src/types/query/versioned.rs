use std::collections::HashMap;

use database::{Database, PrefixDB};

use crate::{
    error::KEY_EXISTS_MSG,
    types::kv::immutable::{KVStore, KVStoreBackend},
    QueryableMultiKVStore, StoreKey,
};

use super::kv::QueryKVStore;

#[derive(Debug)]
pub struct VersionedQueryMultiStore<'a, DB, SK>(
    pub(super) HashMap<SK, QueryKVStore<'a, PrefixDB<DB>>>,
);

impl<DB: Database, SK: StoreKey> QueryableMultiKVStore<PrefixDB<DB>, SK>
    for VersionedQueryMultiStore<'_, DB, SK>
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
