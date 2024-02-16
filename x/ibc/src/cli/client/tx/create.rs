use std::{fs::File, io::Read, path::PathBuf};

use clap::Args;
use proto_messages::cosmos::ibc::{
    protobuf::{PrimitiveAny, PrimitiveProtobuf},
    tx::MsgCreateClient,
    types::{RawConsensusState, RawSigner},
};

use crate::types::Signer;

#[derive(Args, Debug)]
pub struct CliCreateClient {
    pub client_state: PathBuf,    //  TODO: User could pass not only file
    pub consensus_state: PathBuf, //  TODO: User could pass not only file
    pub signer: Signer,
}

pub(super) fn tx_command_handler(msg: CliCreateClient) -> anyhow::Result<crate::message::Message> {
    let CliCreateClient {
        client_state,
        consensus_state,
        signer,
    } = msg;
    let mut buffer = Vec::<u8>::new();

    File::open(client_state)?.read_to_end(&mut buffer)?;
    let client_state = <RawConsensusState as PrimitiveProtobuf<PrimitiveAny>>::decode_vec(&buffer)?;
    File::open(consensus_state)?.read_to_end(&mut buffer)?;
    let consensus_state =
        <RawConsensusState as PrimitiveProtobuf<PrimitiveAny>>::decode_vec(&buffer)?;

    let raw_msg = MsgCreateClient {
        client_state,
        consensus_state,
        signer: RawSigner::from(signer.0),
    };

    Ok(crate::message::Message::ClientCreate(raw_msg))
}
