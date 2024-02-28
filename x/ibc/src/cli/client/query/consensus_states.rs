use clap::Args;
use gears::{client::query::run_query, types::context::query_context::QueryContext};
use prost::Message;
use proto_messages::cosmos::ibc::{
    query::{QueryConsensusStatesResponse, RawQueryConsensusStatesResponse},
    types::core::client::context::types::proto::v1::QueryConsensusStatesRequest,
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
    let query = QueryConsensusStatesRequest {
        client_id: args.client_id,
        pagination: None,
    };

    let result = run_query::<QueryConsensusStatesResponse, RawQueryConsensusStatesResponse>(
        query.encode_to_vec(),
        "/ibc.core.client.v1.Query/ConsensusStates".to_owned(),
        node,
        height,
    )?;

    let result = serde_json::to_string_pretty(&result)?;

    Ok(result)
}
