use std::{fs::File, io::Read, str::FromStr};

use clap::Args;
pub use ibc::core::client::types::msgs::MsgSubmitMisbehaviour as RawMsgSubmitMisbehaviour;
use ibc::{
    core::host::types::identifiers::ClientId as RawClientId, primitives::Signer as RawSigner,
};
use prost::Message;
use proto_messages::cosmos::ibc::protobuf::Any;

use crate::types::{ClientId, Signer};

#[derive(Args, Debug)]
pub struct MsgSubmitMisbehaviour {
    pub client_id: ClientId,
    pub misbehaviour: String,
    pub signer: Signer,
}

pub(super) fn tx_command_handler(
    msg: MsgSubmitMisbehaviour,
) -> anyhow::Result<crate::message::Message> {
    let MsgSubmitMisbehaviour {
        client_id,
        misbehaviour,
        signer,
    } = msg;

    let mut buffer = Vec::<u8>::new();
    File::open(misbehaviour)?.read_to_end(&mut buffer)?;

    let Any { type_url, value } = Any::decode(buffer.as_slice())?;

    let raw_msg = RawMsgSubmitMisbehaviour {
        client_id: RawClientId::from_str(&client_id.0)?,
        misbehaviour: ibc::primitives::proto::Any { type_url, value },
        signer: RawSigner::from(signer.0),
    };

    Ok(crate::message::Message::SubmitMisbehaviour(raw_msg.into()))
}
