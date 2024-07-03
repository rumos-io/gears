use crate::{Message, MsgUnjail};
use anyhow::Result;
use clap::{Args, Subcommand};
use gears::types::address::AccAddress;

#[derive(Args, Debug, Clone)]
/// Unjail validator previously jailed for downtime
pub struct SlashingTxCli {
    #[command(subcommand)]
    pub command: SlashingCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SlashingCommands {
    Unjail,
}

pub fn run_staking_tx_command(args: SlashingTxCli, from_address: AccAddress) -> Result<Message> {
    match &args.command {
        SlashingCommands::Unjail => Ok(Message::Unjail(MsgUnjail {
            from_address: from_address.clone(),
            validator_address: from_address.clone().into(),
        })),
    }
}
