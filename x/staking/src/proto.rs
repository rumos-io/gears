use gears::{
    crypto::public::PublicKey,
    tendermint::types::proto::validator::ValidatorUpdate,
    types::{address::ValAddress, base::coin::Coin},
    types::{base::send::SendCoins, decimal256::Decimal256, uint::Uint256},
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BondStatus {
    Unbonded = 0,
    Unbonding = 1,
    Bonded = 2,
}

impl Display for BondStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BondStatus::Unbonded => write!(f, "Unbonded"),
            BondStatus::Unbonding => write!(f, "Unbonding"),
            BondStatus::Bonded => write!(f, "Bonded"),
        }
    }
}

/// Validator defines a validator, together with the total amount of the
/// Validator's bond shares and their exchange rate to coins. Slashing results in
/// a decrease in the exchange rate, allowing correct calculation of future
/// undelegations without iterating over delegators. When coins are delegated to
/// this validator, the validator is credited with a delegation whose number of
/// bond shares is based on the amount of coins delegated divided by the current
/// exchange rate. Voting power can be calculated as total bonded shares
/// multiplied by exchange rate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Validator {
    pub operator_address: ValAddress,
    pub delegator_shares: Decimal256,
    /// consensus_pubkey is the consensus public key of the validator, as a Protobuf Any.
    pub consensus_pubkey: PublicKey,
    /// jailed defined whether the validator has been jailed from bonded status or not.
    pub jailed: bool,
    /// tokens define the delegated tokens (incl. self-delegation).
    pub tokens: Coin,
    /// unbonding_height defines, if unbonding, the height at which this validator has begun unbonding.
    pub unbonding_height: i64,
    /// unbonding_time defines, if unbonding, the min time for the validator to complete unbonding.
    pub unbonding_time: std::time::Duration,
    /// commission defines the commission parameters.
    // TODO: original code has complex structure for the field
    pub commission: SendCoins,
    pub min_self_delegation: Uint256,
    pub status: BondStatus,
}

impl Validator {
    pub fn abci_validator_update(&self) -> ValidatorUpdate {
        todo!()
        // ValidatorUpdate {
        //     pub_key: Some(self.consensus_pubkey.into()),
        //     power: self.tendermint_power(),
        // }
    }
    pub fn tendermint_power(&self) -> i64 {
        if self.status == BondStatus::Bonded {
            return self.potential_tendermint_power();
        }
        0
    }
    pub fn potential_tendermint_power(&self) -> i64 {
        // let amount = self
        //     .tokens
        //     .amount
        //     .parse::<i64>()
        //     .expect("Unexpected conversion error");
        // amount / 10i64.pow(6)
        //TODO: original code above doesn't compile
        12
    }
}
