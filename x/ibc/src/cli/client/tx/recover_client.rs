use clap::Args;
pub use ibc::core::client::types::proto::v1::MsgRecoverClient as RawMsgRecoverClient;

use crate::types::Signer;

#[derive(Args, Debug)]
pub struct MsgRecoverClient {
    pub subject_client_id: String,
    pub substitute_client_id: String,
    pub signer: Signer,
}

pub(super) fn tx_command_handler(msg: MsgRecoverClient) -> anyhow::Result<crate::message::Message> {
    let MsgRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer,
    } = msg;

    let raw_msg = RawMsgRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer: signer.0,
    };

    Ok(crate::message::Message::RecoverClient(raw_msg))
}
