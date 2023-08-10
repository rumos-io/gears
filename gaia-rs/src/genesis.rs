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
                balances: vec![],
                params: bank::Params {
                    default_send_enabled: true,
                },
            },
            auth: auth::GenesisState {
                accounts: vec![],
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
