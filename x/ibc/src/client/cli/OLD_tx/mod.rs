use anyhow::Result;
use clap::{Args, Subcommand};
use gears::core::address::AccAddress;

use crate::{ics02_client::client::cli::tx::ClientTxCli, message::Message as IbcMessage};

#[derive(Args, Debug, Clone)]
pub struct IbcTxCli {
    #[command(subcommand)]
    pub command: IbcCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum IbcCommands {
    /// IBC client transaction subcommands
    Client(ClientTxCli),
}

pub fn run_ibc_tx_command(args: IbcTxCli, from_address: AccAddress) -> Result<IbcMessage> {
    match args.command {
        IbcCommands::Client(args) => {
            crate::ics02_client::client::cli::tx::tx_command_handler(args, from_address)
        }
    }
}
