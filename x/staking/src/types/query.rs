use gears::{core::errors::Error, tendermint::types::proto::Protobuf, types::address::ValAddress};
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{consts::expect::SERDE_ENCODING_DOMAIN_TYPE, Validator};

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

// ===
// responses
// ===

/// QueryValidatorResponseRaw is the response type for the Query/Validator RPC method.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct QueryValidatorResponse {
    /// Full data about validator.
    pub validator: Validator,
}

impl TryFrom<QueryValidatorResponseRaw> for QueryValidatorResponse {
    type Error = Error;

    fn try_from(raw: QueryValidatorResponseRaw) -> Result<Self, Self::Error> {
        let validator: Validator = serde_json::from_slice(&raw.validator)
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
            // TODO: consider implement Protobuf for Validator
            validator: serde_json::to_vec(&query.validator).expect(SERDE_ENCODING_DOMAIN_TYPE),
        }
    }
}

impl Protobuf<QueryValidatorResponseRaw> for QueryValidatorResponse {}
