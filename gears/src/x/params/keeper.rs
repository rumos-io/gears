use database::{Database, PrefixDB};
use store_crate::{kv_store_key::SimpleKvStoreKey, StoreKey};

use crate::types::context::{context::Context, read_context::ReadContext};
use std::{hash::Hash, marker::PhantomData};
use strum::IntoEnumIterator;

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

    pub fn get_raw_subspace<'a, DB: Database>(
        &'a self,
        ctx: &'a impl ReadContext<SK, DB>,
        params_subspace_key: &PSK,
    ) -> store_crate::ImmutablePrefixStore<'_, PrefixDB<DB>> {
        let store = ctx.get_kv_store(&self.store_key);
        match store {
            store_crate::AnyKVStore::KVStore(store) => store.get_immutable_prefix_store(
                SimpleKvStoreKey::try_from(params_subspace_key.name().as_bytes().to_vec())
                    .expect("Params subspace key name shouldn't be empty "),
            ),
            store_crate::AnyKVStore::QueryKVStore(store) => store.get_immutable_prefix_store(
                SimpleKvStoreKey::try_from(params_subspace_key.name().as_bytes().to_vec())
                    .expect("Params subspace key name shouldn't be empty "),
            ),
        }
    }

    pub fn get_mutable_raw_subspace<'a, DB: Database>(
        &self,
        ctx: &'a mut Context<'_, '_, DB, SK>,
        params_subspace_key: &PSK,
    ) -> store_crate::MutablePrefixStore<'a, PrefixDB<DB>> {
        let params_store = ctx.get_mutable_kv_store(&self.store_key);

        params_store.get_mutable_prefix_store(
            SimpleKvStoreKey::try_from(params_subspace_key.name().as_bytes().to_vec())
                .expect("Params subspace key name shouldn't be empty "),
        )
    }
}
