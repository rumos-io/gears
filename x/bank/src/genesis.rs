use proto_messages::cosmos::{
    base::v1beta1::SendCoins,
    tx::v1beta1::tx_metadata::{DenomUnit, Metadata},
};
use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

use crate::Params;

// TODO: should remove total supply since it can be derived from the balances
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisState {
    pub balances: Vec<Balance>,
    pub params: Params,
    pub denom_metadata: Vec<Metadata>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Balance {
    pub address: AccAddress,
    pub coins: SendCoins,
}

impl Default for GenesisState {
    fn default() -> Self {
        Self {
            balances: vec![],
            params: Params {
                default_send_enabled: true,
            },
            //TODO: this denom metadata should not be hard coded into the bank module
            // this has been added here for short term convenience. There should be a
            // CLI command to add denom metadata to the genesis state
            denom_metadata: vec![Metadata {
                description: String::new(),
                denom_units: vec![
                    DenomUnit {
                        denom: "ATOM".parse().expect("hard coded value is valid"),
                        exponent: 6,
                        aliases: Vec::new(),
                    },
                    DenomUnit {
                        denom: "uatom".parse().expect("hard coded value is valid"),
                        exponent: 0,
                        aliases: Vec::new(),
                    },
                ],
                base: "uatom".into(),
                display: "ATOM".into(),
                name: String::new(),
                symbol: String::new(),
            }],
        }
    }
}

impl GenesisState {
    /// NOTE: If the genesis_state already contains an entry for the given address then this method
    /// will add another entry to the list i.e. it does not merge entries
    pub fn add_genesis_account(&mut self, address: AccAddress, coins: SendCoins) {
        self.balances.push(Balance { address, coins })
    }
}
