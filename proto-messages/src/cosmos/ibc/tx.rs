use std::str::FromStr;

use ibc::core::client::types::error::ClientError;
use ibc::core::commitment_types::commitment::CommitmentProofBytes;
pub use ibc_proto::cosmos::tx::v1beta1::SignDoc;
pub use ibc_proto::cosmos::tx::v1beta1::TxRaw;

pub use ibc_proto::cosmos::tx::v1beta1::{
    mode_info::{Single, Sum},
    ModeInfo,
};

use crate::any::Any;
use crate::cosmos::ibc::types::tendermint::{
    consensus_state::{RawConsensusState, WrappedConsensusState},
    WrappedTendermintClientState,
};

use ibc::core::host::types::identifiers::ClientId as RawClientId;
use ibc::primitives::Signer as RawSigner;

pub use ibc::core::client::types::msgs::MsgCreateClient as RawMsgCreateClient;
pub use ibc::core::client::types::msgs::MsgSubmitMisbehaviour as RawMsgSubmitMisbehaviour;
pub use ibc::core::client::types::msgs::MsgUpdateClient as RawMsgUpdateClient;
pub use ibc::core::client::types::msgs::MsgUpgradeClient as RawMsgUpgradeClient;
pub use ibc::core::client::types::proto::v1::MsgCreateClient as RawProtoMsgCreateClient;
pub use ibc::core::client::types::proto::v1::MsgRecoverClient as RawProtoMsgRecoverClient;
pub use ibc::core::client::types::proto::v1::MsgSubmitMisbehaviour as RawProtoMsgSubmitMisbehaviour;
pub use ibc::core::client::types::proto::v1::MsgUpdateClient as RawProtoMsgUpdateClient;
pub use ibc_proto::ibc::core::client::v1::MsgUpgradeClient as RawProtoMsgUpgradeClient;

pub use ibc::core::client::types::msgs::{
    CREATE_CLIENT_TYPE_URL, SUBMIT_MISBEHAVIOUR_TYPE_URL, UPDATE_CLIENT_TYPE_URL,
    UPGRADE_CLIENT_TYPE_URL,
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MsgUpgradeClient {
    pub client_id: RawClientId,
    pub upgraded_client_state: WrappedTendermintClientState,
    pub upgraded_consensus_state: WrappedConsensusState,
    pub proof_upgrade_client: CommitmentProofBytes,
    pub proof_upgrade_consensus_state: CommitmentProofBytes,
    pub signer: RawSigner, // TODO: Is validation required?
}

impl TryFrom<RawMsgUpgradeClient> for MsgUpgradeClient {
    type Error = ClientError;

    fn try_from(value: RawMsgUpgradeClient) -> Result<Self, Self::Error> {
        let RawMsgUpgradeClient {
            client_id,
            upgraded_client_state,
            upgraded_consensus_state,
            proof_upgrade_client,
            proof_upgrade_consensus_state,
            signer,
        } = value;

        Ok(Self {
            client_id,
            upgraded_client_state: upgraded_client_state.try_into()?,
            upgraded_consensus_state: upgraded_consensus_state.try_into()?,
            proof_upgrade_client,
            proof_upgrade_consensus_state,
            signer,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MsgUpdateClient {
    pub client_id: RawClientId,
    pub client_message: Any, // TODO: Concrete validated type?
    pub signer: RawSigner,   // TODO: Is validation required?
}

impl From<RawMsgUpdateClient> for MsgUpdateClient {
    fn from(value: RawMsgUpdateClient) -> Self {
        let RawMsgUpdateClient {
            client_id,
            client_message,
            signer,
        } = value;

        Self {
            client_id,
            client_message: client_message.into(),
            signer,
        }
    }
}

pub const RECOVER_CLIENT_TYPE_URL: &str = "ibc.core.client.v1.MsgRecoverClient";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MsgRecoverClient {
    pub subject_client_id: RawClientId,
    pub substitute_client_id: RawClientId,
    pub signer: RawSigner, // TODO: Is validation required?
}

impl TryFrom<RawProtoMsgRecoverClient> for MsgRecoverClient {
    type Error = ibc::core::host::types::error::IdentifierError;

    fn try_from(value: RawProtoMsgRecoverClient) -> Result<Self, Self::Error> {
        let RawProtoMsgRecoverClient {
            subject_client_id,
            substitute_client_id,
            signer,
        } = value;

        Ok(Self {
            subject_client_id: RawClientId::from_str(&subject_client_id)?,
            substitute_client_id: RawClientId::from_str(&substitute_client_id)?,
            signer: RawSigner::from(signer),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MsgSubmitMisbehaviour {
    pub client_id: RawClientId,
    pub misbehaviour: Any, // TODO: Concrete validated type?
    pub signer: RawSigner, // TODO: Is validation required?
}

impl From<RawMsgSubmitMisbehaviour> for MsgSubmitMisbehaviour {
    fn from(value: RawMsgSubmitMisbehaviour) -> Self {
        let RawMsgSubmitMisbehaviour {
            client_id,
            misbehaviour,
            signer,
        } = value;

        Self {
            client_id,
            misbehaviour: Any {
                type_url: misbehaviour.type_url,
                value: misbehaviour.value,
            },
            signer,
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MsgCreateClient {
    pub client_state: ibc::clients::tendermint::client_state::ClientState,
    pub consensus_state: RawConsensusState,
    pub signer: RawSigner, // TODO: Is validation required?
}

impl TryFrom<RawMsgCreateClient> for MsgCreateClient {
    type Error = ibc::core::client::types::error::ClientError;

    fn try_from(value: RawMsgCreateClient) -> Result<Self, Self::Error> {
        let RawMsgCreateClient {
            client_state,
            consensus_state,
            signer,
        } = value;

        Ok(Self {
            client_state: client_state.try_into()?,
            consensus_state: consensus_state.try_into()?,
            signer,
        })
    }
}
