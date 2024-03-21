use std::borrow::Cow;

use clap::{Args, Subcommand};
use prost::Message;
use proto_messages::cosmos::{
    ibc::{
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
    pub command: IbcQueryCommands,
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
