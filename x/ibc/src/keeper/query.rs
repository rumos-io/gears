use database::Database;
use gears::{types::context::query_context::QueryContext, x::params::ParamsSubspaceKey};
use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryClientStatesRequest;
use store::StoreKey;

use crate::params::AbciParamsKeeper;

#[derive(Debug, Clone)]
pub struct QueryKeeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    store_key: SK,
    params_keeper: AbciParamsKeeper<SK, PSK>,
    // auth_keeper: auth::Keeper<SK, PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> QueryKeeper<SK, PSK> {
    pub fn new(
        store_key: SK,
        params_keeper: gears::x::params::Keeper<SK, PSK>,
        params_subspace_key: PSK,
    ) -> Self {
        let abci_params_keeper = AbciParamsKeeper {
            params_keeper,
            params_subspace_key,
        };
        QueryKeeper {
            store_key,
            params_keeper: abci_params_keeper,
        }
    }

    pub fn client_params<DB: Database + Send + Sync>(
        &mut self,
        ctx: &mut QueryContext<'_, DB, SK>,
        query : QueryClientStatesRequest,
    ) {
    }

    pub fn client_state() {}

    pub fn client_states() {}

    pub fn client_status() {}

    pub fn consensus_state_height() {}

    pub fn consensus_state() {}

    pub fn consensus_states() {}
}
