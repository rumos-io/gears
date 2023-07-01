use proto_messages::cosmos::base::v1beta1::Coin;
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
    pub coins: Vec<Coin>,
}

//TODO: implement default?
