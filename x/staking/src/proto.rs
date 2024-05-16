use crate::VALIDATORS_BY_POWER_INDEX_KEY;
use chrono::Utc;
use gears::{
    tendermint::types::proto::{crypto::PublicKey, validator::ValidatorUpdate},
    types::{
        address::{AccAddress, ConsAddress, ValAddress},
        base::{coin::Coin, send::SendCoins},
        decimal256::Decimal256,
        uint::Uint256,
    },
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DvvTriplet {
    pub del_addr: AccAddress,
    pub val_src_addr: ValAddress,
    pub val_dst_addr: ValAddress,
}
impl DvvTriplet {
    pub fn new(del_addr: AccAddress, val_src_addr: ValAddress, val_dst_addr: ValAddress) -> Self {
        Self {
            del_addr,
            val_src_addr,
            val_dst_addr,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DvPair {
    pub val_addr: ValAddress,
    pub del_addr: AccAddress,
}
impl DvPair {
    pub fn new(val_addr: ValAddress, del_addr: AccAddress) -> Self {
        Self { val_addr, del_addr }
    }
}

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
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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
    pub unbonding_time: chrono::DateTime<Utc>,
    /// commission defines the commission parameters.
    // TODO: original code has complex structure for the field
    pub commission: SendCoins,
    pub min_self_delegation: Uint256,
    pub status: BondStatus,
}

impl Validator {
    pub fn abci_validator_update(&self, power: i64) -> ValidatorUpdate {
        ValidatorUpdate {
            pub_key: self.consensus_pubkey.clone(),
            power: self.consensus_power(power),
        }
    }

    pub fn abci_validator_update_zero(&self) -> ValidatorUpdate {
        self.abci_validator_update(0)
    }

    pub fn get_cons_addr(&self) -> ConsAddress {
        self.consensus_pubkey.clone().into()
    }

    pub fn update_status(&mut self, status: BondStatus) {
        self.status = status;
    }

    pub fn consensus_power(&self, power: i64) -> i64 {
        match self.status {
            BondStatus::Bonded => self.potential_consensus_power(power),
            _ => 0,
        }
    }

    pub fn potential_consensus_power(&self, power: i64) -> i64 {
        self.tokens_to_consensus_power(power)
    }

    pub fn tokens_to_consensus_power(&self, power: i64) -> i64 {
        let amount = self.tokens.amount;
        let amount = amount / Uint256::from(power as u64);
        amount
            .to_string()
            .parse::<i64>()
            .expect("It is expected that we can get i64 substitution from type Uint256")
    }

    /// GetValidatorsByPowerIndexKey creates the validator by power index.
    /// Power index is the key used in the power-store, and represents the relative
    /// power ranking of the validator.
    /// VALUE: validator operator address ([]byte)
    pub fn key_by_power_index_key(&self, power_reduction: i64) -> Vec<u8> {
        // NOTE the address doesn't need to be stored because counter bytes must always be different
        // NOTE the larger values are of higher value
        let consensus_power = self.tokens_to_consensus_power(power_reduction);
        let consensus_power_bytes = consensus_power.to_ne_bytes();

        let oper_addr_invr = self
            .operator_address
            .to_string()
            .as_bytes()
            .iter()
            .map(|b| 255 ^ b)
            .collect::<Vec<_>>();

        // key is of format prefix || powerbytes || addrLen (1byte) || addrBytes
        let mut key = VALIDATORS_BY_POWER_INDEX_KEY.to_vec();
        key.extend_from_slice(&consensus_power_bytes);
        key.push(oper_addr_invr.len() as u8);
        key.extend_from_slice(&oper_addr_invr);
        key
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
