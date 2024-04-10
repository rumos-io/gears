use crate::query::GaiaQuery;
use crate::query::GaiaQueryResponse;
use anyhow::Result;
use auth::cli::query::AuthQueryHandler;
use bank::cli::query::BankQueryHandler;
use client::tx_command_handler;
use client::GaiaQueryCommands;
use gears::application::client::Client;
use gears::application::handlers::AuxHandler;
use gears::application::handlers::{QueryHandler, TxHandler};
use gears::application::node::Node;
use gears::application::ApplicationInfo;
use gears::commands::NilAux;
use gears::commands::NilAuxCommand;
use gears::ibc::address::AccAddress;
use genesis::GenesisState;
// use ibc::client::cli::query_handler::IbcQueryHandler;
use crate::abci_handler::GaiaABCIHandler;
use crate::store_keys::{GaiaParamsStoreKey, GaiaStoreKey};
use rest::get_router;
use serde::Serialize;

pub mod abci_handler;
pub mod ante;
pub mod client;
pub mod config;
pub mod genesis;
pub mod message;
pub mod query;
pub mod rest;
pub mod store_keys;

#[derive(Debug, Clone, Serialize)]
pub struct GaiaApplication;

impl ApplicationInfo for GaiaApplication {
    const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
    const APP_VERSION: &'static str = env!("GIT_HASH");
}

pub struct GaiaCore;

pub struct GaiaCoreClient;

impl TxHandler for GaiaCoreClient {
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

impl QueryHandler for GaiaCoreClient {
    type QueryRequest = GaiaQuery;

    type QueryCommands = GaiaQueryCommands;

    type QueryResponse = GaiaQueryResponse;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let res = match command {
            GaiaQueryCommands::Bank(command) => {
                Self::QueryRequest::Bank(BankQueryHandler.prepare_query_request(command)?)
            }
            GaiaQueryCommands::Auth(command) => {
                Self::QueryRequest::Auth(AuthQueryHandler.prepare_query_request(command)?)
            } // GaiaQueryCommands::Ibc(command) => {
              //     Self::QueryRequest::Ibc(IbcQueryHandler.prepare_query_request(command)?)
              // }
        };

        Ok(res)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let res = match command {
            GaiaQueryCommands::Bank(command) => Self::QueryResponse::Bank(
                BankQueryHandler.handle_raw_response(query_bytes, command)?,
            ),
            GaiaQueryCommands::Auth(command) => Self::QueryResponse::Auth(
                AuthQueryHandler.handle_raw_response(query_bytes, command)?,
            ),
            // GaiaQueryCommands::Ibc(command) => {
            //     Self::QueryResponse::Ibc(IbcQueryHandler.handle_raw_response(query_bytes, command)?)
            // }
        };

        Ok(res)
    }
}

impl AuxHandler for GaiaCoreClient {
    type AuxCommands = NilAuxCommand;
    type Aux = NilAux;

    fn prepare_aux(&self, _: Self::AuxCommands) -> anyhow::Result<Self::Aux> {
        println!("{} doesn't have any AUX command", GaiaApplication::APP_NAME);
        Ok(NilAux)
    }
}

impl Client for GaiaCoreClient {}

impl Node for GaiaCore {
    type Message = message::Message;
    type Genesis = GenesisState;
    type StoreKey = GaiaStoreKey;
    type ParamsSubspaceKey = GaiaParamsStoreKey;
    type ABCIHandler = GaiaABCIHandler;
    type ApplicationConfig = config::AppConfig;

    fn router<AI: ApplicationInfo>() -> axum::Router<
        gears::rest::RestState<
            Self::StoreKey,
            Self::ParamsSubspaceKey,
            Self::Message,
            Self::ABCIHandler,
            Self::Genesis,
            AI,
        >,
    > {
        get_router()
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
