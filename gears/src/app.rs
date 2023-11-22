use crate::baseapp::ante::AnteHandler;
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
use clap::FromArgMatches;
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

/// An empty AUX command if the user does not want to add auxillary commands.
#[derive(Subcommand, Debug)]
pub enum NilAuxCommand {}

/// A Gears application.
pub trait Application {
    const APP_NAME: &'static str;
    const APP_VERSION: &'static str;

    type Genesis: Genesis;
    type StoreKey: StoreKey;
    type ParamsSubspaceKey: ParamsSubspaceKey;
    type Message: Message;
    type Handler: Handler<Self::Message, Self::StoreKey, Self::Genesis>;
    type QuerySubcommand: Subcommand;
    type TxSubcommand: Subcommand;
    type ApplicationConfig: ApplicationConfig;
    type AuxCommands: Subcommand; // TODO: use NilAuxCommand as default if/when associated type defaults land https://github.com/rust-lang/rust/issues/29661
    type AnteHandler: AnteHandler<Self::StoreKey>;

    fn get_params_store_key(&self) -> Self::StoreKey;

    fn get_params_subspace_key(&self) -> Self::ParamsSubspaceKey;

    fn handle_tx_command(
        &self,
        command: Self::TxSubcommand,
        from_address: AccAddress,
    ) -> Result<Self::Message>;

    fn handle_query_command(
        &self,
        command: Self::QuerySubcommand,
        node: &str,
        height: Option<Height>,
    ) -> Result<()>;

    #[allow(unused_variables)]
    fn handle_aux_commands(&self, command: Self::AuxCommands) -> Result<()> {
        Ok(())
    }
}

pub struct Node<'a, App: Application> {
    app: App,
    router: Router<
        RestState<
            App::StoreKey,
            App::ParamsSubspaceKey,
            App::Message,
            App::Handler,
            App::Genesis,
            App::AnteHandler,
        >,
        Body,
    >,
    handler_builder: &'a dyn Fn(Config<App::ApplicationConfig>) -> App::Handler,
    ante_handler: App::AnteHandler,
}

impl<'a, App: Application> Node<'a, App> {
    pub fn new(
        app: App,
        router: Router<
            RestState<
                App::StoreKey,
                App::ParamsSubspaceKey,
                App::Message,
                App::Handler,
                App::Genesis,
                App::AnteHandler,
            >,
            Body,
        >,
        handler_builder: &'a dyn Fn(Config<App::ApplicationConfig>) -> App::Handler,
        ante_handler: App::AnteHandler,
    ) -> Self {
        Self {
            app,
            router,
            handler_builder,
            ante_handler,
        }
    }

    /// Runs the command passed on the command line.
    pub fn run_command(self) -> Result<()> {
        setup_panic!();

        let cli = build_cli::<App::TxSubcommand, App::QuerySubcommand, App::AuxCommands>(
            App::APP_NAME,
            App::APP_VERSION,
        );

        let matches = cli.get_matches();

        match matches.subcommand() {
            Some(("init", sub_matches)) => run_init_command::<_, App::ApplicationConfig>(
                sub_matches,
                App::APP_NAME,
                &App::Genesis::default(),
            ),
            Some(("run", sub_matches)) => {
                run_run_command::<_, _, _, _, _, App::ApplicationConfig, _>(
                    sub_matches,
                    App::APP_NAME,
                    App::APP_VERSION,
                    ParamsKeeper::new(self.app.get_params_store_key()),
                    self.app.get_params_subspace_key(),
                    self.handler_builder,
                    self.router,
                    self.ante_handler,
                )
            }
            Some(("query", sub_matches)) => {
                run_query_command(sub_matches, |command, node, height| {
                    self.app.handle_query_command(command, node, height)
                })?
            }
            Some(("keys", sub_matches)) => run_keys_command(sub_matches, App::APP_NAME)?,
            Some(("tx", sub_matches)) => {
                run_tx_command(sub_matches, App::APP_NAME, |command, from_address| {
                    self.app.handle_tx_command(command, from_address)
                })?
            }
            Some(("completions", sub_matches)) => {
                run_completions_command::<App::TxSubcommand, App::QuerySubcommand, App::AuxCommands>(
                    sub_matches,
                    App::APP_NAME,
                    App::APP_VERSION,
                )
            }
            Some(("add-genesis-account", sub_matches)) => {
                run_add_genesis_account_command::<App::Genesis>(sub_matches, App::APP_NAME)?
            }
            _ => {
                self.app.handle_aux_commands(
                    App::AuxCommands::from_arg_matches(&matches).expect(
                        "exhausted list of subcommands and subcommand_required prevents `None`",
                    ),
                )?;
            }
        };

        Ok(())
    }
}
