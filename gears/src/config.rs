use std::error::Error;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;

use serde::Deserialize;
use tendermint_rpc::Url;

pub const DEFAULT_REST_LISTEN_ADDR: &str = "127.0.0.1:1317";
pub const DEFAULT_ADDRESS: &str = "127.0.0.1:26658";
pub const DEFAULT_TENDERMINT_RPC_ADDRESS: &str = "http://localhost:26657";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Config {
    pub tendermint_rpc_address: Url,
    pub rest_listen_addr: SocketAddr,
    pub address: SocketAddr,
}

impl Config {
    pub fn new(filename: PathBuf) -> Result<Config, Box<dyn Error>> {
        let s = fs::read_to_string(filename)?;
        Ok(toml::from_str(&s)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tendermint_rpc_address: DEFAULT_TENDERMINT_RPC_ADDRESS
                .parse::<Url>()
                .expect("hard coded address should be valid"),
            rest_listen_addr: DEFAULT_REST_LISTEN_ADDR
                .parse::<SocketAddr>()
                .expect("hard coded address should be valid"),
            address: DEFAULT_ADDRESS
                .parse::<SocketAddr>()
                .expect("hard coded address should be valid"),
        }
    }
}
