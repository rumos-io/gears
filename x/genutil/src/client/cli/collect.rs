use std::{marker::PhantomData, path::PathBuf};

use clap::{ArgAction, Args, ValueHint};
use gears::application::ApplicationInfo;

use crate::collect_txs::{CollectGentxCmd, CollectMode};

#[derive(Args, Debug, Clone)]
pub struct CollectGentxCliAux<AI: ApplicationInfo> {
    #[arg(long, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = AI::home_dir().join("config/gentx/"), help = "directory for config and data")]
    pub gentx_dir: PathBuf,
    #[arg(long, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = AI::home_dir(), help = "directory for config and data")]
    pub home: PathBuf,
    /// Backup original files
    #[arg(long, default_value_t = false)]
    pub backup: bool,
    /// Print edited files to STDOUT
    #[arg(long, default_value_t = false)]
    pub generate_only: bool,

    #[arg(skip)]
    _marker: PhantomData<AI>,
}

impl<AI: ApplicationInfo> TryFrom<CollectGentxCliAux<AI>> for CollectGentxCmd {
    type Error = anyhow::Error;

    fn try_from(
        CollectGentxCliAux {
            gentx_dir,
            home,

            backup,
            generate_only,
            _marker,
        }: CollectGentxCliAux<AI>,
    ) -> Result<Self, Self::Error> {
        if !gentx_dir.exists() {
            Err(anyhow::anyhow!(
                "Failed to find directory: {}",
                gentx_dir.to_string_lossy()
            ))?
        }

        match (gentx_dir.is_dir(), home.is_dir()) {
            (true, true) => match (backup, generate_only) {
                (true, true) => Err(anyhow::anyhow!(
                    "Can't use `backup` and `generate-only` at the same time"
                ))?,
                (true, false) => Ok(Self {
                    gentx_dir,
                    home,
                    mode: CollectMode::File(true),
                }),
                (false, true) => Ok(Self {
                    gentx_dir,
                    home,
                    mode: CollectMode::Display,
                }),
                (false, false) => Ok(Self {
                    gentx_dir,
                    home,
                    mode: CollectMode::File(false),
                }),
            },
            _ => Err(anyhow::anyhow!(
                "`gentx-dir` and `home` args should be dirs"
            )),
        }
    }
}
