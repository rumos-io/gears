use std::str::FromStr;

use bytes::Bytes;
use gears::store::database::Database;
use gears::types::context::query_context::QueryContext;
use gears::types::context::QueryableContext;
use gears::x::params::ParamsSubspaceKey;
use prost::Message;
use proto_messages::cosmos::ibc::types::core::client::context::client_state::ClientStateCommon;
use proto_messages::cosmos::ibc::types::core::client::context::types::Status;
use proto_messages::{
    any::PrimitiveAny,
    cosmos::ibc::{
        protobuf::Protobuf,
        query::response::{
            QueryClientParamsResponse, QueryClientStateResponse, QueryClientStatesResponse,
            QueryClientStatusResponse, QueryConsensusStateHeightsResponse,
            QueryConsensusStateResponse, QueryConsensusStatesResponse, RawQueryClientStateResponse,
        },
        types::core::{
            client::{
                context::types::proto::v1::{
                    QueryClientStateRequest, QueryClientStatesRequest, QueryClientStatusRequest,
                    QueryConsensusStateHeightsRequest, QueryConsensusStateRequest,
                    QueryConsensusStatesRequest,
                },
                proto::{IdentifiedClientState, RawIdentifiedClientState},
                types::{
                    ConsensusStateWithHeight, Height, ProtoHeight, RawConsensusStateWithHeight,
                },
            },
            host::identifiers::ClientId,
        },
    },
};
use store::types::prefix::immutable::ImmutablePrefixStore;
use store::{QueryableKVStore, StoreKey};

use crate::errors::query::client::{
    ConsensusStateError, ConsensusStateHeightError, ConsensusStatesError, ParamsError, StateError,
    StatesError, StatusError,
};
use crate::keeper::{params_get, KEY_CLIENT_STORE_PREFIX, KEY_CONSENSUS_STATE_PREFIX};
use crate::params::AbciParamsKeeper;

use super::{client_consensus_state, client_state_get};

#[derive(Debug, Clone)]
pub struct QueryKeeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    store_key: SK,
    params_keeper: AbciParamsKeeper<SK, PSK>,
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
        Self {
            store_key,
            params_keeper: abci_params_keeper,
        }
    }

    pub fn client_params<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
    ) -> Result<QueryClientParamsResponse, ParamsError> {
        let params = params_get(&self.params_keeper, ctx)?;

        let response = QueryClientParamsResponse { params };

        Ok(response)
    }

    pub fn client_state<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        QueryClientStateRequest { client_id }: QueryClientStateRequest,
    ) -> Result<QueryClientStateResponse, StateError> {
        let client_id = ClientId::from_str(&client_id)?;

        let client_state = client_state_get(&self.store_key, ctx, &client_id)?;
        let revision_number = ctx.chain_id().revision_number();

        let response = RawQueryClientStateResponse {
            client_state: Some(client_state.into()),
            proof: Vec::new(), // TODO: ?
            proof_height: Some(ProtoHeight {
                revision_number,
                revision_height: ctx.height(),
            }),
        };

        Ok(response.try_into()?)
    }

    pub fn client_states<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        QueryClientStatesRequest { pagination: _ }: QueryClientStatesRequest,
    ) -> Result<QueryClientStatesResponse, StatesError> {
        let any_store = ctx.kv_store(&self.store_key);
        let store: ImmutablePrefixStore<'_, database::PrefixDB<DB>> =
            any_store.prefix_store(KEY_CLIENT_STORE_PREFIX.to_owned().into_bytes());

        let mut states = Vec::<IdentifiedClientState>::new();
        for (_key, value) in store.range(..) {
            states.push(
                RawIdentifiedClientState::decode::<Bytes>(value.into())?
                    .try_into()
                    .map_err(|e: std::convert::Infallible| StatesError::Custom(e.to_string()))?,
            );
        }

        let response = QueryClientStatesResponse {
            client_states: states,
            pagination: None,
        };

        Ok(response)
    }

    pub fn client_status<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        QueryClientStatusRequest { client_id }: QueryClientStatusRequest,
    ) -> Result<QueryClientStatusResponse, StatusError> {
        let client_id = ClientId::from_str(&client_id)?;
        let client_state = client_state_get(&self.store_key, ctx, &client_id)?;
        let client_type = client_state.client_type();

        let params = params_get(&self.params_keeper, ctx)?;

        let status = if !params.is_client_allowed(&client_type) {
            Status::Unauthorized
        } else {
            // TODO
            // client_state.status(&ContextShim::new(ctx.into(), self.store_key.clone()), &client_id)?
            Status::Unauthorized
        };

        let response = QueryClientStatusResponse {
            status: status.to_string(),
        };

        Ok(response)
    }

    pub fn consensus_state_heights<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        QueryConsensusStateHeightsRequest {
            client_id,
            pagination: _,
        }: QueryConsensusStateHeightsRequest,
    ) -> Result<QueryConsensusStateHeightsResponse, ConsensusStateHeightError> {
        let client_id = ClientId::from_str(&client_id)?;
        let store = ctx.kv_store(&self.store_key).prefix_store(
            format!("{KEY_CLIENT_STORE_PREFIX}/{client_id}/{KEY_CONSENSUS_STATE_PREFIX}")
                .into_bytes(),
        );

        let mut heights = Vec::<Height>::new();
        for (_key, value) in store.range(..) {
            heights.push(
                Height::decode_vec(&value)
                    .map_err(|e| ConsensusStateHeightError::Decode(e.to_string()))?,
            );
        }

        let response = QueryConsensusStateHeightsResponse {
            consensus_state_heights: heights,
            pagination: None,
        };

        Ok(response)
    }

    pub fn consensus_state<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        QueryConsensusStateRequest {
            client_id,
            revision_number,
            revision_height,
            latest_height,
        }: QueryConsensusStateRequest,
    ) -> Result<QueryConsensusStateResponse, ConsensusStateError> {
        let client_id = ClientId::from_str(&client_id)?;

        let height = Height::new(revision_number, revision_height)?;
        let state = match latest_height {
            true => {
                let latest_height = client_state_get(&self.store_key, ctx, &client_id)?
                    .inner()
                    .latest_height;

                client_consensus_state(&self.store_key, ctx, &client_id, &latest_height)?
            }
            false => client_consensus_state(&self.store_key, ctx, &client_id, &height)?,
        };

        let response = QueryConsensusStateResponse {
            consensus_state: Some(PrimitiveAny::from(state.0).into()),
            proof: Vec::new(), // TODO: ?
            proof_height: Some(height),
        };

        Ok(response)
    }

    pub fn consensus_states<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        QueryConsensusStatesRequest {
            client_id,
            pagination: _,
        }: QueryConsensusStatesRequest,
    ) -> Result<QueryConsensusStatesResponse, ConsensusStatesError> {
        let client_id = ClientId::from_str(&client_id)?;

        let states = {
            let any_store = ctx.kv_store(&self.store_key);
            let store = any_store
                .prefix_store(format!("{KEY_CONSENSUS_STATE_PREFIX}/{client_id}").into_bytes());

            let mut states = Vec::<ConsensusStateWithHeight>::new();
            for (_key, value) in store.range(..) {
                states.push(RawConsensusStateWithHeight::decode::<Bytes>(value.into())?.try_into()?)
            }

            states
        };

        let response = QueryConsensusStatesResponse {
            consensus_states: states,
            pagination: None,
        };

        Ok(response)
    }
}
