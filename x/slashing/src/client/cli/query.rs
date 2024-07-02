use clap::{Args, Subcommand};
use gears::{
    application::handlers::client::QueryHandler,
    core::Protobuf,
    tendermint::types::proto::crypto::PublicKey,
    types::{address::ConsAddress, query::Query},
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Debug};

use crate::{QuerySigningInfoRequest, QuerySigningInfoResponse};

#[derive(Args, Debug)]
pub struct SlashingQueryCli {
    #[command(subcommand)]
    pub command: SlashingCommands,
}

#[derive(Subcommand, Debug)]
pub enum SlashingCommands {
    SigningInfo(SigningInfoCommand),
}

/// Query signing info.
#[derive(Args, Debug, Clone)]
pub struct SigningInfoCommand {
    /// validator public key
    pub pubkey: PublicKey,
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
        };

        Ok(res)
    }
}

#[derive(Clone, PartialEq)]
pub enum SlashingQueryRequest {
    SigningInfo(QuerySigningInfoRequest),
}

impl Query for SlashingQueryRequest {
    fn query_url(&self) -> Cow<'static, str> {
        match self {
            SlashingQueryRequest::SigningInfo(_) => {
                Cow::Borrowed("/cosmos.slashing.v1beta1.Query/SigningInfo")
            }
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            SlashingQueryRequest::SigningInfo(var) => var.encode_vec(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum SlashingQueryResponse {
    SigningInfo(QuerySigningInfoResponse),
}
