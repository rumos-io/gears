use std::str::FromStr;

use crate::{error::NumericError, types::address::ValAddress};
use cosmwasm_std::{Decimal256, Uint256};
use prost::Enumeration;
use serde::{Deserialize, Serialize};
use tendermint::types::proto::crypto::PublicKey;

pub trait StakingValidator {
    fn operator(&self) -> &ValAddress;
    fn tokens(&self) -> Uint256;
    fn bonded_tokens(&self) -> Uint256;
    fn delegator_shares(&self) -> Decimal256;
    fn cons_pub_key(&self) -> &PublicKey;
    fn is_jailed(&self) -> bool;
    fn min_self_delegation(&self) -> Uint256;
    fn commission(&self) -> Decimal256;
    fn status(&self) -> BondStatus;
    fn tokens_from_shares(&self, shares: Decimal256) -> Result<Decimal256, NumericError>;
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Serialize,
    Deserialize,
    Enumeration,
    strum::Display,
    strum::EnumString,
)]
pub enum BondStatus {
    #[serde(rename = "BOND_STATUS_UNSPECIFIED")]
    #[strum(to_string = "Unspecified")]
    #[strum(serialize = "unspecified")]
    Unspecified = 0,
    #[serde(rename = "BOND_STATUS_UNBONDED")]
    #[strum(to_string = "Unbonded")]
    #[strum(serialize = "unbonded")]
    Unbonded = 1,
    #[serde(rename = "BOND_STATUS_UNBONDING")]
    #[strum(to_string = "Unbonding")]
    #[strum(serialize = "unbonding")]
    Unbonding = 2,
    #[serde(rename = "BOND_STATUS_BONDED")]
    #[strum(to_string = "Bonded")]
    #[strum(serialize = "bonded")]
    Bonded = 3,
}

impl TryFrom<String> for BondStatus {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(BondStatus::from_str(&value)?)
    }
}

impl From<BondStatus> for String {
    fn from(value: BondStatus) -> Self {
        value.to_string()
    }
}
