use gears::{
    derive::{Protobuf, Query, Raw},
    types::{
        address::ConsAddress,
        pagination::{request::PaginationRequest, response::PaginationResponse},
    },
};
use ibc_proto::cosmos::base::query::v1beta1::{PageRequest, PageResponse};
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{SlashingParams, SlashingParamsRaw, ValidatorSigningInfo, ValidatorSigningInfoRaw};

// =====
// Requests
// =====

/// QuerySigningInfoRequest is the request type for the Query/SigningInfo RPC
/// method
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Raw, Protobuf, Query)]
#[query(url = "/cosmos.slashing.v1beta1.Query/SigningInfo")]
pub struct QuerySigningInfoRequest {
    /// cons_address is the address to query signing info of
    #[raw(raw = String, kind(string))]
    pub cons_address: ConsAddress,
}

/// QuerySigningInfosRequest is the request type for the Query/SigningInfos RPC
/// method
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Query, Raw, Protobuf)]
#[query(url = "/cosmos.slashing.v1beta1.Query/SigningInfos")]
pub struct QuerySigningInfosRequest {
    /// pagination defines an optional pagination for the request.
    #[proto(optional)]
    #[raw(kind(message), optional, raw = PageRequest)]
    pub pagination: PaginationRequest,
}

#[derive(Clone, PartialEq, Message, Query, Raw, Protobuf)]
#[query(url = "/cosmos.slashing.v1beta1.Query/Params")]
pub struct QueryParamsRequest {}

// =====
// Responses
// =====

/// QuerySigningInfoResponse is the response type for the Query/SigningInfo RPC
/// method
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Query, Raw, Protobuf)]
pub struct QuerySigningInfoResponse {
    /// val_signing_info is the signing info of requested val cons address
    #[proto(optional)]
    #[raw(kind(message), raw = ValidatorSigningInfoRaw, optional)]
    pub val_signing_info: ValidatorSigningInfo,
}

/// QuerySigningInfosResponse is the response type for the Query/SigningInfos RPC
/// method
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Raw, Protobuf)]
pub struct QuerySigningInfosResponse {
    /// Info is the signing info of all validators
    #[raw(kind(message), raw = ValidatorSigningInfoRaw, repeated)]
    #[proto(repeated)]
    pub info: Vec<ValidatorSigningInfo>,
    #[raw(kind(message), raw = PageResponse, optional)]
    #[proto(optional)]
    pub pagination: Option<PaginationResponse>,
}

/// QueryParamsResponse is the response type for the Query/Params RPC method
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Query, Raw, Protobuf)]
pub struct QueryParamsResponse {
    #[proto(optional)]
    #[raw(kind(message), raw = SlashingParamsRaw, optional)]
    pub params: SlashingParams,
}
