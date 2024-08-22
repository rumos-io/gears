use std::path::PathBuf;

use clap::Args;
use staking::cli::tx::CreateValidatorCli;

use crate::gentx::GentxCmd;

#[derive(Args, Debug, Clone)]
pub struct GentxCli {
    pub output: Option<PathBuf>,
    #[command(flatten)]
    pub validator: CreateValidatorCli,
}

impl TryFrom<GentxCli> for GentxCmd {
    type Error = anyhow::Error;

    fn try_from(GentxCli { output, validator }: GentxCli) -> Result<Self, Self::Error> {
        Ok(Self { validator, output })
    }
}
