use clap::Args;
use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryClientParamsRequest;
use tendermint::informal::block::Height;

pub(crate) const PARAMS_URL: &str = "/ibc.core.client.v1.Query/ClientParams";

#[derive(Args, Debug, Clone)]
pub struct CliClientParams {
    client_id: String,
}

pub(super) fn handle_query(
    _args: CliClientParams,
    _node: &str,
    _height: Option<Height>,
) -> QueryClientParamsRequest {
    QueryClientParamsRequest {}
}
