use std::{marker::PhantomData, net::SocketAddr, path::PathBuf};

use clap::{ArgAction, Args, Subcommand, ValueHint};
use clap_complete::Shell;
use proto_messages::cosmos::base::v1beta1::SendCoins;
use proto_types::AccAddress;
use rand::distributions::DistString;
use tendermint::informal::chain::Id;

use crate::{baseapp::run::RunCommand, client::{genesis_account::GenesisCommand, init::InitCommand, keys::{AddKeyCommand, KeyCommand, KeyringBackend}, tx::TxCommand}, config::{DEFAULT_ADDRESS, DEFAULT_REST_LISTEN_ADDR, DEFAULT_TENDERMINT_RPC_ADDRESS}, ApplicationCommands, ApplicationInfo, NilAuxCommand};


pub(crate) fn home_dir<T : ApplicationInfo>() -> std::path::PathBuf
{
    dirs::home_dir().expect( "failed to get home dir").join( T ::APP_NAME)
}

pub(crate) const RAND_LENGTH : usize = 10;

pub(crate) fn rand_string() -> String
{
    rand::distributions::Alphanumeric.sample_string(&mut  rand::thread_rng(), RAND_LENGTH)
}

/// Initialize configuration files
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliInitCommand< T : ApplicationInfo> {
    #[arg(long,  global = true, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = crate::cli::home_dir:: <T>(), help = "directory for config and data")]
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

/// Run the full node application
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliRunCommand< T : ApplicationInfo>
{
    #[arg(long,  global = true, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = crate::cli::home_dir:: <T>(), help = "directory for config and data")]
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
#[command( about = "Manage your application's keys")]
pub enum CliKeyCommand< T : ApplicationInfo> {
    Add( CliAddKeyCommand<T>)
}

#[derive(Debug, Clone, ::clap::Args)]
#[command(about = "Add a private key (either newly generated or recovered) saving it to <NAME> file")]
pub struct CliAddKeyCommand< T : ApplicationInfo>
{
    #[arg(required = true)]
    name: String,
    #[arg(short, long, action = ArgAction::SetTrue, help = "Provide seed phrase to recover existing key instead of creating" )]
    recover: bool,
    #[arg(long, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = crate::cli::home_dir:: <T>(), help = "directory for config and data")]
    home: PathBuf,
    /// select keyring's backend
    #[arg(long = "keyring-backend",  action = ArgAction::Set, default_value_t = KeyringBackend::File )]
    keyring_backend: KeyringBackend,

    #[arg(skip)]
    _marker : PhantomData<T>,
}

impl< T : ApplicationInfo> From< CliAddKeyCommand<T>> for  AddKeyCommand
{
    fn from(value: CliAddKeyCommand<T>) -> Self {
        let CliAddKeyCommand { name, recover, home, keyring_backend, _marker } = value;

        Self{ name, recover, home, keyring_backend }
    }
}

impl< T : ApplicationInfo> From<CliKeyCommand<T>> for KeyCommand
{
    fn from(value: CliKeyCommand<T>) -> Self {
        match value
        {
            CliKeyCommand::Add( cmd ) => KeyCommand::Add( cmd.into() ),
        }
    }
}

/// Add a genesis account to genesis.json. The provided account must specify the 
/// account address and a list of initial coins. The list of initial tokens must contain valid denominations.
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliGenesisCommand< T : ApplicationInfo> {
    #[arg(long, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = crate::cli::home_dir:: <T>(), help = "directory for config and data")]
    home: PathBuf,
    #[arg( required = true)]
    address: AccAddress,
    #[arg( required = true)]
    coins: SendCoins,

    #[arg(skip)]
    _marker : PhantomData<T>,
}

impl< T : ApplicationInfo>  From<CliGenesisCommand<T>> for GenesisCommand
{
    fn from(value: CliGenesisCommand<T>) -> Self {
        let CliGenesisCommand { home, address, coins, _marker } = value;
        
        Self { home, address, coins }
    }
}

#[derive(Debug, Clone, ::clap::Args)]
#[command( about = format!( "{} doesn't handle any aux command", T::APP_NAME))]
pub struct CliNilAuxCommand< T : ApplicationInfo>
{
    #[arg(skip)]
    _marker : PhantomData<T>,
}

impl< T : ApplicationInfo> From<CliNilAuxCommand<T>> for NilAuxCommand
{
    fn from(_value: CliNilAuxCommand<T>) -> Self {
        Self
    }
}

/// Transaction subcommands
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliTxCommand< T : ApplicationInfo, C : Subcommand>
{
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = crate::cli::home_dir:: <T>(), help = "directory for config and data")]
    pub home: PathBuf,
    /// <host>:<port> to Tendermint RPC interface for this chain
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::Url, default_value_t = DEFAULT_TENDERMINT_RPC_ADDRESS.parse().expect( "const should be valid"))]
    pub node : tendermint::rpc::Url,
    /// From key
    #[arg( required = true)]
    pub from_key : String,
    /// file chain-id, if left blank will be randomly created
    #[arg(long =  "chain-id", global = true, action = ArgAction::Set, default_value_t = Id::try_from( crate::cli::rand_string() ).expect("rand should be valid"),)]
    pub chain_id: Id,
    /// TODO
    #[arg(long, global = true, action = ArgAction::Set)]
    pub fee : Option<SendCoins>,
    /// select keyring's backend
    #[arg(long = "keyring-backend",  global = true, action = ArgAction::Set, default_value_t = KeyringBackend::File )]
    pub keyring_backend: KeyringBackend,

    #[command(subcommand)]
    pub command : C,

    #[arg(skip)]
    _marker : PhantomData<T>,
}

impl< T, C, AC> From<CliTxCommand<T, C>> for TxCommand<AC>
where
    T : ApplicationInfo, 
    C : Subcommand,
    AC : From<C>
{
    fn from(value: CliTxCommand<T, C>) -> Self {
        let CliTxCommand { home, node, from_key, chain_id, fee, keyring_backend, _marker, command } = value;

        Self { home, node, from_key, chain_id, fee, keyring_backend, inner: command.into() }
    }
}

#[derive(Debug, Clone, ::clap::Subcommand)]
pub enum CliApplicationCommands<T, CliAUX, CliTX>
where
    T : ApplicationInfo,
    CliAUX : Args,
    CliTX : Subcommand,
{
    Init( CliInitCommand<T>),
    Run( CliRunCommand<T>),
    #[command(subcommand)]
    Keys( CliKeyCommand<T>),
    GenesisAdd( CliGenesisCommand<T>),
    Aux( CliAUX ),
    Tx( CliTxCommand<T, CliTX> ),
}

impl<T : ApplicationInfo> From<CliRunCommand<T>> for RunCommand
{
    fn from(value: CliRunCommand<T>) -> Self {
        let CliRunCommand { home, address, rest_listen_addr, read_buf_size, verbose, quiet, _marker } = value;

        Self { home, address, rest_listen_addr, read_buf_size, verbose, quiet }
    }
}

#[derive(Debug, Clone, ::clap::Parser)]
#[command(name = T::APP_NAME)]
pub struct CliApplicationArgs<T, CliAUX, CliTX>
where
    T : ApplicationInfo,
    CliAUX : Args,
    CliTX : Subcommand,
{
    #[command(subcommand, value_parser = value_parser!(PhantomData))]
    pub command : Option<CliApplicationCommands<T, CliAUX, CliTX>>, 
    /// If provided, outputs the completion file for given shell
    #[clap(long = "completion")]
    pub completion: Option<Shell>,
}

impl<T : ApplicationInfo, CliAUX, AUX, CliTX, TX> From<CliApplicationCommands<T, CliAUX, CliTX>> for ApplicationCommands<AUX,TX>
where 
    CliAUX : Args + Into<AUX>,
    CliTX : Subcommand, 
    TX: From<CliTX>
{
    fn from(value: CliApplicationCommands<T, CliAUX, CliTX>) -> Self {
        match value
        {
            CliApplicationCommands::Init( cmd ) => Self::Init( cmd.into() ),
            CliApplicationCommands::Run( cmd ) => Self::Run( cmd.into() ),
            CliApplicationCommands::Keys( cmd ) => Self::Keys( cmd.into() ),
            CliApplicationCommands::GenesisAdd( cmd ) => Self::GenesisAdd( cmd.into() ),
            CliApplicationCommands::Aux( cmd ) => Self::Aux( cmd.into() ),
            CliApplicationCommands::Tx( cmd ) => Self::Tx( cmd.into() ),
        }
    }
}