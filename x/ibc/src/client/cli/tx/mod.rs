//pub mod recover_client;
//pub mod update;
//pub mod upgrade;
use anyhow::Result;
use clap::{Args, Subcommand};
use gears::core::address::AccAddress;

use crate::message::Message as IbcMessage;

use self::create::CliCreateClient;
// use self::{
//     create::CliCreateClient, recover_client::CliRecoverClient, update::CliUpdateClient,
//     upgrade::CliUpgradeClient,
// };

pub mod create;

#[derive(Args, Debug, Clone)]
pub struct IbcTxCli {
    #[command(subcommand)]
    pub command: IbcCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum IbcCommands {
    #[command(name = "create")]
    ClientCreate(CliCreateClient),
    // #[command(name = "update")]
    // ClientUpdate(CliUpdateClient),
    // #[command(name = "upgrade")]
    // ClientUpgrade(CliUpgradeClient),
    // #[command(name = "recover")]
    // RecoverClientProposal(CliRecoverClient),
}

pub fn run_ibc_tx_command(args: IbcTxCli, _from_address: AccAddress) -> Result<IbcMessage> {
    match args.command {
        IbcCommands::ClientCreate(msg) => create::tx_command_handler(msg),
        // IbcCommands::ClientUpdate(msg) => update::tx_command_handler(msg),
        // IbcCommands::ClientUpgrade(msg) => upgrade::tx_command_handler(msg),
        // IbcCommands::RecoverClientProposal(msg) => recover_client::tx_command_handler(msg),
        // IbcCommands::IBCUpgradeProposal => todo!(),
    }
}
