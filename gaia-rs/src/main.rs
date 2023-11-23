#![warn(rust_2018_idioms)]

use anyhow::Result;
use auth::Keeper as AuthKeeper;
use bank::Keeper as BankKeeper;
use client::query_command_handler;
use client::tx_command_handler;
use gears::baseapp::ante::BaseAnteHandler;
use gears::x::params::Keeper as ParamsKeeper;
use gears::Application;
use gears::NilAuxCommand;
use gears::Node;
use genesis::GenesisState;
use rest::get_router;

use crate::abci_handler::ABCIHandler;
use crate::store_keys::{GaiaParamsStoreKey, GaiaStoreKey};

mod abci_handler;
mod client;
mod config;
mod genesis;
mod message;
mod rest;
mod store_keys;

struct GaiaApplication;

impl Application for GaiaApplication {
    const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
    const APP_VERSION: &'static str = env!("GIT_HASH");
    type Genesis = GenesisState;
    type StoreKey = GaiaStoreKey;
    type ParamsSubspaceKey = GaiaParamsStoreKey;
    type Message = message::Message;
    type ABCIHandler = ABCIHandler;
    type QuerySubcommand = client::QueryCommands;
    type TxSubcommand = client::Commands;
    type ApplicationConfig = config::AppConfig;
    type AuxCommands = NilAuxCommand;
    type AnteHandler = BaseAnteHandler<
        BankKeeper<Self::StoreKey, Self::ParamsSubspaceKey>,
        AuthKeeper<Self::StoreKey, Self::ParamsSubspaceKey>,
        Self::StoreKey,
    >;

    fn get_params_store_key(&self) -> Self::StoreKey {
        Self::StoreKey::Params
    }

    fn get_params_subspace_key(&self) -> Self::ParamsSubspaceKey {
        Self::ParamsSubspaceKey::BaseApp
    }

    fn handle_tx_command(
        &self,
        command: Self::TxSubcommand,
        from_address: proto_types::AccAddress,
    ) -> Result<Self::Message> {
        tx_command_handler(command, from_address)
    }

    fn handle_query_command(
        &self,
        command: Self::QuerySubcommand,
        node: &str,
        height: Option<tendermint_informal::block::Height>,
    ) -> Result<()> {
        query_command_handler(command, node, height)
    }
}

fn main() -> Result<()> {
    let params_keeper = ParamsKeeper::new(GaiaStoreKey::Params);
    let auth_keeper = AuthKeeper::new(
        GaiaStoreKey::Auth,
        params_keeper.clone(),
        GaiaParamsStoreKey::Auth,
    );
    let bank_keeper = BankKeeper::new(
        GaiaStoreKey::Bank,
        params_keeper.clone(),
        GaiaParamsStoreKey::Bank,
        auth_keeper.clone(),
    );
    let abci_handler_builder = |cfg| ABCIHandler::new(cfg);
    let ante_handler = BaseAnteHandler::new(bank_keeper, auth_keeper);

    Node::new(
        GaiaApplication,
        get_router(),
        &abci_handler_builder,
        ante_handler,
    )
    .run_command()
}
