use gears::{
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    types::context::init_context::InitContext,
};

use super::{params::ClientParamsKeeper, GenesisState};

#[derive(Debug, Clone)]
pub struct Keeper<SK, PSK> {
    _store_key: SK,
    client_params_keeper: ClientParamsKeeper<SK, PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Keeper<SK, PSK> {
    pub fn new(
        store_key: SK,
        params_keeper: gears::params::Keeper<SK, PSK>,
        params_subspace_key: PSK,
    ) -> Self {
        let client_params_keeper = ClientParamsKeeper {
            params_keeper,
            params_subspace_key,
        };
        Self {
            _store_key: store_key,
            client_params_keeper,
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) {
        self.client_params_keeper.set(ctx, genesis.params.clone());
    }
}
