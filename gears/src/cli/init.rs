use std::{marker::PhantomData, path::PathBuf};

use clap::{ArgAction, ValueHint};
use tendermint::informal::chain::Id;

use crate::{client::init::InitCommand, ApplicationInfo};

/// Initialize configuration files
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliInitCommand<T: ApplicationInfo> {
    #[arg(long,  global = true, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = T::home_dir(), help = "directory for config and data")]
    pub home: PathBuf,
    #[arg(required = true)]
    pub moniker: String,
    #[arg(long =  "chain-id",  action = ArgAction::Set, default_value_t = Id::try_from( "test-chain" ).expect("unrechable: default should be valid"), help = "genesis file chain-id",)]
    pub chain_id: Id,

    #[arg(skip)]
    _marker: PhantomData<T>,
}

impl<T: ApplicationInfo> From<CliInitCommand<T>> for InitCommand {
    fn from(value: CliInitCommand<T>) -> Self {
        let CliInitCommand {
            home,
            moniker,
            chain_id,
            _marker,
        } = value;

        Self {
            home,
            moniker,
            chain_id,
        }
    }
}
