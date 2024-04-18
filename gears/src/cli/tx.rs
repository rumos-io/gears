use std::{marker::PhantomData, path::PathBuf, str::FromStr};

use clap::{ArgAction, Subcommand, ValueHint};
use tendermint::types::chain_id::ChainId;

use crate::{
    application::ApplicationInfo,
    commands::client::{
        keys::KeyringBackend,
        tx::{Keyring as TxKeyring, LocalInfo, TxCommand},
    },
    config::DEFAULT_TENDERMINT_RPC_ADDRESS,
    types::base::send::SendCoins,
};

/// Transaction subcommands
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliTxCommand<T: ApplicationInfo, C: Subcommand> {
    /// <host>:<port> to Tendermint RPC interface for this chain
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::Url, default_value_t = DEFAULT_TENDERMINT_RPC_ADDRESS.parse().expect( "const should be valid"))]
    pub node: url::Url,
    /// the network chain-id
    #[arg(long =  "chain-id", global = true, action = ArgAction::Set, default_value_t = ChainId::from_str( "test-chain" ).expect("unreachable: default should be valid"))]
    pub chain_id: ChainId,
    /// TODO
    #[arg(long, global = true, action = ArgAction::Set)]
    pub fee: Option<SendCoins>,

    #[command(subcommand)]
    pub keyring: Keyring<C, T>,

    #[arg(skip)]
    _marker: PhantomData<T>,
}

#[derive(Debug, Clone, ::clap::Args)]
pub struct Local<C: Subcommand, T: ApplicationInfo> {
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = T::home_dir(), help = "directory for config and data")]
    home: PathBuf,

    /// from key
    #[arg(required = true)]
    from_key: String,

    /// select keyring's backend
    #[arg(long = "keyring-backend",  global = true, action = ArgAction::Set, default_value_t = KeyringBackend::File )]
    keyring_backend: KeyringBackend,

    #[command(subcommand)]
    command: C,

    #[arg(skip)]
    _marker: PhantomData<T>,
}

#[derive(Debug, Clone, ::clap::Args)]
pub struct Ledger<C: Subcommand> {
    #[command(subcommand)]
    command: C,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Keyring<C: Subcommand, T: ApplicationInfo> {
    /// Use a Ledger device to sign the transaction
    Ledger(Ledger<C>),
    /// Use a local keyring to source the signing key
    Local(Local<C, T>),
}

impl<T, C, AC, ERR> TryFrom<CliTxCommand<T, C>> for TxCommand<AC>
where
    T: ApplicationInfo,
    C: Subcommand,
    AC: TryFrom<C, Error = ERR>,
{
    type Error = ERR;

    fn try_from(value: CliTxCommand<T, C>) -> Result<Self, Self::Error> {
        let CliTxCommand {
            node,
            chain_id,
            fee,
            _marker,
            keyring,
        } = value;

        let (keyring, command) = match keyring {
            Keyring::Ledger(Ledger { command }) => (TxKeyring::Ledger, command),
            Keyring::Local(Local {
                home,
                from_key,
                keyring_backend,
                command,
                _marker,
            }) => (
                TxKeyring::Local(LocalInfo {
                    keyring_backend,
                    from_key,
                    home,
                }),
                command,
            ),
        };

        Ok(Self {
            node,
            chain_id,
            fee,
            keyring,
            inner: command.try_into()?,
        })
    }
}
