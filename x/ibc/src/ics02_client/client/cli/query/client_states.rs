use clap::Args;
use ibc::core::client::types::proto::v1::QueryClientStatesRequest;

pub(crate) const STATES_URL: &str = "/ibc.core.client.v1.Query/ClientStates";

/// Query all available light clients
#[derive(Args, Debug, Clone)]
pub struct CliClientStates; // TODO: pagination

pub(crate) fn handle_query(_args: &CliClientStates) -> QueryClientStatesRequest {
    QueryClientStatesRequest { pagination: None }
}
