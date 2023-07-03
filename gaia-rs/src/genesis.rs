use serde::{Deserialize, Serialize};

use auth::GenesisState as AuthGenesis;
use bank::GenesisState as BankGenesis;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GenesisState {
    pub bank: BankGenesis,
    pub auth: AuthGenesis,
}

impl Default for GenesisState {
    fn default() -> Self {
        Self {
            bank: bank::GenesisState {
                balances: vec![bank::Balance {
                    address: proto_types::AccAddress::from_bech32(
                        "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
                    )
                    .unwrap(),
                    coins: vec![proto_messages::cosmos::base::v1beta1::Coin {
                        denom: proto_types::Denom::try_from(String::from("uatom")).unwrap(),
                        amount: cosmwasm_std::Uint256::from_u128(34),
                    }],
                }],
                params: bank::Params {
                    default_send_enabled: true,
                },
            },
            auth: auth::GenesisState {
                accounts: vec![proto_messages::cosmos::auth::v1beta1::BaseAccount {
                    address: proto_types::AccAddress::from_bech32(
                        "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
                    )
                    .unwrap(),
                    pub_key: None,
                    account_number: 0,
                    sequence: 0,
                }],
                params: gears::x::auth::Params {
                    max_memo_characters: 256,
                    tx_sig_limit: 7,
                    tx_size_cost_per_byte: 10,
                    sig_verify_cost_ed25519: 590,
                    sig_verify_cost_secp256k1: 1000,
                },
            },
        }
    }
}
