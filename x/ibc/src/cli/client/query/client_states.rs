use clap::Args;
use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryClientStatesRequest;
use tendermint::informal::block::Height;

pub(crate) const STATES_URL: &str = "/ibc.core.client.v1.Query/ClientStates";

#[derive(Args, Debug, Clone)]
pub struct CliClientStates; // TODO: pagination

pub(super) fn handle_query(
    _args: CliClientStates,
    _node: &str,
    _height: Option<Height>,
) -> QueryClientStatesRequest {
    QueryClientStatesRequest { pagination: None }
}
