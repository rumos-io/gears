use gears::baseapp::genesis::Genesis;
use serde::{Deserialize, Serialize};

use crate::{params::MintingParams, types::minter::Minter};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MintGenesis {
    pub minter: Minter,
    pub params: MintingParams,
}

impl Genesis for MintGenesis {
    fn add_genesis_account(
        &mut self,
        _address: gears::types::address::AccAddress,
        _coins: gears::types::base::coins::UnsignedCoins,
    ) -> Result<(), gears::baseapp::genesis::GenesisError> {
        Ok(())
    }
}
