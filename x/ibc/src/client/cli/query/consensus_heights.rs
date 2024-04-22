use clap::Args;
use ibc::core::{
    client::types::proto::v1::QueryConsensusStateHeightsRequest, host::types::identifiers::ClientId,
};

// use proto_messages::cosmos::ibc::types::core::{
//     client::context::types::proto::v1::QueryConsensusStateHeightsRequest,
//     host::identifiers::ClientId,
// };

pub(crate) const CONSESUS_HEIGHTS_URL: &str = "/ibc.core.client.v1.Query/ConsensusStateHeights";

/// Query the heights of all consensus states of a client
#[derive(Args, Debug, Clone)]
pub struct CliConsensusHeight {
    // TODO: Pagination
    client_id: ClientId,
}

pub(crate) fn handle_query(args: &CliConsensusHeight) -> QueryConsensusStateHeightsRequest {
    QueryConsensusStateHeightsRequest {
        client_id: args.client_id.to_string(),
        pagination: None,
    }
}
