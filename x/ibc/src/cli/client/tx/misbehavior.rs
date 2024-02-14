use clap::Args;
pub use ibc::core::client::types::msgs::MsgSubmitMisbehaviour as RawMsgSubmitMisbehaviour;

use crate::types::{ClientId, Signer};

#[derive(Args, Debug)]
pub struct MsgSubmitMisbehaviour {
    pub client_id: ClientId,
    /// misbehaviour used for freezing the light client
    pub misbehaviour: String,
    pub signer: Signer,
}

pub(super) fn tx_command_handler(
    _msg: MsgSubmitMisbehaviour,
) -> anyhow::Result<crate::message::Message> {
    unimplemented!()
}
