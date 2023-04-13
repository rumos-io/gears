use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    x::{auth::GenesisState as AuthGenesis, bank::GenesisState as BankGenesis},
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct GenesisState {
    pub bank: BankGenesis,
    pub auth: AuthGenesis,
}

impl FromStr for GenesisState {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let genesis: GenesisState = serde_json::from_str(s).map_err(|e| {
            AppError::Genesis(format!(
                "could not deserialize genesis data: {}",
                e.to_string()
            ))
        })?;
        Ok(genesis)
    }
}

impl From<GenesisState> for String {
    fn from(value: GenesisState) -> Self {
        serde_json::to_string(&value).expect("this cannot fail")
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Uint256;
    use proto_messages::cosmos::{auth::v1beta1::BaseAccount, base::v1beta1::Coin};
    use proto_types::AccAddress;

    use crate::x::bank::Balance;

    use super::*;

    #[test]
    fn from_string_works() {
        let genesis = r#"{
            "auth": {
              "params": {
                "max_memo_characters": "256",
                "tx_sig_limit": "7",
                "tx_size_cost_per_byte": "10",
                "sig_verify_cost_ed25519": "590",
                "sig_verify_cost_secp256k1": "1000"
              },
              "accounts": [
                {
                  "@type": "/cosmos.auth.v1beta1.BaseAccount",
                  "address": "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
                  "pub_key": null,
                  "account_number": "0",
                  "sequence": "0"
                }
              ]
            },
            "bank": {
              "params": {
                "send_enabled": [],
                "default_send_enabled": true
              },
              "balances": [
                {
                  "address": "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
                  "coins": [
                    {
                      "denom": "uatom",
                      "amount": "34"
                    }
                  ]
                }
              ],
              "supply": [
                {
                  "denom": "uatom",
                  "amount": "34"
                }
              ],
              "denom_metadata": []
            }
          }"#;

        let expected = GenesisState {
            bank: BankGenesis {
                balances: vec![Balance {
                    address: AccAddress::from_bech32(
                        "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
                    )
                    .unwrap(),
                    coins: vec![Coin {
                        denom: proto_types::Denom::try_from(String::from("uatom")).unwrap(),
                        amount: Uint256::from_u128(34),
                    }],
                }],
                params: crate::x::bank::Params {
                    default_send_enabled: true,
                },
            },
            auth: AuthGenesis {
                accounts: vec![BaseAccount {
                    address: AccAddress::from_bech32(
                        "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
                    )
                    .unwrap(),
                    pub_key: None,
                    account_number: 0,
                    sequence: 0,
                }],
                params: crate::x::auth::Params {
                    max_memo_characters: 256,
                    tx_sig_limit: 7,
                    tx_size_cost_per_byte: 10,
                    sig_verify_cost_ed25519: 590,
                    sig_verify_cost_secp256k1: 1000,
                },
            },
        };

        assert_eq!(GenesisState::from_str(genesis).unwrap(), expected);
    }

    #[test]
    fn to_string_works() {
        let genesis = r#"{
        "auth": {
          "params": {
            "max_memo_characters": "256",
            "tx_sig_limit": "7",
            "tx_size_cost_per_byte": "10",
            "sig_verify_cost_ed25519": "590",
            "sig_verify_cost_secp256k1": "1000"
          },
          "accounts": [
            {
              "@type": "/cosmos.auth.v1beta1.BaseAccount",
              "address": "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
              "pub_key": null,
              "account_number": "0",
              "sequence": "0"
            }
          ]
        },
        "bank": {
          "params": {
            "send_enabled": [],
            "default_send_enabled": true
          },
          "balances": [
            {
              "address": "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
              "coins": [
                {
                  "denom": "uatom",
                  "amount": "34"
                }
              ]
            }
          ],
          "supply": [
            {
              "denom": "uatom",
              "amount": "34"
            }
          ],
          "denom_metadata": []
        }
      }"#;

        let expected_string = r#"{"bank":{"balances":[{"address":"cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux","coins":[{"denom":"uatom","amount":"34"}]}],"params":{"default_send_enabled":true}},"auth":{"accounts":[{"address":"cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux","pub_key":null,"account_number":"0","sequence":"0"}],"params":{"max_memo_characters":"256","tx_sig_limit":"7","tx_size_cost_per_byte":"10","sig_verify_cost_ed25519":"590","sig_verify_cost_secp256k1":"1000"}}}"#;
        let genesis_state = GenesisState::from_str(genesis).unwrap();
        assert_eq!(String::from(genesis_state), expected_string);
    }
}
