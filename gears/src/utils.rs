use std::path::PathBuf;

const CONFIG_DIR: &str = "config";
const GENESIS_FILE_NAME: &str = "genesis.json";

pub fn get_default_home_dir(app_name: &str) -> Option<PathBuf> {
    dirs::home_dir().map(|mut h| {
        h.push(format!(".{}", app_name));
        h
    })
}

pub fn get_default_genesis_file(app_name: &str) -> Option<PathBuf> {
    let mut home = get_default_home_dir(app_name)?;
    get_genesis_file_from_home_dir(&mut home);
    Some(home)
}

pub fn get_genesis_file_from_home_dir(home: &mut PathBuf) {
    get_config_dir_from_home_dir(home);
    home.push(GENESIS_FILE_NAME)
}

pub fn get_config_dir_from_home_dir(home: &mut PathBuf) {
    home.push(CONFIG_DIR)
}
