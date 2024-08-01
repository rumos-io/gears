use crate::{
    QueryDelegationRequest, QueryDelegationResponse, QueryRedelegationRequest,
    QueryRedelegationResponse, QueryUnbondingDelegationResponse, QueryValidatorRequest,
    QueryValidatorResponse,
};
use clap::{Args, Subcommand};
use gears::{
    application::handlers::client::QueryHandler,
    derive::Query,
    tendermint::types::proto::Protobuf as _,
    types::address::{AccAddress, ValAddress},
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Args, Debug)]
pub struct StakingQueryCli {
    #[command(subcommand)]
    pub command: StakingCommands,
}

#[derive(Subcommand, Debug)]
pub enum StakingCommands {
    Validator(ValidatorCommand),
    Delegation(DelegationCommand),
    Redelegation(RedelegationCommand),
    UnbondingDelegation(UnbondingDelegationCommand),
}

/// Query for validator account by address
#[derive(Args, Debug, Clone)]
pub struct ValidatorCommand {
    /// address
    pub address: ValAddress,
}

/// Query for delegation from a delegator address to validator address
#[derive(Args, Debug, Clone)]
pub struct DelegationCommand {
    /// Delegator address who sent delegation
    pub delegator_address: AccAddress,
    /// Validator address which is addressed to delegation
    pub validator_address: ValAddress,
}

/// Query implements the command to query a single redelegation record.
#[derive(Args, Debug, Clone)]
pub struct RedelegationCommand {
    /// Delegator address who sent delegation
    pub delegator_address: AccAddress,
    /// Source validator address which is addressed to delegation
    pub src_validator_address: ValAddress,
    /// Destination validator address which is addressed to delegation
    pub dst_validator_address: ValAddress,
}

/// Query an unbonding-delegation record based on delegator and validator address
#[derive(Args, Debug, Clone)]
pub struct UnbondingDelegationCommand {
    /// Delegator address who sent unbonding request
    pub delegator_address: AccAddress,
    /// Validator address from which coins are unbonded
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
            StakingCommands::Redelegation(RedelegationCommand {
                delegator_address,
                src_validator_address,
                dst_validator_address,
            }) => StakingQuery::Redelegation(QueryRedelegationRequest {
                delegator_address: delegator_address.clone().into(),
                src_validator_address: src_validator_address.clone().into(),
                dst_validator_address: dst_validator_address.clone().into(),
                pagination: None,
            }),
            StakingCommands::UnbondingDelegation(UnbondingDelegationCommand {
                delegator_address,
                validator_address,
            }) => StakingQuery::UnbondingDelegation(QueryDelegationRequest {
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
            StakingCommands::Redelegation(_) => StakingQueryResponse::Redelegation(
                QueryRedelegationResponse::decode_vec(&query_bytes)?,
            ),
            StakingCommands::UnbondingDelegation(_) => StakingQueryResponse::UnbondingDelegation(
                QueryUnbondingDelegationResponse::decode_vec(&query_bytes)?,
            ),
        };

        Ok(res)
    }
}

#[derive(Clone, PartialEq, Query)]
#[query(request)]
pub enum StakingQuery {
    Validator(QueryValidatorRequest),
    Delegation(QueryDelegationRequest),
    Redelegation(QueryRedelegationRequest),
    UnbondingDelegation(QueryDelegationRequest),
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum StakingQueryResponse {
    Validator(QueryValidatorResponse),
    Delegation(QueryDelegationResponse),
    Redelegation(QueryRedelegationResponse),
    UnbondingDelegation(QueryUnbondingDelegationResponse),
}
