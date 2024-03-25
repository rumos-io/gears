use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::proto::v1::QueryClientParamsResponse as RawQueryClientParamsResponse;
use ibc_proto::Protobuf;
use serde::{Deserialize, Serialize};

use crate::any::Any;
use crate::cosmos::bank::v1beta1::PageResponse;
use crate::cosmos::ibc::types::core::client::{
    proto::IdentifiedClientState,
    types::{ConsensusStateWithHeight, Height, Params},
};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
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

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryClientStateResponse {
    pub client_state: Any,
    pub proof: Vec<u8>,
    pub proof_height: Option<Height>,
}

impl Protobuf<RawQueryClientStateResponse> for QueryClientStateResponse {}

impl TryFrom<RawQueryClientStateResponse> for QueryClientStateResponse {
    type Error = ClientError;

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
            client_state: client_state
                .map(Any::from)
                .ok_or(ClientError::MissingRawClientState)?,
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
            client_state: Some(client_state.into()),
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

impl From<QueryClientStatusResponse> for RawQueryClientStatusResponse {
    fn from(value: QueryClientStatusResponse) -> Self {
        let QueryClientStatusResponse { status } = value;

        Self { status }
    }
}

impl From<RawQueryClientStatusResponse> for QueryClientStatusResponse {
    fn from(value: RawQueryClientStatusResponse) -> Self {
        let RawQueryClientStatusResponse { status } = value;

        Self { status }
    }
}

pub use ibc::core::client::context::types::proto::v1::QueryConsensusStateHeightsResponse as RawQueryConsensusStateHeightsResponse;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct QueryConsensusStateHeightsResponse {
    pub consensus_state_heights: Vec<Height>,
    pub pagination: Option<PageResponse>,
}

impl Protobuf<RawQueryConsensusStateHeightsResponse> for QueryConsensusStateHeightsResponse {}

impl TryFrom<RawQueryConsensusStateHeightsResponse> for QueryConsensusStateHeightsResponse {
    type Error = ClientError;

    fn try_from(value: RawQueryConsensusStateHeightsResponse) -> Result<Self, Self::Error> {
        let RawQueryConsensusStateHeightsResponse {
            consensus_state_heights,
            pagination: _,
        } = value;

        let mut heights = Vec::with_capacity(consensus_state_heights.capacity());
        for height in consensus_state_heights {
            heights.push(Height::try_from(height)?);
        }

        Ok(Self {
            consensus_state_heights: heights,
            pagination: None,
        })
    }
}

impl From<QueryConsensusStateHeightsResponse> for RawQueryConsensusStateHeightsResponse {
    fn from(value: QueryConsensusStateHeightsResponse) -> Self {
        let QueryConsensusStateHeightsResponse {
            consensus_state_heights,
            pagination: _,
        } = value;

        Self {
            consensus_state_heights: consensus_state_heights
                .into_iter()
                .map(ibc::core::client::types::proto::v1::Height::from)
                .collect(),
            pagination: None,
        }
    }
}

pub use ibc::core::client::context::types::proto::v1::QueryConsensusStateResponse as RawQueryConsensusStateResponse;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct QueryConsensusStateResponse {
    pub consensus_state: Option<Any>,
    pub proof: Vec<u8>,
    pub proof_height: Option<Height>,
}

impl Protobuf<RawQueryConsensusStateResponse> for QueryConsensusStateResponse {}

impl TryFrom<RawQueryConsensusStateResponse> for QueryConsensusStateResponse {
    type Error = ClientError;

    fn try_from(value: RawQueryConsensusStateResponse) -> Result<Self, Self::Error> {
        let RawQueryConsensusStateResponse {
            consensus_state,
            proof,
            proof_height,
        } = value;

        let proof_height = if let Some(var) = proof_height {
            Some(var.try_into()?)
        } else {
            None
        };

        Ok(Self {
            consensus_state: consensus_state.map(Any::from),
            proof,
            proof_height,
        })
    }
}

impl From<QueryConsensusStateResponse> for RawQueryConsensusStateResponse {
    fn from(value: QueryConsensusStateResponse) -> Self {
        let QueryConsensusStateResponse {
            consensus_state,
            proof,
            proof_height,
        } = value;

        Self {
            consensus_state: consensus_state.map(Any::into),
            proof,
            proof_height: proof_height.map(Height::into),
        }
    }
}

pub use ibc::core::client::types::proto::v1::QueryConsensusStatesResponse as RawQueryConsensusStatesResponse;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct QueryConsensusStatesResponse {
    pub consensus_states: Vec<ConsensusStateWithHeight>,
    pub pagination: Option<PageResponse>,
}

impl Protobuf<RawQueryConsensusStatesResponse> for QueryConsensusStatesResponse {}

impl TryFrom<RawQueryConsensusStatesResponse> for QueryConsensusStatesResponse {
    type Error = ClientError;

    fn try_from(value: RawQueryConsensusStatesResponse) -> Result<Self, Self::Error> {
        let RawQueryConsensusStatesResponse {
            consensus_states,
            pagination: _,
        } = value;

        let mut states = Vec::with_capacity(consensus_states.len());
        for state in consensus_states {
            states.push(state.try_into()?);
        }

        Ok(Self {
            consensus_states: states,
            pagination: None,
        })
    }
}

impl From<QueryConsensusStatesResponse> for RawQueryConsensusStatesResponse {
    fn from(value: QueryConsensusStatesResponse) -> Self {
        let QueryConsensusStatesResponse {
            consensus_states,
            pagination: _,
        } = value;

        Self {
            consensus_states: consensus_states
                .into_iter()
                .map(ConsensusStateWithHeight::into)
                .collect(),
            pagination: None,
        }
    }
}
