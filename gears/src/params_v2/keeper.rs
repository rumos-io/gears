use database::{prefix::PrefixDB, Database};
use store_crate::{QueryableKVStore, StoreKey, TransactionalKVStore};

use crate::types::context::{QueryableContext, TransactionalContext};

use super::{space::ParamsSpace, space_mut::ParamsSpaceMut, ParamsSubspaceKeyV2};

#[derive(Debug, Clone)]
pub struct ParamsKeeper<SK> {
    store_key: SK,
}

impl<SK> ParamsKeeper<SK> {
    pub fn new(store_key: SK) -> Self {
        Self { store_key }
    }
}

impl<SK: StoreKey> ParamsKeeper<SK> {
    pub fn subspace<'a, DB: Database, CTX: QueryableContext<DB, SK>, PSK: ParamsSubspaceKeyV2>(
        &self,
        ctx: &'a CTX,
        params_subspace_key: &PSK,
    ) -> ParamsSpace<'a, PrefixDB<DB>> {
        ParamsSpace {
            inner: ctx
                .kv_store(&self.store_key)
                .prefix_store(params_subspace_key.name().as_bytes().to_vec()),
        }
    }

    pub fn subspace_mut<
        'a,
        DB: Database,
        CTX: TransactionalContext<DB, SK>,
        PSK: ParamsSubspaceKeyV2,
    >(
        &self,
        ctx: &'a mut CTX,
        params_subspace_key: &PSK,
    ) -> ParamsSpaceMut<'a, PrefixDB<DB>> {
        ParamsSpaceMut {
            inner: ctx
                .kv_store_mut(&self.store_key)
                .prefix_store_mut(params_subspace_key.name().as_bytes().to_vec()),
        }
    }
}
