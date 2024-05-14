use std::{marker::PhantomData, path::PathBuf};

use clap::{ArgAction, ValueHint};

use crate::{
    application::ApplicationInfo,
    commands::node::genesis::GenesisCommand,
    types::{address::AccAddress, base::send::SendCoins},
};

/// Add a genesis account to genesis.json. The provided account must specify the
/// account address and a list of initial coins. The list of initial tokens must contain valid denominations.
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliGenesisCommand<T: ApplicationInfo> {
    #[arg(long, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = T::home_dir(), help = "directory for config and data")]
    home: PathBuf,
    #[arg(required = true)]
    address: AccAddress,
    #[arg(required = true)]
    coins: SendCoins,

    #[arg(skip)]
    _marker: PhantomData<T>,
}

impl<T: ApplicationInfo> From<CliGenesisCommand<T>> for GenesisCommand {
    fn from(value: CliGenesisCommand<T>) -> Self {
        let CliGenesisCommand {
            home,
            address,
            coins,
            _marker,
        } = value;

        Self {
            home,
            address,
            coins,
        }
    }
}
