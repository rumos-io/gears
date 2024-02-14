use clap::Args;
pub use ibc::core::client::types::msgs::MsgUpgradeClient as RawMsgUpgradeClient;

use crate::types::{ClientId, Signer};

#[derive(Args, Debug)]
pub struct MsgUpgradeClient {
    pub client_id: ClientId,
    pub upgraded_client_state: String,
    pub upgraded_consensus_state: String,
    pub proof_upgrade_client: String,
    pub proof_upgrade_consensus_state: String,
    pub signer: Signer,
}

pub(super) fn tx_command_handler(_msg: MsgUpgradeClient) {
    unimplemented!()
}
