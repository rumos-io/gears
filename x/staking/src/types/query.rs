use crate::{consts::error::SERDE_ENCODING_DOMAIN_TYPE, Delegation, Validator};
use gears::{
    core::{errors::Error, Protobuf},
    error::IBC_ENCODE_UNWRAP,
    tendermint::types::proto::Protobuf as TendermintProtobuf,
    types::{
        address::{AccAddress, ValAddress},
        base::coin::Coin,
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
    type Error = Error;

    fn try_from(raw: QueryValidatorRequestRaw) -> Result<Self, Self::Error> {
        let address = ValAddress::from_bech32(&raw.address)
            .map_err(|e| Error::DecodeAddress(e.to_string()))?;

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
    type Error = Error;

    fn try_from(raw: QueryDelegationRequestRaw) -> Result<Self, Self::Error> {
        let delegator_address = AccAddress::from_bech32(&raw.delegator_address)
            .map_err(|e| Error::DecodeAddress(e.to_string()))?;
        let validator_address = ValAddress::from_bech32(&raw.validator_address)
            .map_err(|e| Error::DecodeAddress(e.to_string()))?;

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

// ===
// responses
// ===

/// QueryValidatorResponse is the response type for the Query/Validator RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct QueryValidatorResponse {
    /// Full data about validator.
    pub validator: Validator,
}

impl TryFrom<QueryValidatorResponseRaw> for QueryValidatorResponse {
    type Error = Error;

    fn try_from(raw: QueryValidatorResponseRaw) -> Result<Self, Self::Error> {
        let validator: Validator = Validator::decode_vec(&raw.validator)
            .map_err(|e| Error::DecodeGeneral(e.to_string()))?;

        Ok(QueryValidatorResponse { validator })
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct QueryValidatorResponseRaw {
    #[prost(bytes)]
    pub validator: Vec<u8>,
}

impl From<QueryValidatorResponse> for QueryValidatorResponseRaw {
    fn from(query: QueryValidatorResponse) -> QueryValidatorResponseRaw {
        Self {
            validator: query.validator.encode_vec(),
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
    type Error = Error;

    fn try_from(raw: DelegationResponseRaw) -> Result<Self, Self::Error> {
        let delegation: Delegation = serde_json::from_slice(&raw.delegation)
            .map_err(|e| Error::DecodeGeneral(e.to_string()))?;
        let balance =
            Coin::decode_vec(&raw.balance).map_err(|e| Error::DecodeProtobuf(e.to_string()))?;

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
            // TODO: consider implement Protobuf for Validator
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
    pub delegation_response: DelegationResponse,
}

impl TryFrom<QueryDelegationResponseRaw> for QueryDelegationResponse {
    type Error = Error;

    fn try_from(raw: QueryDelegationResponseRaw) -> Result<Self, Self::Error> {
        Ok(QueryDelegationResponse {
            delegation_response: raw
                .delegation_response
                .ok_or(Error::MissingField(
                    "Value should exists. It's the proto3 rule to have Option<T> instead of T"
                        .into(),
                ))?
                .try_into()?,
        })
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
            delegation_response: Some(query.delegation_response.into()),
        }
    }
}

impl Protobuf<QueryDelegationResponseRaw> for QueryDelegationResponse {}
