use std::fmt::Debug;

use bytes::Bytes;
use clap::{Args, Subcommand};

use gears::{
    application::handlers::client::QueryHandler,
    cli::pagination::CliPaginationRequest,
    error::IBC_ENCODE_UNWRAP,
    ext::FallibleMapExt,
    tendermint::types::proto::Protobuf,
    types::{address::AccAddress, pagination::request::PaginationRequest, query::Query},
};
use serde::{Deserialize, Serialize};

use crate::types::query::{
    QueryAllBalancesRequest, QueryAllBalancesResponse, QueryDenomsMetadataRequest,
    QueryDenomsMetadataResponse,
};

#[derive(Args, Debug)]
pub struct BankQueryCli {
    #[command(subcommand)]
    pub command: BankCommands,
}

#[derive(Subcommand, Debug)]
pub enum BankCommands {
    Balances(BalancesCommand),
    DenomMetadata {
        #[command(flatten)]
        pagination: Option<CliPaginationRequest>,
    },
}

/// Query for account balances by address
#[derive(Args, Debug, Clone)]
pub struct BalancesCommand {
    /// address
    pub address: AccAddress,
    #[command(flatten)]
    pub pagination: Option<CliPaginationRequest>,
}

#[derive(Debug, Clone)]
pub struct BankQueryHandler;

impl QueryHandler for BankQueryHandler {
    type QueryRequest = BankQuery;

    type QueryResponse = BankQueryResponse;

    type QueryCommands = BankQueryCli;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let res = match &command.command {
            BankCommands::Balances(BalancesCommand {
                address,
                pagination,
            }) => BankQuery::Balances(QueryAllBalancesRequest {
                address: address.clone(),
                pagination: pagination.to_owned().try_map(PaginationRequest::try_from)?,
            }),
            BankCommands::DenomMetadata { pagination } => {
                BankQuery::DenomMetadata(QueryDenomsMetadataRequest {
                    pagination: pagination.to_owned().try_map(PaginationRequest::try_from)?,
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
        let res = match &command.command {
            BankCommands::Balances(_) => BankQueryResponse::Balances(
                QueryAllBalancesResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            BankCommands::DenomMetadata { pagination: _ } => BankQueryResponse::DenomMetadata(
                QueryDenomsMetadataResponse::decode::<Bytes>(query_bytes.into())?,
            ),
        };

        Ok(res)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BankQuery {
    Balances(QueryAllBalancesRequest),
    DenomMetadata(QueryDenomsMetadataRequest),
}

impl Query for BankQuery {
    fn query_url(&self) -> &'static str {
        match self {
            BankQuery::Balances(_) => "/cosmos.bank.v1beta1.Query/AllBalances",
            BankQuery::DenomMetadata(_) => "/cosmos.bank.v1beta1.Query/DenomsMetadata",
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            BankQuery::Balances(var) => var.encode_vec().expect(IBC_ENCODE_UNWRAP), // TODO:IBC
            BankQuery::DenomMetadata(var) => var.encode_vec().expect(IBC_ENCODE_UNWRAP),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum BankQueryResponse {
    Balances(QueryAllBalancesResponse),
    DenomMetadata(QueryDenomsMetadataResponse),
}
