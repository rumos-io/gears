use clap::{Args, Subcommand};
use collect::CollectGentxCliAux;
use gears::application::ApplicationInfo;

use crate::cmd::GenesisCmd;

pub mod collect;
pub mod gentx;

/// variety of genesis utility functionalities for usage within a blockchain application
#[derive(Args, Debug, Clone)]
pub struct GenesisAuxCli<AI: ApplicationInfo> {
    #[command(subcommand)]
    pub command: GenesisCommands<AI>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GenesisCommands<AI: ApplicationInfo> {
    /// Collect genesis txs and output a genesis.json file
    CollectGentxs(CollectGentxCliAux<AI>),
}

impl<AI: ApplicationInfo> TryFrom<GenesisAuxCli<AI>> for GenesisCmd {
    type Error = anyhow::Error;

    fn try_from(value: GenesisAuxCli<AI>) -> Result<Self, Self::Error> {
        Ok(match value.command {
            GenesisCommands::CollectGentxs(cmd) => Self::CollectGentxs(cmd.try_into()?),
        })
    }
}
