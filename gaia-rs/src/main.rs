#![warn(rust_2018_idioms)]

use anyhow::Result;
use clap::Command;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use clap_complete::Generator;
use client::query_command_handler;
use client::tx_command_handler;
use gaia_rs::GaiaApplication;
use gears::cli::CliApplicationArgs;
use gears::ApplicationBuilder;
use gears::ApplicationCore;
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

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

fn main() -> Result<()> {
    let args: CliApplicationArgs<GaiaApplication> = CliApplicationArgs::parse();

    if let Some(generator) = args.completion {
        let mut cmd = CliApplicationArgs::<GaiaApplication>::command();
        print_completions(generator, &mut cmd);
    }

    if let Some(command) = args.command {
        ApplicationBuilder::<'_, _, GaiaApplication>::new(
            GaiaCore,
            get_router(),
            &ABCIHandler::new,
            GaiaStoreKey::Params,
            GaiaParamsStoreKey::BaseApp,
        )
        .execute(command.into())
    } else {
        CliApplicationArgs::<GaiaApplication>::command().print_long_help()?;
        Ok(())
    }
}
