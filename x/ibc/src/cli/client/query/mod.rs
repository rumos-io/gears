use std::borrow::Cow;

use clap::{Args, Subcommand};
use gears::application::handlers::QueryHandler;
use prost::bytes::Bytes;
use prost::Message;
use proto_messages::cosmos::{
    ibc::{
        protobuf::Protobuf,
        query::{
            QueryClientParamsResponse, QueryClientStateResponse, QueryClientStatesResponse,
            QueryClientStatusResponse, QueryConsensusStateHeightsResponse,
            QueryConsensusStateResponse, QueryConsensusStatesResponse,
        },
        types::core::client::context::types::proto::v1::{
            QueryClientParamsRequest, QueryClientStateRequest, QueryClientStatesRequest,
            QueryClientStatusRequest, QueryConsensusStateHeightsRequest,
            QueryConsensusStateRequest, QueryConsensusStatesRequest,
        },
    },
    query::Query,
};
use serde::{Deserialize, Serialize};

use self::{
    client_params::PARAMS_URL, client_state::STATE_URL, client_states::STATES_URL,
    client_status::STATUS_URL, consensus_heights::CONSESUS_HEIGHTS_URL,
    consensus_state::CONSENSUS_STATE_URL, consensus_states::CONSENSUS_STATES_URL,
};

pub mod client_params;
pub mod client_state;
pub mod client_states;
pub mod client_status;
pub mod consensus_heights;
pub mod consensus_state;
pub mod consensus_states;
#[allow(dead_code)]
pub mod query_header;
#[allow(dead_code)]
pub mod self_consensus_state;

/// Querying commands for the ibc module
#[derive(Args, Debug)]
pub struct IbcQueryCli {
    #[command(subcommand)]
    command: IbcQueryCommands,
}

#[derive(Subcommand, Debug)]
pub enum IbcQueryCommands {
    ClientParams(client_params::CliClientParams),
    ClientState(client_state::CliClientState),
    ClientStates(client_states::CliClientStates),
    ClientStatus(client_status::CliClientStatus),
    ConsensusState(consensus_state::CliConsensusState),
    ConsensusStates(consensus_states::CliConsensusStates),
    ConsensusStateHeights(consensus_heights::CliClientHeight),
    // Header(query_header::CliClientParams),
    // SelfState(self_consensus_state::CliClientParams),
}

#[derive(Debug, Clone)]
pub struct IbcQueryHandler;

impl QueryHandler for IbcQueryHandler {
    type QueryRequest = IbcQuery;
    type QueryCommands = IbcQueryCli;
    type QueryResponse = IbcQueryResponse;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let res = match &command.command {
            IbcQueryCommands::ClientParams(args) => {
                Self::QueryRequest::ClientParams(client_params::handle_query(args))
            }
            IbcQueryCommands::ClientState(args) => {
                Self::QueryRequest::ClientState(client_state::handle_query(args))
            }
            IbcQueryCommands::ClientStates(args) => {
                Self::QueryRequest::ClientStates(client_states::handle_query(args))
            }
            IbcQueryCommands::ClientStatus(args) => {
                Self::QueryRequest::ClientStatus(client_status::handle_query(args))
            }
            IbcQueryCommands::ConsensusState(args) => {
                Self::QueryRequest::ConsensusState(consensus_state::handle_query(args))
            }
            IbcQueryCommands::ConsensusStates(args) => {
                Self::QueryRequest::ConsensusStates(consensus_states::handle_query(args))
            }
            IbcQueryCommands::ConsensusStateHeights(args) => {
                Self::QueryRequest::ConsensusStateHeights(consensus_heights::handle_query(args))
            }
        };

        Ok(res)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let res = match &command.command {
            // *This is fine*.png
            IbcQueryCommands::ClientParams(_) => IbcQueryResponse::ClientParams(
                QueryClientParamsResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            IbcQueryCommands::ClientState(_) => IbcQueryResponse::ClientState(
                QueryClientStateResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            IbcQueryCommands::ClientStates(_) => IbcQueryResponse::ClientStates(
                QueryClientStatesResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            IbcQueryCommands::ClientStatus(_) => IbcQueryResponse::ClientStatus(
                QueryClientStatusResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            IbcQueryCommands::ConsensusState(_) => IbcQueryResponse::ConsensusState(
                QueryConsensusStateResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            IbcQueryCommands::ConsensusStates(_) => IbcQueryResponse::ConsensusStates(
                QueryConsensusStatesResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            IbcQueryCommands::ConsensusStateHeights(_) => IbcQueryResponse::ConsensusStateHeights(
                QueryConsensusStateHeightsResponse::decode::<Bytes>(query_bytes.into())?,
            ),
        };

        Ok(res)
    }
}

#[derive(Clone, PartialEq)]
pub enum IbcQuery {
    ClientParams(QueryClientParamsRequest),
    ClientState(QueryClientStateRequest),
    ClientStates(QueryClientStatesRequest),
    ClientStatus(QueryClientStatusRequest),
    ConsensusState(QueryConsensusStateRequest),
    ConsensusStates(QueryConsensusStatesRequest),
    ConsensusStateHeights(QueryConsensusStateHeightsRequest),
}

impl Query for IbcQuery {
    fn query_url(&self) -> Cow<'static, str> {
        match self {
            IbcQuery::ClientParams(_) => Cow::Borrowed(PARAMS_URL),
            IbcQuery::ClientState(_) => Cow::Borrowed(STATE_URL),
            IbcQuery::ClientStates(_) => Cow::Borrowed(STATES_URL),
            IbcQuery::ClientStatus(_) => Cow::Borrowed(STATUS_URL),
            IbcQuery::ConsensusState(_) => Cow::Borrowed(CONSENSUS_STATE_URL),
            IbcQuery::ConsensusStates(_) => Cow::Borrowed(CONSENSUS_STATES_URL),
            IbcQuery::ConsensusStateHeights(_) => Cow::Borrowed(CONSESUS_HEIGHTS_URL),
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            IbcQuery::ClientParams(var) => var.encode_to_vec(),
            IbcQuery::ClientState(var) => var.encode_to_vec(),
            IbcQuery::ClientStates(var) => var.encode_to_vec(),
            IbcQuery::ClientStatus(var) => var.encode_to_vec(),
            IbcQuery::ConsensusState(var) => var.encode_to_vec(),
            IbcQuery::ConsensusStates(var) => var.encode_to_vec(),
            IbcQuery::ConsensusStateHeights(var) => var.encode_to_vec(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IbcQueryResponse {
    ClientParams(QueryClientParamsResponse),
    ClientState(QueryClientStateResponse),
    ClientStates(QueryClientStatesResponse),
    ClientStatus(QueryClientStatusResponse),
    ConsensusState(QueryConsensusStateResponse),
    ConsensusStates(QueryConsensusStatesResponse),
    ConsensusStateHeights(QueryConsensusStateHeightsResponse),
}
