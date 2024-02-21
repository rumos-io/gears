use anyhow::Result;
use auth::cli::query::{run_auth_query_command, QueryCli as AuthQueryCli};
use bank::cli::{
    query::{run_bank_query_command, QueryCli as BankQueryCli},
    tx::{run_bank_tx_command, Cli},
};
use clap::Subcommand;
use ibc::cli::client::tx::{run_ibc_tx_command, IbcCli};
use proto_types::AccAddress;
use tendermint::informal::block::Height;

use crate::message::Message;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Bank transaction subcommands
    Bank(Cli),
    /// IBC transaction subcommands
    IBC(IbcCli),
}

pub fn tx_command_handler(command: Commands, from_address: AccAddress) -> Result<Message> {
    match command {
        Commands::Bank(args) => run_bank_tx_command(args, from_address).map(Message::Bank),
        Commands::IBC(args) => run_ibc_tx_command(args, from_address).map(Message::Ibc),
    }
}

#[derive(Subcommand, Debug)]
pub enum QueryCommands {
    /// Querying commands for the bank module
    Bank(BankQueryCli),
    /// Querying commands for the auth module
    Auth(AuthQueryCli),
}

pub fn query_command_handler(
    command: QueryCommands,
    node: &str,
    height: Option<Height>,
) -> Result<()> {
    let res = match command {
        QueryCommands::Bank(args) => run_bank_query_command(args, node, height),
        QueryCommands::Auth(args) => run_auth_query_command(args, node, height),
    }?;

    println!("{}", res);
    Ok(())
}
