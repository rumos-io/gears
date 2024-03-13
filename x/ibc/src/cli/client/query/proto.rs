use super::{IbcQueryResponse, RawIbcQueryResponse};
use prost::encoding::{DecodeContext, WireType};
use proto_messages::cosmos::ibc::{protobuf::Protobuf, types::core::client::types::HeightError};

impl Default for RawIbcQueryResponse {
    fn default() -> Self {
        Self::ClientParams(Default::default())
    }
}

impl Protobuf<RawIbcQueryResponse> for IbcQueryResponse {}

impl From<IbcQueryResponse> for RawIbcQueryResponse {
    fn from(value: IbcQueryResponse) -> Self {
        match value {
            IbcQueryResponse::ClientParams(var) => Self::ClientParams(var.into()),
            IbcQueryResponse::ClientState(var) => Self::ClientState(var.into()),
            IbcQueryResponse::ClientStates(var) => Self::ClientStates(var.into()),
            IbcQueryResponse::ClientStatus(var) => Self::ClientStatus(var.into()),
            IbcQueryResponse::ConsensusState(var) => Self::ConsensusState(var.into()),
            IbcQueryResponse::ConsensusStates(var) => Self::ConsensusStates(var.into()),
            IbcQueryResponse::ConsensusStateHeights(var) => Self::ConsensusStateHeights(var.into()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IbcProtoError {
    #[error("{0}")]
    Infallible(#[from] std::convert::Infallible),
    #[error("{0}")]
    HeightError(#[from] HeightError),
}

impl TryFrom<RawIbcQueryResponse> for IbcQueryResponse {
    type Error = IbcProtoError;

    fn try_from(value: RawIbcQueryResponse) -> Result<Self, Self::Error> {
        let res = match value {
            RawIbcQueryResponse::ClientParams(var) => Self::ClientParams(var.try_into()?),
            RawIbcQueryResponse::ClientState(var) => Self::ClientState(var.try_into()?),
            RawIbcQueryResponse::ClientStates(var) => Self::ClientStates(var.try_into()?),
            RawIbcQueryResponse::ClientStatus(var) => Self::ClientStatus(var.try_into()?),
            RawIbcQueryResponse::ConsensusState(var) => Self::ConsensusState(var.try_into()?),
            RawIbcQueryResponse::ConsensusStates(var) => Self::ConsensusStates(var.try_into()?),
            RawIbcQueryResponse::ConsensusStateHeights(var) => {
                Self::ConsensusStateHeights(var.try_into()?)
            }
        };

        Ok(res)
    }
}

impl prost::Message for RawIbcQueryResponse {
    fn encode_raw<B>(&self, buf: &mut B)
    where
        B: prost::bytes::BufMut,
        Self: Sized,
    {
        match self {
            RawIbcQueryResponse::ClientParams(var) => var.encode_raw(buf),
            RawIbcQueryResponse::ClientState(var) => var.encode_raw(buf),
            RawIbcQueryResponse::ClientStates(var) => var.encode_raw(buf),
            RawIbcQueryResponse::ClientStatus(var) => var.encode_raw(buf),
            RawIbcQueryResponse::ConsensusState(var) => var.encode_raw(buf),
            RawIbcQueryResponse::ConsensusStates(var) => var.encode_raw(buf),
            RawIbcQueryResponse::ConsensusStateHeights(var) => var.encode_raw(buf),
        }
    }

    fn merge_field<B>(
        &mut self,
        tag: u32,
        wire_type: WireType,
        buf: &mut B,
        ctx: DecodeContext,
    ) -> Result<(), prost::DecodeError>
    where
        B: prost::bytes::Buf,
        Self: Sized,
    {
        match self {
            RawIbcQueryResponse::ClientParams(var) => var.merge_field(tag, wire_type, buf, ctx),
            RawIbcQueryResponse::ClientState(var) => var.merge_field(tag, wire_type, buf, ctx),
            RawIbcQueryResponse::ClientStates(var) => var.merge_field(tag, wire_type, buf, ctx),
            RawIbcQueryResponse::ClientStatus(var) => var.merge_field(tag, wire_type, buf, ctx),
            RawIbcQueryResponse::ConsensusState(var) => var.merge_field(tag, wire_type, buf, ctx),
            RawIbcQueryResponse::ConsensusStates(var) => var.merge_field(tag, wire_type, buf, ctx),
            RawIbcQueryResponse::ConsensusStateHeights(var) => {
                var.merge_field(tag, wire_type, buf, ctx)
            }
        }
    }

    fn encoded_len(&self) -> usize {
        match self {
            RawIbcQueryResponse::ClientParams(var) => var.encoded_len(),
            RawIbcQueryResponse::ClientState(var) => var.encoded_len(),
            RawIbcQueryResponse::ClientStates(var) => var.encoded_len(),
            RawIbcQueryResponse::ClientStatus(var) => var.encoded_len(),
            RawIbcQueryResponse::ConsensusState(var) => var.encoded_len(),
            RawIbcQueryResponse::ConsensusStates(var) => var.encoded_len(),
            RawIbcQueryResponse::ConsensusStateHeights(var) => var.encoded_len(),
        }
    }

    fn clear(&mut self) {
        match self {
            RawIbcQueryResponse::ClientParams(var) => var.clear(),
            RawIbcQueryResponse::ClientState(var) => var.clear(),
            RawIbcQueryResponse::ClientStates(var) => var.clear(),
            RawIbcQueryResponse::ClientStatus(var) => var.clear(),
            RawIbcQueryResponse::ConsensusState(var) => var.clear(),
            RawIbcQueryResponse::ConsensusStates(var) => var.clear(),
            RawIbcQueryResponse::ConsensusStateHeights(var) => var.clear(),
        }
    }
}
