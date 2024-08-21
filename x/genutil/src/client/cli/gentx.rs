use std::{marker::PhantomData, path::PathBuf};

use clap::{ArgAction, Args, ValueHint};
use gears::application::ApplicationInfo;
use staking::cli::tx::CreateValidatorCli;

#[derive(Args, Debug, Clone)]
pub struct GentxCli<AI: ApplicationInfo> {
    #[arg(long, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = AI::home_dir(), help = "directory for config and data")]
    pub home: PathBuf,
    #[command(flatten)]
    pub moniker: CreateValidatorCli,

    #[arg(skip)]
    _marker: PhantomData<AI>,
}
