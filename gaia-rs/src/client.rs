use anyhow::Result;
use auth::cli::query::run_auth_query_command;
use bank::cli::{
    query::run_bank_query_command,
    tx::{run_bank_tx_command, Cli},
};
use clap::{ArgMatches, Subcommand};
use proto_types::AccAddress;
use tendermint_informal::block::Height;

use crate::message::Message;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Bank transaction subcommands
    Bank(Cli),
}

pub fn tx_command_handler(command: Commands, from_address: AccAddress) -> Result<Message> {
    match command {
        Commands::Bank(args) => {
            run_bank_tx_command(args, from_address).map(|msg| Message::Bank(msg))
        }
    }
}

pub fn query_command_handler(matches: &ArgMatches) -> Result<()> {
    let node = matches
        .get_one::<String>("node")
        .expect("Node arg has a default value so this cannot be `None`.");

    let height = *matches
        .get_one::<Height>("height")
        .expect("Height arg has a default value so this cannot be `None`.");

    let res = match matches.subcommand() {
        Some(("bank", sub_matches)) => run_bank_query_command(sub_matches, node, Some(height)),
        Some(("auth", sub_matches)) => run_auth_query_command(sub_matches, node),

        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }?;

    println!("{}", res);
    Ok(())
}
