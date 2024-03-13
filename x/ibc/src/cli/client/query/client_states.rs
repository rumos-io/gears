use clap::Args;
use gears::client::query::run_query;
use prost::Message;
use proto_messages::cosmos::ibc::{
    query::{QueryClientStatesResponse, RawQueryClientStatesResponse},
    types::core::client::context::types::proto::v1::QueryClientStatesRequest,
};
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

pub(super) fn query_command_handler(
    _args: CliClientStates,
    node: &str,
    height: Option<Height>,
) -> anyhow::Result<String> {
    let query = QueryClientStatesRequest { pagination: None };

    let result = run_query::<QueryClientStatesResponse, RawQueryClientStatesResponse>(
        query.encode_to_vec(),
        "/ibc.core.client.v1.Query/ClientStates".to_owned(),
        node,
        height,
    )?;

    let result = serde_json::to_string_pretty(&result)?;

    Ok(result)
}
