use std::collections::HashMap;

use database::{Database, PrefixDB};

use crate::{
    error::KEY_EXISTS_MSG,
    types::kv_2::immutable::{KVStoreBackend, KVStoreV2},
    QueryableMultiKVStoreV2, StoreKey,
};

use super::kv::QueryKVStoreV2;

pub struct VersionedQueryMultiStore<'a, DB, SK>(
    pub(super) HashMap<&'a SK, QueryKVStoreV2<'a, PrefixDB<DB>>>,
);

impl<DB: Database, SK: StoreKey> QueryableMultiKVStoreV2<PrefixDB<DB>, SK>
    for VersionedQueryMultiStore<'_, DB, SK>
{
    fn kv_store(&self, store_key: &SK) -> KVStoreV2<'_, PrefixDB<DB>> {
        KVStoreV2(KVStoreBackend::Query(
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
