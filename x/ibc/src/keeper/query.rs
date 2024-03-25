use std::str::FromStr;

use database::Database;
use gears::types::context::query_context::QueryContext;
use proto_messages::{
    any::PrimitiveAny,
    cosmos::ibc::{
        protobuf::Protobuf,
        query::response::{
            QueryClientParamsResponse, QueryClientStateResponse,
            QueryConsensusStateHeightsResponse, QueryConsensusStateResponse,
            RawQueryClientStateResponse,
        },
        types::core::{
            client::{
                context::types::proto::v1::{
                    QueryClientStateRequest, QueryConsensusStateHeightsRequest,
                    QueryConsensusStateRequest,
                },
                types::{Height, ProtoHeight},
            },
            host::identifiers::ClientId,
        },
    },
};
use store::StoreKey;

use crate::keeper::{KEY_CLIENT_STORE_PREFIX, KEY_CONSENSUS_STATE_PREFIX};

use super::{client_consensus_state, client_state_get};

#[derive(Debug, Clone)]
pub struct QueryKeeper<SK: StoreKey> {
    store_key: SK,
}

impl<SK: StoreKey> QueryKeeper<SK> {
    pub fn new(store_key: SK) -> Self {
        QueryKeeper { store_key }
    }

    pub fn client_params<DB: Database + Send + Sync>(
        &mut self,
        _ctx: &mut QueryContext<'_, DB, SK>,
    ) -> QueryClientParamsResponse {
        todo!()
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

    pub fn client_states() -> anyhow::Result<()> {
        Ok(())
    }

    pub fn client_status() -> anyhow::Result<()> {
        Ok(())
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

    pub fn consensus_states() -> anyhow::Result<()> {
        Ok(())
    }
}