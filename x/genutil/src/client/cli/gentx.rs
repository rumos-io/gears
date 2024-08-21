use std::path::PathBuf;

use clap::Subcommand;
use staking::cli::tx::CreateValidatorCli;

use crate::gentx::GentxCmd;

#[derive(Subcommand, Debug, Clone)]
pub enum GentxCli {
    Validator {
        output: Option<PathBuf>,
        #[command(flatten)]
        validator: CreateValidatorCli,
    },
}

impl TryFrom<GentxCli> for GentxCmd {
    type Error = anyhow::Error;

    fn try_from(cmd: GentxCli) -> Result<Self, Self::Error> {
        match cmd {
            GentxCli::Validator { output, validator } => Ok(Self { validator, output }),
        }
    }
}
