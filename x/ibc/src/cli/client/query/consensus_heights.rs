use clap::Args;

use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryConsensusStateHeightsRequest;
use tendermint::informal::block::Height;

pub(crate) const CONSESUS_HEIGHTS_URL: &str = "/ibc.core.client.v1.Query/ConsensusStateHeights";

#[derive(Args, Debug, Clone)]
pub struct CliClientHeight {
    // TODO: Pagination
    client_id: String,
}

pub(super) fn handle_query(
    args: CliClientHeight,
    _node: &str,
    _height: Option<Height>,
) -> QueryConsensusStateHeightsRequest {
    QueryConsensusStateHeightsRequest {
        client_id: args.client_id,
        pagination: None,
    }
}
