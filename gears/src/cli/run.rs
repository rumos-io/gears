use std::{marker::PhantomData, net::SocketAddr, path::PathBuf};

use clap::{ArgAction, ValueHint};

use crate::{
    application::ApplicationInfo,
    baseapp::run::RunCommand,
    config::{DEFAULT_ADDRESS, DEFAULT_REST_LISTEN_ADDR},
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
    #[arg(short, long, action = ArgAction::SetTrue, help = "Increase output logging verbosity to DEBUG level" )]
    pub verbose: bool,
    #[arg(short, long, action = ArgAction::SetTrue, help = format!("Suppress all output logging (overrides --{})", stringify!( verbose )) )]
    pub quiet: bool,

    #[arg(skip)]
    pub _marker: PhantomData<T>,
}

impl<T: ApplicationInfo> From<CliRunCommand<T>> for RunCommand {
    fn from(value: CliRunCommand<T>) -> Self {
        let CliRunCommand {
            home,
            address,
            rest_listen_addr,
            read_buf_size,
            verbose,
            quiet,
            _marker,
        } = value;

        Self {
            home,
            address,
            rest_listen_addr,
            read_buf_size,
            verbose,
            quiet,
        }
    }
}
