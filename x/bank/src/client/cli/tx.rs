use anyhow::{Ok, Result};
use clap::{Args, Subcommand};
use gears::types::{
    address::AccAddress,
    base::{coin::UnsignedCoin, coins::UnsignedCoins},
    msg::send::MsgSend,
};

use crate::Message as BankMessage;

#[derive(Args, Debug, Clone)]
pub struct BankTxCli {
    #[command(subcommand)]
    pub command: BankCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum BankCommands {
    /// Send funds from one account to another
    Send {
        /// to address
        to_address: AccAddress,
        /// amount
        amount: UnsignedCoin,
    },
}

pub fn run_bank_tx_command(args: BankTxCli, from_address: AccAddress) -> Result<BankMessage> {
    match &args.command {
        BankCommands::Send { to_address, amount } => Ok(BankMessage::Send(MsgSend {
            from_address,
            to_address: to_address.clone(),
            amount: UnsignedCoins::new(vec![amount.clone()])?,
        })),
    }
}
