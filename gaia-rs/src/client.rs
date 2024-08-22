use anyhow::Result;
use auth::cli::query::AuthQueryCli;
use bank::cli::{
    query::BankQueryCli,
    tx::{run_bank_tx_command, BankTxCli},
};
use clap::{Args, Subcommand};
use gears::{
    commands::client::tx::ClientTxContext,
    types::{address::AccAddress, tx::Messages},
};
use ibc_rs::client::cli::{
    query::IbcQueryCli,
    tx::{run_ibc_tx_command, IbcTxCli},
};
use staking::cli::{
    query::StakingQueryCli,
    tx::{run_staking_tx_command, StakingTxCli},
};

use crate::message::Message;

#[derive(Debug, Clone, Args)]
pub struct GaiaTxArgs {
    #[command(subcommand)]
    pub command: GaiaTxCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GaiaTxCommands {
    /// Bank transaction subcommands
    Bank(BankTxCli),
    /// Staking transaction subcommands
    Staking(StakingTxCli),
    /// IBC transaction subcommands
    IBC(IbcTxCli),
}

pub fn tx_command_handler(
    _ctx: Option<&ClientTxContext>,
    command: GaiaTxCommands,
    from_address: AccAddress,
) -> Result<Messages<Message>> {
    match command {
        GaiaTxCommands::Bank(args) => run_bank_tx_command(args, from_address)
            .map(Message::Bank)
            .map(Into::into),
        GaiaTxCommands::Staking(args) => run_staking_tx_command(args, from_address)
            .map(Message::Staking)
            .map(Into::into),
        GaiaTxCommands::IBC(args) => run_ibc_tx_command(args, from_address)
            .map(Message::IBC)
            .map(Into::into),
    }
}

#[derive(Subcommand, Debug)]
pub enum GaiaQueryCommands {
    /// Querying commands for the bank module
    Bank(BankQueryCli),
    /// Querying commands for the auth module
    Auth(AuthQueryCli),
    /// Querying commands for the auth module
    Staking(StakingQueryCli),
    /// Querying commands for the ibc module
    Ibc(IbcQueryCli),
}

/// Wraps `GaiaTxCommands`. This structure exists to satisfy interface needs of TxHandler
#[derive(Debug, Clone)]
pub struct WrappedGaiaTxCommands(pub GaiaTxCommands);

impl TryFrom<GaiaTxArgs> for WrappedGaiaTxCommands {
    type Error = anyhow::Error;

    fn try_from(command: GaiaTxArgs) -> Result<Self, Self::Error> {
        Ok(Self(command.command))
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
