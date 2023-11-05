use crate::error::AppError;
use database::{Database, PrefixDB};
use store_crate::{MultiStore, QueryKVStore, QueryMultiStore, StoreKey};

pub struct QueryContext<'a, T: Database, SK: StoreKey> {
    pub multi_store: QueryMultiStore<'a, T, SK>,
    //_height: u64,
}

impl<'a, T: Database, SK: StoreKey> QueryContext<'a, T, SK> {
    pub fn new(multi_store: &'a MultiStore<T, SK>, version: u32) -> Result<Self, AppError> {
        let multi_store = QueryMultiStore::new(multi_store, version)
            .map_err(|e| AppError::InvalidRequest(e.to_string()))?;
        Ok(QueryContext {
            multi_store,
            //_height: height,
        })
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: &SK) -> &QueryKVStore<'_, PrefixDB<T>> {
        return self.multi_store.get_kv_store(store_key);
    }

    // pub fn _get_height(&self) -> u64 {
    //     self._height
    // }
}
