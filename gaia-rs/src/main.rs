#![warn(rust_2018_idioms)]

use anyhow::Result;
use clap::Parser;
use client::query_command_handler;
use client::tx_command_handler;
use gears::ApplicationBuilder;
use gears::ApplicationCore;
use gears::DefaultApplication;
use gears::NilAuxCommand;
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

struct GaiaCore;

impl ApplicationCore for GaiaCore {
    type Genesis = GenesisState;
    type StoreKey = GaiaStoreKey;
    type ParamsSubspaceKey = GaiaParamsStoreKey;
    type Message = message::Message;
    type ABCIHandler = ABCIHandler;
    type QuerySubcommand = client::QueryCommands;
    type TxSubcommand = client::Commands;
    type ApplicationConfig = config::AppConfig;
    type AuxCommands = NilAuxCommand;

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
        height: Option<tendermint::informal::block::Height>,
    ) -> Result<()> {
        query_command_handler(command, node, height)
    }
}

fn main() -> Result<()> {
    let args: gears::cli::CliApplicationArgs<DefaultApplication> =
        gears::cli::CliApplicationArgs::parse();

    ApplicationBuilder::<'_, _, DefaultApplication>::new(
        GaiaCore,
        get_router(),
        &ABCIHandler::new,
        GaiaStoreKey::Params,
        GaiaParamsStoreKey::BaseApp,
    )
    .execute(args.into())
}
