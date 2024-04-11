use std::str::FromStr;

use tendermint::types::{
    chain_id::{ChainId, ChainIdErrors},
    proto::{block::BlockId, consensus::Consensus, header::RawHeader},
    time::Timestamp,
};

/// Header defines the structure of a Tendermint block header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub version: Option<Consensus>,
    pub chain_id: ChainId,
    pub height: i64,
    pub time: Option<Timestamp>,
    pub last_block_id: Option<BlockId>,
    pub last_commit_hash: Vec<u8>,
    pub data_hash: Vec<u8>,
    pub validators_hash: Vec<u8>,
    pub next_validators_hash: Vec<u8>,
    pub consensus_hash: Vec<u8>,
    pub app_hash: Vec<u8>,
    pub last_results_hash: Vec<u8>,
    pub evidence_hash: Vec<u8>,
    pub proposer_address: Vec<u8>,
}

impl TryFrom<RawHeader> for Header {
    type Error = ChainIdErrors;

    fn try_from(
        RawHeader {
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
        }: RawHeader,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: ChainId::from_str(&chain_id)?,
            version,
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
        })
    }
}
