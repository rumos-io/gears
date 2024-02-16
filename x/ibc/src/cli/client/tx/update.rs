use std::{fs::File, io::Read, str::FromStr};

use clap::Args;
pub use ibc::core::client::types::msgs::MsgUpdateClient as RawMsgUpdateClient;
use ibc::{
    core::host::types::identifiers::ClientId as RawClientId,
    primitives::{proto::Any, Signer as RawSigner},
};
use prost::Message;

use crate::types::{ClientId, Signer};

#[derive(Args, Debug)]
pub struct MsgUpdateClient {
    pub client_id: ClientId,
    pub client_message: String, // TODO: more appriate type
    pub signer: Signer,
}

pub(super) fn tx_command_handler(msg: MsgUpdateClient) -> anyhow::Result<crate::message::Message> {
    let MsgUpdateClient {
        client_id,
        client_message,
        signer,
    } = msg;

    let mut buffer = Vec::<u8>::new();
    File::open(client_message)?.read_to_end(&mut buffer)?;

    let cl_msg = proto_messages::cosmos::ibc::protobuf::Any::decode(buffer.as_slice())?;

    let raw_msg = RawMsgUpdateClient {
        client_id: RawClientId::from_str(&client_id.0)?,
        client_message: Any {
            type_url: cl_msg.type_url,
            value: cl_msg.value,
        },
        signer: RawSigner::from(signer.0),
    };

    Ok(crate::message::Message::ClientUpdate(raw_msg.into()))
}
