use crate::error::AppError;
use database::{Database, PrefixDB};
use proto_messages::cosmos::ibc::types::core::host::identifiers::ChainId;
use store_crate::{MultiStore, QueryKVStore, QueryMultiStore, StoreKey};

pub struct QueryContext<'a, DB, SK> {
    pub multi_store: QueryMultiStore<'a, DB, SK>,
    //_height: u64,
}

impl<'a, DB: Database, SK: StoreKey> QueryContext<'a, DB, SK> {
    pub fn new(multi_store: &'a MultiStore<DB, SK>, version: u32) -> Result<Self, AppError> {
        let multi_store = QueryMultiStore::new(multi_store, version)
            .map_err(|e| AppError::InvalidRequest(e.to_string()))?;
        Ok(QueryContext {
            multi_store,
            //_height: height,
        })
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: &SK) -> &QueryKVStore<'_, PrefixDB<DB>> {
        return self.multi_store.get_kv_store(store_key);
    }

    pub fn height(&self) -> u64 {
        unimplemented!()
    }

    pub fn chain_id(&self) -> &ChainId {
        unimplemented!()
    }
}
