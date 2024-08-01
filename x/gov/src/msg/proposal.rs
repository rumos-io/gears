use bytes::Bytes;
use gears::{
    core::errors::CoreError,
    error::IBC_ENCODE_UNWRAP,
    tendermint::types::proto::Protobuf,
    types::{
        address::AccAddress,
        base::{coins::UnsignedCoins, errors::CoinError},
        tx::TxMessage,
    },
};
use ibc_proto::google::protobuf::Any;
use serde::{Deserialize, Serialize};

mod inner {
    pub use ibc_proto::cosmos::gov::v1beta1::MsgSubmitProposal;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgSubmitProposal {
    pub content: Any,
    pub initial_deposit: UnsignedCoins,
    pub proposer: AccAddress,
}

impl MsgSubmitProposal {
    pub const TYPE_URL: &'static str = "/cosmos.gov.v1beta1/MsgSubmitProposal";
}

impl TxMessage for MsgSubmitProposal {
    fn get_signers(&self) -> Vec<&AccAddress> {
        vec![&self.proposer]
    }

    fn type_url(&self) -> &'static str {
        MsgSubmitProposal::TYPE_URL
    }
}

impl Protobuf<inner::MsgSubmitProposal> for MsgSubmitProposal {}

impl TryFrom<inner::MsgSubmitProposal> for MsgSubmitProposal {
    type Error = CoreError;

    fn try_from(
        inner::MsgSubmitProposal {
            content,
            initial_deposit,
            proposer,
        }: inner::MsgSubmitProposal,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            content: content.ok_or(CoreError::MissingField(
                "MsgSubmitProposal missing content".to_owned(),
            ))?,
            initial_deposit: UnsignedCoins::new({
                let mut coins = Vec::with_capacity(initial_deposit.len());
                for coin in initial_deposit {
                    coins.push(
                        coin.try_into()
                            .map_err(|e: CoinError| CoreError::Coin(e.to_string()))?,
                    )
                }

                coins
            })
            .map_err(|e| CoreError::Coins(e.to_string()))?,
            proposer: AccAddress::from_bech32(&proposer)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
        })
    }
}

impl From<MsgSubmitProposal> for inner::MsgSubmitProposal {
    fn from(
        MsgSubmitProposal {
            content,
            initial_deposit,
            proposer,
        }: MsgSubmitProposal,
    ) -> Self {
        Self {
            content: Some(content),
            initial_deposit: initial_deposit
                .into_inner()
                .into_iter()
                .map(|e| e.into())
                .collect(),
            proposer: proposer.to_string(),
        }
    }
}

impl TryFrom<Any> for MsgSubmitProposal {
    type Error = CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        if value.type_url != Self::TYPE_URL {
            Err(CoreError::DecodeGeneral(
                "message type not recognized".into(),
            ))?
        }
        MsgSubmitProposal::decode::<Bytes>(value.value.into())
            .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))
    }
}

impl From<MsgSubmitProposal> for Any {
    fn from(msg: MsgSubmitProposal) -> Self {
        Any {
            type_url: MsgSubmitProposal::TYPE_URL.to_string(),
            value: msg.encode_vec().expect(IBC_ENCODE_UNWRAP),
        }
    }
}
