use std::borrow::Cow;

use clap::{Args, Subcommand};
use gears::types::query::Query;
use ibc::core::client::types::proto::v1::{
    QueryClientParamsRequest, QueryClientParamsResponse, QueryClientStateRequest,
    QueryClientStateResponse, QueryClientStatesRequest, QueryClientStatesResponse,
    QueryClientStatusRequest, QueryClientStatusResponse, QueryConsensusStateHeightsResponse,
    QueryConsensusStateRequest, QueryConsensusStateResponse, QueryConsensusStatesRequest,
    QueryConsensusStatesResponse,
};
use prost::Message;

use serde::{Deserialize, Serialize};

use crate::ics02_client::client::cli::query::{ClientQuery, ClientQueryCli, ClientQueryResponse};

use self::{
    client_params::PARAMS_URL, client_state::STATE_URL, client_states::STATES_URL,
    client_status::STATUS_URL, consensus_state::CONSENSUS_STATE_URL,
    consensus_states::CONSENSUS_STATES_URL,
};

pub mod client_params;
pub mod client_state;
pub mod client_states;
pub mod client_status;
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

#[derive(Subcommand, Debug, Clone)]
pub enum IbcQueryCommands {
    Client(ClientQueryCli),
}

// #[derive(Subcommand, Debug)]
// pub enum IbcQueryCommands {
//     #[command(name = "params")]
//     ClientParams(client_params::CliClientParams),
//     #[command(name = "state")]
//     ClientState(client_state::CliClientState),
//     #[command(name = "states")]
//     ClientStates(client_states::CliClientStates),
//     #[command(name = "status")]
//     ClientStatus(client_status::CliClientStatus),
//     #[command(name = "consensus-state")]
//     ConsensusState(consensus_state::CliConsensusState),
//     #[command(name = "consensus-states")]
//     ConsensusStates(consensus_states::CliConsensusStates),
//     // Header(query_header::CliClientParams),
//     // SelfConsensusState(self_consensus_state::CliClientParams),
// }

#[derive(Clone, PartialEq)]
pub enum IbcQuery {
    Client(ClientQuery),
    // ClientParams(QueryClientParamsRequest),
    // ClientState(QueryClientStateRequest),
    // ClientStates(QueryClientStatesRequest),
    // ClientStatus(QueryClientStatusRequest),
    // ConsensusState(QueryConsensusStateRequest),
    // ConsensusStates(QueryConsensusStatesRequest),
}

impl Query for IbcQuery {
    fn query_url(&self) -> Cow<'static, str> {
        match self {
            IbcQuery::Client(var) => var.query_url(),
            // IbcQuery::ClientParams(_) => Cow::Borrowed(PARAMS_URL),
            // IbcQuery::ClientState(_) => Cow::Borrowed(STATE_URL),
            // IbcQuery::ClientStates(_) => Cow::Borrowed(STATES_URL),
            // IbcQuery::ClientStatus(_) => Cow::Borrowed(STATUS_URL),
            // IbcQuery::ConsensusState(_) => Cow::Borrowed(CONSENSUS_STATE_URL),
            // IbcQuery::ConsensusStates(_) => Cow::Borrowed(CONSENSUS_STATES_URL),
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            IbcQuery::Client(var) => var.into_bytes(),
            // IbcQuery::ClientParams(var) => var.encode_to_vec(),
            // IbcQuery::ClientState(var) => var.encode_to_vec(),
            // IbcQuery::ClientStates(var) => var.encode_to_vec(),
            // IbcQuery::ClientStatus(var) => var.encode_to_vec(),
            // IbcQuery::ConsensusState(var) => var.encode_to_vec(),
            // IbcQuery::ConsensusStates(var) => var.encode_to_vec(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IbcQueryResponse {
    Client(ClientQueryResponse),
    // ClientParams(QueryClientParamsResponse),
    // ClientState(QueryClientStateResponse),
    // ClientStates(QueryClientStatesResponse),
    // ClientStatus(QueryClientStatusResponse),
    // ConsensusState(QueryConsensusStateResponse),
    // ConsensusStates(QueryConsensusStatesResponse),
    // ConsensusStateHeights(QueryConsensusStateHeightsResponse),
}
