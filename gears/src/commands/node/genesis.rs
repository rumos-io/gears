use std::path::PathBuf;

use anyhow::Result;
use tendermint::informal::genesis::Genesis;

use crate::{
    baseapp::genesis::{Genesis as SDKGenesis, GenesisError},
    config::ConfigDirectory,
    types::{address::AccAddress, base::coins::UnsignedCoins},
};

#[derive(Debug, Clone, former::Former)]
pub struct GenesisCommand {
    pub home: PathBuf,
    pub address: AccAddress,
    pub coins: UnsignedCoins,
}

#[derive(Debug, thiserror::Error)]
pub enum GenesisInitError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    Genesis(#[from] GenesisError),
}

pub fn genesis_account_add<G: SDKGenesis>(cmd: GenesisCommand) -> Result<(), GenesisInitError> {
    let GenesisCommand {
        home,
        address,
        coins,
    } = cmd;

    let genesis_file_path = ConfigDirectory::GenesisFile.path_from_hone(&home);

    let raw_genesis = std::fs::read_to_string(genesis_file_path.clone())?;
    let mut genesis: Genesis<G> = serde_json::from_str(&raw_genesis)?;
    genesis.app_state.add_genesis_account(address, coins)?;
    std::fs::write(genesis_file_path, serde_json::to_string_pretty(&genesis)?)?;

    Ok(())
}
