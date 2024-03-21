use clap::Args;

use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryConsensusStateHeightsRequest;

pub(crate) const CONSESUS_HEIGHTS_URL: &str = "/ibc.core.client.v1.Query/ConsensusStateHeights";

#[derive(Args, Debug, Clone)]
pub struct CliClientHeight {
    // TODO: Pagination
    client_id: String,
}

pub(crate) fn handle_query(args: &CliClientHeight) -> QueryConsensusStateHeightsRequest {
    QueryConsensusStateHeightsRequest {
        client_id: args.client_id.clone(),
        pagination: None,
    }
}
