use std::{marker::PhantomData, path::PathBuf};

use clap::{ArgAction, ValueHint};
use tendermint::informal::chain::Id;

use crate::{client::init::InitCommand, ApplicationInfo};

use super::utils::{home_dir, rand_string};

/// Initialize configuration files
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliInitCommand<T: ApplicationInfo> {
    #[arg(long,  global = true, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = home_dir:: <T>(), help = "directory for config and data")]
    pub home: PathBuf,
    #[arg(required = true)]
    pub moniker: String,
    #[arg(long =  "chain-id",  action = ArgAction::Set, help = "genesis file chain-id, if left blank will be randomly created",)]
    pub chain_id: Option<Id>,

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
            chain_id: chain_id
                .unwrap_or(Id::try_from(rand_string()).expect("rand should be valid")),
        }
    }
}
