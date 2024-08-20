use gears::{
    core::{any::google::Any, errors::CoreError, query::request::PageRequest, Protobuf},
    derive::{Protobuf, Query, Raw},
    tendermint::informal::Hash,
    types::pagination::{request::PaginationRequest, response::PaginationResponse},
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// QueryEvidenceRequest is the request type for the Query/Evidence RPC method.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Raw, Query)]
#[query(url = "/cosmos.evidence.v1beta1.Query/Evidence")]
pub struct QueryEvidenceRequest {
    /// evidence_hash defines the hash of the requested evidence.
    #[raw(raw = String, kind(string))]
    pub evidence_hash: Hash,
}

impl From<QueryEvidenceRequest> for RawQueryEvidenceRequest {
    fn from(QueryEvidenceRequest { evidence_hash }: QueryEvidenceRequest) -> Self {
        RawQueryEvidenceRequest {
            evidence_hash: evidence_hash.to_string(),
        }
    }
}

impl TryFrom<RawQueryEvidenceRequest> for QueryEvidenceRequest {
    type Error = CoreError;

    fn try_from(
        RawQueryEvidenceRequest { evidence_hash }: RawQueryEvidenceRequest,
    ) -> Result<Self, Self::Error> {
        Ok(QueryEvidenceRequest {
            evidence_hash: Hash::from_str(&evidence_hash)
                .map_err(|e| CoreError::DecodeGeneral(format!("{e}")))?,
        })
    }
}

impl Protobuf<RawQueryEvidenceRequest> for QueryEvidenceRequest {}

/// QueryAllEvidenceRequest is the request type for the Query/AllEvidence RPC
/// method.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Query, Raw, Protobuf)]
#[query(url = "/cosmos.evidence.v1beta1.Query/AllEvidence")]
pub struct QueryAllEvidenceRequest {
    /// pagination defines an optional pagination for the request.
    #[proto(optional)]
    #[raw(kind(message), optional, raw = PageRequest)]
    pub pagination: PaginationRequest,
}

// =============

/// QueryEvidenceResponse is the response type for the Query/Evidence RPC
/// method
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Query, Raw, Protobuf)]
pub struct QueryEvidenceResponse {
    /// evidence returns the requested evidence.
    #[proto(optional)]
    #[raw(kind(message), raw = Any, optional)]
    pub evidence: Option<Any>,
}

/// QueryAllEvidenceResponse is the response type for the Query/AllEvidence RPC
/// method
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Query, Raw, Protobuf)]
pub struct QueryAllEvidenceResponse {
    /// evidence returns all evidences.
    #[raw(kind(message), raw = Any, repeated)]
    #[proto(repeated)]
    pub evidence: Vec<Any>,
    #[raw(kind(message), raw = gears::core::query::response::PageResponse, optional)]
    #[proto(optional)]
    pub pagination: Option<PaginationResponse>,
}
