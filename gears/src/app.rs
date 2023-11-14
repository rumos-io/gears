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
use proto_messages::cosmos::tx::v1beta1::Message;
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

fn run_completions_command<
    TxSubcommand: Subcommand,
    QuerySubcommand: Subcommand,
    AuxCommands: Subcommand,
>(
    matches: &ArgMatches,
    app_name: &'static str,
    version: &'static str,
) {
    if let Some(generator) = matches.get_one::<Shell>("shell").copied() {
        let mut cmd = build_cli::<TxSubcommand, QuerySubcommand, AuxCommands>(app_name, version);
        print_completions(generator, &mut cmd);
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

fn build_cli<TxSubcommand: Subcommand, QuerySubcommand: Subcommand, AuxCommands: Subcommand>(
    app_name: &'static str,
    version: &'static str,
) -> Command {
    let cli = Command::new(app_name)
        .version(version)
        .subcommand_required(true)
        .subcommand(get_init_command(app_name))
        .subcommand(get_run_command(app_name))
        .subcommand(get_query_command::<QuerySubcommand>())
        .subcommand(get_keys_command(app_name))
        .subcommand(get_tx_command::<TxSubcommand>(app_name))
        .subcommand(get_completions_command())
        .subcommand(get_add_genesis_account_command(app_name));

    AuxCommands::augment_subcommands(cli)
}

/// A default type for the AuxCommands parameter if the user does not want to add auxillary commands.
#[derive(Subcommand, Debug)]
pub enum DefaultAuxCommandParam {}

/// The main application struct.
pub struct Application<
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
    AC,
    AuxCommands = DefaultAuxCommandParam,
    AuxCommandsHandler = fn(AuxCommands) -> Result<()>,
> where
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
    AC: ApplicationConfig,
    HandlerBuilder: FnOnce(Config<AC>) -> H,
    AuxCommands: Subcommand,
    AuxCommandsHandler: FnOnce(AuxCommands) -> Result<()>,
{
    app_name: &'static str,
    app_version: &'static str,
    bank_keeper: BK,
    auth_keeper: AK,
    params_keeper: ParamsKeeper<SK, PSK>,
    params_subspace_key: PSK,
    handler_builder: HandlerBuilder,
    query_command_handler: QueryCmdHandler,
    tx_command_handler: TxCmdHandler,
    router: Router<RestState<SK, PSK, M, BK, AK, H, G>, Body>,
    aux_commands_handler: Option<AuxCommandsHandler>,
    phantom: std::marker::PhantomData<AC>,
    phantom2: std::marker::PhantomData<TxSubcommand>,
    phantom3: std::marker::PhantomData<QuerySubcommand>,
    phantom4: std::marker::PhantomData<AuxCommands>,
}

impl<
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
        AuxCommands,
        AuxCommandsHandler,
    >
    Application<
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
        AC,
        AuxCommands,
        AuxCommandsHandler,
    >
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
    AC: ApplicationConfig,
    HandlerBuilder: FnOnce(Config<AC>) -> H,
    AuxCommands: Subcommand,
    AuxCommandsHandler: FnOnce(AuxCommands) -> Result<()>,
{
    /// Creates a new application.
    pub fn new(
        app_name: &'static str,
        app_version: &'static str,
        bank_keeper: BK,
        auth_keeper: AK,
        params_keeper: ParamsKeeper<SK, PSK>,
        params_subspace_key: PSK,
        handler_builder: HandlerBuilder,
        query_command_handler: QueryCmdHandler,
        tx_command_handler: TxCmdHandler,
        router: Router<RestState<SK, PSK, M, BK, AK, H, G>, Body>,
    ) -> Self {
        Self {
            app_name,
            app_version,
            bank_keeper,
            auth_keeper,
            params_keeper,
            params_subspace_key,
            handler_builder,
            query_command_handler,
            tx_command_handler,
            router,
            aux_commands_handler: None,
            phantom: std::marker::PhantomData,
            phantom2: std::marker::PhantomData,
            phantom3: std::marker::PhantomData,
            phantom4: std::marker::PhantomData,
        }
    }

    /// Add custom commands to the application.
    pub fn add_aux_commands(mut self, aux_commands_handler: AuxCommandsHandler) -> Self {
        self.aux_commands_handler = Some(aux_commands_handler);
        self
    }

    /// Runs the command passed on the command line.
    pub fn run_command(self) -> Result<()> {
        setup_panic!();

        let cli = build_cli::<TxSubcommand, QuerySubcommand, AuxCommands>(
            self.app_name,
            self.app_version,
        );

        let matches = cli.get_matches();

        match matches.subcommand() {
            Some(("init", sub_matches)) => {
                run_init_command::<_, AC>(sub_matches, self.app_name, &G::default())
            }
            Some(("run", sub_matches)) => run_run_command::<_, _, _, _, _, _, _, _, AC>(
                sub_matches,
                self.app_name,
                self.app_version,
                self.bank_keeper,
                self.auth_keeper,
                self.params_keeper,
                self.params_subspace_key,
                self.handler_builder,
                self.router,
            ),
            Some(("query", sub_matches)) => {
                run_query_command(sub_matches, self.query_command_handler)?
            }
            Some(("keys", sub_matches)) => run_keys_command(sub_matches, self.app_name)?,
            Some(("tx", sub_matches)) => {
                run_tx_command(sub_matches, self.app_name, self.tx_command_handler)?
            }
            Some(("completions", sub_matches)) => {
                run_completions_command::<TxSubcommand, QuerySubcommand, AuxCommands>(
                    sub_matches,
                    self.app_name,
                    self.app_version,
                )
            }
            Some(("add-genesis-account", sub_matches)) => {
                run_add_genesis_account_command::<G>(sub_matches, self.app_name)?
            }
            _ => {
                if let Some(aux_commands_handler) = self.aux_commands_handler {
                    aux_commands_handler(AuxCommands::from_arg_matches(&matches).expect(
                        "exhausted list of subcommands and subcommand_required prevents `None`",
                    ))?;
                } else {
                    unreachable!(
                        "exhausted list of subcommands and subcommand_required prevents `None`"
                    )
                }
            }
        };

        Ok(())
    }
}
