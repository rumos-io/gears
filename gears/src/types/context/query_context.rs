use crate::error::AppError;
use database::{Database, PrefixDB};
use store_crate::{AnyKVStore, MultiStore, QueryKVStore, QueryMultiStore, StoreKey};

use super::{context::Context, init_context::InitContext};

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

    // pub fn _get_height(&self) -> u64 {
    //     self._height
    // }
}

pub trait ReadContext<SK, DB: Database> {
    fn get_kv_store(&self, store_key: &SK) -> AnyKVStore<'_, PrefixDB<DB>>;
}

impl<SK: StoreKey, DB: Database> ReadContext<SK, DB> for QueryContext<'_, DB, SK> {
    fn get_kv_store(&self, store_key: &SK) -> AnyKVStore<'_, PrefixDB<DB>> {
        AnyKVStore::QueryKVStore(self.multi_store.get_kv_store(store_key))
    }
}

impl<SK: StoreKey, DB: Database> ReadContext<SK, DB> for InitContext<'_, DB, SK> {
    fn get_kv_store(&self, store_key: &SK) -> AnyKVStore<'_, PrefixDB<DB>> {
        AnyKVStore::KVStore(self.multi_store.get_kv_store(store_key))
    }
}

impl<SK: StoreKey, DB: Database> ReadContext<SK, DB> for Context<'_, '_, DB, SK> {
    fn get_kv_store(&self, store_key: &SK) -> AnyKVStore<'_, PrefixDB<DB>> {
        AnyKVStore::KVStore(self.get_kv_store(store_key))
    }
}
