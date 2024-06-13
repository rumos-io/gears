use gears::types::{address::AccAddress, decimal256::Decimal256};
use serde::{Deserialize, Serialize};

use crate::keeper::KEY_VOTES_PREFIX;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: AccAddress,
    pub options: Vec<VoteOption>,
}

impl Vote {
    pub fn key(&self) -> Vec<u8> {
        [
            KEY_VOTES_PREFIX.as_slice(),
            &self.proposal_id.to_be_bytes(),
            &[self.voter.len()],
            self.voter.as_ref(),
        ]
        .concat()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteOption {
    option: VoteOptions,
    weight: Decimal256,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VoteOptions {
    Empty = 0,
    Yes,
    Abstain,
    No,
    NoWithVeto,
}
