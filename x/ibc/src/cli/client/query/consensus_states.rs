use clap::Args;
use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryConsensusStatesRequest;
use tendermint::informal::block::Height;

pub(crate) const CONSENSUS_STATES_URL: &str = "/ibc.core.client.v1.Query/ConsensusStates";

#[derive(Args, Debug, Clone)]
pub struct CliConsensusStates {
    // TODO: Pagination
    client_id: String,
}

pub(super) fn handle_query(
    args: CliConsensusStates,
    _node: &str,
    _height: Option<Height>,
) -> QueryConsensusStatesRequest {
    QueryConsensusStatesRequest {
        client_id: args.client_id,
        pagination: None,
    }
}
