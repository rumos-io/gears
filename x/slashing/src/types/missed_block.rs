use gears::types::address::ConsAddress;
use serde::{Deserialize, Serialize};

/// MissedBlock contains height and missed status as boolean.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MissedBlock {
    /// index is the height at which the block was missed.
    pub index: i64,
    /// missed is the missed status.
    pub missed: bool,
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
