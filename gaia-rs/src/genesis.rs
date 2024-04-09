use auth::GenesisState as AuthGenesis;
use bank::GenesisState as BankGenesis;
use gears::{error::AppError, ibc::address::AccAddress, types::base::send::SendCoins};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct GenesisState {
    pub bank: BankGenesis,
    pub auth: AuthGenesis,
}

impl gears::baseapp::genesis::Genesis for GenesisState {
    fn add_genesis_account(
        &mut self,
        address: AccAddress,
        coins: SendCoins,
    ) -> Result<(), AppError> {
        self.bank.add_genesis_account(address.clone(), coins);
        self.auth.add_genesis_account(address)
    }
}
