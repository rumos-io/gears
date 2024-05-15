use database::Database;
use std::{hash::Hash, marker::PhantomData};
use store_crate::{
    types::prefix::{immutable::ImmutablePrefixStore, mutable::MutablePrefixStore},
    QueryableKVStore, StoreKey, TransactionalKVStore,
};
use store_crate::{QueryableMultiKVStore, TransactionalMultiKVStore};
use strum::IntoEnumIterator;

pub trait ParamsSubspaceKey: Hash + Eq + IntoEnumIterator + Clone + Send + Sync + 'static {
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct Keeper<SK, PSK> {
    store_key: SK,
    p: PhantomData<PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Keeper<SK, PSK> {
    pub fn new(store_key: SK) -> Self {
        Keeper {
            store_key,
            p: PhantomData,
        }
    }

    pub fn raw_subspace<'a, DB: Database, KV: QueryableMultiKVStore<DB, SK>>(
        &self,
        store: &'a KV,
        params_subspace_key: &PSK,
    ) -> ImmutablePrefixStore<'a, DB> {
        let store = store.kv_store(&self.store_key);
        store.prefix_store(params_subspace_key.name().as_bytes().to_vec())
    }

    pub fn raw_subspace_mut<'a, DB: Database, KV: TransactionalMultiKVStore<DB, SK>>(
        &self,
        store: &'a mut KV,
        params_subspace_key: &PSK,
    ) -> MutablePrefixStore<'a, DB> {
        let params_store = store.kv_store_mut(&self.store_key);
        params_store.prefix_store_mut(params_subspace_key.name().as_bytes().to_vec())
    }
}
