#![warn(rust_2018_idioms)]

use anyhow::Result;
use auth::cli::query::AuthQueryHandler;
use bank::cli::query::BankQueryHandler;
use clap::Parser;
use client::tx_command_handler;
use client::GaiaQueryCommands;
use client::GaiaTxCommands;
use gaia_rs::query::GaiaQuery;
use gaia_rs::query::GaiaQueryResponse;
use gaia_rs::GaiaApplication;
use gears::application::client::Client;
use gears::application::client::ClientApplication;
use gears::application::command::NilAux;
use gears::application::command::NilAuxCommand;
use gears::application::handlers::AuxHandler;
use gears::application::handlers::{QueryHandler, TxHandler};
use gears::application::node::Node;
use gears::application::node::NodeApplication;
use gears::application::ApplicationInfo;
use gears::cli::aux::CliNilAuxCommand;
use gears::cli::CliApplicationArgs;
use genesis::GenesisState;
use ibc::cli::client::query::IbcQueryHandler;
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

    fn prepare_tx(
        &self,
        command: Self::TxCommands,
        from_address: AccAddress,
    ) -> Result<Self::Message> {
        tx_command_handler(command, from_address)
    }
}

impl QueryHandler for GaiaCore {
    type Query = GaiaQuery;

    type QueryCommands = GaiaQueryCommands;

    type QueryResponse = GaiaQueryResponse;

    fn prepare_query(&self, command: &Self::QueryCommands) -> anyhow::Result<Self::Query> {
        let res = match command {
            GaiaQueryCommands::Bank(command) => {
                Self::Query::Bank(BankQueryHandler.prepare_query(command)?)
            }
            GaiaQueryCommands::Auth(command) => {
                Self::Query::Auth(AuthQueryHandler.prepare_query(command)?)
            }
            GaiaQueryCommands::Ibc(command) => {
                Self::Query::Ibc(IbcQueryHandler.prepare_query(command)?)
            }
        };

        Ok(res)
    }

    fn handle_query(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let res = match command {
            GaiaQueryCommands::Bank(command) => {
                Self::QueryResponse::Bank(BankQueryHandler.handle_query(query_bytes, command)?)
            }
            GaiaQueryCommands::Auth(command) => {
                Self::QueryResponse::Auth(AuthQueryHandler.handle_query(query_bytes, command)?)
            }
            GaiaQueryCommands::Ibc(command) => {
                Self::QueryResponse::Ibc(IbcQueryHandler.handle_query(query_bytes, command)?)
            }
        };

        Ok(res)
    }

    fn render_query(&self, query: Self::QueryResponse) -> anyhow::Result<String> {
        let res = match query {
            GaiaQueryResponse::Auth(query) => AuthQueryHandler.render_query(query)?,
            GaiaQueryResponse::Bank(query) => BankQueryHandler.render_query(query)?,
            GaiaQueryResponse::Ibc(query) => IbcQueryHandler.render_query(query)?,
        };

        Ok(res)
    }
}

impl AuxHandler for GaiaCore {
    type AuxCommands = NilAuxCommand;
    type Aux = NilAux;

    fn prepare_aux(&self, _: Self::AuxCommands) -> anyhow::Result<Self::Aux> {
        println!("{} doesn't have any AUX command", GaiaApplication::APP_NAME);
        Ok(NilAux)
    }
}

impl Client for GaiaCore {}

impl Node for GaiaCore {
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

type Args = CliApplicationArgs<
    GaiaApplication,
    CliNilAuxCommand,
    CliNilAuxCommand,
    GaiaTxCommands,
    GaiaQueryCommands,
>;

fn main() -> Result<()> {
    let args = Args::parse();

    args.execute_or_help(
        |command| ClientApplication::new(GaiaCore).execute(command.try_into()?),
        |command| {
            NodeApplication::<'_, GaiaCore, GaiaApplication>::new(
                GaiaCore,
                &ABCIHandler::new,
                GaiaStoreKey::Params,
                GaiaParamsStoreKey::BaseApp,
            )
            .execute(command.try_into()?)
        },
    )
}
