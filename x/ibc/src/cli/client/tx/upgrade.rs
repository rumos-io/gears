use std::{fs::File, io::Read, str::FromStr};

use clap::Args;
use prost::Message;
use proto_messages::cosmos::ibc::{
    protobuf::Any,
    tx::MsgUpgradeClient,
    types::{RawClientId, RawCommitmentProofBytes, RawSigner},
};

use crate::types::{ClientId, Signer};

#[derive(Args, Debug)]
pub struct CliUpgradeClient {
    pub client_id: ClientId,
    pub upgraded_client_state: String,
    pub upgraded_consensus_state: String,
    pub proof_upgrade_client: String,
    pub proof_upgrade_consensus_state: String,
    pub signer: Signer,
}

pub(super) fn tx_command_handler(msg: CliUpgradeClient) -> anyhow::Result<crate::message::Message> {
    let CliUpgradeClient {
        client_id,
        upgraded_client_state,
        upgraded_consensus_state,
        proof_upgrade_client,
        proof_upgrade_consensus_state,
        signer,
    } = msg;

    let mut buffer = Vec::<u8>::new();

    File::open(upgraded_client_state)?.read_to_end(&mut buffer)?;
    let upgraded_client_state = Any::decode(buffer.as_slice())?;

    File::open(upgraded_consensus_state)?.read_to_end(&mut buffer)?;
    let upgraded_consensus_state = Any::decode(buffer.as_slice())?;

    let raw_msg = MsgUpgradeClient {
        client_id: RawClientId::from_str(&client_id.0)?,
        upgraded_client_state,
        upgraded_consensus_state,
        proof_upgrade_client: RawCommitmentProofBytes::try_from(proof_upgrade_client.into_bytes())?,
        proof_upgrade_consensus_state: RawCommitmentProofBytes::try_from(
            proof_upgrade_consensus_state.into_bytes(),
        )?,
        signer: RawSigner::from(signer.0),
    };

    Ok(crate::message::Message::ClientUpgrade(raw_msg.into()))
}
