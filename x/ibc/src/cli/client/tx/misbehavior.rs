use std::{fs::File, io::Read, str::FromStr};

use clap::Args;
use prost::Message;
use proto_messages::cosmos::ibc::{
    protobuf::Any,
    tx::MsgSubmitMisbehaviour,
    types::{RawClientId, RawSigner},
};

use crate::types::{ClientId, Signer};

#[derive(Args, Debug)]
pub struct CliSubmitMisbehaviour {
    pub client_id: ClientId,
    pub misbehaviour: String,
    pub signer: Signer,
}

pub(super) fn tx_command_handler(
    msg: CliSubmitMisbehaviour,
) -> anyhow::Result<crate::message::Message> {
    let CliSubmitMisbehaviour {
        client_id,
        misbehaviour,
        signer,
    } = msg;

    let mut buffer = Vec::<u8>::new();

    File::open(misbehaviour)?.read_to_end(&mut buffer)?;
    let misbehaviour = Any::decode(buffer.as_slice())?;

    let raw_msg = MsgSubmitMisbehaviour {
        client_id: RawClientId::from_str(&client_id.0)?,
        misbehaviour,
        signer: RawSigner::from(signer.0),
    };

    Ok(crate::message::Message::SubmitMisbehaviour(raw_msg.into()))
}
