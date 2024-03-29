use std::{fs::File, io::Read};

use clap::Args;
use proto_messages::cosmos::ibc::{
    tx::MsgCreateClient,
    types::tendermint::{consensus_state::RawConsensusState, WrappedTendermintClientState},
};

use crate::types::Signer;

/// create new IBC client
#[derive(Args, Debug, Clone)]
pub struct CliCreateClient {
    pub client_state: String,
    pub consensus_state: String,
    pub signer: Signer,
}

pub(super) fn tx_command_handler(msg: CliCreateClient) -> anyhow::Result<crate::message::Message> {
    let CliCreateClient {
        client_state,
        consensus_state,
        signer,
    } = msg;

    let mut buffer = Vec::<u8>::new();

    let client_state_result = serde_json::from_str::<WrappedTendermintClientState>(&client_state);
    dbg!(&client_state_result);
    let client_state = if let Ok(client_state) = client_state_result {
        client_state
    } else {
        File::open(client_state)?.read_to_end(&mut buffer)?;
        serde_json::from_slice(&buffer)?
    };

    let consensus_state_result = serde_json::from_str::<RawConsensusState>(&consensus_state);
    dbg!(&consensus_state_result);
    let consensus_state = if let Ok(consensus_state) = consensus_state_result {
        consensus_state
    } else {
        File::open(consensus_state)?.read_to_end(&mut buffer)?;
        serde_json::from_slice(&buffer)?
    };

    let raw_msg = MsgCreateClient {
        client_state,
        consensus_state,
        signer: proto_messages::cosmos::ibc::types::primitives::Signer::from(signer.0),
    };

    Ok(crate::message::Message::ClientCreate(raw_msg))
}
