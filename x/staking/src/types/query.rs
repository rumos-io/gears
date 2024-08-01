use crate::{
    consts::error::SERDE_ENCODING_DOMAIN_TYPE, Delegation, Params, Redelegation, RedelegationEntry,
    UnbondingDelegation, Validator,
};
use gears::{
    core::{
        errors::CoreError,
        query::{request::PageRequest, response::PageResponse},
        Protobuf,
    },
    derive::{Protobuf, Query, Raw},
    store::database::ext::UnwrapCorrupt,
    types::{
        address::{AccAddress, ValAddress},
        base::coin::UnsignedCoin,
        errors::DenomError,
        pagination::{request::PaginationRequest, response::PaginationResponse},
        uint::Uint256,
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};

// ===
// requests
// ===

/// QueryValidatorRequest is the request type for the Query/Validator RPC method.
#[derive(Clone, Debug, PartialEq, Query, Raw, Protobuf)]
#[query(url = "/cosmos.staking.v1beta1.Query/Validator")]
pub struct QueryValidatorRequest {
    /// Address of queried validator.
    #[raw(kind(string), raw = String)]
    pub address: ValAddress,
}

/// QueryDelegationRequest is request type for the Query/Delegation RPC method.
#[derive(Clone, Debug, PartialEq, Query, Raw, Protobuf)]
#[query(url = "/cosmos.staking.v1beta1.Query/Delegation")]
pub struct QueryDelegationRequest {
    /// delegator_addr defines the delegator address to query for.
    #[raw(kind(string), raw = String)]
    pub delegator_address: AccAddress,
    /// validator_addr defines the validator address to query for.
    #[raw(kind(string), raw = String)]
    pub validator_address: ValAddress,
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

// ===
// responses
// ===

/// QueryValidatorResponse is the response type for the Query/Validator RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Raw, Protobuf)]

pub struct QueryValidatorResponse {
    /// Full data about validator.
    #[raw(kind(bytes), raw = Vec::<u8>, optional )]
    #[proto(optional)]
    pub validator: Option<Validator>,
}

/// DelegationResponse is equivalent to Delegation except that it contains a
/// balance in addition to shares which is more suitable for client responses.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Raw, Protobuf)]
pub struct DelegationResponse {
    #[raw(kind(bytes), raw = Vec::<u8>)]
    pub delegation: Delegation,
    #[raw(kind(bytes), raw = Vec::<u8>)]
    pub balance: UnsignedCoin,
}

/// QueryDelegationResponse is the response type for the Query/Delegation RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Raw, Protobuf)]
pub struct QueryDelegationResponse {
    /// Delegation with balance.
    #[raw(kind(message), raw = RawDelegationResponse, optional)]
    #[proto(optional)]
    pub delegation_response: Option<DelegationResponse>,
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
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Raw)]
pub struct QueryUnbondingDelegationResponse {
    /// UnbondingDelegation with balance.
    #[raw(kind(bytes), raw = Vec::<u8>, optional)]
    pub unbond: Option<UnbondingDelegation>,
}

impl TryFrom<RawQueryUnbondingDelegationResponse> for QueryUnbondingDelegationResponse {
    type Error = CoreError;

    fn try_from(raw: RawQueryUnbondingDelegationResponse) -> Result<Self, Self::Error> {
        if let Some(ubd) = raw.unbond {
            Ok(QueryUnbondingDelegationResponse {
                unbond: Some(serde_json::from_slice(&ubd).unwrap_or_corrupt()),
            })
        } else {
            Ok(QueryUnbondingDelegationResponse { unbond: None })
        }
    }
}

impl From<QueryUnbondingDelegationResponse> for RawQueryUnbondingDelegationResponse {
    fn from(query: QueryUnbondingDelegationResponse) -> Self {
        Self {
            unbond: query
                .unbond
                .map(|ubd| serde_json::to_vec(&ubd).expect(SERDE_ENCODING_DOMAIN_TYPE)),
        }
    }
}

impl Protobuf<RawQueryUnbondingDelegationResponse> for QueryUnbondingDelegationResponse {}

/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query)]
pub struct QueryParamsResponse {
    pub params: Params,
}

impl TryFrom<QueryParamsResponseRaw> for QueryParamsResponse {
    type Error = CoreError;

    fn try_from(raw: QueryParamsResponseRaw) -> Result<Self, Self::Error> {
        let params = Params::new(
            raw.unbonding_time,
            raw.max_validators,
            raw.max_entries,
            raw.historical_entries,
            raw.bond_denom
                .try_into()
                .map_err(|e: DenomError| CoreError::Custom(e.to_string()))?,
        )
        .map_err(|e| CoreError::Custom(e.to_string()))?;
        Ok(QueryParamsResponse { params })
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct QueryParamsResponseRaw {
    #[prost(int64)]
    pub unbonding_time: i64,
    #[prost(uint32)]
    pub max_validators: u32,
    #[prost(uint32)]
    pub max_entries: u32,
    #[prost(uint32)]
    pub historical_entries: u32,
    #[prost(string)]
    pub bond_denom: String,
}

impl From<QueryParamsResponse> for QueryParamsResponseRaw {
    fn from(query: QueryParamsResponse) -> Self {
        Self {
            unbonding_time: query.params.unbonding_time(),
            max_validators: query.params.max_validators(),
            max_entries: query.params.max_entries(),
            historical_entries: query.params.historical_entries(),
            bond_denom: query.params.bond_denom().to_string(),
        }
    }
}

impl Protobuf<QueryParamsResponseRaw> for QueryParamsResponse {}
