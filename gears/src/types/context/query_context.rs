use crate::error::AppError;
use database::{Database, PrefixDB};
use proto_messages::cosmos::ibc::types::core::host::identifiers::ChainId;
use store_crate::{
    types::{
        multi::MultiStore,
        query::{kv::QueryKVStore, multi::QueryMultiStore},
    },
    ReadMultiKVStore, StoreKey,
};

use super::QueryableContext;

pub struct QueryContext<'a, DB, SK> {
    pub multi_store: QueryMultiStore<'a, DB, SK>,
    pub height: u64,
    pub chain_id: ChainId,
}

impl<'a, DB: Database, SK: StoreKey> QueryContext<'a, DB, SK> {
    pub fn new(
        multi_store: &'a MultiStore<DB, SK>,
        version: u32,
        // chain_id: ChainId,
    ) -> Result<Self, AppError> {
        let multi_store = QueryMultiStore::new(multi_store, version)
            .map_err(|e| AppError::InvalidRequest(e.to_string()))?;
        Ok(QueryContext {
            multi_store,
            height: version as u64, // TODO:
            chain_id: ChainId::new("todo-900").expect("default should be valid"), // TODO:
        })
    }
}

impl<'a, DB: Database, SK: StoreKey> QueryableContext<PrefixDB<DB>, SK>
    for QueryContext<'a, DB, SK>
{
    type KVStore = QueryKVStore<'a, PrefixDB<DB>>;

    fn kv_store(&self, store_key: &SK) -> &Self::KVStore {
        self.multi_store.kv_store(store_key)
    }

    fn height(&self) -> u64 {
        self.height
    }

    fn chain_id(&self) -> &tendermint::informal::chain::Id {
        // &self.chain_id
        unimplemented!() // TODO:NOW
    }
}
