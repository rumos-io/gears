use clap::Args;

use proto_messages::cosmos::ibc::types::core::client::context::types::proto::v1::QueryConsensusStateRequest;
use tendermint::informal::block::Height;

pub(crate) const CONSENSUS_STATE_URL: &str = "/ibc.core.client.v1.Query/ConsensusState";

#[derive(Args, Debug, Clone)]
pub struct CliConsensusState {
    pub client_id: String,
    pub revision_number: u64,
    pub revision_height: u64,
    #[arg(long)]
    pub latest_height: bool,
}

pub(super) fn handle_query(
    args: CliConsensusState,
    _node: &str,
    _height: Option<Height>,
) -> QueryConsensusStateRequest {
    let CliConsensusState {
        client_id,
        revision_number,
        revision_height,
        latest_height,
    } = args;

    QueryConsensusStateRequest {
        client_id,
        revision_number,
        revision_height,
        latest_height,
    }
}
