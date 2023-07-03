use crate::baseapp::ante_v2::{AuthKeeper, BankKeeper};
use crate::baseapp::Handler;
use anyhow::Result;
use proto_messages::cosmos::tx::v1beta1::tx_v2::Message;
use serde::Serialize;
use store_crate::StoreKey;
use strum::IntoEnumIterator;
// use auth::cli::query::get_auth_query_command;
// use auth::Keeper as AuthKeeper;
// use bank::cli::query::get_bank_query_command;
// use bank::Keeper as BankKeeper;
use crate::baseapp::cli::get_run_command;
use crate::client::query::get_query_command_v2;
use crate::client::{init::get_init_command, query::get_query_command, tx::get_tx_command};
use crate::x::params::{Keeper as ParamsKeeper, ParamsSubspaceKey};
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use clap_complete::{generate, Generator, Shell};
use human_panic::setup_panic;
use std::hash::Hash;

use crate::{
    baseapp::cli::run_run_command_micro,
    client::{
        init::run_init_command,
        keys::{get_keys_command, run_keys_command},
        query::run_query_command,
        tx::run_tx_command,
    },
};

// use crate::genesis::GenesisState;
// use crate::handler::Handler;
// use crate::message::Message;
// use crate::store_keys::{GaiaParamsStoreKey, GaiaStoreKey};

// mod client;
// mod genesis;
// mod handler;
// mod message;
// mod store_keys;

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

fn run_completions_command(matches: &ArgMatches, app_name: &'static str, version: &'static str) {
    if let Some(generator) = matches.get_one::<Shell>("shell").copied() {
        let mut cmd = build_cli(app_name, version);
        print_completions(generator, &mut cmd);
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

fn build_cli(app_name: &'static str, version: &'static str) -> Command {
    Command::new(app_name)
        .version(version)
        .subcommand_required(true)
        .subcommand(get_init_command(app_name))
        .subcommand(get_run_command(app_name))
        // .subcommand(get_query_command_v2(vec![
        //     get_bank_query_command(),
        //     get_auth_query_command(),
        // ]))
        .subcommand(get_keys_command(app_name))
        //.subcommand(get_tx_command(APP_NAME))
        .subcommand(get_completions_command())
}

pub fn run<G, SK, PSK, M, BK, AK, H>(
    version: &'static str,
    genesis_state: G,
    app_name: &'static str,
    bank_keeper: BK,
    auth_keeper: AK,
    params_keeper: ParamsKeeper<SK, PSK>,
    params_subspace_key: PSK,
    handler: H,
    //query_commands: Vec<Command>,
    //tx_commands: Vec<Command>,
) -> Result<()>
where
    G: Serialize,
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK> + 'static,
{
    setup_panic!();

    let cli = build_cli(app_name, version);
    let matches = cli.get_matches();

    match matches.subcommand() {
        Some(("init", sub_matches)) => run_init_command(sub_matches, app_name, genesis_state),
        Some(("run", sub_matches)) => run_run_command_micro(
            sub_matches,
            app_name,
            bank_keeper,
            auth_keeper,
            params_keeper,
            params_subspace_key,
            handler,
        ),
        // Some(("query", sub_matches)) => run_query_command(sub_matches)?,
        Some(("keys", sub_matches)) => run_keys_command(sub_matches, app_name)?,
        //Some(("tx", sub_matches)) => run_tx_command(sub_matches, APP_NAME)?,
        Some(("completions", sub_matches)) => {
            run_completions_command(sub_matches, app_name, version)
        }
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    Ok(())
}
