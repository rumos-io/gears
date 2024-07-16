use crate::{
    QueryParamsRequest, QueryParamsResponse, QuerySigningInfoRequest, QuerySigningInfoResponse,
    QuerySigningInfosRequest, QuerySigningInfosResponse,
};
use clap::{Args, Subcommand};
use gears::{
    application::handlers::client::QueryHandler,
    core::{query::request::PageRequest, Protobuf},
    error::AppError,
    tendermint::types::proto::crypto::PublicKey,
    types::{address::ConsAddress, query::Query},
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Args, Debug)]
pub struct SlashingQueryCli {
    #[command(subcommand)]
    pub command: SlashingCommands,
}

#[derive(Subcommand, Debug)]
pub enum SlashingCommands {
    SigningInfo(SigningInfoCommand),
    SigningInfos(SigningInfosCommand),
    Params,
}

/// Query signing info.
#[derive(Args, Debug, Clone)]
pub struct SigningInfoCommand {
    /// validator public key
    pub pubkey: PublicKey,
}

/// Query signing infos.
#[derive(Args, Debug, Clone)]
pub struct SigningInfosCommand {
    /// Sets offset to a multiple of limit
    #[arg(long, default_value_t = 1)]
    pub page: u64,
    /// Pagination page-key
    #[arg(long, default_value_t = String::default())]
    pub page_key: String,
    /// Pagination offset
    #[arg(long, default_value_t = 0)]
    pub offset: u64,
    /// Pagination limit
    #[arg(long, default_value_t = 100)]
    pub limit: u64,
    /// Count total number of records
    #[arg(long, default_value_t = false)]
    pub count_total: bool,
    /// Results are sorted in descending order
    #[arg(long, default_value_t = false)]
    pub reverse: bool,
}

#[derive(Debug, Clone)]
pub struct SlashingQueryHandler;

impl QueryHandler for SlashingQueryHandler {
    type QueryRequest = SlashingQueryRequest;

    type QueryResponse = SlashingQueryResponse;

    type QueryCommands = SlashingQueryCli;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let res = match &command.command {
            SlashingCommands::SigningInfo(SigningInfoCommand { pubkey }) => {
                let cons_address: ConsAddress = pubkey.clone().into();
                Self::QueryRequest::SigningInfo(QuerySigningInfoRequest { cons_address })
            }
            SlashingCommands::SigningInfos(SigningInfosCommand {
                page,
                page_key,
                offset,
                limit,
                count_total,
                reverse,
            }) => {
                if *page > 1 && *offset > 0 {
                    return Err(AppError::InvalidRequest(
                        "page and offset cannot be used together".to_string(),
                    )
                    .into());
                }

                let offset = if *page > 1 {
                    (*page - 1) * limit
                } else {
                    *offset
                };

                Self::QueryRequest::SigningInfos(QuerySigningInfosRequest {
                    pagination: PageRequest {
                        key: page_key.as_bytes().to_vec(),
                        offset,
                        limit: *limit,
                        count_total: *count_total,
                        reverse: *reverse,
                    },
                })
            }
            SlashingCommands::Params => Self::QueryRequest::Params(QueryParamsRequest {}),
        };

        Ok(res)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let res = match &command.command {
            SlashingCommands::SigningInfo(_) => SlashingQueryResponse::SigningInfo(
                QuerySigningInfoResponse::decode_vec(&query_bytes)?,
            ),
            SlashingCommands::SigningInfos(_) => SlashingQueryResponse::SigningInfos(
                QuerySigningInfosResponse::decode_vec(&query_bytes)?,
            ),
            SlashingCommands::Params => {
                SlashingQueryResponse::Params(QueryParamsResponse::decode_vec(&query_bytes)?)
            }
        };

        Ok(res)
    }
}

#[derive(Clone, PartialEq)]
pub enum SlashingQueryRequest {
    SigningInfo(QuerySigningInfoRequest),
    SigningInfos(QuerySigningInfosRequest),
    Params(QueryParamsRequest),
}

impl Query for SlashingQueryRequest {
    fn query_url(&self) -> &'static str {
        match self {
            SlashingQueryRequest::SigningInfo(_) => "/cosmos.slashing.v1beta1.Query/SigningInfo",
            SlashingQueryRequest::SigningInfos(_) => "/cosmos.slashing.v1beta1.Query/SigningInfos",
            SlashingQueryRequest::Params(_) => "/cosmos.slashing.v1beta1.Query/Params",
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            SlashingQueryRequest::SigningInfo(var) => var.encode_vec(),
            SlashingQueryRequest::SigningInfos(var) => var.encode_vec(),
            SlashingQueryRequest::Params(var) => var.encode_vec(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum SlashingQueryResponse {
    SigningInfo(QuerySigningInfoResponse),
    SigningInfos(QuerySigningInfosResponse),
    Params(QueryParamsResponse),
}
