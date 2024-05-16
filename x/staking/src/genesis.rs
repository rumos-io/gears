use crate::{
    Delegation, LastValidatorPower, Params, Pool, Redelegation, UnbondingDelegation, Validator,
};
use gears::types::uint::Uint256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GenesisState {
    /// params defines all the parameters of related to deposit.
    pub params: Params,
    /// validators defines the validator set at genesis.
    pub validators: Vec<Validator>,
    pub pool: Pool,
    pub last_total_power: Uint256,
    pub exported: bool,
    pub last_validator_powers: Vec<LastValidatorPower>,
    pub delegations: Vec<Delegation>,
    pub unbonding_delegations: Vec<UnbondingDelegation>,
    pub redelegations: Vec<Redelegation>,
}

impl GenesisState {
    pub fn validate(&self) -> Result<(), String> {
        // TODO
        // self.validate_validators()?;
        self.params.validate()
    }
}
