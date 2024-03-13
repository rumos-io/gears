use std::borrow::Cow;

use clap::{Args, Subcommand};
use gears::application::handlers_v2::QueryHandler;
use prost::Message;
use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryClientParamsResponse as RawQueryClientParamsResponse;
use proto_messages::cosmos::{
    ibc::{
        query::{
            QueryClientParamsResponse, QueryClientStateResponse, QueryClientStatesResponse,
            QueryClientStatusResponse, QueryConsensusStateHeightsResponse,
            QueryConsensusStateResponse, QueryConsensusStatesResponse, RawQueryClientStateResponse,
            RawQueryClientStatesResponse, RawQueryClientStatusResponse,
            RawQueryConsensusStateHeightsResponse, RawQueryConsensusStateResponse,
            RawQueryConsensusStatesResponse,
        },
        types::core::client::context::types::proto::v1::{
            QueryClientParamsRequest, QueryClientStateRequest, QueryClientStatesRequest,
            QueryClientStatusRequest, QueryConsensusStateHeightsRequest,
            QueryConsensusStateRequest, QueryConsensusStatesRequest,
        },
    },
    query::Query,
};
use serde::Serialize;
use tendermint::informal::block::Height;

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
mod proto;
#[allow(dead_code)]
pub mod query_header;
#[allow(dead_code)]
pub mod self_consensus_state;

pub use self::proto::IbcProtoError;

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

pub fn run_ibc_query_command(
    args: IbcQueryCli,
    node: &str,
    height: Option<Height>,
) -> anyhow::Result<String> {
    match args.command {
        IbcQueryCommands::ClientParams(args) => {
            client_params::query_command_handler(args, node, height)
        }
        IbcQueryCommands::ClientState(args) => {
            client_state::query_command_handler(args, node, height)
        }
        IbcQueryCommands::ClientStates(args) => {
            client_states::query_command_handler(args, node, height)
        }
        IbcQueryCommands::ClientStatus(args) => {
            client_status::query_command_handler(args, node, height)
        }
        IbcQueryCommands::ConsensusState(args) => {
            consensus_state::query_command_handler(args, node, height)
        }
        IbcQueryCommands::ConsensusStates(args) => {
            consensus_states::query_command_handler(args, node, height)
        }
        IbcQueryCommands::ConsensusStateHeights(args) => {
            consensus_heights::query_command_handler(args, node, height)
        } // IbcCommands::Header(args) => query_header::query_command_handler(args, node, height),
          // IbcCommands::SelfState(args) => {
          //     self_consensus_state::query_command_handler(args, node, height)
          // }
    }
}

#[derive(Debug, Clone)]
pub struct IbcQueryHandler;

impl QueryHandler for IbcQueryHandler {
    type Query = IbcQuery;
    type QueryCommands = IbcQueryCli;
    type QueryResponse = IbcQueryResponse;
    type RawQueryResponse = RawIbcQueryResponse;

    fn prepare_query(
        &self,
        command: Self::QueryCommands,
        node: &str,
        height: Option<Height>,
    ) -> anyhow::Result<Self::Query> {
        let res = match command.command {
            IbcQueryCommands::ClientParams(args) => {
                Self::Query::ClientParams(client_params::handle_query(args, node, height))
            }
            IbcQueryCommands::ClientState(args) => {
                Self::Query::ClientState(client_state::handle_query(args, node, height))
            }
            IbcQueryCommands::ClientStates(args) => {
                Self::Query::ClientStates(client_states::handle_query(args, node, height))
            }
            IbcQueryCommands::ClientStatus(args) => {
                Self::Query::ClientStatus(client_status::handle_query(args, node, height))
            }
            IbcQueryCommands::ConsensusState(args) => {
                Self::Query::ConsensusState(consensus_state::handle_query(args, node, height))
            }
            IbcQueryCommands::ConsensusStates(args) => {
                Self::Query::ConsensusStates(consensus_states::handle_query(args, node, height))
            }
            IbcQueryCommands::ConsensusStateHeights(args) => Self::Query::ConsensusStateHeights(
                consensus_heights::handle_query(args, node, height),
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

    fn as_bytes(self) -> Vec<u8> {
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

#[derive(Clone, Serialize)]
pub enum IbcQueryResponse {
    ClientParams(QueryClientParamsResponse),
    ClientState(QueryClientStateResponse),
    ClientStates(QueryClientStatesResponse),
    ClientStatus(QueryClientStatusResponse),
    ConsensusState(QueryConsensusStateResponse),
    ConsensusStates(QueryConsensusStatesResponse),
    ConsensusStateHeights(QueryConsensusStateHeightsResponse),
}

#[derive(Debug, Clone)]
pub enum RawIbcQueryResponse {
    ClientParams(RawQueryClientParamsResponse),
    ClientState(RawQueryClientStateResponse),
    ClientStates(RawQueryClientStatesResponse),
    ClientStatus(RawQueryClientStatusResponse),
    ConsensusState(RawQueryConsensusStateResponse),
    ConsensusStates(RawQueryConsensusStatesResponse),
    ConsensusStateHeights(RawQueryConsensusStateHeightsResponse),
}
