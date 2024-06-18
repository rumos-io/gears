use crate::{
    consts::error::SERDE_ENCODING_DOMAIN_TYPE, Delegation, Redelegation, RedelegationEntry,
    Validator,
};
use gears::{
    core::{errors::CoreError, Protobuf},
    error::IBC_ENCODE_UNWRAP,
    tendermint::types::proto::Protobuf as TendermintProtobuf,
    types::{
        address::{AccAddress, ValAddress},
        base::coin::Coin,
        response::PageResponse,
        uint::Uint256,
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};

// ===
// requests
// ===

/// QueryValidatorRequest is the request type for the Query/Validator RPC method.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryValidatorRequest {
    /// Address of queried validator.
    pub address: ValAddress,
}

impl TryFrom<QueryValidatorRequestRaw> for QueryValidatorRequest {
    type Error = CoreError;

    fn try_from(raw: QueryValidatorRequestRaw) -> Result<Self, Self::Error> {
        let address = ValAddress::from_bech32(&raw.address)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;

        Ok(QueryValidatorRequest { address })
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct QueryValidatorRequestRaw {
    #[prost(string)]
    pub address: String,
}

impl From<QueryValidatorRequest> for QueryValidatorRequestRaw {
    fn from(query: QueryValidatorRequest) -> QueryValidatorRequestRaw {
        Self {
            address: query.address.to_string(),
        }
    }
}

impl Protobuf<QueryValidatorRequestRaw> for QueryValidatorRequest {}

/// QueryDelegationRequest is request type for the Query/Delegation RPC method.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryDelegationRequest {
    /// delegator_addr defines the delegator address to query for.
    pub delegator_address: AccAddress,
    /// validator_addr defines the validator address to query for.
    pub validator_address: ValAddress,
}

impl TryFrom<QueryDelegationRequestRaw> for QueryDelegationRequest {
    type Error = CoreError;

    fn try_from(raw: QueryDelegationRequestRaw) -> Result<Self, Self::Error> {
        let delegator_address = AccAddress::from_bech32(&raw.delegator_address)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        let validator_address = ValAddress::from_bech32(&raw.validator_address)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;

        Ok(QueryDelegationRequest {
            delegator_address,
            validator_address,
        })
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct QueryDelegationRequestRaw {
    #[prost(string)]
    pub delegator_address: String,
    #[prost(string)]
    pub validator_address: String,
}

impl From<QueryDelegationRequest> for QueryDelegationRequestRaw {
    fn from(query: QueryDelegationRequest) -> QueryDelegationRequestRaw {
        Self {
            delegator_address: query.delegator_address.to_string(),
            validator_address: query.validator_address.to_string(),
        }
    }
}

impl Protobuf<QueryDelegationRequestRaw> for QueryDelegationRequest {}

/// QueryRedelegationRequest is request type for the Query/Redelegation RPC method.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryRedelegationRequest {
    /// delegator_addr defines the delegator address to query for.
    pub delegator_address: Option<AccAddress>,
    /// src_validator_addr defines the validator address to redelegate from.
    pub src_validator_address: Option<ValAddress>,
    /// dst_validator_addr defines the validator address to redelegate to.
    pub dst_validator_address: Option<ValAddress>,
    /// pagination defines an optional pagination for the request.
    pub pagination: Option<PageResponse>,
}

impl TryFrom<QueryRedelegationRequestRaw> for QueryRedelegationRequest {
    type Error = CoreError;

    fn try_from(raw: QueryRedelegationRequestRaw) -> Result<Self, Self::Error> {
        let delegator_address = if let Some(addr) = raw.delegator_address {
            Some(
                AccAddress::from_bech32(&addr)
                    .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            )
        } else {
            None
        };
        let src_validator_address = if let Some(addr) = raw.src_validator_address {
            Some(
                ValAddress::from_bech32(&addr)
                    .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            )
        } else {
            None
        };
        let dst_validator_address = if let Some(addr) = raw.dst_validator_address {
            Some(
                ValAddress::from_bech32(&addr)
                    .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            )
        } else {
            None
        };

        Ok(QueryRedelegationRequest {
            delegator_address,
            src_validator_address,
            dst_validator_address,
            pagination: raw.pagination.map(|p| p.into()),
        })
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct QueryRedelegationRequestRaw {
    #[prost(string, optional)]
    pub delegator_address: Option<String>,
    #[prost(string, optional)]
    pub src_validator_address: Option<String>,
    #[prost(string, optional)]
    pub dst_validator_address: Option<String>,
    #[prost(message, optional)]
    pub pagination: Option<gears::core::query::response::PageResponse>,
}

impl From<QueryRedelegationRequest> for QueryRedelegationRequestRaw {
    fn from(query: QueryRedelegationRequest) -> QueryRedelegationRequestRaw {
        Self {
            delegator_address: query.delegator_address.map(|a| a.to_string()),
            src_validator_address: query.src_validator_address.map(|a| a.to_string()),
            dst_validator_address: query.dst_validator_address.map(|a| a.to_string()),
            pagination: query.pagination.map(|p| p.into()),
        }
    }
}

impl Protobuf<QueryRedelegationRequestRaw> for QueryRedelegationRequest {}

// ===
// responses
// ===

/// QueryValidatorResponse is the response type for the Query/Validator RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct QueryValidatorResponse {
    /// Full data about validator.
    pub validator: Option<Validator>,
}

impl TryFrom<QueryValidatorResponseRaw> for QueryValidatorResponse {
    type Error = CoreError;

    fn try_from(raw: QueryValidatorResponseRaw) -> Result<Self, Self::Error> {
        if let Some(bytes) = raw.validator {
            let validator: Validator = Validator::decode_vec(&bytes)
                .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?;

            Ok(QueryValidatorResponse {
                validator: Some(validator),
            })
        } else {
            Ok(QueryValidatorResponse { validator: None })
        }
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct QueryValidatorResponseRaw {
    #[prost(bytes, optional)]
    pub validator: Option<Vec<u8>>,
}

impl From<QueryValidatorResponse> for QueryValidatorResponseRaw {
    fn from(query: QueryValidatorResponse) -> QueryValidatorResponseRaw {
        Self {
            validator: query.validator.map(|v| v.encode_vec()),
        }
    }
}

impl Protobuf<QueryValidatorResponseRaw> for QueryValidatorResponse {}

/// DelegationResponse is equivalent to Delegation except that it contains a
/// balance in addition to shares which is more suitable for client responses.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DelegationResponse {
    pub delegation: Delegation,
    pub balance: Coin,
}

impl TryFrom<DelegationResponseRaw> for DelegationResponse {
    type Error = CoreError;

    fn try_from(raw: DelegationResponseRaw) -> Result<Self, Self::Error> {
        let delegation: Delegation = serde_json::from_slice(&raw.delegation)
            .map_err(|e| CoreError::DecodeGeneral(e.to_string()))?;
        let balance =
            Coin::decode_vec(&raw.balance).map_err(|e| CoreError::DecodeProtobuf(e.to_string()))?;

        Ok(DelegationResponse {
            delegation,
            balance,
        })
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct DelegationResponseRaw {
    #[prost(bytes)]
    pub delegation: Vec<u8>,
    #[prost(bytes)]
    pub balance: Vec<u8>,
}

impl From<DelegationResponse> for DelegationResponseRaw {
    fn from(query: DelegationResponse) -> DelegationResponseRaw {
        Self {
            delegation: serde_json::to_vec(&query.delegation).expect(SERDE_ENCODING_DOMAIN_TYPE),
            balance: query.balance.encode_vec().expect(IBC_ENCODE_UNWRAP),
        }
    }
}

impl Protobuf<DelegationResponseRaw> for DelegationResponse {}

/// QueryDelegationResponse is the response type for the Query/Delegation RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct QueryDelegationResponse {
    /// Delegation with balance.
    pub delegation_response: Option<DelegationResponse>,
}

impl TryFrom<QueryDelegationResponseRaw> for QueryDelegationResponse {
    type Error = CoreError;

    fn try_from(raw: QueryDelegationResponseRaw) -> Result<Self, Self::Error> {
        if let Some(delegation_response) = raw.delegation_response {
            Ok(QueryDelegationResponse {
                delegation_response: Some(delegation_response.try_into()?),
            })
        } else {
            Ok(QueryDelegationResponse {
                delegation_response: None,
            })
        }
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct QueryDelegationResponseRaw {
    #[prost(message, optional)]
    pub delegation_response: Option<DelegationResponseRaw>,
}

impl From<QueryDelegationResponse> for QueryDelegationResponseRaw {
    fn from(query: QueryDelegationResponse) -> Self {
        Self {
            delegation_response: query.delegation_response.map(Into::into),
        }
    }
}

impl Protobuf<QueryDelegationResponseRaw> for QueryDelegationResponse {}

/// RedelegationEntryResponse is equivalent to a RedelegationEntry except that it
/// contains a balance in addition to shares which is more suitable for client
/// responses.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RedelegationEntryResponse {
    pub redelegation_entry: RedelegationEntry,
    pub balance: Uint256,
}

impl TryFrom<RedelegationEntryResponseRaw> for RedelegationEntryResponse {
    type Error = CoreError;

    fn try_from(raw: RedelegationEntryResponseRaw) -> Result<Self, Self::Error> {
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

#[derive(Clone, PartialEq, Message)]
pub struct RedelegationEntryResponseRaw {
    #[prost(bytes)]
    pub redelegation_entry: Vec<u8>,
    #[prost(bytes)]
    pub balance: Vec<u8>,
}

impl From<RedelegationEntryResponse> for RedelegationEntryResponseRaw {
    fn from(query: RedelegationEntryResponse) -> RedelegationEntryResponseRaw {
        Self {
            redelegation_entry: serde_json::to_vec(&query.redelegation_entry)
                .expect(SERDE_ENCODING_DOMAIN_TYPE),
            balance: serde_json::to_vec(&query.balance).expect(SERDE_ENCODING_DOMAIN_TYPE),
        }
    }
}

/// RedelegationResponse is equivalent to a Redelegation except that its entries
/// contain a balance in addition to shares which is more suitable for client responses.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RedelegationResponse {
    pub redelegation: Redelegation,
    pub entries: Vec<RedelegationEntryResponse>,
}

impl TryFrom<RedelegationResponseRaw> for RedelegationResponse {
    type Error = CoreError;

    fn try_from(raw: RedelegationResponseRaw) -> Result<Self, Self::Error> {
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

#[derive(Clone, PartialEq, Message)]
pub struct RedelegationResponseRaw {
    #[prost(bytes)]
    pub redelegation: Vec<u8>,
    #[prost(bytes)]
    pub entries: Vec<u8>,
}

impl From<RedelegationResponse> for RedelegationResponseRaw {
    fn from(query: RedelegationResponse) -> RedelegationResponseRaw {
        Self {
            redelegation: serde_json::to_vec(&query.redelegation)
                .expect(SERDE_ENCODING_DOMAIN_TYPE),
            entries: serde_json::to_vec(&query.entries).expect(SERDE_ENCODING_DOMAIN_TYPE),
        }
    }
}

/// QueryRedelegationResponse is the response type for the Query/Redelegation RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct QueryRedelegationResponse {
    /// Redelegation with balance.
    pub redelegation_responses: Vec<RedelegationResponse>,
    pub pagination: Option<PageResponse>,
}

impl TryFrom<QueryRedelegationResponseRaw> for QueryRedelegationResponse {
    type Error = CoreError;

    fn try_from(raw: QueryRedelegationResponseRaw) -> Result<Self, Self::Error> {
        let redelegation_responses: Vec<RedelegationResponse> = {
            let mut redelegations = Vec::with_capacity(raw.redelegation_responses.len());
            for red in raw.redelegation_responses {
                redelegations.push(red.try_into()?)
            }
            redelegations
        };
        Ok(QueryRedelegationResponse {
            redelegation_responses,
            pagination: raw.pagination.map(|p| p.into()),
        })
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct QueryRedelegationResponseRaw {
    #[prost(message, repeated)]
    pub redelegation_responses: Vec<RedelegationResponseRaw>,
    #[prost(message, optional)]
    pub pagination: Option<gears::core::query::response::PageResponse>,
}

impl From<QueryRedelegationResponse> for QueryRedelegationResponseRaw {
    fn from(query: QueryRedelegationResponse) -> Self {
        Self {
            redelegation_responses: query
                .redelegation_responses
                .into_iter()
                .map(|red| red.into())
                .collect(),
            pagination: query.pagination.map(|p| p.into()),
        }
    }
}

impl Protobuf<QueryRedelegationResponseRaw> for QueryRedelegationResponse {}
