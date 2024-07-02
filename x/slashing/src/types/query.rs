use gears::{
    core::{errors::CoreError, Protobuf},
    types::address::{AddressError, ConsAddress},
};
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{ValidatorSigningInfo, ValidatorSigningInfoRaw};

// =====
// Requests
// =====

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct QuerySigningInfoRequestRaw {
    #[prost(bytes)]
    pub cons_address: Vec<u8>,
}

impl From<QuerySigningInfoRequest> for QuerySigningInfoRequestRaw {
    fn from(value: QuerySigningInfoRequest) -> Self {
        Self {
            cons_address: value.cons_address.into(),
        }
    }
}

/// QuerySigningInfoRequest is the request type for the Query/SigningInfo RPC
/// method
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuerySigningInfoRequest {
    /// cons_address is the address to query signing info of
    pub cons_address: ConsAddress,
}

impl TryFrom<QuerySigningInfoRequestRaw> for QuerySigningInfoRequest {
    type Error = AddressError;

    fn try_from(value: QuerySigningInfoRequestRaw) -> Result<Self, Self::Error> {
        Ok(QuerySigningInfoRequest {
            cons_address: ConsAddress::try_from(value.cons_address)?,
        })
    }
}

impl Protobuf<QuerySigningInfoRequestRaw> for QuerySigningInfoRequest {}

// =====
// Responses
// =====

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct QuerySigningInfoResponseRaw {
    #[prost(message, optional)]
    pub val_signing_info: Option<ValidatorSigningInfoRaw>,
}

impl From<QuerySigningInfoResponse> for QuerySigningInfoResponseRaw {
    fn from(value: QuerySigningInfoResponse) -> Self {
        Self {
            val_signing_info: Some(value.val_signing_info.into()),
        }
    }
}

/// QuerySigningInfoResponse is the response type for the Query/SigningInfo RPC
/// method
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuerySigningInfoResponse {
    /// val_signing_info is the signing info of requested val cons address
    pub val_signing_info: ValidatorSigningInfo,
}

impl TryFrom<QuerySigningInfoResponseRaw> for QuerySigningInfoResponse {
    type Error = CoreError;

    fn try_from(value: QuerySigningInfoResponseRaw) -> Result<Self, Self::Error> {
        Ok(QuerySigningInfoResponse {
            val_signing_info: value
                .val_signing_info
                .ok_or(CoreError::MissingField(
                    "Missing field 'val_signing_info'.".into(),
                ))?
                .try_into()?,
        })
    }
}

impl Protobuf<QuerySigningInfoResponseRaw> for QuerySigningInfoResponse {}
