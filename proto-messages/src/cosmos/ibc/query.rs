use ibc::core::client::types::{
    proto::v1::QueryClientParamsResponse as RawQueryClientParamsResponse, Height,
};
use ibc_proto::Protobuf;
use serde::{Deserialize, Serialize};

use super::types::core::client::proto::IdentifiedClientState;
use super::types::core::client::types::Params;
use crate::any::Any;
use crate::cosmos::bank::v1beta1::PageResponse;

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

pub use ibc::core::client::types::proto::v1::QueryClientStatesResponse as RawQueryClientStatesResponse;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct QueryClientStatesResponse {
    pub client_states: Vec<IdentifiedClientState>,
    pub pagination: Option<PageResponse>,
}

impl Protobuf<RawQueryClientStatesResponse> for QueryClientStatesResponse {}

impl From<RawQueryClientStatesResponse> for QueryClientStatesResponse {
    fn from(value: RawQueryClientStatesResponse) -> Self {
        let RawQueryClientStatesResponse {
            client_states,
            pagination,
        } = value;

        let pagination = if let Some(pagination) = pagination {
            Some(PageResponse {
                next_key: pagination.next_key,
                total: pagination.total,
            })
        } else {
            None
        };

        Self {
            client_states: client_states
                .into_iter()
                .map(IdentifiedClientState::from)
                .collect(),
            pagination,
        }
    }
}

impl From<QueryClientStatesResponse> for RawQueryClientStatesResponse {
    fn from(value: QueryClientStatesResponse) -> Self {
        let QueryClientStatesResponse {
            client_states,
            pagination: _,
        } = value;

        Self {
            client_states: client_states
                .into_iter()
                .map(ibc::core::client::types::proto::v1::IdentifiedClientState::from)
                .collect(),
            pagination: None,
        }
    }
}

pub use ibc::core::client::context::types::proto::v1::QueryClientStatusResponse as RawQueryClientStatusResponse;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct QueryClientStatusResponse {
    pub status: String,
}

impl Protobuf<RawQueryClientStatusResponse> for QueryClientStatusResponse {}

impl From<QueryClientStatusResponse> for RawQueryClientStatusResponse
{
    fn from(value: QueryClientStatusResponse) -> Self {
        let QueryClientStatusResponse { status } = value;

        Self { status }
    }
}

impl From<RawQueryClientStatusResponse> for QueryClientStatusResponse
{
    fn from(value: RawQueryClientStatusResponse) -> Self {
        let RawQueryClientStatusResponse { status } = value;

        Self { status }
    }
}
