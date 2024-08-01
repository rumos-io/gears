use crate::types::{chain_id::ChainId, time::timestamp::Timestamp};

use super::{block::BlockId, consensus::Consensus};

/// Header defines the structure of a Tendermint block header.
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
pub struct Header {
    /// basic block info
    pub version: Consensus,
    pub chain_id: ChainId,
    #[serde(with = "crate::types::serializers::from_str")]
    pub height: u32,
    pub time: Timestamp,
    /// prev block info
    pub last_block_id: BlockId,
    /// hashes of block data
    ///
    /// commit from validators from the last block
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub last_commit_hash: Vec<u8>,
    /// transactions
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub data_hash: Vec<u8>,
    /// hashes from the app output from the prev block
    ///
    /// validators for the current block
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub validators_hash: Vec<u8>,
    /// validators for the next block
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub next_validators_hash: Vec<u8>,
    /// consensus params for current block
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub consensus_hash: Vec<u8>,
    /// state after txs from the previous block
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub app_hash: Vec<u8>,
    /// root hash of all results from the txs from the previous block
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub last_results_hash: Vec<u8>,
    /// consensus info
    ///
    /// evidence included in the block
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub evidence_hash: Vec<u8>,
    /// original proposer of the block
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub proposer_address: Vec<u8>,
}

impl From<Header> for inner::Header {
    fn from(
        Header {
            version,
            chain_id,
            height,
            time,
            last_block_id,
            last_commit_hash,
            data_hash,
            validators_hash,
            next_validators_hash,
            consensus_hash,
            app_hash,
            last_results_hash,
            evidence_hash,
            proposer_address,
        }: Header,
    ) -> Self {
        Self {
            version: Some(version.into()),
            chain_id: chain_id.to_string(),
            height: height.into(),
            time: Some(time.into()),
            last_block_id: Some(last_block_id.into()),
            last_commit_hash,
            data_hash,
            validators_hash,
            next_validators_hash,
            consensus_hash,
            app_hash,
            last_results_hash,
            evidence_hash,
            proposer_address,
        }
    }
}

impl TryFrom<inner::Header> for Header {
    type Error = crate::error::Error;

    fn try_from(
        inner::Header {
            version,
            chain_id,
            height,
            time,
            last_block_id,
            last_commit_hash,
            data_hash,
            validators_hash,
            next_validators_hash,
            consensus_hash,
            app_hash,
            last_results_hash,
            evidence_hash,
            proposer_address,
        }: inner::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            version: version
                .ok_or_else(|| Self::Error::InvalidData("version is missing".into()))?
                .into(),
            chain_id: chain_id
                .parse()
                .map_err(|e| Self::Error::InvalidData(format!("invalid chain_id: {e}")))?,
            height: height.try_into().map_err(|_| {
                Self::Error::InvalidData(format!("provided height, {height}, is less than zero"))
            })?,
            time: time
                .ok_or_else(|| Self::Error::InvalidData("time is missing".into()))?
                .try_into()
                .map_err(|e| Self::Error::InvalidData(format!("{e}")))?,
            last_block_id: last_block_id
                .ok_or_else(|| Self::Error::InvalidData("last_block_id is missing".into()))?
                .into(),

            last_commit_hash,
            data_hash,
            validators_hash,
            next_validators_hash,
            consensus_hash,
            app_hash,
            last_results_hash,
            evidence_hash,
            proposer_address,
        })
    }
}

/// PartsetHeader
#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct PartSetHeader {
    #[prost(uint32, tag = "1")]
    #[serde(with = "crate::types::serializers::part_set")]
    pub total: u32,
    #[prost(bytes = "vec", tag = "2")]
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub hash: Vec<u8>,
}

impl From<inner::PartSetHeader> for PartSetHeader {
    fn from(inner::PartSetHeader { total, hash }: inner::PartSetHeader) -> Self {
        Self { total, hash }
    }
}

impl From<PartSetHeader> for inner::PartSetHeader {
    fn from(PartSetHeader { total, hash }: PartSetHeader) -> Self {
        Self { total, hash }
    }
}

pub(crate) mod inner {
    pub use tendermint_proto::types::Header;
    pub use tendermint_proto::types::PartSetHeader;
}
