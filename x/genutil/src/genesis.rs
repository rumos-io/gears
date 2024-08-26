use gears::baseapp::genesis::Genesis;
use staking::CreateValidator;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GenutilGenesis {
    pub gen_txs: Vec<CreateValidator>,
}

impl Genesis for GenutilGenesis {
    fn add_genesis_account(
        &mut self,
        _address: gears::types::address::AccAddress,
        _coins: gears::types::base::coins::UnsignedCoins,
    ) -> Result<(), gears::baseapp::genesis::GenesisError> {
        Ok(())
    }
}
