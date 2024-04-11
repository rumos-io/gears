use std::path::PathBuf;

use anyhow::Result;
use core_types::address::AccAddress;
use tendermint::informal::genesis::Genesis;

use crate::{
    baseapp::genesis::Genesis as SDKGenesis, config::ConfigDirectory, error::AppError,
    types::base::send::SendCoins,
};

#[derive(Debug, Clone, derive_builder::Builder)]
pub struct GenesisCommand {
    pub home: PathBuf,
    pub address: AccAddress,
    pub coins: SendCoins,
}

#[derive(Debug, thiserror::Error)]
pub enum GenesisError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    AppError(#[from] AppError),
}

pub fn genesis_account_add<G: SDKGenesis>(cmd: GenesisCommand) -> Result<(), GenesisError> {
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
