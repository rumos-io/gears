use crate::{
    commands::client::keys::KeyringBackend,
    config::{ConfigDirectory, DEFAULT_TENDERMINT_RPC_ADDRESS},
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr, sync::OnceLock};
use tendermint::types::chain_id::ChainId;

pub fn client_config(home: &PathBuf) -> &'static ClientConfig {
    static CLIENT_CONFIG: OnceLock<ClientConfig> = OnceLock::new();

    CLIENT_CONFIG.get_or_init(|| ClientConfig::from_home(home).unwrap_or_default())
}

pub const CHAIN_ID: &str = "test-chain";
// TODO: make it working and add enum
pub const OUTPUT: &str = "json";
// TODO: make it working and add enum
pub const BROADCAST_MODE: &str = "sync";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientConfigRaw {
    #[serde(rename = "chain-id")]
    chain_id: String,
    #[serde(rename = "keyring-backend")]
    keyring_backend: KeyringBackend,
    output: String,
    node: String,
    #[serde(rename = "broadcast-mode")]
    broadcast_mode: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(try_from = "ClientConfigRaw")]
pub struct ClientConfig {
    chain_id: ChainId,
    keyring_backend: KeyringBackend,
    output: String,
    node: url::Url,
    broadcast_mode: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            chain_id: ChainId::from_str(CHAIN_ID).expect("hardcoded value cannot fail"),
            keyring_backend: KeyringBackend::default(),
            output: OUTPUT.to_string(),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS
                .parse()
                .expect("hardcoded value cannot fail"),
            broadcast_mode: BROADCAST_MODE.to_string(),
        }
    }
}

impl TryFrom<ClientConfigRaw> for ClientConfig {
    type Error = anyhow::Error;

    fn try_from(
        ClientConfigRaw {
            chain_id,
            keyring_backend,
            output,
            node,
            broadcast_mode,
        }: ClientConfigRaw,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: chain_id.try_into()?,
            keyring_backend,
            output,
            node: node.parse()?,
            broadcast_mode,
        })
    }
}

impl ClientConfig {
    pub fn from_home(home: &PathBuf) -> Result<ClientConfig, anyhow::Error> {
        let config_path = ConfigDirectory::ClientConfigFile.path_from_home(&home);
        let config = if config_path.exists() {
            ClientConfig::from_file(&config_path)?
        } else {
            ClientConfig::write_default(&config_path)?;
            ClientConfig::default()
        };

        Ok(config)
    }

    pub fn from_file(file_path: &PathBuf) -> Result<ClientConfig, anyhow::Error> {
        let s = std::fs::read_to_string(file_path)?;
        let res = toml::from_str(&s);
        Ok(res?)
    }

    pub fn write_default(file_path: &PathBuf) -> Result<(), anyhow::Error> {
        let mut handlebars = handlebars::Handlebars::new();
        handlebars
            .register_template_string("client_config", CONFIG_TEMPLATE)
            .expect("hard coded config template is valid");

        let cfg = ClientConfig::default();

        let config = handlebars
            .render("client_config", &cfg)
            .expect("Client config will always work with the CONFIG_TEMPLATE");

        Ok(std::fs::write(file_path, config.as_bytes())?)
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }

    pub fn node(&self) -> url::Url {
        self.node.clone()
    }

    pub fn keyring_backend(&self) -> KeyringBackend {
        self.keyring_backend.clone()
    }
}

const CONFIG_TEMPLATE: &str = r#"# This is a TOML config file.
# For more information, see https://github.com/toml-lang/toml

#######################################################################
###                     Client Config Options                       ###
#######################################################################

# Name of chain to connect to
chain-id = "{{chain_id.id}}"

# Name of keyring backend to read private keys
keyring-backend = "{{keyring_backend}}"

# Format of output: json, toml or text
output = "{{output}}"

# Address of tendermint node
node = "{{node}}"

# Mode for committing the changes: sync, async, broadcast
broadcast-mode = "{{broadcast_mode}}"
"#;
