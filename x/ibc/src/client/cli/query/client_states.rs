use clap::Args;
use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryClientStatesRequest;

pub(crate) const STATES_URL: &str = "/ibc.core.client.v1.Query/ClientStates";

#[derive(Args, Debug, Clone)]
pub struct CliClientStates; // TODO: pagination

pub(super) fn handle_query(_args: &CliClientStates) -> QueryClientStatesRequest {
    QueryClientStatesRequest { pagination: None }
}
