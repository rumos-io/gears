use std::path::PathBuf;

use crate::application::ApplicationInfo;

pub const CONFIG_DIR: &str = "config";
pub const GENESIS_FILE_NAME: &str = "genesis.json";
pub const CONFIG_FILE_NAME: &str = "app.toml";
pub const CLIENT_CONFIG_FILE_NAME: &str = "client.toml";

pub const DEFAULT_DIR_NAME: &str = ".tendermint";

pub fn home_dir() -> Option<PathBuf> {
    Some(dirs::home_dir()?.join(DEFAULT_DIR_NAME))
}

pub fn default_home_dir<AI: ApplicationInfo>() -> Option<PathBuf> {
    dirs::home_dir().map(|mut h| {
        h.push(format!(".{}", AI::APP_NAME));
        h
    })
}
