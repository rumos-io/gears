use axum::body::Body;
use axum::Router;
use clap::{arg, value_parser, Arg, ArgAction, ArgMatches, Command};
use database::RocksDB;
use proto_messages::cosmos::tx::v1beta1::message::Message;
use std::net::SocketAddr;
use std::path::PathBuf;
use store_crate::StoreKey;
use tendermint::abci::ServerBuilder;
use tracing::metadata::LevelFilter;
use tracing::{error, info};

use crate::baseapp::BaseApp;
use crate::client::rest::{run_rest_server, RestState};
use crate::config::{ApplicationConfig, Config, DEFAULT_ADDRESS, DEFAULT_REST_LISTEN_ADDR};
use crate::utils::{get_config_file_from_home_dir, get_default_home_dir};
use crate::x::params::{Keeper, ParamsSubspaceKey};

use super::{ABCIHandler, Genesis};

#[derive(Debug, Clone)]
pub struct RunOptions {
    home: Option<PathBuf>,
    address: Option<SocketAddr>,
    rest_listen_addr: Option<SocketAddr>,
    read_buf_size: usize,
    verbose: bool,
    quiet: bool,
}

// #[cfg(feature = "cli")]
#[derive(Debug, Clone, thiserror::Error)]
#[error("Error parsing args {0}")]
pub struct RunOptionsParseError(pub String);

// #[cfg(feature = "cli")]
impl TryFrom<&ArgMatches> for RunOptions {
    type Error = RunOptionsParseError;

    fn try_from(value: &ArgMatches) -> Result<Self, Self::Error> {
        let address = value.get_one::<SocketAddr>("address").cloned();

        let read_buf_size = value
            .get_one::<usize>("read_buf_size")
            .ok_or(RunOptionsParseError(
                "Read buf size arg has a default value so this cannot be `None`.".to_owned(),
            ))?
            .clone();

        let rest_listen_addr = value.get_one::<SocketAddr>("rest_listen_addr").cloned();

        let verbose = value.get_flag("verbose");
        let quiet = value.get_flag("quiet");

        let home = value.get_one::<PathBuf>("home").cloned();

        Ok(Self {
            home,
            address,
            rest_listen_addr,
            read_buf_size,
            verbose,
            quiet,
        })
    }
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

pub fn run<
    'a,
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AC: ApplicationConfig,
>(
    options: RunOptions,
    app_name: &'static str,
    app_version: &'static str,
    params_keeper: Keeper<SK, PSK>,
    params_subspace_key: PSK,
    abci_handler_builder: &'a dyn Fn(Config<AC>) -> H,
    router: Router<RestState<SK, PSK, M, H, G>, Body>,
) -> Result<(), RunError> {
    let RunOptions {
        home,
        address,
        rest_listen_addr,
        read_buf_size,
        verbose,
        quiet,
    } = options;

    let log_level = if quiet {
        LevelFilter::OFF
    } else if verbose {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .try_init()
        .map_err(|_| RunError::Custom("failed to set tracing".to_owned()))?;

    let default_home_directory = get_default_home_dir(app_name);
    let home = home
        .or(default_home_directory)
        .ok_or(RunError::HomeDirectory(
            "Home argument not provided and OS does not provide a default home directory"
                .to_owned(),
        ))?;

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

    let app: BaseApp<SK, PSK, M, H, G> = BaseApp::new(
        db,
        app_name,
        app_version,
        params_keeper,
        params_subspace_key,
        abci_handler,
    );

    let rest_listen_addr = rest_listen_addr.unwrap_or(config.rest_listen_addr);

    run_rest_server(
        app.clone(),
        rest_listen_addr,
        router,
        config.tendermint_rpc_address,
    );

    let address = address.unwrap_or(config.address);

    let server = ServerBuilder::new(read_buf_size).bind(address, app)?;

    server.listen().map_err(|e| e.into())
}

// #[cfg(feature = "cli")]
pub fn get_run_command(app_name: &str) -> Command {
    Command::new("run")
        .about("Run the full node application")
        .arg(
            arg!(--home)
                .help(format!(
                    "Directory for config and data [default: {}]",
                    get_default_home_dir(app_name)
                        .unwrap_or_default()
                        .display()
                ))
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(--address)
                .help(format!("Application listen address. Overrides any listen address in the config. Default value is used if neither this argument nor a config value is provided [default: {}]",DEFAULT_ADDRESS ))
                .action(ArgAction::Set)
                .value_parser(value_parser!(SocketAddr))
        )
        .arg(
            arg!(--rest_listen_addr)
                .help(format!("Bind the REST server to this address. Overrides any listen address in the config. Default value is used if neither this argument nor a config value is provided [default: {}]",DEFAULT_REST_LISTEN_ADDR ))
                .action(ArgAction::Set)
                .value_parser(value_parser!(SocketAddr))
        )
        .arg(
            arg!(-r - -read_buf_size)
                .help(
                    "The default server read buffer size, in bytes, for each incoming client
                connection",
                )
                .action(ArgAction::Set)
                .value_parser(value_parser!(usize))
                .default_value("1048576"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Increase output logging verbosity to DEBUG level"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Suppress all output logging (overrides --verbose)"),
        )
}
