use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tendermint_rpc::Url;

pub const DEFAULT_REST_LISTEN_ADDR: &str = "127.0.0.1:1317";
pub const DEFAULT_ADDRESS: &str = "127.0.0.1:26658";
pub const DEFAULT_TENDERMINT_RPC_ADDRESS: &str = "http://localhost:26657";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Config {
    pub tendermint_rpc_address: Url,
    pub rest_listen_addr: SocketAddr,
    pub address: SocketAddr,
}

impl Config {
    pub fn from_file(filename: PathBuf) -> Result<Config, Box<dyn Error>> {
        let s = fs::read_to_string(filename)?;
        Ok(toml::from_str(&s)?)
    }

    pub fn write_default(mut file: File) -> Result<(), Box<dyn Error>> {
        let mut handlebars = handlebars::Handlebars::new();
        handlebars
            .register_template_string("config", CONFIG_TEMPLATE)
            .expect("hard coded config template is valid");

        let cfg = Config::default();

        let config = handlebars
            .render("config", &cfg)
            .expect("Config will always work with the CONFIG_TEMPLATE");

        file.write_all(config.as_bytes()).map_err(|e| e.into())
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

const CONFIG_TEMPLATE: &str = r#"# This is a TOML config file.
# For more information, see https://github.com/toml-lang/toml

#######################################################################
###                   Main Base Config Options                      ###
#######################################################################

# ABCI application TCP socket address
address = "{{address}}"

# REST service TCP socket address
rest_listen_addr = "{{rest_listen_addr}}"

# Tendermint node RPC proxy address
tendermint_rpc_address = "{{tendermint_rpc_address}}"
"#;
