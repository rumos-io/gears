use std::path::PathBuf;

pub const CONFIG_DIR: &str = "config";
pub const GENESIS_FILE_NAME: &str = "genesis.json";
pub const CONFIG_FILE_NAME: &str = "app.toml";

pub const DEFAULT_DIR_NAME: &str = ".tendermint";

pub fn home_dir() -> Option<PathBuf> {
    Some(dirs::home_dir()?.join(DEFAULT_DIR_NAME))
}

// pub fn get_default_home_dir(app_name: &str) -> Option<PathBuf> {
//     dirs::home_dir().map(|mut h| {
//         h.push(format!(".{}", app_name));
//         h
//     })
// }

// pub fn get_genesis_file_from_home_dir(home: &mut PathBuf) {
//     get_config_dir_from_home_dir(home);
//     home.push(GENESIS_FILE_NAME)
// }

// pub fn get_config_file_from_home_dir(home: &mut PathBuf) {
//     get_config_dir_from_home_dir(home);
//     home.push(CONFIG_FILE_NAME)
// }

// pub fn get_config_dir_from_home_dir(home: &mut PathBuf) {
//     home.push(CONFIG_DIR)
// }
