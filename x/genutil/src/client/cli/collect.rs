use std::{marker::PhantomData, path::PathBuf};

use clap::{ArgAction, Args, ValueHint};
use gears::application::ApplicationInfo;

use crate::collect_txs::CollectGentxCmd;

#[derive(Args, Debug, Clone)]
pub struct CollectGentxCliAux<AI: ApplicationInfo> {
    #[arg(long, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = AI::home_dir().join("config/gentx/"), help = "directory for config and data")]
    pub gentx_dir: PathBuf,
    #[arg(long, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = AI::home_dir(), help = "directory for config and data")]
    pub home: PathBuf,
    #[arg(required = true)]
    pub moniker: String,

    #[arg(skip)]
    _marker: PhantomData<AI>,
}

impl<AI: ApplicationInfo> TryFrom<CollectGentxCliAux<AI>> for CollectGentxCmd {
    type Error = anyhow::Error;

    fn try_from(
        CollectGentxCliAux {
            gentx_dir,
            home,
            moniker,
            _marker,
        }: CollectGentxCliAux<AI>,
    ) -> Result<Self, Self::Error> {
        if !gentx_dir.exists() {
            Err(anyhow::anyhow!(
                "Failed to find folder: {}",
                gentx_dir.to_string_lossy()
            ))?
        }

        match (gentx_dir.is_dir(), home.is_dir()) {
            (true, true) => Ok(Self {
                gentx_dir,
                home,
                moniker,
            }),
            _ => Err(anyhow::anyhow!(
                "`gentx-dir` and `home` args should be dirs"
            )),
        }
    }
}
