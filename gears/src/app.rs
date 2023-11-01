use crate::baseapp::ante::{AuthKeeper, BankKeeper};
use crate::baseapp::cli::get_run_command;
use crate::baseapp::{Genesis, Handler};
use crate::client::genesis_account::{
    get_add_genesis_account_command, run_add_genesis_account_command,
};
use crate::client::init::get_init_command;
use crate::client::query::{get_query_command, run_query_command};
use crate::client::rest::RestState;
use crate::client::tx::{get_tx_command, run_tx_command};
use crate::config::{ApplicationConfig, Config};
use crate::x::params::{Keeper as ParamsKeeper, ParamsSubspaceKey};
use anyhow::Result;
use axum::body::Body;
use axum::Router;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command, Subcommand};
use clap_complete::{generate, Generator, Shell};
use human_panic::setup_panic;
use proto_messages::cosmos::tx::v1beta1::message::Message;
use proto_types::AccAddress;
use std::env;
use store_crate::StoreKey;
use tendermint_informal::block::Height;

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

fn run_completions_command<TxSubcommand: Subcommand, QuerySubcommand: Subcommand>(
    matches: &ArgMatches,
    app_name: &'static str,
    version: &'static str,
) {
    if let Some(generator) = matches.get_one::<Shell>("shell").copied() {
        let mut cmd = build_cli::<TxSubcommand, QuerySubcommand>(app_name, version);
        print_completions(generator, &mut cmd);
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

fn build_cli<TxSubcommand: Subcommand, QuerySubcommand: Subcommand>(
    app_name: &'static str,
    version: &'static str,
) -> Command {
    Command::new(app_name)
        .version(version)
        .subcommand_required(true)
        .subcommand(get_init_command(app_name))
        .subcommand(get_run_command(app_name))
        .subcommand(get_query_command::<QuerySubcommand>())
        .subcommand(get_keys_command(app_name))
        .subcommand(get_tx_command::<TxSubcommand>(app_name))
        .subcommand(get_completions_command())
        .subcommand(get_add_genesis_account_command(app_name))
}

pub fn run<
    G,
    SK,
    PSK,
    M,
    BK,
    AK,
    H,
    HandlerBuilder,
    QuerySubcommand,
    QueryCmdHandler,
    TxSubcommand,
    TxCmdHandler,
    AC: ApplicationConfig,
>(
    app_name: &'static str,
    app_version: &'static str,
    genesis_state: G,
    bank_keeper: BK,
    auth_keeper: AK,
    params_keeper: ParamsKeeper<SK, PSK>,
    params_subspace_key: PSK,
    handler_builder: HandlerBuilder,
    query_command_handler: QueryCmdHandler,
    tx_command_handler: TxCmdHandler,
    router: Router<RestState<SK, PSK, M, BK, AK, H, G>, Body>,
) -> Result<()>
where
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
    QuerySubcommand: Subcommand,
    QueryCmdHandler: FnOnce(QuerySubcommand, &str, Option<Height>) -> Result<()>,
    TxSubcommand: Subcommand,
    TxCmdHandler: FnOnce(TxSubcommand, AccAddress) -> Result<M>,
    HandlerBuilder: FnOnce(Config<AC>) -> H,
{
    setup_panic!();

    let cli = build_cli::<TxSubcommand, QuerySubcommand>(app_name, app_version);

    let matches = cli.get_matches();

    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            run_init_command::<_, AC>(sub_matches, app_name, genesis_state)
        }
        Some(("run", sub_matches)) => run_run_command::<_, _, _, _, _, _, _, _, AC>(
            sub_matches,
            app_name,
            app_version,
            bank_keeper,
            auth_keeper,
            params_keeper,
            params_subspace_key,
            handler_builder,
            router,
        ),
        Some(("query", sub_matches)) => run_query_command(sub_matches, query_command_handler)?,
        Some(("keys", sub_matches)) => run_keys_command(sub_matches, app_name)?,
        Some(("tx", sub_matches)) => run_tx_command(sub_matches, app_name, tx_command_handler)?,
        Some(("completions", sub_matches)) => run_completions_command::<
            TxSubcommand,
            QuerySubcommand,
        >(sub_matches, app_name, app_version),
        Some(("add-genesis-account", sub_matches)) => {
            run_add_genesis_account_command(sub_matches, app_name, handler_builder)?
        }
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    Ok(())
}
