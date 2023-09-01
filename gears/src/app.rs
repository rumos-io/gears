use crate::baseapp::ante::{AuthKeeper, BankKeeper};
use crate::baseapp::cli::get_run_command;
use crate::baseapp::{BaseApp, Genesis, Handler};
use crate::client::genesis_account::{
    get_add_genesis_account_command, run_add_genesis_account_command,
};
use crate::client::init::get_init_command;
use crate::client::query::get_query_command;
use crate::client::tx::{get_tx_command, run_tx_command};
use crate::x::params::{Keeper as ParamsKeeper, ParamsSubspaceKey};
use anyhow::Result;
use axum::body::Body;
use axum::Router;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command, Subcommand};
use clap_complete::{generate, Generator, Shell};
use human_panic::setup_panic;
use proto_messages::cosmos::tx::v1beta1::Message;
use proto_types::AccAddress;
use std::env;
use store_crate::StoreKey;

use crate::{
    baseapp::cli::run_run_command,
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

fn run_completions_command<TxSubcommand: Subcommand>(
    matches: &ArgMatches,
    app_name: &'static str,
    version: &'static str,
    query_commands: Vec<Command>,
) {
    if let Some(generator) = matches.get_one::<Shell>("shell").copied() {
        let mut cmd = build_cli::<TxSubcommand>(app_name, version, query_commands);
        print_completions(generator, &mut cmd);
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

fn build_cli<TxSubcommand: Subcommand>(
    app_name: &'static str,
    version: &'static str,
    query_commands: Vec<Command>,
) -> Command {
    Command::new(app_name)
        .version(version)
        .subcommand_required(true)
        .subcommand(get_init_command(app_name))
        .subcommand(get_run_command(app_name))
        .subcommand(get_query_command(query_commands))
        .subcommand(get_keys_command(app_name))
        .subcommand(get_tx_command::<TxSubcommand>(app_name))
        .subcommand(get_completions_command())
        .subcommand(get_add_genesis_account_command(app_name))
}

pub fn run<G, SK, PSK, M, BK, AK, H, FQ, TxSubcommand, TxCmdHandler>(
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
    tx_command_handler: TxCmdHandler,
    router: Router<BaseApp<SK, PSK, M, BK, AK, H, G>, Body>,
) -> Result<()>
where
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    FQ: FnOnce(&ArgMatches) -> Result<()>,
    G: Genesis,
    TxSubcommand: Subcommand,
    TxCmdHandler: FnOnce(TxSubcommand, AccAddress) -> Result<M>,
{
    setup_panic!();

    let cli = build_cli::<TxSubcommand>(app_name, app_version, query_commands.clone());

    let matches = cli.get_matches();

    match matches.subcommand() {
        Some(("init", sub_matches)) => run_init_command(sub_matches, app_name, genesis_state),
        Some(("run", sub_matches)) => run_run_command(
            sub_matches,
            app_name,
            app_version,
            bank_keeper,
            auth_keeper,
            params_keeper,
            params_subspace_key,
            handler,
            router,
        ),
        Some(("query", sub_matches)) => query_command_handler(sub_matches)?,
        Some(("keys", sub_matches)) => run_keys_command(sub_matches, app_name)?,
        Some(("tx", sub_matches)) => run_tx_command(sub_matches, app_name, tx_command_handler)?,
        Some(("completions", sub_matches)) => run_completions_command::<TxSubcommand>(
            sub_matches,
            app_name,
            app_version,
            query_commands,
        ),
        Some(("add-genesis-account", sub_matches)) => {
            run_add_genesis_account_command::<G, H, SK, M>(sub_matches, app_name, handler)?
        }
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    Ok(())
}
