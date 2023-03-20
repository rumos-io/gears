use std::path::PathBuf;

use baseapp::BaseApp;
use database::RocksDB;
use error::AppError;
use structopt::{clap::App, StructOpt};
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

#[derive(Debug, StructOpt)]
struct Opt {
    /// Directory for config and data
    #[structopt(long, about = "Directory for config and data", parse(from_os_str))]
    home: Option<PathBuf>,

    /// Bind the TCP server to this host.
    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Bind the TCP server to this port.
    #[structopt(short, long, default_value = "26658")]
    port: u16,

    /// The default server read buffer size, in bytes, for each incoming client
    /// connection.
    #[structopt(short, long, default_value = "1048576")]
    read_buf_size: usize,

    /// Increase output logging verbosity to DEBUG level.
    #[structopt(short, long)]
    verbose: bool,

    /// Suppress all output logging (overrides --verbose).
    #[structopt(short, long)]
    quiet: bool,
}

fn main() {
    let opt: Opt = Opt::from_args();
    let log_level = if opt.quiet {
        LevelFilter::OFF
    } else if opt.verbose {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    tracing_subscriber::fmt().with_max_level(log_level).init();

    let home = opt
        .home
        .or(dirs::config_dir().map(|mut h| {
            h.push(APP_NAME);
            h
        }))
        .unwrap_or_else(|| {
            error!("Home argument not provided and OS does not provide a default config directory");
            std::process::exit(1)
        });
    info!("Using directory {} for config and data.", home.display());
    let mut db_dir = home.clone();
    db_dir.push("data");
    db_dir.push("application.db");
    let db = RocksDB::new(db_dir).unwrap_or_else(|e| {
        error!("Could not open database {}", e);
        std::process::exit(1)
    });

    let app = BaseApp::new(db);
    let server = ServerBuilder::new(opt.read_buf_size)
        .bind(format!("{}:{}", opt.host, opt.port), app)
        .unwrap();
    server.listen().unwrap();
}
