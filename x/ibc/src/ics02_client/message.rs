use ibc::{
    clients::tendermint::{client_state::ClientState, consensus_state::ConsensusState},
    core::client::types::{
        error::ClientError,
        //      msgs::{MsgUpdateClient, MsgUpgradeClient},
        proto::v1::MsgCreateClient as RawMsgCreateClient, //, MsgRecoverClient},
    },
    primitives::{proto::Protobuf, Signer},
};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MsgCreateClient {
    pub client_state: ClientState,
    pub consensus_state: ConsensusState,
    pub signer: Signer,
}

impl MsgCreateClient {
    pub fn new(client_state: ClientState, consensus_state: ConsensusState, signer: Signer) -> Self {
        MsgCreateClient {
            client_state,
            consensus_state,
            signer,
        }
    }
}

impl Protobuf<RawMsgCreateClient> for MsgCreateClient {}

impl TryFrom<RawMsgCreateClient> for MsgCreateClient {
    type Error = ClientError;

    fn try_from(raw: RawMsgCreateClient) -> Result<Self, Self::Error> {
        let raw_client_state = raw.client_state.ok_or(ClientError::MissingRawClientState)?;

        let raw_consensus_state = raw
            .consensus_state
            .ok_or(ClientError::MissingRawConsensusState)?;

        Ok(MsgCreateClient::new(
            raw_client_state.try_into()?,
            raw_consensus_state.try_into()?,
            raw.signer.into(),
        ))
    }
}

impl From<MsgCreateClient> for RawMsgCreateClient {
    fn from(ics_msg: MsgCreateClient) -> Self {
        RawMsgCreateClient {
            client_state: Some(ics_msg.client_state.into()),
            consensus_state: Some(ics_msg.consensus_state.into()),
            signer: ics_msg.signer.to_string(),
        }
    }
}
