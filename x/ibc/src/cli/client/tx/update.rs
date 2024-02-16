use std::{fs::File, io::Read, str::FromStr};

use clap::Args;
use prost::Message;
use proto_messages::cosmos::ibc::{
    tx::MsgUpdateClient,
    types::{RawClientId, RawSigner},
};

use crate::types::{ClientId, Signer};

#[derive(Args, Debug)]
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
    File::open(client_message)?.read_to_end(&mut buffer)?;

    let cl_msg = proto_messages::cosmos::ibc::protobuf::Any::decode(buffer.as_slice())?;

    let raw_msg = MsgUpdateClient {
        client_id: RawClientId::from_str(&client_id.0)?,
        client_message: cl_msg,
        signer: RawSigner::from(signer.0),
    };

    Ok(crate::message::Message::ClientUpdate(raw_msg.into()))
}
