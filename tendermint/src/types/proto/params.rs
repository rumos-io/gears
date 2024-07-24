use crate::types::duration::Duration;

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct BlockParams {
    /// Note: must be greater than 0
    #[prost(int64, tag = "1")]
    pub max_bytes: i64,
    /// Note: must be greater or equal to -1
    #[prost(int64, tag = "2")]
    pub max_gas: i64,
}

impl From<BlockParams> for inner::BlockParams {
    fn from(BlockParams { max_bytes, max_gas }: BlockParams) -> Self {
        Self { max_bytes, max_gas }
    }
}

impl From<inner::BlockParams> for BlockParams {
    fn from(inner::BlockParams { max_bytes, max_gas }: inner::BlockParams) -> Self {
        Self { max_bytes, max_gas }
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct EvidenceParams {
    /// Max age of evidence, in blocks.
    ///
    /// The basic formula for calculating this is: MaxAgeDuration / {average block
    /// time}.
    #[prost(int64, tag = "1")]
    pub max_age_num_blocks: i64,
    /// Max age of evidence, in time.
    ///
    /// It should correspond with an app's "unbonding period" or other similar
    /// mechanism for handling [Nothing-At-Stake
    /// attacks](<https://github.com/ethereum/wiki/wiki/Proof-of-Stake-FAQ#what-is-the-nothing-at-stake-problem-and-how-can-it-be-fixed>).
    #[prost(message, optional, tag = "2")]
    pub max_age_duration: Option<Duration>,
    /// This sets the maximum size of total evidence in bytes that can be committed in a single block.
    /// and should fall comfortably under the max block bytes.
    /// Default is 1048576 or 1MB
    #[prost(int64, tag = "3")]
    #[serde(with = "crate::types::serializers::from_str", default)]
    pub max_bytes: i64,
}

impl From<EvidenceParams> for inner::EvidenceParams {
    fn from(
        EvidenceParams {
            max_age_num_blocks,
            max_age_duration,
            max_bytes,
        }: EvidenceParams,
    ) -> Self {
        Self {
            max_age_num_blocks,
            max_age_duration: max_age_duration.map(|e| e.into()),
            max_bytes,
        }
    }
}

impl From<inner::EvidenceParams> for EvidenceParams {
    fn from(
        inner::EvidenceParams {
            max_age_num_blocks,
            max_age_duration,
            max_bytes,
        }: inner::EvidenceParams,
    ) -> Self {
        Self {
            max_age_num_blocks,
            max_age_duration: max_age_duration.map(|e| e.into()),
            max_bytes,
        }
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ValidatorParams {
    #[prost(string, repeated, tag = "1")]
    pub pub_key_types: Vec<String>,
}

impl From<ValidatorParams> for inner::ValidatorParams {
    fn from(ValidatorParams { pub_key_types }: ValidatorParams) -> Self {
        Self { pub_key_types }
    }
}

impl From<inner::ValidatorParams> for ValidatorParams {
    fn from(inner::ValidatorParams { pub_key_types }: inner::ValidatorParams) -> Self {
        Self { pub_key_types }
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct VersionParams {
    #[prost(uint64, tag = "1")]
    pub app_version: u64,
}

impl From<inner::VersionParams> for VersionParams {
    fn from(inner::VersionParams { app_version }: inner::VersionParams) -> Self {
        Self { app_version }
    }
}

impl From<VersionParams> for inner::VersionParams {
    fn from(VersionParams { app_version }: VersionParams) -> Self {
        Self { app_version }
    }
}

pub(crate) mod inner {
    pub use tendermint_proto::abci::BlockParams;
    pub use tendermint_proto::types::EvidenceParams;
    pub use tendermint_proto::types::ValidatorParams;
    pub use tendermint_proto::types::VersionParams;
}
