use std::{marker::PhantomData, path::PathBuf};

use clap::{ArgAction, Subcommand, ValueHint};
use proto_messages::cosmos::base::v1beta1::SendCoins;

use tendermint::informal::chain::Id;

use crate::{
    app_v2::ApplicationInfo,
    client::{keys::KeyringBackend, tx::TxCommand},
    config::DEFAULT_TENDERMINT_RPC_ADDRESS,
};

/// Transaction subcommands
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliTxCommand<T: ApplicationInfo, C: Subcommand> {
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = T::home_dir(), help = "directory for config and data")]
    pub home: PathBuf,
    /// <host>:<port> to Tendermint RPC interface for this chain
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::Url, default_value_t = DEFAULT_TENDERMINT_RPC_ADDRESS.parse().expect( "const should be valid"))]
    pub node: url::Url,
    /// From key
    #[arg(required = true)]
    pub from_key: String,
    /// file chain-id
    #[arg(long =  "chain-id", global = true, action = ArgAction::Set, default_value_t = Id::try_from( "test-chain" ).expect("unrechable: default should be valid"))]
    pub chain_id: Id,
    /// TODO
    #[arg(long, global = true, action = ArgAction::Set)]
    pub fee: Option<SendCoins>,
    /// select keyring's backend
    #[arg(long = "keyring-backend",  global = true, action = ArgAction::Set, default_value_t = KeyringBackend::File )]
    pub keyring_backend: KeyringBackend,

    #[command(subcommand)]
    pub command: C,

    #[arg(skip)]
    _marker: PhantomData<T>,
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
            home,
            node,
            from_key,
            chain_id,
            fee,
            keyring_backend,
            _marker,
            command,
        } = value;

        Ok(Self {
            home,
            node,
            from_key,
            chain_id,
            fee,
            keyring_backend,
            inner: command.try_into()?,
        })
    }
}
