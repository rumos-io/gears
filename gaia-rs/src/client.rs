use anyhow::Result;
use auth::cli::query::{run_auth_query_command, AuthQueryCli};
use bank::cli::{
    query::{run_bank_query_command, BankQueryCli},
    tx::{run_bank_tx_command, BankTxCli},
};
use clap::Subcommand;
use ibc::cli::client::{
    query::{run_ibc_query_command, IbcQueryCli},
    tx::{run_ibc_tx_command, IbcTxCli},
};
use proto_types::AccAddress;
use tendermint::informal::block::Height;

use crate::message::Message;

#[derive(Subcommand, Debug, Clone)]
pub enum GaiaCommands {
    /// Bank transaction subcommands
    Bank(BankTxCli),
    /// IBC transaction subcommands
    IBC(IbcTxCli),
}

pub fn tx_command_handler(command: GaiaCommands, from_address: AccAddress) -> Result<Message> {
    match command {
        GaiaCommands::Bank(args) => run_bank_tx_command(args, from_address).map(Message::Bank),
        GaiaCommands::IBC(args) => run_ibc_tx_command(args, from_address).map(Message::Ibc),
    }
}

#[derive(Subcommand, Debug)]
pub enum QueryCommands {
    /// Querying commands for the bank module
    Bank(BankQueryCli),
    /// Querying commands for the auth module
    Auth(AuthQueryCli),
    Ibc(IbcQueryCli),
}

pub fn query_command_handler(
    command: QueryCommands,
    node: &str,
    height: Option<Height>,
) -> Result<()> {
    let res = match command {
        QueryCommands::Bank(args) => run_bank_query_command(args, node, height),
        QueryCommands::Auth(args) => run_auth_query_command(args, node, height),
        QueryCommands::Ibc(args) => run_ibc_query_command(args, node, height),
    }?;

    println!("{}", res);
    Ok(())
}
