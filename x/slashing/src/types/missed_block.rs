use gears::{
    tendermint::types::proto::Protobuf,
    types::address::{AddressError, ConsAddress},
};
use prost::Message;
use serde::{Deserialize, Serialize};

/// MissedBlock contains height and missed status as boolean.
#[derive(Clone, PartialEq, Deserialize, Serialize, Message)]
pub struct MissedBlock {
    /// index is the height at which the block was missed.
    #[prost(uint32)]
    pub index: u32,
    /// missed is the missed status.
    #[prost(bool)]
    pub missed: bool,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, Message)]
pub struct ValidatorMissedBlocksRaw {
    #[prost(bytes)]
    pub address: Vec<u8>,
    #[prost(message, repeated)]
    pub missed_blocks: Vec<MissedBlock>,
}

impl From<ValidatorMissedBlocks> for ValidatorMissedBlocksRaw {
    fn from(src: ValidatorMissedBlocks) -> Self {
        Self {
            address: src.address.as_ref().to_vec(),
            missed_blocks: src.missed_blocks,
        }
    }
}

/// ValidatorMissedBlocks contains array of missed blocks of corresponding
/// address.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ValidatorMissedBlocks {
    /// address is the validator address.
    pub address: ConsAddress,
    /// missed_blocks is an array of missed blocks by the validator.
    pub missed_blocks: Vec<MissedBlock>,
}

impl TryFrom<ValidatorMissedBlocksRaw> for ValidatorMissedBlocks {
    type Error = AddressError;

    fn try_from(src: ValidatorMissedBlocksRaw) -> Result<Self, Self::Error> {
        Ok(ValidatorMissedBlocks {
            address: ConsAddress::try_from(src.address)?,
            missed_blocks: src.missed_blocks,
        })
    }
}

impl Protobuf<ValidatorMissedBlocksRaw> for ValidatorMissedBlocks {}
