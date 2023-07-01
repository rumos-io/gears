use anyhow::Result;
use auth::Keeper as AuthKeeper;
use bank::Keeper as BankKeeper;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use clap_complete::{generate, Generator, Shell};
use gears::baseapp::cli::get_run_command;
use gears::client::{init::get_init_command, query::get_query_command, tx::get_tx_command};
use gears::x::params::Keeper as ParamsKeeper;
use human_panic::setup_panic;

use gears::{
    baseapp::cli::run_run_command_micro,
    client::{
        init::run_init_command,
        keys::{get_keys_command, run_keys_command},
        query::run_query_command,
        tx::run_tx_command,
    },
};

use crate::genesis::GenesisState;
use crate::handler::Handler;
use crate::message::Message;
use crate::store_keys::{GaiaParamsStoreKey, GaiaStoreKey};

mod client;
mod genesis;
mod handler;
mod message;
mod store_keys;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

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
        .subcommand(get_init_command(APP_NAME))
        .subcommand(get_run_command(APP_NAME))
        .subcommand(get_query_command())
        .subcommand(get_keys_command(APP_NAME))
        .subcommand(get_tx_command(APP_NAME))
        .subcommand(get_completions_command())
}

fn main() -> Result<()> {
    setup_panic!();

    let cli = build_cli();
    let matches = cli.get_matches();

    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            run_init_command(sub_matches, APP_NAME, GenesisState::default())
        }
        Some(("run", sub_matches)) => {
            let params_keeper = ParamsKeeper::new(GaiaStoreKey::Params);

            let auth_keeper = AuthKeeper::new(
                GaiaStoreKey::Auth,
                params_keeper.clone(),
                GaiaParamsStoreKey::Auth,
            );

            let bank_keeper = BankKeeper::new(
                GaiaStoreKey::Bank,
                params_keeper.clone(),
                GaiaParamsStoreKey::Bank,
                auth_keeper.clone(),
            );

            run_run_command_micro::<
                GaiaStoreKey,
                GaiaParamsStoreKey,
                Message,
                BankKeeper<GaiaStoreKey, GaiaParamsStoreKey>,
                AuthKeeper<GaiaStoreKey, GaiaParamsStoreKey>,
                Handler,
            >(
                sub_matches,
                APP_NAME,
                bank_keeper,
                auth_keeper,
                params_keeper,
                GaiaParamsStoreKey::BaseApp,
                Handler::new(),
            )
        }
        Some(("query", sub_matches)) => run_query_command(sub_matches)?,
        Some(("keys", sub_matches)) => run_keys_command(sub_matches, APP_NAME)?,
        Some(("tx", sub_matches)) => run_tx_command(sub_matches, APP_NAME)?,
        Some(("completions", sub_matches)) => run_completions_command(sub_matches),
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    Ok(())
}
