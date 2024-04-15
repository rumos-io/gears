use tendermint::types::proto::Protobuf;

use crate::types::{denom::Denom, tx::metadata::Metadata};

mod inner {
    pub use core_types::bank::Metadata;
    pub use core_types::query::request::bank::QueryDenomMetadataRequest;
}

#[derive(Clone, PartialEq)]
pub struct QueryDenomMetadataRequest {
    /// denom is the coin denom to query metadata for.
    pub denom: Denom,
}

impl TryFrom<inner::QueryDenomMetadataRequest> for QueryDenomMetadataRequest {
    type Error = core_types::errors::Error;

    fn try_from(raw: inner::QueryDenomMetadataRequest) -> Result<Self, Self::Error> {
        let denom = raw
            .denom
            .try_into()
            .map_err(|_| Self::Error::Coin(String::from("invalid denom")))?;

        Ok(QueryDenomMetadataRequest { denom })
    }
}

impl From<QueryDenomMetadataRequest> for inner::QueryDenomMetadataRequest {
    fn from(query: QueryDenomMetadataRequest) -> inner::QueryDenomMetadataRequest {
        Self {
            denom: query.denom.to_string(),
        }
    }
}

impl Protobuf<inner::QueryDenomMetadataRequest> for QueryDenomMetadataRequest {}

/// We use our own version of the QueryDenomMetadataResponse struct because the
/// Metadata struct in ibc_proto has additional fields that were added in SDK
/// v46 (uri and uri_hash).
#[derive(Clone, PartialEq, prost::Message)]
pub struct RawQueryDenomMetadataResponse {
    /// metadata describes and provides all the client information for the requested token.
    #[prost(message, optional, tag = "1")]
    pub metadata: Option<inner::Metadata>,
}

#[derive(Clone)]
pub struct QueryDenomMetadataResponse {
    /// metadata describes and provides all the client information for the requested token.
    pub metadata: Option<Metadata>,
}

impl TryFrom<RawQueryDenomMetadataResponse> for QueryDenomMetadataResponse {
    type Error = core_types::errors::Error;

    fn try_from(raw: RawQueryDenomMetadataResponse) -> Result<Self, Self::Error> {
        let metadata = raw
            .metadata
            .map(Metadata::try_from)
            .transpose()
            .map_err(|_| core_types::errors::Error::Coin(String::from("invalid metadata")))?;

        Ok(QueryDenomMetadataResponse { metadata })
    }
}

impl From<QueryDenomMetadataResponse> for RawQueryDenomMetadataResponse {
    fn from(query: QueryDenomMetadataResponse) -> RawQueryDenomMetadataResponse {
        RawQueryDenomMetadataResponse {
            metadata: query.metadata.map(inner::Metadata::from),
        }
    }
}

impl Protobuf<RawQueryDenomMetadataResponse> for QueryDenomMetadataResponse {}
