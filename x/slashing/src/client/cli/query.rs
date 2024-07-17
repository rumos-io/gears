use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use clap::{Args, Subcommand};
use gears::{
    application::handlers::client::QueryHandler,
    cli::pagination::CliPaginationRequest,
    core::Protobuf,
    tendermint::types::proto::crypto::PublicKey,
    types::{address::ConsAddress, pagination::request::PaginationRequest, query::Query},
};

use crate::{
    QueryParamsRequest, QueryParamsResponse, QuerySigningInfoRequest, QuerySigningInfoResponse,
    QuerySigningInfosRequest, QuerySigningInfosResponse,
};

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
    #[command(flatten)]
    pub pagination: CliPaginationRequest,
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
            SlashingCommands::SigningInfos(cmd) => {
                let pagination = PaginationRequest::try_from(cmd.to_owned().pagination)?;

                Self::QueryRequest::SigningInfos(QuerySigningInfosRequest { pagination })
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
