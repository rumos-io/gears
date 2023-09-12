use axum::body::Body;
use axum::Router;
use clap::{arg, value_parser, Arg, ArgAction, ArgMatches, Command};
use database::RocksDB;
use proto_messages::cosmos::tx::v1beta1::Message;
use std::net::SocketAddr;
use std::path::PathBuf;
use store_crate::StoreKey;
use tendermint_abci::ServerBuilder;
use tracing::metadata::LevelFilter;
use tracing::{error, info};

use crate::baseapp::BaseApp;
use crate::client::rest::{run_rest_server, RestState};
use crate::config::Config;
use crate::utils::get_default_home_dir;
use crate::x::params::{Keeper, ParamsSubspaceKey};

use super::ante::{AuthKeeper, BankKeeper};
use super::{Genesis, Handler};

const DEFAULT_REST_LISTEN_ADDR: &str = "127.0.0.1:1317";
const DEFAULT_ADDRESS: &str = "127.0.0.1:26658";

pub fn run_run_command<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>(
    matches: &ArgMatches,
    app_name: &'static str,
    app_version: &'static str,
    bank_keeper: BK,
    auth_keeper: AK,
    params_keeper: Keeper<SK, PSK>,
    params_subspace_key: PSK,
    handler: H,
    router: Router<RestState<SK, PSK, M, BK, AK, H, G>, Body>,
) {
    let address = matches.get_one::<SocketAddr>("address").cloned();

    let read_buf_size = matches
        .get_one::<usize>("read_buf_size")
        .expect("Read buf size arg has a default value so this cannot be `None`.");

    let rest_listen_addr = matches.get_one::<SocketAddr>("rest_listen_addr").cloned();

    let verbose = matches.get_flag("verbose");
    let quiet = matches.get_flag("quiet");

    let log_level = if quiet {
        LevelFilter::OFF
    } else if verbose {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    tracing_subscriber::fmt().with_max_level(log_level).init();

    let default_home_directory = get_default_home_dir(app_name);
    let home = matches
        .get_one::<PathBuf>("home")
        .or(default_home_directory.as_ref())
        .unwrap_or_else(|| {
            error!("Home argument not provided and OS does not provide a default home directory");
            std::process::exit(1)
        });
    info!("Using directory {} for config and data", home.display());

    let mut db_dir = home.clone();
    db_dir.push("data");
    db_dir.push("application.db");
    let db = RocksDB::new(db_dir).unwrap_or_else(|e| {
        error!("Could not open database: {}", e);
        std::process::exit(1)
    });

    let app: BaseApp<SK, PSK, M, BK, AK, H, G> = BaseApp::new(
        db,
        app_name,
        app_version,
        bank_keeper,
        auth_keeper,
        params_keeper,
        params_subspace_key,
        handler,
    );

    let mut cfg_file = home.clone();
    cfg_file.push("config"); //TODO: filenames + directories should be written as constants
    cfg_file.push("app.toml");

    let config = Config::new(cfg_file).unwrap_or_else(|err| {
        error!("Error reading config file: {:?}", err);
        std::process::exit(1)
    });

    let rest_listen_addr = rest_listen_addr.or(config.rest_listen_addr).unwrap_or(
        DEFAULT_REST_LISTEN_ADDR
            .parse::<SocketAddr>()
            .expect("hard coded address should be valid"),
    );

    run_rest_server(
        app.clone(),
        rest_listen_addr,
        router,
        config.tendermint_rpc_address,
    );

    let address = address.or(config.address).unwrap_or(
        DEFAULT_ADDRESS
            .parse::<SocketAddr>()
            .expect("hard coded address should be valid"),
    );

    let server = ServerBuilder::new(*read_buf_size)
        .bind(address, app)
        .unwrap_or_else(|e| {
            error!("Error binding to host: {}", e);
            std::process::exit(1)
        });
    server.listen().unwrap_or_else(|e| {
        error!("Fatal server error: {}", e);
        std::process::exit(1)
    });

    unreachable!("server.listen() will not return `Ok`")
}

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
                        .to_string()
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
