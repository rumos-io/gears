use super::{
    handlers::{ABCIHandler, AuxHandler},
    ApplicationInfo,
};
use crate::{
    baseapp::genesis::Genesis,
    commands::node::{genesis::genesis_account_add, init::init, run::run, AppCommands},
    x::params::Keeper as ParamsKeeper,
};
use crate::{
    config::{ApplicationConfig, Config},
    rest::RestState,
    types::tx::TxMessage,
    x::params::ParamsSubspaceKey,
};
use axum::Router;
use store_crate::StoreKey;

/// A Gears application.
pub trait Node: AuxHandler {
    type Message: TxMessage;
    type Genesis: Genesis;
    type StoreKey: StoreKey;
    type ParamsSubspaceKey: ParamsSubspaceKey;
    type ABCIHandler: ABCIHandler<Self::Message, Self::StoreKey, Self::Genesis>;
    type ApplicationConfig: ApplicationConfig;

    /// Builder method for defining routes of rest server
    fn router<AI: ApplicationInfo>() -> Router<
        RestState<
            Self::StoreKey,
            Self::ParamsSubspaceKey,
            Self::Message,
            Self::ABCIHandler,
            Self::Genesis,
            AI,
        >,
    >;
}

pub struct NodeApplication<'a, Core: Node, AI: ApplicationInfo> {
    core: Core,
    router: Router<
        RestState<
            Core::StoreKey,
            Core::ParamsSubspaceKey,
            Core::Message,
            Core::ABCIHandler,
            Core::Genesis,
            AI,
        >,
    >,
    abci_handler_builder: &'a dyn Fn(Config<Core::ApplicationConfig>) -> Core::ABCIHandler,

    params_store_key: Core::StoreKey,
    params_subspace_key: Core::ParamsSubspaceKey,
}

impl<'a, Core: Node, AI: ApplicationInfo> NodeApplication<'a, Core, AI> {
    pub fn new(
        core: Core,
        abci_handler_builder: &'a dyn Fn(Config<Core::ApplicationConfig>) -> Core::ABCIHandler,
        params_store_key: Core::StoreKey,
        params_subspace_key: Core::ParamsSubspaceKey,
    ) -> Self {
        Self {
            core,
            router: Core::router(),
            abci_handler_builder,
            params_store_key,
            params_subspace_key,
        }
    }

    /// Runs the command passed on the command line.
    pub fn execute(self, command: AppCommands<Core::AuxCommands>) -> anyhow::Result<()> {
        match command {
            AppCommands::Init(cmd) => {
                init::<_, Core::ApplicationConfig>(cmd, &Core::Genesis::default())?
            }
            AppCommands::Run(cmd) => run(
                cmd,
                ParamsKeeper::new(self.params_store_key),
                self.params_subspace_key,
                self.abci_handler_builder,
                self.router,
            )?,
            AppCommands::GenesisAdd(cmd) => genesis_account_add::<Core::Genesis>(cmd)?,
            AppCommands::Aux(cmd) => {
                let cmd = self.core.prepare_aux(cmd)?;
                self.core.handle_aux(cmd)?;
            }
        };

        Ok(())
    }
}
