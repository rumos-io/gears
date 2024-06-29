use bytes::Bytes;
use gears::{
    core::errors::CoreError,
    error::IBC_ENCODE_UNWRAP,
    tendermint::types::proto::Protobuf,
    types::{address::AccAddress, tx::TxMessage},
};
use ibc_proto::google::protobuf::Any;
use serde::{Deserialize, Serialize};

use super::GovMsg;

mod inner {
    pub use ibc_proto::cosmos::gov::v1beta1::MsgVote;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgVote {
    pub proposal_id: u64,
    pub voter: AccAddress,
    pub option: VoteOption,
}

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

impl MsgVote {
    pub const TYPE_URL: &'static str = "/cosmos.gov.v1beta1/MsgVote";
}

impl TxMessage for MsgVote {
    fn get_signers(&self) -> Vec<&AccAddress> {
        vec![&self.voter]
    }

    fn validate_basic(&self) -> Result<(), String> {
        Ok(())
    }

    fn type_url(&self) -> &'static str {
        MsgVote::TYPE_URL
    }
}

impl Protobuf<inner::MsgVote> for MsgVote {}

impl TryFrom<inner::MsgVote> for MsgVote {
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

impl From<MsgVote> for inner::MsgVote {
    fn from(
        MsgVote {
            proposal_id,
            voter,
            option,
        }: MsgVote,
    ) -> Self {
        Self {
            proposal_id,
            voter: voter.into(),
            option: option as i32,
        }
    }
}

impl TryFrom<Any> for MsgVote {
    type Error = CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        if value.type_url != Self::TYPE_URL {
            Err(CoreError::DecodeGeneral(
                "message type not recognized".into(),
            ))?
        }
        MsgVote::decode::<Bytes>(value.value.into())
            .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))
    }
}

impl From<MsgVote> for Any {
    fn from(msg: MsgVote) -> Self {
        Any {
            type_url: MsgVote::TYPE_URL.to_string(),
            value: msg.encode_vec().expect(IBC_ENCODE_UNWRAP),
        }
    }
}

impl From<MsgVote> for GovMsg {
    fn from(value: MsgVote) -> Self {
        Self::Vote(value)
    }
}
