use crate::{
    Delegation, LastValidatorPower, Redelegation, StakingParams, UnbondingDelegation, Validators,
};
use gears::{baseapp::genesis::Genesis, types::uint::Uint256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct GenesisState {
    /// params defines all the parameters of related to deposit.
    pub params: StakingParams,
    /// validators defines the validator set at genesis.
    pub validators: Validators,
    pub last_total_power: Uint256,
    pub exported: bool,
    pub last_validator_powers: Vec<LastValidatorPower>,
    pub delegations: Vec<Delegation>,
    pub unbonding_delegations: Vec<UnbondingDelegation>,
    pub redelegations: Vec<Redelegation>,
}

impl Genesis for GenesisState {
    fn add_genesis_account(
        &mut self,
        _address: gears::types::address::AccAddress,
        _coins: gears::types::base::coins::UnsignedCoins,
    ) -> Result<(), gears::baseapp::genesis::GenesisError> {
        Ok(()) // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_genesis() {
        let genesis = r#"{
            "params": {
                "unbonding_time": 1814400,
                "max_validators": 100,
                "max_entries": 7,
                "historical_entries": 10000,
                "bond_denom": "stake"
            },
            "validators": [
                {
    "operator_address": "cosmosvaloper1sp6zygg2wch",
    "delegator_shares": "1",
    "description": {
        "moniker": "validator1",
        "identity": "",
        "website": "",
        "security_contact": "",
        "details": ""
    },
    "consensus_pubkey": {
        "type": "tendermint/PubKeyEd25519",
        "value": "cVp6"
    },
    "jailed": false,
    "tokens": "1",
    "unbonding_height": "0",
    "unbonding_time": "1970-01-01T00:00:10.0000001Z",
    "commission": {
        "commission_rates": {
            "rate": "1",
            "max_rate": "1",
            "max_change_rate": "1"
        },
        "update_time": "1970-01-01T00:00:10.0000001Z"
    },
    "min_self_delegation": "1",
    "status": "BOND_STATUS_BONDED"
}
            ],
            "last_total_power": "0",
            "exported": false,
            "last_validator_powers": [],
            "delegations": [],
            "unbonding_delegations": [],
            "redelegations": []
        }"#;

        assert!(serde_json::from_str::<GenesisState>(genesis).is_ok());
    }

    #[test]
    /// Fails because validator is jailed and bonded
    fn test_deserialize_genesis_fail() {
        let genesis = r#"{
            "params": {
                "unbonding_time": 1814400,
                "max_validators": 100,
                "max_entries": 7,
                "historical_entries": 10000,
                "bond_denom": "stake"
            },
            "validators": [
                {
    "operator_address": "cosmosvaloper1sp6zygg2wch",
    "delegator_shares": "1",
    "description": {
        "moniker": "validator1",
        "identity": "",
        "website": "",
        "security_contact": "",
        "details": ""
    },
    "consensus_pubkey": {
        "type": "tendermint/PubKeyEd25519",
        "value": "cVp6"
    },
    "jailed": true,
    "tokens": "1",
    "unbonding_height": "0",
    "unbonding_time": "1970-01-01T00:00:10.0000001Z",
    "commission": {
        "commission_rates": {
            "rate": "1",
            "max_rate": "1",
            "max_change_rate": "1"
        },
        "update_time": "1970-01-01T00:00:10.0000001Z"
    },
    "min_self_delegation": "1",
    "status": "BOND_STATUS_BONDED"
}
            ],
            "last_total_power": "0",
            "exported": false,
            "last_validator_powers": [],
            "delegations": [],
            "unbonding_delegations": [],
            "redelegations": []
        }"#;

        assert_eq!(serde_json::from_str::<GenesisState>(genesis).unwrap_err().to_string(),
        "validator is bonded and jailed in genesis state: moniker validator1, address cosmosvalcons1u33k3satgu7ehms6q7379wuq3zq2w563y75hu0 at line 39 column 13".to_string());
    }

    #[test]
    /// Fails because params are invalid
    fn test_deserialize_genesis_fail_invalid_params() {
        let genesis = r#"{
            "params": {
                "unbonding_time": -1,
                "max_validators": 100,
                "max_entries": 7,
                "historical_entries": 10000,
                "bond_denom": "stake"
            },
            "validators": [
                {
    "operator_address": "cosmosvaloper1sp6zygg2wch",
    "delegator_shares": "1",
    "description": {
        "moniker": "validator1",
        "identity": "",
        "website": "",
        "security_contact": "",
        "details": ""
    },
    "consensus_pubkey": {
        "type": "tendermint/PubKeyEd25519",
        "value": "cVp6"
    },
    "jailed": true,
    "tokens": "1",
    "unbonding_height": "0",
    "unbonding_time": "1970-01-01T00:00:10.0000001Z",
    "commission": {
        "commission_rates": {
            "rate": "1",
            "max_rate": "1",
            "max_change_rate": "1"
        },
        "update_time": "1970-01-01T00:00:10.0000001Z"
    },
    "min_self_delegation": "1",
    "status": "BOND_STATUS_BONDED"
}
            ],
            "last_total_power": "0",
            "exported": false,
            "last_validator_powers": [],
            "delegations": [],
            "unbonding_delegations": [],
            "redelegations": []
        }"#;

        assert_eq!(
            serde_json::from_str::<GenesisState>(genesis)
                .unwrap_err()
                .to_string(),
            "unbonding time must be non negative: -1 at line 8 column 13".to_string()
        );
    }
}
