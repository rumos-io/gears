use crate::{
    QueryDelegationRequest, QueryDelegationResponse, QueryDelegatorDelegationsRequest,
    QueryDelegatorDelegationsResponse, QueryDelegatorUnbondingDelegationsRequest,
    QueryDelegatorUnbondingDelegationsResponse, QueryRedelegationRequest,
    QueryRedelegationResponse, QueryUnbondingDelegationResponse, QueryValidatorRequest,
    QueryValidatorResponse, QueryValidatorsRequest, QueryValidatorsResponse,
};
use clap::{Args, Subcommand};
use gears::{
    application::handlers::client::QueryHandler,
    cli::pagination::CliPaginationRequest,
    core::Protobuf,
    derive::Query,
    ext::FallibleMapExt,
    types::{
        address::{AccAddress, ValAddress},
        pagination::request::PaginationRequest,
    },
    x::types::validator::BondStatus,
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
    Validators(ValidatorsCommand),
    Delegation(DelegationCommand),
    Delegations(DelegatorDelegationsCommand),
    UnbondingDelegation(UnbondingDelegationCommand),
    UnbondingDelegations(UnbondingDelegationsCommand),
    Redelegation(RedelegationCommand),
}

/// Query for validator account by address
#[derive(Args, Debug, Clone)]
pub struct ValidatorCommand {
    /// address
    pub address: ValAddress,
}

/// Validators implements the query all validators command
#[derive(Args, Debug, Clone)]
pub struct ValidatorsCommand {
    #[command(flatten)]
    pub pagination: Option<CliPaginationRequest>,
}

/// Query for delegation from a delegator address to validator address
#[derive(Args, Debug, Clone)]
pub struct DelegationCommand {
    /// Delegator address who sent delegation
    pub delegator_address: AccAddress,
    /// Validator address which is addressed to delegation
    pub validator_address: ValAddress,
}

/// Query all the delegations made from one delegator
#[derive(Args, Debug, Clone)]
pub struct DelegatorDelegationsCommand {
    /// Delegator address who made delegations
    pub delegator_address: AccAddress,
    #[command(flatten)]
    pub pagination: Option<CliPaginationRequest>,
}

/// Query an unbonding-delegation record based on delegator and validator address
#[derive(Args, Debug, Clone)]
pub struct UnbondingDelegationCommand {
    /// Delegator address who sent unbonding request
    pub delegator_address: AccAddress,
    /// Validator address from which coins are unbonded
    pub validator_address: ValAddress,
}

/// Query unbonding-delegation records for a delegator
#[derive(Args, Debug, Clone)]
pub struct UnbondingDelegationsCommand {
    /// Delegator address who sent unbonding request
    pub delegator_address: AccAddress,
    #[command(flatten)]
    pub pagination: Option<CliPaginationRequest>,
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
                    validator_addr: address.clone(),
                })
            }
            StakingCommands::Validators(ValidatorsCommand { pagination }) => {
                StakingQuery::Validators(QueryValidatorsRequest {
                    status: BondStatus::Unspecified,
                    pagination: pagination.to_owned().try_map(PaginationRequest::try_from)?,
                })
            }
            StakingCommands::Delegation(DelegationCommand {
                delegator_address,
                validator_address,
            }) => StakingQuery::Delegation(QueryDelegationRequest {
                delegator_addr: delegator_address.clone(),
                validator_addr: validator_address.clone(),
            }),
            StakingCommands::Delegations(DelegatorDelegationsCommand {
                delegator_address,
                pagination,
            }) => StakingQuery::Delegations(QueryDelegatorDelegationsRequest {
                delegator_addr: delegator_address.clone(),
                pagination: pagination.to_owned().try_map(PaginationRequest::try_from)?,
            }),
            StakingCommands::UnbondingDelegation(UnbondingDelegationCommand {
                delegator_address,
                validator_address,
            }) => StakingQuery::UnbondingDelegation(QueryDelegationRequest {
                delegator_addr: delegator_address.clone(),
                validator_addr: validator_address.clone(),
            }),
            StakingCommands::UnbondingDelegations(UnbondingDelegationsCommand {
                delegator_address,
                pagination,
            }) => StakingQuery::UnbondingDelegations(QueryDelegatorUnbondingDelegationsRequest {
                delegator_addr: delegator_address.clone(),
                pagination: pagination.to_owned().try_map(PaginationRequest::try_from)?,
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
            StakingCommands::Validators(_) => {
                StakingQueryResponse::Validators(QueryValidatorsResponse::decode_vec(&query_bytes)?)
            }
            StakingCommands::Delegation(_) => {
                StakingQueryResponse::Delegation(QueryDelegationResponse::decode_vec(&query_bytes)?)
            }
            StakingCommands::Delegations(_) => StakingQueryResponse::Delegations(
                QueryDelegatorDelegationsResponse::decode_vec(&query_bytes)?,
            ),
            StakingCommands::UnbondingDelegation(_) => StakingQueryResponse::UnbondingDelegation(
                QueryUnbondingDelegationResponse::decode_vec(&query_bytes)?,
            ),
            StakingCommands::UnbondingDelegations(_) => StakingQueryResponse::UnbondingDelegations(
                QueryDelegatorUnbondingDelegationsResponse::decode_vec(&query_bytes)?,
            ),
            StakingCommands::Redelegation(_) => StakingQueryResponse::Redelegation(
                QueryRedelegationResponse::decode_vec(&query_bytes)?,
            ),
        };

        Ok(res)
    }
}

#[derive(Clone, PartialEq, Query)]
#[query(request)]
pub enum StakingQuery {
    Validator(QueryValidatorRequest),
    Validators(QueryValidatorsRequest),
    Delegation(QueryDelegationRequest),
    Delegations(QueryDelegatorDelegationsRequest),
    UnbondingDelegation(QueryDelegationRequest),
    UnbondingDelegations(QueryDelegatorUnbondingDelegationsRequest),
    Redelegation(QueryRedelegationRequest),
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum StakingQueryResponse {
    Validator(QueryValidatorResponse),
    Validators(QueryValidatorsResponse),
    Delegation(QueryDelegationResponse),
    Delegations(QueryDelegatorDelegationsResponse),
    UnbondingDelegation(QueryUnbondingDelegationResponse),
    UnbondingDelegations(QueryDelegatorUnbondingDelegationsResponse),
    Redelegation(QueryRedelegationResponse),
}
