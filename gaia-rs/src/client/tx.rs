use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bank::cli::tx::{get_bank_tx_command, run_bank_tx_command};
use clap::{arg, value_parser, ArgAction, ArgMatches, Command};
use gears::utils::get_default_home_dir;

pub fn run_tx_command(matches: &ArgMatches, app_name: &str) -> Result<()> {
    let node = matches
        .get_one::<String>("node")
        .expect("Node arg has a default value so this cannot be `None`.");

    let default_home_directory = get_default_home_dir(app_name);
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

pub fn get_tx_command(app_name: &str) -> Command {
    Command::new("tx")
        .about("Transaction subcommands")
        .subcommand(get_bank_tx_command())
        .subcommand_required(true)
        .arg(
            arg!(--node)
                .help("<host>:<port> to Tendermint RPC interface for this chain")
                .default_value("http://localhost:26657")
                .action(ArgAction::Set)
                .global(true),
        )
        .arg(
            arg!(--home)
                .help(format!(
                    "Directory for config and data [default: {}]",
                    get_default_home_dir(app_name)
                        .unwrap_or_default()
                        .display()
                        .to_string()
                ))
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf)),
        )
}
