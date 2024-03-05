use std::{marker::PhantomData, path::PathBuf};

use clap::ArgAction;
use rand::distributions::DistString;
use tendermint::informal::chain::Id;

use crate::client::init::InitCommand;

pub trait ApplicationCli {
    const APP_NAME: &'static str;
    const APP_VERSION: &'static str;
}

pub fn home_dir<T : ApplicationCli>() -> std::path::PathBuf
{
    dirs::home_dir().expect( "failed to get home dir").join( T ::APP_NAME)
}

pub const RAND_LENGHT : usize = 10;

pub fn rand_string() -> String
{
    rand::distributions::Alphanumeric.sample_string(&mut  rand::thread_rng(), RAND_LENGHT)
}

pub struct TmpImpl;

impl ApplicationCli for TmpImpl
{
    const APP_NAME: &'static str = "";

    const APP_VERSION: &'static str = "";
}

#[derive(Debug, Clone, ::clap::Args)]
pub struct CliInitCommand< T : ApplicationCli> {
    #[arg(long, action = ArgAction::Set, default_value_os_t = crate::cli::home_dir:: <T>(), help = format!( "directory for config and data (default \"{:?}\")", crate::cli::home_dir::<T>() ))]
    pub home: PathBuf,
    #[arg(required = true)]
    pub moniker: String,
    #[arg(long =  "chain-id",  action = ArgAction::Set, default_value_t = Id::try_from( crate::cli::rand_string() ).expect("rand should be valid"), help = "genesis file chain-id, if left blank will be randomly created",)]
    pub chain_id: Id,

    #[arg(skip)]
    _marker : PhantomData<T>,
}

impl<T : ApplicationCli> From<CliInitCommand<T>> for InitCommand
{
    fn from(value: CliInitCommand<T>) -> Self {
        let CliInitCommand { home, moniker, chain_id, _marker } = value;

        Self { home, moniker, chain_id }    
    }
}


#[derive(Debug, Clone, ::clap::Subcommand)]
pub enum ApplicationCommands<T : ApplicationCli>
{
    Init( CliInitCommand<T>),
}