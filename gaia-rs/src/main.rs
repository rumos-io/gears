#![warn(rust_2018_idioms)]

use anyhow::Result;
use auth::Keeper as AuthKeeper;
use bank::Keeper as BankKeeper;
use client::query_command_handler;
use client::tx_command_handler;
use gears::x::params::Keeper as ParamsKeeper;
use gears::Application;
use gears::NilAuxCommand;
use gears::Node;
use genesis::GenesisState;
use rest::get_router;

use crate::handler::Handler;
use crate::store_keys::{GaiaParamsStoreKey, GaiaStoreKey};

mod client;
mod config;
mod genesis;
mod handler;
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
    type BankKeeper = BankKeeper<GaiaStoreKey, GaiaParamsStoreKey>;
    type AuthKeeper = AuthKeeper<GaiaStoreKey, GaiaParamsStoreKey>;
    type Handler = Handler;
    type QuerySubcommand = client::QueryCommands;
    type TxSubcommand = client::Commands;
    type ApplicationConfig = config::AppConfig;
    type AuxCommands = NilAuxCommand;

    fn get_router(
        &self,
    ) -> axum::Router<
        gears::client::rest::RestState<
            Self::StoreKey,
            Self::ParamsSubspaceKey,
            Self::Message,
            Self::BankKeeper,
            Self::AuthKeeper,
            Self::Handler,
            Self::Genesis,
        >,
        axum::body::Body,
    > {
        get_router()
    }

    fn get_params_store_key(&self) -> Self::StoreKey {
        Self::StoreKey::Params
    }

    fn get_params_subspace_key(&self) -> Self::ParamsSubspaceKey {
        Self::ParamsSubspaceKey::BaseApp
    }

    fn get_handler(&self, cfg: gears::config::Config<Self::ApplicationConfig>) -> Self::Handler {
        Handler::new(cfg)
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

    Node::new(bank_keeper, auth_keeper, GaiaApplication).run_command()
}
