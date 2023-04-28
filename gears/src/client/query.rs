use anyhow::Result;
use clap::{arg, ArgAction, ArgMatches, Command};

use crate::x::{
    auth::client::cli::query::{get_auth_query_command, run_auth_query_command},
    bank::client::cli::query::{get_bank_query_command, run_bank_query_command},
};

pub fn get_query_command() -> Command {
    Command::new("query")
        .about("Querying subcommands")
        .subcommand(get_bank_query_command())
        .subcommand(get_auth_query_command())
        .subcommand_required(true)
        .arg(
            arg!(--node)
                .help("<host>:<port> to Tendermint RPC interface for this chain")
                .default_value("http://localhost:26657")
                .action(ArgAction::Set)
                .global(true),
        )
}

pub fn run_query_command(matches: &ArgMatches) -> Result<()> {
    let node = matches
        .get_one::<String>("node")
        .expect("Node arg has a default value so this cannot be `None`.");

    let res = match matches.subcommand() {
        Some(("bank", sub_matches)) => run_bank_query_command(sub_matches, node),
        Some(("auth", sub_matches)) => run_auth_query_command(sub_matches, node),

        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }?;

    println!("{}", res);
    Ok(())
}
