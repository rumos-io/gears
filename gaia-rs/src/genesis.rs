use gears::{error::AppError, ibc::address::AccAddress, types::base::send::SendCoins};
// use proto_messages::cosmos::base::v1beta1::SendCoins;
// use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

use auth::GenesisState as AuthGenesis;
use bank::GenesisState as BankGenesis;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct GenesisState {
    pub bank: BankGenesis,
    pub auth: AuthGenesis,
}

impl gears::baseapp::Genesis for GenesisState {
    fn add_genesis_account(
        &mut self,
        address: AccAddress,
        coins: SendCoins,
    ) -> Result<(), AppError> {
        self.bank.add_genesis_account(address.clone(), coins);
        self.auth.add_genesis_account(address)
    }
}
