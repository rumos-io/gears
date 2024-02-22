use crate::error::AppError;
use database::{Database, PrefixDB};
use store_crate::{AnyKVStore, MultiStore, QueryKVStore, QueryMultiStore, StoreKey};

use super::context::Context;

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

pub enum ReadContext<'a, 'b, SK: StoreKey, DB: Database> {
    QueryContext(&'a QueryContext<'a, DB, SK>),
    Context(&'a Context<'a, 'b, DB, SK>),
}

impl<'a, SK: StoreKey, DB: Database> From<&'a QueryContext<'a, DB, SK>>
    for ReadContext<'a, '_, SK, DB>
{
    fn from(ctx: &'a QueryContext<'a, DB, SK>) -> Self {
        ReadContext::QueryContext(ctx)
    }
}

impl<'a, 'b, SK: StoreKey, DB: Database> From<&'a Context<'a, 'b, DB, SK>>
    for ReadContext<'a, 'b, SK, DB>
{
    fn from(ctx: &'a Context<'a, 'b, DB, SK>) -> Self {
        ReadContext::Context(ctx)
    }
}

impl<SK: StoreKey, DB: Database> ReadContext<'_, '_, SK, DB> {
    pub fn get_kv_store(&self, store_key: &SK) -> AnyKVStore<'_, PrefixDB<impl Database>> {
        match self {
            ReadContext::QueryContext(ctx) => AnyKVStore::QueryKVStore(ctx.get_kv_store(store_key)),
            ReadContext::Context(ctx) => AnyKVStore::KVStore(ctx.get_kv_store(store_key)),
        }
    }
}
