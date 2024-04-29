use anyhow::Result;
use auth::cli::query::AuthQueryCli;
use bank::cli::{
    query::BankQueryCli,
    tx::{run_bank_tx_command, BankTxCli},
};
use clap::Subcommand;
use gears::core::address::AccAddress;

use crate::message::Message;

#[derive(Subcommand, Debug, Clone)]
pub enum GaiaTxCommands {
    /// Bank transaction subcommands
    Bank(BankTxCli),
    // /// IBC transaction subcommands
    // IBC(IbcTxCli),
}

pub fn tx_command_handler(command: GaiaTxCommands, from_address: AccAddress) -> Result<Message> {
    match command {
        GaiaTxCommands::Bank(args) => run_bank_tx_command(args, from_address).map(Message::Bank),
        // GaiaTxCommands::IBC(args) => run_ibc_tx_command(args, from_address).map(Message::Ibc),
    }
}

#[derive(Subcommand, Debug)]
pub enum GaiaQueryCommands {
    /// Querying commands for the bank module
    Bank(BankQueryCli),
    /// Querying commands for the auth module
    Auth(AuthQueryCli),
    // Ibc(IbcQueryCli),
}

/// Wraps `GaiaTxCommands`. This structure exists to satisfy interface needs of TxHandler
#[derive(Debug, Clone)]
pub struct WrappedGaiaTxCommands(pub GaiaTxCommands);

impl TryFrom<GaiaTxCommands> for WrappedGaiaTxCommands {
    type Error = anyhow::Error;

    fn try_from(command: GaiaTxCommands) -> Result<Self, Self::Error> {
        Ok(Self(command))
    }
}

/// Wraps `GaiaQueryCommands`. This structure exists to satisfy interface needs of TxHandler
#[derive(Debug)]
pub struct WrappedGaiaQueryCommands(pub GaiaQueryCommands);

impl TryFrom<GaiaQueryCommands> for WrappedGaiaQueryCommands {
    type Error = anyhow::Error;

    fn try_from(command: GaiaQueryCommands) -> Result<Self, Self::Error> {
        Ok(Self(command))
    }
}
