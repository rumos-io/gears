use std::path::PathBuf;

use clap::Args;
pub use ibc::core::client::types::msgs::MsgCreateClient as RawMsgCreateClient;
use ibc::primitives::Signer;

#[derive(Args, Debug)]
pub struct MsgCreateClient {
    pub client_state: PathBuf,
    pub consensus_state: PathBuf,
    pub signer: Signer,
}

pub(super) fn tx_command_handler(_msg: MsgCreateClient) {
    unimplemented!()
}
