use crate::types::time::timestamp::Timestamp;

use super::validator::Validator;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LastCommitInfo {
    pub round: i32,
    pub votes: Vec<VoteInfo>,
}

impl From<LastCommitInfo> for inner::LastCommitInfo {
    fn from(LastCommitInfo { round, votes }: LastCommitInfo) -> Self {
        Self {
            round,
            votes: votes.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<inner::LastCommitInfo> for LastCommitInfo {
    type Error = crate::error::Error;

    fn try_from(
        inner::LastCommitInfo { round, votes }: inner::LastCommitInfo,
    ) -> Result<Self, Self::Error> {
        let mut votes_res = vec![];
        for v in votes {
            votes_res.push(v.try_into()?);
        }
        Ok(Self {
            round,
            votes: votes_res,
        })
    }
}

/// VoteInfo
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VoteInfo {
    pub validator: Validator,
    pub signed_last_block: bool,
}

impl TryFrom<inner::VoteInfo> for VoteInfo {
    type Error = crate::error::Error;

    fn try_from(
        inner::VoteInfo {
            validator,
            signed_last_block,
        }: inner::VoteInfo,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            validator: validator
                .ok_or(Self::Error::InvalidData("validator is missing".into()))?
                .try_into()
                .map_err(|e| Self::Error::InvalidData(format!("{e}")))?,
            signed_last_block,
        })
    }
}

impl From<VoteInfo> for inner::VoteInfo {
    fn from(
        VoteInfo {
            validator,
            signed_last_block,
        }: VoteInfo,
    ) -> Self {
        Self {
            validator: Some(validator.into()),
            signed_last_block,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Evidence {
    pub r#type: i32,
    /// The offending validator
    pub validator: Validator,
    /// The height when the offense occurred
    pub height: i64,
    /// The corresponding time where the offense occurred
    pub time: Timestamp,
    /// Total voting power of the validator set in case the ABCI application does
    /// not store historical validators.
    pub total_voting_power: i64,
}

impl Evidence {
    pub fn kind(&self) -> EvidenceType {
        EvidenceType::try_from(self.r#type).unwrap_or(EvidenceType::Unknown)
    }
}

impl TryFrom<inner::Evidence> for Evidence {
    type Error = crate::error::Error;

    fn try_from(
        inner::Evidence {
            r#type,
            validator,
            height,
            time,
            total_voting_power,
        }: inner::Evidence,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            r#type,
            validator: validator
                .ok_or(Self::Error::InvalidData("validator is missing".into()))?
                .try_into()
                .map_err(|e| Self::Error::InvalidData(format!("{e}")))?,
            height,
            time: time
                .ok_or(Self::Error::InvalidData("time is missing".into()))?
                .try_into()
                .map_err(|e| Self::Error::InvalidData(format!("{e}")))?,
            total_voting_power,
        })
    }
}

impl From<Evidence> for inner::Evidence {
    fn from(
        Evidence {
            r#type,
            validator,
            height,
            time,
            total_voting_power,
        }: Evidence,
    ) -> Self {
        Self {
            r#type,
            validator: Some(validator.into()),
            height,
            time: Some(time.into()),
            total_voting_power,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum EvidenceType {
    Unknown = 0,
    DuplicateVote = 1,
    LightClientAttack = 2,
}
impl EvidenceType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            EvidenceType::Unknown => "UNKNOWN",
            EvidenceType::DuplicateVote => "DUPLICATE_VOTE",
            EvidenceType::LightClientAttack => "LIGHT_CLIENT_ATTACK",
        }
    }
}

impl From<inner::EvidenceType> for EvidenceType {
    fn from(value: inner::EvidenceType) -> Self {
        match value {
            inner::EvidenceType::Unknown => Self::Unknown,
            inner::EvidenceType::DuplicateVote => Self::DuplicateVote,
            inner::EvidenceType::LightClientAttack => Self::LightClientAttack,
        }
    }
}

impl From<EvidenceType> for inner::EvidenceType {
    fn from(value: EvidenceType) -> Self {
        match value {
            EvidenceType::Unknown => Self::Unknown,
            EvidenceType::DuplicateVote => Self::DuplicateVote,
            EvidenceType::LightClientAttack => Self::LightClientAttack,
        }
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    /// The height at which the snapshot was taken
    #[prost(uint64, tag = "1")]
    pub height: u64,
    /// The application-specific snapshot format
    #[prost(uint32, tag = "2")]
    pub format: u32,
    /// Number of chunks in the snapshot
    #[prost(uint32, tag = "3")]
    pub chunks: u32,
    /// Arbitrary snapshot hash, equal only if identical
    #[prost(bytes = "bytes", tag = "4")]
    pub hash: ::prost::bytes::Bytes,
    /// Arbitrary application metadata
    #[prost(bytes = "bytes", tag = "5")]
    pub metadata: ::prost::bytes::Bytes,
}

impl From<Snapshot> for inner::Snapshot {
    fn from(
        Snapshot {
            height,
            format,
            chunks,
            hash,
            metadata,
        }: Snapshot,
    ) -> Self {
        Self {
            height,
            format,
            chunks,
            hash,
            metadata,
        }
    }
}

impl From<inner::Snapshot> for Snapshot {
    fn from(
        inner::Snapshot {
            height,
            format,
            chunks,
            hash,
            metadata,
        }: inner::Snapshot,
    ) -> Self {
        Self {
            height,
            format,
            chunks,
            hash,
            metadata,
        }
    }
}

pub(crate) mod inner {
    pub use tendermint_proto::abci::Evidence;
    pub use tendermint_proto::abci::EvidenceType;
    pub use tendermint_proto::abci::LastCommitInfo;
    pub use tendermint_proto::abci::Snapshot;
    pub use tendermint_proto::abci::VoteInfo;
}
