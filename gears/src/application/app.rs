use axum::{body::Body, Router};
use proto_messages::cosmos::tx::v1beta1::message::Message;
use store_crate::StoreKey;

use crate::{
    baseapp::{run, ABCIHandler, Genesis},
    client::{genesis_account, init, keys, rest::RestState},
    config::{ApplicationConfig, Config},
    x::params::ParamsSubspaceKey,
};

use super::{command::app::AppCommands, ApplicationInfo};
use crate::x::params::Keeper as ParamsKeeper;

/// A Gears application.
pub trait ApplicationTrait {
    type Message: Message;
    type Genesis: Genesis;
    type StoreKey: StoreKey;
    type ParamsSubspaceKey: ParamsSubspaceKey;
    type ABCIHandler: ABCIHandler<Self::Message, Self::StoreKey, Self::Genesis>;
    type ApplicationConfig: ApplicationConfig;

    fn router<AI: ApplicationInfo>() -> Router<
        RestState<
            Self::StoreKey,
            Self::ParamsSubspaceKey,
            Self::Message,
            Self::ABCIHandler,
            Self::Genesis,
            AI,
        >,
        Body,
    >;
}

pub struct Application<'a, AppCore: ApplicationTrait, AI: ApplicationInfo> {
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

impl<'a, Core: ApplicationTrait, AI: ApplicationInfo> Application<'a, Core, AI> {
    pub fn new(
        abci_handler_builder: &'a dyn Fn(Config<Core::ApplicationConfig>) -> Core::ABCIHandler,

        params_store_key: Core::StoreKey,
        params_subspace_key: Core::ParamsSubspaceKey,
    ) -> Self {
        Self {
            router: Core::router(),
            abci_handler_builder,
            params_store_key,
            params_subspace_key,
        }
    }

    /// Runs the command passed on the command line.
    pub fn execute(self, command: AppCommands) -> anyhow::Result<()> {
        match command {
            AppCommands::Init(cmd) => {
                init::init::<_, Core::ApplicationConfig>(cmd, &Core::Genesis::default())?
            }
            AppCommands::Run(cmd) => run::run(
                cmd,
                ParamsKeeper::new(self.params_store_key),
                self.params_subspace_key,
                self.abci_handler_builder,
                self.router,
            )?,
            AppCommands::Keys(cmd) => keys::keys(cmd)?,
            AppCommands::GenesisAdd(cmd) => {
                genesis_account::genesis_account_add::<Core::Genesis>(cmd)?
            }
        };

        Ok(())
    }
}
