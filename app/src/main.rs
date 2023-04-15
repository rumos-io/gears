use std::path::PathBuf;

use baseapp::BaseApp;
use clap::{arg, value_parser, Arg, ArgAction, ArgMatches, Command};
use database::RocksDB;
use tendermint_abci::ServerBuilder;
use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;

use crate::baseapp::APP_NAME;

mod baseapp;
mod crypto;
mod error;
mod store;
mod types;
mod x;

// TODO:
// 1. remove unwraps
// 2. ip address value_parser / validator

fn get_default_home_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|mut h| {
        h.push(APP_NAME);
        h
    })
}

fn run_init_command(sub_matches: &ArgMatches) {
    let moniker = sub_matches
        .get_one::<String>("moniker")
        .expect("moniker argument is required preventing `None`");

    let default_home_directory = get_default_home_dir();

    let home = sub_matches
        .get_one::<PathBuf>("home")
        .or(default_home_directory.as_ref())
        .unwrap_or_else(|| {
            error!("Home argument not provided and OS does not provide a default config directory");
            std::process::exit(1)
        });

    //TODO: remove these and complete command
    println!("Using {} as moniker", moniker);
    println!("Using directory {} for config and data", home.display());
}

fn run_run_command(matches: &ArgMatches) {
    let host = matches
        .get_one::<String>("host")
        .expect("Host arg has a default value so this cannot be `None`");

    let port = matches
        .get_one::<u16>("port")
        .expect("Port arg has a default value so this cannot be `None`");

    let read_buf_size = matches
        .get_one::<usize>("read_buf_size")
        .expect("Read buf size arg has a default value so this cannot be `None`.");

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

    let default_home_directory = get_default_home_dir();
    let home = matches
        .get_one::<PathBuf>("home")
        .or(default_home_directory.as_ref())
        .unwrap_or_else(|| {
            error!("Home argument not provided and OS does not provide a default config directory");
            std::process::exit(1)
        });
    info!("Using directory {} for config and data", home.display());

    let mut db_dir = home.clone();
    db_dir.push("data");
    db_dir.push("application.db");
    let db = RocksDB::new(db_dir).unwrap_or_else(|e| {
        error!("Could not open database {}", e);
        std::process::exit(1)
    });

    let app = BaseApp::new(db);
    let server = ServerBuilder::new(*read_buf_size)
        .bind(format!("{}:{}", host, port), app)
        .unwrap();
    server.listen().unwrap();
}

fn get_run_command() -> Command {
    Command::new("run")
        .about("Run the full node application")
        .arg(
            arg!(--home)
                .help(format!(
                    "Directory for config and data [default: {}]",
                    get_default_home_dir()
                        .unwrap_or_default()
                        .display()
                        .to_string()
                ))
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(--host)
                .help("Bind the TCP server to this host")
                .action(ArgAction::Set)
                .value_parser(value_parser!(String)) //TODO: replace with IP address value parser
                .default_value("127.0.0.1"),
        )
        .arg(
            arg!(-p - -port)
                .help("Bind the TCP server to this port")
                .action(ArgAction::Set)
                .value_parser(value_parser!(u16))
                .default_value("26658"),
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

fn get_init_command() -> Command {
    Command::new("init")
        .about("Initialize configuration files")
        .arg(Arg::new("moniker").required(true))
        .arg(
            arg!(--home)
                .help(format!(
                    "Directory for config and data [default: {}]",
                    get_default_home_dir()
                        .unwrap_or_default()
                        .display()
                        .to_string()
                ))
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf)),
        )
}

fn main() {
    let cli = Command::new("CLI")
        .subcommand(get_init_command())
        .subcommand(get_run_command())
        .subcommand_required(true);

    let matches = cli.get_matches();

    match matches.subcommand() {
        Some(("init", sub_matches)) => run_init_command(sub_matches),
        Some(("run", sub_matches)) => run_run_command(sub_matches),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}
