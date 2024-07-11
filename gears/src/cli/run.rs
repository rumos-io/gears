use std::{marker::PhantomData, net::SocketAddr, path::PathBuf};

use clap::{ArgAction, ValueHint};

use crate::{
    application::ApplicationInfo,
    commands::node::run::{LogLevel, RunCommand},
    config::{DEFAULT_ADDRESS, DEFAULT_REST_LISTEN_ADDR},
    types::base::min_gas::MinGasPrices,
};

/// Run the full node application
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliRunCommand<T: ApplicationInfo> {
    #[arg(long,  global = true, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = T::home_dir(), help = "directory for config and data")]
    pub home: PathBuf,
    #[arg(long, action = ArgAction::Set, default_value_t = DEFAULT_ADDRESS, help = "Application listen address. Overrides any listen address in the config. Default value is used if neither this argument nor a config value is provided" )]
    pub address: SocketAddr,
    #[arg(long, action = ArgAction::Set, default_value_t = DEFAULT_REST_LISTEN_ADDR, help = "Bind the REST server to this address. Overrides any listen address in the config. Default value is used if neither this argument nor a config value is provided")]
    pub rest_listen_addr: SocketAddr,
    #[arg(short, long, action = ArgAction::Set, default_value_t = 1048576, help = "The default server read buffer size, in bytes, for each incoming client connection")]
    pub read_buf_size: usize,
    /// The logging level
    #[arg(long, action = ArgAction::Set, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,
    /// Minimum gas prices to accept for transactions; Any fee in a tx must meet this minimum (e.g. 0.01photino,0.0001stake)
    #[arg(long, action = ArgAction::Set)]
    pub min_gas_prices: Option<MinGasPrices>,

    #[arg(skip)]
    pub _marker: PhantomData<T>,
}

impl<T: ApplicationInfo> From<CliRunCommand<T>> for RunCommand {
    fn from(
        CliRunCommand {
            home,
            address,
            rest_listen_addr,
            read_buf_size,
            _marker,
            log_level,
            min_gas_prices,
        }: CliRunCommand<T>,
    ) -> Self {
        Self {
            home,
            address,
            rest_listen_addr,
            read_buf_size,
            log_level,
            min_gas_prices,
        }
    }
}
