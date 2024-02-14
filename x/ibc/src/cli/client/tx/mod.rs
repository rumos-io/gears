pub mod misbehavior;
pub mod recover_client;
pub mod update;
pub mod upgrade;
use anyhow::Result;
use clap::{Args, Subcommand};
use proto_types::AccAddress;

use crate::message::Message as IbcMessage;

use self::{
    create::MsgCreateClient, misbehavior::MsgSubmitMisbehaviour, recover_client::MsgRecoverClient,
    update::MsgUpdateClient, upgrade::MsgUpgradeClient,
};

pub mod create;

#[derive(Args, Debug)]
pub struct IbcCli {
    #[command(subcommand)]
    command: IbcCommands,
}

#[derive(Subcommand, Debug)]
pub enum IbcCommands {
    ClientCreate(MsgCreateClient),
    ClientUpdate(MsgUpdateClient),
    ClientUpgrade(MsgUpgradeClient),
    Misbehavior(MsgSubmitMisbehaviour),
    RecoverClientProposal(MsgRecoverClient),
    // IBCUpgradeProposal,
}

pub fn run_ibc_tx_command(args: IbcCli, _from_address: AccAddress) -> Result<IbcMessage> {
    match args.command {
        IbcCommands::ClientCreate(msg) => create::tx_command_handler(msg),
        IbcCommands::ClientUpdate(msg) => update::tx_command_handler(msg),
        IbcCommands::ClientUpgrade(msg) => upgrade::tx_command_handler(msg),
        IbcCommands::Misbehavior(msg) => misbehavior::tx_command_handler(msg),
        IbcCommands::RecoverClientProposal(msg) => recover_client::tx_command_handler(msg),
        // IbcCommands::IBCUpgradeProposal => todo!(),
    }
}
