use crate::types::proto::info::Snapshot;

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct RequestOfferSnapshot {
    /// snapshot offered by peers
    #[prost(message, optional, tag = "1")]
    pub snapshot: Option<Snapshot>,
    /// light client-verified app hash for snapshot height
    #[prost(bytes = "bytes", tag = "2")]
    pub app_hash: ::prost::bytes::Bytes,
}

impl From<RequestOfferSnapshot> for super::inner::RequestOfferSnapshot {
    fn from(RequestOfferSnapshot { snapshot, app_hash }: RequestOfferSnapshot) -> Self {
        Self {
            snapshot: snapshot.map(Into::into),
            app_hash,
        }
    }
}

impl From<super::inner::RequestOfferSnapshot> for RequestOfferSnapshot {
    fn from(
        super::inner::RequestOfferSnapshot { snapshot, app_hash }: super::inner::RequestOfferSnapshot,
    ) -> Self {
        Self {
            snapshot: snapshot.map(Into::into),
            app_hash,
        }
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct RequestLoadSnapshotChunk {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    #[prost(uint32, tag = "2")]
    pub format: u32,
    #[prost(uint32, tag = "3")]
    pub chunk: u32,
}

impl From<RequestLoadSnapshotChunk> for super::inner::RequestLoadSnapshotChunk {
    fn from(
        RequestLoadSnapshotChunk {
            height,
            format,
            chunk,
        }: RequestLoadSnapshotChunk,
    ) -> Self {
        Self {
            height,
            format,
            chunk,
        }
    }
}

impl From<super::inner::RequestLoadSnapshotChunk> for RequestLoadSnapshotChunk {
    fn from(
        super::inner::RequestLoadSnapshotChunk {
            height,
            format,
            chunk,
        }: super::inner::RequestLoadSnapshotChunk,
    ) -> Self {
        Self {
            height,
            format,
            chunk,
        }
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct RequestApplySnapshotChunk {
    #[prost(uint32, tag = "1")]
    pub index: u32,
    #[prost(bytes = "bytes", tag = "2")]
    pub chunk: ::prost::bytes::Bytes,
    #[prost(string, tag = "3")]
    pub sender: String,
}

impl From<super::inner::RequestApplySnapshotChunk> for RequestApplySnapshotChunk {
    fn from(
        super::inner::RequestApplySnapshotChunk {
            index,
            chunk,
            sender,
        }: super::inner::RequestApplySnapshotChunk,
    ) -> Self {
        Self {
            index,
            chunk,
            sender,
        }
    }
}

impl From<RequestApplySnapshotChunk> for super::inner::RequestApplySnapshotChunk {
    fn from(
        RequestApplySnapshotChunk {
            index,
            chunk,
            sender,
        }: RequestApplySnapshotChunk,
    ) -> Self {
        Self {
            index,
            chunk,
            sender,
        }
    }
}
