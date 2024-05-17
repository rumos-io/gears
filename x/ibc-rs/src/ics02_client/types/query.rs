use gears::error::AppError;
use ibc::{
    core::{
        client::types::proto::v1::IdentifiedClientState as RawIdentifiedClientState,
        host::types::identifiers::ClientId,
    },
    primitives::proto::Protobuf,
};
use serde::{Deserialize, Serialize};

use super::client_state::ClientState;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryClientStatesResponse {
    pub client_states: Vec<IdentifiedClientState>,
    pub pagination: Option<PageResponse>,
}

impl TryFrom<RawQueryClientStatesResponse> for QueryClientStatesResponse {
    type Error = AppError;

    fn try_from(raw: RawQueryClientStatesResponse) -> Result<Self, Self::Error> {
        let client_states: Result<Vec<IdentifiedClientState>, Self::Error> = raw
            .client_states
            .into_iter()
            .map(|x| x.try_into())
            .collect();

        Ok(QueryClientStatesResponse {
            client_states: client_states?,
            pagination: raw.pagination,
        })
    }
}

impl From<QueryClientStatesResponse> for RawQueryClientStatesResponse {
    fn from(query: QueryClientStatesResponse) -> Self {
        RawQueryClientStatesResponse {
            client_states: query.client_states.into_iter().map(|x| x.into()).collect(),
            pagination: query.pagination, //TODO: copy pagination
        }
    }
}

impl Protobuf<RawQueryClientStatesResponse> for QueryClientStatesResponse {}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct IdentifiedClientState {
    pub client_id: ClientId,
    pub client_state: ClientState,
}

impl PartialEq for IdentifiedClientState {
    fn eq(&self, other: &Self) -> bool {
        self.client_id == other.client_id
    }
}

impl Eq for IdentifiedClientState {}

impl PartialOrd for IdentifiedClientState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.client_id.cmp(&other.client_id))
    }
}

impl Ord for IdentifiedClientState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.client_id.cmp(&other.client_id)
    }
}

impl From<IdentifiedClientState> for RawIdentifiedClientState {
    fn from(value: IdentifiedClientState) -> Self {
        RawIdentifiedClientState {
            client_id: value.client_id.to_string(),
            client_state: Some(value.client_state.into()),
        }
    }
}

impl TryFrom<RawIdentifiedClientState> for IdentifiedClientState {
    type Error = AppError;

    fn try_from(value: RawIdentifiedClientState) -> Result<Self, Self::Error> {
        Ok(IdentifiedClientState {
            client_id: value.client_id.parse().unwrap(), //TODO: unwrap
            client_state: value.client_state.unwrap().try_into().unwrap(), //TODO: unwraps
        })
    }
}

/// We implement this ourselves because the ibc crate doesn't export it. TODO: see if we can get it exported from the IBC crate
/// PageResponse is to be embedded in gRPC response messages where the
/// corresponding request message has used PageRequest.
///
///   message SomeResponse {
///           repeated Bar results = 1;
///           PageResponse page = 2;
///   }
#[derive(::serde::Serialize, ::serde::Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct PageResponse {
    /// next_key is the key to be passed to PageRequest.key to
    /// query the next page most efficiently. It will be empty if
    /// there are no more results.
    #[prost(bytes = "vec", tag = "1")]
    pub next_key: ::prost::alloc::vec::Vec<u8>,
    /// total is total number of results available if PageRequest.count_total
    /// was set, its value is undefined otherwise
    #[prost(uint64, tag = "2")]
    pub total: u64,
}

/// We implement this ourselves because the IBC crate doesn't export PageResponse.
#[derive(Clone, PartialEq, prost::Message)]
pub(crate) struct RawQueryClientStatesResponse {
    /// list of stored ClientStates of the chain.
    #[prost(message, repeated, tag = "1")]
    client_states: Vec<RawIdentifiedClientState>,
    /// pagination response
    #[prost(message, optional, tag = "2")]
    pagination: Option<PageResponse>,
}
