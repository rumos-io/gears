use store_crate::database::{Database, PrefixDB};
use store_crate::QueryableMultiKVStore;
use store_crate::{
    error::StoreError,
    types::{
        multi::MultiStore,
        query::{kv::QueryKVStore, multi::QueryMultiStore},
    },
    StoreKey,
};
use tendermint::types::chain_id::ChainId;

use super::{QueryKVContext, QueryableContext};

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
    ) -> Result<Self, StoreError> {
        let multi_store = QueryMultiStore::new(multi_store, version)?;
        Ok(QueryContext {
            multi_store,
            height: version as u64, // TODO:
            chain_id: ChainId::new("todo-900").expect("default should be valid"),
        })
    }
}

impl<'a, DB: Database, SK: StoreKey> QueryableContext<PrefixDB<DB>, SK>
    for QueryContext<'a, DB, SK>
{
    fn height(&self) -> u64 {
        self.height
    }

    fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }
}

impl<DB: Database, SK: StoreKey> QueryKVContext<PrefixDB<DB>, SK> for QueryContext<'_, DB, SK> {
    fn query_store(&self, store_key: &SK) -> &QueryKVStore<PrefixDB<DB>> {
        self.multi_store.kv_store(store_key)
    }
}
