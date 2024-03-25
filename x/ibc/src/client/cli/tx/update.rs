use std::{fs::File, io::Read, str::FromStr};

use clap::Args;
use prost::Message;
use proto_messages::{any::Any, cosmos::ibc::tx::MsgUpdateClient};

use crate::types::{ClientId, Signer};

#[derive(Args, Debug, Clone)]
pub struct CliUpdateClient {
    pub client_id: ClientId,
    pub client_message: String, // TODO: more appriate type
    pub signer: Signer,
}

pub(super) fn tx_command_handler(msg: CliUpdateClient) -> anyhow::Result<crate::message::Message> {
    let CliUpdateClient {
        client_id,
        client_message,
        signer,
    } = msg;

    let mut buffer = Vec::<u8>::new();

    let client_message_result = serde_json::from_str::<Any>(&client_message);
    let client_message = if let Ok(client_message) = client_message_result {
        client_message
    } else {
        File::open(client_message)?.read_to_end(&mut buffer)?;
        Any::decode(buffer.as_slice())? // TODO: Should decode as protobuf or with serde?
    };

    let raw_msg = MsgUpdateClient {
        client_id: proto_messages::cosmos::ibc::types::core::host::identifiers::ClientId::from_str(
            &client_id.0,
        )?,
        client_message,
        signer: proto_messages::cosmos::ibc::types::primitives::Signer::from(signer.0),
    };

    Ok(crate::message::Message::ClientUpdate(raw_msg))
}
