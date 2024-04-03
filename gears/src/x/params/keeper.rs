use database::{Database, PrefixDB};
use std::{hash::Hash, marker::PhantomData};
use store_crate::{
    types::prefix::{immutable::ImmutablePrefixStore, mutable::MutablePrefixStore},
    QueryableKVStore, StoreKey, TransactionalKVStore,
};
use strum::IntoEnumIterator;

use crate::types::context::{QueryableContext, TransactionalContext};

pub trait ParamsSubspaceKey: Hash + Eq + IntoEnumIterator + Clone + Send + Sync + 'static {
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
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

    pub fn raw_subspace<'a, DB: Database, CTX: QueryableContext<PrefixDB<DB>, SK>>(
        &self,
        ctx: &'a CTX,
        params_subspace_key: &PSK,
    ) -> ImmutablePrefixStore<'a, PrefixDB<DB>> {
        let store = ctx.kv_store(&self.store_key);

        store.prefix_store(params_subspace_key.name().as_bytes().to_vec())
    }

    pub fn raw_subspace_mut<'a, DB: Database, CTX: TransactionalContext<PrefixDB<DB>, SK>>(
        &self,
        ctx: &'a mut CTX,
        params_subspace_key: &PSK,
    ) -> MutablePrefixStore<'a, PrefixDB<DB>> {
        let params_store = ctx.kv_store_mut(&self.store_key);
        params_store.prefix_store_mut(params_subspace_key.name().as_bytes().to_vec())
    }
}
