use clap::Args;
use ibc::core::{
    client::types::proto::v1::QueryClientStateRequest, host::types::identifiers::ClientId,
};

// use proto_messages::cosmos::ibc::types::core::{
//     client::context::types::proto::v1::QueryClientStateRequest, host::identifiers::ClientId,
// };

pub(crate) const STATE_URL: &str = "/ibc.core.client.v1.Query/UpgradedClientState";

/// Query a client state
#[derive(Args, Debug, Clone)]
pub struct CliClientState {
    client_id: ClientId,
}

pub(crate) fn handle_query(args: &CliClientState) -> QueryClientStateRequest {
    QueryClientStateRequest {
        client_id: args.client_id.to_string(),
    }
}
