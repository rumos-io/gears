use gears::{
    context::init::InitContext,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
};

use crate::{genesis::GovGenesisState, params::GovParamsKeeper};

// const PROPOSAL_ID_KEY: [u8; 1] = [0x03];

#[allow(dead_code)]
pub struct GovKeeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    store_key: SK,
    gov_params_keeper: GovParamsKeeper<PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> GovKeeper<SK, PSK> {
    pub fn new(store_key: SK, params_subspace_key: PSK) -> Self {
        Self {
            store_key,
            gov_params_keeper: GovParamsKeeper {
                params_subspace_key,
            },
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        _ctx: &mut InitContext<'_, DB, SK>,
        _genesis: GovGenesisState,
    ) {
    }
}
