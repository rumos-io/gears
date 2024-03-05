use std::path::PathBuf;

use derive_builder::Builder;
use gears::{baseapp::Genesis as GenesisTrait, error::AppError};
use proto_messages::cosmos::base::v1beta1::SendCoins;
use proto_types::AccAddress;
use tendermint::informal::Genesis;

pub const DEFAULT_DIR_NAME: &str = ".tendermint";

fn default_home() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to retrieve home dir")
        .join(DEFAULT_DIR_NAME)
}

#[derive(Debug, Clone, Builder)]
pub struct GenesisOptions {
    #[builder(default = "default_home()")]
    home: PathBuf,
    address: AccAddress,
    coins: SendCoins,
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

pub fn genesis_account_add<G: GenesisTrait>(opt: GenesisOptions) -> Result<(), GenesisError> {
    let GenesisOptions {
        home,
        address,
        coins,
    } = opt;

    let mut genesis_file_path = home.clone();
    gears::utils::get_genesis_file_from_home_dir(&mut genesis_file_path);

    let raw_genesis = std::fs::read_to_string(genesis_file_path.clone())?;
    let mut genesis: Genesis<G> = serde_json::from_str(&raw_genesis)?;
    genesis.app_state.add_genesis_account(address, coins)?;
    std::fs::write(genesis_file_path, &serde_json::to_string_pretty(&genesis)?)?;

    Ok(())
}
