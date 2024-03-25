use std::str::FromStr;

use bytes::Bytes;
use database::Database;
use gears::types::context::query_context::QueryContext;
use gears::x::params::ParamsSubspaceKey;
use prost::Message;
use proto_messages::cosmos::ibc::types::core::client::context::client_state::ClientStateCommon;
use proto_messages::cosmos::ibc::types::core::client::context::client_state::ClientStateValidation;
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
use store::StoreKey;

use crate::keeper::{params_get, KEY_CLIENT_STORE_PREFIX, KEY_CONSENSUS_STATE_PREFIX};
use crate::params::AbciParamsKeeper;
use crate::types::ContextShim;

use super::{client_consensus_state, client_state_get};

#[derive(Debug, Clone)]
pub struct QueryKeeper<SK: StoreKey> {
    store_key: SK,
}

impl<SK: StoreKey> QueryKeeper<SK> {
    pub fn new(store_key: SK) -> Self {
        Self { store_key }
    }

    pub fn client_params<DB: Database + Send + Sync, PSK: ParamsSubspaceKey>(
        &mut self,
        ctx: &mut QueryContext<'_, DB, SK>,
        params_keeper: &AbciParamsKeeper<SK, PSK>,
    ) -> anyhow::Result<QueryClientParamsResponse> {
        let params = params_get(params_keeper, ctx)?;

        let response = QueryClientParamsResponse { params };

        Ok(response)
    }

    pub fn client_state<DB: Database>(
        &mut self,
        ctx: &mut QueryContext<'_, DB, SK>,
        QueryClientStateRequest { client_id }: QueryClientStateRequest,
    ) -> anyhow::Result<QueryClientStateResponse> {
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
        &mut self,
        ctx: &mut QueryContext<'_, DB, SK>,
        QueryClientStatesRequest { pagination: _ }: QueryClientStatesRequest,
    ) -> anyhow::Result<QueryClientStatesResponse> {
        let any_store = ctx.get_kv_store(&self.store_key);
        let store: store::ImmutablePrefixStore<'_, database::PrefixDB<DB>> =
            any_store.get_immutable_prefix_store(KEY_CLIENT_STORE_PREFIX.to_owned().into_bytes());

        let mut states = Vec::<IdentifiedClientState>::new();
        for (_key, value) in store.range(..) {
            states.push(RawIdentifiedClientState::decode::<Bytes>(value.into())?.try_into()?);
        }

        let response = QueryClientStatesResponse {
            client_states: states,
            pagination: None,
        };

        Ok(response)
    }

    pub fn client_status<DB: Database + Send + Sync, PSK: ParamsSubspaceKey>(
        &mut self,
        ctx: &mut QueryContext<'_, DB, SK>,
        params_keeper: &AbciParamsKeeper<SK, PSK>,
        QueryClientStatusRequest { client_id }: QueryClientStatusRequest,
    ) -> anyhow::Result<QueryClientStatusResponse> {
        let client_id = ClientId::from_str(&client_id)?;
        let client_state = client_state_get(&self.store_key, ctx, &client_id)?;
        let client_type = client_state.client_type();

        let params = params_get(&params_keeper, ctx)?;

        let status = if !params.is_client_allowed(&client_type) {
            Status::Unauthorized
        } else {
            client_state.status(&ContextShim::from(&*ctx), &client_id)?
        };

        let response = QueryClientStatusResponse {
            status: status.to_string(),
        };

        Ok(response)
    }

    pub fn consensus_state_heights<DB: Database>(
        &mut self,
        ctx: &mut QueryContext<'_, DB, SK>,
        QueryConsensusStateHeightsRequest {
            client_id,
            pagination: _,
        }: QueryConsensusStateHeightsRequest,
    ) -> anyhow::Result<QueryConsensusStateHeightsResponse> {
        let client_id = ClientId::from_str(&client_id)?;
        let store = ctx
            .get_kv_store(&self.store_key)
            .get_immutable_prefix_store(
                format!("{KEY_CLIENT_STORE_PREFIX}/{client_id}/{KEY_CONSENSUS_STATE_PREFIX}")
                    .into_bytes(),
            );

        let mut heights = Vec::<Height>::new();
        for (_key, value) in store.range(..) {
            heights.push(Height::decode_vec(&value)?);
        }

        let response = QueryConsensusStateHeightsResponse {
            consensus_state_heights: heights,
            pagination: None,
        };

        Ok(response)
    }

    pub fn consensus_state<DB: Database>(
        &mut self,
        ctx: &mut QueryContext<'_, DB, SK>,
        QueryConsensusStateRequest {
            client_id,
            revision_number,
            revision_height,
            latest_height,
        }: QueryConsensusStateRequest,
    ) -> anyhow::Result<QueryConsensusStateResponse> {
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
        &mut self,
        ctx: &mut QueryContext<'_, DB, SK>,
        QueryConsensusStatesRequest {
            client_id,
            pagination: _,
        }: QueryConsensusStatesRequest,
    ) -> anyhow::Result<QueryConsensusStatesResponse> {
        let client_id = ClientId::from_str(&client_id)?;

        let states = {
            let any_store = ctx.get_kv_store(&self.store_key);
            let store = any_store.get_immutable_prefix_store(
                format!("{KEY_CONSENSUS_STATE_PREFIX}/{client_id}").into_bytes(),
            );

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
