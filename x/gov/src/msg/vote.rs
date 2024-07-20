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

use super::GovMsg;

mod inner {
    pub use ibc_proto::cosmos::gov::v1beta1::MsgVote;
    pub use ibc_proto::cosmos::gov::v1beta1::Vote;
    pub use ibc_proto::cosmos::gov::v1beta1::WeightedVoteOption;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: AccAddress,
    pub option: VoteOption,
}

impl TryFrom<inner::Vote> for Vote {
    type Error = CoreError;

    #[allow(deprecated)] // This structure would be removed with field
    fn try_from(
        inner::Vote {
            proposal_id,
            voter,
            option,
            options: _,
        }: inner::Vote,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            voter: AccAddress::from_bech32(&voter)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            option: option.try_into()?,
        })
    }
}

impl From<Vote> for inner::Vote {
    #[allow(deprecated)] // This structure would be removed with field
    fn from(
        Vote {
            proposal_id,
            voter,
            option,
        }: Vote,
    ) -> Self {
        Self {
            proposal_id,
            voter: voter.to_string(),
            option: option.clone() as i32,
            options: vec![inner::WeightedVoteOption {
                option: option as i32,
                weight: Decimal256::one().to_cosmos_proto_string(),
            }],
        }
    }
}

impl Protobuf<inner::Vote> for Vote {}

#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, strum::EnumIter, strum::EnumString,
)]
pub enum VoteOption {
    #[strum(serialize = "empty", serialize = "e")]
    Empty = 0,
    #[strum(serialize = "yes", serialize = "y")]
    Yes = 1,
    #[strum(serialize = "abstain", serialize = "a")]
    Abstain = 2,
    #[strum(serialize = "no", serialize = "n")]
    No = 3,
    #[strum(serialize = "veto")]
    NoWithVeto = 4,
}

impl TryFrom<i32> for VoteOption {
    type Error = CoreError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => VoteOption::Empty,
            1 => VoteOption::Yes,
            2 => VoteOption::Abstain,
            3 => VoteOption::No,
            4 => VoteOption::NoWithVeto,
            _ => Err(CoreError::DecodeGeneral(
                "Vote option bigger than possible value".to_owned(),
            ))?,
        })
    }
}

impl Vote {
    pub const TYPE_URL: &'static str = "/cosmos.gov.v1beta1/MsgVote";
}

impl TxMessage for Vote {
    fn get_signers(&self) -> Vec<&AccAddress> {
        vec![&self.voter]
    }

    fn type_url(&self) -> &'static str {
        Vote::TYPE_URL
    }
}

impl Protobuf<inner::MsgVote> for Vote {}

impl TryFrom<inner::MsgVote> for Vote {
    type Error = CoreError;

    fn try_from(
        inner::MsgVote {
            proposal_id,
            voter,
            option,
        }: inner::MsgVote,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            voter: AccAddress::from_bech32(&voter)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            option: option.try_into()?,
        })
    }
}

impl From<Vote> for inner::MsgVote {
    fn from(
        Vote {
            proposal_id,
            voter,
            option,
        }: Vote,
    ) -> Self {
        Self {
            proposal_id,
            voter: voter.into(),
            option: option as i32,
        }
    }
}

impl TryFrom<Any> for Vote {
    type Error = CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        if value.type_url != Self::TYPE_URL {
            Err(CoreError::DecodeGeneral(
                "message type not recognized".into(),
            ))?
        }
        <Vote as Protobuf<inner::Vote>>::decode::<Bytes>(value.value.into())
            .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))
    }
}

impl From<Vote> for Any {
    fn from(msg: Vote) -> Self {
        Any {
            type_url: Vote::TYPE_URL.to_string(),
            value: <Vote as Protobuf<inner::Vote>>::encode_vec(&msg).expect(IBC_ENCODE_UNWRAP),
        }
    }
}

impl From<Vote> for GovMsg {
    fn from(value: Vote) -> Self {
        Self::Vote(value)
    }
}
