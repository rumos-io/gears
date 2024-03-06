use std::{marker::PhantomData, net::SocketAddr, path::PathBuf};

use clap::ArgAction;
use rand::distributions::DistString;
use tendermint::informal::chain::Id;

use crate::{baseapp::run::RunCommand, client::init::InitCommand, config::{DEFAULT_ADDRESS, DEFAULT_REST_LISTEN_ADDR}, ApplicationCommands, ApplicationInfo};


pub(crate) fn home_dir<T : ApplicationInfo>() -> std::path::PathBuf
{
    dirs::home_dir().expect( "failed to get home dir").join( T ::APP_NAME)
}

pub(crate) const RAND_LENGHT : usize = 10;

pub(crate) fn rand_string() -> String
{
    rand::distributions::Alphanumeric.sample_string(&mut  rand::thread_rng(), RAND_LENGHT)
}

#[derive(Debug, Clone, ::clap::Args)]
pub struct CliInitCommand< T : ApplicationInfo> {
    #[arg(long, action = ArgAction::Set, default_value_os_t = crate::cli::home_dir:: <T>(), help = "directory for config and data")]
    pub home: PathBuf,
    #[arg(required = true)]
    pub moniker: String,
    #[arg(long =  "chain-id",  action = ArgAction::Set, default_value_t = Id::try_from( crate::cli::rand_string() ).expect("rand should be valid"), help = "genesis file chain-id, if left blank will be randomly created",)]
    pub chain_id: Id,

    #[arg(skip)]
    _marker : PhantomData<T>,
}

impl<T : ApplicationInfo> From<CliInitCommand<T>> for InitCommand
{
    fn from(value: CliInitCommand<T>) -> Self {
        let CliInitCommand { home, moniker, chain_id, _marker } = value;

        Self { home, moniker, chain_id }    
    }
}

#[derive(Debug, Clone, ::clap::Args)]
pub struct CliRunCommand< T : ApplicationInfo>
{
    #[arg(long, action = ArgAction::Set, default_value_os_t = crate::cli::home_dir:: <T>(), help = "directory for config and data")]
    pub home: PathBuf,
    #[arg(long, action = ArgAction::Set, default_value_t = DEFAULT_ADDRESS, help = "Application listen address. Overrides any listen address in the config. Default value is used if neither this argument nor a config value is provided" )]
    address : SocketAddr,
    #[arg(long, action = ArgAction::Set, default_value_t = DEFAULT_REST_LISTEN_ADDR, help = "Bind the REST server to this address. Overrides any listen address in the config. Default value is used if neither this argument nor a config value is provided")]
    rest_listen_addr : SocketAddr,
    #[arg(short, long, action = ArgAction::Set, default_value_t = 1048576, help = "The default server read buffer size, in bytes, for each incoming client connection")]
    read_buf_size : usize,
    #[arg(short, long, action = ArgAction::SetTrue, help = "Increase output logging verbosity to DEBUG level" )]
    verbose : bool,
    #[arg(short, long, action = ArgAction::SetTrue, help = format!("Suppress all output logging (overrides --{})", stringify!( verbose )) )]
    quiet : bool,

    #[arg(skip)]
    _marker : PhantomData<T>,
}


#[derive(Debug, Clone, ::clap::Subcommand)]
pub enum CliApplicationCommands<T : ApplicationInfo>
{
    Init( CliInitCommand<T>),
    Run( CliRunCommand<T>),
}

impl<T : ApplicationInfo> From<CliRunCommand<T>> for RunCommand
{
    fn from(value: CliRunCommand<T>) -> Self {
        let CliRunCommand { home, address, rest_listen_addr, read_buf_size, verbose, quiet, _marker } = value;

        Self { home, address, rest_listen_addr, read_buf_size, verbose, quiet }
    }
}

#[derive(Debug, Clone, ::clap::Parser)]
pub struct CliApplicationArgs<T : ApplicationInfo + Clone + Send + Sync>
{
    #[command(subcommand, value_parser = value_parser!(PhantomData))]
    command : CliApplicationCommands<T>, 
}

impl<T : ApplicationInfo + Clone + Send + Sync> From<CliApplicationArgs<T>> for ApplicationCommands
{
    fn from(value: CliApplicationArgs<T>) -> Self {
        match value.command
        {
            CliApplicationCommands::Init( cmd ) => Self::Init( cmd.into() ),
            CliApplicationCommands::Run( cmd ) => Self::Run( cmd.into() ),
        }
    }
}