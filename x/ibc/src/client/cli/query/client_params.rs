use clap::Args;
use proto_messages::cosmos::ibc::types::core::{
    client::context::types::proto::v1::QueryClientParamsRequest, host::identifiers::ClientId,
};

pub(crate) const PARAMS_URL: &str = "/ibc.core.client.v1.Query/ClientParams";

/// Query the current ibc client parameters
#[derive(Args, Debug, Clone)]
pub struct CliClientParams {
    client_id: ClientId,
}

pub(crate) fn handle_query(_args: &CliClientParams) -> QueryClientParamsRequest {
    QueryClientParamsRequest {} // TODO: is this struct missing something? Like client_id?
}
