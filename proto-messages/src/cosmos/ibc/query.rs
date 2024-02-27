use ibc::core::client::types::{
    proto::v1::QueryClientParamsResponse as RawQueryClientParamsResponse, Height,
};
pub use ibc_proto::cosmos::base::query::v1beta1::PageResponse;
use ibc_proto::Protobuf;
use serde::{Deserialize, Serialize};

use super::types::core::client::types::Params;
use crate::any::Any;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryClientParamsResponse {
    pub params: Params,
}

impl Protobuf<RawQueryClientParamsResponse> for QueryClientParamsResponse {}

impl TryFrom<RawQueryClientParamsResponse> for QueryClientParamsResponse {
    type Error = std::convert::Infallible;

    fn try_from(value: RawQueryClientParamsResponse) -> Result<Self, Self::Error> {
        let params = value.params.unwrap_or_default().try_into()?;

        Ok(Self { params })
    }
}

impl From<QueryClientParamsResponse> for RawQueryClientParamsResponse {
    fn from(value: QueryClientParamsResponse) -> Self {
        let QueryClientParamsResponse { params } = value;

        Self {
            params: Some(params.into()),
        }
    }
}

pub use ibc::core::client::types::proto::v1::QueryClientStateResponse as RawQueryClientStateResponse;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryClientStateResponse {
    pub client_state: Option<Any>,
    pub proof: Vec<u8>,
    pub proof_height: Option<Height>,
}

impl Protobuf<RawQueryClientStateResponse> for QueryClientStateResponse {}

impl TryFrom<RawQueryClientStateResponse> for QueryClientStateResponse {
    type Error = ibc::core::client::types::error::ClientError;

    fn try_from(value: RawQueryClientStateResponse) -> Result<Self, Self::Error> {
        let RawQueryClientStateResponse {
            client_state,
            proof,
            proof_height,
        } = value;

        let height = if let Some(var) = proof_height {
            Some(var.try_into()?)
        } else {
            None
        };

        Ok(Self {
            client_state: client_state.map(Any::from),
            proof,
            proof_height: height,
        })
    }
}

impl From<QueryClientStateResponse> for RawQueryClientStateResponse {
    fn from(value: QueryClientStateResponse) -> Self {
        let QueryClientStateResponse {
            client_state,
            proof,
            proof_height,
        } = value;

        Self {
            client_state: client_state.map(|e| e.into()),
            proof,
            proof_height: proof_height.map(|e| e.into()),
        }
    }
}
