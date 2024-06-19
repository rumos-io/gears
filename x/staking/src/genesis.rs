use crate::{
    BondStatus, Delegation, LastValidatorPower, Params, Redelegation, UnbondingDelegation,
    Validator,
};
use gears::{
    error::AppError,
    types::{address::ConsAddress, uint::Uint256},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct GenesisState {
    /// params defines all the parameters of related to deposit.
    pub params: Params,
    /// validators defines the validator set at genesis.
    pub validators: Vec<Validator>,
    pub last_total_power: Uint256,
    pub exported: bool,
    pub last_validator_powers: Vec<LastValidatorPower>,
    pub delegations: Vec<Delegation>,
    pub unbonding_delegations: Vec<UnbondingDelegation>,
    pub redelegations: Vec<Redelegation>,
}

impl GenesisState {
    /// Validates the provided staking genesis state to ensure the
    /// expected invariants holds. (i.e. params in correct bounds, no duplicate validators)
    pub fn validate(&self) -> Result<(), AppError> {
        self.validate_validators()?;
        self.params.validate()
    }

    fn validate_validators(&self) -> Result<(), AppError> {
        let mut addr_set: HashSet<&[u8]> = HashSet::new();
        for v in self.validators.iter() {
            let cons_pub_key_raw = v.consensus_pubkey.raw();
            if addr_set.contains(cons_pub_key_raw) {
                let str_pub_addr: ConsAddress = v.consensus_pubkey.clone().into();
                return Err(AppError::Custom(format!(
                    "duplicate validator in genesis state: moniker {}, address {}",
                    v.description.moniker, str_pub_addr
                )));
            }

            if v.jailed && v.status == BondStatus::Bonded {
                let str_pub_addr: ConsAddress = v.consensus_pubkey.clone().into();
                return Err(AppError::Custom(format!(
                    "validator is bonded and jailed in genesis state: moniker {}, address {}",
                    v.description.moniker, str_pub_addr
                )));
            }

            if v.delegator_shares.is_zero() && v.status != BondStatus::Unbonding {
                return Err(AppError::Custom(format!(
                    "bonded/unbonded genesis validator cannot have zero delegator shares, validator: {}",
                    v.operator_address
                )));
            }

            addr_set.insert(cons_pub_key_raw);
        }
        Ok(())
    }
}
