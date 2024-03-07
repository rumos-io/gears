#![warn(rust_2018_idioms)]

use anyhow::Result;
use clap::Command;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use clap_complete::Generator;
use client::query_command_handler;
use client::tx_command_handler;
use client::GaiaQueryCommands;
use client::GaiaTxCommands;
use gaia_rs::GaiaApplication;
use gears::cli::CliApplicationArgs;
use gears::cli::CliNilAuxCommand;
use gears::ApplicationBuilder;
use gears::ApplicationCore;
use gears::NilAuxCommand;
use gears::QueryHandler;
use gears::TxHandler;
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

impl ApplicationCore for GaiaCore {
    type Genesis = GenesisState;
    type StoreKey = GaiaStoreKey;
    type ParamsSubspaceKey = GaiaParamsStoreKey;

    type ABCIHandler = ABCIHandler;

    type ApplicationConfig = config::AppConfig;
    type AuxCommands = NilAuxCommand;

    fn handle_aux_commands(&self, _command: Self::AuxCommands) -> Result<()> {
        Ok(())
    }
}

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
        query_command_handler(command, node, height)
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

type Args = CliApplicationArgs<
    GaiaApplication,
    CliNilAuxCommand<GaiaApplication>,
    GaiaTxCommands,
    GaiaQueryCommands,
>;

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(generator) = args.completion {
        let mut cmd = Args::command();
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
        Args::command().print_long_help()?;
        Ok(())
    }
}
