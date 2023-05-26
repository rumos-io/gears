use anyhow::Result;
use app::{cli::get_run_command, APP_NAME};
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use clap_complete::{generate, Generator, Shell};
use client::{init::get_init_command, query::get_query_command, tx::get_tx_command};
use human_panic::setup_panic;

use crate::{
    app::cli::run_run_command,
    client::{
        init::run_init_command,
        keys::{get_keys_command, run_keys_command},
        query::run_query_command,
        tx::run_tx_command,
    },
};

mod app;
mod client;
mod crypto;
mod error;
mod store;
mod types;
mod utils;
mod x;

const TM_ADDRESS: &str = "http://localhost:26657"; // used by rest service when proxying requests to tendermint // TODO: this needs to be configurable

fn get_completions_command() -> Command {
    Command::new("completions")
        .about("Output shell completions")
        .arg(
            Arg::new("shell")
                .required(true)
                .action(ArgAction::Set)
                .value_parser(value_parser!(Shell)),
        )
}

fn run_completions_command(matches: &ArgMatches) {
    if let Some(generator) = matches.get_one::<Shell>("shell").copied() {
        let mut cmd = build_cli();
        print_completions(generator, &mut cmd);
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

fn build_cli() -> Command {
    Command::new(APP_NAME)
        .version(env!("GIT_HASH"))
        .subcommand_required(true)
        .subcommand(get_init_command())
        .subcommand(get_run_command())
        .subcommand(get_query_command())
        .subcommand(get_keys_command())
        .subcommand(get_tx_command())
        .subcommand(get_completions_command())
}

fn main() -> Result<()> {
    setup_panic!();

    let cli = build_cli();
    let matches = cli.get_matches();

    match matches.subcommand() {
        Some(("init", sub_matches)) => run_init_command(sub_matches),
        Some(("run", sub_matches)) => run_run_command(sub_matches),
        Some(("query", sub_matches)) => run_query_command(sub_matches)?,
        Some(("keys", sub_matches)) => run_keys_command(sub_matches)?,
        Some(("tx", sub_matches)) => run_tx_command(sub_matches)?,
        Some(("completions", sub_matches)) => run_completions_command(sub_matches),
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    Ok(())
}
