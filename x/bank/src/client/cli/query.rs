use std::borrow::Cow;
use std::fmt::Debug;

use bytes::Bytes;
use clap::{Args, Subcommand};

use gears::{
    application::handlers::QueryHandler,
    ibc::{address::AccAddress, query::request::bank::QueryDenomsMetadataRequest},
    tendermint::types::proto::Protobuf,
    types::query::Query,
};
use prost::Message;
// use proto_messages::cosmos::{
//     bank::v1beta1::{
//         QueryAllBalancesRequest, QueryAllBalancesResponse, QueryDenomsMetadataRequest,
//         QueryDenomsMetadataResponse,
//     },
//     ibc::protobuf::Protobuf,
//     query::Query,
// };
// use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

use crate::types::query::{
    QueryAllBalancesRequest, QueryAllBalancesResponse, QueryDenomsMetadataResponse,
};

#[derive(Args, Debug)]
pub struct BankQueryCli {
    #[command(subcommand)]
    pub command: BankCommands,
}

#[derive(Subcommand, Debug)]
pub enum BankCommands {
    Balances(BalancesCommand),
    DenomMetadata,
}

/// Query for account balances by address
#[derive(Args, Debug, Clone)]
pub struct BalancesCommand {
    /// address
    pub address: AccAddress,
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
            BankCommands::Balances(BalancesCommand { address }) => {
                BankQuery::Balances(QueryAllBalancesRequest {
                    address: address.clone(),
                    pagination: None,
                })
            }
            BankCommands::DenomMetadata => {
                BankQuery::DenomMetadata(QueryDenomsMetadataRequest { pagination: None })
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
            BankCommands::DenomMetadata => BankQueryResponse::DenomMetadata(
                QueryDenomsMetadataResponse::decode::<Bytes>(query_bytes.into())?,
            ),
        };

        Ok(res)
    }
}

#[derive(Clone, PartialEq)]
pub enum BankQuery {
    Balances(QueryAllBalancesRequest),
    DenomMetadata(QueryDenomsMetadataRequest),
}

impl Query for BankQuery {
    fn query_url(&self) -> Cow<'static, str> {
        match self {
            BankQuery::Balances(_) => Cow::Borrowed("/cosmos.bank.v1beta1.Query/AllBalances"),
            BankQuery::DenomMetadata(_) => {
                Cow::Borrowed("/cosmos.bank.v1beta1.Query/DenomsMetadata")
            }
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            BankQuery::Balances(var) => var.encode_vec().expect("msg"), // TODO:NOW
            BankQuery::DenomMetadata(var) => var.encode_to_vec(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum BankQueryResponse {
    Balances(QueryAllBalancesResponse),
    DenomMetadata(QueryDenomsMetadataResponse),
}
