use clap::Args;
pub use ibc::core::client::types::msgs::MsgUpdateClient as RawMsgUpdateClient;

use crate::types::{ClientId, Signer};

#[derive(Args, Debug)]
pub struct MsgUpdateClient {
    pub client_id: ClientId,
    pub client_message: String, // TODO: more appriate type
    pub signer: Signer,
}

pub(super) fn tx_command_handler(_msg: MsgUpdateClient) {
    unimplemented!()
}
