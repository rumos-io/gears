use bytes::Bytes;
use clap::{Args, Subcommand};
use gears::application::handlers::client::QueryHandler;
use gears::derive::Query;
use gears::tendermint::types::proto::Protobuf as _;
use gears::types::address::AccAddress;
use serde::{Deserialize, Serialize};

use crate::query::{QueryAccountRequest, QueryAccountResponse};

#[derive(Args, Debug)]
pub struct AuthQueryCli {
    #[command(subcommand)]
    pub command: AuthCommands,
}

#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    Account(AccountCommand),
}

/// Query for account by address
#[derive(Args, Debug, Clone)]
pub struct AccountCommand {
    /// address
    pub address: AccAddress,
}

#[derive(Clone, PartialEq, Query)]
#[query(kind = "request")]
pub enum AuthQuery {
    Account(QueryAccountRequest),
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
#[query(kind = "response")]
#[serde(untagged)]
pub enum AuthQueryResponse {
    Account(QueryAccountResponse),
}

#[derive(Debug, Clone)]
pub struct AuthQueryHandler;

impl QueryHandler for AuthQueryHandler {
    type QueryRequest = AuthQuery;

    type QueryCommands = AuthQueryCli;

    type QueryResponse = AuthQueryResponse;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let res = match &command.command {
            AuthCommands::Account(AccountCommand { address }) => {
                AuthQuery::Account(QueryAccountRequest {
                    address: address.clone(),
                })
            }
        };

        Ok(res)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let res =
            match command.command {
                AuthCommands::Account(_) => AuthQueryResponse::Account(
                    QueryAccountResponse::decode::<Bytes>(query_bytes.into())?,
                ),
            };

        Ok(res)
    }
}
