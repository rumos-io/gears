use std::str::FromStr;

use bytes::Bytes;
use gears::{
    core::errors::CoreError,
    error::IBC_ENCODE_UNWRAP,
    tendermint::types::proto::Protobuf,
    types::{
        address::AccAddress,
        decimal256::{CosmosDecimalProtoString, Decimal256},
        tx::TxMessage,
    },
};
use ibc_proto::google::protobuf::Any;
use serde::{Deserialize, Serialize};

use super::{
    vote::{Vote, VoteOption},
    GovMsg,
};

mod inner {
    pub use ibc_proto::cosmos::gov::v1beta1::MsgVoteWeighted;
    pub use ibc_proto::cosmos::gov::v1beta1::Vote;
    pub use ibc_proto::cosmos::gov::v1beta1::WeightedVoteOption;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgVoteWeighted {
    pub proposal_id: u64,
    pub voter: AccAddress,
    pub options: Vec<VoteOptionWeighted>,
}

impl MsgVoteWeighted {
    /// We always store vote with weight
    pub(crate) const KEY_PREFIX: [u8; 1] = [0x20];
    pub const TYPE_URL: &'static str = "/cosmos.gov.v1beta1/MsgVoteWeighted";

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

impl From<Vote> for MsgVoteWeighted {
    fn from(
        Vote {
            proposal_id,
            voter,
            option,
        }: Vote,
    ) -> Self {
        Self {
            proposal_id,
            voter,
            options: vec![VoteOptionWeighted {
                option,
                weight: VoteWeight::try_from(Decimal256::zero()).expect("default is valid"),
            }],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteOptionWeighted {
    pub option: VoteOption,
    pub weight: VoteWeight,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, thiserror::Error)]
#[error("Failed to parse from string: format [vote]_[weight:decimal]")]
pub struct VoteOptionWeightedError;

impl FromStr for VoteOptionWeighted {
    type Err = VoteOptionWeightedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('_').collect::<Vec<_>>();
        if parts.len() == 2 {
            Ok(Self {
                option: parts[0].parse().map_err(|_| VoteOptionWeightedError)?,
                weight: parts[1].parse().map_err(|_| VoteOptionWeightedError)?,
            })
        } else {
            Err(VoteOptionWeightedError)
        }
    }
}

impl TryFrom<inner::WeightedVoteOption> for VoteOptionWeighted {
    type Error = CoreError;

    fn try_from(
        inner::WeightedVoteOption { option, weight }: inner::WeightedVoteOption,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            option: option.try_into()?,
            weight: Decimal256::from_str(&weight)
                .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?
                .try_into()
                .map_err(|e: VoteWeightError| CoreError::DecodeGeneral(e.to_string()))?,
        })
    }
}

impl From<VoteOptionWeighted> for inner::WeightedVoteOption {
    fn from(VoteOptionWeighted { option, weight }: VoteOptionWeighted) -> Self {
        Self {
            option: option as i32,
            weight: weight.0.to_cosmos_proto_string(), // TODO:NOW IS THIS CORRECT?
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Decimal256")]
pub struct VoteWeight(Decimal256);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, thiserror::Error)]
#[error("parse error: Invalid weight for vote. Required to be positive and not greater than 1")]
pub struct VoteWeightError;

impl FromStr for VoteWeight {
    type Err = VoteWeightError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(Decimal256::from_str(s).map_err(|_| VoteWeightError)?)
    }
}

impl TryFrom<Decimal256> for VoteWeight {
    type Error = VoteWeightError;

    fn try_from(value: Decimal256) -> Result<Self, Self::Error> {
        if value < Decimal256::zero() || value > Decimal256::zero() {
            return Err(VoteWeightError);
        }

        Ok(Self(value))
    }
}

impl From<VoteWeight> for Decimal256 {
    fn from(value: VoteWeight) -> Self {
        value.0
    }
}

impl TxMessage for MsgVoteWeighted {
    fn get_signers(&self) -> Vec<&AccAddress> {
        vec![&self.voter]
    }

    fn type_url(&self) -> &'static str {
        MsgVoteWeighted::TYPE_URL
    }
}

impl Protobuf<inner::MsgVoteWeighted> for MsgVoteWeighted {}

impl TryFrom<inner::MsgVoteWeighted> for MsgVoteWeighted {
    type Error = CoreError;

    fn try_from(
        inner::MsgVoteWeighted {
            proposal_id,
            voter,
            options,
        }: inner::MsgVoteWeighted,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            voter: AccAddress::from_bech32(&voter)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            options: {
                let mut mapped_options = Vec::new();
                for option in options {
                    mapped_options.push(option.try_into()?)
                }

                mapped_options
            },
        })
    }
}

impl From<MsgVoteWeighted> for inner::MsgVoteWeighted {
    fn from(
        MsgVoteWeighted {
            proposal_id,
            voter,
            options,
        }: MsgVoteWeighted,
    ) -> Self {
        Self {
            proposal_id,
            voter: voter.into(),
            options: options.into_iter().map(|this| this.into()).collect(),
        }
    }
}

impl TryFrom<Any> for MsgVoteWeighted {
    type Error = CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        if value.type_url != Self::TYPE_URL {
            Err(CoreError::DecodeGeneral(
                "message type not recognized".into(),
            ))?
        }
        <MsgVoteWeighted as Protobuf<inner::MsgVoteWeighted>>::decode::<Bytes>(value.value.into())
            .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))
    }
}

impl From<MsgVoteWeighted> for Any {
    fn from(msg: MsgVoteWeighted) -> Self {
        Any {
            type_url: MsgVoteWeighted::TYPE_URL.to_string(),
            value: <MsgVoteWeighted as Protobuf<inner::MsgVoteWeighted>>::encode_vec(&msg)
                .expect(IBC_ENCODE_UNWRAP),
        }
    }
}

impl From<MsgVoteWeighted> for GovMsg {
    fn from(value: MsgVoteWeighted) -> Self {
        Self::Weighted(value)
    }
}

impl TryFrom<inner::Vote> for MsgVoteWeighted {
    type Error = CoreError;

    #[allow(deprecated)]
    fn try_from(
        inner::Vote {
            proposal_id,
            voter,
            option: _,
            options,
        }: inner::Vote,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            voter: AccAddress::from_bech32(&voter)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            options: {
                let mut result = Vec::with_capacity(options.len());

                for option in options {
                    result.push(option.try_into()?);
                }

                result
            },
        })
    }
}

impl From<MsgVoteWeighted> for inner::Vote {
    #[allow(deprecated)]
    fn from(
        MsgVoteWeighted {
            proposal_id,
            voter,
            options,
        }: MsgVoteWeighted,
    ) -> Self {
        Self {
            proposal_id,
            voter: voter.to_string(),
            option: 0, // VOTE_OPTION_UNSPECIFIED
            options: options.into_iter().map(|this| this.into()).collect(),
        }
    }
}

impl Protobuf<inner::Vote> for MsgVoteWeighted {}
