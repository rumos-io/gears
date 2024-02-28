use clap::Args;
use gears::{client::query::run_query, types::context::query_context::QueryContext};
use prost::Message;
use proto_messages::cosmos::ibc::{
    query::{QueryConsensusStateHeightsResponse, RawQueryConsensusStateHeightsResponse},
    types::core::client::context::types::proto::v1::QueryConsensusStateHeightsRequest,
};
use tendermint::informal::block::Height;

#[derive(Args, Debug, Clone)]
pub struct CliClientParams {
    // TODO: Pagination
    client_id: String,
}

#[allow(dead_code)]
pub(super) fn query_command_handler<DB, SK>(
    _ctx: &QueryContext<'_, DB, SK>,
    args: CliClientParams,
    node: &str,
    height: Option<Height>,
) -> anyhow::Result<String> {
    let query = QueryConsensusStateHeightsRequest {
        client_id: args.client_id,
        pagination: None,
    };

    let result =
        run_query::<QueryConsensusStateHeightsResponse, RawQueryConsensusStateHeightsResponse>(
            query.encode_to_vec(),
            "/ibc.core.client.v1.Query/ConsensusStateHeights".to_owned(),
            node,
            height,
        )?;

    let result = serde_json::to_string_pretty(&result)?;

    Ok(result)
}
