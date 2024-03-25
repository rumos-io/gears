use clap::Args;

use proto_messages::cosmos::ibc::types::core::{
    client::context::types::proto::v1::QueryClientStatusRequest, host::identifiers::ClientId,
};

pub(crate) const STATUS_URL: &str = "/ibc.core.client.v1.Query/ClientStatus";

/// Query client status
#[derive(Args, Debug, Clone)]
pub struct CliClientStatus {
    client_id: ClientId,
}

pub(crate) fn handle_query(args: &CliClientStatus) -> QueryClientStatusRequest {
    QueryClientStatusRequest {
        client_id: args.client_id.to_string(),
    }
}
