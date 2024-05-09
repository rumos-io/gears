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
