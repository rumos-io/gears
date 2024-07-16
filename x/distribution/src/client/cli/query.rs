use crate::{
    QueryParamsRequest, QueryParamsResponse, QueryValidatorCommissionRequest,
    QueryValidatorCommissionResponse, QueryValidatorOutstandingRewardsRequest,
    QueryValidatorOutstandingRewardsResponse,
};
use clap::{Args, Subcommand};
use gears::{
    application::handlers::client::QueryHandler,
    core::Protobuf,
    types::{address::ValAddress, query::Query},
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Args, Debug)]
pub struct DistributionQueryCli {
    #[command(subcommand)]
    pub command: DistributionCommands,
}

#[derive(Subcommand, Debug)]
pub enum DistributionCommands {
    ValidatorOutstandingRewards(ValidatorOutstandingRewardsCommand),
    ValidatorCommission(ValidatorCommissionCommand),
    /// Query distribution params
    Params,
}

/// Query distribution outstanding (un-withdrawn) rewards for a validator and all their delegations
#[derive(Args, Debug, Clone)]
pub struct ValidatorOutstandingRewardsCommand {
    /// validator address
    pub address: ValAddress,
}

/// Query distribution validator commission
#[derive(Args, Debug, Clone)]
pub struct ValidatorCommissionCommand {
    /// validator address
    pub address: ValAddress,
}

#[derive(Debug, Clone)]
pub struct DistributionQueryHandler;

impl QueryHandler for DistributionQueryHandler {
    type QueryRequest = DistributionQueryRequest;

    type QueryResponse = DistributionQueryResponse;

    type QueryCommands = DistributionQueryCli;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let res = match &command.command {
            DistributionCommands::ValidatorOutstandingRewards(
                ValidatorOutstandingRewardsCommand { address },
            ) => Self::QueryRequest::ValidatorOutstandingRewards(
                QueryValidatorOutstandingRewardsRequest {
                    validator_address: address.clone(),
                },
            ),
            DistributionCommands::ValidatorCommission(ValidatorCommissionCommand { address }) => {
                Self::QueryRequest::ValidatorCommission(QueryValidatorCommissionRequest {
                    validator_address: address.clone(),
                })
            }
            DistributionCommands::Params => Self::QueryRequest::Params(QueryParamsRequest {}),
        };

        Ok(res)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let res = match &command.command {
            DistributionCommands::ValidatorOutstandingRewards(_) => {
                DistributionQueryResponse::ValidatorOutstandingRewards(
                    QueryValidatorOutstandingRewardsResponse::decode_vec(&query_bytes)?,
                )
            }
            DistributionCommands::ValidatorCommission(_) => {
                DistributionQueryResponse::ValidatorCommission(
                    QueryValidatorCommissionResponse::decode_vec(&query_bytes)?,
                )
            }
            DistributionCommands::Params => {
                DistributionQueryResponse::Params(QueryParamsResponse::decode_vec(&query_bytes)?)
            }
        };

        Ok(res)
    }
}

#[derive(Clone, PartialEq)]
pub enum DistributionQueryRequest {
    ValidatorOutstandingRewards(QueryValidatorOutstandingRewardsRequest),
    ValidatorCommission(QueryValidatorCommissionRequest),
    Params(QueryParamsRequest),
}

impl Query for DistributionQueryRequest {
    fn query_url(&self) -> &'static str {
        match self {
            DistributionQueryRequest::ValidatorOutstandingRewards(_) => {
                "/cosmos.distribution.v1beta1.Query/ValidatorOutstandingRewards"
            }
            DistributionQueryRequest::ValidatorCommission(_) => {
                "/cosmos.distribution.v1beta1.Query/ValidatorCommission"
            }
            DistributionQueryRequest::Params(_) => "/cosmos.distribution.v1beta1.Query/Params",
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            DistributionQueryRequest::ValidatorOutstandingRewards(var) => var.encode_vec(),
            DistributionQueryRequest::ValidatorCommission(var) => var.encode_vec(),
            DistributionQueryRequest::Params(var) => var.encode_vec(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum DistributionQueryResponse {
    ValidatorOutstandingRewards(QueryValidatorOutstandingRewardsResponse),
    ValidatorCommission(QueryValidatorCommissionResponse),
    Params(QueryParamsResponse),
}
