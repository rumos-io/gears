use anyhow::Result;
use clap::{Args, Subcommand};

use gears::{
    core::address::AccAddress,
    // types::{
    //     base::{coin::Coin, send::SendCoins},
    //     msg::send::MsgSend,
    // },
};

use crate::Message as StakingMessage;

#[derive(Args, Debug, Clone)]
pub struct StakingTxCli {
    #[command(subcommand)]
    pub command: StakingCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum StakingCommands {
    // TODO: fill
    Todo,
}

pub fn run_staking_tx_command(
    args: StakingTxCli,
    _from_address: AccAddress,
) -> Result<StakingMessage> {
    match &args.command {
        StakingCommands::Todo => todo!(),
    }
}
