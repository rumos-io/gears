use ibc::core::client::context::client_state::ClientStateCommon;
use ibc::primitives::proto::Any;
pub use ibc::{
    core::host::types::identifiers::ClientId as RawClientId,
    primitives::Signer as RawSigner,
};
pub use ibc::core::host::types::error::IdentifierError;
pub use ibc::core::commitment_types::commitment::CommitmentProofBytes as RawCommitmentProofBytes;
pub use ibc::clients::tendermint::consensus_state::ConsensusState as RawConsensusState;

pub use ibc::clients::tendermint::client_state::ClientState as RawClientState;
pub use ibc::core::host::types::identifiers::ClientType;
pub use ibc::core::client::types::error::ClientError;
pub use ibc::core::client::context::*;
pub use ibc_proto::ibc::core::client::v1::*;

pub use ibc::primitives::Timestamp;
pub use ibc::core::handler::types::error::ContextError;
pub use ibc::core::host::types::*;

pub use ibc::core::commitment_types::commitment::*;

pub use tendermint::informal::abci::{Event, EventAttribute};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConsensusState( pub RawConsensusState );

impl From<RawConsensusState> for ConsensusState
{
    fn from(value: RawConsensusState) -> Self {
        Self(value)
    }
}

impl ClientStateCommon for ConsensusState
{
    fn verify_consensus_state(&self, _consensus_state: ibc::primitives::proto::Any) -> Result<(), ClientError> {
        todo!()
    }

    fn client_type(&self) -> ClientType {
        todo!() // TODO: impl it
    }

    fn latest_height(&self) -> types::Height {
        todo!() // TODO: impl it
    }

    fn validate_proof_height(&self, _proof_height: types::Height) -> Result<(), ClientError> {
        todo!()
    }

    fn verify_upgrade_client(
        &self,
        _upgraded_client_state: ibc::primitives::proto::Any,
        _upgraded_consensus_state: ibc::primitives::proto::Any,
        _proof_upgrade_client: RawCommitmentProofBytes,
        _proof_upgrade_consensus_state: RawCommitmentProofBytes,
        _root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        todo!()
    }

    fn verify_membership(
        &self,
        _prefix: &CommitmentPrefix,
        _proof: &RawCommitmentProofBytes,
        _root: &CommitmentRoot,
        _path: path::Path,
        _value: Vec<u8>,
    ) -> Result<(), ClientError> {
        todo!()
    }

    fn verify_non_membership(
        &self,
        _prefix: &CommitmentPrefix,
        _proof: &RawCommitmentProofBytes,
        _root: &CommitmentRoot,
        _path: path::Path,
    ) -> Result<(), ClientError> {
        todo!()
    }
}

impl TryFrom<Any> for ConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        Ok(Self(RawConsensusState::try_from(raw)?))
    }
}

impl From<ConsensusState> for Any {
    fn from(client_state: ConsensusState) -> Self {
        client_state.0.into()
    }
}