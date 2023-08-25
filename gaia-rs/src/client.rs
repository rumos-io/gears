use std::path::PathBuf;

use anyhow::Result;
use auth::cli::query::run_auth_query_command;
use bank::cli::{
    query::run_bank_query_command,
    tx::{run_bank_tx_command, Cli},
};
use clap::{ArgMatches, Subcommand};
use tendermint_informal::block::Height;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Bank transaction subcommands
    Bank(Cli),
}

pub fn tx_command_handler(command: Commands, node: &str, home: PathBuf) -> Result<()> {
    match command {
        Commands::Bank(args) => run_bank_tx_command(args, node, home),
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
