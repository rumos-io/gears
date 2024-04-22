use clap::Args;
use ibc::core::{
    client::types::proto::v1::QueryConsensusStatesRequest, host::types::identifiers::ClientId,
};
// use proto_messages::cosmos::ibc::types::core::{
//     client::context::types::proto::v1::QueryConsensusStatesRequest, host::identifiers::ClientId,
// };

pub(crate) const CONSENSUS_STATES_URL: &str = "/ibc.core.client.v1.Query/ConsensusStates";

/// Query all the consensus states of a client
#[derive(Args, Debug, Clone)]
pub struct CliConsensusStates {
    // TODO: Pagination
    client_id: ClientId,
}

pub(crate) fn handle_query(args: &CliConsensusStates) -> QueryConsensusStatesRequest {
    QueryConsensusStatesRequest {
        client_id: args.client_id.to_string(),
        pagination: None,
    }
}
