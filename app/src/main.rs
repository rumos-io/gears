use std::path::PathBuf;

use baseapp::BaseApp;
use clap::Parser;
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

const a: &str = "hello world";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub enum Args {
    /// Run the full node application with Tendermint
    Run {
        /// Directory for config and data
        #[clap(long, default_value = None, value_name = "FILE")]
        home: Option<PathBuf>,

        /// Bind the TCP server to this host.
        #[clap(long, default_value = "127.0.0.1")]
        host: String,

        /// Bind the TCP server to this port.
        #[clap(short, long, default_value = "26658")]
        port: u16,

        /// The default server read buffer size, in bytes, for each incoming client
        /// connection.
        #[clap(short, long, default_value = "1048576")]
        read_buf_size: usize,

        /// Increase output logging verbosity to DEBUG level.
        #[clap(short, long)]
        verbose: bool,

        /// Suppress all output logging (overrides --verbose).
        #[clap(short, long)]
        quiet: bool,
    },
    /// Initialize configuration files
    Init {
        /// Moniker
        #[clap(index = 1)]
        moniker: String,

        /// Directory for config and data
        #[clap(long, default_value = None, value_name = "FILE")]
        home: Option<PathBuf>,
    },
}

fn main() {
    let opt = Args::parse();

    match opt {
        Args::Run {
            home,
            host,
            port,
            read_buf_size,
            verbose,
            quiet,
        } => {
            let log_level = if quiet {
                LevelFilter::OFF
            } else if verbose {
                LevelFilter::DEBUG
            } else {
                LevelFilter::INFO
            };

            tracing_subscriber::fmt().with_max_level(log_level).init();

            let home = home
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
            let server = ServerBuilder::new(read_buf_size)
                .bind(format!("{}:{}", host, port), app)
                .unwrap();
            server.listen().unwrap();
        }
        Args::Init { moniker, home } => {
            let home = home
                .or(dirs::config_dir().map(|mut h| {
                    h.push(APP_NAME);
                    h
                }))
                .unwrap_or_else(|| {
                    error!("Home argument not provided and OS does not provide a default config directory");
                    std::process::exit(1)
                });
        }
    };
}
