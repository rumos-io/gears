use crate::baseapp::cli::get_run_command;
use crate::baseapp::{ABCIHandler, Genesis};
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
use proto_messages::cosmos::tx::v1beta1::message::Message;
use proto_types::AccAddress;
use std::env;
use store_crate::StoreKey;
use tendermint::informal::block::Height;

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
pub trait ApplicationCore {
    const APP_NAME: &'static str;
    const APP_VERSION: &'static str;

    type Genesis: Genesis;
    type StoreKey: StoreKey;
    type ParamsSubspaceKey: ParamsSubspaceKey;
    type Message: Message;
    type ABCIHandler: ABCIHandler<Self::Message, Self::StoreKey, Self::Genesis>;
    type QuerySubcommand: Subcommand;
    type TxSubcommand: Subcommand;
    type ApplicationConfig: ApplicationConfig;
    type AuxCommands: Subcommand; // TODO: use NilAuxCommand as default if/when associated type defaults land https://github.com/rust-lang/rust/issues/29661

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

pub struct ApplicationBuilder<'a, AppCore: ApplicationCore> {
    app_core: AppCore,
    router: Router<
        RestState<
            AppCore::StoreKey,
            AppCore::ParamsSubspaceKey,
            AppCore::Message,
            AppCore::ABCIHandler,
            AppCore::Genesis,
        >,
        Body,
    >,
    abci_handler_builder: &'a dyn Fn(Config<AppCore::ApplicationConfig>) -> AppCore::ABCIHandler,

    params_store_key: AppCore::StoreKey,
    params_subspace_key: AppCore::ParamsSubspaceKey,
}

impl<'a, AppCore: ApplicationCore> ApplicationBuilder<'a, AppCore> {
    pub fn new(
        app_core: AppCore,
        router: Router<
            RestState<
                AppCore::StoreKey,
                AppCore::ParamsSubspaceKey,
                AppCore::Message,
                AppCore::ABCIHandler,
                AppCore::Genesis,
            >,
            Body,
        >,
        abci_handler_builder: &'a dyn Fn(
            Config<AppCore::ApplicationConfig>,
        ) -> AppCore::ABCIHandler,

        params_store_key: AppCore::StoreKey,
        params_subspace_key: AppCore::ParamsSubspaceKey,
    ) -> Self {
        Self {
            app_core,
            router,
            abci_handler_builder,

            params_store_key,
            params_subspace_key,
        }
    }

    /// Runs the command passed on the command line.
    pub fn execute(self) -> Result<()> {
        setup_panic!();

        let cli = build_cli::<AppCore::TxSubcommand, AppCore::QuerySubcommand, AppCore::AuxCommands>(
            AppCore::APP_NAME,
            AppCore::APP_VERSION,
        );

        let matches = cli.get_matches();

        match matches.subcommand() {
            Some(("init", sub_matches)) => run_init_command::<_, AppCore::ApplicationConfig>(
                sub_matches,
                AppCore::APP_NAME,
                &AppCore::Genesis::default(),
            ),
            Some(("run", sub_matches)) => {
                run_run_command::<_, _, _, _, _, AppCore::ApplicationConfig>(
                    sub_matches,
                    AppCore::APP_NAME,
                    AppCore::APP_VERSION,
                    ParamsKeeper::new(self.params_store_key),
                    self.params_subspace_key,
                    self.abci_handler_builder,
                    self.router,
                )
            }
            Some(("query", sub_matches)) => {
                run_query_command(sub_matches, |command, node, height| {
                    self.app_core.handle_query_command(command, node, height)
                })?
            }
            Some(("keys", sub_matches)) => run_keys_command(sub_matches, AppCore::APP_NAME)?,
            Some(("tx", sub_matches)) => {
                run_tx_command(sub_matches, AppCore::APP_NAME, |command, from_address| {
                    self.app_core.handle_tx_command(command, from_address)
                })?
            }
            Some(("completions", sub_matches)) => {
                run_completions_command::<
                    AppCore::TxSubcommand,
                    AppCore::QuerySubcommand,
                    AppCore::AuxCommands,
                >(sub_matches, AppCore::APP_NAME, AppCore::APP_VERSION)
            }
            Some(("add-genesis-account", sub_matches)) => {
                run_add_genesis_account_command::<AppCore::Genesis>(sub_matches, AppCore::APP_NAME)?
            }
            _ => {
                self.app_core.handle_aux_commands(
                    AppCore::AuxCommands::from_arg_matches(&matches).expect(
                        "exhausted list of subcommands and subcommand_required prevents `None`",
                    ),
                )?;
            }
        };

        Ok(())
    }
}
