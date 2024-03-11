#![warn(rust_2018_idioms)]

use anyhow::Result;
use clap::Parser;
use client::query_command_handler;
use client::tx_command_handler;
use client::GaiaQueryCommands;
use client::GaiaTxCommands;
use gaia_rs::GaiaApplication;
use gears::application::app::Application;
use gears::application::app::ApplicationTrait;
use gears::application::client::ClientApplication;
use gears::application::client::ClientTrait;
use gears::application::command::NilAuxCommand;
use gears::application::handlers::AuxHandler;
use gears::application::handlers::QueryHandler;
use gears::application::handlers::TxHandler;
use gears::application::ApplicationInfo;
use gears::cli::aux::CliNilAuxCommand;
use gears::cli::CliApplicationArgs;
use genesis::GenesisState;
use proto_types::AccAddress;
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

struct GaiaCore;

impl TxHandler for GaiaCore {
    type Message = message::Message;
    type TxCommands = client::GaiaTxCommands;

    fn handle_tx_command(
        &self,
        command: Self::TxCommands,
        from_address: AccAddress,
    ) -> Result<Self::Message> {
        tx_command_handler(command, from_address)
    }
}

impl QueryHandler for GaiaCore {
    type QueryCommands = client::GaiaQueryCommands;

    fn handle_query_command(
        &self,
        command: Self::QueryCommands,
        node: &str,
        height: Option<tendermint::informal::block::Height>,
    ) -> Result<()> {
        tokio::runtime::Runtime::new()
            .expect("unclear why this would ever fail")
            .block_on(query_command_handler(command, node, height))
    }
}

impl AuxHandler for GaiaCore {
    type AuxCommands = NilAuxCommand;

    fn handle_aux_commands(&self, _command: Self::AuxCommands) -> Result<()> {
        println!("{} doesn't have any AUX command", GaiaApplication::APP_NAME);
        Ok(())
    }
}

impl ClientTrait for GaiaCore {}

impl ApplicationTrait for GaiaCore {
    type Message = message::Message;
    type Genesis = GenesisState;
    type StoreKey = GaiaStoreKey;
    type ParamsSubspaceKey = GaiaParamsStoreKey;
    type ABCIHandler = ABCIHandler;
    type ApplicationConfig = config::AppConfig;

    fn router<AI: ApplicationInfo>() -> axum::Router<
        gears::client::rest::RestState<
            Self::StoreKey,
            Self::ParamsSubspaceKey,
            Self::Message,
            Self::ABCIHandler,
            Self::Genesis,
            AI,
        >,
        axum::body::Body,
    > {
        get_router()
    }
}

type Args =
    CliApplicationArgs<GaiaApplication, CliNilAuxCommand, GaiaTxCommands, GaiaQueryCommands>;

fn main() -> Result<()> {
    let args = Args::parse();

    args.execute_or_help(
        |command| ClientApplication::new(GaiaCore).execute(command.try_into()?),
        |command| {
            Application::<'_, GaiaCore, GaiaApplication>::new(
                &ABCIHandler::new,
                GaiaStoreKey::Params,
                GaiaParamsStoreKey::BaseApp,
            )
            .execute(command.try_into()?)
        },
    )
}
