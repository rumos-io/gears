use super::params::{BlockParams, EvidenceParams, ValidatorParams, VersionParams};

/// ConsensusParams contains all consensus-relevant parameters
/// that can be adjusted by the abci app
#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ConsensusParams {
    #[prost(message, optional, tag = "1")]
    pub block: Option<BlockParams>,
    #[prost(message, optional, tag = "2")]
    pub evidence: Option<EvidenceParams>,
    #[prost(message, optional, tag = "3")]
    pub validator: Option<ValidatorParams>,
    #[prost(message, optional, tag = "4")]
    pub version: Option<VersionParams>,
}

impl From<ConsensusParams> for inner::ConsensusParams {
    fn from(
        ConsensusParams {
            block,
            evidence,
            validator,
            version,
        }: ConsensusParams,
    ) -> Self {
        Self {
            block: block.map(Into::into),
            evidence: evidence.map(Into::into),
            validator: validator.map(Into::into),
            version: version.map(Into::into),
        }
    }
}

impl From<inner::ConsensusParams> for ConsensusParams {
    fn from(
        inner::ConsensusParams {
            block,
            evidence,
            validator,
            version,
        }: inner::ConsensusParams,
    ) -> Self {
        Self {
            block: block.map(Into::into),
            evidence: evidence.map(Into::into),
            validator: validator.map(Into::into),
            version: version.map(Into::into),
        }
    }
}

/// Consensus captures the consensus rules for processing a block in the blockchain,
/// including all blockchain data structures and the rules of the application's
/// state transition machine.
#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct Consensus {
    #[prost(uint64, tag = "1")]
    #[serde(with = "crate::types::serializers::from_str")]
    pub block: u64,
    #[prost(uint64, tag = "2")]
    #[serde(with = "crate::types::serializers::from_str", default)]
    pub app: u64,
}

impl From<Consensus> for inner::Consensus {
    fn from(Consensus { block, app }: Consensus) -> Self {
        Self { block, app }
    }
}

impl From<inner::Consensus> for Consensus {
    fn from(inner::Consensus { block, app }: inner::Consensus) -> Self {
        Self { block, app }
    }
}

pub(crate) mod inner {
    pub use tendermint_proto::abci::ConsensusParams;
    pub use tendermint_proto::version::Consensus;
}
