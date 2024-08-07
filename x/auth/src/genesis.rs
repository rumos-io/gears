use gears::{
    baseapp::genesis::GenesisError,
    types::{
        account::{Account, BaseAccount},
        address::AccAddress,
    },
};
use serde::{Deserialize, Serialize};

use crate::AuthsParams;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisState {
    pub accounts: Vec<Account>,
    pub params: AuthsParams,
}

impl Default for GenesisState {
    fn default() -> Self {
        Self {
            accounts: vec![],
            params: AuthsParams {
                max_memo_characters: 256,
                tx_sig_limit: 7,
                tx_size_cost_per_byte: 10,
                sig_verify_cost_ed25519: 590,
                sig_verify_cost_secp256k1: 1000,
            },
        }
    }
}

impl GenesisState {
    pub fn add_genesis_account(&mut self, address: AccAddress) -> Result<(), GenesisError> {
        let mut contains = false;
        for acct in &self.accounts {
            if acct.get_address() == &address {
                contains = true;
                break;
            }
        }

        if !contains {
            self.accounts.push(Account::Base(BaseAccount {
                address,
                pub_key: None,
                account_number: 0, // This is ignored when initializing from genesis
                sequence: 0,       //TODO: make a BaseAccount constructor
            }));
            Ok(())
        } else {
            Err(GenesisError(address))?
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn add_genesis_account_works() {
        let mut genesis_state = GenesisState::default();
        let address = "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux"
            .parse()
            .unwrap();
        genesis_state
            .add_genesis_account(address)
            .expect("will succeed because address is not in genesis_state.accounts");

        assert_eq!(genesis_state.accounts.len(), 1);
        assert!(matches!(
                &genesis_state.accounts[0],
                Account::Base(BaseAccount {
                    address,
                    pub_key: None,
                    account_number: 0,
                    sequence: 0,
                })
             if address == &AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux").unwrap()),);
    }

    #[test]
    fn test_deserialize_genesis() {
        let genesis = r#"{
            "accounts": [
                {
                    "@type": "/cosmos.auth.v1beta1.ModuleAccount",
                    "base_account": {
                        "account_number": "0",
                        "address": "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh",
                        "pub_key": null,
                        "sequence": "0"
                    },
                    "name": "bonded_tokens_pool",
                    "permissions": [
                        "burner",
                        "staking"
                    ]
                },
                {
                    "@type": "/cosmos.auth.v1beta1.ModuleAccount",
                    "base_account": {
                        "account_number": "1",
                        "address": "cosmos1tygms3xhhs3yv487phx3dw4a95jn7t7lpm470r",
                        "pub_key": null,
                        "sequence": "0"
                    },
                    "name": "not_bonded_tokens_pool",
                    "permissions": [
                        "burner",
                        "staking"
                    ]
                },
                {
                    "@type": "/cosmos.auth.v1beta1.BaseAccount",
                    "account_number": "2",
                    "address": "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
                    "pub_key": {
                        "@type": "/cosmos.crypto.secp256k1.PubKey",
                        "key": "AvUEsFHbsr40nTSmWh7CWYRZHGwf4cpRLtJlaRO4VAoq"
                    },
                    "sequence": "1"
                },
                {
                    "@type": "/cosmos.auth.v1beta1.ModuleAccount",
                    "base_account": {
                        "account_number": "3",
                        "address": "cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta",
                        "pub_key": null,
                        "sequence": "0"
                    },
                    "name": "fee_collector",
                    "permissions": []
                }
            ],
            "params": {
                "max_memo_characters": "256",
                "sig_verify_cost_ed25519": "590",
                "sig_verify_cost_secp256k1": "1000",
                "tx_sig_limit": "7",
                "tx_size_cost_per_byte": "10"
            }
        }"#;

        serde_json::from_str::<GenesisState>(genesis).unwrap();
    }
}
