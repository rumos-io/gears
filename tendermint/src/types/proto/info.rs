use crate::types::time::Timestamp;

use super::validator::Validator;

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct LastCommitInfo {
    #[prost(int32, tag = "1")]
    pub round: i32,
    #[prost(message, repeated, tag = "2")]
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

impl From<inner::LastCommitInfo> for LastCommitInfo {
    fn from(inner::LastCommitInfo { round, votes }: inner::LastCommitInfo) -> Self {
        Self {
            round,
            votes: votes.into_iter().map(Into::into).collect(),
        }
    }
}

/// VoteInfo
#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct VoteInfo {
    #[prost(message, optional, tag = "1")]
    pub validator: Option<Validator>,
    #[prost(bool, tag = "2")]
    pub signed_last_block: bool,
}

impl From<inner::VoteInfo> for VoteInfo {
    fn from(
        inner::VoteInfo {
            validator,
            signed_last_block,
        }: inner::VoteInfo,
    ) -> Self {
        Self {
            validator: validator.map(Into::into),
            signed_last_block,
        }
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
            validator: validator.map(Into::into),
            signed_last_block,
        }
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct Evidence {
    #[prost(enumeration = "EvidenceType", tag = "1")]
    pub r#type: i32,
    /// The offending validator
    #[prost(message, optional, tag = "2")]
    pub validator: Option<Validator>,
    /// The height when the offense occurred
    #[prost(int64, tag = "3")]
    pub height: i64,
    /// The corresponding time where the offense occurred
    #[prost(message, optional, tag = "4")]
    pub time: Option<Timestamp>,
    /// Total voting power of the validator set in case the ABCI application does
    /// not store historical validators.
    #[prost(int64, tag = "5")]
    pub total_voting_power: i64,
}

impl From<inner::Evidence> for Evidence {
    fn from(
        inner::Evidence {
            r#type,
            validator,
            height,
            time,
            total_voting_power,
        }: inner::Evidence,
    ) -> Self {
        Self {
            r#type,
            validator: validator.map(Into::into),
            height,
            time: time.map(Into::into),
            total_voting_power,
        }
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
            validator: validator.map(Into::into),
            height,
            time: time.map(Into::into),
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

pub mod inner {
    pub use tendermint_proto::abci::Evidence;
    pub use tendermint_proto::abci::EvidenceType;
    pub use tendermint_proto::abci::LastCommitInfo;
    pub use tendermint_proto::abci::Snapshot;
    pub use tendermint_proto::abci::VoteInfo;
}
