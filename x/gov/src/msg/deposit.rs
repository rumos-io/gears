use bytes::Bytes;
use gears::types::address::AccAddress;
use gears::types::base::coins::UnsignedCoins;
use gears::{
    core::{any::google::Any, errors::CoreError},
    error::IBC_ENCODE_UNWRAP,
    tendermint::types::proto::Protobuf,
    types::{base::errors::CoinError, tx::TxMessage},
};
use serde::{Deserialize, Serialize};

use crate::msg::GovMsg;

mod inner {
    pub use ibc_proto::cosmos::gov::v1beta1::Deposit;
    pub use ibc_proto::cosmos::gov::v1beta1::MsgDeposit;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deposit {
    pub proposal_id: u64,
    pub depositor: AccAddress,
    pub amount: UnsignedCoins,
}

impl Deposit {
    pub(crate) const KEY_PREFIX: [u8; 1] = [0x10];
    pub const TYPE_URL: &'static str = "/cosmos.gov.v1beta1/MsgDeposit";

    pub(crate) fn key(proposal_id: u64, depositor: &AccAddress) -> Vec<u8> {
        [
            Self::KEY_PREFIX.as_slice(),
            &proposal_id.to_be_bytes(),
            &[depositor.len()],
            depositor.as_ref(),
        ]
        .concat()
    }
}

impl Protobuf<inner::MsgDeposit> for Deposit {}

impl TxMessage for Deposit {
    fn get_signers(&self) -> Vec<&AccAddress> {
        vec![&self.depositor]
    }

    fn type_url(&self) -> &'static str {
        Deposit::TYPE_URL
    }
}

impl TryFrom<inner::MsgDeposit> for Deposit {
    type Error = CoreError;

    fn try_from(
        inner::MsgDeposit {
            proposal_id,
            depositor,
            amount,
        }: inner::MsgDeposit,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            depositor: AccAddress::from_bech32(&depositor)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            amount: {
                let mut coins = Vec::with_capacity(amount.len());
                for coin in amount {
                    coins.push(
                        coin.try_into()
                            .map_err(|e: CoinError| CoreError::Coin(e.to_string()))?,
                    )
                }
                UnsignedCoins::new(coins).map_err(|e| CoreError::DecodeAddress(e.to_string()))?
            },
        })
    }
}

impl From<Deposit> for inner::MsgDeposit {
    fn from(
        Deposit {
            proposal_id,
            depositor,
            amount,
        }: Deposit,
    ) -> Self {
        Self {
            proposal_id,
            depositor: depositor.into(),
            amount: amount
                .into_inner()
                .into_iter()
                .map(|this| this.into())
                .collect(),
        }
    }
}

impl TryFrom<Any> for Deposit {
    type Error = CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        if value.type_url != Self::TYPE_URL {
            Err(CoreError::DecodeGeneral(
                "message type not recognized".into(),
            ))?
        }
        Deposit::decode::<Bytes>(value.value.into())
            .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))
    }
}

impl From<Deposit> for Any {
    fn from(msg: Deposit) -> Self {
        Any {
            type_url: Deposit::TYPE_URL.to_string(),
            value: msg.encode_vec().expect(IBC_ENCODE_UNWRAP),
        }
    }
}

impl From<Deposit> for GovMsg {
    fn from(value: Deposit) -> Self {
        Self::Deposit(value)
    }
}

impl TryFrom<inner::Deposit> for Deposit {
    type Error = CoreError;

    fn try_from(
        inner::Deposit {
            proposal_id,
            depositor,
            amount,
        }: inner::Deposit,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            depositor: AccAddress::from_bech32(&depositor)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            amount: UnsignedCoins::new({
                let mut result = Vec::with_capacity(amount.len());

                for coin in amount {
                    result.push(
                        coin.try_into()
                            .map_err(|e| CoreError::Coins(format!("Deposit: {e}")))?,
                    );
                }

                result
            })
            .map_err(|e| CoreError::Coins(e.to_string()))?,
        })
    }
}

impl From<Deposit> for inner::Deposit {
    fn from(
        Deposit {
            proposal_id,
            depositor,
            amount,
        }: Deposit,
    ) -> Self {
        Self {
            proposal_id,
            depositor: depositor.to_string(),
            amount: amount.into_iter().map(|this| this.into()).collect(),
        }
    }
}
