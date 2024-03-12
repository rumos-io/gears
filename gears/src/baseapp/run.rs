use axum::body::Body;
use axum::Router;
use database::RocksDB;
use proto_messages::cosmos::tx::v1beta1::message::Message;
use std::net::SocketAddr;
use std::path::PathBuf;
use store_crate::StoreKey;
use tendermint::abci::ServerBuilder;
use tracing::{error, info};

use crate::application::ApplicationInfo;
use crate::baseapp::BaseApp;
use crate::client::rest::{run_rest_server, RestState};
use crate::config::{ApplicationConfig, Config};
use crate::utils::get_config_file_from_home_dir;
use crate::x::params::{Keeper, ParamsSubspaceKey};
use tracing::metadata::LevelFilter;

use super::{ABCIHandler, Genesis};

#[derive(Debug, Clone)]
pub struct RunCommand {
    pub home: PathBuf,
    pub address: SocketAddr,
    pub rest_listen_addr: SocketAddr,
    pub read_buf_size: usize,
    pub log_level: LogLevel,
}

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("{0}")]
    HomeDirectory(String),
    #[error("{0}")]
    Database(#[from] database::error::Error),
    #[error("{0}")]
    TendermintServer(#[from] tendermint::abci::Error),
    #[error("{0}")]
    Custom(String),
}

#[derive(Debug, Clone, Default, strum::Display)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum LogLevel {
    #[strum(to_string = "debug")]
    Debug,
    #[default]
    #[strum(to_string = "info")]
    Info,
    #[strum(to_string = "warn")]
    Warn,
    #[strum(to_string = "error")]
    Error,
    #[strum(to_string = "off")]
    Off,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Info => Self::INFO,
            LogLevel::Warn => Self::WARN,
            LogLevel::Error => Self::ERROR,
            LogLevel::Off => Self::OFF,
        }
    }
}

pub fn run<
    'a,
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AC: ApplicationConfig,
    AI: ApplicationInfo,
>(
    cmd: RunCommand,
    params_keeper: Keeper<SK, PSK>,
    params_subspace_key: PSK,
    abci_handler_builder: &'a dyn Fn(Config<AC>) -> H, // TODO: why trait object here. Why not FnOnce?
    router: Router<RestState<SK, PSK, M, H, G, AI>, Body>,
) -> Result<(), RunError> {
    let RunCommand {
        home,
        address,
        rest_listen_addr,
        read_buf_size,
        log_level,
    } = cmd;

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .try_init()
        .map_err(|e| RunError::Custom(format!("Failed to set logger: {}", e.to_string())))?;

    info!("Using directory {} for config and data", home.display());

    let mut db_dir = home.clone();
    db_dir.push("data");
    db_dir.push("application.db");
    let db = RocksDB::new(db_dir)?;

    let mut cfg_file_path = home.clone();
    get_config_file_from_home_dir(&mut cfg_file_path);

    let config: Config<AC> = Config::from_file(cfg_file_path)
        .map_err(|e| RunError::Custom(format!("Error reading config file: {:?}", e)))?;

    let abci_handler = abci_handler_builder(config.clone());

    let app: BaseApp<SK, PSK, M, H, G, AI> =
        BaseApp::new(db, params_keeper, params_subspace_key, abci_handler);

    run_rest_server(
        app.clone(),
        rest_listen_addr,
        router,
        config.tendermint_rpc_address,
    );

    let server = ServerBuilder::new(read_buf_size).bind(address, app)?;

    server.listen().map_err(|e| e.into())
}
