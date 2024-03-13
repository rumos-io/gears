use clap::Args;
use gears::client::query::run_query;
use prost::Message;
use proto_messages::cosmos::ibc::{
    query::{QueryClientStatusResponse, RawQueryClientStatusResponse},
    types::core::client::context::types::proto::v1::QueryClientStatusRequest,
};
use tendermint::informal::block::Height;

pub(crate) const STATUS_URL: &str = "/ibc.core.client.v1.Query/ClientStatus";

#[derive(Args, Debug, Clone)]
pub struct CliClientStatus {
    client_id: String,
}

pub(super) fn handle_query(
    args: CliClientStatus,
    _node: &str,
    _height: Option<Height>,
) -> QueryClientStatusRequest {
    QueryClientStatusRequest {
        client_id: args.client_id,
    }
}

pub(super) fn query_command_handler(
    args: CliClientStatus,
    node: &str,
    height: Option<Height>,
) -> anyhow::Result<String> {
    let query: QueryClientStatusRequest = QueryClientStatusRequest {
        client_id: args.client_id,
    };

    let result = run_query::<QueryClientStatusResponse, RawQueryClientStatusResponse>(
        query.encode_to_vec(),
        "/ibc.core.client.v1.Query/ClientStatus".to_owned(),
        node,
        height,
    )?;

    let result = serde_json::to_string_pretty(&result)?;

    Ok(result)
}
