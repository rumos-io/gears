use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct MintQueryCli {
    #[command(subcommand)]
    pub command: MintCommands,
}

#[derive(Subcommand, Debug)]
pub enum MintCommands {
    /// Query the current minting parameters
    Params,
    /// Query the current minting inflation value
    Inflation,
    /// Query the current minting annual provisions value
    AnnualProvisions,
}
