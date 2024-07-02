use crate::{SignerInfo, SlashingParams, ValidatorMissedBlocks};
use serde::{Deserialize, Serialize};

/// GenesisState defines the slashing module's genesis state.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GenesisState {
    /// params defines all the paramaters of related to deposit.
    pub params: SlashingParams,
    /// signing_infos represents a map between validator addresses and their
    /// signing infos.
    pub signing_infos: Vec<SignerInfo>,
    /// missed_blocks represents a map between validator addresses and their
    /// missed blocks.
    pub missed_blocks: Vec<ValidatorMissedBlocks>,
}
