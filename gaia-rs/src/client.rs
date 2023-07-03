use std::path::PathBuf;

use anyhow::{anyhow, Result};
use auth::cli::query::run_auth_query_command;
use bank::cli::{query::run_bank_query_command, tx::run_bank_tx_command};
use clap::ArgMatches;
use gears::utils::get_default_home_dir;

use crate::APP_NAME;

pub fn tx_command_handler(matches: &ArgMatches) -> Result<()> {
    let node = matches
        .get_one::<String>("node")
        .expect("Node arg has a default value so this cannot be `None`.");

    let default_home_directory = get_default_home_dir(APP_NAME);
    let home = matches
        .get_one::<PathBuf>("home")
        .or(default_home_directory.as_ref())
        .ok_or(anyhow!(
            "Home argument not provided and OS does not provide a default home directory"
        ))?
        .to_owned();

    match matches.subcommand() {
        Some(("bank", sub_matches)) => run_bank_tx_command(sub_matches, node, home),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}

pub fn query_command_handler(matches: &ArgMatches) -> Result<()> {
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
