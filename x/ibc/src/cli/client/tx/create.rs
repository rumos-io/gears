use std::{fs::File, io::Read, path::PathBuf};

use clap::Args;
pub use ibc::core::client::types::msgs::MsgCreateClient as RawMsgCreateClient;
use ibc::{
    clients::tendermint::consensus_state::ConsensusState,
    primitives::{
        proto::{Any, Protobuf},
        Signer as RawSigner,
    },
};

use crate::types::Signer;

#[derive(Args, Debug)]
pub struct MsgCreateClient {
    pub client_state: PathBuf,
    pub consensus_state: PathBuf,
    pub signer: Signer,
}

pub(super) fn tx_command_handler(msg: MsgCreateClient) -> anyhow::Result<crate::message::Message> {
    let MsgCreateClient {
        client_state,
        consensus_state,
        signer,
    } = msg;
    let mut buffer = Vec::<u8>::new();

    File::open(client_state)?.read_to_end(&mut buffer)?;
    let client_state = <ConsensusState as Protobuf<Any>>::decode_vec(&buffer)?;
    File::open(consensus_state)?.read_to_end(&mut buffer)?;
    let consensus_state = <ConsensusState as Protobuf<Any>>::decode_vec(&buffer)?;

    let raw_msg = RawMsgCreateClient::new(
        client_state.into(),
        consensus_state.into(),
        RawSigner::from(signer.0),
    );

    Ok(crate::message::Message::ClientCreate(raw_msg.into()))
}
