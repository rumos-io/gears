use bytes::Bytes;
use gears::{
    core::{address::AccAddress, any::google::Any},
    types::tx::TxMessage,
};
use ibc::{
    clients::tendermint::{client_state::ClientState, consensus_state::ConsensusState},
    core::client::types::{error::ClientError, proto::v1::MsgCreateClient as RawMsgCreateClient},
    primitives::proto::Protobuf,
};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MsgCreateClient {
    pub client_state: ClientState,
    pub consensus_state: ConsensusState,
    pub signer: AccAddress,
}

impl MsgCreateClient {
    pub fn new(
        client_state: ClientState,
        consensus_state: ConsensusState,
        signer: AccAddress,
    ) -> Self {
        MsgCreateClient {
            client_state,
            consensus_state,
            signer,
        }
    }
}

// TODO: fill in the gaps below
impl TxMessage for MsgCreateClient {
    fn get_signers(&self) -> Vec<&AccAddress> {
        vec![&self.signer]
    }

    fn validate_basic(&self) -> Result<(), String> {
        //TODO: implement this
        Ok(())
    }

    fn type_url(&self) -> &'static str {
        "/ibc.core.client.v1.MsgCreateClient"
    }
}

impl From<MsgCreateClient> for Any {
    fn from(msg: MsgCreateClient) -> Self {
        Any {
            type_url: msg.type_url().to_string(),
            value: msg.encode_vec(),
        }
    }
}

impl TryFrom<Any> for MsgCreateClient {
    type Error = gears::core::errors::Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "/ibc.core.client.v1.MsgCreateClient" => {
                MsgCreateClient::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::Error::DecodeProtobuf(e.to_string()))
            }
            _ => Err(gears::core::errors::Error::DecodeGeneral(
                "message type not recognized".into(),
            )),
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

        let signer =
            AccAddress::from_bech32(&raw.signer).map_err(|e| ClientError::InvalidSigner {
                reason: e.to_string(),
            })?;

        Ok(MsgCreateClient::new(
            raw_client_state.try_into()?,
            raw_consensus_state.try_into()?,
            signer,
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
