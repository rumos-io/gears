use crate::{
    consts::error::SERDE_ENCODING_DOMAIN_TYPE, Delegation, Pool, Redelegation, RedelegationEntry,
    StakingParams, UnbondingDelegation, Validator,
};
use gears::{
    core::{
        errors::CoreError,
        query::{request::PageRequest, response::PageResponse},
        Protobuf,
    },
    derive::{Protobuf, Query, Raw},
    types::{
        address::{AccAddress, ValAddress},
        base::coin::UnsignedCoin,
        pagination::{request::PaginationRequest, response::PaginationResponse},
        uint::Uint256,
    },
    x::types::validator::BondStatus,
};
use prost::Message;
use serde::{Deserialize, Serialize};

mod inner {
    pub use ibc_proto::cosmos::staking::v1beta1::DelegationResponse;
    pub use ibc_proto::cosmos::staking::v1beta1::QueryPoolResponse;
    pub use ibc_proto::cosmos::staking::v1beta1::QueryUnbondingDelegationResponse;
    pub use ibc_proto::cosmos::staking::v1beta1::{
        QueryDelegationRequest, QueryDelegationResponse,
    };
    pub use ibc_proto::cosmos::staking::v1beta1::{
        QueryDelegatorDelegationsRequest, QueryDelegatorDelegationsResponse,
    };
    pub use ibc_proto::cosmos::staking::v1beta1::{QueryParamsRequest, QueryParamsResponse};
    pub use ibc_proto::cosmos::staking::v1beta1::{QueryValidatorRequest, QueryValidatorResponse};
    pub use ibc_proto::cosmos::staking::v1beta1::{
        QueryValidatorsRequest, QueryValidatorsResponse,
    };
}

// ===
// requests
// ===

/// QueryValidatorRequest is the request type for the Query/Validator RPC method.
#[derive(Clone, Debug, PartialEq, Query, Protobuf)]
#[query(url = "/cosmos.staking.v1beta1.Query/Validator")]
#[proto(raw = "inner::QueryValidatorRequest")]
pub struct QueryValidatorRequest {
    /// Address of queried validator.
    pub validator_addr: ValAddress,
}

/// QueryValidatorsRequest is request type for Query/Validators RPC method.
#[derive(Clone, Debug, PartialEq, Query, Protobuf)]
#[query(url = "/cosmos.staking.v1beta1.Query/Validators")]
#[proto(raw = "inner::QueryValidatorsRequest")]
pub struct QueryValidatorsRequest {
    /// status enables to query for validators matching a given status.
    pub status: BondStatus,
    /// pagination defines an optional pagination for the request.
    #[proto(optional)]
    pub pagination: Option<PaginationRequest>,
}

/// QueryDelegationRequest is request type for the Query/Delegation RPC method.
#[derive(Clone, Debug, PartialEq, Query, Protobuf)]
#[query(url = "/cosmos.staking.v1beta1.Query/Delegation")]
#[proto(raw = "inner::QueryDelegationRequest")]
pub struct QueryDelegationRequest {
    /// delegator_addr defines the delegator address to query for.
    pub delegator_addr: AccAddress,
    /// validator_addr defines the validator address to query for.
    pub validator_addr: ValAddress,
}

/// QueryDelegatorDelegationsRequest is request type for the
/// Query/DelegatorDelegations RPC method.
#[derive(Clone, Debug, PartialEq, Query, Protobuf)]
#[query(url = "/cosmos.staking.v1beta1.Query/Delegations")]
#[proto(raw = "inner::QueryDelegatorDelegationsRequest")]
pub struct QueryDelegatorDelegationsRequest {
    /// delegator_addr defines the delegator address to query for.
    pub delegator_addr: AccAddress,
    /// pagination defines an optional pagination for the request.
    #[proto(optional)]
    pub pagination: Option<PaginationRequest>,
}

#[derive(Clone, Debug, PartialEq, Query, Raw, Protobuf)]
#[query(url = "/cosmos.staking.v1beta1.Query/UnbondingDelegation")]
pub struct QueryUnboundingDelegationRequest {
    /// delegator_addr defines the delegator address to query for.
    #[raw(kind(string), raw = String)]
    pub delegator_address: AccAddress,
    /// validator_addr defines the validator address to query for.
    #[raw(kind(string), raw = String)]
    pub validator_address: ValAddress,
}

/// QueryRedelegationRequest is request type for the Query/Redelegation RPC method.
#[derive(Clone, Debug, PartialEq, Query, Raw, Protobuf)]
#[query(url = "/cosmos.staking.v1beta1.Query/Redelegation")]
pub struct QueryRedelegationRequest {
    /// delegator_addr defines the delegator address to query for.
    #[raw(kind(string), raw = String, optional)]
    #[proto(optional)]
    pub delegator_address: Option<AccAddress>,
    /// src_validator_addr defines the validator address to redelegate from.
    #[raw(kind(string), raw = String, optional)]
    #[proto(optional)]
    pub src_validator_address: Option<ValAddress>,
    /// dst_validator_addr defines the validator address to redelegate to.
    #[raw(kind(string), raw = String, optional)]
    #[proto(optional)]
    pub dst_validator_address: Option<ValAddress>,
    /// pagination defines an optional pagination for the request.
    #[raw(kind(message), raw = PageRequest, optional)]
    #[proto(optional)]
    pub pagination: Option<PaginationRequest>,
}

#[derive(Clone, PartialEq, Message, Raw, Query, Protobuf)]
#[query(url = "/cosmos.staking.v1beta1.Query/Pool")]
pub struct QueryPoolRequest {}

#[derive(Clone, PartialEq, Message, Query, Protobuf)]
#[query(url = "/cosmos.staking.v1beta1.Query/Params")]
#[proto(raw = "inner::QueryParamsRequest")]
pub struct QueryParamsRequest {}

// ===
// responses
// ===

/// QueryValidatorResponse is the response type for the Query/Validator RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "inner::QueryValidatorResponse")]
pub struct QueryValidatorResponse {
    /// Full data about validator.
    #[proto(optional)]
    pub validator: Option<Validator>,
}

/// QueryValidatorsResponse is response type for the Query/Validators RPC method
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "inner::QueryValidatorsResponse")]
pub struct QueryValidatorsResponse {
    /// validators contains all the queried validators.
    #[proto(repeated)]
    pub validators: Vec<Validator>,
    /// pagination defines the pagination in the response.
    #[proto(optional)]
    pub pagination: Option<PaginationResponse>,
}

/// DelegationResponse is equivalent to Delegation except that it contains a
/// balance in addition to shares which is more suitable for client responses.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "inner::DelegationResponse")]
pub struct DelegationResponse {
    #[proto(optional)]
    pub delegation: Option<Delegation>,
    #[proto(optional)]
    pub balance: Option<UnsignedCoin>,
}

/// QueryDelegationResponse is the response type for the Query/Delegation RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "inner::QueryDelegationResponse")]
pub struct QueryDelegationResponse {
    /// Delegation with balance.
    #[proto(optional)]
    pub delegation_response: Option<DelegationResponse>,
}

/// QueryDelegatorDelegationsResponse is response type for the
/// Query/DelegatorDelegations RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "inner::QueryDelegatorDelegationsResponse")]
pub struct QueryDelegatorDelegationsResponse {
    /// delegation_responses defines all the delegations' info of a delegator.
    #[proto(repeated)]
    pub delegation_responses: Vec<DelegationResponse>,
    /// pagination defines the pagination in the response.
    #[proto(optional)]
    pub pagination: Option<PaginationResponse>,
}

/// RedelegationEntryResponse is equivalent to a RedelegationEntry except that it
/// contains a balance in addition to shares which is more suitable for client
/// responses.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Raw)]
pub struct RedelegationEntryResponse {
    #[raw(kind(bytes), raw = Vec::<u8>)]
    pub redelegation_entry: RedelegationEntry,
    #[raw(kind(bytes), raw = Vec::<u8>)]
    pub balance: Uint256,
}

impl TryFrom<RawRedelegationEntryResponse> for RedelegationEntryResponse {
    type Error = CoreError;

    fn try_from(raw: RawRedelegationEntryResponse) -> Result<Self, Self::Error> {
        let redelegation_entry: RedelegationEntry = serde_json::from_slice(&raw.redelegation_entry)
            .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?;
        let balance = serde_json::from_slice(&raw.balance)
            .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))?;

        Ok(RedelegationEntryResponse {
            redelegation_entry,
            balance,
        })
    }
}

impl From<RedelegationEntryResponse> for RawRedelegationEntryResponse {
    fn from(query: RedelegationEntryResponse) -> RawRedelegationEntryResponse {
        Self {
            redelegation_entry: serde_json::to_vec(&query.redelegation_entry)
                .expect(SERDE_ENCODING_DOMAIN_TYPE),
            balance: serde_json::to_vec(&query.balance).expect(SERDE_ENCODING_DOMAIN_TYPE),
        }
    }
}

impl Protobuf<RawRedelegationEntryResponse> for RedelegationEntryResponse {}

/// RedelegationResponse is equivalent to a Redelegation except that its entries
/// contain a balance in addition to shares which is more suitable for client responses.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Raw)]
pub struct RedelegationResponse {
    #[raw(kind(bytes), raw = Vec::<u8>)]
    pub redelegation: Redelegation,
    #[raw(kind(bytes), raw = Vec::<u8>)]
    pub entries: Vec<RedelegationEntryResponse>,
}

impl TryFrom<RawRedelegationResponse> for RedelegationResponse {
    type Error = CoreError;

    fn try_from(raw: RawRedelegationResponse) -> Result<Self, Self::Error> {
        let redelegation: Redelegation = serde_json::from_slice(&raw.redelegation)
            .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?;
        let entries = serde_json::from_slice(&raw.entries)
            .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?;

        Ok(RedelegationResponse {
            redelegation,
            entries,
        })
    }
}

impl From<RedelegationResponse> for RawRedelegationResponse {
    fn from(query: RedelegationResponse) -> RawRedelegationResponse {
        Self {
            redelegation: serde_json::to_vec(&query.redelegation)
                .expect(SERDE_ENCODING_DOMAIN_TYPE),
            entries: serde_json::to_vec(&query.entries).expect(SERDE_ENCODING_DOMAIN_TYPE),
        }
    }
}

impl Protobuf<RawRedelegationResponse> for RedelegationResponse {}

/// QueryRedelegationResponse is the response type for the Query/Redelegation RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Raw, Protobuf)]
pub struct QueryRedelegationResponse {
    /// Redelegation with balance
    #[raw(kind(message), raw = RawRedelegationResponse, repeated)]
    #[proto(repeated)]
    pub redelegation_responses: Vec<RedelegationResponse>,
    #[raw(kind(message), raw = PageResponse, optional)]
    #[proto(optional)]
    pub pagination: Option<PaginationResponse>,
}

/// QueryUnbondingDelegationResponse is the response type for the Query/UnbondingDelegation RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "inner::QueryUnbondingDelegationResponse")]
pub struct QueryUnbondingDelegationResponse {
    /// UnbondingDelegation with balance.
    #[proto(optional)]
    pub unbond: Option<UnbondingDelegation>,
}

/// QueryPoolResponse is response type for the Query/Pool RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "inner::QueryPoolResponse")]
pub struct QueryPoolResponse {
    /// UnbondingDelegation with balance.
    #[proto(optional)]
    pub pool: Option<Pool>,
}

/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "inner::QueryParamsResponse")]
pub struct QueryParamsResponse {
    #[proto(optional)]
    pub params: Option<StakingParams>,
}
