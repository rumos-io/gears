use crate::baseapp::ante::{AuthKeeper, BankKeeper};
use crate::baseapp::cli::get_run_command;
use crate::baseapp::Handler;
use crate::client::init::get_init_command;
use crate::client::query::get_query_command;
use crate::client::tx::get_tx_command;
use crate::x::params::{Keeper as ParamsKeeper, ParamsSubspaceKey};
use anyhow::Result;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use clap_complete::{generate, Generator, Shell};
use human_panic::setup_panic;
use proto_messages::cosmos::tx::v1beta1::Message;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use store_crate::StoreKey;
use strum::IntoEnumIterator;

use crate::{
    baseapp::cli::run_run_command_micro,
    client::{
        init::run_init_command,
        keys::{get_keys_command, run_keys_command},
    },
};

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

fn run_completions_command(
    matches: &ArgMatches,
    app_name: &'static str,
    version: &'static str,
    query_commands: Vec<Command>,
    tx_commands: Vec<Command>,
) {
    if let Some(generator) = matches.get_one::<Shell>("shell").copied() {
        let mut cmd = build_cli(app_name, version, query_commands, tx_commands);
        print_completions(generator, &mut cmd);
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

fn build_cli(
    app_name: &'static str,
    version: &'static str,
    query_commands: Vec<Command>,
    tx_commands: Vec<Command>,
) -> Command {
    Command::new(app_name)
        .version(version)
        .subcommand_required(true)
        .subcommand(get_init_command(app_name))
        .subcommand(get_run_command(app_name))
        .subcommand(get_query_command(query_commands))
        .subcommand(get_keys_command(app_name))
        .subcommand(get_tx_command(app_name, tx_commands))
        .subcommand(get_completions_command())
}

pub fn run<G, SK, PSK, M, BK, AK, H, FQ, FT>(
    app_name: &'static str,
    app_version: &'static str,
    genesis_state: G,
    bank_keeper: BK,
    auth_keeper: AK,
    params_keeper: ParamsKeeper<SK, PSK>,
    params_subspace_key: PSK,
    handler: H,
    query_commands: Vec<Command>,
    query_command_handler: FQ,
    tx_commands: Vec<Command>,
    tx_command_handler: FT,
) -> Result<()>
where
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    FQ: FnOnce(&ArgMatches) -> Result<()>,
    FT: FnOnce(&ArgMatches) -> Result<()>,
    G: DeserializeOwned + Clone + Send + Sync + 'static + Serialize,
{
    setup_panic!();

    let cli = build_cli(
        app_name,
        app_version,
        query_commands.clone(),
        tx_commands.clone(),
    );
    let matches = cli.get_matches();

    match matches.subcommand() {
        Some(("init", sub_matches)) => run_init_command(sub_matches, app_name, genesis_state),
        Some(("run", sub_matches)) => run_run_command_micro(
            sub_matches,
            app_name,
            app_version,
            bank_keeper,
            auth_keeper,
            params_keeper,
            params_subspace_key,
            handler,
        ),
        Some(("query", sub_matches)) => query_command_handler(sub_matches)?,
        Some(("keys", sub_matches)) => run_keys_command(sub_matches, app_name)?,
        Some(("tx", sub_matches)) => tx_command_handler(sub_matches)?,
        Some(("completions", sub_matches)) => run_completions_command(
            sub_matches,
            app_name,
            app_version,
            query_commands,
            tx_commands,
        ),
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    Ok(())
}
