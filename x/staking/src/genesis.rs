use crate::{Params, Validator};
use chrono::Utc;
use gears::{
    core::{
        address::{AccAddress, ValAddress},
        base::coin::Coin,
    },
    types::{decimal256::Decimal256, uint::Uint256},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pool {
    pub not_bonded_tokens: Coin,
    pub bonded_tokens: Coin,
}

/// Last validator power, needed for validator set update logic
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LastValidatorPower {
    pub address: ValAddress,
    pub power: i64,
}

/// Delegation represents the bond with tokens held by an account. It is
/// owned by one delegator, and is associated with the voting power of one
/// validator.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Delegation {
    pub delegator_address: AccAddress,
    pub validator_address: ValAddress,
    pub shares: Decimal256,
}

/// Delegation represents the bond with tokens held by an account. It is
/// owned by one delegator, and is associated with the voting power of one
/// validator.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnbondingDelegation {
    pub delegator_address: AccAddress,
    pub validator_address: ValAddress,
    pub entries: Vec<UnbondingDelegationEntry>,
}

/// UnbondingDelegationEntry - entry to an UnbondingDelegation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnbondingDelegationEntry {
    pub creation_height: i64,
    pub completion_time: chrono::DateTime<Utc>,
    pub initial_balance: Coin,
    pub balance: Coin,
}

/// Redelegation contains the list of a particular delegator's
/// redelegating bonds from a particular source validator to a
/// particular destination validator
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Redelegation {
    pub delegator_address: AccAddress,
    pub validator_src_address: ValAddress,
    pub validator_dst_address: ValAddress,
    pub entries: Vec<RedelegationEntry>,
}

/// RedelegationEntry - entry to a Redelegation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedelegationEntry {
    pub creation_height: i64,
    pub completion_time: chrono::DateTime<Utc>,
    pub initial_balance: Coin,
    pub share_dst: Decimal256,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GenesisState {
    /// params defines all the parameters of related to deposit.
    pub params: Params,
    /// validators defines the validator set at genesis.
    pub validators: Vec<Validator>,
    pub pool: Pool,
    // TODO: sdk.Int
    pub last_total_power: Uint256,
    pub exported: bool,
    pub last_validator_powers: Vec<LastValidatorPower>,
    pub delegations: Vec<Delegation>,
    pub unbonding_delegations: Vec<UnbondingDelegation>,
    pub redelegations: Vec<Redelegation>,
}

impl GenesisState {
    pub fn validate(&self) -> Result<(), String> {
        // self.validate_validators()?;
        self.params.validate()
    }

    // func validateGenesisStateValidators(validators []types.Validator) error {
    //     addrMap := make(map[string]bool, len(validators))
    //
    //     for i := 0; i < len(validators); i++ {
    //         val := validators[i]
    //         consPk, err := val.ConsPubKey()
    //         if err != nil {
    //             return err
    //         }
    //
    //         strKey := string(consPk.Bytes())
    //
    //         if _, ok := addrMap[strKey]; ok {
    //             consAddr, err := val.GetConsAddr()
    //             if err != nil {
    //                 return err
    //             }
    //             return fmt.Errorf("duplicate validator in genesis state: moniker %v, address %v", val.Description.Moniker, consAddr)
    //         }
    //
    //         if val.Jailed && val.IsBonded() {
    //             consAddr, err := val.GetConsAddr()
    //             if err != nil {
    //                 return err
    //             }
    //             return fmt.Errorf("validator is bonded and jailed in genesis state: moniker %v, address %v", val.Description.Moniker, consAddr)
    //         }
    //
    //         if val.DelegatorShares.IsZero() && !val.IsUnbonding() {
    //             return fmt.Errorf("bonded/unbonded genesis validator cannot have zero delegator shares, validator: %v", val)
    //         }
    //
    //         addrMap[strKey] = true
    //     }
    //
    //     return nil
    // }
}
