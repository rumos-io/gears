use std::{fs::File, io::Read, str::FromStr};

use clap::Args;
pub use ibc::core::client::types::msgs::MsgUpgradeClient as RawMsgUpgradeClient;
use ibc::{
    core::{
        commitment_types::commitment::CommitmentProofBytes,
        host::types::identifiers::ClientId as RawClientId,
    },
    primitives::Signer as RawSigner,
};
use prost::Message;
use proto_messages::cosmos::ibc::protobuf::Any;

use crate::types::{ClientId, Signer};

#[derive(Args, Debug)]
pub struct MsgUpgradeClient {
    pub client_id: ClientId,
    pub upgraded_client_state: String,
    pub upgraded_consensus_state: String,
    pub proof_upgrade_client: String,
    pub proof_upgrade_consensus_state: String,
    pub signer: Signer,
}

pub(super) fn tx_command_handler(msg: MsgUpgradeClient) -> anyhow::Result<crate::message::Message> {
    let MsgUpgradeClient {
        client_id,
        upgraded_client_state,
        upgraded_consensus_state,
        proof_upgrade_client,
        proof_upgrade_consensus_state,
        signer,
    } = msg;

    let mut buffer = Vec::<u8>::new();

    File::open(upgraded_client_state)?.read_to_end(&mut buffer)?;
    let upgraded_client_state = {
        let Any { type_url, value } = Any::decode(buffer.as_slice())?;

        ibc::primitives::proto::Any { type_url, value }
    };

    File::open(upgraded_consensus_state)?.read_to_end(&mut buffer)?;
    let upgraded_consensus_state = {
        let Any { type_url, value } = Any::decode(buffer.as_slice())?;

        ibc::primitives::proto::Any { type_url, value }
    };

    let raw_msg = RawMsgUpgradeClient {
        client_id: RawClientId::from_str(&client_id.0)?,
        upgraded_client_state,
        upgraded_consensus_state,
        proof_upgrade_client: CommitmentProofBytes::try_from(proof_upgrade_client.into_bytes())?,
        proof_upgrade_consensus_state: CommitmentProofBytes::try_from(
            proof_upgrade_consensus_state.into_bytes(),
        )?,
        signer: RawSigner::from(signer.0),
    };

    Ok(crate::message::Message::ClientUpgrade(raw_msg.into()))
}
