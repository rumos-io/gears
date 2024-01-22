use proto_messages::cosmos::base::v1beta1::SendCoins;
use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

use crate::Params;

// TODO: should remove total supply since it can be derived from the balances
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisState {
    pub balances: Vec<Balance>,
    pub params: Params,
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
