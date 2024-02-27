use database::{Database, PrefixDB};
use store_crate::{AnyKVStore, StoreKey};

use super::{
    context::Context, init_context::InitContext, query_context::QueryContext, tx_context::TxContext,
};

pub trait ReadContext<SK, DB: Database> {
    fn get_kv_store(&self, store_key: &SK) -> AnyKVStore<'_, PrefixDB<DB>>;
}

impl<SK: StoreKey, DB: Database> ReadContext<SK, DB> for QueryContext<'_, DB, SK> {
    fn get_kv_store(&self, store_key: &SK) -> AnyKVStore<'_, PrefixDB<DB>> {
        AnyKVStore::QueryKVStore(self.get_kv_store(store_key))
    }
}

impl<SK: StoreKey, DB: Database> ReadContext<SK, DB> for InitContext<'_, DB, SK> {
    fn get_kv_store(&self, store_key: &SK) -> AnyKVStore<'_, PrefixDB<DB>> {
        AnyKVStore::KVStore(self.get_kv_store(store_key))
    }
}

impl<SK: StoreKey, DB: Database> ReadContext<SK, DB> for TxContext<'_, DB, SK> {
    fn get_kv_store(&self, store_key: &SK) -> AnyKVStore<'_, PrefixDB<DB>> {
        AnyKVStore::KVStore(self.get_kv_store(store_key))
    }
}

impl<SK: StoreKey, DB: Database> ReadContext<SK, DB> for Context<'_, '_, DB, SK> {
    fn get_kv_store(&self, store_key: &SK) -> AnyKVStore<'_, PrefixDB<DB>> {
        AnyKVStore::KVStore(self.get_kv_store(store_key))
    }
}
