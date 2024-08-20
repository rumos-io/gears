use clap::{command, Args, Subcommand};
use collect::CollectGentxCliAux;
use gears::application::ApplicationInfo;

pub mod collect;

#[derive(Args, Debug, Clone)]
pub struct GenesisAuxCli<AI: ApplicationInfo> {
    #[command(subcommand)]
    pub command: BankCommands<AI>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum BankCommands<AI: ApplicationInfo> {
    /// Collect genesis txs and output a genesis.json file
    CollectGentxs(CollectGentxCliAux<AI>),
}
