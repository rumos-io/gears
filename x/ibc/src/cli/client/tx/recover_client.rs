use std::str::FromStr;

use clap::Args;
use proto_messages::cosmos::ibc::{tx::MsgRecoverClient, types::core::host::identifiers::ClientId};

use crate::types::Signer;

#[derive(Args, Debug)]
pub struct CliRecoverClient {
    pub subject_client_id: String,
    pub substitute_client_id: String,
    pub signer: Signer,
}

pub(super) fn tx_command_handler(msg: CliRecoverClient) -> anyhow::Result<crate::message::Message> {
    let CliRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer,
    } = msg;

    let raw_msg = MsgRecoverClient {
        subject_client_id: ClientId::from_str(&subject_client_id)?,
        substitute_client_id: ClientId::from_str(&substitute_client_id)?,
        signer: proto_messages::cosmos::ibc::types::primitives::Signer::from(signer.0),
    };

    Ok(crate::message::Message::RecoverClient(raw_msg))
}
