use gears::{
    baseapp::genesis::Genesis,
    types::{address::AccAddress, base::coins::UnsignedCoins, tx::metadata::Metadata},
};
use serde::{Deserialize, Serialize};

use crate::BankParams;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisState {
    pub balances: Vec<Balance>,
    pub params: BankParams,
    pub denom_metadata: Vec<Metadata>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Balance {
    pub address: AccAddress,
    pub coins: UnsignedCoins,
}

impl Default for GenesisState {
    fn default() -> Self {
        Self {
            balances: vec![],
            params: BankParams {
                // TODO: maybe default denom
                send_enabled: vec![],
                default_send_enabled: true,
            },
            //TODO: this denom metadata should not be hard coded into the bank module
            // this has been added here for short term convenience. There should be a
            // CLI command to add denom metadata to the genesis state
            denom_metadata: vec![],
            // denom_metadata: vec![Metadata {
            //     description: String::new(),
            //     denom_units: vec![
            //         DenomUnit {
            //             denom: "ATOM".parse().expect("hard coded value is valid"),
            //             exponent: 6,
            //             aliases: Vec::new(),
            //         },
            //         DenomUnit {
            //             denom: "uatom".parse().expect("hard coded value is valid"),
            //             exponent: 0,
            //             aliases: Vec::new(),
            //         },
            //     ],
            //     base: "uatom".into(),
            //     display: "ATOM".into(),
            //     name: String::new(),
            //     symbol: String::new(),
            // }],
        }
    }
}

impl GenesisState {
    /// NOTE: If the genesis_state already contains an entry for the given address then this method
    /// will add another entry to the list i.e. it does not merge entries
    pub fn add_genesis_account(&mut self, address: AccAddress, coins: UnsignedCoins) {
        self.balances.push(Balance { address, coins })
    }
}

impl Genesis for GenesisState {
    fn add_genesis_account(
        &mut self,
        address: AccAddress,
        coins: UnsignedCoins,
    ) -> Result<(), gears::baseapp::genesis::GenesisError> {
        self.add_genesis_account(address, coins);

        Ok(())
    }
}
