use anyhow::{Ok, Result};
use clap::{Args, Subcommand};

use proto_messages::cosmos::{
    bank::v1beta1::MsgSend,
    base::v1beta1::{Coin, SendCoins},
};
use proto_types::AccAddress;

use crate::Message as BankMessage;

#[derive(Args, Debug)]
pub struct BankTxCli {
    #[command(subcommand)]
    command: BankCommands,
}

#[derive(Subcommand, Debug)]
pub enum BankCommands {
    /// Send funds from one account to another
    Send {
        /// to address
        to_address: AccAddress,
        /// amount
        amount: Coin,
    },
}

pub fn run_bank_tx_command(args: BankTxCli, from_address: AccAddress) -> Result<BankMessage> {
    match args.command {
        BankCommands::Send { to_address, amount } => Ok(BankMessage::Send(MsgSend {
            from_address,
            to_address,
            amount: SendCoins::new(vec![amount])?,
        })),
    }
}
