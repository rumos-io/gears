use clap::Args;
use ibc::{
    core::{client::types::proto::v1::MsgRecoverClient, host::types::identifiers::ClientId},
    primitives::Signer,
};
//use proto_messages::cosmos::ibc::{tx::MsgRecoverClient, types::core::host::identifiers::ClientId};

#[derive(Args, Debug, Clone)]
pub struct CliRecoverClient {
    pub subject_client_id: ClientId,
    pub substitute_client_id: ClientId,
    pub signer: Signer,
}

pub(super) fn tx_command_handler(msg: CliRecoverClient) -> anyhow::Result<crate::message::Message> {
    let CliRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer,
    } = msg;

    let raw_msg = MsgRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer,
    };

    Ok(crate::message::Message::RecoverClient(raw_msg))
}
