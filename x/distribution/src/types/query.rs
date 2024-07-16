use crate::{
    DistributionParams, DistributionParamsRaw, ValidatorOutstandingRewards,
    ValidatorOutstandingRewardsRaw,
};
use gears::{
    core::{errors::CoreError, Protobuf},
    types::address::{AddressError, ValAddress},
};
use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct QueryValidatorOutstandingRewardsRequestRaw {
    #[prost(bytes, tag = "1")]
    pub validator_address: Vec<u8>,
}

impl From<QueryValidatorOutstandingRewardsRequest> for QueryValidatorOutstandingRewardsRequestRaw {
    fn from(
        QueryValidatorOutstandingRewardsRequest { validator_address }: QueryValidatorOutstandingRewardsRequest,
    ) -> Self {
        Self {
            validator_address: validator_address.into(),
        }
    }
}

/// QueryValidatorOutstandingRewardsRequest is the request type for the
/// Query/ValidatorOutstandingRewards RPC method.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryValidatorOutstandingRewardsRequest {
    /// validator_address defines the validator address to query for.
    pub validator_address: ValAddress,
}

impl TryFrom<QueryValidatorOutstandingRewardsRequestRaw>
    for QueryValidatorOutstandingRewardsRequest
{
    type Error = AddressError;

    fn try_from(
        QueryValidatorOutstandingRewardsRequestRaw { validator_address }: QueryValidatorOutstandingRewardsRequestRaw,
    ) -> Result<Self, Self::Error> {
        Ok(QueryValidatorOutstandingRewardsRequest {
            validator_address: ValAddress::try_from(validator_address)?,
        })
    }
}

impl Protobuf<QueryValidatorOutstandingRewardsRequestRaw>
    for QueryValidatorOutstandingRewardsRequest
{
}

#[derive(Clone, PartialEq, Message)]
pub struct QueryParamsRequest {}

impl Protobuf<QueryParamsRequest> for QueryParamsRequest {}

// ====
// responses
// ====

#[derive(Clone, Serialize, Message)]
pub struct QueryValidatorOutstandingRewardsResponseRaw {
    #[prost(message, optional, tag = "1")]
    pub rewards: Option<ValidatorOutstandingRewardsRaw>,
}

impl From<QueryValidatorOutstandingRewardsResponse>
    for QueryValidatorOutstandingRewardsResponseRaw
{
    fn from(
        QueryValidatorOutstandingRewardsResponse { rewards }: QueryValidatorOutstandingRewardsResponse,
    ) -> Self {
        Self {
            rewards: rewards.map(Into::into),
        }
    }
}

/// QueryValidatorOutstandingRewardsResponse is the response type for the
/// Query/ValidatorOutstandingRewards RPC method.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct QueryValidatorOutstandingRewardsResponse {
    pub rewards: Option<ValidatorOutstandingRewards>,
}

impl TryFrom<QueryValidatorOutstandingRewardsResponseRaw>
    for QueryValidatorOutstandingRewardsResponse
{
    type Error = CoreError;

    fn try_from(
        QueryValidatorOutstandingRewardsResponseRaw { rewards }: QueryValidatorOutstandingRewardsResponseRaw,
    ) -> Result<Self, Self::Error> {
        let rewards = if let Some(rew) = rewards {
            Some(rew.try_into()?)
        } else {
            None
        };
        Ok(Self { rewards })
    }
}

impl Protobuf<QueryValidatorOutstandingRewardsResponseRaw>
    for QueryValidatorOutstandingRewardsResponse
{
}

#[derive(Clone, Serialize, Message)]
pub struct QueryParamsResponseRaw {
    #[prost(message, optional, tag = "1")]
    pub params: Option<DistributionParamsRaw>,
}

impl From<QueryParamsResponse> for QueryParamsResponseRaw {
    fn from(QueryParamsResponse { params }: QueryParamsResponse) -> Self {
        Self {
            params: Some(params.into()),
        }
    }
}

/// QueryParamsResponse is the response type for the Query/Params RPC method
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct QueryParamsResponse {
    pub params: DistributionParams,
}

impl TryFrom<QueryParamsResponseRaw> for QueryParamsResponse {
    type Error = CoreError;

    fn try_from(
        QueryParamsResponseRaw { params }: QueryParamsResponseRaw,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            params: params
                .ok_or(CoreError::MissingField("Missing field 'params'.".into()))?
                .try_into()
                .map_err(|e| CoreError::DecodeGeneral(format!("{e}")))?,
        })
    }
}

impl Protobuf<QueryParamsResponseRaw> for QueryParamsResponse {}
