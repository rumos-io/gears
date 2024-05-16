use crate::application::handlers::node::ABCIHandler;
use crate::application::ApplicationInfo;
use crate::baseapp::genesis::Genesis;
use crate::baseapp::BaseApp;
use crate::config::{ApplicationConfig, Config, ConfigDirectory};
use crate::grpc::run_grpc_server;
use crate::params::{Keeper, ParamsSubspaceKey};
use crate::rest::{run_rest_server, RestState};
use crate::types::tx::TxMessage;
use axum::Router;
use database::RocksDB;
use std::net::SocketAddr;
use std::path::PathBuf;
use store_crate::StoreKey;
use tendermint::abci::ServerBuilder;
use tendermint::application::ABCI;
use tracing::metadata::LevelFilter;
use tracing::{error, info};

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
    TendermintServer(#[from] tendermint::abci::errors::Error),
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
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AC: ApplicationConfig,
    AI: ApplicationInfo,
>(
    cmd: RunCommand,
    params_keeper: Keeper<SK, PSK>,
    params_subspace_key: PSK,
    abci_handler_builder: &dyn Fn(Config<AC>) -> H, // TODO: why trait object here. Why not FnOnce?
    router: Router<RestState<SK, PSK, M, H, G, AI>>,
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
        .map_err(|e| RunError::Custom(format!("Failed to set logger: {}", e)))?;

    info!("Using directory {} for config and data", home.display());

    let db_dir = home.join("data");
    let db = RocksDB::new(db_dir.join("application.db"))?;

    let cfg_file_path = ConfigDirectory::ConfigFile.path_from_hone(&home);

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

    run_grpc_server();

    let server = ServerBuilder::new(read_buf_size).bind(address, ABCI::from(app))?;

    server.listen().map_err(|e| e.into())
}
