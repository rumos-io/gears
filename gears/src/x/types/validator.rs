use cosmwasm_std::{Decimal256, Uint256};
use prost::Enumeration;
use serde::{Deserialize, Serialize};
use tendermint::types::proto::crypto::PublicKey;

use crate::{error::AppError, types::address::ValAddress};

pub trait StakingValidator {
    fn operator(&self) -> &ValAddress;
    fn bonded_tokens(&self) -> &Uint256;
    fn delegator_shares(&self) -> &Decimal256;
    fn cons_pub_key(&self) -> &PublicKey;
    fn is_jailed(&self) -> bool;
    fn min_self_delegation(&self) -> &Uint256;
    fn commission(&self) -> Decimal256;
    fn tokens_from_shares(&self, shares: Decimal256) -> Result<Decimal256, AppError>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Enumeration, strum::Display)]
pub enum BondStatus {
    #[strum(to_string = "Unbonded")]
    Unbonded = 0,
    #[strum(to_string = "Unbonding")]
    Unbonding = 1,
    #[strum(to_string = "Bonded")]
    Bonded = 2,
}
