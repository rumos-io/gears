use crate::{
    QueryDelegationRequest, QueryDelegationResponse, QueryValidatorRequest, QueryValidatorResponse,
};
use clap::{Args, Subcommand};
use gears::{
    application::handlers::client::QueryHandler,
    error::IBC_ENCODE_UNWRAP,
    tendermint::types::proto::Protobuf,
    types::{
        address::{AccAddress, ValAddress},
        query::Query,
    },
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Debug};

#[derive(Args, Debug)]
pub struct StakingQueryCli {
    #[command(subcommand)]
    pub command: StakingCommands,
}

#[derive(Subcommand, Debug)]
pub enum StakingCommands {
    Validator(ValidatorCommand),
    Delegation(DelegationCommand),
}

/// Query for validator account by address
#[derive(Args, Debug, Clone)]
pub struct ValidatorCommand {
    /// address
    pub address: ValAddress,
}

/// Query for validator account by address
#[derive(Args, Debug, Clone)]
pub struct DelegationCommand {
    /// Delegator address who sent delegation
    pub delegator_address: AccAddress,
    /// Validator address which is addressed to delegation
    pub validator_address: ValAddress,
}

#[derive(Debug, Clone)]
pub struct StakingQueryHandler;

impl QueryHandler for StakingQueryHandler {
    type QueryRequest = StakingQuery;

    type QueryResponse = StakingQueryResponse;

    type QueryCommands = StakingQueryCli;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let res = match &command.command {
            StakingCommands::Validator(ValidatorCommand { address }) => {
                StakingQuery::Validator(QueryValidatorRequest {
                    address: address.clone(),
                })
            }
            StakingCommands::Delegation(DelegationCommand {
                delegator_address,
                validator_address,
            }) => StakingQuery::Delegation(QueryDelegationRequest {
                delegator_address: delegator_address.clone(),
                validator_address: validator_address.clone(),
            }),
        };

        Ok(res)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let res = match &command.command {
            StakingCommands::Validator(_) => {
                StakingQueryResponse::Validator(QueryValidatorResponse::decode_vec(&query_bytes)?)
            }
            StakingCommands::Delegation(_) => {
                StakingQueryResponse::Delegation(QueryDelegationResponse::decode_vec(&query_bytes)?)
            }
        };

        Ok(res)
    }
}

#[derive(Clone, PartialEq)]
pub enum StakingQuery {
    Validator(QueryValidatorRequest),
    Delegation(QueryDelegationRequest),
}

impl Query for StakingQuery {
    fn query_url(&self) -> Cow<'static, str> {
        match self {
            StakingQuery::Validator(_) => Cow::Borrowed("/cosmos.staking.v1beta1.Query/Validator"),
            StakingQuery::Delegation(_) => {
                Cow::Borrowed("/cosmos.staking.v1beta1.Query/Delegation")
            }
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            StakingQuery::Validator(var) => var.encode_vec().expect(IBC_ENCODE_UNWRAP), // TODO:IBC
            StakingQuery::Delegation(var) => var.encode_vec().expect(IBC_ENCODE_UNWRAP), // TODO:IBC
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum StakingQueryResponse {
    Validator(QueryValidatorResponse),
    Delegation(QueryDelegationResponse),
}
