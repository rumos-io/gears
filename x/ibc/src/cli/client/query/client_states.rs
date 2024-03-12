use clap::Args;
use gears::client::query::run_query;
use prost::Message;
use proto_messages::cosmos::ibc::{
    query::{QueryClientStatesResponse, RawQueryClientStatesResponse},
    types::core::client::context::types::proto::v1::QueryClientStatesRequest,
};
use tendermint::informal::block::Height;

#[derive(Args, Debug, Clone)]
pub struct CliClientParams; // TODO: pagination

pub(super) async fn query_command_handler(
    _args: CliClientParams,
    node: &str,
    height: Option<Height>,
) -> anyhow::Result<String> {
    let query = QueryClientStatesRequest { pagination: None };

    let result = run_query::<QueryClientStatesResponse, RawQueryClientStatesResponse>(
        query.encode_to_vec(),
        "/ibc.core.client.v1.Query/ClientStates".to_owned(),
        node,
        height,
    )
    .await?;

    let result = serde_json::to_string_pretty(&result)?;

    Ok(result)
}
