use std::borrow::Cow;

use bytes::Bytes;
use clap::{Args, Subcommand};

use gears::ibc::Protobuf;
use gears::ibc::{address::AccAddress, query::request::account::QueryAccountRequest};
use gears::tendermint::types::proto::Protobuf as _;
use gears::{
    application::handlers::QueryHandler,
    types::query::{account::QueryAccountResponse, Query},
};
use serde::{Deserialize, Serialize};

// use proto_messages::cosmos::{
//     auth::v1beta1::{QueryAccountRequest, QueryAccountResponse},
//     ibc::protobuf::Protobuf,
//     query::Query,
// };
// use proto_types::AccAddress;

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

#[derive(Clone, PartialEq)]
pub enum AuthQuery {
    Account(QueryAccountRequest),
}

impl Query for AuthQuery {
    fn query_url(&self) -> Cow<'static, str> {
        match self {
            AuthQuery::Account(_) => Cow::Borrowed("/cosmos.auth.v1beta1.Query/Account"),
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            AuthQuery::Account(cmd) => cmd.encode_vec().expect("msg"), //TODO:NOW
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
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
