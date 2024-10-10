use auth::GenesisState as AuthGenesis;
use bank::GenesisState as BankGenesis;
use gears::{
    baseapp::genesis::GenesisError,
    types::{address::AccAddress, base::coins::UnsignedCoins},
};
use genutil::genesis::GenutilGenesis;
use gov::{genesis::GovGenesisState, proposal::Proposals};
use ibc_rs::GenesisState as IBCGenesis;
use serde::{Deserialize, Serialize};
use staking::GenesisState as StakingGenesis;

use crate::store_keys::GaiaParamsStoreKey;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct GenesisState {
    pub bank: BankGenesis,
    pub auth: AuthGenesis,
    pub staking: StakingGenesis,
    pub ibc: IBCGenesis,
    pub genutil: GenutilGenesis,
    pub gov: GovGenesisState<Proposals<GaiaParamsStoreKey>>,
}

impl gears::baseapp::genesis::Genesis for GenesisState {
    fn add_genesis_account(
        &mut self,
        address: AccAddress,
        coins: UnsignedCoins,
    ) -> Result<(), GenesisError> {
        self.bank.add_genesis_account(address.clone(), coins);
        self.auth.add_genesis_account(address)
    }
}
