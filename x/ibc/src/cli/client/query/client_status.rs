use clap::Args;
use gears::client::query::run_query;
use prost::Message;
use proto_messages::cosmos::ibc::{
    query::{QueryClientStatusResponse, RawQueryClientStatusResponse},
    types::core::client::context::types::proto::v1::QueryClientStatusRequest,
};
use tendermint::informal::block::Height;

#[derive(Args, Debug, Clone)]
pub struct CliClientParams {
    client_id: String,
}

pub(super) async fn query_command_handler(
    args: CliClientParams,
    node: &str,
    height: Option<Height>,
) -> anyhow::Result<String> {
    let query = QueryClientStatusRequest {
        client_id: args.client_id,
    };

    let result = run_query::<QueryClientStatusResponse, RawQueryClientStatusResponse>(
        query.encode_to_vec(),
        "/ibc.core.client.v1.Query/ClientStatus".to_owned(),
        node,
        height,
    )
    .await?;

    let result = serde_json::to_string_pretty(&result)?;

    Ok(result)
}
