use std::error::Error;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use tendermint_rpc::Url;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub tendermint_rpc_address: Url,
}

impl Config {
    pub fn new(filename: PathBuf) -> Result<Config, Box<dyn Error>> {
        let s = fs::read_to_string(filename)?;
        Ok(toml::from_str(&s)?)
    }
}
