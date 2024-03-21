use clap::Args;
use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryClientParamsRequest;

pub(crate) const CLIENT_PARAMS_URL: &str = "/ibc.core.client.v1.Query/ClientParams";

#[derive(Args, Debug, Clone)]
pub struct CliClientParams {
    client_id: String,
}

pub(crate) fn handle_query(_args: &CliClientParams) -> QueryClientParamsRequest {
    QueryClientParamsRequest {} // TODO: is this struct missing something? Like client_id?
}
