use super::{
    handlers::{node::ABCIHandler, AuxHandler},
    ApplicationInfo,
};
use crate::{
    baseapp::{genesis::Genesis, QueryRequest, QueryResponse},
    commands::node::{
        genesis::genesis_account_add,
        init::init,
        run::{run, RouterBuilder},
        AppCommands,
    },
    params::Keeper as ParamsKeeper,
};
use crate::{
    config::{ApplicationConfig, Config},
    params::ParamsSubspaceKey,
    types::tx::TxMessage,
};
use store_crate::StoreKey;

/// A Gears application.
pub trait Node: AuxHandler + RouterBuilder<Self::QReq, Self::QRes> {
    type Message: TxMessage;
    type Genesis: Genesis;
    type StoreKey: StoreKey;
    type ParamsSubspaceKey: ParamsSubspaceKey;
    type ABCIHandler: ABCIHandler<
        Self::Message,
        Self::StoreKey,
        Self::Genesis,
        Self::QReq,
        Self::QRes,
    >;
    type ApplicationConfig: ApplicationConfig;
    type QReq: QueryRequest;
    type QRes: QueryResponse;
}

pub struct NodeApplication<'a, Core: Node> {
    core: Core,
    abci_handler_builder: &'a dyn Fn(Config<Core::ApplicationConfig>) -> Core::ABCIHandler,

    params_store_key: Core::StoreKey,
    params_subspace_key: Core::ParamsSubspaceKey,
}

impl<'a, Core: Node> NodeApplication<'a, Core> {
    pub fn new(
        core: Core,
        abci_handler_builder: &'a dyn Fn(Config<Core::ApplicationConfig>) -> Core::ABCIHandler,
        params_store_key: Core::StoreKey,
        params_subspace_key: Core::ParamsSubspaceKey,
    ) -> Self {
        Self {
            core,
            abci_handler_builder,
            params_store_key,
            params_subspace_key,
        }
    }

    /// Runs the command passed on the command line.
    pub fn execute<AI: ApplicationInfo>(
        self,
        command: AppCommands<Core::AuxCommands>,
    ) -> anyhow::Result<()> {
        match command {
            AppCommands::Init(cmd) => {
                init::<_, Core::ApplicationConfig>(cmd, &Core::Genesis::default())?
            }
            AppCommands::Run(cmd) => run::<_, _, _, _, _, _, AI, _, _>(
                cmd,
                ParamsKeeper::new(self.params_store_key),
                self.params_subspace_key,
                self.abci_handler_builder,
                self.core,
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
