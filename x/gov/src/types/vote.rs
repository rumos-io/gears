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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteWeight(Decimal256);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, thiserror::Error)]
#[error("parse error: Invalid weight for vote. Required to be positive and not greater than 1")]
pub struct VoteWeightError;

impl TryFrom<Decimal256> for VoteWeight {
    type Error = VoteWeightError;

    fn try_from(value: Decimal256) -> Result<Self, Self::Error> {
        if value < Decimal256::zero() || value > Decimal256::zero() {
            return Err(VoteWeightError);
        }

        Ok(Self(value))
    }
}
