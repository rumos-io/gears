use clap::Args;
use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryConsensusStatesRequest;

pub(crate) const CONSENSUS_STATES_URL: &str = "/ibc.core.client.v1.Query/ConsensusStates";

#[derive(Args, Debug, Clone)]
pub struct CliConsensusStates {
    // TODO: Pagination
    client_id: String,
}

pub(crate) fn handle_query(args: &CliConsensusStates) -> QueryConsensusStatesRequest {
    QueryConsensusStatesRequest {
        client_id: args.client_id.clone(),
        pagination: None,
    }
}
