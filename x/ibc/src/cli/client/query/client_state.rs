use clap::Args;

use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryClientStateRequest;
use tendermint::informal::block::Height;

pub(crate) const STATE_URL: &str = "/ibc.core.client.v1.Query/UpgradedClientState";

#[derive(Args, Debug, Clone)]
pub struct CliClientState {
    client_id: String,
}

pub(super) fn handle_query(
    args: CliClientState,
    _node: &str,
    _height: Option<Height>,
) -> QueryClientStateRequest {
    QueryClientStateRequest {
        client_id: args.client_id,
    }
}
