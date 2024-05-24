use super::{
    handlers::{node::ABCIHandler, AuxHandler},
    ApplicationInfo,
};
use crate::{
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
};

/// A Gears application.
pub trait Node:
    AuxHandler
    + RouterBuilder<<Self::Handler as ABCIHandler>::QReq, <Self::Handler as ABCIHandler>::QRes>
{
    type ParamsSubspaceKey: ParamsSubspaceKey;
    type Handler: ABCIHandler;
    type ApplicationConfig: ApplicationConfig;
}

pub struct NodeApplication<'a, Core: Node> {
    core: Core,
    abci_handler_builder: &'a dyn Fn(Config<Core::ApplicationConfig>) -> Core::Handler,

    params_store_key: <<Core as Node>::Handler as ABCIHandler>::StoreKey,
    params_subspace_key: Core::ParamsSubspaceKey,
}

impl<'a, Core: Node> NodeApplication<'a, Core> {
    pub fn new(
        core: Core,
        abci_handler_builder: &'a dyn Fn(Config<Core::ApplicationConfig>) -> Core::Handler,
        params_store_key: <<Core as Node>::Handler as ABCIHandler>::StoreKey,
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
            AppCommands::Init(cmd) => init::<_, Core::ApplicationConfig>(
                cmd,
                &<<Core as Node>::Handler as ABCIHandler>::Genesis::default(),
            )?,
            AppCommands::Run(cmd) => run::<_, _, _, AI, _>(
                cmd,
                ParamsKeeper::new(self.params_store_key),
                self.params_subspace_key,
                self.abci_handler_builder,
                self.core,
            )?,
            AppCommands::GenesisAdd(cmd) => {
                genesis_account_add::<<<Core as Node>::Handler as ABCIHandler>::Genesis>(cmd)?
            }
            AppCommands::Aux(cmd) => {
                let cmd = self.core.prepare_aux(cmd)?;
                self.core.handle_aux(cmd)?;
            }
        };

        Ok(())
    }
}
