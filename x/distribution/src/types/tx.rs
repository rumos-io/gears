use gears::{
    core::{errors::CoreError, Protobuf},
    derive::AppMessage,
    types::{
        address::{AccAddress, AddressError, ValAddress},
        base::coins::UnsignedCoins,
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct MsgWithdrawDelegatorRewardRaw {
    #[prost(bytes, tag = "1")]
    pub validator_address: Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub delegator_address: Vec<u8>,
    #[prost(bool, tag = "3")]
    pub withdraw_commission: bool,
}

impl From<MsgWithdrawDelegatorReward> for MsgWithdrawDelegatorRewardRaw {
    fn from(
        MsgWithdrawDelegatorReward {
            validator_address,
            delegator_address,
            withdraw_commission,
        }: MsgWithdrawDelegatorReward,
    ) -> Self {
        Self {
            validator_address: validator_address.into(),
            delegator_address: delegator_address.into(),
            withdraw_commission,
        }
    }
}

/// MsgWithdrawDelegatorReward represents delegation withdrawal to a delegator
/// from a single validator.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmos.distribution.v1beta1.WithdrawRewards")]
pub struct MsgWithdrawDelegatorReward {
    pub validator_address: ValAddress,
    #[msg(signer)]
    pub delegator_address: AccAddress,
    pub withdraw_commission: bool,
}

impl TryFrom<MsgWithdrawDelegatorRewardRaw> for MsgWithdrawDelegatorReward {
    type Error = AddressError;

    fn try_from(
        MsgWithdrawDelegatorRewardRaw {
            validator_address,
            delegator_address,
            withdraw_commission,
        }: MsgWithdrawDelegatorRewardRaw,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_address: ValAddress::try_from(validator_address)?,
            delegator_address: AccAddress::try_from(delegator_address)?,
            withdraw_commission,
        })
    }
}

impl Protobuf<MsgWithdrawDelegatorRewardRaw> for MsgWithdrawDelegatorReward {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct MsgSetWithdrawAddrRaw {
    #[prost(bytes, tag = "1")]
    pub delegator_address: Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub withdraw_address: Vec<u8>,
}

impl From<MsgSetWithdrawAddr> for MsgSetWithdrawAddrRaw {
    fn from(
        MsgSetWithdrawAddr {
            delegator_address,
            withdraw_address,
        }: MsgSetWithdrawAddr,
    ) -> Self {
        Self {
            delegator_address: delegator_address.into(),
            withdraw_address: withdraw_address.into(),
        }
    }
}

/// MsgSetWithdrawAddr represents delegation withdrawal to a delegator
/// from a single validator.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmos.distribution.v1beta1.SetWithdrawAddr")]
pub struct MsgSetWithdrawAddr {
    #[msg(signer)]
    pub delegator_address: AccAddress,
    pub withdraw_address: AccAddress,
}

impl TryFrom<MsgSetWithdrawAddrRaw> for MsgSetWithdrawAddr {
    type Error = AddressError;

    fn try_from(
        MsgSetWithdrawAddrRaw {
            delegator_address,
            withdraw_address,
        }: MsgSetWithdrawAddrRaw,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            delegator_address: AccAddress::try_from(delegator_address)?,
            withdraw_address: AccAddress::try_from(withdraw_address)?,
        })
    }
}

impl Protobuf<MsgSetWithdrawAddrRaw> for MsgSetWithdrawAddr {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct MsgFundCommunityPoolRaw {
    #[prost(bytes, tag = "1")]
    pub amount: Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub depositor: Vec<u8>,
}

impl From<MsgFundCommunityPool> for MsgFundCommunityPoolRaw {
    fn from(MsgFundCommunityPool { amount, depositor }: MsgFundCommunityPool) -> Self {
        Self {
            amount: serde_json::to_vec(&amount).expect("serialization of domain type never fail"),
            depositor: depositor.into(),
        }
    }
}

/// MsgFundCommunityPool represents delegation withdrawal to a delegator
/// from a single validator.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmos.distribution.v1beta1.FundCommunityPool")]
pub struct MsgFundCommunityPool {
    pub amount: UnsignedCoins,
    #[msg(signer)]
    pub depositor: AccAddress,
}

impl TryFrom<MsgFundCommunityPoolRaw> for MsgFundCommunityPool {
    type Error = CoreError;

    fn try_from(
        MsgFundCommunityPoolRaw { amount, depositor }: MsgFundCommunityPoolRaw,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            amount: serde_json::from_slice(&amount)
                .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?,
            depositor: AccAddress::try_from(depositor)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
        })
    }
}

impl Protobuf<MsgFundCommunityPoolRaw> for MsgFundCommunityPool {}
