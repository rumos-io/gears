use std::path::PathBuf;

use anyhow::Result;

use proto_messages::cosmos::base::v1beta1::SendCoins;
use proto_types::AccAddress;
use tendermint::informal::Genesis;

use crate::{baseapp::Genesis as SDKGenesis, error::AppError};

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

    let mut genesis_file_path = home.clone();
    crate::utils::get_genesis_file_from_home_dir(&mut genesis_file_path);

    let raw_genesis = std::fs::read_to_string(genesis_file_path.clone())?;
    let mut genesis: Genesis<G> = serde_json::from_str(&raw_genesis)?;
    genesis.app_state.add_genesis_account(address, coins)?;
    std::fs::write(genesis_file_path, serde_json::to_string_pretty(&genesis)?)?;

    Ok(())
}
