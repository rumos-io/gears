use crate::types::proto::info::Snapshot;

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ResponseListSnapshots {
    #[prost(message, repeated, tag = "1")]
    pub snapshots: Vec<Snapshot>,
}

impl From<ResponseListSnapshots> for super::inner::ResponseListSnapshots {
    fn from(ResponseListSnapshots { snapshots }: ResponseListSnapshots) -> Self {
        Self {
            snapshots: snapshots.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<super::inner::ResponseListSnapshots> for ResponseListSnapshots {
    fn from(
        super::inner::ResponseListSnapshots { snapshots }: super::inner::ResponseListSnapshots,
    ) -> Self {
        Self {
            snapshots: snapshots.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OfferResult {
    /// Unknown result, abort all snapshot restoration
    Unknown = 0,
    /// Snapshot accepted, apply chunks
    Accept = 1,
    /// Abort all snapshot restoration
    Abort = 2,
    /// Reject this specific snapshot, try others
    Reject = 3,
    /// Reject all snapshots of this format, try others
    RejectFormat = 4,
    /// Reject all snapshots from the sender(s), try others
    RejectSender = 5,
}

impl OfferResult {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            OfferResult::Unknown => "UNKNOWN",
            OfferResult::Accept => "ACCEPT",
            OfferResult::Abort => "ABORT",
            OfferResult::Reject => "REJECT",
            OfferResult::RejectFormat => "REJECT_FORMAT",
            OfferResult::RejectSender => "REJECT_SENDER",
        }
    }
}

impl From<OfferResult> for inner::Result {
    fn from(value: OfferResult) -> Self {
        match value {
            OfferResult::Unknown => Self::Unknown,
            OfferResult::Accept => Self::Accept,
            OfferResult::Abort => Self::Abort,
            OfferResult::Reject => Self::Reject,
            OfferResult::RejectFormat => Self::RejectFormat,
            OfferResult::RejectSender => Self::RejectSender,
        }
    }
}

impl From<inner::Result> for OfferResult {
    fn from(value: inner::Result) -> Self {
        match value {
            inner::Result::Unknown => Self::Unknown,
            inner::Result::Accept => Self::Accept,
            inner::Result::Abort => Self::Abort,
            inner::Result::Reject => Self::Reject,
            inner::Result::RejectFormat => Self::RejectFormat,
            inner::Result::RejectSender => Self::RejectSender,
        }
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ResponseOfferSnapshot {
    #[prost(enumeration = "OfferResult", tag = "1")]
    pub result: i32,
}

impl From<super::inner::ResponseOfferSnapshot> for ResponseOfferSnapshot {
    fn from(
        super::inner::ResponseOfferSnapshot { result }: super::inner::ResponseOfferSnapshot,
    ) -> Self {
        Self { result }
    }
}

impl From<ResponseOfferSnapshot> for super::inner::ResponseOfferSnapshot {
    fn from(ResponseOfferSnapshot { result }: ResponseOfferSnapshot) -> Self {
        Self { result }
    }
}
#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]

pub struct ResponseLoadSnapshotChunk {
    #[prost(bytes = "bytes", tag = "1")]
    pub chunk: ::prost::bytes::Bytes,
}

impl From<ResponseLoadSnapshotChunk> for super::inner::ResponseLoadSnapshotChunk {
    fn from(ResponseLoadSnapshotChunk { chunk }: ResponseLoadSnapshotChunk) -> Self {
        Self { chunk }
    }
}

impl From<super::inner::ResponseLoadSnapshotChunk> for ResponseLoadSnapshotChunk {
    fn from(
        super::inner::ResponseLoadSnapshotChunk { chunk }: super::inner::ResponseLoadSnapshotChunk,
    ) -> Self {
        Self { chunk }
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct ResponseApplySnapshotChunk {
    #[prost(enumeration = "OfferResult", tag = "1")]
    pub result: i32,
    /// Chunks to refetch and reapply
    #[prost(uint32, repeated, tag = "2")]
    pub refetch_chunks: Vec<u32>,
    /// Chunk senders to reject and ban
    #[prost(string, repeated, tag = "3")]
    pub reject_senders: Vec<String>,
}

impl From<super::inner::ResponseApplySnapshotChunk> for ResponseApplySnapshotChunk {
    fn from(
        super::inner::ResponseApplySnapshotChunk {
            result,
            refetch_chunks,
            reject_senders,
        }: super::inner::ResponseApplySnapshotChunk,
    ) -> Self {
        Self {
            result,
            refetch_chunks,
            reject_senders,
        }
    }
}

impl From<ResponseApplySnapshotChunk> for super::inner::ResponseApplySnapshotChunk {
    fn from(
        ResponseApplySnapshotChunk {
            result,
            refetch_chunks,
            reject_senders,
        }: ResponseApplySnapshotChunk,
    ) -> Self {
        Self {
            result,
            refetch_chunks,
            reject_senders,
        }
    }
}

pub mod inner {
    pub use tendermint_proto::abci::response_offer_snapshot::Result;
}
