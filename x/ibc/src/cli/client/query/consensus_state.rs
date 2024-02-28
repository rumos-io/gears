use clap::Args;
use gears::{client::query::run_query, types::context::query_context::QueryContext};
use prost::Message;
use proto_messages::cosmos::ibc::{query::{QueryConsensusStateResponse, RawQueryConsensusStateResponse}, types::core::client::context::types::proto::v1::QueryConsensusStateRequest};
use tendermint::informal::block::Height;

#[derive(Args, Debug, Clone)]
pub struct CliClientParams
{
    pub client_id: String,
    pub revision_number: u64,
    pub revision_height: u64,
    pub latest_height: bool,
}

#[allow(dead_code)]
pub(super) fn query_command_handler<DB, SK>(
    _ctx: &QueryContext<'_, DB, SK>,
    args: CliClientParams,
    node: &str,
    height: Option<Height>,
) -> anyhow::Result<String> {
    let CliClientParams { client_id, revision_number, revision_height, latest_height } = args;
    let query = QueryConsensusStateRequest{ client_id, revision_number, revision_height, latest_height };

    let result =
        run_query::<QueryConsensusStateResponse, RawQueryConsensusStateResponse>(
            query.encode_to_vec(),
            "/ibc.core.client.v1.Query/ConsensusState".to_owned(),
            node,
            height,
        )?;

    let result = serde_json::to_string_pretty(&result)?;

    Ok(result)
}
