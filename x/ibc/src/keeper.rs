use gears::{
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    types::context::init_context::InitContext,
};

use crate::{
    ics02_client::Keeper as ClientKeeper, params::IBCParamsKeeper, types::genesis::GenesisState,
};

#[derive(Debug, Clone)]
pub struct Keeper<SK, PSK> {
    _store_key: SK,
    _ibc_params_keeper: IBCParamsKeeper<SK, PSK>,
    client_keeper: ClientKeeper<SK, PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Keeper<SK, PSK> {
    pub fn new(
        store_key: SK,
        params_keeper: gears::params::Keeper<SK, PSK>,
        params_subspace_key: PSK,
    ) -> Self {
        let ibc_params_keeper = IBCParamsKeeper {
            params_keeper: params_keeper.clone(),
            params_subspace_key: params_subspace_key.clone(),
        };
        Self {
            _store_key: store_key.clone(),
            _ibc_params_keeper: ibc_params_keeper,
            client_keeper: ClientKeeper::new(store_key, params_keeper, params_subspace_key),
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) {
        self.client_keeper.init_genesis(ctx, genesis.client_genesis);
    }
}
