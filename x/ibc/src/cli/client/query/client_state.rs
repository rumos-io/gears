use clap::Args;
use gears::client::query::run_query;
use prost::Message;
use proto_messages::cosmos::ibc::{
    query::{QueryClientStateResponse, RawQueryClientStateResponse},
    types::core::client::context::types::proto::v1::QueryClientStateRequest,
};
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

pub(super) fn query_command_handler(
    args: CliClientState,
    node: &str,
    height: Option<Height>,
) -> anyhow::Result<String> {
    let query = QueryClientStateRequest {
        client_id: args.client_id,
    };

    let result = run_query::<QueryClientStateResponse, RawQueryClientStateResponse>(
        query.encode_to_vec(),
        "/ibc.core.client.v1.Query/UpgradedClientState".to_owned(),
        node,
        height,
    )?;

    let result = serde_json::to_string_pretty(&result)?;

    Ok(result)
}
