use std::str::FromStr;

use gears::{
    core::Protobuf,
    types::{
        address::{AccAddress, ValAddress},
        decimal256::Decimal256,
        errors::StdError,
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};

/// DelegatorWithdrawInfo is the address for where distributions rewards are
/// withdrawn to by default this struct is only used at genesis to feed in
/// default withdraw addresses.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DelegatorWithdrawInfo {
    /// delegator_address is the address of the delegator.
    pub delegator_address: AccAddress,
    /// withdraw_address is the address to withdraw the delegation rewards to.
    pub withdraw_address: AccAddress,
}

/// DelegatorStartingInfoRecord used for import / export via genesis json.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DelegatorStartingInfoRecord {
    /// delegator_address is the address of the delegator.
    pub delegator_address: AccAddress,
    /// validator_address is the address of the validator.
    pub validator_address: ValAddress,
    /// starting_info defines the starting info of a delegator.
    pub starting_info: DelegatorStartingInfo,
}

#[derive(Clone, PartialEq, Message)]
pub struct DelegatorStartingInfoRaw {
    #[prost(uint64, tag = "1")]
    pub previous_period: u64,
    #[prost(string, tag = "2")]
    pub stake: String,
    #[prost(uint64, tag = "3")]
    pub height: u64,
}

impl From<DelegatorStartingInfo> for DelegatorStartingInfoRaw {
    fn from(
        DelegatorStartingInfo {
            previous_period,
            stake,
            height,
        }: DelegatorStartingInfo,
    ) -> Self {
        Self {
            previous_period,
            stake: stake.to_string(),
            height,
        }
    }
}

/// DelegatorStartingInfo represents the starting info for a delegator reward
/// period. It tracks the previous validator period, the delegation's amount of
/// staking token, and the creation height (to check later on if any slashes have
/// occurred). NOTE: Even though validators are slashed to whole staking tokens,
/// the delegators within the validator may be left with less than a full token,
/// thus sdk.Dec is used.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DelegatorStartingInfo {
    pub previous_period: u64,
    pub stake: Decimal256,
    pub height: u64,
}

impl TryFrom<DelegatorStartingInfoRaw> for DelegatorStartingInfo {
    type Error = StdError;

    fn try_from(
        DelegatorStartingInfoRaw {
            previous_period,
            stake,
            height,
        }: DelegatorStartingInfoRaw,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            previous_period,
            stake: Decimal256::from_str(&stake)?,
            height,
        })
    }
}

impl Protobuf<DelegatorStartingInfoRaw> for DelegatorStartingInfo {}
