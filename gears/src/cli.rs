use std::{marker::PhantomData, path::PathBuf};

use clap::ArgAction;
use rand::distributions::DistString;
use tendermint::informal::chain::Id;

use crate::{client::init::InitCommand, ApplicationCommands};

pub trait ApplicationCli {
    const APP_NAME: &'static str;
    const APP_VERSION: &'static str;
}

pub(crate) fn home_dir<T : ApplicationCli>() -> std::path::PathBuf
{
    dirs::home_dir().expect( "failed to get home dir").join( T ::APP_NAME)
}

pub(crate) const RAND_LENGHT : usize = 10;

pub(crate) fn rand_string() -> String
{
    rand::distributions::Alphanumeric.sample_string(&mut  rand::thread_rng(), RAND_LENGHT)
}

#[derive(Debug, Clone)]
pub struct DefaultCli;

impl ApplicationCli for DefaultCli
{
    const APP_NAME: &'static str = ".gaia";
    const APP_VERSION: &'static str = "1"; // TODO: GIT_HASH
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
pub enum CliApplicationCommands<T : ApplicationCli>
{
    Init( CliInitCommand<T>),
}

#[derive(Debug, Clone, ::clap::Parser)]
pub struct CliApplicationArgs<T : ApplicationCli + Clone + Send + Sync>
{
    #[command(subcommand, value_parser = value_parser!(PhantomData))]
    command : CliApplicationCommands<T>, 
}

impl<T : ApplicationCli + Clone + Send + Sync> From<CliApplicationArgs<T>> for ApplicationCommands
{
    fn from(value: CliApplicationArgs<T>) -> Self {
        match value.command
        {
            CliApplicationCommands::Init( cmd ) => Self::Init( cmd.into() ),
        }
    }
}