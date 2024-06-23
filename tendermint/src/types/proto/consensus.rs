use super::params::{BlockParams, EvidenceParams, ValidatorParams, VersionParams};

/// ConsensusParams contains all consensus-relevant parameters
/// that can be adjusted by the abci app
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
pub struct ConsensusParams {
    pub block: BlockParams,
    pub evidence: EvidenceParams,
    pub validator: ValidatorParams,
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
            block: Some(block.into()),
            evidence: Some(evidence.into()),
            validator: Some(validator.into()),
            version: version.map(Into::into),
        }
    }
}

impl TryFrom<inner::ConsensusParams> for ConsensusParams {
    type Error = crate::error::Error;

    fn try_from(
        inner::ConsensusParams {
            block,
            evidence,
            validator,
            version,
        }: inner::ConsensusParams,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            block: block
                .ok_or_else(|| Self::Error::InvalidData("block params is missing".into()))?
                .into(),
            evidence: evidence
                .ok_or_else(|| Self::Error::InvalidData("evidence params is missing".into()))?
                .into(),
            validator: validator
                .ok_or_else(|| Self::Error::InvalidData("validator params is missing".into()))?
                .into(),
            version: version.map(Into::into),
        })
    }
}

/// Consensus captures the consensus rules for processing a block in the blockchain,
/// including all blockchain data structures and the rules of the application's
/// state transition machine.
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
pub struct Consensus {
    #[serde(with = "crate::types::serializers::from_str")]
    pub block: u64,
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
