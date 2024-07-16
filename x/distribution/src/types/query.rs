use gears::core::{errors::CoreError, Protobuf};
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{DistributionParams, DistributionParamsRaw};

#[derive(Clone, PartialEq, Message)]
pub struct QueryParamsRequest {}
impl Protobuf<QueryParamsRequest> for QueryParamsRequest {}

// ====
// responses
// ====

/// QueryParamsResponse is the response type for the Query/Params RPC method
#[derive(Clone, Serialize, Message)]
pub struct QueryParamsResponseRaw {
    #[prost(message, optional)]
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
