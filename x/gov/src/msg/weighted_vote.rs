use gears::types::{address::AccAddress, decimal256::Decimal256};
use serde::{Deserialize, Serialize};

use super::vote::{MsgVote, VoteOption};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteWeighted {
    pub proposal_id: u64,
    pub voter: AccAddress,
    pub options: Vec<VoteOptionWeighted>,
}

impl VoteWeighted {
    /// We always store vote with weight
    pub(crate) const KEY_PREFIX: [u8; 1] = [0x20];

    pub fn key(proposal_id: u64, voter: &AccAddress) -> Vec<u8> {
        [
            Self::KEY_PREFIX.as_slice(),
            &proposal_id.to_be_bytes(),
            &[voter.len()],
            voter.as_ref(),
        ]
        .concat()
    }
}

impl From<MsgVote> for VoteWeighted {
    fn from(
        MsgVote {
            proposal_id,
            voter,
            option: options,
        }: MsgVote,
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
