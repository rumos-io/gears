use crate::types::time::Timestamp;

use super::{block_id::BlockId, consensus::Consensus};

/// Header defines the structure of a Tendermint block header.
#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct Header {
    /// basic block info
    #[prost(message, optional, tag = "1")]
    pub version: Option<Consensus>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    #[serde(with = "crate::types::serializers::from_str")]
    pub height: i64,
    #[prost(message, optional, tag = "4")]
    #[serde(with = "crate::types::serializers::optional")]
    pub time: ::core::option::Option<Timestamp>,
    /// prev block info
    #[prost(message, optional, tag = "5")]
    pub last_block_id: Option<BlockId>,
    /// hashes of block data
    ///
    /// commit from validators from the last block
    #[prost(bytes = "vec", tag = "6")]
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub last_commit_hash: Vec<u8>,
    /// transactions
    #[prost(bytes = "vec", tag = "7")]
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub data_hash: Vec<u8>,
    /// hashes from the app output from the prev block
    ///
    /// validators for the current block
    #[prost(bytes = "vec", tag = "8")]
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub validators_hash: Vec<u8>,
    /// validators for the next block
    #[prost(bytes = "vec", tag = "9")]
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub next_validators_hash: Vec<u8>,
    /// consensus params for current block
    #[prost(bytes = "vec", tag = "10")]
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub consensus_hash: Vec<u8>,
    /// state after txs from the previous block
    #[prost(bytes = "vec", tag = "11")]
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub app_hash: Vec<u8>,
    /// root hash of all results from the txs from the previous block
    #[prost(bytes = "vec", tag = "12")]
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub last_results_hash: Vec<u8>,
    /// consensus info
    ///
    /// evidence included in the block
    #[prost(bytes = "vec", tag = "13")]
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub evidence_hash: Vec<u8>,
    /// original proposer of the block
    #[prost(bytes = "vec", tag = "14")]
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
            version: version.map(Into::into),
            chain_id,
            height,
            time: time.map(Into::into),
            last_block_id: last_block_id.map(Into::into),
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

impl From<inner::Header> for Header {
    fn from(
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
    ) -> Self {
        Self {
            version: version.map(Into::into),
            chain_id,
            height,
            time: time.map(Into::into),
            last_block_id: last_block_id.map(Into::into),
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
