use std::{marker::PhantomData, path::PathBuf};

use clap::{ArgAction, Args, ValueHint};
use gears::{application::ApplicationInfo, types::address::AccAddress};
use staking::cli::tx::CreateValidatorCli;

#[derive(Args, Debug, Clone)]
pub struct GentxCli<AI: ApplicationInfo> {
    #[arg(long, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = AI::home_dir(), help = "directory for config and data")]
    pub home: PathBuf,
    #[command(flatten)]
    pub validator: CreateValidatorCli,
    #[arg(long)]
    pub from_address: AccAddress,

    #[arg(skip)]
    _marker: PhantomData<AI>,
}

// impl<AI: ApplicationInfo> TryFrom<GentxCli<AI>> for GentxCmd {
//     type Error = anyhow::Error;

//     fn try_from(
//         GentxCli {
//             home,
//             validator,
//             from_address,
//             _marker,
//         }: GentxCli<AI>,
//     ) -> Result<Self, Self::Error> {
//         Ok(Self {
//             home,
//             validator: validator.try_into_cmd(from_address)?,
//         })
//     }
// }
