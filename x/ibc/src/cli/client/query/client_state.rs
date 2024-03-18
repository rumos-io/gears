use clap::Args;

use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryClientStateRequest;

pub(crate) const STATE_URL: &str = "/ibc.core.client.v1.Query/UpgradedClientState";

#[derive(Args, Debug, Clone)]
pub struct CliClientState {
    client_id: String,
}

pub(super) fn handle_query(args: &CliClientState) -> QueryClientStateRequest {
    QueryClientStateRequest {
        client_id: args.client_id.clone(),
    }
}
