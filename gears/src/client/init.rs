use std::path::PathBuf;

use log::info;
use serde::Serialize;
use tendermint::informal::chain::Id;

use crate::config::ApplicationConfig;

#[derive(Debug, Clone, derive_builder::Builder,)]
pub struct InitCommand {
    pub home: PathBuf,
    pub moniker: String,
    pub chain_id: Id,
}

pub fn init<G: Serialize, AC: ApplicationConfig>(
    cmd: InitCommand,
    app_genesis_state: &G,
) -> Result<(), InitError> {
    let InitCommand {
        moniker,
        home,
        chain_id,
    } = cmd;

    // Create config directory
    let config_dir = home.join("config");
    std::fs::create_dir_all(&config_dir).map_err(|e| InitError::CreateConfigDirectory(e))?;

    // Create data directory
    let data_dir = home.join("data");
    std::fs::create_dir_all(&data_dir).map_err(|e| InitError::CreateDataDirectory(e))?;

    // Write tendermint config file
    let tm_config_file_path = config_dir.join("config.toml");
    let tm_config_file = std::fs::File::create(&tm_config_file_path)
        .map_err(|e| InitError::CreateConfigDirectory(e))?;

    tendermint::write_tm_config(tm_config_file, &moniker)
        .map_err(|e| InitError::WriteConfigFile(e))?;

    info!(
        "Tendermint config written to {}",
        tm_config_file_path.display()
    );

    // Create node key file
    let node_key_file_path = config_dir.join("node_key.json");
    let node_key_file =
        std::fs::File::create(&node_key_file_path).map_err(|e| InitError::CreateNodeKeyFile(e))?;

    // Create private validator key file
    let priv_validator_key_file_path = config_dir.join("priv_validator_key.json");
    let priv_validator_key_file = std::fs::File::create(&priv_validator_key_file_path)
        .map_err(|e| InitError::PrivValidatorKey(e))?;

    let app_state = serde_json::to_value(app_genesis_state)?;

    // Create genesis file
    let mut genesis_file_path = home.clone();
    crate::utils::get_genesis_file_from_home_dir(&mut genesis_file_path);
    let genesis_file =
        std::fs::File::create(&genesis_file_path).map_err(|e| InitError::CreateGenesisFile(e))?;

    // Create config file
    let mut cfg_file_path = home.clone();
    crate::utils::get_config_file_from_home_dir(&mut cfg_file_path);
    let cfg_file =
        std::fs::File::create(&cfg_file_path).map_err(|e| InitError::CreateConfigFile(e))?;

    crate::config::Config::<AC>::write_default(cfg_file)
        .map_err(|e| InitError::WriteDefaultConfigFile(e.to_string()))?;

    info!("Config file written to {}", cfg_file_path.display());

    // Write key and genesis
    tendermint::write_keys_and_genesis(
        node_key_file,
        priv_validator_key_file,
        genesis_file,
        app_state,
        chain_id,
    )
    .map_err(|e| InitError::WriteKeysAndGenesis(e))?;

    info!(
        "Key files written to {} and {}",
        node_key_file_path.display(),
        priv_validator_key_file_path.display()
    );
    info!("Genesis file written to {}", genesis_file_path.display());

    // Write private validator state file
    let state_file_path = data_dir.join("priv_validator_state.json");
    let state_file =
        std::fs::File::create(&state_file_path).map_err(|e| InitError::PrivValidatorKey(e))?;

    tendermint::write_priv_validator_state(state_file)
        .map_err(|e| InitError::WritePrivValidatorKey(e))?;

    info!(
        "Private validator state written to {}",
        state_file_path.display()
    );

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    // TODO: reduce error count
    #[error("Could not create config directory {0}")]
    CreateConfigDirectory(#[source] std::io::Error),
    #[error("Could not create data directory {0}")]
    CreateDataDirectory(#[source] std::io::Error),
    #[error("Could not create config file {0}")]
    CreateConfigFile(#[source] std::io::Error),
    #[error("Error writing config file {0}")]
    WriteConfigFile(#[source] tendermint::error::Error),
    #[error("{0}")]
    WriteDefaultConfigFile(String),
    #[error("Could not create node key file {0}")]
    CreateNodeKeyFile(#[source] std::io::Error),
    #[error("Could not create private validator key file {0}")]
    PrivValidatorKey(#[source] std::io::Error),
    #[error("Error writing private validator state file {0}")]
    WritePrivValidatorKey(#[source] tendermint::error::Error),
    #[error("{0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("Could not create genesis file {0}")]
    CreateGenesisFile(#[source] std::io::Error),
    #[error("Could not create config file {0}")]
    CreateConfigError(#[source] std::io::Error),
    #[error("Error writing config file {0}")]
    WriteConfigError(#[source] std::io::Error),
    #[error("Error writing key and genesis files {0}")]
    WriteKeysAndGenesis(#[source] tendermint::error::Error),
}