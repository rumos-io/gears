use crate::app_v2::handlers::{AuxHandler, QueryHandler, TxHandler};
use crate::app_v2::ApplicationInfo;
use crate::baseapp::run;
use crate::baseapp::{ABCIHandler, Genesis};
use crate::client::query::{run_query_command, QueryCommand};
use crate::client::rest::RestState;
use crate::client::tx::{run_tx_command, TxCommand};
use crate::client::{genesis_account, init, keys};
use crate::config::{ApplicationConfig, Config};
use crate::x::params::{Keeper as ParamsKeeper, ParamsSubspaceKey};
use anyhow::Result;
use axum::body::Body;
use axum::Router;
use human_panic::setup_panic;
use std::env;
use store_crate::StoreKey;

/// An empty AUX command if the user does not want to add auxillary commands.
#[derive(Debug, Clone)]
pub struct NilAuxCommand;

/// A Gears application.
pub trait ApplicationCore: TxHandler + QueryHandler + AuxHandler {
    type Genesis: Genesis;
    type StoreKey: StoreKey;
    type ParamsSubspaceKey: ParamsSubspaceKey;
    type ABCIHandler: ABCIHandler<Self::Message, Self::StoreKey, Self::Genesis>;
    type ApplicationConfig: ApplicationConfig;
}

pub struct ApplicationBuilder<'a, AppCore: ApplicationCore, AI: ApplicationInfo> {
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
    abci_handler_builder: &'a dyn Fn(Config<AppCore::ApplicationConfig>) -> AppCore::ABCIHandler,

    params_store_key: AppCore::StoreKey,
    params_subspace_key: AppCore::ParamsSubspaceKey,
}

#[derive(Debug, Clone)]
pub enum ApplicationCommands<AUX, TX, QUE> {
    Init(crate::client::init::InitCommand),
    Run(crate::baseapp::run::RunCommand),
    Keys(crate::client::keys::KeyCommand),
    GenesisAdd(crate::client::genesis_account::GenesisCommand),
    Aux(AUX),
    Tx(TxCommand<TX>),
    Query(QueryCommand<QUE>),
}

impl<'a, AppCore: ApplicationCore, AI: ApplicationInfo> ApplicationBuilder<'a, AppCore, AI> {
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
    pub fn execute(
        self,
        command: ApplicationCommands<
            AppCore::AuxCommands,
            AppCore::TxCommands,
            AppCore::QueryCommands,
        >,
    ) -> Result<()> {
        setup_panic!();

        match command {
            ApplicationCommands::Init(cmd) => {
                init::init::<_, AppCore::ApplicationConfig>(cmd, &AppCore::Genesis::default())?
            }
            ApplicationCommands::Run(cmd) => run::run(
                cmd,
                ParamsKeeper::new(self.params_store_key),
                self.params_subspace_key,
                self.abci_handler_builder,
                self.router,
            )?,
            ApplicationCommands::Keys(cmd) => keys::keys(cmd)?,
            ApplicationCommands::GenesisAdd(cmd) => {
                genesis_account::genesis_account_add::<AppCore::Genesis>(cmd)?
            }
            ApplicationCommands::Aux(cmd) => self.app_core.handle_aux_commands(cmd)?,
            ApplicationCommands::Tx(cmd) => {
                tokio::runtime::Runtime::new()
                    .expect("unclear why this would ever fail")
                    .block_on(run_tx_command::<AppCore::Message, _, _>(
                        cmd,
                        &self.app_core,
                    ))?;
            }
            ApplicationCommands::Query(cmd) => run_query_command(cmd, &self.app_core)?,
        };

        Ok(())
    }
}
