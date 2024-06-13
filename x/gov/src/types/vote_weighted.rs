use gears::types::{address::AccAddress, decimal256::Decimal256};
use serde::{Deserialize, Serialize};

use crate::keeper::KEY_VOTES_PREFIX;

use super::votes::{Vote, VoteOption};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteWeighted {
    pub proposal_id: u64,
    pub voter: AccAddress,
    pub options: Vec<VoteOptionWeighted>,
}

impl VoteWeighted {
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

impl From<Vote> for VoteWeighted {
    fn from(
        Vote {
            proposal_id,
            voter,
            option: options,
        }: Vote,
    ) -> Self {
        Self {
            proposal_id,
            voter,
            options: vec![VoteOptionWeighted::non_split(options)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteOptionWeighted {
    option: VoteOption,
    weight: VoteWeight,
}

impl VoteOptionWeighted {
    pub fn non_split(option: VoteOption) -> Self {
        Self {
            option,
            weight: VoteWeight::try_from(Decimal256::zero()).expect("default is valid"),
        }
    }
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
