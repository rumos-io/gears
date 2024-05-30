use database::{prefix::PrefixDB, Database};
use kv_store::{QueryableKVStore, StoreKey, TransactionalKVStore};
use space::GasParamsSpace;
use space_mut::GasParamsSpaceMut;

use crate::context::{ImmutableGasContext, MutableGasContext};

use super::ParamsSubspaceKey;

pub mod space;
pub mod space_mut;

pub fn subspace<
    'a,
    DB: Database,
    SK: StoreKey,
    CTX: ImmutableGasContext<DB, SK>,
    PSK: ParamsSubspaceKey,
>(
    ctx: &'a CTX,
    params_subspace_key: &PSK,
) -> GasParamsSpace<'a, PrefixDB<DB>> {
    GasParamsSpace {
        inner: ctx
            .kv_store(SK::params())
            .prefix_store(params_subspace_key.name().as_bytes().to_vec()),
    }
}

pub fn subspace_mut<
    'a,
    DB: Database,
    SK: StoreKey,
    CTX: MutableGasContext<DB, SK>,
    PSK: ParamsSubspaceKey,
>(
    ctx: &'a mut CTX,
    params_subspace_key: &PSK,
) -> GasParamsSpaceMut<'a, PrefixDB<DB>> {
    GasParamsSpaceMut {
        inner: ctx
            .kv_store_mut(SK::params())
            .prefix_store_mut(params_subspace_key.name().as_bytes().to_vec()),
    }
}
