use clap::Args;

use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryClientStatusRequest;
use tendermint::informal::block::Height;

pub(crate) const STATUS_URL: &str = "/ibc.core.client.v1.Query/ClientStatus";

#[derive(Args, Debug, Clone)]
pub struct CliClientStatus {
    client_id: String,
}

pub(super) fn handle_query(
    args: CliClientStatus,
    _node: &str,
    _height: Option<Height>,
) -> QueryClientStatusRequest {
    QueryClientStatusRequest {
        client_id: args.client_id,
    }
}
