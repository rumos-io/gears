use crate::baseapp::run::{self, run};
use crate::baseapp::{ABCIHandler, Genesis};
use crate::client::{genesis_account, init, keys};
use crate::client::keys::keys;
use crate::client::query::{get_query_command, run_query_command};
use crate::client::rest::RestState;
use crate::client::tx::{get_tx_command, run_tx_command};
use crate::config::{ApplicationConfig, Config};
use crate::x::params::{Keeper as ParamsKeeper, ParamsSubspaceKey};
use anyhow::Result;
use axum::body::Body;
use axum::Router;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command, Subcommand};
use human_panic::setup_panic;
use proto_messages::cosmos::tx::v1beta1::message::Message;
use proto_types::AccAddress;
use std::env;
use store_crate::StoreKey;
use tendermint::informal::block::Height;

pub trait ApplicationInfo : Clone + Sync + Send + 'static {
    const APP_NAME: &'static str;
    const APP_VERSION: &'static str;
}

#[derive(Debug, Clone)]
pub struct DefaultApplication;

impl ApplicationInfo for DefaultApplication
{
    const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
    const APP_VERSION: &'static str = "1"; // TODO: GIT_HASH
}

/// An empty AUX command if the user does not want to add auxillary commands.
#[derive(Subcommand, Debug)]
pub enum NilAuxCommand {}

/// A Gears application.
pub trait ApplicationCore {
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

pub struct ApplicationBuilder<'a, AppCore: ApplicationCore, AI : ApplicationInfo> {
    app_core: AppCore,
    router: Router<
        RestState<
            AppCore::StoreKey,
            AppCore::ParamsSubspaceKey,
            AppCore::Message,
            AppCore::ABCIHandler,
            AppCore::Genesis,
            AI
        >,
        Body,
    >,
    abci_handler_builder: &'a dyn Fn(Config<AppCore::ApplicationConfig>) -> AppCore::ABCIHandler,

    params_store_key: AppCore::StoreKey,
    params_subspace_key: AppCore::ParamsSubspaceKey,
}

#[derive(Debug, Clone)]
pub enum ApplicationCommands
{
    Init( crate::client::init::InitCommand),
    Run( crate::baseapp::run::RunCommand ),
    Keys( crate::client::keys::KeyCommand),
    GenesisAdd( crate::client::genesis_account::GenesisCommand ),
}

impl<'a, AppCore: ApplicationCore, AI : ApplicationInfo> ApplicationBuilder<'a, AppCore, AI> {
    pub fn new(
        app_core: AppCore,
        router: Router<
            RestState<
                AppCore::StoreKey,
                AppCore::ParamsSubspaceKey,
                AppCore::Message,
                AppCore::ABCIHandler,
                AppCore::Genesis,
                AI,
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
    pub fn execute(self, command : ApplicationCommands) -> Result<()> {
        setup_panic!();

        match command 
        {
            ApplicationCommands::Init( cmd ) => init::init::<_, AppCore::ApplicationConfig>( cmd, &AppCore::Genesis::default())?,
            ApplicationCommands::Run( cmd ) => run::run(cmd, ParamsKeeper::new(self.params_store_key), self.params_subspace_key, self.abci_handler_builder, self.router)?,
            ApplicationCommands::Keys( cmd ) => keys::keys( cmd)?,
            ApplicationCommands::GenesisAdd( cmd ) => genesis_account::genesis_account_add::<AppCore::Genesis>(cmd)?,
        };

        // match matches.subcommand() {
        //     Some(("query", sub_matches)) => {
        //         // TODO: refactor this for new approach
        //         run_query_command(sub_matches, |command, node, height| {
        //             self.app_core.handle_query_command(command, node, height)
        //         })?
        //     }
        //     Some(("tx", sub_matches)) => {
        //         // TODO: refactor this for new approach
        //         run_tx_command(sub_matches, AppCore::APP_NAME, |command, from_address| {
        //             self.app_core.handle_tx_command(command, from_address)
        //         })?
        //     }
        //     _ => {
        //         self.app_core.handle_aux_commands(
        //             AppCore::AuxCommands::from_arg_matches(&matches).expect(
        //                 "exhausted list of subcommands and subcommand_required prevents `None`",
        //             ),
        //         )?;
        //     }
        // };

        Ok(())
    }
}
