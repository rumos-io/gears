use super::header::PartSetHeader;

/// BlockID
#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct BlockId {
    #[prost(bytes = "vec", tag = "1")]
    #[serde(with = "crate::types::serializers::bytes::hexstring")]
    pub hash: Vec<u8>,
    #[prost(message, optional, tag = "2")]
    #[serde(alias = "parts")]
    pub part_set_header: Option<PartSetHeader>,
}

impl From<BlockId> for inner::BlockId {
    fn from(
        BlockId {
            hash,
            part_set_header,
        }: BlockId,
    ) -> Self {
        Self {
            hash,
            part_set_header: part_set_header.map(Into::into),
        }
    }
}

impl From<inner::BlockId> for BlockId {
    fn from(
        inner::BlockId {
            hash,
            part_set_header,
        }: inner::BlockId,
    ) -> Self {
        Self {
            hash,
            part_set_header: part_set_header.map(Into::into),
        }
    }
}

pub use tendermint_informal::block::Height; // TODO

pub(crate) mod inner {
    pub use tendermint_proto::types::BlockId;
}
