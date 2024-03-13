use clap::Args;
use gears::client::query::run_query;
use prost::Message;
use proto_messages::cosmos::ibc::{
    query::{QueryConsensusStatesResponse, RawQueryConsensusStatesResponse},
    types::core::client::context::types::proto::v1::QueryConsensusStatesRequest,
};
use tendermint::informal::block::Height;

pub(crate) const CONSENSUS_STATES_URL: &str = "/ibc.core.client.v1.Query/ConsensusStates";

#[derive(Args, Debug, Clone)]
pub struct CliConsensusStates {
    // TODO: Pagination
    client_id: String,
}

pub(super) fn handle_query(
    args: CliConsensusStates,
    _node: &str,
    _height: Option<Height>,
) -> QueryConsensusStatesRequest {
    QueryConsensusStatesRequest {
        client_id: args.client_id,
        pagination: None,
    }
}

pub(super) fn query_command_handler(
    args: CliConsensusStates,
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
