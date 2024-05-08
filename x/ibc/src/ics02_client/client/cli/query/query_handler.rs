use gears::application::handlers::client::QueryHandler;
use ibc::core::client::types::proto::v1::{
    QueryClientParamsResponse, QueryClientStateResponse, QueryClientStatusResponse,
    QueryConsensusStateResponse, QueryConsensusStatesResponse,
};
//use prost::bytes::Bytes;
//use prost_012::Message;

use crate::ics02_client::types::query::QueryClientStatesResponse;

use super::{
    client_states::{self, CliClientStates},
    ClientQuery, ClientQueryCli, ClientQueryCommands, ClientQueryResponse,
};
use ibc::primitives::proto::Protobuf;

// use super::query::{
//     client_params, client_state, client_states, client_status, consensus_state, consensus_states,
//     IbcQuery, IbcQueryCli, IbcQueryCommands, IbcQueryResponse,
// };

#[derive(Debug, Clone)]
pub struct ClientQueryHandler;

impl QueryHandler for ClientQueryHandler {
    type QueryRequest = ClientQuery;
    type QueryCommands = ClientQueryCli;
    type QueryResponse = ClientQueryResponse;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let res = match &command.command {
            ClientQueryCommands::ClientParams(_) => todo!(),
            ClientQueryCommands::ClientState(_) => todo!(),
            ClientQueryCommands::ClientStates(args) => {
                Self::QueryRequest::ClientStates(client_states::handle_query(args))
            }
            ClientQueryCommands::ClientStatus(_) => todo!(),
            ClientQueryCommands::ConsensusState(_) => todo!(),
            ClientQueryCommands::ConsensusStates(_) => todo!(),
            // IbcQueryCommands::Client(command) => {
            //     Self::QueryRequest::Client(ClientQueryHandler.prepare_query_request(command)?)
            // } // IbcQueryCommands::ClientParams(args) => {
            //   //     Self::QueryRequest::ClientParams(client_params::handle_query(args))
            //   // }
            //   // IbcQueryCommands::ClientState(args) => {
            //   //     Self::QueryRequest::ClientState(client_state::handle_query(args))
            //   // }
            //   // IbcQueryCommands::ClientStates(args) => {
            //   //     Self::QueryRequest::ClientStates(client_states::handle_query(args))
            //   // }
            //   // IbcQueryCommands::ClientStatus(args) => {
            //   //     Self::QueryRequest::ClientStatus(client_status::handle_query(args))
            //   // }
            //   // IbcQueryCommands::ConsensusState(args) => {
            //   //     Self::QueryRequest::ConsensusState(consensus_state::handle_query(args))
            //   // }
            //   // IbcQueryCommands::ConsensusStates(args) => {
            //   //     Self::QueryRequest::ConsensusStates(consensus_states::handle_query(args))
            //   // }
        };

        Ok(res)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let res = match &command.command {
            ClientQueryCommands::ClientParams(_) => todo!(),
            ClientQueryCommands::ClientState(_) => todo!(),
            ClientQueryCommands::ClientStates(_) => ClientQueryResponse::ClientStates(
                QueryClientStatesResponse::decode_vec(&query_bytes)?,
            ),
            ClientQueryCommands::ClientStatus(_) => todo!(),
            ClientQueryCommands::ConsensusState(_) => todo!(),
            ClientQueryCommands::ConsensusStates(_) => todo!(),
            //IbcQueryCommands::Client(_) => todo!(),
            // // *This is fine*.png
            // IbcQueryCommands::ClientParams(_) => IbcQueryResponse::ClientParams(
            //     QueryClientParamsResponse::decode::<Bytes>(query_bytes.into())?,
            // ),
            // IbcQueryCommands::ClientState(_) => IbcQueryResponse::ClientState(
            //     QueryClientStateResponse::decode::<Bytes>(query_bytes.into())?,
            // ),
            // IbcQueryCommands::ClientStates(_) => IbcQueryResponse::ClientStates(
            //     QueryClientStatesResponse::decode::<Bytes>(query_bytes.into())?,
            // ),
            // IbcQueryCommands::ClientStatus(_) => IbcQueryResponse::ClientStatus(
            //     QueryClientStatusResponse::decode::<Bytes>(query_bytes.into())?,
            // ),
            // IbcQueryCommands::ConsensusState(_) => IbcQueryResponse::ConsensusState(
            //     QueryConsensusStateResponse::decode::<Bytes>(query_bytes.into())?,
            // ),
            // IbcQueryCommands::ConsensusStates(_) => IbcQueryResponse::ConsensusStates(
            //     QueryConsensusStatesResponse::decode::<Bytes>(query_bytes.into())?,
            // ),
        };

        Ok(res)
    }
}
