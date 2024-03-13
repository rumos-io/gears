use clap::Args;
use gears::client::query::run_query;
use prost::Message;
use proto_messages::cosmos::ibc::{
    query::QueryClientParamsResponse,
    types::core::{
        client::context::types::proto::v1::{
            QueryClientParamsRequest, QueryClientParamsResponse as RawQueryClientParamsResponse,
        },
        connection::proto::v1::QueryClientConnectionsRequest,
    },
};
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

pub(super) fn query_command_handler(
    args: CliClientParams,
    node: &str,
    height: Option<Height>,
) -> anyhow::Result<String> {
    let query = QueryClientConnectionsRequest {
        client_id: args.client_id,
    };

    let result = run_query::<QueryClientParamsResponse, RawQueryClientParamsResponse>(
        query.encode_to_vec(),
        "/ibc.core.client.v1.Query/ClientParams".to_owned(),
        node,
        height,
    )?;

    let result = serde_json::to_string_pretty(&result)?;

    Ok(result)
}
